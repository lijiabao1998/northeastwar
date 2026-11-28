//! Interactive 17 January survival & civilian resistance loop in Bevy.
//!
//! Renders `sim::survival::WorldSim` and `sim::figure` meshes with zero
//! per-frame mesh allocation (`FigureAtlas`), keeping `--shots` and `--bench` fast.

use crate::{
    audio::{AudioQueue, AudioState},
    menu_ui::MenuScreen,
    to_mesh, Clock, Eye, Heihe, Mats, Player,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::Indices,
    picking::prelude::{Pickable, PointerButton, PointerClick, PointerOut, PointerOver},
    prelude::*,
    render::render_resource::PrimitiveTopology,
    window::PrimaryWindow,
};
use sim::{
    arsenal::PrimaryWeapon,
    figure::{build_figure, build_lancet_soup, build_vehicle_hull, build_vehicle_turret, pose, AnimState, FigureOpts, HeldItem, Role},
    geom::P2,
    interior::{build_curtain_soup, build_door_leaf_soup, DoorStyle, FLOOR_H},
    look::Rgb,
    soup::Soup,
    survival::{BarricadeKind, ExplosionFx, Faction, TracerFx, VehicleKind, Weather, WorldSim},
};
use std::collections::{HashMap, HashSet};

const SPEED_LEVELS: [f32; 5] = [1.0, 2.0, 4.0, 8.0, 16.0];
/// Game hours a real second while days pass at a glance: a day every two seconds.
const PASS_HOURS_PER_SECOND: f32 = 12.0;

#[derive(Clone, Copy, Debug)]
pub enum SpeedAction {
    Set(f32),
    TogglePause,
    ToggleFlashlight,
}

#[derive(Resource, Default)]
pub struct SpeedActionQueue {
    pub actions: Vec<SpeedAction>,
    pub hovered: u32,
}

#[derive(Component)]
pub struct SpeedBarRoot;

#[derive(Component)]
pub struct SpeedBtn(pub f32);

#[derive(Resource)]
pub struct Interactive {
    pub play: bool,
    pub hud: bool,
}

