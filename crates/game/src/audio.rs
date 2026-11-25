use std::num::{NonZeroU16, NonZeroU32};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{
    AddAudioSource, AudioPlayer, AudioSink, AudioSinkPlayback, ChannelCount, Decodable,
    PlaybackSettings, Sample, SampleRate, Source, Volume,
};
use bevy::prelude::*;
use bevy::reflect::TypePath;

#[derive(Asset, TypePath, Clone)]
pub struct PcmSound {
    pub samples: Arc<[f32]>,
    pub channels: ChannelCount,
    pub sample_rate: SampleRate,
}

pub struct PcmDecoder {
    samples: Arc<[f32]>,
    idx: usize,
    channels: ChannelCount,
    sample_rate: SampleRate,
}

impl Iterator for PcmDecoder {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let s = *self.samples.get(self.idx)?;
        self.idx += 1;
        Some(s)
    }
}

impl Source for PcmDecoder {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.idx))
    }

    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let ch = self.channels.get() as u64;
        let sr = self.sample_rate.get() as u64;
        let frames = (self.samples.len() as u64) / ch.max(1);
        Some(Duration::from_secs_f64(frames as f64 / sr.max(1) as f64))
    }
}

impl Decodable for PcmSound {
    type Decoder = PcmDecoder;

    fn decoder(&self) -> Self::Decoder {
        PcmDecoder {
            samples: Arc::clone(&self.samples),
            idx: 0,
            channels: self.channels,
            sample_rate: self.sample_rate,
        }
    }
}

fn load_wav_pcm(path: &std::path::Path) -> Result<PcmSound, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{}: not a RIFF WAVE file", path.display()));
    }

    let mut channels = 1u16;
    let mut sample_rate = 22_050u32;
    let mut bits_per_sample = 16u16;
    let mut data: Option<&[u8]> = None;

    let mut pos = 12usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let body_start = pos + 8;
        let body_end = body_start.saturating_add(size).min(bytes.len());
        let body = &bytes[body_start..body_end];

        if id == b"fmt " && body.len() >= 16 {
            let fmt = u16::from_le_bytes([body[0], body[1]]);
            if fmt != 1 {
                return Err(format!("{}: unsupported wav fmt {fmt}", path.display()));
            }
            channels = u16::from_le_bytes([body[2], body[3]]);
            sample_rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
            bits_per_sample = u16::from_le_bytes([body[14], body[15]]);
        } else if id == b"data" {
            data = Some(body);
        }

        pos = body_start + size + (size & 1);
    }

    if bits_per_sample != 16 {
        return Err(format!("{}: expected 16-bit PCM, got {bits_per_sample}", path.display()));
    }
    let data = data.ok_or_else(|| format!("{}: missing data chunk", path.display()))?;
    let mut samples = Vec::with_capacity(data.len() / 2);
    for pair in data.chunks_exact(2) {
        let raw = i16::from_le_bytes([pair[0], pair[1]]);
        samples.push(raw as f32 / 32768.0);
    }

    Ok(PcmSound {
        samples: Arc::from(samples),
        channels: NonZeroU16::new(channels.max(1)).unwrap(),
        sample_rate: NonZeroU32::new(sample_rate.max(8000)).unwrap(),
    })
}

fn audio_dir() -> PathBuf {
    sim::data_dir()
        .parent()
        .map(|p| p.join("assets").join("audio"))
        .unwrap_or_else(|| PathBuf::from("assets/audio"))
}

#[derive(Resource)]
struct AudioBank {
    wind: Handle<PcmSound>,
    crunch: Handle<PcmSound>,
    drone: Handle<PcmSound>,
    whistle: Handle<PcmSound>,
    boom: Handle<PcmSound>,
    phone: Handle<PcmSound>,
}

#[derive(Resource, Default)]
pub struct AudioState {
    pub sheltered: bool,
    pub fog: f32,
    pub drone_dist: Option<f32>,
    pub muted: bool,
    pub menu_open: bool,
    pub indoor_sealed: bool,
    pub indoor_broken: bool,
}

#[derive(Resource, Default)]
pub struct AudioQueue {
    pub footstep: Option<bool>,
    pub whistle: bool,
    pub explosions: Vec<f32>,
    pub gunfire: u32,
    pub sms: bool,
    pub door_action: bool,
    pub door_thud: bool,
    pub glass_break: bool,
    pub tank_cannon: bool,
    pub slab_collapse: bool,
}

#[derive(Component)]
struct WindLoop;

#[derive(Component)]
struct DroneLoop;

pub fn install(app: &mut App, interactive: bool) {
    app.add_audio_source::<PcmSound>()
        .init_resource::<AudioState>()
        .init_resource::<AudioQueue>();

    if interactive {
        app.add_systems(Startup, setup_audio)
            .add_systems(Update, (drive_loops, flush_queue));
    }
}

