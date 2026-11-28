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
    rain: Handle<PcmSound>,
    stove_crackle: Handle<PcmSound>,
    crunch: Handle<PcmSound>,
    drone: Handle<PcmSound>,
    whistle: Handle<PcmSound>,
    boom: Handle<PcmSound>,
    grenade_boom: Handle<PcmSound>,
    tank_cannon: Handle<PcmSound>,
    rifle_shot: Handle<PcmSound>,
    mg_burst: Handle<PcmSound>,
    shotgun_blast: Handle<PcmSound>,
    rpg_launch: Handle<PcmSound>,
    smoke_hiss: Handle<PcmSound>,
    flashlight_click: Handle<PcmSound>,
    door_wood: Handle<PcmSound>,
    door_thud: Handle<PcmSound>,
    glass_break: Handle<PcmSound>,
    slab_collapse: Handle<PcmSound>,
    phone: Handle<PcmSound>,
    voice_ru_stoy: Handle<PcmSound>,
    voice_ru_grenade: Handle<PcmSound>,
    voice_radio_orders: Handle<PcmSound>,
    voice_radio_mandown: Handle<PcmSound>,
    voice_radio_cleared: Handle<PcmSound>,
    voice_radio_rescued: Handle<PcmSound>,
    voice_radio_contact: Handle<PcmSound>,
}

#[derive(Resource, Default)]
pub struct AudioState {
    pub sheltered: bool,
    pub fog: f32,
    pub raining: bool,
    pub near_stove: bool,
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
    pub shotgun: bool,
    pub mg_burst: bool,
    pub rpg_launch: bool,
    pub grenade_boom: bool,
    pub smoke_hiss: bool,
    pub flashlight_click: bool,
    pub sms: bool,
    pub door_action: bool,
    pub door_thud: bool,
    pub glass_break: bool,
    pub tank_cannon: bool,
    pub slab_collapse: bool,
    pub shout_stoy: bool,
    pub shout_grenade: bool,
    pub radio_code: Option<&'static str>,
}

#[derive(Component)]
struct WindLoop;

#[derive(Component)]
struct RainLoop;