#[derive(Resource)]
pub struct PlayState {
    pub sim: WorldSim,
    pub started: bool,
    pub paused: bool,
    pub forces: sim::forces::Forces,
    pub cursor_world: Option<P2>,
    pub preview_barricade: bool,
    pub click_cooldown: f32,
    pub flashlight_yaw: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtlasKey {
    pub role: Role,
    pub variant: u8,
    pub anim: AnimState,
    pub frame: u8,
}

#[derive(Resource)]
pub struct FigureAtlas {
    pub meshes: HashMap<AtlasKey, Handle<Mesh>>,
    pub bar_meshes: [Handle<Mesh>; 8],
    pub bar_scenes: [Option<Handle<WorldAsset>>; 8],
    pub bar_scene_adjust: [(f32, f32); 8],
    pub cache_mesh: Handle<Mesh>,
    pub cache_scene: Option<Handle<WorldAsset>>,
    pub drop_mesh: Handle<Mesh>,
    pub drop_scene: Option<Handle<WorldAsset>>,
    pub drone_mesh: Handle<Mesh>,
    pub drone_scene: Option<Handle<WorldAsset>>,
    pub lancet_mesh: Handle<Mesh>,
    pub lancet_scene: Option<Handle<WorldAsset>>,
    pub friendly_fpv_mesh: Handle<Mesh>,
    pub tank_hull_meshes: [Handle<Mesh>; 17],
    pub tank_turret_meshes: [Handle<Mesh>; 17],
    pub tank_scenes: [Option<Handle<WorldAsset>>; 17],
    pub tank_scene_adjust: [(f32, f32); 17],
    pub door_meshes: [Handle<Mesh>; 4],
    pub curtain_closed_mesh: Handle<Mesh>,
    pub curtain_open_mesh: Handle<Mesh>,
    pub laser_mesh: Handle<Mesh>,
    pub tracer_mesh: Handle<Mesh>,
    pub burst_mesh: Handle<Mesh>,
    pub soot_mesh: Handle<Mesh>,
    pub crater_mesh: Handle<Mesh>,
    pub track_mesh: Handle<Mesh>,
    pub veh_track_mesh: Handle<Mesh>,
    pub smoke_puff_mesh: Handle<Mesh>,
    pub ember_mesh: Handle<Mesh>,
    pub wheel_mesh: Handle<Mesh>,
    pub snow_mesh: Handle<Mesh>,
    pub ring_mesh: Handle<Mesh>,
    pub circle_ring_mesh: Handle<Mesh>,
    pub fibre_mesh: Handle<Mesh>,
    pub grenade_mesh: Handle<Mesh>,
    pub blood_mesh: Handle<Mesh>,
    pub flashlight_beam_mesh: Handle<Mesh>,
    pub flashlight_bulb_mesh: Handle<Mesh>,
    pub fire_mat: Handle<StandardMaterial>,
    pub ember_mat: Handle<StandardMaterial>,
    pub wreck_charred_mat: Handle<StandardMaterial>,
    pub smoke_white_mat: Handle<StandardMaterial>,
    pub smoke_dark_mat: Handle<StandardMaterial>,
    pub xray_mat: Handle<StandardMaterial>,
    pub snow_mat: Handle<StandardMaterial>,
    pub soot_mat: Handle<StandardMaterial>,
    pub ghost_mat: Handle<StandardMaterial>,
    pub ring_mat: Handle<StandardMaterial>,
    pub entry_ring_mat: Handle<StandardMaterial>,
    pub white_ring_mat: Handle<StandardMaterial>,
    pub danger_ring_mat: Handle<StandardMaterial>,
    pub blood_mat: Handle<StandardMaterial>,
    pub flashlight_beam_mat: Handle<StandardMaterial>,
    pub flashlight_halo_mat: Handle<StandardMaterial>,
    pub flashlight_bulb_mat: Handle<StandardMaterial>,
    pub fibre_mat: Handle<StandardMaterial>,
    pub laser_mat: Handle<StandardMaterial>,
    pub pla_laser_mat: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct PedActor {
    pub idx: usize,
}

#[derive(Component)]
pub struct PooledActor;

#[derive(Component)]
pub struct PooledScene;

#[derive(Component)]
pub struct InteriorPart;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum IndoorLightKind {
    Room,
    Stove,
    Flashlight,
    FlashlightBounce,
}

#[derive(Component)]
pub struct FlashlightSpot;

pub struct InteriorVisualSet {
    pub static_solid: Entity,
    pub window_glass: Option<Entity>,
    pub window_boards: Option<Entity>,
    pub curtains: Option<Entity>,
    pub barricade_front: Option<Entity>,
    pub barricade_back: Option<Entity>,
    pub floor_built: u8,
    pub revision: u32,
}

#[derive(Resource, Default)]
pub struct InteriorVisuals {
    pub spawned: HashMap<usize, InteriorVisualSet>,
}

#[derive(Resource, Default)]
pub struct ActorPools {
    soldiers: HashMap<u32, Entity>,
    militia: HashMap<u32, Entity>,
    caches: HashMap<u32, Entity>,
    drops: HashMap<u32, Entity>,
    tracks: HashMap<u32, Entity>,
    room_npcs: HashMap<u32, Entity>,
    room_loots: HashMap<u32, Entity>,
    live_doors: HashMap<u32, Entity>,
    barricades: Vec<(Entity, BarricadeKind)>,
    drone: Option<Entity>,
    fpv: Option<Entity>,
    fpv_fibre: Option<Entity>,
    lancet: Option<Entity>,
    friendly_fpv: Option<Entity>,
    tank_hulls: HashMap<u32, Entity>,
    tank_turrets: HashMap<u32, Entity>,
    tank_lasers: HashMap<u32, Entity>,
    ghost_preview: Option<Entity>,
    door_ring: Option<Entity>,
    entry_ring: Option<Entity>,
    stair_ring: Option<Entity>,
    craters: HashMap<u32, Entity>,
    wrecks_hull: HashMap<u32, Entity>,
    wrecks_turret: HashMap<u32, Entity>,
    wrecks_ember: HashMap<u32, Entity>,
    wrecks_wheel: HashMap<u32, Entity>,
    smoke_clouds: HashMap<u32, Entity>,
    xray_silhouettes: HashMap<u32, Entity>,
    militia_rings: HashMap<u32, Entity>,
    cmd_ring: Option<Entity>,
    danger_rings: HashMap<u32, Entity>,
    grenades: HashMap<u32, Entity>,
    blood_marks: HashMap<u32, Entity>,
    flashlight_beam: Option<Entity>,
    flashlight_halo: Option<Entity>,
    flashlight_bulb: Option<Entity>,
}

#[derive(Component)]
pub struct TransientFx {
    ttl: f32,
    vel: Vec3,
    grow: f32,
}

#[derive(Component)]
pub struct SnowParticle {
    offset: Vec3,
    speed: f32,
    phase: f32,
}

#[derive(Component)]
pub struct SootMark;

#[derive(Component)]
pub struct PauseBtnText;

#[derive(Component)]
pub struct FlashlightBtnText;

#[derive(Component)]
pub enum HudRole {
    TopRight,
    Notice,
    Prompt,
    RadioFeed,
    Stats,
    Sms,
    Death,
    Objective,
}

pub fn install(app: &mut App, interactive: bool, combat_shots: bool) {
    let forces_path = sim::data_dir().join("russia/forces.json");
    let forces = sim::forces::Forces::from_json(&std::fs::read_to_string(forces_path).expect("forces.json"))
        .expect("valid forces.json");

    let show_hud = interactive || combat_shots;
    app.insert_resource(Interactive { play: interactive, hud: show_hud })
        .insert_resource(PlayStateSeed(forces))
        .init_resource::<ActorPools>()
        .init_resource::<SpeedActionQueue>()
        .add_systems(Startup, setup_play.after(crate::setup));

    if interactive {
        app.add_systems(Update, step_play.after(crate::tick).before(crate::follow))
            .add_systems(Update, (sync_actors, sync_fx, sync_hud, autosave).after(step_play));
    } else if combat_shots {
        app.add_systems(Update, (sync_actors, sync_fx, sync_hud));
    }
}

#[derive(Resource)]
pub struct PlayStateSeed(pub sim::forces::Forces);

fn build_prop_soup(size: [f32; 3], col: Rgb) -> Soup {
    let mut s = Soup::default();
    s.boxed(P2::default(), 0.0, size, 0.0, col);
    s
}

/// Smooth 12-sided, 4-ring tapered cone extending along `-Z` (`0` -> `-1`) with per-vertex RGBA
/// attenuation so `AlphaMode::Add` creates a feathered volumetric light cone with no hard edges.
fn build_flashlight_beam_mesh() -> Mesh {
    const SIDES: usize = 12;
    const RINGS: [(f32, f32, [f32; 4]); 4] = [
        (0.0, 0.09, [1.0, 1.0, 1.0, 1.0]),
        (-0.35, 0.38, [0.42, 0.45, 0.48, 0.45]),
        (-0.70, 0.70, [0.12, 0.14, 0.16, 0.15]),
        (-1.0, 1.0, [0.0, 0.0, 0.0, 0.0]),
    ];
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(SIDES * RINGS.len());
    let mut nor: Vec<[f32; 3]> = Vec::with_capacity(SIDES * RINGS.len());
    let mut uv: Vec<[f32; 2]> = Vec::with_capacity(SIDES * RINGS.len());
    let mut col: Vec<[f32; 4]> = Vec::with_capacity(SIDES * RINGS.len());
    let mut idx: Vec<u32> = Vec::with_capacity(SIDES * (RINGS.len() - 1) * 6);

    for (ri, &(z, radius, rgba)) in RINGS.iter().enumerate() {
        let v = ri as f32 / (RINGS.len() - 1) as f32;
        for si in 0..SIDES {
            let a = si as f32 * std::f32::consts::TAU / SIDES as f32;
            let (sa, ca) = (a.sin(), a.cos());
            pos.push([ca * radius, sa * radius, z]);
            nor.push([ca, sa, 0.25]);
            uv.push([si as f32 / SIDES as f32, v]);
            col.push(rgba);
        }
    }
    for ri in 0..(RINGS.len() - 1) {
        let base0 = (ri * SIDES) as u32;
        let base1 = ((ri + 1) * SIDES) as u32;
        for si in 0..SIDES {
            let s0 = si as u32;
            let s1 = ((si + 1) % SIDES) as u32;
            idx.extend_from_slice(&[base0 + s0, base0 + s1, base1 + s1]);
            idx.extend_from_slice(&[base0 + s0, base1 + s1, base1 + s0]);
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nor)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
        .with_inserted_indices(Indices::U32(idx))
}

fn build_ring_soup() -> Soup {
    let mut s = Soup::default();
    let col = Rgb { r: 1.0, g: 0.85, b: 0.35 };
    let r = 1.4;
    s.boxed(P2::new(0.0, r), 0.0, [2.8, 0.05, 0.14], 0.0, col);
    s.boxed(P2::new(0.0, -r), 0.0, [2.8, 0.05, 0.14], 0.0, col);
    s.boxed(P2::new(r, 0.0), 0.0, [0.14, 0.05, 2.8], 0.0, col);
    s.boxed(P2::new(-r, 0.0), 0.0, [0.14, 0.05, 2.8], 0.0, col);
    s
}

fn build_circle_ring_soup(radius: f32, thickness: f32) -> Soup {
    let mut s = Soup::default();
    let col = Rgb { r: 1.0, g: 1.0, b: 1.0 };
    let r_in = (radius - thickness * 0.5).max(0.05);
    let r_out = radius + thickness * 0.5;
    let y = 0.05;
    let segs = 28usize;
    for i in 0..segs {
        let a0 = i as f32 * std::f32::consts::TAU / segs as f32;
        let a1 = (i + 1) as f32 * std::f32::consts::TAU / segs as f32;
        let o0 = [a0.cos() * r_out, y, -a0.sin() * r_out];
        let o1 = [a1.cos() * r_out, y, -a1.sin() * r_out];
        let i1 = [a1.cos() * r_in, y, -a1.sin() * r_in];
        let i0 = [a0.cos() * r_in, y, -a0.sin() * r_in];
        s.quad([o0, o1, i1, i0], [0.0, 1.0, 0.0], col);
    }
    s
}

fn build_crater_soup() -> Soup {
    let mut s = Soup::default();
    let pit_col = Rgb { r: 0.07, g: 0.06, b: 0.06 };
    let inner_rim = Rgb { r: 0.22, g: 0.17, b: 0.13 };
    let mud_rim = Rgb { r: 0.34, g: 0.28, b: 0.22 };
    let segs = 14usize;
    for i in 0..segs {
        let a0 = i as f32 * std::f32::consts::TAU / segs as f32;
        let a1 = (i + 1) as f32 * std::f32::consts::TAU / segs as f32;
        let p = |a: f32, r: f32, y: f32| [a.cos() * r, y, -a.sin() * r];
        // Blackened center pit
        s.tri([[0.0, 0.015, 0.0], p(a0, 0.58, 0.02), p(a1, 0.58, 0.02)], [0.0, 1.0, 0.0], pit_col);
        // Inner scorched crater wall
        s.quad(
            [p(a0, 0.85, 0.14), p(a1, 0.85, 0.14), p(a1, 0.58, 0.02), p(a0, 0.58, 0.02)],
            [0.0, 1.0, 0.0],
            inner_rim,
        );
        // Outer raised muddy-snow rim
        s.quad(
            [p(a0, 1.12, 0.02), p(a1, 1.12, 0.02), p(a1, 0.85, 0.14), p(a0, 0.85, 0.14)],
            [0.0, 1.0, 0.0],
            mud_rim,
        );
    }
    s
}

fn build_smoke_soup() -> Soup {
    let mut s = Soup::default();
    let white = Rgb { r: 0.94, g: 0.96, b: 0.98 };
    s.boxed(P2::default(), 0.25, [1.5, 1.15, 1.5], 0.0, white);
    s.boxed(P2::new(-0.52, 0.32), 0.15, [1.15, 0.95, 1.15], 0.4, white);
    s.boxed(P2::new(0.48, -0.28), 0.20, [1.20, 1.05, 1.10], -0.35, white);
    s.boxed(P2::new(0.08, 0.12), 1.05, [1.10, 0.85, 1.05], 0.2, white);
    s
}

fn build_veh_track_soup() -> Soup {
    let mut s = Soup::default();
    let rut_col = Rgb { r: 0.30, g: 0.33, b: 0.38 };
    s.boxed(P2::new(-0.92, 0.0), 0.0, [0.42, 0.03, 1.25], 0.0, rut_col);
    s.boxed(P2::new(0.92, 0.0), 0.0, [0.42, 0.03, 1.25], 0.0, rut_col);
    s
}

fn build_drone_soup() -> Soup {
    let mut s = Soup::default();
    let dark = Rgb { r: 0.10, g: 0.10, b: 0.12 };
    s.boxed(P2::default(), 0.0, [0.9, 0.18, 0.9], 0.0, dark);
    s.boxed(P2::default(), 0.04, [1.8, 0.06, 0.14], 0.78, dark);
    s.boxed(P2::default(), 0.04, [1.8, 0.06, 0.14], -0.78, dark);
    s.boxed(P2::new(-0.7, 0.0), 0.10, [0.18, 0.12, 0.18], 0.0, Rgb { r: 1.0, g: 0.15, b: 0.1 });
    s.boxed(P2::new(0.7, 0.0), 0.10, [0.18, 0.12, 0.18], 0.0, Rgb { r: 0.15, g: 1.0, b: 0.3 });
    s
}

fn build_friendly_fpv_soup() -> Soup {
    let mut s = Soup::default();
    let olive = Rgb { r: 0.14, g: 0.20, b: 0.14 };
    let led = Rgb { r: 0.15, g: 0.95, b: 0.90 };
    s.boxed(P2::default(), 0.0, [0.9, 0.18, 0.9], 0.0, olive);
    s.boxed(P2::default(), 0.04, [1.8, 0.06, 0.14], 0.78, olive);
    s.boxed(P2::default(), 0.04, [1.8, 0.06, 0.14], -0.78, olive);
    s.boxed(P2::new(-0.7, 0.0), 0.10, [0.18, 0.12, 0.18], 0.0, led);
    s.boxed(P2::new(0.7, 0.0), 0.10, [0.18, 0.12, 0.18], 0.0, led);
    s
}

fn add_tilted_box(s: &mut Soup, center: Vec3, euler: [f32; 3], size: [f32; 3], col: Rgb) {
    let rot = Quat::from_euler(EulerRot::XYZ, euler[0], euler[1], euler[2]);
    let (hx, hy, hz) = (size[0] * 0.5, size[1] * 0.5, size[2] * 0.5);
    let pt = |sx: f32, sy: f32, sz: f32| -> [f32; 3] {
        (center + rot * Vec3::new(sx * hx, sy * hy, sz * hz)).to_array()
    };
    let dir = |nx: f32, ny: f32, nz: f32| -> [f32; 3] { (rot * Vec3::new(nx, ny, nz)).to_array() };
    let faces = [
        (dir(0.0, 0.0, 1.0), [pt(-1.0, -1.0, 1.0), pt(1.0, -1.0, 1.0), pt(1.0, 1.0, 1.0), pt(-1.0, 1.0, 1.0)]),
        (dir(0.0, 0.0, -1.0), [pt(1.0, -1.0, -1.0), pt(-1.0, -1.0, -1.0), pt(-1.0, 1.0, -1.0), pt(1.0, 1.0, -1.0)]),
        (dir(1.0, 0.0, 0.0), [pt(1.0, -1.0, 1.0), pt(1.0, -1.0, -1.0), pt(1.0, 1.0, -1.0), pt(1.0, 1.0, 1.0)]),
        (dir(-1.0, 0.0, 0.0), [pt(-1.0, -1.0, -1.0), pt(-1.0, -1.0, 1.0), pt(-1.0, 1.0, 1.0), pt(-1.0, 1.0, -1.0)]),
        (dir(0.0, 1.0, 0.0), [pt(-1.0, 1.0, 1.0), pt(1.0, 1.0, 1.0), pt(1.0, 1.0, -1.0), pt(-1.0, 1.0, -1.0)]),
        (dir(0.0, -1.0, 0.0), [pt(-1.0, -1.0, -1.0), pt(1.0, -1.0, -1.0), pt(1.0, -1.0, 1.0), pt(-1.0, -1.0, 1.0)]),
    ];
    for (n, c) in faces {
        s.quad(c, n, col);
    }
}

fn build_barricade_soup(kind: BarricadeKind) -> Soup {
    let mut s = Soup::default();
    match kind {
        BarricadeKind::Sandbag => {
            let col = Rgb { r: 0.45, g: 0.37, b: 0.25 };
            s.boxed(P2::new(-0.5, 0.0), 0.0, [0.75, 0.36, 0.48], 0.0, col);
            s.boxed(P2::new(0.3, 0.0), 0.0, [0.75, 0.36, 0.48], 0.0, col);
            s.boxed(P2::new(-0.1, 0.0), 0.36, [0.85, 0.32, 0.42], 0.0, col);
        }
        BarricadeKind::IronGate => {
            let iron = Rgb { r: 0.18, g: 0.20, b: 0.22 };
            s.boxed(P2::new(-1.0, 0.0), 0.0, [0.12, 1.8, 0.12], 0.0, iron);
            s.boxed(P2::new(1.0, 0.0), 0.0, [0.12, 1.8, 0.12], 0.0, iron);
            for y in [0.35, 0.85, 1.35] {
                s.boxed(P2::default(), y, [2.0, 0.08, 0.08], 0.0, iron);
            }
        }
        BarricadeKind::FireBarrel => {
            s.boxed(P2::default(), 0.0, [0.68, 0.85, 0.68], 0.0, Rgb { r: 0.20, g: 0.13, b: 0.10 });
            s.boxed(P2::default(), 0.85, [0.46, 0.18, 0.46], 0.0, Rgb { r: 0.95, g: 0.42, b: 0.08 });
        }
        BarricadeKind::DeadCar => {
            s.boxed(P2::default(), 0.0, [3.6, 0.65, 1.7], 0.0, Rgb { r: 0.56, g: 0.46, b: 0.14 });
            s.boxed(P2::new(-0.2, 0.0), 0.65, [1.6, 0.55, 1.45], 0.0, Rgb { r: 0.66, g: 0.56, b: 0.20 });
        }
        BarricadeKind::Concrete => {
            s.boxed(P2::default(), 0.0, [2.3, 0.48, 0.78], 0.0, Rgb { r: 0.52, g: 0.52, b: 0.50 });
            s.boxed(P2::default(), 0.48, [2.0, 0.38, 0.44], 0.0, Rgb { r: 0.60, g: 0.60, b: 0.58 });
        }
        BarricadeKind::Trench => {
            let pit = Rgb { r: 0.09, g: 0.08, b: 0.07 };
            let wood = Rgb { r: 0.32, g: 0.24, b: 0.16 };
            let earth = Rgb { r: 0.26, g: 0.22, b: 0.17 };
            let snow = Rgb { r: 0.78, g: 0.82, b: 0.86 };
            let bag = Rgb { r: 0.45, g: 0.37, b: 0.25 };
            s.boxed(P2::default(), 0.0, [4.4, 0.04, 1.4], 0.0, pit);
            for z in [-0.72f32, 0.72] {
                s.boxed(P2::new(0.0, z), 0.0, [4.5, 0.34, 0.14], 0.0, wood);
            }
            for z in [-1.04f32, 1.04] {
                s.boxed(P2::new(0.0, z), 0.0, [4.8, 0.44, 0.56], 0.0, earth);
                s.boxed(P2::new(0.0, z), 0.44, [4.7, 0.10, 0.50], 0.0, snow);
            }
            for x in [-1.45f32, 1.45] {
                s.boxed(P2::new(x, -0.78), 0.44, [0.95, 0.22, 0.38], 0.0, bag);
            }
        }
        BarricadeKind::TarpDugout => {
            let pit = Rgb { r: 0.08, g: 0.07, b: 0.06 };
            let wood = Rgb { r: 0.28, g: 0.20, b: 0.12 };
            let earth = Rgb { r: 0.26, g: 0.22, b: 0.17 };
            let snow = Rgb { r: 0.78, g: 0.82, b: 0.86 };
            let olive = Rgb { r: 0.24, g: 0.29, b: 0.21 };
            let tan = Rgb { r: 0.38, g: 0.34, b: 0.25 };
            s.boxed(P2::default(), 0.0, [3.4, 0.04, 1.5], 0.0, pit);
            for z in [-1.02f32, 1.02] {
                s.boxed(P2::new(0.0, z), 0.0, [3.8, 0.42, 0.52], 0.0, earth);
                s.boxed(P2::new(0.0, z), 0.42, [3.7, 0.08, 0.46], 0.0, snow);
            }
            for x in [-1.62f32, 1.62] {
                for z in [-0.88f32, 0.88] {
                    s.boxed(P2::new(x, z), 0.0, [0.12, 1.78, 0.12], 0.0, wood);
                }
            }
            add_tilted_box(&mut s, Vec3::new(0.0, 1.76, 0.0), [0.12, 0.0, 0.0], [3.75, 0.10, 2.25], olive);
            add_tilted_box(&mut s, Vec3::new(-0.45, 1.83, -0.15), [0.12, 0.0, 0.0], [1.95, 0.06, 1.15], snow);
            add_tilted_box(&mut s, Vec3::new(0.95, 1.82, 0.25), [0.12, 0.0, 0.0], [1.05, 0.06, 0.85], tan);
        }
        BarricadeKind::BarbedWire => {
            let steel = Rgb { r: 0.22, g: 0.24, b: 0.26 };
            let wire = Rgb { r: 0.34, g: 0.36, b: 0.38 };
            for hx in [-1.45f32, 0.0, 1.45] {
                add_tilted_box(&mut s, Vec3::new(hx, 0.52, 0.0), [0.72, 0.35, 0.72], [0.10, 1.38, 0.10], steel);
                add_tilted_box(&mut s, Vec3::new(hx, 0.52, 0.0), [-0.72, 0.35, -0.72], [0.10, 1.38, 0.10], steel);
                add_tilted_box(&mut s, Vec3::new(hx, 0.52, 0.0), [0.68, -0.45, -0.68], [0.10, 1.38, 0.10], steel);
            }
            for wy in [0.30, 0.65, 0.95] {
                s.boxed(P2::default(), wy, [3.8, 0.05, 0.05], 0.0, wire);
            }
        }
    }
    s
}

fn cache_poses(
    meshes: &mut Assets<Mesh>,
    out: &mut HashMap<AtlasKey, Handle<Mesh>>,
    role: Role,
    variant: u8,
    opts: FigureOpts,
    cycles: &[(AnimState, f32, u8)],
) {
    for &(anim, speed, frames) in cycles {
        for f in 0..frames {
            let phase = (f as f32 / frames.max(1) as f32) * std::f32::consts::TAU;
            let p = pose(anim, speed, phase, 0.0);
            let soup = build_figure(&opts, &p);
            out.insert(AtlasKey { role, variant, anim, frame: f }, meshes.add(to_mesh(&soup)));
        }
    }
}

fn load_hq_scene(assets_dir: &std::path::Path, assets: &AssetServer, rel: &str) -> Option<Handle<WorldAsset>> {
    assets_dir
        .join(rel)
        .is_file()
        .then(|| assets.load(GltfAssetLabel::Scene(0).from_asset(rel.to_string())))
}

pub fn setup_play(
    mut commands: Commands,
    heihe: Res<Heihe>,
    mats: Res<Mats>,
    seed: Res<PlayStateSeed>,
    interactive: Res<Interactive>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_q: Single<(Entity, &mut Mesh3d), With<Player>>,
    mut clock: ResMut<Clock>,
) {
    let mut map = HashMap::new();
    let player_cycles = [
        (AnimState::Idle, 0.0, 1),
        (AnimState::Walk, 3.4, 8),
        (AnimState::Run, 6.0, 8),
        (AnimState::Aim, 0.0, 1),
        (AnimState::Shoot, 0.0, 1),
        (AnimState::Loot, 0.0, 1),
        (AnimState::HandsUp, 0.0, 1),
        (AnimState::Crouch, 0.0, 1),
        (AnimState::Prone, 0.0, 1),
        (AnimState::Vault, 0.0, 1),
        (AnimState::Dead, 0.0, 1),
    ];
    for v in 0..4u8 {
        let coat_upgrade = v & 1 != 0;
        let has_rifle = v & 2 != 0;
        let opts = FigureOpts {
            role: Role::Civilian,
            coat: Rgb { r: 0.08, g: 0.09, b: 0.11 },
            pants: Rgb { r: 0.12, g: 0.14, b: 0.18 },
            item: if has_rifle { HeldItem::Rifle } else { HeldItem::None },
            female: false,
            coat_upgrade,
            optics: false,
        };
        cache_poses(&mut meshes, &mut map, Role::Civilian, v, opts, &player_cycles);
    }

    let ped_coats = [
        Rgb { r: 0.10, g: 0.10, b: 0.12 },
        Rgb { r: 0.62, g: 0.12, b: 0.12 },
        Rgb { r: 0.78, g: 0.62, b: 0.38 },
        Rgb { r: 0.18, g: 0.26, b: 0.42 },
        Rgb { r: 0.45, g: 0.16, b: 0.18 },
    ];
    for (ci, coat) in ped_coats.into_iter().enumerate() {
        let opts = FigureOpts { role: Role::Civilian, coat, female: ci % 3 == 1, ..Default::default() };
        cache_poses(&mut meshes, &mut map, Role::Civilian, 10 + ci as u8, opts, &[(AnimState::Idle, 0.0, 1), (AnimState::Walk, 1.8, 6), (AnimState::Run, 3.8, 4)]);
    }

    let militia_coats = [
        Rgb { r: 0.12, g: 0.12, b: 0.14 },
        Rgb { r: 0.18, g: 0.22, b: 0.28 },
        Rgb { r: 0.42, g: 0.16, b: 0.16 },
        Rgb { r: 0.16, g: 0.24, b: 0.34 },
    ];
    for (ci, coat) in militia_coats.into_iter().enumerate() {
        let opts = FigureOpts { role: Role::Militia, coat, item: HeldItem::Rifle, ..Default::default() };
        cache_poses(
            &mut meshes,
            &mut map,
            Role::Militia,
            ci as u8,
            opts,
            &[
                (AnimState::Idle, 0.0, 1),
                (AnimState::Walk, 2.5, 6),
                (AnimState::Aim, 0.0, 1),
                (AnimState::Crouch, 0.0, 1),
                (AnimState::Prone, 0.0, 1),
                (AnimState::Vault, 0.0, 1),
                (AnimState::Dead, 0.0, 1),
            ],
        );
    }

    for optics in [false, true] {
        let opts = FigureOpts { role: Role::RussianSoldier, item: HeldItem::Rifle, optics, ..Default::default() };
        cache_poses(
            &mut meshes,
            &mut map,
            Role::RussianSoldier,
            u8::from(optics),
            opts,
            &[
                (AnimState::Idle, 0.0, 1),
                (AnimState::Walk, 1.5, 6),
                (AnimState::Run, 3.1, 4),
                (AnimState::Aim, 0.0, 1),
                (AnimState::Crouch, 0.0, 1),
                (AnimState::Vault, 0.0, 1),
                (AnimState::Dead, 0.0, 1),
            ],
        );
    }

    // Replace default static player mesh with articulated figure mesh!
    let (_pe, mut pmesh) = player_q.into_inner();
    if let Some(h) = map.get(&AtlasKey { role: Role::Civilian, variant: 0, anim: AnimState::Idle, frame: 0 }) {
        pmesh.0 = h.clone();
    }

    let fire_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.65, 0.2),
        emissive: LinearRgba::new(16.0, 6.5, 1.4, 1.0),
        unlit: true,
        ..default()
    });
    let ember_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.38, 0.08),
        emissive: LinearRgba::new(14.0, 4.2, 0.6, 1.0),
        unlit: true,
        ..default()
    });
    let wreck_charred_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.11, 0.12),
        perceptual_roughness: 0.96,
        ..default()
    });
    let smoke_white_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.90, 0.93, 0.96, 0.56),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.95,
        ..default()
    });
    let smoke_dark_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.16, 0.17, 0.19, 0.64),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.98,
        ..default()
    });
    let xray_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.30, 0.80, 1.0, 0.62),
        emissive: LinearRgba::new(0.45, 1.75, 3.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let snow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.96, 0.98, 1.0, 0.88),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let soot_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.11, 0.10, 0.09), perceptual_roughness: 0.98, ..default() });
    let ghost_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.78, 0.28, 0.42),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.88, 0.35, 0.78),
        emissive: LinearRgba::new(2.4, 1.9, 0.6, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let entry_ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.32, 0.90, 0.96, 0.85),
        emissive: LinearRgba::new(0.85, 2.5, 2.8, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let white_ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.98, 1.0, 0.86),
        emissive: LinearRgba::new(1.4, 1.6, 1.8, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let danger_ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.16, 0.14, 0.90),
        emissive: LinearRgba::new(3.2, 0.4, 0.3, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let blood_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.62, 0.04, 0.05, 0.85),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.72,
        ..default()
    });
    let flashlight_beam_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.32, 0.38, 0.46, 1.0),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        cull_mode: None,
        fog_enabled: false,
        ..default()
    });
    let flashlight_halo_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.16, 0.20, 0.26, 1.0),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        cull_mode: None,
        fog_enabled: false,
        ..default()
    });
    let flashlight_bulb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.96, 0.98, 1.0),
        emissive: LinearRgba::new(14.0, 16.0, 18.0, 1.0),
        unlit: true,
        fog_enabled: false,
        ..default()
    });
    let fibre_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.96, 1.0),
        emissive: LinearRgba::new(2.2, 2.8, 3.2, 1.0),
        unlit: true,
        ..default()
    });
    let laser_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.16, 0.12),
        emissive: LinearRgba::new(10.0, 1.2, 0.8, 1.0),
        unlit: true,
        ..default()
    });
    let pla_laser_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.96, 0.45),
        emissive: LinearRgba::new(0.8, 10.0, 4.5, 1.0),
        unlit: true,
        ..default()
    });

    let assets_dir = sim::data_dir()
        .parent()
        .map(|p| p.join("assets"))
        .unwrap_or_else(|| std::path::PathBuf::from("assets"));

    let atlas = FigureAtlas {
        meshes: map,
        bar_meshes: [
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::Sandbag))),
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::IronGate))),
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::FireBarrel))),
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::DeadCar))),
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::Concrete))),
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::Trench))),
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::TarpDugout))),
            meshes.add(to_mesh(&build_barricade_soup(BarricadeKind::BarbedWire))),
        ],
        bar_scenes: [
            load_hq_scene(&assets_dir, &assets, "models/colab_hq/bar_sandbag.glb"),
            load_hq_scene(&assets_dir, &assets, "models/colab_hq/bar_iron_gate.glb"),
            load_hq_scene(&assets_dir, &assets, "models/colab_hq/bar_fire_barrel.glb"),
            load_hq_scene(&assets_dir, &assets, "models/colab_hq/bar_dead_car.glb"),
            load_hq_scene(&assets_dir, &assets, "models/colab_hq/bar_concrete.glb"),
            load_hq_scene(&assets_dir, &assets, "models/colab_hq/bar_trench.glb"),
            load_hq_scene(&assets_dir, &assets, "models/colab_hq/bar_tarp_dugout.glb"),
            None, // bar_barbed_wire.glb renders as a tall billboard/gate frame; keep procedural hedgehogs + wire
        ],
        bar_scene_adjust: [
            (0.0, 0.82),
            (-std::f32::consts::FRAC_PI_2, 1.0),
            (0.0, 1.0),
            (-std::f32::consts::FRAC_PI_2, 1.0),
            (-std::f32::consts::FRAC_PI_2, 1.0),
            (-std::f32::consts::FRAC_PI_2, 0.78),
            (-std::f32::consts::FRAC_PI_2, 0.88),
            (-std::f32::consts::FRAC_PI_2, 0.55),
        ],
        cache_mesh: meshes.add(to_mesh(&build_prop_soup([0.9, 0.72, 0.65], Rgb { r: 0.18, g: 0.56, b: 0.28 }))),
        cache_scene: load_hq_scene(&assets_dir, &assets, "models/colab_hq/supply_cache.glb"),
        drop_mesh: meshes.add(to_mesh(&build_prop_soup([0.55, 0.25, 0.42], Rgb { r: 0.34, g: 0.36, b: 0.22 }))),
        drop_scene: load_hq_scene(&assets_dir, &assets, "models/colab_hq/loot_drop.glb"),
        drone_mesh: meshes.add(to_mesh(&build_drone_soup())),
        drone_scene: load_hq_scene(&assets_dir, &assets, "models/colab_hq/air_drone_quad.glb"),
        lancet_mesh: meshes.add(to_mesh(&build_lancet_soup())),
        lancet_scene: load_hq_scene(&assets_dir, &assets, "models/colab_hq/air_lancet.glb"),
        friendly_fpv_mesh: meshes.add(to_mesh(&build_friendly_fpv_soup())),
        tank_hull_meshes: std::array::from_fn(|i| meshes.add(to_mesh(&build_vehicle_hull(VehicleKind::ALL[i])))),
        tank_turret_meshes: std::array::from_fn(|i| meshes.add(to_mesh(&build_vehicle_turret(VehicleKind::ALL[i])))),
        tank_scenes: std::array::from_fn(|i| {
            let rel = match VehicleKind::ALL[i] {
                VehicleKind::T72B3 | VehicleKind::T80Bvm | VehicleKind::T90M | VehicleKind::T62M => Some("models/colab_hq/veh_t90m.glb"),
                // veh_tos1a.glb has pale untextured grey albedo and oversized X-aligned block geometry; keep procedural hull/turret
                VehicleKind::Tos1A | VehicleKind::Bmpt => None,
                VehicleKind::Pgz09 | VehicleKind::Zbd04A => Some("models/colab_hq/veh_zbd04a.glb"),
                VehicleKind::Bmp3 | VehicleKind::Bmd4M => Some("models/colab_hq/veh_bmp3.glb"),
                VehicleKind::Btr82A => Some("models/colab_hq/veh_btr82a.glb"),
                VehicleKind::TigrM | VehicleKind::Csk181 => Some("models/colab_hq/veh_mengshi.glb"),
                VehicleKind::Ztz99A | VehicleKind::Ztz96B | VehicleKind::Ztq15 => Some("models/colab_hq/veh_ztz99a.glb"),
                VehicleKind::Zbl08 => Some("models/colab_hq/veh_zbl08.glb"),
            };
            rel.and_then(|r| load_hq_scene(&assets_dir, &assets, r))
        }),
        tank_scene_adjust: std::array::from_fn(|i| match VehicleKind::ALL[i] {
            VehicleKind::Ztz99A | VehicleKind::Ztz96B => (0.0, 0.85),
            VehicleKind::Ztq15 => (0.0, 0.78),
            VehicleKind::T90M | VehicleKind::T80Bvm | VehicleKind::T72B3 => (0.0, 0.88),
            VehicleKind::T62M => (0.0, 0.82),
            VehicleKind::Pgz09 | VehicleKind::Zbd04A => (0.0, 0.62),
            VehicleKind::Bmp3 | VehicleKind::Bmd4M => (0.0, 0.78),
            VehicleKind::Btr82A => (0.0, 0.76),
            VehicleKind::Zbl08 => (0.0, 0.68),
            VehicleKind::TigrM => (0.0, 0.78),
            VehicleKind::Csk181 => (0.0, 0.82),
            _ => (0.0, 1.0),
        }),
        door_meshes: [
            meshes.add(to_mesh(&build_door_leaf_soup(DoorStyle::WoodFlat, 1.4))),
            meshes.add(to_mesh(&build_door_leaf_soup(DoorStyle::ShopGlass, 1.4))),
            meshes.add(to_mesh(&build_door_leaf_soup(DoorStyle::SteelSecurity, 1.4))),
            meshes.add(to_mesh(&build_door_leaf_soup(DoorStyle::WoodFlat, 1.4))),
        ],
        curtain_closed_mesh: meshes.add(to_mesh(&build_curtain_soup(false, 1.4))),
        curtain_open_mesh: meshes.add(to_mesh(&build_curtain_soup(true, 1.4))),
        laser_mesh: meshes.add(to_mesh(&build_prop_soup([0.05, 0.05, 1.0], Rgb { r: 1.0, g: 0.18, b: 0.14 }))),
        tracer_mesh: meshes.add(to_mesh(&build_prop_soup([0.05, 0.05, 1.0], Rgb { r: 1.0, g: 0.85, b: 0.4 }))),
        burst_mesh: meshes.add(to_mesh(&build_prop_soup([1.5, 1.5, 1.5], Rgb { r: 1.0, g: 0.45, b: 0.1 }))),
        soot_mesh: meshes.add(to_mesh(&build_prop_soup([4.8, 0.04, 4.8], Rgb { r: 0.12, g: 0.10, b: 0.09 }))),
        crater_mesh: meshes.add(to_mesh(&build_crater_soup())),
        track_mesh: meshes.add(to_mesh(&build_prop_soup([0.28, 0.03, 0.50], Rgb { r: 0.52, g: 0.60, b: 0.72 }))),
        veh_track_mesh: meshes.add(to_mesh(&build_veh_track_soup())),
        smoke_puff_mesh: meshes.add(to_mesh(&build_smoke_soup())),
        ember_mesh: meshes.add(to_mesh(&build_prop_soup([1.45, 0.65, 1.85], Rgb { r: 1.0, g: 0.42, b: 0.10 }))),
        wheel_mesh: meshes.add(to_mesh(&build_prop_soup([0.36, 0.82, 0.82], Rgb { r: 0.10, g: 0.11, b: 0.12 }))),
        snow_mesh: meshes.add(to_mesh(&build_prop_soup([0.09, 0.09, 0.09], Rgb { r: 1.0, g: 1.0, b: 1.0 }))),
        ring_mesh: meshes.add(to_mesh(&build_ring_soup())),
        circle_ring_mesh: meshes.add(to_mesh(&build_circle_ring_soup(1.0, 0.09))),
        fibre_mesh: meshes.add(to_mesh(&build_prop_soup([0.04, 0.04, 1.0], Rgb { r: 0.88, g: 0.96, b: 1.0 }))),
        grenade_mesh: meshes.add(to_mesh(&build_prop_soup([0.16, 0.22, 0.16], Rgb { r: 0.22, g: 0.30, b: 0.16 }))),
        blood_mesh: meshes.add(to_mesh(&build_prop_soup([0.52, 0.025, 0.68], Rgb { r: 0.58, g: 0.04, b: 0.05 }))),
        flashlight_beam_mesh: meshes.add(build_flashlight_beam_mesh()),
        flashlight_bulb_mesh: meshes.add(to_mesh(&build_prop_soup([0.16, 0.16, 0.16], Rgb { r: 1.0, g: 1.0, b: 1.0 }))),
        fire_mat,
        ember_mat,
        wreck_charred_mat,
        smoke_white_mat,
        smoke_dark_mat,
        xray_mat,
        snow_mat,
        soot_mat,
        ghost_mat,
        ring_mat,
        entry_ring_mat,
        white_ring_mat,
        danger_ring_mat,
        blood_mat,
        flashlight_beam_mat,
        flashlight_halo_mat,
        flashlight_bulb_mat,
        fibre_mat,
        laser_mat,
        pla_laser_mat,
    };

    let mut sim = WorldSim::new(&heihe.city);
    // Batch all exterior doorway frames; live doors/curtains render dynamically when interactive HUD is active.
    let mut portals = Soup::default();
    for intr in &sim.interiors {
        intr.append_exterior_portal(&heihe.palette, &mut portals, !interactive.hud);
    }
    for chunk in portals.chunks(120.0) {
        commands.spawn((
            Mesh3d(meshes.add(to_mesh(&chunk))),
            MeshMaterial3d(mats.matte.clone()),
        ));
    }
    commands.insert_resource(InteriorVisuals::default());

    // Spawn ambient pedestrians on sidewalks (visible both in --shots and interactive play!).
    for (idx, ped) in sim.pedestrians.iter().enumerate() {
        let variant = 10 + (idx % 5) as u8;
        if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::Civilian, variant, anim: AnimState::Walk, frame: 0 }) {
            commands.spawn((
                Mesh3d(h.clone()),
                MeshMaterial3d(mats.matte.clone()),
                Transform::from_xyz(ped.at.x, 0.0, -ped.at.y).with_rotation(Quat::from_rotation_y(ped.facing + std::f32::consts::FRAC_PI_2)),
                PedActor { idx },
            ));
        }
    }

    if interactive.hud {
        for i in 0..90u32 {
            let h1 = sim::geom::hash01(i as i64, 11);
            let h2 = sim::geom::hash01(i as i64, 23);
            let h3 = sim::geom::hash01(i as i64, 37);
            let off = Vec3::new((h1 - 0.5) * 48.0, 1.0 + h2 * 22.0, (h3 - 0.5) * 48.0);
            commands.spawn((
                Mesh3d(atlas.snow_mesh.clone()),
                MeshMaterial3d(atlas.snow_mat.clone()),
                Transform::from_translation(Vec3::new(sim.player_at.x + off.x, off.y, -sim.player_at.y + off.z)),
                Visibility::Hidden,
                SnowParticle {
                    offset: off,
                    speed: 3.4 + (i % 5) as f32 * 0.55,
                    phase: h1 * std::f32::consts::TAU,
                },
            ));
        }
        sim.hour = clock.hour;
        clock.blackout = sim.hour >= sim::survival::BLACKOUT_HOUR;
        spawn_hud(&mut commands, assets.load("fonts/NotoSansSC-VF.ttf"));
        // 3 pooled dynamic indoor lights (overhead ceiling light, coal stove glow, player flashlight).
        commands.spawn((
            PointLight { color: Color::srgb(1.0, 0.90, 0.70), intensity: 65_000.0, range: 16.0, shadow_maps_enabled: false, ..default() },
            Transform::default(),
            Visibility::Hidden,
            IndoorLightKind::Room,
        ));
        commands.spawn((
            PointLight { color: Color::srgb(1.0, 0.44, 0.12), intensity: 56_000.0, range: 14.0, shadow_maps_enabled: false, ..default() },
            Transform::default(),
            Visibility::Hidden,
            IndoorLightKind::Stove,
        ));
        commands.spawn((
            PointLight { color: Color::srgb(0.90, 0.95, 1.0), intensity: 55_000.0, range: 14.0, shadow_maps_enabled: false, ..default() },
            Transform::default(),
            Visibility::Hidden,
            IndoorLightKind::Flashlight,
        ));
        commands.spawn((
            PointLight { color: Color::srgb(0.84, 0.92, 1.0), intensity: 120_000.0, range: 22.0, shadow_maps_enabled: false, ..default() },
            Transform::default(),
            Visibility::Hidden,
            IndoorLightKind::FlashlightBounce,
        ));
        commands.spawn((
            SpotLight {
                color: Color::srgb(0.95, 0.98, 1.0),
                intensity: 650_000.0,
                range: 30.0,
                inner_angle: 0.22,
                outer_angle: 0.44,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::default(),
            Visibility::Hidden,
            FlashlightSpot,
        ));
    }

    let init_yaw = sim.player_facing;
    commands.insert_resource(atlas);
    commands.insert_resource(PlayState {
        sim,
        started: !interactive.play,
        paused: false,
        forces: seed.0.clone(),
        cursor_world: None,
        preview_barricade: false,
        click_cooldown: 0.0,
        flashlight_yaw: init_yaw,
    });
}

