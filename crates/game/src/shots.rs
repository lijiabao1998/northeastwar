//! `northeast --shots <folder>`: stand in a list of places at a list of
//! hours, save what the camera sees, and quit. The window stays hidden; the
//! pictures are taken of it, at its real size (3200 x 1800 on a 1600 x 900
//! window at 200% scaling). NORTHEAST_SHOT_TARGET=image draws to a 1600 x 900
//! image instead, which leaks memory in Bevy 0.20 with this many meshes (see
//! to_window). The frame time at each view goes to frame_times.txt.

use crate::{set_zoom, Clock, Eye, Player};
use bevy::{
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{save_to_disk, Capturing, Screenshot},
    },
};
use sim::geom::P2;

struct View {
    name: &'static str,
    at: P2,
    zoom: f32,
    hour: f32,
}

#[derive(Resource)]
struct Plan {
    dir: String,
    views: Vec<View>,
    index: usize,
    frames: u32,
    target: Handle<Image>,
    /// Frame times in ms at the current view, after it has settled.
    times: Vec<f32>,
    report: Vec<String>,
}

const WARM_UP_SECS: f32 = 10.0;
const SETTLE: u32 = 50;
const AFTER: u32 = 30;

pub fn install(app: &mut App, dir: String) {
    let _ = std::fs::create_dir_all(&dir);
    let views = vec![
        View { name: "01_central_street_noon", at: P2::new(-262.0, -322.0), zoom: 1.0, hour: 12.5 },
        View { name: "02_central_street_wide", at: P2::new(-300.0, -300.0), zoom: 0.42, hour: 12.5 },
        View { name: "03_housing_noon", at: P2::new(-140.0, -250.0), zoom: 0.8, hour: 13.5 },
        View { name: "04_mother_square", at: P2::new(0.0, -8.0), zoom: 0.9, hour: 12.0 },
        View { name: "05_river_and_far_bank", at: P2::new(0.0, 120.0), zoom: 0.16, hour: 12.0 },
        View { name: "06_whole_town", at: P2::new(-180.0, -200.0), zoom: 0.14, hour: 13.0 },
        View { name: "07_central_street_dusk", at: P2::new(-262.0, -322.0), zoom: 1.0, hour: 16.2 },
        View { name: "08_housing_night", at: P2::new(-140.0, -250.0), zoom: 0.8, hour: 18.8 },
        View { name: "09_central_street_night", at: P2::new(-262.0, -322.0), zoom: 0.6, hour: 18.8 },
        View { name: "10_shop_boards_close", at: P2::new(-139.0, -221.0), zoom: 2.2, hour: 13.0 },
        View { name: "11_shop_boards_night", at: P2::new(-139.0, -221.0), zoom: 2.2, hour: 18.8 },
    ];
    app.insert_resource(PlanSeed { dir, views }).add_systems(Startup, retarget.after(crate::setup)).add_systems(Update, run);
}

#[derive(Resource)]
struct PlanSeed {
    dir: String,
    views: Vec<View>,
}

fn retarget(mut commands: Commands, mut images: ResMut<Assets<Image>>, eye: Single<Entity, With<Eye>>, seed: Option<ResMut<PlanSeed>>) {
    let Some(mut seed) = seed else { return };
    // Only when asked: see to_window.
    let target = if to_window() {
        Handle::default()
    } else {
        let target = images.add(Image::new_target_texture(1600, 900, TextureFormat::Rgba8Unorm, Some(TextureFormat::Rgba8UnormSrgb)));
        commands.entity(*eye).insert(RenderTarget::Image(target.clone().into()));
        target
    };
    commands.insert_resource(Plan { dir: seed.dir.clone(), views: std::mem::take(&mut seed.views), index: 0, frames: 0, target, times: Vec::new(), report: Vec::new() });
    commands.remove_resource::<PlanSeed>();
}

fn run(
    mut commands: Commands,
    plan: Option<ResMut<Plan>>,
    mut clock: ResMut<Clock>,
    mut player: Single<(&mut Player, &mut Transform)>,
    mut eye: Single<(&mut Eye, &mut Projection)>,
    mut exit: MessageWriter<AppExit>,
    mut warm: Local<u32>,
    time: Res<Time>,
    capturing: Query<(), With<Capturing>>,
    heihe: Res<crate::Heihe>,
) {
    let Some(mut plan) = plan else { return };
    // The first seconds go to uploading some 2,600 building meshes and building pipelines, and
    // show nothing. Counted in seconds: how many frames that takes depends on the machine.
    *warm += 1;
    let warm_up = std::env::var("NORTHEAST_SHOT_WARM").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(WARM_UP_SECS);
    if time.elapsed_secs() < warm_up || *warm < 60 {
        return;
    }
    // A picture is read back from the GPU some frames after it is asked for.
    // Nothing moves, and the program does not end, until it is on disk.
    if !capturing.is_empty() {
        return;
    }
    if plan.index >= plan.views.len() {
        if plan.frames > AFTER {
            let text = plan.report.join("\n");
            println!("{text}");
            let _ = std::fs::write(format!("{}/frame_times.txt", plan.dir), text + "\n");
            exit.write(AppExit::Success);
        }
        plan.frames += 1;
        return;
    }
    let (name, at, zoom, hour) = {
        let v = &plan.views[plan.index];
        (v.name, v.at, v.zoom, v.hour)
    };
    if plan.frames == 0 {
        let (who, transform) = &mut *player;
        // Stand where a person could: a view point that falls inside a wall is pushed out of it.
        let at = heihe.ground.step(at, at, 0.35);
        who.at = at;
        transform.translation = Vec3::new(at.x, 0.0, -at.y);
        clock.hour = hour;
        let (eye, projection) = &mut *eye;
        eye.zoom = zoom;
        set_zoom(projection, zoom);
    }
    // Frame time while standing still at this view, before the picture is taken.
    if plan.frames > SETTLE / 2 && plan.frames < SETTLE {
        let ms = time.delta_secs() * 1000.0;
        plan.times.push(ms);
    }
    if plan.frames == SETTLE {
        let mut t = std::mem::take(&mut plan.times);
        t.sort_by(|a, b| a.total_cmp(b));
        let mean = t.iter().sum::<f32>() / t.len().max(1) as f32;
        let p95 = t.get((t.len() as f32 * 0.95) as usize).copied().unwrap_or(0.0);
        let line = format!("{:<28} mean {:6.1} ms  p95 {:6.1} ms  over {} frames", name, mean, p95, t.len());
        plan.report.push(line);
        let path = format!("{}/{}.png", plan.dir, name);
        let shot = if to_window() { Screenshot::primary_window() } else { Screenshot::image(plan.target.clone()) };
        commands.spawn(shot).observe(save_to_disk(path));
    }
    plan.frames += 1;
    if plan.frames > SETTLE + AFTER {
        plan.frames = 0;
        plan.index += 1;
    }
}

/// Pictures are taken of the hidden window. Drawing to an image target instead
/// (NORTHEAST_SHOT_TARGET=image) grows memory by about 80 MB a second with
/// this many meshes in Bevy 0.20, until the GPU runs out: measured 16 November 2026.
fn to_window() -> bool {
    std::env::var("NORTHEAST_SHOT_TARGET").map(|v| v != "image").unwrap_or(true)
}