fn setup_audio(mut commands: Commands, mut sounds: ResMut<Assets<PcmSound>>) {
    let dir = audio_dir();
    let mut load = |file: &str| -> Handle<PcmSound> {
        let path = dir.join(file);
        match load_wav_pcm(&path) {
            Ok(pcm) => sounds.add(pcm),
            Err(e) => {
                warn!("audio load fallback ({e})");
                sounds.add(PcmSound {
                    samples: Arc::from([0.0f32; 64]),
                    channels: NonZeroU16::new(1).unwrap(),
                    sample_rate: NonZeroU32::new(22_050).unwrap(),
                })
            }
        }
    };

    let bank = AudioBank {
        wind: load("wind.wav"),
        crunch: load("crunch.wav"),
        drone: load("drone.wav"),
        whistle: load("whistle.wav"),
        boom: load("boom.wav"),
        phone: load("phone.wav"),
    };

    commands.spawn((
        AudioPlayer(bank.wind.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.22)),
        WindLoop,
    ));
    commands.spawn((
        AudioPlayer(bank.drone.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
        DroneLoop,
    ));

    commands.insert_resource(bank);
}

fn drive_loops(
    state: Res<AudioState>,
    mut wind: Query<&mut AudioSink, (With<WindLoop>, Without<DroneLoop>)>,
    mut drone: Query<&mut AudioSink, (With<DroneLoop>, Without<WindLoop>)>,
) {
    let target_wind = if state.muted {
        0.0
    } else if state.menu_open {
        0.10
    } else if state.indoor_sealed {
        0.04
    } else if state.indoor_broken {
        0.16
    } else if state.sheltered {
        0.07
    } else {
        0.24 + 0.12 * state.fog.clamp(0.0, 1.0)
    };
    for mut sink in &mut wind {
        sink.set_volume(Volume::Linear(target_wind));
    }

    let target_drone = if state.muted || state.menu_open {
        0.0
    } else {
        match state.drone_dist {
            Some(d) => ((1.0 - d / 160.0).clamp(0.0, 1.0)).powf(1.4) * 0.40,
            None => 0.0,
        }
    };
    for mut sink in &mut drone {
        sink.set_volume(Volume::Linear(target_drone));
    }
}

fn flush_queue(
    mut commands: Commands,
    mut queue: ResMut<AudioQueue>,
    state: Res<AudioState>,
    bank: Option<Res<AudioBank>>,
    time: Res<Time>,
    mut last_step: Local<f32>,
    mut last_shot: Local<f32>,
) {
    let clear_q = |q: &mut AudioQueue| {
        q.footstep = None;
        q.whistle = false;
        q.explosions.clear();
        q.gunfire = 0;
        q.sms = false;
        q.door_action = false;
        q.door_thud = false;
        q.glass_break = false;
        q.tank_cannon = false;
        q.slab_collapse = false;
    };
    let Some(bank) = bank else {
        clear_q(&mut queue);
        return;
    };
    if state.muted || state.menu_open {
        clear_q(&mut queue);
        return;
    }

    let now = time.elapsed_secs();

    if let Some(sprinting) = queue.footstep.take() {
        let min_gap = if sprinting { 0.21 } else { 0.34 };
        if now - *last_step >= min_gap {
            *last_step = now;
            let vol = if sprinting { 0.28 } else { 0.17 };
            let speed = if sprinting { 1.15 } else { 0.96 };
            commands.spawn((
                AudioPlayer(bank.crunch.clone()),
                PlaybackSettings::DESPAWN
                    .with_volume(Volume::Linear(vol))
                    .with_speed(speed),
            ));
        }
    }

    if queue.whistle {
        queue.whistle = false;
        commands.spawn((
            AudioPlayer(bank.whistle.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.45)),
        ));
    }

    for dist in queue.explosions.drain(..) {
        let vol = (1.0 - dist / 180.0).clamp(0.14, 0.85);
        commands.spawn((
            AudioPlayer(bank.boom.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(vol)),
        ));
    }

    if queue.gunfire > 0 {
        queue.gunfire = 0;
        if now - *last_shot >= 0.09 {
            *last_shot = now;
            commands.spawn((
                AudioPlayer(bank.boom.clone()),
                PlaybackSettings::DESPAWN
                    .with_volume(Volume::Linear(0.18))
                    .with_speed(2.4),
            ));
        }
    }

    if queue.sms {
        queue.sms = false;
        commands.spawn((
            AudioPlayer(bank.phone.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.50)),
        ));
    }

    if queue.door_action {
        queue.door_action = false;
        commands.spawn((
            AudioPlayer(bank.crunch.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(0.22))
                .with_speed(0.75),
        ));
    }

    if queue.door_thud {
        queue.door_thud = false;
        commands.spawn((
            AudioPlayer(bank.boom.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(0.36))
                .with_speed(0.70),
        ));
    }

    if queue.glass_break {
        queue.glass_break = false;
        commands.spawn((
            AudioPlayer(bank.crunch.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(0.46))
                .with_speed(2.25),
        ));
    }

    if queue.tank_cannon {
        queue.tank_cannon = false;
        commands.spawn((
            AudioPlayer(bank.boom.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(0.92))
                .with_speed(0.58),
        ));
    }

    if queue.slab_collapse {
        queue.slab_collapse = false;
        commands.spawn((
            AudioPlayer(bank.boom.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(0.75))
                .with_speed(0.45),
        ));
        commands.spawn((
            AudioPlayer(bank.crunch.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::Linear(0.60))
                .with_speed(0.65),
        ));
    }
}