fn spawn_speed_btn(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    action: SpeedAction,
    btn_speed: f32,
) {
    let normal_border = Color::srgba(0.45, 0.62, 0.80, 0.38);
    let hover_border = Color::srgba(0.92, 0.96, 1.0, 0.90);
    let mut ec = parent.spawn((
        Node {
            height: Val::Px(26.0),
            padding: UiRect::horizontal(Val::Px(9.0)),
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.08, 0.13, 0.82)),
        BorderColor::all(normal_border),
        SpeedBtn(btn_speed),
    ));
    ec.observe(move |ev: On<PointerOver>, mut q: Query<&mut BorderColor>, mut queue: ResMut<SpeedActionQueue>| {
        queue.hovered += 1;
        if let Ok(mut bd) = q.get_mut(ev.entity) {
            *bd = BorderColor::all(hover_border);
        }
    })
    .observe(move |ev: On<PointerOut>, mut q: Query<&mut BorderColor>, mut queue: ResMut<SpeedActionQueue>| {
        queue.hovered = queue.hovered.saturating_sub(1);
        if let Ok(mut bd) = q.get_mut(ev.entity) {
            *bd = BorderColor::all(normal_border);
        }
    })
    .observe(move |ev: On<PointerClick>, mut queue: ResMut<SpeedActionQueue>| {
        if ev.button == PointerButton::Primary {
            queue.actions.push(action);
        }
    });
    ec.with_children(|b| {
        let mut t = b.spawn((
            Text::new(label),
            TextFont {
                font: font.clone().into(),
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ));
        match action {
            SpeedAction::TogglePause => {
                t.insert(PauseBtnText);
            }
            SpeedAction::ToggleFlashlight => {
                t.insert(FlashlightBtnText);
            }
            SpeedAction::Set(_) => {}
        }
    });
}

fn spawn_hud(commands: &mut Commands, font: Handle<Font>) {
    let tf = |px: f32| TextFont { font: font.clone().into(), font_size: bevy::text::FontSize::Px(px), ..default() };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(74.0),
                left: Val::Px(26.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(5.0),
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            SpeedBarRoot,
        ))
        .with_children(|bar| {
            spawn_speed_btn(bar, &font, "蓝方 BLUE | 暂停 P", SpeedAction::TogglePause, -1.0);
            for spd in SPEED_LEVELS {
                let lbl = format!("{}x", spd as u32);
                spawn_speed_btn(bar, &font, &lbl, SpeedAction::Set(spd), spd);
            }
            spawn_speed_btn(bar, &font, "手电 L: 关", SpeedAction::ToggleFlashlight, -2.0);
        });
    commands.spawn((
        Text::new(""),
        tf(15.0),
        TextColor(Color::srgb(0.86, 0.91, 0.96)),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, top: Val::Px(112.0), left: Val::Px(26.0), max_width: Val::Px(520.0), ..default() },
        HudRole::Objective,
    ));
    commands.spawn((
        Text::new(""),
        tf(19.0),
        TextColor(Color::WHITE),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, top: Val::Px(20.0), right: Val::Px(26.0), ..default() },
        HudRole::TopRight,
    ));
    commands.spawn((
        Text::new(""),
        tf(21.0),
        TextColor(Color::srgb(1.0, 0.82, 0.45)),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, top: Val::Px(22.0), left: Val::Percent(33.0), ..default() },
        HudRole::Notice,
    ));
    commands.spawn((
        Text::new(""),
        tf(20.0),
        TextColor(Color::srgb(1.0, 0.92, 0.58)),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(124.0), left: Val::Percent(22.0), max_width: Val::Percent(56.0), ..default() },
        HudRole::Prompt,
    ));
    commands.spawn((
        Text::new(""),
        tf(14.5),
        TextColor(Color::srgb(0.88, 0.95, 1.0)),
        TextShadow::default(),
        BackgroundColor(Color::srgba(0.03, 0.06, 0.10, 0.86)),
        BorderColor::all(Color::srgba(0.24, 0.62, 0.96, 0.65)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(74.0),
            left: Val::Percent(24.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)),
            border: UiRect::all(Val::Px(1.2)),
            max_width: Val::Percent(52.0),
            ..default()
        },
        Visibility::Hidden,
        HudRole::RadioFeed,
    ));
    commands.spawn((
        Text::new(""),
        tf(15.0),
        TextColor(Color::srgb(0.92, 0.94, 0.90)),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(34.0), left: Val::Px(18.0), ..default() },
        HudRole::Stats,
    ));
    commands.spawn((
        Text::new(""),
        tf(15.0),
        TextColor(Color::srgb(0.94, 0.97, 0.92)),
        TextShadow::default(),
        BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.78)),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(96.0), right: Val::Px(24.0), padding: UiRect::all(Val::Px(12.0)), max_width: Val::Px(340.0), ..default() },
        Visibility::Hidden,
        HudRole::Sms,
    ));
    commands.spawn((
        Text::new(""),
        tf(24.0),
        TextColor(Color::WHITE),
        TextShadow::default(),
        BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.84)),
        Node { position_type: PositionType::Absolute, top: Val::Percent(28.0), left: Val::Percent(24.0), padding: UiRect::all(Val::Px(30.0)), ..default() },
        Visibility::Hidden,
        HudRole::Death,
    ));
}

fn mouse_ground(cam: &Camera, cam_tf: &GlobalTransform, window: &Window) -> Option<P2> {
    let cursor = window.cursor_position()?;
    let ray = cam.viewport_to_world(cam_tf, cursor).ok()?;
    if ray.direction.y.abs() < 1.0e-4 {
        return None;
    }
    let t = -ray.origin.y / ray.direction.y;
    if t <= 0.0 {
        return None;
    }
    let hit = ray.origin + ray.direction * t;
    Some(P2::new(hit.x, -hit.z))
}

pub fn spawn_fx(
    commands: &mut Commands,
    atlas: &FigureAtlas,
    tracers: &[TracerFx],
    explosions: &[ExplosionFx],
) {
    for t in tracers {
        let a = Vec3::from_array(t.from);
        let b = Vec3::from_array(t.to);
        let mid = (a + b) * 0.5;
        let len = a.distance(b).max(0.2);
        let look = Transform::from_translation(mid)
            .looking_at(b, Vec3::Y)
            .with_scale(Vec3::new(1.0, 1.0, len));
        commands.spawn((
            Mesh3d(atlas.tracer_mesh.clone()),
            MeshMaterial3d(atlas.fire_mat.clone()),
            look,
            TransientFx { ttl: 0.10, vel: Vec3::ZERO, grow: 0.0 },
        ));
    }
    for e in explosions {
        let scale = (e.radius * 0.28).clamp(0.8, 3.0);
        commands.spawn((
            Mesh3d(atlas.burst_mesh.clone()),
            MeshMaterial3d(atlas.fire_mat.clone()),
            Transform::from_xyz(e.at.x, 0.8, -e.at.y).with_scale(Vec3::splat(scale)),
            PointLight {
                color: Color::srgb(1.0, 0.5, 0.15),
                intensity: 120_000.0,
                range: 24.0,
                shadow_maps_enabled: false,
                ..default()
            },
            TransientFx { ttl: 0.35, vel: Vec3::Y * 1.4, grow: 1.5 },
        ));
        commands.spawn((
            Mesh3d(atlas.soot_mesh.clone()),
            MeshMaterial3d(atlas.soot_mat.clone()),
            Transform::from_xyz(e.at.x, 0.02, -e.at.y),
            SootMark,
        ));
    }
}

