//! `northeast --shots <folder>`: stand in a list of places at a list of
//! hours, save what the camera sees, and quit. The window stays hidden; the
//! pictures are taken of it, at its real size (3200 x 1800 on a 1600 x 900
//! window at 200% scaling). NORTHEAST_SHOT_TARGET=image draws to a 1600 x 900
//! image instead, which leaks memory in Bevy 0.20 with this many meshes (see
//! to_window). The frame time at each view goes to frame_times.txt.

use crate::{
    map_ui::TacticalMap,
    menu_ui::{MenuScreen, ReturnScreen},
    play::{spawn_fx, FigureAtlas, PlayState},
    set_zoom, Clock, Eye, Player,
};
use bevy::{
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{save_to_disk, Capturing, Screenshot},
    },
};
use sim::{
    figure::AnimState,
    geom::P2,
    look::Rgb,
    survival::{
        hm, ActiveGrenade, Barricade, BarricadeKind, BloodMark, Crater, ExplosionFx, FpvDrone,
        MilitiaMember, MilitiaOrder, RadioCallout, SmokeCloud, SnowTrack, Soldier, Squad,
        SquadState, TracerFx, VehicleKind, VehicleWreck, Weather, WorldSim,
    },
};

pub(crate) struct View {
    pub(crate) name: &'static str,
    pub(crate) at: P2,
    pub(crate) zoom: f32,
    pub(crate) hour: f32,
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
    let views = views();
    app.insert_resource(PlanSeed { dir, views }).add_systems(Startup, retarget.after(crate::setup)).add_systems(Update, run);
}

