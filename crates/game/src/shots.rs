//! `northeast --shots <folder>`: stand in a list of places at a list of
//! hours, save what the camera sees, and quit. It draws to a picture, not to
//! the window, so it works with the window hidden.

use crate::{set_zoom, Clock, Eye, Player};
use bevy::{
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{save_to_disk, Screenshot},
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
}

const WARM_UP: u32 = 240;
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
    let image = Image::new_target_texture(1600, 900, TextureFormat::Rgba8Unorm, Some(TextureFormat::Rgba8UnormSrgb));
    let target = images.add(image);
    commands.entity(*eye).insert(RenderTarget::Image(target.clone().into()));
    commands.insert_resource(Plan { dir: seed.dir.clone(), views: std::mem::take(&mut seed.views), index: 0, frames: 0, target });
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
) {
    let Some(mut plan) = plan else { return };
    // The first frames go to building pipelines, and show nothing.
    *warm += 1;
    if *warm < WARM_UP {
        return;
    }
    if plan.index >= plan.views.len() {
        if plan.frames > AFTER {
            exit.write(AppExit::Success);
        }
        plan.frames += 1;
        return;
    }
    let view = &plan.views[plan.index];
    if plan.frames == 0 {
        let (who, transform) = &mut *player;
        who.at = view.at;
        transform.translation = Vec3::new(view.at.x, 0.0, -view.at.y);
        clock.hour = view.hour;
        let (eye, projection) = &mut *eye;
        eye.zoom = view.zoom;
        set_zoom(projection, view.zoom);
    }
    if plan.frames == SETTLE {
        let path = format!("{}/{}.png", plan.dir, view.name);
        commands.spawn(Screenshot::image(plan.target.clone())).observe(save_to_disk(path));
    }
    plan.frames += 1;
    if plan.frames > SETTLE + AFTER {
        plan.frames = 0;
        plan.index += 1;
    }
}