/// Save the run by itself every morning; when you die, its saves go with you
/// (one life, docs/DESIGN.md rule 1).
fn autosave(play: Option<Res<PlayState>>, mut last_morning: Local<Option<u32>>, mut buried: Local<bool>) {
    let Some(p) = play else { return };
    if !p.started {
        *last_morning = None;
        return;
    }
    if p.sim.dead {
        if !*buried {
            *buried = true;
            sim::save::delete_all();
        }
        return;
    }
    *buried = false;
    match *last_morning {
        Some(m) if p.sim.mornings > m => {
            if let Err(e) = sim::save::write(&p.sim, "auto") {
                eprintln!("autosave: {e}");
            }
            *last_morning = Some(p.sim.mornings);
        }
        Some(m) if p.sim.mornings == m => {}
        _ => *last_morning = Some(p.sim.mornings),
    }
}

pub fn step_play(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut eye_q: Single<(&mut Eye, &Camera, &GlobalTransform)>,
    heihe: Res<Heihe>,
    atlas: Res<FigureAtlas>,
    mut play: ResMut<PlayState>,
    mut clock: ResMut<Clock>,
    mut player: Single<(&mut Player, &mut Transform, &mut Mesh3d)>,
    soot_q: Query<Entity, With<SootMark>>,
    (menu, mut audio_q, mut audio_state, mut speed_q, mut map, scenario, mut last_audio): (
        Res<MenuScreen>,
        ResMut<AudioQueue>,
        ResMut<AudioState>,
        ResMut<SpeedActionQueue>,
        Option<ResMut<crate::map_ui::TacticalMap>>,
        Option<Res<crate::menu_ui::StartScenario>>,
        Local<(bool, Option<(&'static str, String)>)>,
    ),
) {
    if *menu != MenuScreen::Playing || !play.started {
        audio_state.menu_open = true;
        speed_q.actions.clear();
        return;
    }
    audio_state.menu_open = false;
    play.click_cooldown = (play.click_cooldown - time.delta_secs()).max(0.0);
    let map_open = map.as_ref().is_some_and(|m| m.open);

    // Drain HUD speed bar clicks first so click_cooldown suppresses accidental firing on click.
    for action in speed_q.actions.drain(..) {
        match action {
            SpeedAction::Set(s) => {
                clock.speed = s;
                play.paused = false;
                play.click_cooldown = 0.24;
            }
            SpeedAction::TogglePause => {
                play.paused = !play.paused;
                play.click_cooldown = 0.24;
            }
            SpeedAction::ToggleFlashlight => {
                play.sim.flashlight_on = !play.sim.flashlight_on;
                clock.flashlight_on = play.sim.flashlight_on;
                play.click_cooldown = 0.24;
            }
        }
    }
    if play.sim.flashlight_on != last_audio.0 {
        last_audio.0 = play.sim.flashlight_on;
        audio_q.flashlight_click = true;
    }

    if play.sim.dead && keys.just_pressed(KeyCode::Enter) {
        let scen = scenario.map(|s| s.0).unwrap_or(0);
        let p_mut = &mut *play;
        crate::menu_ui::apply_scenario(
            scen,
            &heihe.city,
            &heihe.ground,
            &p_mut.forces,
            &mut p_mut.sim,
            &mut clock,
        );
        p_mut.paused = false;
        p_mut.preview_barricade = false;
        p_mut.flashlight_yaw = p_mut.sim.player_facing;
        eye_q.0.cam_pan = P2::default();
        player.0.at = p_mut.sim.player_at;
        player.1.translation = Vec3::new(p_mut.sim.player_at.x, 0.0, -p_mut.sim.player_at.y);
        for e in &soot_q {
            commands.entity(e).despawn();
        }
    }

    // Time acceleration hotkeys (Tab / =,+ / -,Numpad+- / [] / F5-F8).
    let step_speed = |cur: f32, dir: i32| -> f32 {
        let idx = SPEED_LEVELS
            .iter()
            .position(|&s| (s - cur).abs() < 0.1)
            .unwrap_or(0) as i32;
        let next = (idx + dir).clamp(0, (SPEED_LEVELS.len() - 1) as i32) as usize;
        SPEED_LEVELS[next]
    };
    if !play.sim.dead && !map_open {
        if keys.just_pressed(KeyCode::Tab) {
            let idx = SPEED_LEVELS
                .iter()
                .position(|&s| (s - clock.speed).abs() < 0.1)
                .unwrap_or(0);
            clock.speed = SPEED_LEVELS[(idx + 1) % SPEED_LEVELS.len()];
        }
        if keys.just_pressed(KeyCode::Equal)
            || keys.just_pressed(KeyCode::NumpadAdd)
            || keys.just_pressed(KeyCode::BracketRight)
        {
            clock.speed = step_speed(clock.speed, 1);
        }
        if keys.just_pressed(KeyCode::Minus)
            || keys.just_pressed(KeyCode::NumpadSubtract)
            || keys.just_pressed(KeyCode::BracketLeft)
        {
            clock.speed = step_speed(clock.speed, -1);
        }
        if keys.just_pressed(KeyCode::F5) {
            clock.speed = 1.0;
        } else if keys.just_pressed(KeyCode::F6) {
            clock.speed = 2.0;
        } else if keys.just_pressed(KeyCode::F7) {
            clock.speed = 4.0;
        } else if keys.just_pressed(KeyCode::F8) {
            clock.speed = 8.0;
        }
    }

    let near_stairs = !map_open
        && play.sim.active_interior.is_some_and(|i| {
            play.sim.interiors[i].num_floors > 1
                && play.sim.player_at.dist(play.sim.interiors[i].stairwell_world()) <= 2.4
        });
    if !play.sim.dead && !map_open {
        if keys.just_pressed(KeyCode::KeyP) || (!near_stairs && keys.just_pressed(KeyCode::Space)) {
            play.paused = !play.paused;
        }
    }

    let up = P2::new(-0.7071, 0.7071);
    let right = P2::new(0.7071, 0.7071);
    let mut wish = P2::new(0.0, 0.0);
    if !map_open {
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) { wish = wish.add(up); }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) { wish = wish.sub(up); }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { wish = wish.add(right); }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) { wish = wish.sub(right); }
    }

    let aim = mouse_ground(eye_q.1, eye_q.2, &window);
    play.cursor_world = aim;
    if let Some(c) = aim {
        let d = c.sub(play.sim.player_at);
        if d.len() > 0.35 {
            play.flashlight_yaw = d.y.atan2(d.x);
        }
    }

    let selected = if map_open {
        None
    } else if keys.just_pressed(KeyCode::Digit1) {
        Some(BarricadeKind::Sandbag)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(BarricadeKind::IronGate)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(BarricadeKind::FireBarrel)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(BarricadeKind::DeadCar)
    } else if keys.just_pressed(KeyCode::Digit5) {
        Some(BarricadeKind::Concrete)
    } else if keys.just_pressed(KeyCode::Digit6) {
        Some(BarricadeKind::Trench)
    } else if keys.just_pressed(KeyCode::Digit7) {
        Some(BarricadeKind::TarpDugout)
    } else if keys.just_pressed(KeyCode::Digit8) {
        Some(BarricadeKind::BarbedWire)
    } else {
        None
    };
    if selected.is_some() {
        play.preview_barricade = true;
    }
    let build = !map_open && keys.just_pressed(KeyCode::KeyB);
    if build {
        play.preview_barricade = false;
    }

    let over_speed_bar = window
        .cursor_position()
        .is_some_and(|p| p.x >= 20.0 && p.x <= 555.0 && p.y >= 68.0 && p.y <= 106.0);
    let allow_mouse = !map_open && play.click_cooldown <= 0.0 && speed_q.hovered == 0 && !over_speed_bar;
    let map_militia_cmd = map.as_mut().and_then(|m| m.pending_militia_order.take());
    let frame_dt = time.delta_secs().clamp(0.0, 0.1);

    if play.paused && !play.sim.dead {
        if !map_open && keys.just_pressed(KeyCode::KeyL) {
            play.sim.flashlight_on = !play.sim.flashlight_on;
        }
        if play.sim.flashlight_on != last_audio.0 {
            last_audio.0 = play.sim.flashlight_on;
            audio_q.flashlight_click = true;
        }
        clock.flashlight_on = play.sim.flashlight_on;
        if wish.len() > 0.05 {
            let next_pan = eye_q.0.cam_pan.add(wish.norm().mul(28.0 * frame_dt));
            eye_q.0.cam_pan = P2::new(next_pan.x.clamp(-90.0, 90.0), next_pan.y.clamp(-90.0, 90.0));
        }
        if keys.just_pressed(KeyCode::Home) {
            eye_q.0.cam_pan = P2::default();
        }
        if let Some(sel) = selected {
            play.sim.selected_barricade = sel;
        }
        if let Some(dest) = map_militia_cmd.or_else(|| if allow_mouse && mouse.just_pressed(MouseButton::Right) { aim } else { None }) {
            play.sim.issue_militia_order(dest);
        }
        let cur_radio = play.sim.radio_feed.as_ref().map(|r| (r.code_en, r.text_zh.clone()));
        if cur_radio != last_audio.1 {
            if let Some((code, _)) = &cur_radio {
                audio_q.radio_code = Some(*code);
            }
            last_audio.1 = cur_radio;
        }
        return;
    } else {
        let decay = (1.0 - frame_dt * 10.0).max(0.0);
        eye_q.0.cam_pan = eye_q.0.cam_pan.mul(decay);
    }

    let floor_delta = if map_open {
        0
    } else if (near_stairs && keys.just_pressed(KeyCode::Space)) || keys.just_pressed(KeyCode::PageUp) {
        1
    } else if keys.just_pressed(KeyCode::ControlLeft)
        || keys.just_pressed(KeyCode::ControlRight)
        || keys.just_pressed(KeyCode::PageDown)
    {
        -1
    } else {
        0
    };
    let input = sim::survival::SimInput {
        wish_move: wish,
        sprinting: !map_open && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)),
        interact_held: !map_open && keys.pressed(KeyCode::KeyE),
        interact_just_pressed: !map_open && keys.just_pressed(KeyCode::KeyE),
        use_bandage: !map_open && keys.just_pressed(KeyCode::KeyQ),
        use_food: !map_open && keys.just_pressed(KeyCode::KeyZ),
        use_water: !map_open && keys.just_pressed(KeyCode::KeyX),
        use_warmer: !map_open && keys.just_pressed(KeyCode::KeyC),
        rally_resistance: !map_open && keys.just_pressed(KeyCode::KeyH),
        select_barricade: selected,
        build_barricade: build,
        build_at: if build { aim } else { None },
        toggle_militia_order: !map_open && keys.just_pressed(KeyCode::KeyG),
        command_militia_to: map_militia_cmd.or_else(|| if allow_mouse && mouse.just_pressed(MouseButton::Right) { aim } else { None }),
        fire_at: if allow_mouse && mouse.pressed(MouseButton::Left) { aim } else { None },
        fire_rpg_at: if !map_open && keys.just_pressed(KeyCode::KeyF) { aim } else { None },
        throw_molotov_at: if !map_open && keys.just_pressed(KeyCode::KeyV) { aim } else { None },
        throw_grenade_at: if !map_open && keys.just_pressed(KeyCode::KeyY) { aim } else { None },
        throw_smoke_at: if !map_open && keys.just_pressed(KeyCode::KeyO) { aim } else { None },
        launch_fpv_at: if !map_open && keys.just_pressed(KeyCode::KeyI) { aim } else { None },
        call_radio_at: if !map_open && keys.just_pressed(KeyCode::KeyR) { aim } else { None },
        cycle_primary: !map_open && keys.just_pressed(KeyCode::KeyN),
        select_primary: None,
        cycle_at: !map_open && keys.just_pressed(KeyCode::KeyU),
        select_at: None,
        toggle_flashlight: !map_open && keys.just_pressed(KeyCode::KeyL),
        toggle_curtains: !map_open && keys.just_pressed(KeyCode::KeyK),
        fortify_or_stove: !map_open && keys.just_pressed(KeyCode::KeyT),
        floor_delta,
        clock_running: true,
        toggle_sleep: false,
    };
    // J: tired, lie down; not tired, let the days pass by the routine; again, stop.
    let mut input = input;
    if !map_open && keys.just_pressed(KeyCode::KeyJ) {
        if play.sim.passing {
            play.sim.passing = false;
            play.sim.subtitle_zh = "你停了下来。".into();
            play.sim.subtitle_en = "You stop.".into();
            play.sim.subtitle_ttl = 3.0;
        } else if !play.sim.sleeping && play.sim.body.fatigue < sim::survival::SLEEP_MIN_FATIGUE && play.sim.can_rest().is_ok() {
            play.sim.passing = true;
        } else {
            input.toggle_sleep = true;
        }
    }
    if play.sim.passing {
        // Days at a glance: a day every two seconds, until something needs you.
        let state = &mut *play;
        let passed = state.sim.pass_time(
            &heihe.city,
            &heihe.ground,
            &state.forces,
            &sim::routine::Routine::default(),
            frame_dt * PASS_HOURS_PER_SECOND,
        );
        if passed.stop != sim::routine::Stop::Done {
            state.sim.passing = false;
            if passed.stop != sim::routine::Stop::Dead {
                let (zh, en) = passed.stop.text();
                state.sim.subtitle_zh = zh.into();
                state.sim.subtitle_en = en.into();
                state.sim.subtitle_ttl = 8.0;
            }
        }
        clock.hour = state.sim.hour;
        clock.blackout = state.sim.blackout;
        clock.fog = state.sim.fog_density;
        clock.overcast = overcast(state.sim.weather);
        clock.flashlight_on = state.sim.flashlight_on;
        return;
    }

    if play.sim.flashlight_on && wish.len() < 0.05 && !play.sim.sleeping && !play.sim.dead && !map_open {
        play.sim.player_facing = play.flashlight_yaw;
    }

    let was_danger = play.sim.challenged
        || play.sim.under_fire_for > 0.0
        || play.sim.fpv.is_some()
        || play.sim.lancet.is_some()
        || play.sim.tank_lock_warning;
    // Asleep, the world runs fast in long steps (the sim watches for what wakes you
    // at every one); awake, at the chosen speed in short ones.
    let asleep = play.sim.sleeping || input.toggle_sleep;
    let (sim_dt, steps) = if asleep {
        let sim_dt = frame_dt.max(1.0 / 240.0) * sim::survival::SLEEP_PLAY_SPEED;
        (sim_dt, (sim_dt / 0.25).ceil().clamp(1.0, 16.0) as usize)
    } else {
        let sim_dt = frame_dt * clock.speed.max(1.0);
        (sim_dt, (sim_dt / 0.05).ceil().clamp(1.0, 20.0) as usize)
    };
    let sub_dt = (sim_dt / steps as f32).min(0.25);

    let prev_at = play.sim.inv.total_at_ammo();
    let prev_grenades = play.sim.grenades.len();
    let prev_smoke = play.sim.smoke_clouds.len();
    let mut fx = sim::survival::TickOutput::default();
    {
        let state = &mut *play;
        for s in 0..steps {
            let step_input = if s == 0 {
                input.clone()
            } else {
                sim::survival::SimInput {
                    wish_move: input.wish_move,
                    sprinting: input.sprinting,
                    interact_held: input.interact_held,
                    fire_at: input.fire_at,
                    clock_running: input.clock_running,
                    ..Default::default()
                }
            };
            let out = state.sim.tick(&heihe.city, &heihe.ground, &state.forces, &step_input, sub_dt);
            fx.tracers.extend(out.tracers);
            fx.explosions.extend(out.explosions);
            fx.footstep |= out.footstep;
            fx.mortar_whistle |= out.mortar_whistle;
            fx.sms_arrived |= out.sms_arrived;
            fx.shout_stoy |= out.shout_stoy;
            fx.player_hit |= out.player_hit;
            fx.door_action |= out.door_action;
            fx.door_breach_thud |= out.door_breach_thud;
            fx.glass_break |= out.glass_break;
            fx.tank_cannon |= out.tank_cannon;
            fx.slab_collapse |= out.slab_collapse;
            if state.sim.dead || (asleep && !state.sim.sleeping) {
                break;
            }
        }
    }

    let now_danger = play.sim.challenged
        || play.sim.under_fire_for > 0.0
        || play.sim.fpv.is_some()
        || play.sim.lancet.is_some()
        || play.sim.tank_lock_warning
        || fx.mortar_whistle
        || fx.player_hit
        || fx.tank_cannon
        || fx.slab_collapse
        || play.sim.dead;
    if !was_danger && now_danger && clock.speed > 1.0 {
        clock.speed = 1.0;
        if !play.sim.dead {
            play.sim.subtitle_zh = "发现险情，时间自动恢复 1x".into();
            play.sim.subtitle_ttl = play.sim.subtitle_ttl.max(3.5);
        }
    }

    clock.hour = play.sim.hour;
    clock.blackout = play.sim.blackout;
    clock.fog = play.sim.fog_density;
    clock.overcast = overcast(play.sim.weather);
    clock.flashlight_on = play.sim.flashlight_on;

    if fx.footstep {
        audio_q.footstep = Some(input.sprinting);
    }
    if fx.mortar_whistle {
        audio_q.whistle = true;
    }
    if fx.sms_arrived {
        audio_q.sms = true;
    }
    if fx.shout_stoy {
        audio_q.shout_stoy = true;
    }
    if fx.door_action {
        audio_q.door_action = true;
    }
    if fx.door_breach_thud {
        audio_q.door_thud = true;
    }
    if fx.glass_break {
        audio_q.glass_break = true;
    }
    if fx.tank_cannon {
        audio_q.tank_cannon = true;
    }
    if fx.slab_collapse {
        audio_q.slab_collapse = true;
    }
    if play.sim.flashlight_on != last_audio.0 {
        last_audio.0 = play.sim.flashlight_on;
        audio_q.flashlight_click = true;
    }
    if play.sim.inv.total_at_ammo() < prev_at {
        audio_q.rpg_launch = true;
    }
    if play.sim.smoke_clouds.len() > prev_smoke && play.sim.smoke_clouds.last().is_some_and(|s| !s.dark) {
        audio_q.smoke_hiss = true;
    }
    if play.sim.grenades.len() > prev_grenades && play.sim.grenades.last().is_some_and(|g| !g.friendly) {
        audio_q.shout_grenade = true;
    }
    let cur_radio = play.sim.radio_feed.as_ref().map(|r| (r.code_en, r.text_zh.clone()));
    if cur_radio != last_audio.1 {
        if let Some((code, _)) = &cur_radio {
            audio_q.radio_code = Some(*code);
        }
        last_audio.1 = cur_radio;
    }
    if !fx.tracers.is_empty() {
        if input.fire_at.is_some() && play.sim.inv.has_rifle {
            match play.sim.inv.active_primary {
                PrimaryWeapon::Saiga12 => audio_q.shotgun = true,
                PrimaryWeapon::Pkm => audio_q.mg_burst = true,
                _ => audio_q.gunfire = audio_q.gunfire.saturating_add(fx.tracers.len() as u32),
            }
        } else {
            audio_q.gunfire = audio_q.gunfire.saturating_add(fx.tracers.len() as u32);
        }
    }
    for e in &fx.explosions {
        if e.radius <= 8.2 && !fx.tank_cannon && play.sim.grenades.len() < prev_grenades {
            audio_q.grenade_boom = true;
        } else {
            audio_q.explosions.push(e.at.dist(play.sim.player_at));
        }
    }

    audio_state.sheltered = play.sim.body.sheltered;
    audio_state.fog = play.sim.fog_density;
    audio_state.raining = play.sim.weather == Weather::Rain;
    if let Some(i_idx) = play.sim.active_interior {
        let intr = &play.sim.interiors[i_idx];
        let broken = intr.has_unboarded_broken_window();
        audio_state.indoor_broken = broken;
        audio_state.indoor_sealed = !broken && intr.open_doors_count() == 0;
        audio_state.near_stove = (intr.stove_fuel_s > 0.0 && play.sim.player_floor == 0) || play.sim.near_fire();
    } else {
        audio_state.indoor_broken = false;
        audio_state.indoor_sealed = false;
        audio_state.near_stove = play.sim.near_fire();
    }
    let d_recon = if play.sim.drone.active {
        Some(play.sim.drone.at.dist(play.sim.player_at))
    } else {
        None
    };
    let d_fpv = play.sim.fpv.as_ref().map(|f| f.at.dist(play.sim.player_at));
    audio_state.drone_dist = match (d_recon, d_fpv) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    };

    // Update player transform & articulated mesh pose (including floor elevation).
    let (who, tf, pmesh) = &mut *player;
    who.at = play.sim.player_at;
    let floor_y = if play.sim.active_interior.is_some() {
        play.sim.player_floor as f32 * FLOOR_H + 0.07
    } else {
        0.0
    };
    let vault_y = vault_hop_y(play.sim.player_vault_t);
    tf.translation = Vec3::new(who.at.x, floor_y + vault_y, -who.at.y);
    tf.rotation = Quat::from_rotation_y(play.sim.player_facing + std::f32::consts::FRAC_PI_2);

    let variant = u8::from(play.sim.body.coat_bonus > 0.0) | (u8::from(play.sim.inv.has_rifle) << 1);
    let anim = play.sim.player_anim;
    let frames = match anim {
        AnimState::Walk | AnimState::Run => 8,
        _ => 1,
    };
    let frame = if frames > 1 {
        ((play.sim.player_phase / std::f32::consts::TAU).rem_euclid(1.0) * frames as f32) as u8 % frames
    } else {
        0
    };
    if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::Civilian, variant, anim, frame }) {
        if pmesh.0 != *h {
            pmesh.0 = h.clone();
        }
    }

    spawn_fx(&mut commands, &atlas, &fx.tracers, &fx.explosions);
}