/// The places, zooms and hours pictures are taken at (and --bench times).
pub(crate) fn views() -> Vec<View> {
    vec![
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
    ]
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

pub fn install_combat(app: &mut App, dir: String) {
    let _ = std::fs::create_dir_all(&dir);
    app.insert_resource(CombatPlanSeed { dir })
        .add_systems(Startup, retarget_combat.after(crate::play::setup_play))
        .add_systems(Update, run_combat.before(crate::follow).before(crate::play::sync_actors));
}

#[derive(Resource)]
struct CombatPlanSeed {
    dir: String,
}

#[derive(Resource)]
struct CombatPlan {
    dir: String,
    index: usize,
    frames: u32,
    target: Handle<Image>,
}

const COMBAT_NAMES: [&str; 3] = [
    "01_ambush_central_street",
    "02_river_fog_blackout",
    "03_tactical_map",
];

fn retarget_combat(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    eye: Single<Entity, With<Eye>>,
    seed: Option<Res<CombatPlanSeed>>,
) {
    let Some(seed) = seed else { return };
    let target = if to_window() {
        Handle::default()
    } else {
        let target = images.add(Image::new_target_texture(
            1600,
            900,
            TextureFormat::Rgba8Unorm,
            Some(TextureFormat::Rgba8UnormSrgb),
        ));
        commands.entity(*eye).insert(RenderTarget::Image(target.clone().into()));
        target
    };
    commands.insert_resource(CombatPlan {
        dir: seed.dir.clone(),
        index: 0,
        frames: 0,
        target,
    });
    commands.remove_resource::<CombatPlanSeed>();
}

fn configure_combat_scene(
    idx: usize,
    heihe: &crate::Heihe,
    sim: &mut WorldSim,
    clock: &mut Clock,
    map: &mut TacticalMap,
    images: &mut Assets<Image>,
    forces: &sim::forces::Forces,
) -> (P2, f32, Vec<TracerFx>, Vec<ExplosionFx>, Option<P2>, bool) {
    *sim = WorldSim::new(&heihe.city);
    sim.inv.has_rifle = true;
    sim.inv.ammo = 24;
    sim.inv.has_radio = true;
    sim.inv.scrap = 18;
    sim.inv.coal = 12;
    sim.body.coat_bonus = 1.0;
    sim.resistance_active = true;
    sim.alert_stage = 2;

    let step = |p: P2| heihe.ground.step(p, p, 0.35);

    match idx {
        0 => {
            // 01: 18:12 Central St PLA vs Russia armored engagement + Lancet-3 + modern arsenal.
            let c = heihe.city.spawn;
            sim.hour = hm(18, 12);
            sim.blackout = false;
            sim.weather_override = Some(Weather::Clear);
            sim.weather = Weather::Clear;
            sim.fog_density = 0.0;
            sim.inv.unlock_all_arsenal();
            sim.player_at = c;
            sim.player_facing = 2.1;
            sim.player_anim = AnimState::Aim;
            sim.under_fire_for = 1.2;
            sim.selected_barricade = BarricadeKind::Sandbag;
            sim.objective_zh = "PLA 重装反击：ZTZ-99A + PGZ-09 迎战俄军 T-90M（带防无人机格栅）+ TOS-1A 温压炮".into();

            sim.spawn_tank_wave_at(sim::survival::VehicleKind::Ztz99A, c, forces);
            if let Some(t) = sim.tanks.last_mut() {
                t.at = step(c.add(P2::new(-5.5, -3.5)));
                t.hull_yaw = 1.6;
                t.turret_yaw = 1.65;
            }
            sim.spawn_tank_wave_at(sim::survival::VehicleKind::Pgz09, c, forces);
            if let Some(t) = sim.tanks.last_mut() {
                t.at = step(c.add(P2::new(6.2, -3.0)));
                t.hull_yaw = 1.5;
                t.turret_yaw = 1.55;
            }
            sim.spawn_tank_wave_at(sim::survival::VehicleKind::T90M, c, forces);
            if let Some(t) = sim.tanks.last_mut() {
                t.at = step(c.add(P2::new(-4.5, 16.5)));
                t.hull_yaw = -1.55;
                t.turret_yaw = -1.55;
                t.locked_target = Some((c.add(P2::new(-5.5, -3.5)), 0));
                t.lock_timer = 1.0;
            }
            sim.spawn_tank_wave_at(sim::survival::VehicleKind::Tos1A, c, forces);
            if let Some(t) = sim.tanks.last_mut() {
                t.at = step(c.add(P2::new(5.8, 18.0)));
                t.hull_yaw = -1.65;
                t.turret_yaw = -1.65;
            }
            sim.lancet = Some(sim::survival::LancetDrone {
                origin: c.add(P2::new(15.0, 26.0)),
                at: c.add(P2::new(6.0, 9.0)),
                height: 8.5,
                target: c,
                hp: 20.0,
                hit_tank_id: None,
            });

            sim.weather_override = Some(Weather::Flurries);
            sim.weather = Weather::Flurries;

            let b1 = step(c.add(P2::new(-2.8, 3.2)));
            let b2 = step(c.add(P2::new(1.8, 2.6)));
            let b_tarp = step(c.add(P2::new(-4.8, 1.4)));
            let b_trench = step(c.add(P2::new(3.6, 1.8)));
            let b_wire = step(c.add(P2::new(-0.5, 6.2)));
            sim.barricades.push(Barricade { at: b1, kind: BarricadeKind::Sandbag, yaw: 0.4, shoved: false, hp: 80.0 });
            sim.barricades.push(Barricade { at: b2, kind: BarricadeKind::FireBarrel, yaw: 0.0, shoved: false, hp: 40.0 });
            sim.barricades.push(Barricade { at: b_tarp, kind: BarricadeKind::TarpDugout, yaw: 0.4, shoved: false, hp: 150.0 });
            sim.barricades.push(Barricade { at: b_trench, kind: BarricadeKind::Trench, yaw: 0.15, shoved: false, hp: 180.0 });
            sim.barricades.push(Barricade { at: b_wire, kind: BarricadeKind::BarbedWire, yaw: 0.0, shoved: false, hp: 110.0 });
            sim.fire_barrels.push(b2);

            let crater_at = step(c.add(P2::new(1.2, 8.8)));
            sim.craters.push(Crater { id: 701, at: crater_at, radius: 3.1 });

            let wreck_at = step(c.add(P2::new(7.5, 9.5)));
            sim.wrecks.push(VehicleWreck {
                id: 702,
                kind: VehicleKind::Btr82A,
                at: wreck_at,
                hull_yaw: -1.1,
                turret_yaw: -0.7,
                smoke_ttl: 30.0,
            });
            for vi in 0..4 {
                let vp = step(wreck_at.add(P2::new(vi as f32 * 0.9, vi as f32 * 1.45 + 1.5)));
                sim.tracks.push(SnowTrack {
                    id: 600 + vi as u32,
                    at: vp,
                    dir: P2::new(-0.5, -0.86).norm(),
                    age: 8.0 + vi as f32 * 2.0,
                    by_player: false,
                    is_vehicle: true,
                });
            }

            let smoke_at = step(c.add(P2::new(-5.8, 8.2)));
            sim.smoke_clouds.push(SmokeCloud {
                id: 703,
                at: smoke_at,
                radius: 4.5,
                ttl: 22.0,
                dark: false,
            });

            let m1 = b_tarp;
            let m2 = b_trench;
            let m3 = step(c.add(P2::new(-1.2, -3.2)));
            sim.militia = vec![
                MilitiaMember {
                    id: 1,
                    name: "老王",
                    at: m1,
                    floor: 0,
                    facing: 1.95,
                    hp: 100.0,
                    coat: Rgb { r: 0.25, g: 0.30, b: 0.38 },
                    order: MilitiaOrder::Hold(m1),
                    fire_cd: 0.85,
                    speed: 0.0,
                    phase: 0.0,
                    in_cover: true,
                    vault_t: 0.0,
                    downed: false,
                    bleedout_s: 0.0,
                    rescuing_id: None,
                    rescue_progress: 0.0,
                },
                MilitiaMember {
                    id: 2,
                    name: "小周",
                    at: m2,
                    floor: 0,
                    facing: 2.15,
                    hp: 100.0,
                    coat: Rgb { r: 0.32, g: 0.28, b: 0.22 },
                    order: MilitiaOrder::Hold(m2),
                    fire_cd: 0.80,
                    speed: 0.0,
                    phase: 0.0,
                    in_cover: true,
                    vault_t: 0.0,
                    downed: false,
                    bleedout_s: 0.0,
                    rescuing_id: None,
                    rescue_progress: 0.0,
                },
                MilitiaMember {
                    id: 3,
                    name: "李姐",
                    at: m3,
                    floor: 0,
                    facing: 0.4,
                    hp: 18.0,
                    coat: Rgb { r: 0.44, g: 0.22, b: 0.22 },
                    order: MilitiaOrder::Follow,
                    fire_cd: 0.0,
                    speed: 0.45,
                    phase: 1.2,
                    in_cover: false,
                    vault_t: 0.0,
                    downed: true,
                    bleedout_s: 22.0,
                    rescuing_id: None,
                    rescue_progress: 0.0,
                },
            ];
            for bi in 0..3 {
                sim.blood_marks.push(BloodMark {
                    id: 901 + bi as u32,
                    at: m3.add(P2::new(-(bi as f32) * 0.85, -(bi as f32) * 0.45)),
                    yaw: 0.4,
                    age: bi as f32 * 2.0,
                });
            }

            let s1 = step(c.add(P2::new(-9.5, 13.5)));
            let s2 = step(c.add(P2::new(-12.5, 15.0)));
            let s3 = step(c.add(P2::new(-6.8, 16.0)));
            sim.squads.push(Squad {
                id: 1,
                state: SquadState::Engage,
                route_idx: 0,
                wp_idx: 0,
                wp_dir: 1,
                men: vec![
                    Soldier { id: 101, at: s1, floor: 0, facing: -0.95, hp: 65.0, optics: true, shot_cd: 0.2, speed: 0.0, phase: 0.0, in_cover: true, vault_t: 0.0, grenade_cd: 4.0 },
                    Soldier { id: 102, at: s2, floor: 0, facing: -0.90, hp: 65.0, optics: false, shot_cd: 0.4, speed: 1.4, phase: 0.8, in_cover: false, vault_t: 0.0, grenade_cd: 6.0 },
                    Soldier { id: 103, at: s3, floor: 0, facing: -1.05, hp: 65.0, optics: false, shot_cd: 0.6, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 8.0 },
                ],
                meter: 1.0,
                last_seen: c,
                lost_for: 0.0,
                timer: 4.0,
                ignore_for: 0.0,
                held_at: s1,
                shouted: true,
                hostile: true,
                stuck_t: 0.0,
                last_lead_at: s1,
            });

            let g_tgt = step(c.add(P2::new(-8.2, 12.2)));
            sim.grenades.push(ActiveGrenade {
                id: 801,
                from: m2,
                at: m2.lerp(g_tgt, 0.52),
                target: g_tgt,
                floor: 0,
                fuse: 1.1,
                flight_t: 0.36,
                flight_dur: 0.70,
                friendly: true,
            });
            sim.militia_cmd_marker = Some((m2, 3.0));
            sim.radio_feed = Some(RadioCallout {
                unit: "蓝队-1".into(),
                code_en: "MAN DOWN",
                text_zh: "友军重伤倒地！队友正前往掩护抢救".into(),
                urgent: true,
                ttl: 5.0,
            });

            sim.drone.active = true;
            sim.drone.at = c.add(P2::new(-5.5, 7.5));
            sim.drone.centre = c;
            sim.drone.altitude = 9.5;
            sim.drone.tracking_trail = true;

            for i in 0..8 {
                let p = step(c.add(P2::new(i as f32 * 0.9 - 1.5, -(i as f32) * 1.1)));
                sim.tracks.push(SnowTrack {
                    id: 10 + i as u32,
                    at: p,
                    dir: P2::new(-0.6, 0.8).norm(),
                    age: i as f32 * 3.5,
                    by_player: true,
                    is_vehicle: false,
                });
            }

            let tracers = vec![
                TracerFx { from: [m1.x, 1.35, -m1.y], to: [s1.x, 1.3, -s1.y], friendly: true },
                TracerFx { from: [s2.x, 1.35, -s2.y], to: [b1.x, 0.9, -b1.y], friendly: false },
                TracerFx { from: [m2.x, 1.35, -m2.y], to: [s3.x, 1.2, -s3.y], friendly: true },
            ];
            let cursor = Some(step(c.add(P2::new(-1.2, 4.0))));

            clock.hour = sim.hour;
            clock.speed = 1.0;
            clock.blackout = sim.blackout;
            clock.fog = sim.fog_density;
            map.open = false;
            (c, 1.15, tracers, Vec::new(), cursor, true)
        }
        1 => {
            // 02: 21:15 blackout + dense黑龙江 river fog at Mother Square / promenade + FireBarrel glow + incoming fibre FPV.
            let c = step(P2::new(0.0, -8.0));
            sim.hour = hm(21, 15);
            sim.blackout = true;
            sim.weather_override = Some(Weather::RiverFog);
            sim.weather = Weather::RiverFog;
            sim.fog_density = 0.92;
            sim.player_at = c;
            sim.player_facing = 2.0;
            sim.player_anim = AnimState::Idle;
            sim.body.sheltered = true;
            sim.body.heated = false;
            sim.body.temp = 35.1;
            sim.objective_zh = "21:15全城停电停暖 · 江雾掩护下躲避光纤FPV无人机".into();

            let b_fire = step(c.add(P2::new(2.2, 1.8)));
            let b_bag = step(c.add(P2::new(-2.4, 2.2)));
            sim.barricades.push(Barricade { at: b_fire, kind: BarricadeKind::FireBarrel, yaw: 0.0, shoved: false, hp: 40.0 });
            sim.barricades.push(Barricade { at: b_bag, kind: BarricadeKind::Sandbag, yaw: 0.3, shoved: false, hp: 80.0 });
            sim.fire_barrels.push(b_fire);

            sim.fpv = Some(FpvDrone {
                origin: c.add(P2::new(-4.0, 26.0)),
                at: c.add(P2::new(-5.2, 7.2)),
                height: 4.5,
                target: c,
                hit: true,
                fibre_optic: true,
                jammed: false,
            });

            for i in 0..9 {
                let p = step(c.add(P2::new(i as f32 * 0.7, -(i as f32) * 1.1)));
                sim.tracks.push(SnowTrack {
                    id: 50 + i as u32,
                    at: p,
                    dir: P2::new(-0.5, 0.85).norm(),
                    age: i as f32 * 2.5,
                    by_player: true,
                    is_vehicle: false,
                });
            }

            clock.hour = sim.hour;
            clock.speed = 1.0;
            clock.blackout = sim.blackout;
            clock.fog = sim.fog_density;
            map.open = false;
            (c, 0.85, Vec::new(), Vec::new(), None, false)
        }
        _ => {
            // 03: 19:48 Tactical Map open over active Central St resistance scene.
            let c = heihe.city.spawn;
            sim.hour = hm(19, 48);
            sim.blackout = true;
            sim.weather_override = Some(Weather::Flurries);
            sim.weather = Weather::Flurries;
            sim.fog_density = 0.25;
            sim.player_at = c;
            sim.player_facing = 2.0;

            for dx in -4..=3 {
                for dy in -3..=3 {
                    let p = c.add(P2::new(dx as f32 * 20.0, dy as f32 * 20.0));
                    sim.visited.insert(((p.x / 20.0).floor() as i32, (p.y / 20.0).floor() as i32));
                }
            }

            let m1 = step(c.add(P2::new(-18.0, 22.0)));
            let m2 = step(c.add(P2::new(22.0, -12.0)));
            sim.militia = vec![
                MilitiaMember {
                    id: 10,
                    name: "民兵一",
                    at: m1,
                    floor: 0,
                    facing: 1.5,
                    hp: 100.0,
                    coat: Rgb { r: 0.25, g: 0.30, b: 0.38 },
                    order: MilitiaOrder::Hold(m1),
                    fire_cd: 0.0,
                    speed: 0.0,
                    phase: 0.0,
                    in_cover: false,
                    vault_t: 0.0,
                    downed: false,
                    bleedout_s: 0.0,
                    rescuing_id: None,
                    rescue_progress: 0.0,
                },
                MilitiaMember {
                    id: 11,
                    name: "民兵二",
                    at: m2,
                    floor: 0,
                    facing: 1.5,
                    hp: 100.0,
                    coat: Rgb { r: 0.32, g: 0.28, b: 0.22 },
                    order: MilitiaOrder::Follow,
                    fire_cd: 0.0,
                    speed: 0.0,
                    phase: 0.0,
                    in_cover: false,
                    vault_t: 0.0,
                    downed: false,
                    bleedout_s: 0.0,
                    rescuing_id: None,
                    rescue_progress: 0.0,
                },
            ];
            let s1 = step(c.add(P2::new(-32.0, 52.0)));
            let s2 = step(c.add(P2::new(25.0, 65.0)));
            sim.squads.push(Squad {
                id: 2,
                state: SquadState::Patrol,
                route_idx: 0,
                wp_idx: 0,
                wp_dir: 1,
                men: vec![
                    Soldier { id: 201, at: s1, floor: 0, facing: -1.4, hp: 65.0, optics: true, shot_cd: 0.0, speed: 1.5, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 6.0 },
                    Soldier { id: 202, at: s2, floor: 0, facing: -1.4, hp: 65.0, optics: false, shot_cd: 0.0, speed: 1.5, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 6.0 },
                ],
                meter: 0.2,
                last_seen: s1,
                lost_for: 0.0,
                timer: 0.0,
                ignore_for: 0.0,
                held_at: s1,
                shouted: false,
                hostile: false,
                stuck_t: 0.0,
                last_lead_at: s1,
            });
            sim.barricades.push(Barricade { at: c.add(P2::new(-8.0, 12.0)), kind: BarricadeKind::FireBarrel, yaw: 0.0, shoved: false, hp: 40.0 });
            sim.fire_barrels.push(c.add(P2::new(-8.0, 12.0)));
            sim.drone.active = true;
            sim.drone.at = c.add(P2::new(16.0, 38.0));
            sim.drone.centre = sim.drone.at;

            sim.spawn_tank_wave_at(sim::survival::VehicleKind::Ztz99A, c, forces);
            sim.spawn_tank_wave_at(sim::survival::VehicleKind::T90M, c.add(P2::new(0.0, 45.0)), forces);

            clock.hour = sim.hour;
            clock.speed = 1.0;
            clock.blackout = sim.blackout;
            clock.fog = sim.fog_density;
            map.open = true;
            map.set_view(c, 2.0);
            map.redraw_now(sim, &heihe.city, forces, images);
            (c, 1.0, Vec::new(), Vec::new(), None, false)
        }
    }
}

fn run_combat(
    mut commands: Commands,
    plan: Option<ResMut<CombatPlan>>,
    mut clock: ResMut<Clock>,
    mut player: Single<(&mut Player, &mut Transform, &mut Mesh3d)>,
    mut eye: Single<(&mut Eye, &mut Projection)>,
    mut exit: MessageWriter<AppExit>,
    mut warm: Local<u32>,
    (mut pending_tracers, mut pending_bursts): (Local<Vec<TracerFx>>, Local<Vec<ExplosionFx>>),
    time: Res<Time>,
    capturing: Query<(), With<Capturing>>,
    heihe: Res<crate::Heihe>,
    mut play: Option<ResMut<PlayState>>,
    mut map: Option<ResMut<TacticalMap>>,
    atlas: Option<Res<FigureAtlas>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut plan) = plan else { return };
    let (Some(play), Some(map), Some(atlas)) = (play.as_deref_mut(), map.as_deref_mut(), atlas.as_deref()) else {
        return;
    };

    *warm += 1;
    let warm_up = std::env::var("NORTHEAST_SHOT_WARM")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(WARM_UP_SECS);
    if time.elapsed_secs() < warm_up || *warm < 60 {
        return;
    }
    if !capturing.is_empty() {
        return;
    }
    if plan.index >= COMBAT_NAMES.len() {
        if plan.frames > AFTER {
            exit.write(AppExit::Success);
        }
        plan.frames += 1;
        return;
    }

    if plan.frames == 0 {
        let forces = play.forces.clone();
        let (at, zoom, tracers, bursts, cursor, preview) = configure_combat_scene(
            plan.index,
            &heihe,
            &mut play.sim,
            &mut clock,
            map,
            &mut images,
            &forces,
        );
        play.started = true;
        play.cursor_world = cursor;
        play.preview_barricade = preview;
        *pending_tracers = tracers;
        *pending_bursts = bursts;

        let (who, tf, pmesh) = &mut *player;
        who.at = at;
        tf.translation = Vec3::new(at.x, 0.0, -at.y);
        tf.rotation = Quat::from_rotation_y(play.sim.player_facing + std::f32::consts::FRAC_PI_2);
        if let Some(h) = atlas.meshes.get(&crate::play::AtlasKey {
            role: sim::figure::Role::Civilian,
            variant: u8::from(play.sim.body.coat_bonus > 0.0) | (u8::from(play.sim.inv.has_rifle) << 1),
            anim: play.sim.player_anim,
            frame: 0,
        }) {
            pmesh.0 = h.clone();
        }
        let (eye_c, proj) = &mut *eye;
        eye_c.zoom = zoom;
        set_zoom(proj, zoom);
    }

    if plan.frames == SETTLE - 2 && (!pending_tracers.is_empty() || !pending_bursts.is_empty()) {
        spawn_fx(&mut commands, atlas, &pending_tracers, &pending_bursts);
        pending_tracers.clear();
        pending_bursts.clear();
    }

    if plan.frames == SETTLE {
        let path = format!("{}/{}.png", plan.dir, COMBAT_NAMES[plan.index]);
        let shot = if to_window() {
            Screenshot::primary_window()
        } else {
            Screenshot::image(plan.target.clone())
        };
        commands.spawn(shot).observe(save_to_disk(path));
    }

    plan.frames += 1;
    if plan.frames > SETTLE + AFTER {
        plan.frames = 0;
        plan.index += 1;
    }
}