#[derive(Component)]
struct StoveLoop;

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
        rain: load("rain.wav"),
        stove_crackle: load("stove_crackle.wav"),
        crunch: load("crunch.wav"),
        drone: load("drone.wav"),
        whistle: load("whistle.wav"),
        boom: load("boom.wav"),
        grenade_boom: load("grenade_boom.wav"),
        tank_cannon: load("tank_cannon.wav"),
        rifle_shot: load("rifle_shot.wav"),
        mg_burst: load("mg_burst.wav"),
        shotgun_blast: load("shotgun_blast.wav"),
        rpg_launch: load("rpg_launch.wav"),
        smoke_hiss: load("smoke_hiss.wav"),
        flashlight_click: load("flashlight_click.wav"),
        door_wood: load("door_wood.wav"),
        door_thud: load("door_thud.wav"),
        glass_break: load("glass_break.wav"),
        slab_collapse: load("slab_collapse.wav"),
        phone: load("phone.wav"),
        voice_ru_stoy: load("voice_ru_stoy.wav"),
        voice_ru_grenade: load("voice_ru_grenade.wav"),
        voice_radio_orders: load("voice_radio_orders.wav"),
        voice_radio_mandown: load("voice_radio_mandown.wav"),
        voice_radio_cleared: load("voice_radio_cleared.wav"),
        voice_radio_rescued: load("voice_radio_rescued.wav"),
        voice_radio_contact: load("voice_radio_contact.wav"),
    };

    commands.spawn((
        AudioPlayer(bank.wind.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.22)),
        WindLoop,
    ));
    commands.spawn((
        AudioPlayer(bank.rain.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
        RainLoop,
    ));
    commands.spawn((
        AudioPlayer(bank.stove_crackle.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
        StoveLoop,
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
    mut wind: Query<&mut AudioSink, (With<WindLoop>, Without<DroneLoop>, Without<RainLoop>, Without<StoveLoop>)>,
    mut rain: Query<&mut AudioSink, (With<RainLoop>, Without<WindLoop>, Without<DroneLoop>, Without<StoveLoop>)>,
    mut stove: Query<&mut AudioSink, (With<StoveLoop>, Without<WindLoop>, Without<RainLoop>, Without<DroneLoop>)>,
    mut drone: Query<&mut AudioSink, (With<DroneLoop>, Without<WindLoop>, Without<RainLoop>, Without<StoveLoop>)>,
) {
    let target_wind = if state.muted || state.raining {
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

    let target_rain = if state.muted || state.menu_open || !state.raining {
        0.0
    } else if state.indoor_sealed {
        0.06
    } else if state.sheltered {
        0.14
    } else {
        0.26
    };
    for mut sink in &mut rain {
        sink.set_volume(Volume::Linear(target_rain));
    }

    let target_stove = if state.muted || state.menu_open || !state.near_stove {
        0.0
    } else {
        0.22
    };
    for mut sink in &mut stove {
        sink.set_volume(Volume::Linear(target_stove));
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

fn play_sfx(commands: &mut Commands, clip: &Handle<PcmSound>, vol: f32, speed: f32) {
    commands.spawn((
        AudioPlayer(clip.clone()),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(vol))
            .with_speed(speed),
    ));
}

#[allow(clippy::too_many_arguments)]
fn flush_queue(
    mut commands: Commands,
    mut queue: ResMut<AudioQueue>,
    state: Res<AudioState>,
    bank: Option<Res<AudioBank>>,
    time: Res<Time>,
    mut last_step: Local<f32>,
    mut last_shot: Local<f32>,
    mut last_voice: Local<f32>,
    mut last_stoy: Local<f32>,
) {
    let clear_q = |q: &mut AudioQueue| {
        q.footstep = None;
        q.whistle = false;
        q.explosions.clear();
        q.gunfire = 0;
        q.shotgun = false;
        q.mg_burst = false;
        q.rpg_launch = false;
        q.grenade_boom = false;
        q.smoke_hiss = false;
        q.flashlight_click = false;
        q.sms = false;
        q.door_action = false;
        q.door_thud = false;
        q.glass_break = false;
        q.tank_cannon = false;
        q.slab_collapse = false;
        q.shout_stoy = false;
        q.shout_grenade = false;
        q.radio_code = None;
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
            let speed = if sprinting { 1.14 } else { 0.96 };
            play_sfx(&mut commands, &bank.crunch, vol, speed);
        }
    }

    if queue.whistle {
        queue.whistle = false;
        play_sfx(&mut commands, &bank.whistle, 0.46, 1.0);
    }

    if queue.grenade_boom {
        queue.grenade_boom = false;
        play_sfx(&mut commands, &bank.grenade_boom, 0.72, 1.0);
    }

    for dist in queue.explosions.drain(..) {
        let vol = (1.0 - dist / 180.0).clamp(0.14, 0.85);
        play_sfx(&mut commands, &bank.boom, vol, 1.0);
    }

    if queue.shotgun {
        queue.shotgun = false;
        queue.gunfire = 0;
        if now - *last_shot >= 0.14 {
            *last_shot = now;
            play_sfx(&mut commands, &bank.shotgun_blast, 0.56, 1.0);
        }
    } else if queue.mg_burst {
        queue.mg_burst = false;
        queue.gunfire = 0;
        if now - *last_shot >= 0.18 {
            *last_shot = now;
            play_sfx(&mut commands, &bank.mg_burst, 0.46, 1.0);
        }
    } else if queue.gunfire > 0 {
        let multi = queue.gunfire >= 3;
        queue.gunfire = 0;
        if now - *last_shot >= 0.09 {
            *last_shot = now;
            if multi {
                play_sfx(&mut commands, &bank.mg_burst, 0.42, 1.05);
            } else {
                play_sfx(&mut commands, &bank.rifle_shot, 0.42, 1.0);
            }
        }
    }

    if queue.rpg_launch {
        queue.rpg_launch = false;
        play_sfx(&mut commands, &bank.rpg_launch, 0.65, 1.0);
    }

    if queue.smoke_hiss {
        queue.smoke_hiss = false;
        play_sfx(&mut commands, &bank.smoke_hiss, 0.48, 1.0);
    }

    if queue.flashlight_click {
        queue.flashlight_click = false;
        play_sfx(&mut commands, &bank.flashlight_click, 0.42, 1.0);
    }

    if queue.sms {
        queue.sms = false;
        play_sfx(&mut commands, &bank.phone, 0.50, 1.0);
    }

    if queue.door_action {
        queue.door_action = false;
        play_sfx(&mut commands, &bank.door_wood, 0.46, 1.0);
    }

    if queue.door_thud {
        queue.door_thud = false;
        play_sfx(&mut commands, &bank.door_thud, 0.55, 1.0);
    }

    if queue.glass_break {
        queue.glass_break = false;
        play_sfx(&mut commands, &bank.glass_break, 0.54, 1.0);
    }

    if queue.tank_cannon {
        queue.tank_cannon = false;
        play_sfx(&mut commands, &bank.tank_cannon, 0.88, 1.0);
    }

    if queue.slab_collapse {
        queue.slab_collapse = false;
        play_sfx(&mut commands, &bank.slab_collapse, 0.78, 1.0);
    }

    if queue.shout_stoy {
        queue.shout_stoy = false;
        if now - *last_stoy >= 3.5 {
            *last_stoy = now;
            play_sfx(&mut commands, &bank.voice_ru_stoy, 0.68, 1.0);
        }
    }

    if queue.shout_grenade {
        queue.shout_grenade = false;
        if now - *last_stoy >= 2.2 {
            *last_stoy = now;
            play_sfx(&mut commands, &bank.voice_ru_grenade, 0.72, 1.0);
        }
    }

    if let Some(code) = queue.radio_code.take() {
        if now - *last_voice >= 2.4 {
            *last_voice = now;
            let clip = match code {
                "MAN DOWN" | "INCOMING GRENADE" => &bank.voice_radio_mandown,
                "TAKING CASUALTIES" => &bank.voice_radio_contact,
                "SECTOR CLEARED" => &bank.voice_radio_cleared,
                "TEAMMATE RESCUED" => &bank.voice_radio_rescued,
                _ => &bank.voice_radio_orders,
            };
            play_sfx(&mut commands, clip, 0.62, 1.0);
        }
    }
}