fn vault_hop_y(vault_t: f32) -> f32 {
    if vault_t > 0.0 {
        ((vault_t / 0.45).clamp(0.0, 1.0) * std::f32::consts::PI).sin() * 0.55
    } else {
        0.0
    }
}

fn upsert_pooled(
    commands: &mut Commands,
    map: &mut HashMap<u32, Entity>,
    pool_q: &mut Query<(&mut Transform, &mut Mesh3d, &mut Visibility), (With<PooledActor>, Without<PooledScene>, Without<PedActor>, Without<InteriorPart>, Without<IndoorLightKind>, Without<FlashlightSpot>)>,
    key: u32,
    mesh_h: Handle<Mesh>,
    mat_h: &Handle<StandardMaterial>,
    tf: Transform,
    seen: &mut HashSet<u32>,
) {
    seen.insert(key);
    if let Some(&e) = map.get(&key) {
        if let Ok((mut t, mut m, mut v)) = pool_q.get_mut(e) {
            *t = tf;
            *v = Visibility::Inherited;
            if m.0 != mesh_h {
                m.0 = mesh_h;
            }
            return;
        }
    }
    let e = commands
        .spawn((
            Mesh3d(mesh_h),
            MeshMaterial3d(mat_h.clone()),
            tf,
            Visibility::Inherited,
            PooledActor,
        ))
        .id();
    map.insert(key, e);
}

fn upsert_pooled_scene(
    commands: &mut Commands,
    map: &mut HashMap<u32, Entity>,
    scene_pool_q: &mut Query<(&mut Transform, &mut Visibility), (With<PooledScene>, Without<PooledActor>, Without<PedActor>, Without<InteriorPart>, Without<IndoorLightKind>, Without<FlashlightSpot>)>,
    key: u32,
    scene_h: Handle<WorldAsset>,
    tf: Transform,
    seen: &mut HashSet<u32>,
) {
    seen.insert(key);
    if let Some(&e) = map.get(&key) {
        if let Ok((mut t, mut v)) = scene_pool_q.get_mut(e) {
            *t = tf;
            *v = Visibility::Inherited;
            return;
        }
    }
    let e = commands
        .spawn((
            WorldAssetRoot(scene_h),
            tf,
            Visibility::Inherited,
            PooledScene,
        ))
        .id();
    map.insert(key, e);
}

fn prune_pool(commands: &mut Commands, map: &mut HashMap<u32, Entity>, seen: &HashSet<u32>) {
    map.retain(|k, e| {
        if seen.contains(k) {
            true
        } else {
            commands.entity(*e).despawn();
            false
        }
    });
}

fn upsert_slot(
    commands: &mut Commands,
    slot: &mut Option<Entity>,
    pool_q: &mut Query<(&mut Transform, &mut Mesh3d, &mut Visibility), (With<PooledActor>, Without<PooledScene>, Without<PedActor>, Without<InteriorPart>, Without<IndoorLightKind>, Without<FlashlightSpot>)>,
    mesh_h: Handle<Mesh>,
    mat_h: &Handle<StandardMaterial>,
    active_tf: Option<Transform>,
) {
    if let Some(tf) = active_tf {
        if let Some(e) = *slot {
            if let Ok((mut t, mut m, mut v)) = pool_q.get_mut(e) {
                *t = tf;
                *v = Visibility::Inherited;
                if m.0 != mesh_h {
                    m.0 = mesh_h;
                }
                return;
            }
        }
        let e = commands
            .spawn((
                Mesh3d(mesh_h),
                MeshMaterial3d(mat_h.clone()),
                tf,
                Visibility::Inherited,
                PooledActor,
            ))
            .id();
        *slot = Some(e);
    } else if let Some(e) = *slot {
        if let Ok((_, _, mut v)) = pool_q.get_mut(e) {
            *v = Visibility::Hidden;
        }
    }
}

fn upsert_scene_slot(
    commands: &mut Commands,
    slot: &mut Option<Entity>,
    scene_pool_q: &mut Query<(&mut Transform, &mut Visibility), (With<PooledScene>, Without<PooledActor>, Without<PedActor>, Without<InteriorPart>, Without<IndoorLightKind>, Without<FlashlightSpot>)>,
    scene_h: Handle<WorldAsset>,
    active_tf: Option<Transform>,
) {
    if let Some(tf) = active_tf {
        if let Some(e) = *slot {
            if let Ok((mut t, mut v)) = scene_pool_q.get_mut(e) {
                *t = tf;
                *v = Visibility::Inherited;
                return;
            }
        }
        let e = commands
            .spawn((
                WorldAssetRoot(scene_h),
                tf,
                Visibility::Inherited,
                PooledScene,
            ))
            .id();
        *slot = Some(e);
    } else if let Some(e) = *slot {
        if let Ok((_, mut v)) = scene_pool_q.get_mut(e) {
            *v = Visibility::Hidden;
        }
    }
}