pub fn install_menu(app: &mut App, dir: String) {
    let _ = std::fs::create_dir_all(&dir);
    app.insert_resource(MenuPlanSeed { dir })
        .add_systems(Startup, retarget_menu.after(crate::play::setup_play))
        .add_systems(Update, run_menu.before(crate::follow));
}

#[derive(Resource)]
struct MenuPlanSeed {
    dir: String,
}

#[derive(Resource)]
struct MenuPlan {
    dir: String,
    index: usize,
    frames: u32,
}

const MENU_NAMES: [&str; 3] = ["01_start_menu", "02_controls_menu", "03_pause_menu"];

fn retarget_menu(mut commands: Commands, seed: Option<Res<MenuPlanSeed>>) {
    let Some(seed) = seed else { return };
    commands.insert_resource(MenuPlan {
        dir: seed.dir.clone(),
        index: 0,
        frames: 0,
    });
    commands.remove_resource::<MenuPlanSeed>();
}

fn run_menu(
    mut commands: Commands,
    plan: Option<ResMut<MenuPlan>>,
    mut clock: ResMut<Clock>,
    mut screen: ResMut<MenuScreen>,
    mut play: Option<ResMut<PlayState>>,
    mut eye: Single<(&mut Eye, &mut Projection)>,
    mut exit: MessageWriter<AppExit>,
    mut warm: Local<u32>,
    time: Res<Time>,
    capturing: Query<(), With<Capturing>>,
) {
    let Some(mut plan) = plan else { return };
    *warm += 1;
    let warm_up = std::env::var("NORTHEAST_SHOT_WARM")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(WARM_UP_SECS);
    if time.elapsed_secs() < warm_up || *warm < 60 {
        return;
    }
    if !capturing.is_empty() {
        return;
    }
    if plan.index >= MENU_NAMES.len() {
        if plan.frames > AFTER {
            exit.write(AppExit::Success);
        }
        plan.frames += 1;
        return;
    }

    if plan.frames == 0 {
        let (eye_c, proj) = &mut *eye;
        eye_c.zoom = 1.0;
        set_zoom(proj, 1.0);
        clock.blackout = false;
        clock.fog = 0.0;
        match plan.index {
            0 => {
                clock.hour = sim::survival::START_HOUR;
                if let Some(p) = play.as_deref_mut() {
                    p.started = false;
                }
                *screen = MenuScreen::Start;
            }
            1 => {
                clock.hour = sim::survival::START_HOUR;
                *screen = MenuScreen::Controls {
                    return_to: ReturnScreen::Start,
                };
            }
            _ => {
                clock.hour = sim::survival::hm(18, 10);
                if let Some(p) = play.as_deref_mut() {
                    p.started = true;
                }
                *screen = MenuScreen::Paused;
            }
        }
    }

    if plan.frames == SETTLE {
        let path = format!("{}/{}.png", plan.dir, MENU_NAMES[plan.index]);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }

    plan.frames += 1;
    if plan.frames > SETTLE + AFTER {
        plan.frames = 0;
        plan.index += 1;
    }
}

pub fn install_interior(app: &mut App, dir: String) {
    let _ = std::fs::create_dir_all(&dir);
    app.insert_resource(InteriorPlanSeed { dir })
        .add_systems(Startup, retarget_interior.after(crate::play::setup_play))
        .add_systems(Update, run_interior.before(crate::follow).before(crate::play::sync_actors));
}

#[derive(Resource)]
struct InteriorPlanSeed {
    dir: String,
}

#[derive(Resource)]
struct InteriorPlan {
    dir: String,
    index: usize,
    frames: u32,
}

const INTERIOR_NAMES: [&str; 6] = [
    "01_shop_interior_lit",
    "02_blackout_ambush_indoor",
    "03_player_home_stove",
    "04_day2_noon_street",
    "05_basement_asleep",
    "06_march_five_pm",
];

fn retarget_interior(mut commands: Commands, seed: Option<Res<InteriorPlanSeed>>) {
    let Some(seed) = seed else { return };
    commands.insert_resource(InteriorPlan {
        dir: seed.dir.clone(),
        index: 0,
        frames: 0,
    });
    commands.remove_resource::<InteriorPlanSeed>();
}

fn configure_interior_scene(
    idx: usize,
    heihe: &crate::Heihe,
    forces: &sim::forces::Forces,
    sim: &mut WorldSim,
    clock: &mut Clock,
) -> (P2, f32, Vec<TracerFx>) {
    use sim::interior::InteriorKind;
    *sim = WorldSim::new(&heihe.city);
    sim.inv.scrap = 10;
    sim.inv.coal = 6;
    sim.inv.bandage = 2;
    sim.inv.food = 2;
    sim.inv.water = 2;
    sim.body.coat_bonus = 0.35;

    match idx {
        0 => {
            // 01: 17:10 inside Pharmacy — lit overhead ceiling light, tiles, counters, NPC shopkeeper, loot rings.
            let room_i = sim
                .interiors
                .iter()
                .position(|it| it.kind == InteriorKind::Pharmacy)
                .unwrap_or(0);
            let intr = &mut sim.interiors[room_i];
            let pat = intr.to_world(-0.3, intr.split_v * 0.55);
            intr.curtains_closed = false;
            intr.room_temp_c = 18.2;

            sim.hour = hm(17, 10);
            sim.blackout = false;
            sim.weather_override = Some(Weather::Clear);
            sim.weather = Weather::Clear;
            sim.fog_density = 0.0;
            sim.player_at = pat;
            sim.player_facing = 1.9;
            sim.player_anim = AnimState::Idle;
            sim.active_interior = Some(room_i);
            sim.cutaway_interior = Some(room_i);
            sim.current_room_temp = Some(18.2);
            sim.active_interior_name = intr.name.clone();
            sim.body.sheltered = true;
            sim.body.heated = true;
            sim.objective_zh = "推开药店棉门帘进屋 · 按 E 翻找药柜与后屋库房".into();
            sim.prompt = "按 E：翻找屋内的 [前厅柜台/货架]".into();

            clock.hour = sim.hour;
            clock.blackout = false;
            clock.fog = 0.0;
            (pat, 1.55, Vec::new())
        }
        1 => {
            // 02: 20:15 2F multi-story room after T-72B3 tank 125mm hit — blown floor slab hole looking onto 1F, tank turret locking on outside.
            let room_i = sim
                .interiors
                .iter()
                .position(|it| it.num_floors >= 3 && !it.windows.is_empty())
                .unwrap_or(0);
            let spots = sim.interiors[room_i].firing_spots.clone();
            let fd_out = sim.interiors[room_i].front_door.outside;
            let hole_pos = sim.interiors[room_i].to_world(
                sim.interiors[room_i].half_w * 0.42,
                sim.interiors[room_i].depth * 0.25,
            );
            let pat = sim.interiors[room_i].to_world(
                -sim.interiors[room_i].half_w * 0.35,
                sim.interiors[room_i].depth * 0.52,
            );
            {
                let intr = &mut sim.interiors[room_i];
                intr.curtains_closed = false;
                intr.front_door.barricade_hp = intr.front_door.max_barricade_hp;
                if let Some(w) = intr.windows.first_mut() {
                    w.broken = true;
                }
                intr.blast_at(hole_pos, 1, 3.6, 240.0);
                intr.room_temp_c = 1.4;
            }

            sim.hour = hm(20, 15);
            sim.blackout = true;
            sim.alert_stage = 2;
            sim.weather_override = Some(Weather::Flurries);
            sim.weather = Weather::Flurries;
            sim.fog_density = 0.20;
            sim.inv.has_rifle = true;
            sim.inv.ammo = 22;
            sim.inv.rpg = 2;
            sim.flashlight_on = true;
            sim.player_at = pat;
            sim.player_floor = 1;
            sim.player_facing = fd_out.sub(pat).y.atan2(fd_out.sub(pat).x);
            sim.player_anim = AnimState::Aim;
            sim.active_interior = Some(room_i);
            sim.cutaway_interior = Some(room_i);
            sim.current_room_temp = Some(1.4);
            sim.active_interior_name = format!("{} 2F", sim.interiors[room_i].name);
            sim.body.sheltered = true;
            sim.body.heated = false;
            sim.objective_zh = "二楼架枪拒止 T-72B3 坦克 · 防楼板被 125mm 炮弹打穿 · F射RPG / V俯掷火瓶".into();

            let s_pos1 = heihe.ground.step(fd_out.add(P2::new(2.0, 4.5)), fd_out.add(P2::new(2.0, 4.5)), 0.35);
            let s_pos2 = heihe.ground.step(fd_out.add(P2::new(-1.8, 5.2)), fd_out.add(P2::new(-1.8, 5.2)), 0.35);
            let tank_pos = heihe.ground.step(fd_out.add(P2::new(-2.8, 8.5)), fd_out.add(P2::new(-2.8, 8.5)), 1.0);
            sim.spawn_tank_wave_at(sim::survival::VehicleKind::T72B3, tank_pos, forces);
            if let Some(t) = sim.tanks.first_mut() {
                t.at = tank_pos;
                t.hull_yaw = fd_out.sub(tank_pos).y.atan2(fd_out.sub(tank_pos).x);
                t.turret_yaw = pat.sub(tank_pos).y.atan2(pat.sub(tank_pos).x);
                t.locked_target = Some((pat, 1));
                t.lock_timer = 1.1;
            }
            sim.squads.push(Squad {
                id: 5,
                state: SquadState::Engage,
                route_idx: 0,
                wp_idx: 0,
                wp_dir: 1,
                men: vec![
                    Soldier { id: 501, at: s_pos1, floor: 0, facing: pat.sub(s_pos1).y.atan2(pat.sub(s_pos1).x), hp: 65.0, optics: true, shot_cd: 0.2, speed: 0.0, phase: 0.0, in_cover: false, vault_t: 0.0, grenade_cd: 4.0 },
                    Soldier { id: 502, at: s_pos2, floor: 0, facing: pat.sub(s_pos2).y.atan2(pat.sub(s_pos2).x), hp: 65.0, optics: false, shot_cd: 0.4, speed: 1.5, phase: 0.5, in_cover: false, vault_t: 0.0, grenade_cd: 6.0 },
                ],
                meter: 1.0,
                last_seen: pat,
                lost_for: 0.0,
                timer: 5.0,
                ignore_for: 0.0,
                held_at: s_pos1,
                shouted: true,
                hostile: true,
                stuck_t: 0.0,
                last_lead_at: s_pos1,
            });

            for (mi, sp) in spots.iter().take(2).enumerate() {
                sim.militia.push(MilitiaMember {
                    id: 30 + mi as u32,
                    name: if mi == 0 { "老周" } else { "阿敏" },
                    at: *sp,
                    floor: 1,
                    facing: s_pos1.sub(*sp).y.atan2(s_pos1.sub(*sp).x),
                    hp: 85.0,
                    coat: Rgb { r: 0.22, g: 0.26, b: 0.34 },
                    order: MilitiaOrder::Garrison(*sp),
                    fire_cd: 0.85,
                    speed: 0.0,
                    phase: 0.0,
                    in_cover: true,
                    vault_t: 0.0,
                    downed: false,
                    bleedout_s: 0.0,
                    rescuing_id: None,
                    rescue_progress: 0.0,
                });
            }

            let y2 = sim::interior::FLOOR_H + 1.35;
            let tracers = if let Some(sp0) = spots.first() {
                vec![
                    TracerFx { from: [sp0.x, y2, -sp0.y], to: [s_pos1.x, 1.3, -s_pos1.y], friendly: true },
                    TracerFx { from: [s_pos2.x, 1.35, -s_pos2.y], to: [pat.x, y2, -pat.y], friendly: false },
                ]
            } else {
                Vec::new()
            };

            clock.hour = sim.hour;
            clock.speed = 1.0;
            clock.blackout = true;
            clock.fog = sim.fog_density;
            (pat, 1.15, tracers)
        }
        2 => {
            // 03: 21:30 PlayerHome safehouse post-heating-cutoff with lit coal stove orange glow, closed curtains, barricaded steel door.
            let room_i = sim
                .interiors
                .iter()
                .position(|it| it.kind == InteriorKind::PlayerHome)
                .unwrap_or(0);
            let stove_p = sim.interiors[room_i].stove_at.unwrap_or_else(|| sim.interiors[room_i].to_world(0.5, sim.interiors[room_i].depth * 0.7));
            let pat = stove_p.lerp(sim.interiors[room_i].to_world(0.0, sim.interiors[room_i].split_v + 0.8), 0.55);
            {
                let intr = &mut sim.interiors[room_i];
                intr.curtains_closed = true;
                intr.front_door.barricade_hp = intr.front_door.max_barricade_hp;
                intr.stove_fuel_s = 92.0;
                intr.room_temp_c = 15.8;
            }

            sim.hour = hm(21, 30);
            sim.blackout = true;
            sim.alert_stage = 2;
            sim.weather_override = Some(Weather::RiverFog);
            sim.weather = Weather::RiverFog;
            sim.fog_density = 0.85;
            sim.inv.has_rifle = true;
            sim.inv.ammo = 18;
            sim.inv.has_radio = true;
            sim.player_at = pat;
            sim.player_facing = stove_p.sub(pat).y.atan2(stove_p.sub(pat).x);
            sim.player_anim = AnimState::Idle;
            sim.active_interior = Some(room_i);
            sim.cutaway_interior = Some(room_i);
            sim.current_room_temp = Some(15.8);
            sim.active_interior_name = sim.interiors[room_i].name.clone();
            sim.body.sheltered = true;
            sim.body.heated = true;
            sim.body.temp = 36.6;
            sim.objective_zh = "21:30 全城停暖 · 守住自己家安全屋与煤炉，熬过寒夜到 07:00".into();
            sim.prompt = "煤炉燃烧中（余火 92s · 按 T 添 1 煤）".into();

            clock.hour = sim.hour;
            clock.speed = 1.0;
            clock.blackout = true;
            clock.fog = sim.fog_density;
            (pat, 1.55, Vec::new())
        }
        3 | 5 => {
            // 04: noon on 18 January, the morning after; 06: 17:00 on 3 March, the
            // evenings long by then. The clock no longer stops at 07:00.
            let pat = P2::new(-262.0, -322.0);
            // Step over 07:00 once, so the morning's own text is on screen.
            sim.hour = sim::survival::DAWN_HOUR + 0.001;
            let still = sim::survival::SimInput { clock_running: true, ..Default::default() };
            sim.tick(&heihe.city, &heihe.ground, forces, &still, 0.05);
            sim.sms = None;
            sim.hour = if idx == 3 { 24.0 + hm(12, 30) } else { 24.0 * 45.0 + hm(17, 0) };
            sim.blackout = true;
            sim.weather_override = Some(Weather::Clear);
            sim.weather = Weather::Clear;
            sim.fog_density = 0.0;
            sim.player_at = pat;
            sim.player_facing = 1.9;
            sim.player_anim = AnimState::Idle;
            sim.squads.clear();
            sim.tanks.clear();

            clock.hour = sim.hour;
            clock.speed = 1.0;
            clock.blackout = true;
            clock.fog = 0.0;
            (pat, 1.0, Vec::new())
        }
        _ => {
            // 05: asleep in the basement shelter at 22:00 on 18 January.
            let room_i = sim
                .interiors
                .iter()
                .position(|it| it.kind == InteriorKind::BasementShelter)
                .unwrap_or(0);
            let stove_p = sim.interiors[room_i].stove_at.unwrap_or_else(|| sim.interiors[room_i].to_world(0.5, sim.interiors[room_i].depth * 0.7));
            let pat = stove_p.lerp(sim.interiors[room_i].to_world(0.0, sim.interiors[room_i].split_v + 0.8), 0.55);
            {
                let intr = &mut sim.interiors[room_i];
                intr.curtains_closed = true;
                intr.stove_fuel_s = 60.0;
                intr.room_temp_c = 12.0;
            }
            sim.hour = 24.0 + hm(22, 0);
            sim.mornings = 1;
            sim.dawn_survived = true;
            sim.blackout = true;
            sim.weather_override = Some(Weather::Clear);
            sim.weather = Weather::Clear;
            sim.fog_density = 0.0;
            sim.player_at = pat;
            sim.player_anim = AnimState::Idle;
            sim.active_interior = Some(room_i);
            sim.cutaway_interior = Some(room_i);
            sim.current_room_temp = Some(12.0);
            sim.active_interior_name = sim.interiors[room_i].name.clone();
            sim.body.sheltered = true;
            sim.sleeping = true;
            sim.body.asleep = true;
            sim.slept_hours = 2.4;

            clock.hour = sim.hour;
            clock.speed = 1.0;
            clock.blackout = true;
            clock.fog = 0.0;
            (pat, 1.55, Vec::new())
        }
    }
}