fn barricade_mesh_idx(kind: BarricadeKind) -> usize {
    match kind {
        BarricadeKind::Sandbag => 0,
        BarricadeKind::IronGate => 1,
        BarricadeKind::FireBarrel => 2,
        BarricadeKind::DeadCar => 3,
        BarricadeKind::Concrete => 4,
        BarricadeKind::Trench => 5,
        BarricadeKind::TarpDugout => 6,
        BarricadeKind::BarbedWire => 7,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sync_actors(
    mut commands: Commands,
    time: Res<Time>,
    mut smoke_acc: Local<f32>,
    heihe: Res<Heihe>,
    play: Res<PlayState>,
    atlas: Res<FigureAtlas>,
    mats: Res<Mats>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pools: ResMut<ActorPools>,
    mut interiors_vis: Option<ResMut<InteriorVisuals>>,
    mut peds: Query<(&PedActor, &mut Transform, &mut Visibility, &mut Mesh3d)>,
    mut pool_q: Query<(&mut Transform, &mut Mesh3d, &mut Visibility), (With<PooledActor>, Without<PooledScene>, Without<PedActor>, Without<InteriorPart>, Without<IndoorLightKind>, Without<FlashlightSpot>)>,
    mut intr_vis_q: Query<&mut Visibility, (With<InteriorPart>, Without<PooledScene>, Without<PooledActor>, Without<PedActor>, Without<IndoorLightKind>, Without<FlashlightSpot>)>,
    (mut lights_q, mut spot_q, mut scene_pool_q, mut materials, mut last_beam_fog): (
        Query<(&mut Transform, &mut PointLight, &mut Visibility, &IndoorLightKind), (Without<PooledScene>, Without<PooledActor>, Without<PedActor>, Without<InteriorPart>, Without<FlashlightSpot>)>,
        Query<(&mut Transform, &mut SpotLight, &mut Visibility), (With<FlashlightSpot>, Without<PooledScene>, Without<PooledActor>, Without<PedActor>, Without<InteriorPart>, Without<IndoorLightKind>)>,
        Query<(&mut Transform, &mut Visibility), (With<PooledScene>, Without<PooledActor>, Without<PedActor>, Without<InteriorPart>, Without<IndoorLightKind>, Without<FlashlightSpot>)>,
        ResMut<Assets<StandardMaterial>>,
        Local<Option<f32>>,
    ),
) {
    for (pa, mut tf, mut vis, mut mesh) in &mut peds {
        let Some(ped) = play.sim.pedestrians.get(pa.idx) else { continue };
        *vis = if ped.hidden { Visibility::Hidden } else { Visibility::Inherited };
        if ped.hidden {
            continue;
        }
        tf.translation = Vec3::new(ped.at.x, 0.0, -ped.at.y);
        tf.rotation = Quat::from_rotation_y(ped.facing + std::f32::consts::FRAC_PI_2);
        let anim = if play.sim.alert_stage >= 2 { AnimState::Run } else { AnimState::Walk };
        let frames = if anim == AnimState::Run { 4 } else { 6 };
        let frame = ((ped.phase / std::f32::consts::TAU).rem_euclid(1.0) * frames as f32) as u8 % frames;
        let variant = 10 + (pa.idx % 5) as u8;
        if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::Civilian, variant, anim, frame }) {
            if mesh.0 != *h {
                mesh.0 = h.clone();
            }
        }
    }

    let mut seen = HashSet::new();
    for g in &play.sim.squads {
        for s in &g.men {
            let (anim, frame) = if s.hp <= 0.0 {
                (AnimState::Dead, 0)
            } else if s.vault_t > 0.0 {
                (AnimState::Vault, 0)
            } else if s.in_cover && s.speed < 0.2 {
                (AnimState::Crouch, 0)
            } else if matches!(g.state, sim::survival::SquadState::Challenge | sim::survival::SquadState::Engage) && s.speed < 0.2 {
                (AnimState::Aim, 0)
            } else if s.speed > 2.2 {
                (AnimState::Run, ((s.phase / std::f32::consts::TAU).rem_euclid(1.0) * 4.0) as u8 % 4)
            } else if s.speed > 0.2 {
                (AnimState::Walk, ((s.phase / std::f32::consts::TAU).rem_euclid(1.0) * 6.0) as u8 % 6)
            } else {
                (AnimState::Idle, 0)
            };
            if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::RussianSoldier, variant: u8::from(s.optics), anim, frame }) {
                let sy = if s.floor > 0
                    || play.sim.interiors.iter().any(|it| {
                        (it.origin.x - s.at.x).abs() < 18.0 && (it.origin.y - s.at.y).abs() < 18.0 && it.contains(s.at)
                    })
                {
                    s.floor as f32 * FLOOR_H + 0.07
                } else {
                    0.0
                };
                let tf = Transform::from_xyz(s.at.x, sy + vault_hop_y(s.vault_t), -s.at.y)
                    .with_rotation(Quat::from_rotation_y(s.facing + std::f32::consts::FRAC_PI_2));
                upsert_pooled(&mut commands, &mut pools.soldiers, &mut pool_q, s.id, h.clone(), &mats.matte, tf, &mut seen);
            }
        }
    }
    prune_pool(&mut commands, &mut pools.soldiers, &seen);

    seen.clear();
    for (mi, m) in play.sim.militia.iter().enumerate() {
        let (anim, frame) = if m.hp <= 0.0 {
            (AnimState::Dead, 0)
        } else if m.downed {
            (AnimState::Prone, 0)
        } else if m.vault_t > 0.0 {
            (AnimState::Vault, 0)
        } else if m.in_cover && m.speed < 0.2 {
            (AnimState::Crouch, 0)
        } else if m.fire_cd > 0.65 {
            (AnimState::Aim, 0)
        } else if m.speed > 0.2 {
            (AnimState::Walk, ((m.phase / std::f32::consts::TAU).rem_euclid(1.0) * 6.0) as u8 % 6)
        } else {
            (AnimState::Idle, 0)
        };
        if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::Militia, variant: (mi % 4) as u8, anim, frame }) {
            let my = if m.floor > 0
                || play.sim.interiors.iter().any(|it| {
                    (it.origin.x - m.at.x).abs() < 18.0 && (it.origin.y - m.at.y).abs() < 18.0 && it.contains(m.at)
                })
            {
                m.floor as f32 * FLOOR_H + 0.07
            } else {
                0.0
            };
            let tf = Transform::from_xyz(m.at.x, my + vault_hop_y(m.vault_t), -m.at.y)
                .with_rotation(Quat::from_rotation_y(m.facing + std::f32::consts::FRAC_PI_2));
            upsert_pooled(&mut commands, &mut pools.militia, &mut pool_q, m.id, h.clone(), &mats.matte, tf, &mut seen);
        }
    }
    prune_pool(&mut commands, &mut pools.militia, &seen);

    // Lazy-spawn / dynamically rebuild cutaway interior meshes when player_floor or structural revision changes.
    if let Some(iv) = interiors_vis.as_deref_mut() {
        if let Some(active_idx) = play.sim.cutaway_interior {
            if let Some(intr) = play.sim.interiors.get(active_idx) {
                let want_floor = if play.sim.active_interior == Some(active_idx) {
                    play.sim.player_floor
                } else {
                    0
                };
                let want_rev = intr.revision;
                let stale = iv
                    .spawned
                    .get(&active_idx)
                    .is_none_or(|v| v.floor_built != want_floor || v.revision != want_rev);
                if stale {
                    if let Some(old) = iv.spawned.remove(&active_idx) {
                        commands.entity(old.static_solid).despawn();
                        for e in [old.window_glass, old.window_boards, old.curtains, old.barricade_front, old.barricade_back]
                            .into_iter()
                            .flatten()
                        {
                            commands.entity(e).despawn();
                        }
                    }
                    let im = intr.build_mesh_for_floor(want_floor);
                    let static_solid = commands
                        .spawn((
                            Mesh3d(meshes.add(to_mesh(&im.static_solid))),
                            MeshMaterial3d(mats.matte.clone()),
                            Visibility::Hidden,
                            InteriorPart,
                        ))
                        .id();
                    let spawn_opt = |commands: &mut Commands,
                                     meshes: &mut Assets<Mesh>,
                                     soup: &Soup,
                                     mat: &Handle<StandardMaterial>|
                     -> Option<Entity> {
                        (soup.triangles() > 0).then(|| {
                            commands
                                .spawn((
                                    Mesh3d(meshes.add(to_mesh(soup))),
                                    MeshMaterial3d(mat.clone()),
                                    Visibility::Hidden,
                                    InteriorPart,
                                ))
                                .id()
                        })
                    };
                    let window_glass = spawn_opt(&mut commands, &mut meshes, &im.window_glass, &mats.glass);
                    let window_boards = spawn_opt(&mut commands, &mut meshes, &im.window_boards, &mats.matte);
                    let curtains = spawn_opt(&mut commands, &mut meshes, &im.curtains, &mats.matte);
                    let barricade_front = spawn_opt(&mut commands, &mut meshes, &im.barricade_front, &mats.matte);
                    let barricade_back = spawn_opt(&mut commands, &mut meshes, &im.barricade_back, &mats.matte);
                    iv.spawned.insert(
                        active_idx,
                        InteriorVisualSet {
                            static_solid,
                            window_glass,
                            window_boards,
                            curtains,
                            barricade_front,
                            barricade_back,
                            floor_built: want_floor,
                            revision: want_rev,
                        },
                    );
                }
            }
        }
        let set_v = |q: &mut Query<&mut Visibility, (With<InteriorPart>, Without<PooledScene>, Without<PooledActor>, Without<PedActor>, Without<IndoorLightKind>, Without<FlashlightSpot>)>, e: Entity, show: bool| {
            if let Ok(mut v) = q.get_mut(e) {
                let target = if show { Visibility::Inherited } else { Visibility::Hidden };
                if *v != target {
                    *v = target;
                }
            }
        };
        for (&idx, vis_set) in &iv.spawned {
            let Some(intr) = play.sim.interiors.get(idx) else { continue };
            let show = play.sim.cutaway_interior == Some(idx);
            set_v(&mut intr_vis_q, vis_set.static_solid, show);
            if let Some(e) = vis_set.window_glass {
                set_v(&mut intr_vis_q, e, show && intr.windows.iter().any(|w| !w.broken && !w.boarded));
            }
            if let Some(e) = vis_set.window_boards {
                set_v(&mut intr_vis_q, e, show && intr.windows.iter().any(|w| w.boarded));
            }
            if let Some(e) = vis_set.curtains {
                set_v(&mut intr_vis_q, e, show && intr.curtains_closed);
            }
            if let Some(e) = vis_set.barricade_front {
                set_v(&mut intr_vis_q, e, show && intr.front_door.barricade_hp > 0.0);
            }
            if let Some(e) = vis_set.barricade_back {
                set_v(&mut intr_vis_q, e, show && intr.back_door.as_ref().is_some_and(|b| b.barricade_hp > 0.0));
            }
        }
    }

    // Pooled indoor NPCs, searchable loot counter rings & stairwell marker inside cutaway_interior.
    seen.clear();
    let mut seen_loots = HashSet::new();
    let mut stair_tf = None;
    let active_intr_floor = play
        .sim
        .cutaway_interior
        .map(|ci| if play.sim.active_interior == Some(ci) { play.sim.player_floor } else { 0 })
        .unwrap_or(0);
    if let Some(idx) = play.sim.cutaway_interior {
        if let Some(intr) = play.sim.interiors.get(idx) {
            let _ = heihe;
            if active_intr_floor == 0 {
                for npc in &intr.npcs {
                    let variant = 10 + (npc.id % 5) as u8;
                    if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::Civilian, variant, anim: AnimState::Idle, frame: 0 }) {
                        let tf = Transform::from_xyz(npc.at.x, 0.07, -npc.at.y)
                            .with_rotation(Quat::from_rotation_y(npc.facing + std::f32::consts::FRAC_PI_2));
                        upsert_pooled(&mut commands, &mut pools.room_npcs, &mut pool_q, npc.id, h.clone(), &mats.matte, tf, &mut seen);
                    }
                }
                for loot in &intr.loots {
                    if !loot.searched {
                        let tf = Transform::from_xyz(loot.at.x, 0.08, -loot.at.y).with_scale(Vec3::splat(0.42));
                        upsert_pooled(&mut commands, &mut pools.room_loots, &mut pool_q, loot.id, atlas.ring_mesh.clone(), &atlas.ring_mat, tf, &mut seen_loots);
                    }
                }
                if let Some(lk) = intr.locker_at {
                    let st = &intr.stash;
                    if st.food + st.water + st.bandage + st.warmer + st.ammo + st.scrap + st.coal > 0 {
                        let tf = Transform::from_xyz(lk.x, 0.08, -lk.y).with_scale(Vec3::splat(0.46));
                        upsert_pooled(&mut commands, &mut pools.room_loots, &mut pool_q, 99_000 + idx as u32, atlas.ring_mesh.clone(), &atlas.ring_mat, tf, &mut seen_loots);
                    }
                }
            } else if let Some(fl) = intr.floors.get(active_intr_floor as usize) {
                let fy = active_intr_floor as f32 * FLOOR_H + 0.08;
                for loot in &fl.loots {
                    if !loot.searched {
                        let tf = Transform::from_xyz(loot.at.x, fy, -loot.at.y).with_scale(Vec3::splat(0.42));
                        upsert_pooled(&mut commands, &mut pools.room_loots, &mut pool_q, loot.id + active_intr_floor as u32 * 1_000_000, atlas.ring_mesh.clone(), &atlas.ring_mat, tf, &mut seen_loots);
                    }
                }
            }
            if intr.num_floors > 1 {
                let st = intr.stairwell_world();
                stair_tf = Some(Transform::from_xyz(st.x, active_intr_floor as f32 * FLOOR_H + 0.08, -st.y).with_scale(Vec3::splat(0.55)));
            }
        }
    }
    prune_pool(&mut commands, &mut pools.room_npcs, &seen);
    prune_pool(&mut commands, &mut pools.room_loots, &seen_loots);
    upsert_slot(&mut commands, &mut pools.stair_ring, &mut pool_q, atlas.ring_mesh.clone(), &atlas.ring_mat, stair_tf);

    // Update pooled indoor, directional spotlight & fog-reactive flashlight beams aligned with active floor.
    let active_intr = play.sim.cutaway_interior.and_then(|i| play.sim.interiors.get(i));
    let light_base_y = active_intr_floor as f32 * FLOOR_H;
    let fl_base_y = light_base_y + vault_hop_y(play.sim.player_vault_t);
    let fl_on = play.sim.flashlight_on && !play.sim.dead && !play.sim.sleeping;
    let (bright_mul, range_mul) = play.sim.flashlight_illumination_scale();
    let fog = play.sim.fog_density.clamp(0.0, 1.0);
    let beam_yaw = if play.cursor_world.is_some() { play.flashlight_yaw } else { play.sim.player_facing };
    let fwd = P2::new(beam_yaw.cos(), beam_yaw.sin());
    for (mut tf, mut pl, mut vis, kind) in &mut lights_q {
        match kind {
            IndoorLightKind::Room => {
                if let Some(intr) = active_intr.filter(|it| !play.sim.blackout && play.sim.hour < sim::survival::BLACKOUT_HOUR && it.kind != sim::interior::InteriorKind::BasementShelter) {
                    let c = intr.to_world(0.0, intr.depth * 0.48);
                    tf.translation = Vec3::new(c.x, light_base_y + 2.7, -c.y);
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            IndoorLightKind::Stove => {
                if active_intr_floor == 0 {
                    if let Some(st) = active_intr.filter(|it| it.stove_fuel_s > 0.0).and_then(|it| it.stove_at) {
                        tf.translation = Vec3::new(st.x, 0.85, -st.y);
                        *vis = Visibility::Inherited;
                    } else {
                        *vis = Visibility::Hidden;
                    }
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            IndoorLightKind::Flashlight => {
                if fl_on {
                    let ahead = play.sim.player_at.add(fwd.mul(2.4 + 2.6 * fog));
                    tf.translation = Vec3::new(ahead.x, fl_base_y + 1.55 + 0.45 * fog, -ahead.y);
                    pl.intensity = 55_000.0 * bright_mul + 220_000.0 * fog;
                    pl.range = 14.0 * range_mul + 12.0 * fog;
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            IndoorLightKind::FlashlightBounce => {
                if fl_on && fog >= 0.08 && play.sim.active_interior.is_none() {
                    let ahead = play.sim.player_at.add(fwd.mul(6.8 + 4.5 * fog));
                    tf.translation = Vec3::new(ahead.x, fl_base_y + 1.9, -ahead.y);
                    pl.intensity = 320_000.0 * fog;
                    pl.range = 22.0 + 16.0 * fog;
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }

    let fl_origin = Vec3::new(play.sim.player_at.x + fwd.x * 0.45, fl_base_y + 1.38, -(play.sim.player_at.y + fwd.y * 0.45));
    let fl_aim = Vec3::new(play.sim.player_at.x + fwd.x * 13.0, fl_base_y + 0.15, -(play.sim.player_at.y + fwd.y * 13.0));
    for (mut tf, mut sp, mut vis) in &mut spot_q {
        if fl_on {
            *tf = Transform::from_translation(fl_origin).looking_at(fl_aim, Vec3::Y);
            sp.intensity = 650_000.0 * bright_mul;
            sp.range = 30.0 * range_mul;
            sp.inner_angle = 0.22 + 0.12 * fog;
            sp.outer_angle = 0.44 + 0.18 * fog;
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    if last_beam_fog.is_none_or(|prev| (fog - prev).abs() > 0.02) {
        *last_beam_fog = Some(fog);
        if let Some(mut bm) = materials.get_mut(&atlas.flashlight_beam_mat) {
            bm.base_color = Color::srgba(0.28 + 0.40 * fog, 0.34 + 0.44 * fog, 0.42 + 0.46 * fog, 1.0);
        }
        if let Some(mut hm) = materials.get_mut(&atlas.flashlight_halo_mat) {
            hm.base_color = Color::srgba(0.22 * fog, 0.28 * fog, 0.36 * fog, 1.0);
        }
    }

    let (beam_tf, halo_tf, bulb_tf) = if fl_on {
        let base_rot = Transform::from_translation(fl_origin).looking_at(fl_aim, Vec3::Y);
        let beam_len = 11.5 * range_mul;
        let beam_r = 2.4 + 2.6 * fog;
        let beam = base_rot.with_scale(Vec3::new(beam_r, beam_r, beam_len));
        let halo = if fog >= 0.08 {
            let halo_len = 13.5 + 11.5 * fog;
            let halo_r = 4.6 + 4.4 * fog;
            Some(base_rot.with_scale(Vec3::new(halo_r, halo_r, halo_len)))
        } else {
            None
        };
        (Some(beam), halo, Some(Transform::from_translation(fl_origin)))
    } else {
        (None, None, None)
    };
    upsert_slot(&mut commands, &mut pools.flashlight_beam, &mut pool_q, atlas.flashlight_beam_mesh.clone(), &atlas.flashlight_beam_mat, beam_tf);
    upsert_slot(&mut commands, &mut pools.flashlight_halo, &mut pool_q, atlas.flashlight_beam_mesh.clone(), &atlas.flashlight_halo_mat, halo_tf);
    upsert_slot(&mut commands, &mut pools.flashlight_bulb, &mut pool_q, atlas.flashlight_bulb_mesh.clone(), &atlas.flashlight_bulb_mat, bulb_tf);

    // Live hinged doors & cotton curtains within 48m of player (capped to closest 48 doors).
    let mut near_doors: Vec<(f32, u32, &sim::interior::RoomDoor)> = Vec::new();
    for (intr_idx, intr) in play.sim.interiors.iter().enumerate() {
        if (intr.origin.x - play.sim.player_at.x).abs() > 54.0 || (intr.origin.y - play.sim.player_at.y).abs() > 54.0 {
            continue;
        }
        let base_k = (intr_idx as u32) * 4;
        for (slot, door_opt) in [
            (0u32, Some(&intr.front_door)),
            (1u32, intr.back_door.as_ref()),
            (2u32, intr.inner_door.as_ref()),
        ] {
            if let Some(d) = door_opt {
                if !d.destroyed {
                    let dist = d.at.dist(play.sim.player_at);
                    if dist <= 48.0 {
                        near_doors.push((dist, base_k + slot, d));
                    }
                }
            }
        }
    }
    near_doors.sort_by(|a, b| a.0.total_cmp(&b.0));
    near_doors.truncate(48);
    let mut seen_doors = HashSet::new();
    for (_, key, door) in near_doors {
        let door_yaw = door.dir.y.atan2(door.dir.x);
        let scale_x = door.width / 1.4;
        let (mesh_h, tf) = if door.style == DoorStyle::CottonCurtain {
            let m = if door.open {
                atlas.curtain_open_mesh.clone()
            } else {
                atlas.curtain_closed_mesh.clone()
            };
            let t = Transform::from_xyz(door.at.x, 0.0, -door.at.y)
                .with_rotation(Quat::from_rotation_y(door_yaw))
                .with_scale(Vec3::new(scale_x, 1.0, 1.0));
            (m, t)
        } else {
            let mi = match door.style {
                DoorStyle::ShopGlass => 1,
                DoorStyle::SteelSecurity => 2,
                _ => 3,
            };
            let hinge = door.at.sub(door.dir.mul(door.width * 0.5));
            let cross_in = door.dir.x * (-door.out.y) - door.dir.y * (-door.out.x);
            let swing = if door.open {
                if cross_in >= 0.0 { 1.48 } else { -1.48 }
            } else {
                0.0
            };
            let t = Transform::from_xyz(hinge.x, 0.0, -hinge.y)
                .with_rotation(Quat::from_rotation_y(door_yaw + swing))
                .with_scale(Vec3::new(scale_x, 1.0, 1.0));
            (atlas.door_meshes[mi].clone(), t)
        };
        upsert_pooled(&mut commands, &mut pools.live_doors, &mut pool_q, key, mesh_h, &mats.matte, tf, &mut seen_doors);
    }
    prune_pool(&mut commands, &mut pools.live_doors, &seen_doors);

    seen.clear();
    for c in &play.sim.caches {
        if !c.collected && play.sim.hour >= c.unlock_hour {
            let tf = Transform::from_xyz(c.at.x, 0.0, -c.at.y);
            if let Some(scene_h) = &atlas.cache_scene {
                upsert_pooled_scene(&mut commands, &mut pools.caches, &mut scene_pool_q, c.id, scene_h.clone(), tf, &mut seen);
            } else {
                upsert_pooled(&mut commands, &mut pools.caches, &mut pool_q, c.id, atlas.cache_mesh.clone(), &mats.matte, tf, &mut seen);
            }
        }
    }
    prune_pool(&mut commands, &mut pools.caches, &seen);

    seen.clear();
    for (di, d) in play.sim.drops.iter().enumerate() {
        if !d.taken {
            let tf = Transform::from_xyz(d.at.x, 0.0, -d.at.y);
            if let Some(scene_h) = &atlas.drop_scene {
                upsert_pooled_scene(&mut commands, &mut pools.drops, &mut scene_pool_q, di as u32, scene_h.clone(), tf, &mut seen);
            } else {
                upsert_pooled(&mut commands, &mut pools.drops, &mut pool_q, di as u32, atlas.drop_mesh.clone(), &mats.matte, tf, &mut seen);
            }
        }
    }
    prune_pool(&mut commands, &mut pools.drops, &seen);

    seen.clear();
    for tr in &play.sim.tracks {
        let yaw = tr.dir.y.atan2(tr.dir.x);
        let fade = (1.0 - tr.age / 120.0).clamp(0.35, 1.0);
        if tr.is_vehicle {
            let tf = Transform::from_xyz(tr.at.x, 0.028, -tr.at.y)
                .with_rotation(Quat::from_rotation_y(yaw + std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(1.0, 1.0, fade));
            upsert_pooled(&mut commands, &mut pools.tracks, &mut pool_q, tr.id, atlas.veh_track_mesh.clone(), &mats.matte, tf, &mut seen);
        } else {
            let side = if tr.id % 2 == 0 { 0.14 } else { -0.14 };
            let perp = P2::new(-tr.dir.y, tr.dir.x);
            let pos = tr.at.add(perp.mul(side));
            let tf = Transform::from_xyz(pos.x, 0.025, -pos.y)
                .with_rotation(Quat::from_rotation_y(yaw + std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(fade, 1.0, fade));
            upsert_pooled(&mut commands, &mut pools.tracks, &mut pool_q, tr.id, atlas.track_mesh.clone(), &mats.matte, tf, &mut seen);
        }
    }
    prune_pool(&mut commands, &mut pools.tracks, &seen);

    seen.clear();
    for c in &play.sim.craters {
        let tf = Transform::from_xyz(c.at.x, 0.018, -c.at.y).with_scale(Vec3::new(c.radius, 1.0, c.radius));
        upsert_pooled(&mut commands, &mut pools.craters, &mut pool_q, c.id, atlas.crater_mesh.clone(), &mats.matte, tf, &mut seen);
    }
    prune_pool(&mut commands, &mut pools.craters, &seen);

    seen.clear();
    let mut seen_wtur = HashSet::new();
    let mut seen_wemb = HashSet::new();
    let mut seen_wwhl = HashSet::new();
    for w in &play.sim.wrecks {
        let mi = w.kind.index();
        let hull_tf = Transform::from_xyz(w.at.x, -0.05, -w.at.y)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, w.hull_yaw + std::f32::consts::FRAC_PI_2, 0.04, 0.06));
        upsert_pooled(&mut commands, &mut pools.wrecks_hull, &mut pool_q, w.id, atlas.tank_hull_meshes[mi].clone(), &atlas.wreck_charred_mat, hull_tf, &mut seen);

        let turret_y = w.kind.turret_mount_y();
        let turret_tf = Transform::from_xyz(w.at.x, turret_y - 0.04, -w.at.y)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, w.turret_yaw + std::f32::consts::FRAC_PI_2 + 0.3, -0.06, 0.05));
        upsert_pooled(&mut commands, &mut pools.wrecks_turret, &mut pool_q, w.id, atlas.tank_turret_meshes[mi].clone(), &atlas.wreck_charred_mat, turret_tf, &mut seen_wtur);

        if w.smoke_ttl > 0.0 {
            let emb_tf = Transform::from_xyz(w.at.x, turret_y * 0.52, -w.at.y);
            upsert_pooled(&mut commands, &mut pools.wrecks_ember, &mut pool_q, w.id, atlas.ember_mesh.clone(), &atlas.ember_mat, emb_tf, &mut seen_wemb);
        }
        let wheel_off = P2::new((w.id as f32 * 1.7).cos() * 2.5, (w.id as f32 * 2.3).sin() * 2.5);
        let wp = w.at.add(wheel_off);
        let wheel_tf = Transform::from_xyz(wp.x, 0.20, -wp.y)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 1.4, w.hull_yaw, 0.25));
        upsert_pooled(&mut commands, &mut pools.wrecks_wheel, &mut pool_q, w.id, atlas.wheel_mesh.clone(), &mats.matte, wheel_tf, &mut seen_wwhl);
    }
    prune_pool(&mut commands, &mut pools.wrecks_hull, &seen);
    prune_pool(&mut commands, &mut pools.wrecks_turret, &seen_wtur);
    prune_pool(&mut commands, &mut pools.wrecks_ember, &seen_wemb);
    prune_pool(&mut commands, &mut pools.wrecks_wheel, &seen_wwhl);

    seen.clear();
    for sc in &play.sim.smoke_clouds {
        let fade_s = (sc.ttl / 3.0).clamp(0.45, 1.0);
        let xz = sc.radius * 0.62 * fade_s;
        let yy = (sc.radius * 0.34 * fade_s).clamp(1.2, 2.8);
        let tf = Transform::from_xyz(sc.at.x, 0.15, -sc.at.y)
            .with_rotation(Quat::from_rotation_y((sc.id % 12) as f32 * 0.5))
            .with_scale(Vec3::new(xz, yy, xz));
        let mat = if sc.dark { &atlas.smoke_dark_mat } else { &atlas.smoke_white_mat };
        upsert_pooled(&mut commands, &mut pools.smoke_clouds, &mut pool_q, sc.id, atlas.smoke_puff_mesh.clone(), mat, tf, &mut seen);
    }
    prune_pool(&mut commands, &mut pools.smoke_clouds, &seen);

    seen.clear();
    for b in &play.sim.blood_marks {
        let fade = (1.0 - b.age / 160.0).clamp(0.45, 1.0);
        let tf = Transform::from_xyz(b.at.x, 0.026, -b.at.y)
            .with_rotation(Quat::from_rotation_y(b.yaw + std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(fade, 1.0, fade));
        upsert_pooled(&mut commands, &mut pools.blood_marks, &mut pool_q, b.id, atlas.blood_mesh.clone(), &atlas.blood_mat, tf, &mut seen);
    }
    prune_pool(&mut commands, &mut pools.blood_marks, &seen);

    seen.clear();
    for g in &play.sim.grenades {
        let u = (g.flight_t / g.flight_dur.max(0.05)).clamp(0.0, 1.0);
        let arc_y = g.floor as f32 * FLOOR_H + 0.12 + (u * std::f32::consts::PI).sin() * 2.8;
        let tf = Transform::from_xyz(g.at.x, arc_y, -g.at.y)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, u * 8.0, g.id as f32, u * 5.0));
        upsert_pooled(&mut commands, &mut pools.grenades, &mut pool_q, g.id, atlas.grenade_mesh.clone(), &mats.matte, tf, &mut seen);
    }
    prune_pool(&mut commands, &mut pools.grenades, &seen);

    if play.started && !play.paused {
        *smoke_acc += time.delta_secs();
        if *smoke_acc >= 0.32 {
            *smoke_acc = 0.0;
            for w in &play.sim.wrecks {
                if w.smoke_ttl > 0.0 && w.at.dist(play.sim.player_at) <= 130.0 {
                    commands.spawn((
                        Mesh3d(atlas.smoke_puff_mesh.clone()),
                        MeshMaterial3d(atlas.smoke_dark_mat.clone()),
                        Transform::from_xyz(w.at.x, 1.45, -w.at.y).with_scale(Vec3::splat(1.15)),
                        TransientFx { ttl: 2.2, vel: Vec3::new(0.45, 2.2, -0.25), grow: 0.45 },
                    ));
                }
            }
            for t in &play.sim.tanks {
                if t.hp > 0.0 && t.hp < t.kind.max_hp() * 0.45 && t.at.dist(play.sim.player_at) <= 130.0 {
                    commands.spawn((
                        Mesh3d(atlas.smoke_puff_mesh.clone()),
                        MeshMaterial3d(atlas.smoke_dark_mat.clone()),
                        Transform::from_xyz(t.at.x, 1.35, -t.at.y).with_scale(Vec3::splat(0.85)),
                        TransientFx { ttl: 1.5, vel: Vec3::new(0.35, 1.7, -0.2), grow: 0.35 },
                    ));
                }
            }
        }
    }

    if pools.barricades.len() > play.sim.barricades.len() {
        for (e, _) in pools.barricades.drain(play.sim.barricades.len()..) {
            commands.entity(e).despawn();
        }
    }
    for (bi, b) in play.sim.barricades.iter().enumerate() {
        let mut tf = Transform::from_xyz(b.at.x, 0.0, -b.at.y)
            .with_rotation(Quat::from_rotation_y(b.yaw + std::f32::consts::FRAC_PI_2));
        if b.shoved {
            tf = tf.with_rotation(Quat::from_euler(EulerRot::XYZ, 0.4, b.yaw + std::f32::consts::FRAC_PI_2 + 0.6, 1.1));
        }
        let idx = barricade_mesh_idx(b.kind);
        let (yaw_fix, sc) = atlas.bar_scene_adjust[idx];
        let scene_tf = tf * Transform::from_rotation(Quat::from_rotation_y(yaw_fix)).with_scale(Vec3::splat(sc));
        if bi < pools.barricades.len() && pools.barricades[bi].1 == b.kind {
            if atlas.bar_scenes[idx].is_some() {
                if let Ok((mut t, mut v)) = scene_pool_q.get_mut(pools.barricades[bi].0) {
                    *t = scene_tf;
                    *v = Visibility::Inherited;
                }
            } else if let Ok((mut t, _, mut v)) = pool_q.get_mut(pools.barricades[bi].0) {
                *t = tf;
                *v = Visibility::Inherited;
            }
            continue;
        }
        if bi < pools.barricades.len() {
            commands.entity(pools.barricades[bi].0).despawn();
        }
        let e = if let Some(scene_h) = &atlas.bar_scenes[idx] {
            if b.kind == BarricadeKind::FireBarrel {
                commands.spawn((
                    WorldAssetRoot(scene_h.clone()),
                    scene_tf,
                    Visibility::Inherited,
                    PointLight {
                        color: Color::srgb(1.0, 0.45, 0.12),
                        intensity: 48_000.0,
                        range: 13.0,
                        shadow_maps_enabled: false,
                        ..default()
                    },
                    PooledScene,
                )).id()
            } else {
                commands.spawn((
                    WorldAssetRoot(scene_h.clone()),
                    scene_tf,
                    Visibility::Inherited,
                    PooledScene,
                )).id()
            }
        } else if b.kind == BarricadeKind::FireBarrel {
            commands.spawn((
                Mesh3d(atlas.bar_meshes[idx].clone()),
                MeshMaterial3d(mats.matte.clone()),
                tf,
                PointLight {
                    color: Color::srgb(1.0, 0.45, 0.12),
                    intensity: 48_000.0,
                    range: 13.0,
                    shadow_maps_enabled: false,
                    ..default()
                },
                PooledActor,
            )).id()
        } else {
            commands.spawn((
                Mesh3d(atlas.bar_meshes[idx].clone()),
                MeshMaterial3d(mats.matte.clone()),
                tf,
                PooledActor,
            )).id()
        };
        if bi < pools.barricades.len() {
            pools.barricades[bi] = (e, b.kind);
        } else {
            pools.barricades.push((e, b.kind));
        }
    }

    let drone_tf = play.sim.drone.active.then(|| {
        Transform::from_xyz(play.sim.drone.at.x, play.sim.drone.altitude, -play.sim.drone.at.y)
    });
    if let Some(sh) = &atlas.drone_scene {
        upsert_scene_slot(&mut commands, &mut pools.drone, &mut scene_pool_q, sh.clone(), drone_tf);
    } else {
        upsert_slot(&mut commands, &mut pools.drone, &mut pool_q, atlas.drone_mesh.clone(), &mats.matte, drone_tf);
    }

    let fpv_tf = play.sim.fpv.as_ref().map(|fpv| {
        Transform::from_xyz(fpv.at.x, fpv.height, -fpv.at.y)
    });
    if let Some(sh) = &atlas.drone_scene {
        upsert_scene_slot(&mut commands, &mut pools.fpv, &mut scene_pool_q, sh.clone(), fpv_tf);
    } else {
        upsert_slot(&mut commands, &mut pools.fpv, &mut pool_q, atlas.drone_mesh.clone(), &mats.matte, fpv_tf);
    }

    let fibre_tf = play.sim.fpv.as_ref().filter(|f| f.fibre_optic).map(|fpv| {
        let a = Vec3::new(fpv.at.x, fpv.height, -fpv.at.y);
        let b = if fpv.origin.dist(fpv.at) >= 2.0 {
            Vec3::new(fpv.origin.x, 2.2, -fpv.origin.y)
        } else {
            Vec3::new(fpv.at.x - 10.0, 2.2, -(fpv.at.y + 80.0))
        };
        let mid = (a + b) * 0.5;
        let len = a.distance(b).max(0.2);
        Transform::from_translation(mid).looking_at(b, Vec3::Y).with_scale(Vec3::new(1.0, 1.0, len))
    });
    upsert_slot(&mut commands, &mut pools.fpv_fibre, &mut pool_q, atlas.fibre_mesh.clone(), &atlas.fibre_mat, fibre_tf);

    let lancet_tf = play.sim.lancet.as_ref().map(|lan| {
        let d = lan.target.sub(lan.at);
        let yaw = if d.len() > 0.1 { d.y.atan2(d.x) } else { 0.0 };
        Transform::from_xyz(lan.at.x, lan.height, -lan.at.y)
            .with_rotation(Quat::from_rotation_y(yaw + std::f32::consts::FRAC_PI_2))
    });
    if let Some(sh) = &atlas.lancet_scene {
        upsert_scene_slot(&mut commands, &mut pools.lancet, &mut scene_pool_q, sh.clone(), lancet_tf.map(|t| t * Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI))));
    } else {
        upsert_slot(&mut commands, &mut pools.lancet, &mut pool_q, atlas.lancet_mesh.clone(), &mats.matte, lancet_tf);
    }

    let friendly_fpv_tf = play.sim.friendly_fpv.as_ref().map(|ffpv| {
        Transform::from_xyz(ffpv.at.x, ffpv.height, -ffpv.at.y)
    });
    if let Some(sh) = &atlas.drone_scene {
        upsert_scene_slot(&mut commands, &mut pools.friendly_fpv, &mut scene_pool_q, sh.clone(), friendly_fpv_tf);
    } else {
        upsert_slot(&mut commands, &mut pools.friendly_fpv, &mut pool_q, atlas.friendly_fpv_mesh.clone(), &mats.matte, friendly_fpv_tf);
    }

    let mut seen_hulls = HashSet::new();
    let mut seen_turrets = HashSet::new();
    let mut seen_lasers = HashSet::new();
    for t in &play.sim.tanks {
        let mi = t.kind.index();
        let hull_tf = Transform::from_xyz(t.at.x, 0.0, -t.at.y)
            .with_rotation(Quat::from_rotation_y(t.hull_yaw + std::f32::consts::FRAC_PI_2));
        let turret_y = t.kind.turret_mount_y();
        if let Some(scene_h) = &atlas.tank_scenes[mi] {
            let (tyaw, tsc) = atlas.tank_scene_adjust[mi];
            let scene_tf = hull_tf * Transform::from_rotation(Quat::from_rotation_y(tyaw)).with_scale(Vec3::splat(tsc));
            upsert_pooled_scene(&mut commands, &mut pools.tank_hulls, &mut scene_pool_q, t.id, scene_h.clone(), scene_tf, &mut seen_hulls);
        } else {
            upsert_pooled(&mut commands, &mut pools.tank_hulls, &mut pool_q, t.id, atlas.tank_hull_meshes[mi].clone(), &mats.matte, hull_tf, &mut seen_hulls);
            let turret_tf = Transform::from_xyz(t.at.x, turret_y, -t.at.y)
                .with_rotation(Quat::from_rotation_y(t.turret_yaw + std::f32::consts::FRAC_PI_2));
            upsert_pooled(&mut commands, &mut pools.tank_turrets, &mut pool_q, t.id, atlas.tank_turret_meshes[mi].clone(), &mats.matte, turret_tf, &mut seen_turrets);
        }

        if let Some((tgt, tf_floor)) = t.locked_target {
            if t.lock_timer > 0.05 {
                let a = Vec3::new(t.at.x, turret_y + 0.30, -t.at.y);
                let b = Vec3::new(tgt.x, tf_floor as f32 * FLOOR_H + 1.0, -tgt.y);
                let mid = (a + b) * 0.5;
                let len = a.distance(b).max(0.2);
                let laser_tf = Transform::from_translation(mid)
                    .looking_at(b, Vec3::Y)
                    .with_scale(Vec3::new(1.0, 1.0, len));
                let lmat = if t.faction == Faction::Pla { &atlas.pla_laser_mat } else { &atlas.laser_mat };
                upsert_pooled(&mut commands, &mut pools.tank_lasers, &mut pool_q, t.id, atlas.laser_mesh.clone(), lmat, laser_tf, &mut seen_lasers);
            }
        }
    }
    prune_pool(&mut commands, &mut pools.tank_hulls, &seen_hulls);
    prune_pool(&mut commands, &mut pools.tank_turrets, &seen_turrets);
    prune_pool(&mut commands, &mut pools.tank_lasers, &seen_lasers);

    let ghost_idx = barricade_mesh_idx(play.sim.selected_barricade);
    let ghost_tf = if play.preview_barricade && !play.sim.dead {
        let target = play.cursor_world.unwrap_or_else(|| {
            play.sim.player_at.add(P2::new(play.sim.player_facing.cos() * 2.6, play.sim.player_facing.sin() * 2.6))
        });
        let delta = target.sub(play.sim.player_at);
        let d = delta.len();
        let spot = if d > 5.5 {
            play.sim.player_at.add(delta.mul(5.5 / d))
        } else if d < 0.4 {
            play.sim.player_at.add(P2::new(play.sim.player_facing.cos() * 2.2, play.sim.player_facing.sin() * 2.2))
        } else {
            target
        };
        let yaw = if d > 0.25 { delta.y.atan2(delta.x) + std::f32::consts::FRAC_PI_2 } else { play.sim.player_facing + std::f32::consts::FRAC_PI_2 };
        Some(Transform::from_xyz(spot.x, 0.02, -spot.y).with_rotation(Quat::from_rotation_y(yaw)))
    } else {
        None
    };
    upsert_slot(&mut commands, &mut pools.ghost_preview, &mut pool_q, atlas.bar_meshes[ghost_idx].clone(), &atlas.ghost_mat, ghost_tf);

    let mut ring_pos: Option<(P2, f32)> = None;
    for d in &play.sim.doors {
        if !d.searched {
            let dist = d.at.dist(play.sim.player_at);
            if dist <= 18.0 && ring_pos.is_none_or(|(_, best)| dist < best) {
                ring_pos = Some((d.at, dist));
            }
        }
    }
    for c in &play.sim.caches {
        if !c.collected && play.sim.hour >= c.unlock_hour {
            let dist = c.at.dist(play.sim.player_at);
            if dist <= 18.0 && ring_pos.is_none_or(|(_, best)| dist < best) {
                ring_pos = Some((c.at, dist));
            }
        }
    }
    let ring_tf = ring_pos.map(|(p, _)| Transform::from_xyz(p.x, 0.03, -p.y));
    upsert_slot(&mut commands, &mut pools.door_ring, &mut pool_q, atlas.ring_mesh.clone(), &atlas.ring_mat, ring_tf);

    let entry_tf = if play.sim.active_interior.is_none() {
        play.sim.nearest_enterable_door(play.sim.player_at, 24.0).map(|(_, d_at, d_out)| {
            let p = d_at.lerp(d_out, 0.35);
            Transform::from_xyz(p.x, 0.04, -p.y).with_scale(Vec3::splat(0.56))
        })
    } else {
        None
    };
    upsert_slot(
        &mut commands,
        &mut pools.entry_ring,
        &mut pool_q,
        atlas.ring_mesh.clone(),
        &atlas.entry_ring_mat,
        entry_tf,
    );

    // White circular selection rings under friendly militia + command destination ring.
    let mut seen_m_rings = HashSet::new();
    for m in &play.sim.militia {
        if m.hp > 0.0 {
            let my = if m.floor > 0
                || play.sim.interiors.iter().any(|it| {
                    (it.origin.x - m.at.x).abs() < 18.0 && (it.origin.y - m.at.y).abs() < 18.0 && it.contains(m.at)
                })
            {
                m.floor as f32 * FLOOR_H + 0.09
            } else {
                0.03
            };
            let tf = Transform::from_xyz(m.at.x, my, -m.at.y).with_scale(Vec3::splat(0.95));
            upsert_pooled(
                &mut commands,
                &mut pools.militia_rings,
                &mut pool_q,
                m.id,
                atlas.circle_ring_mesh.clone(),
                &atlas.white_ring_mat,
                tf,
                &mut seen_m_rings,
            );
        }
    }
    prune_pool(&mut commands, &mut pools.militia_rings, &seen_m_rings);

    // Light-blue X-ray silhouette when player or friendly militia take cover underneath a TarpDugout.
    let mut seen_xray = HashSet::new();
    if !play.sim.dead && play.sim.under_tarp_dugout(play.sim.player_at) {
        let variant = u8::from(play.sim.body.coat_bonus > 0.0) | (u8::from(play.sim.inv.has_rifle) << 1);
        if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::Civilian, variant, anim: AnimState::Crouch, frame: 0 }) {
            let tf = Transform::from_xyz(play.sim.player_at.x, 1.88, -play.sim.player_at.y)
                .with_rotation(Quat::from_rotation_y(play.sim.player_facing + std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(1.02));
            upsert_pooled(&mut commands, &mut pools.xray_silhouettes, &mut pool_q, 0, h.clone(), &atlas.xray_mat, tf, &mut seen_xray);
        }
    }
    for (mi, m) in play.sim.militia.iter().enumerate() {
        if m.hp > 0.0 && play.sim.under_tarp_dugout(m.at) {
            let xanim = if m.downed { AnimState::Prone } else { AnimState::Crouch };
            if let Some(h) = atlas.meshes.get(&AtlasKey { role: Role::Militia, variant: (mi % 4) as u8, anim: xanim, frame: 0 }) {
                let tf = Transform::from_xyz(m.at.x, 1.88, -m.at.y)
                    .with_rotation(Quat::from_rotation_y(m.facing + std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(1.02));
                upsert_pooled(&mut commands, &mut pools.xray_silhouettes, &mut pool_q, m.id, h.clone(), &atlas.xray_mat, tf, &mut seen_xray);
            }
        }
    }
    prune_pool(&mut commands, &mut pools.xray_silhouettes, &seen_xray);

    let cmd_tf = play.sim.militia_cmd_marker.map(|(p, _)| {
        Transform::from_xyz(p.x, 0.04, -p.y).with_scale(Vec3::splat(1.35))
    });
    upsert_slot(
        &mut commands,
        &mut pools.cmd_ring,
        &mut pool_q,
        atlas.circle_ring_mesh.clone(),
        &atlas.white_ring_mat,
        cmd_tf,
    );

    // Thin red circular warning rings on incoming FPV / Lancet / mortar / grenade impact zones.
    let mut seen_danger = HashSet::new();
    if let Some(fpv) = &play.sim.fpv {
        let tf = Transform::from_xyz(fpv.target.x, 0.05, -fpv.target.y).with_scale(Vec3::splat(2.8));
        upsert_pooled(&mut commands, &mut pools.danger_rings, &mut pool_q, 100_000, atlas.circle_ring_mesh.clone(), &atlas.danger_ring_mat, tf, &mut seen_danger);
    }
    if let Some(lan) = &play.sim.lancet {
        let tf = Transform::from_xyz(lan.target.x, 0.05, -lan.target.y).with_scale(Vec3::splat(3.5));
        upsert_pooled(&mut commands, &mut pools.danger_rings, &mut pool_q, 100_001, atlas.circle_ring_mesh.clone(), &atlas.danger_ring_mat, tf, &mut seen_danger);
    }
    for (mi, m) in play.sim.mortars.iter().enumerate() {
        if !m.friendly && m.fuse <= 2.5 {
            let tf = Transform::from_xyz(m.target.x, 0.05, -m.target.y).with_scale(Vec3::splat(5.5));
            upsert_pooled(&mut commands, &mut pools.danger_rings, &mut pool_q, 200_000 + mi as u32, atlas.circle_ring_mesh.clone(), &atlas.danger_ring_mat, tf, &mut seen_danger);
        }
    }
    for g in &play.sim.grenades {
        let pulse = 1.0 + 0.08 * (g.fuse * 10.0).sin();
        let gy = g.floor as f32 * FLOOR_H + 0.05;
        let tf = Transform::from_xyz(g.target.x, gy, -g.target.y).with_scale(Vec3::splat(6.5 * pulse));
        upsert_pooled(&mut commands, &mut pools.danger_rings, &mut pool_q, 300_000 + g.id, atlas.circle_ring_mesh.clone(), &atlas.danger_ring_mat, tf, &mut seen_danger);
    }
    prune_pool(&mut commands, &mut pools.danger_rings, &seen_danger);
}

/// Rain falls from a sky that hides the sun. The winter's flurries and river fog
/// keep the light they were drawn with.
fn overcast(w: Weather) -> f32 {
    if w == Weather::Rain { 1.0 } else { 0.0 }
}

pub fn sync_fx(
    mut commands: Commands,
    time: Res<Time>,
    play: Option<Res<PlayState>>,
    player: Single<&Player>,
    mut q: Query<(Entity, &mut TransientFx, &mut Transform), Without<SnowParticle>>,
    mut snow_q: Query<(&mut SnowParticle, &mut Transform, &mut Visibility), Without<TransientFx>>,
) {
    let dt = time.delta_secs().clamp(0.0, 0.1);
    for (e, mut fx, mut tf) in &mut q {
        fx.ttl -= dt;
        tf.translation += fx.vel * dt;
        if fx.grow != 0.0 {
            let s = (1.0 + fx.grow * dt).max(0.1);
            tf.scale *= Vec3::splat(s);
        }
        if fx.ttl <= 0.0 {
            commands.entity(e).despawn();
        }
    }
    let show_snow = play.as_deref().is_some_and(|p| p.started && !matches!(p.sim.weather, Weather::Clear));
    // The same flakes, in summer rain: long streaks that fall several times faster and straight.
    let rain = play.as_deref().is_some_and(|p| p.sim.weather == Weather::Rain);
    let paused = play.as_deref().is_some_and(|p| p.paused);
    let px = player.at.x;
    let pz = -player.at.y;
    for (mut sp, mut tf, mut vis) in &mut snow_q {
        if !show_snow {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        tf.scale = if rain { Vec3::new(0.35, 7.0, 0.35) } else { Vec3::ONE };
        if !paused {
            if rain {
                sp.offset.y -= sp.speed * 4.0 * dt;
            } else {
                sp.offset.y -= sp.speed * dt;
                sp.offset.x += (time.elapsed_secs() * 1.4 + sp.phase).sin() * 0.9 * dt;
                sp.offset.z += 0.65 * dt;
            }
            if sp.offset.y < 0.1 {
                sp.offset.y = 22.0;
            }
            if sp.offset.x > 24.0 { sp.offset.x -= 48.0; }
            if sp.offset.x < -24.0 { sp.offset.x += 48.0; }
            if sp.offset.z > 24.0 { sp.offset.z -= 48.0; }
            if sp.offset.z < -24.0 { sp.offset.z += 48.0; }
        }
        tf.translation = Vec3::new(px + sp.offset.x, sp.offset.y, pz + sp.offset.z);
    }
}

pub fn sync_hud(
    play: Res<PlayState>,
    menu: Res<MenuScreen>,
    clock: Res<Clock>,
    interactive: Res<Interactive>,
    mut q: Query<(&HudRole, &mut Text, &mut TextColor, &mut Visibility)>,
    mut speed_bar_q: Query<&mut Visibility, (With<SpeedBarRoot>, Without<HudRole>)>,
    mut speed_btns: Query<(&SpeedBtn, &mut BackgroundColor)>,
    mut pause_txt: Query<&mut Text, (With<PauseBtnText>, Without<HudRole>, Without<FlashlightBtnText>)>,
    mut fl_txt: Query<&mut Text, (With<FlashlightBtnText>, Without<HudRole>, Without<PauseBtnText>)>,
) {
    let s = &play.sim;
    let active = play.started && *menu == MenuScreen::Playing;
    let show_speed = interactive.play && active && !s.dead && !s.sleeping && !s.passing;
    for mut v in &mut speed_bar_q {
        *v = if show_speed { Visibility::Inherited } else { Visibility::Hidden };
    }
    if show_speed {
        for (btn, mut bg) in &mut speed_btns {
            if (btn.0 + 2.0).abs() < 0.01 {
                bg.0 = if s.flashlight_on {
                    Color::srgba(0.24, 0.50, 0.78, 0.94)
                } else {
                    Color::srgba(0.05, 0.08, 0.13, 0.82)
                };
            } else if btn.0 < 0.0 {
                bg.0 = if play.paused {
                    Color::srgba(0.72, 0.24, 0.16, 0.94)
                } else {
                    Color::srgba(0.14, 0.36, 0.64, 0.92)
                };
            } else if btn.0 == 0.0 {
                bg.0 = Color::srgba(0.14, 0.24, 0.34, 0.88);
            } else if !play.paused && (btn.0 - clock.speed).abs() < 0.1 {
                bg.0 = Color::srgba(0.66, 0.20, 0.14, 0.94);
            } else {
                bg.0 = Color::srgba(0.05, 0.08, 0.13, 0.82);
            }
        }
        for mut pt in &mut pause_txt {
            pt.0 = if play.paused {
                "⏸ 已暂停 | P/Space 继续".into()
            } else {
                "蓝方 BLUE | 暂停 P".into()
            };
        }
        for mut ft in &mut fl_txt {
            ft.0 = if s.flashlight_on { "手电 L: 开" } else { "手电 L: 关" }.into();
        }
    }
    for (role, mut text, mut color, mut vis) in &mut q {
        match role {
            HudRole::Objective => {
                *vis = if active && !s.dead && !s.sleeping && !s.passing { Visibility::Inherited } else { Visibility::Hidden };
                let bar = s.selected_barricade.label();
                let (cs, cc) = s.selected_barricade.cost();
                let pla_arm = s.tanks.iter().filter(|t| t.faction == Faction::Pla).count();
                let ru_arm = s.tanks.iter().filter(|t| t.faction == Faction::Russia).count();
                text.0 = format!(
                    "{}\n民兵 {} 人 · 我方装甲 {} · 俄军装甲 {} · 废铁 {} · 煤 {} · 柴 {} · 当前工事 [{} 铁{}煤{}]",
                    s.objective_zh,
                    s.militia.iter().filter(|m| m.hp > 0.0).count(),
                    pla_arm,
                    ru_arm,
                    s.inv.scrap,
                    s.inv.coal,
                    s.inv.wood,
                    bar,
                    cs,
                    cc
                );
            }
            HudRole::TopRight => {
                *vis = if active && !s.dead { Visibility::Inherited } else { Visibility::Hidden };
                let wx_tag = match s.weather {
                    Weather::Clear if s.outdoor_c() < 0.0 => "晴冷",
                    Weather::Clear => "晴",
                    Weather::Flurries => "小雪飘扬",
                    Weather::RiverFog => "江雾弥漫",
                    Weather::Rain => "下雨",
                };
                let curfew = if s.curfew_now() { "  ·  宵禁中" } else { "" };
                let fl_tag = if s.flashlight_on {
                    if s.fog_density >= 0.25 { " · [手电·雾中增亮]" } else { " · [手电]" }
                } else {
                    ""
                };
                if let Some(i_idx) = s.active_interior {
                    let intr = &s.interiors[i_idx];
                    let env_tag = if intr.stove_fuel_s > 0.0 {
                        "煤炉供暖"
                    } else if intr.has_unboarded_broken_window() {
                        "破窗灌风"
                    } else if intr.room_temp_c >= 10.0 {
                        "室内有暖"
                    } else {
                        "停暖阴冷"
                    };
                    let floor_tag = if intr.num_floors > 1 {
                        format!(" [{}F/{}F]", s.player_floor + 1, intr.num_floors)
                    } else {
                        String::new()
                    };
                    let cur_tag = if intr.curtains_closed { " · [窗帘合]" } else { "" };
                    text.0 = format!(
                        "体温 {:.1}°C  ·  {wx_tag}{curfew}\n{}{floor_tag}  {:+.1}°C ({env_tag}{cur_tag}{fl_tag})",
                        s.body.temp, intr.name, intr.room_temp_c
                    );
                } else {
                    let state_tag = if s.body.on_ice {
                        "江面寒风"
                    } else if s.fire_barrels.iter().any(|b| b.dist(s.player_at) <= 3.5) {
                        "火桶取暖"
                    } else if s.body.sheltered && s.body.heated {
                        "门脸供暖"
                    } else if s.body.sheltered {
                        "21:00停暖 挡风"
                    } else {
                        "街头"
                    };
                    text.0 = format!("体温 {:.1}°C  ·  {wx_tag}{curfew}\n室外 {:+.0}°C ({state_tag}{fl_tag})", s.body.temp, s.outdoor_c());
                }
                color.0 = if s.body.temp < 34.4 {
                    Color::srgb(0.55, 0.80, 1.0)
                } else if s.body.temp < 35.4 {
                    Color::srgb(1.0, 0.78, 0.45)
                } else {
                    Color::WHITE
                };
            }
            HudRole::Notice => {
                let msg = if s.tank_lock_warning {
                    "警告！俄军坦克主炮锁定你！快横向撤离或防楼板塌陷！"
                } else if s.under_fire_for > 0.0 {
                    "他们在朝你开枪！快找遮挡"
                } else if s.challenged {
                    "被喝令：Стой！站住别动，或拔腿逃跑"
                } else if s.lancet.is_some() {
                    "警告！俄军柳叶刀-3巡飞弹俯冲袭来！快进室内或击落！"
                } else if let Some(fpv) = &s.fpv {
                    if fpv.fibre_optic {
                        "FPV自杀机逼近！光纤不可干扰 · 快找遮挡"
                    } else {
                        "FPV自杀机逼近！快找门脸躲避"
                    }
                } else if s.drone.active && s.notice < 0.2 {
                    "头顶有侦察无人机"
                } else if s.notice >= 0.05 {
                    let bars = (s.notice * 5.0).ceil() as usize;
                    let on = "▮".repeat(bars.min(5));
                    let off = "▯".repeat(5 - bars.min(5));
                    text.0 = format!("有人在往这边看  {on}{off}");
                    *vis = if active { Visibility::Inherited } else { Visibility::Hidden };
                    continue;
                } else {
                    ""
                };
                text.0 = msg.into();
                *vis = if active && !msg.is_empty() { Visibility::Inherited } else { Visibility::Hidden };
            }
            HudRole::Prompt => {
                *vis = if active && !s.dead { Visibility::Inherited } else { Visibility::Hidden };
                let sub = if s.subtitle_ttl > 0.0 { &s.subtitle_zh } else { "" };
                text.0 = match (s.prompt.is_empty(), sub.is_empty()) {
                    (false, false) => format!("{}\n{}", s.prompt, sub),
                    (false, true) => s.prompt.clone(),
                    (true, false) => sub.into(),
                    (true, true) => String::new(),
                };
            }
            HudRole::RadioFeed => {
                if active && !s.dead && !s.sleeping && !s.passing {
                    if let Some(rf) = &s.radio_feed {
                        text.0 = format!("【{}】 {}  ·  {}", rf.unit, rf.code_en, rf.text_zh);
                        color.0 = if rf.urgent {
                            Color::srgb(1.0, 0.45, 0.38)
                        } else {
                            Color::srgb(0.62, 0.86, 1.0)
                        };
                        *vis = Visibility::Inherited;
                    } else if play.paused {
                        text.0 = "【战术暂停】 PAUSED  ·  WASD 移动视角 · 右键指挥民兵 · Home 复位 · P/Space 继续".into();
                        color.0 = Color::srgb(1.0, 0.86, 0.46);
                        *vis = Visibility::Inherited;
                    } else if !s.militia.is_empty() {
                        text.0 = "【蓝队频道】 AWAITING ORDERS  ·  右键点击地面或地图指派民兵".into();
                        color.0 = Color::srgb(0.72, 0.84, 0.95);
                        *vis = Visibility::Inherited;
                    } else {
                        *vis = Visibility::Hidden;
                    }
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            HudRole::Stats => {
                *vis = if active && !s.dead && !s.sleeping && !s.passing { Visibility::Inherited } else { Visibility::Hidden };
                let bleed = if s.body.bleeding { "  [流血]" } else { "" };
                let coat = if s.body.coat_bonus > 0.0 { "  [厚棉袄]" } else { "" };
                let gun = if s.inv.has_rifle {
                    format!("  [主武器N: {} 弹{}]", s.inv.active_primary.short_label(), s.inv.ammo)
                } else {
                    String::new()
                };
                let at_cur = s.inv.at_ammo(s.inv.active_at);
                let at_tot = s.inv.total_at_ammo();
                let at_tag = if at_tot > 0 {
                    format!("  [反坦U/F: {}×{} (总{})]", s.inv.active_at.short_label(), at_cur, at_tot)
                } else {
                    String::new()
                };
                let gre_tag = if s.inv.grenades > 0 { format!("  手雷(Y){}", s.inv.grenades) } else { String::new() };
                let smk_tag = if s.inv.smoke_grenades > 0 { format!("  烟雾(O){}", s.inv.smoke_grenades) } else { String::new() };
                let fpv_tag = if s.inv.friendly_fpv > 0 { format!("  FPV(I){}", s.inv.friendly_fpv) } else { String::new() };
                let rad = if s.inv.has_radio {
                    if s.radio_cd > 0.5 {
                        format!("  [电台R×{} 冷却{:.0}s]", s.inv.radio_strikes, s.radio_cd)
                    } else {
                        format!("  [电台R×{}可用]", s.inv.radio_strikes)
                    }
                } else {
                    String::new()
                };
                let ill = if s.body.illness >= 5.0 { format!("   病 {:.0}", s.body.illness) } else { String::new() };
                text.0 = format!(
                    "伤势 {:.0}{bleed}   饱 {:.0}   水 {:.0}   困 {:.0}{ill}{coat}{gun}{at_tag}{gre_tag}{smk_tag}{fpv_tag}{rad}\nQ绷带{}/药{} Z吃{} X水{} C暖{} V火瓶{}   N换枪 U换炮 F发射 Y雷 O烟 I放FPV R电台   1-8/B工事 T顶门/炉 K窗帘 L手电   J睡觉/过日子   P/空格暂停   Space/Ctrl上下楼   M地图 Esc菜单",
                    s.body.hp, s.body.hunger, s.body.thirst, s.body.fatigue, s.inv.bandage, s.inv.medicine, s.inv.food, s.inv.water, s.inv.warmer, s.inv.molotov
                );
            }
            HudRole::Sms => {
                if let Some(sms) = &s.sms {
                    text.0 = format!("【短信】{}\n{}", sms.sender, sms.zh);
                    *vis = if active { Visibility::Inherited } else { Visibility::Hidden };
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            HudRole::Death => {
                if active && s.passing && !s.dead {
                    let room = s.active_interior.map(|i| &s.interiors[i]);
                    text.0 = format!(
                        "日子一天天过去……\n2031年{}  ·  第{}天{}\n\n吃的 {} 份 · 水 {} 瓶 · 药 {}\n煤 {} · 柴 {} · 屋里还有家具 {} 件 · 屋里 {:+.0}°C\n饱 {:.0} · 水 {:.0} · 病 {:.0}\n\n有事会停下   按 J 停",
                        sim::clock::label(s.hour),
                        sim::clock::day_index(s.hour) + 1,
                        if s.sleeping { "  ·  夜里睡着" } else { "" },
                        s.inv.food,
                        s.inv.water,
                        s.inv.medicine,
                        s.inv.coal,
                        s.inv.wood,
                        room.map(|r| r.furniture).unwrap_or(0),
                        room.map(|r| r.room_temp_c).unwrap_or(s.outdoor_c()),
                        s.body.hunger,
                        s.body.thirst,
                        s.body.illness
                    );
                    *vis = Visibility::Inherited;
                } else if active && s.sleeping && !s.dead {
                    text.0 = format!(
                        "睡着了\n2031年{}  ·  已睡 {:.1} 小时\n\n有动静会醒   按 J 起来",
                        sim::clock::label(s.hour),
                        s.slept_hours
                    );
                    *vis = Visibility::Inherited;
                } else if active && s.dead {
                    let h = s.hours_alive as u32;
                    let m = ((s.hours_alive - h as f32) * 60.0) as u32;
                    let lasted = if h >= 24 { format!("{} 天 {} 小时", h / 24, h % 24) } else { format!("{h} 小时 {m} 分") };
                    let taken = if s.detentions > 0 {
                        format!("  ·  被带走 {} 次，关了 {:.0} 天", s.detentions, s.days_detained)
                    } else {
                        String::new()
                    };
                    text.0 = format!(
                        "{}\n{}\n\n2031年{}  ·  撑了 {lasted}{taken}\n\n按 Enter 重新开始   或按 Esc 返回主菜单",
                        s.death_zh,
                        s.death_en,
                        sim::clock::label(s.hour)
                    );
                    *vis = Visibility::Inherited;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}