fn run_interior(
    mut commands: Commands,
    plan: Option<ResMut<InteriorPlan>>,
    mut clock: ResMut<Clock>,
    mut player: Single<(&mut Player, &mut Transform, &mut Mesh3d)>,
    mut eye: Single<(&mut Eye, &mut Projection)>,
    mut exit: MessageWriter<AppExit>,
    mut warm: Local<u32>,
    mut pending_tracers: Local<Vec<TracerFx>>,
    time: Res<Time>,
    capturing: Query<(), With<Capturing>>,
    heihe: Res<crate::Heihe>,
    mut play: Option<ResMut<PlayState>>,
    atlas: Option<Res<FigureAtlas>>,
) {
    let Some(mut plan) = plan else { return };
    let (Some(play), Some(atlas)) = (play.as_deref_mut(), atlas.as_deref()) else {
        return;
    };

    *warm += 1;
    let warm_up = std::env::var("NORTHEAST_SHOT_WARM")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(WARM_UP_SECS);
    if time.elapsed_secs() < warm_up || *warm < 60 {
        return;
    }
    if !capturing.is_empty() {
        return;
    }
    if plan.index >= INTERIOR_NAMES.len() {
        if plan.frames > AFTER {
            exit.write(AppExit::Success);
        }
        plan.frames += 1;
        return;
    }

    if plan.frames == 0 {
        let (pat, zoom, tracers) = configure_interior_scene(plan.index, &heihe, &play.forces, &mut play.sim, &mut clock);
        play.started = true;
        *pending_tracers = tracers;

        let (who, tf, pmesh) = &mut *player;
        who.at = pat;
        let fy = if play.sim.active_interior.is_some() {
            play.sim.player_floor as f32 * sim::interior::FLOOR_H + 0.07
        } else {
            0.0
        };
        tf.translation = Vec3::new(pat.x, fy, -pat.y);
        tf.rotation = Quat::from_rotation_y(play.sim.player_facing + std::f32::consts::FRAC_PI_2);
        if let Some(h) = atlas.meshes.get(&crate::play::AtlasKey {
            role: sim::figure::Role::Civilian,
            variant: u8::from(play.sim.body.coat_bonus > 0.0) | (u8::from(play.sim.inv.has_rifle) << 1),
            anim: play.sim.player_anim,
            frame: 0,
        }) {
            pmesh.0 = h.clone();
        }
        let (eye_c, proj) = &mut *eye;
        eye_c.zoom = zoom;
        set_zoom(proj, zoom);
    }

    if plan.frames == SETTLE - 2 && !pending_tracers.is_empty() {
        spawn_fx(&mut commands, atlas, &pending_tracers, &[]);
        pending_tracers.clear();
    }

    if plan.frames == SETTLE {
        let path = format!("{}/{}.png", plan.dir, INTERIOR_NAMES[plan.index]);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }

    plan.frames += 1;
    if plan.frames > SETTLE + AFTER {
        plan.frames = 0;
        plan.index += 1;
    }
}
