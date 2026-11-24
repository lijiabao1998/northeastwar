//! Northeast: The Russian Invasion. Heihe, drawn.
//!
//! This crate only shows things and reads keys. What Heihe is, what colour it
//! is and where you can walk are decided in the `sim` crate.

mod audio;
mod bench;
mod boards;
mod graphics;
mod map_ui;
mod menu_ui;
mod play;
mod shots;
mod sky;

use bevy::{
    asset::RenderAssetUsages,
    camera::ScalingMode,
    input::mouse::MouseWheel,
    mesh::Indices,
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use sim::{city::City, geom::P2, look::Palette, soup::Soup, walk::Ground};

/// The map, the colours and the ground you can stand on.
#[derive(Resource)]
pub struct Heihe {
    pub city: City,
    pub palette: Palette,
    pub ground: Ground,
}

/// The war clock: hours since midnight on 17 January 2031.
#[derive(Resource)]
pub struct Clock {
    pub hour: f32,
    pub speed: f32,
    pub running: bool,
    pub blackout: bool,
    pub fog: f32,
}

#[derive(Component)]
pub struct Player {
    pub at: P2,
}

#[derive(Component)]
pub struct Eye {
    pub zoom: f32,
    pub cam_pan: P2,
}

#[derive(Component)]
struct ClockText;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Solid,
    Window,
}

/// A piece of a building that is cut away when the building hides you.
#[derive(Component)]
struct Cut {
    /// The building up to its parapet, and its crown if it has one (sim::cutaway::Occluder).
    occluder: std::sync::Arc<sim::cutaway::Occluder>,
    building_id: i64,
    part: Part,
    hidden: bool,
    peeled: bool,
}

/// The speck that keeps the cut-away's shader built (see setup).
#[derive(Component)]
struct Speck;

/// The materials buildings are drawn with, for --bench to draw them merged.
#[derive(Resource)]
struct Mats {
    matte: Handle<StandardMaterial>,
    glass: Handle<StandardMaterial>,
    glow: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct Cutaway {
    view: sim::cutaway::View,
    solid: Handle<StandardMaterial>,
    ghost: Handle<StandardMaterial>,
}

/// Buildings between the camera and you turn to glass; enterable buildings peel completely off.
fn cut_away(
    cutaway: Option<Res<Cutaway>>,
    play: Option<Res<play::PlayState>>,
    player: Single<&Player>,
    mut parts: Query<(&mut Cut, &mut MeshMaterial3d<StandardMaterial>, &mut Visibility)>,
) {
    let Some(cutaway) = cutaway else { return };
    let peel_bid = play
        .as_deref()
        .and_then(|p| p.sim.cutaway_interior.and_then(|i| p.sim.interiors.get(i)).map(|it| it.building_id));
    for (mut cut, mut material, mut visibility) in &mut parts {
        let hide = cut.occluder.hides(player.at, &cutaway.view, 2.5);
        let peel = peel_bid == Some(cut.building_id);
        if hide == cut.hidden && peel == cut.peeled {
            continue;
        }
        cut.hidden = hide;
        cut.peeled = peel;
        if peel {
            *visibility = Visibility::Hidden;
        } else {
            match cut.part {
                Part::Solid => {
                    *visibility = Visibility::Inherited;
                    material.0 = if hide { cutaway.ghost.clone() } else { cutaway.solid.clone() };
                }
                Part::Window => {
                    *visibility = if hide { Visibility::Hidden } else { Visibility::Inherited };
                }
            }
        }
    }
}

pub const VIEW_METRES: f32 = 46.0;
/// The camera stands south-east of you and looks north-west, so the river is up and to the right.
/// It is orthographic, so its distance changes nothing on screen (see sim::cutaway::EYE_OFFSET).
const EYE_OFFSET: Vec3 = Vec3::from_array(sim::cutaway::EYE_OFFSET);

pub fn to_mesh(soup: &Soup) -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, soup.pos.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, soup.nor.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, soup.uv.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, soup.col.clone())
        .with_inserted_indices(Indices::U32(soup.idx.clone()))
}

fn arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let (city, palette) = match sim::load_heihe() {
        Ok(found) => found,
        Err(why) => {
            eprintln!("Heihe would not load: {why}");
            std::process::exit(1);
        }
    };
    let interiors = sim::interior::build_interiors(&city);
    let ground = Ground::new_with_interiors(&city, &interiors);
    let assets = sim::data_dir().parent().map(|p| p.join("assets")).unwrap_or_else(|| "assets".into());
    let shots_dir = arg("--shots");
    let bench_dir = arg("--bench");
    let combat_dir = arg("--combat-shots");
    let menu_dir = arg("--menu-shots");
    let interior_dir = arg("--interior-shots");
    // Taking pictures or timing: no window on anyone's screen, and no waiting for the
    // screen's refresh, which would put a floor of one refresh (16.7 ms at 60 Hz) under every frame.
    let unattended = shots_dir.is_some()
        || bench_dir.is_some()
        || combat_dir.is_some()
        || menu_dir.is_some()
        || interior_dir.is_some();
    let interactive = !unattended;
    let default_hour = if interactive || menu_dir.is_some() { sim::survival::START_HOUR } else { 13.0 };
    let hour = arg("--hour").and_then(|h| h.parse::<f32>().ok()).unwrap_or(default_hour);

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "东北：俄罗斯入侵 — 黑河".into(),
                    resolution: (1600, 900).into(),
                    visible: !unattended,
                    present_mode: if unattended { bevy::window::PresentMode::AutoNoVsync } else { bevy::window::PresentMode::Fifo },
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin { file_path: assets.to_string_lossy().into_owned(), ..default() }),
    )
    .insert_resource(Heihe { city, palette, ground })
    .insert_resource(graphics::Graphics::from_env())
    .insert_resource(Clock { hour, speed: 1.0, running: interactive && arg("--hour").is_none(), blackout: false, fog: 0.0 })
    .insert_resource(ClearColor(Color::srgb(0.62, 0.70, 0.80)))
    .add_systems(Startup, (setup, sky::setup))
    .add_systems(Update, (zoom, follow, tick, sky::apply, sky::follow_zoom.after(zoom), hud, cut_away));
    if unattended {
        app.add_systems(Update, walk);
        // A hidden window may count as unfocused, which would hold it to 60 updates a second.
        app.insert_resource(bevy::winit::WinitSettings::continuous());
    }
    audio::install(&mut app, interactive);
    play::install(&mut app, interactive, combat_dir.is_some() || interior_dir.is_some());
    map_ui::install(&mut app, interactive || combat_dir.is_some() || interior_dir.is_some(), interactive);
    menu_ui::install(&mut app, interactive || menu_dir.is_some(), interactive || menu_dir.is_some());
    if let Some(dir) = shots_dir {
        shots::install(&mut app, dir);
    } else if let Some(dir) = bench_dir {
        bench::install(&mut app, dir);
    } else if let Some(dir) = combat_dir {
        shots::install_combat(&mut app, dir);
    } else if let Some(dir) = menu_dir {
        shots::install_menu(&mut app, dir);
    } else if let Some(dir) = interior_dir {
        shots::install_interior(&mut app, dir);
    }
    app.run();
}

pub fn setup(
    mut commands: Commands,
    heihe: Res<Heihe>,
    gfx: Res<graphics::Graphics>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
) {
    let scene = sim::scene::build(&heihe.city, &heihe.palette);

    let matte = materials.add(StandardMaterial { perceptual_roughness: 0.92, reflectance: 0.18, ..default() });
    let snowy = materials.add(StandardMaterial { perceptual_roughness: 0.82, reflectance: 0.3, ..default() });
    let glass = materials.add(StandardMaterial { perceptual_roughness: 0.18, reflectance: 0.6, ..default() });
    // Their light is in real units, so the camera's exposure must apply to it:
    // a lit window is invisible at noon and bright at night, as it is.
    // A lit room behind glass and a curtain: a few tens of candelas per square metre.
    let window_glow = materials.add(StandardMaterial {
        perceptual_roughness: 0.4,
        emissive: LinearRgba::new(22.0, 14.5, 6.6, 1.0),
        emissive_exposure_weight: 1.0,
        ..default()
    });
    let lamp_glow = materials.add(StandardMaterial { perceptual_roughness: 0.5, emissive_exposure_weight: 1.0, ..default() });

    commands.spawn((Mesh3d(meshes.add(to_mesh(&scene.ground))), MeshMaterial3d(snowy)));
    commands.spawn((Mesh3d(meshes.add(to_mesh(&scene.buildings))), MeshMaterial3d(matte.clone())));
    // The Chinese bank, one building at a time, so the one in front of you can be cut away.
    let ghost = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.16),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.9,
        unlit: !gfx.ghost_lit,
        ..default()
    });
    let view = sim::cutaway::View::from_offset(EYE_OFFSET.to_array());
    for block in &scene.blocks {
        // The same occluder the sim's tests check.
        let occluder = std::sync::Arc::new(sim::cutaway::Occluder::of_block(block, &view));
        commands.spawn((
            Mesh3d(meshes.add(to_mesh(&block.solid))),
            MeshMaterial3d(matte.clone()),
            Cut { occluder: occluder.clone(), building_id: block.id, part: Part::Solid, hidden: false, peeled: false },
        ));
        if block.glass.triangles() > 0 {
            commands.spawn((
                Mesh3d(meshes.add(to_mesh(&block.glass))),
                MeshMaterial3d(glass.clone()),
                Cut { occluder: occluder.clone(), building_id: block.id, part: Part::Window, hidden: false, peeled: false },
            ));
        }
        if block.lit.triangles() > 0 {
            commands.spawn((
                Mesh3d(meshes.add(to_mesh(&block.lit))),
                MeshMaterial3d(window_glow.clone()),
                Cut { occluder, building_id: block.id, part: Part::Window, hidden: false, peeled: false },
            ));
        }
    }
    // A speck of the cut-away material that is always on screen, so its shader is built at the
    // start. Otherwise the first time a building is cut away the whole frame comes out empty.
    // Made by to_mesh like every building, so it has the same vertex layout and the
    // pipeline built for it is the one a cut-away building uses.
    let mut dot = Soup::default();
    dot.boxed(P2::new(0.0, 0.0), 0.0, [0.02, 0.02, 0.02], 0.0, sim::look::Rgb { r: 1.0, g: 1.0, b: 1.0 });
    let speck = commands.spawn((Mesh3d(meshes.add(to_mesh(&dot))), MeshMaterial3d(ghost.clone()), Transform::from_xyz(0.0, 0.01, 0.0), Speck)).id();
    commands.insert_resource(Cutaway { view, solid: matte.clone(), ghost });
    for chunk in scene.props.chunks(120.0) {
        commands.spawn((Mesh3d(meshes.add(to_mesh(&chunk))), MeshMaterial3d(matte.clone())));
    }
    if scene.glass.triangles() > 0 {
        commands.spawn((Mesh3d(meshes.add(to_mesh(&scene.glass))), MeshMaterial3d(glass.clone())));
    }
    if scene.lit.triangles() > 0 {
        commands.spawn((Mesh3d(meshes.add(to_mesh(&scene.lit))), MeshMaterial3d(window_glow.clone())));
    }
    for chunk in scene.lamp_glow.chunks(120.0) {
        commands.spawn((Mesh3d(meshes.add(to_mesh(&chunk))), MeshMaterial3d(lamp_glow.clone())));
    }
    commands.insert_resource(Mats { matte: matte.clone(), glass: glass.clone(), glow: window_glow.clone() });
    commands.insert_resource(sky::GlowMaterials { lamps: lamp_glow, windows: window_glow.clone() });

    let sodium = heihe.palette.get("lamp_sodium");
    // One light per lamp head, or one per post as bright as its heads together.
    let lights: Vec<([f32; 3], f32)> = if gfx.lamp_per_head {
        scene.lamp_heads.iter().map(|h| (*h, 1.0)).collect()
    } else {
        scene.lamp_posts.iter().map(|(at, heads)| (*at, *heads as f32)).collect()
    };
    for (head, heads) in &lights {
        commands.spawn((
            // A 250 W sodium lamp gives about 28,000 lumens. sky::apply shows the lights after dark.
            PointLight {
                color: Color::srgb(sodium.r, sodium.g, sodium.b),
                intensity: 62_000.0 * heads,
                range: 22.0,
                radius: graphics::lamp_radius(&gfx),
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(head[0], head[1], head[2]),
            Visibility::Hidden,
            sky::StreetLamp,
        ));
    }

    boards::spawn(&mut commands, &scene.boards, &mut meshes, &mut materials, &mut images);

    // The articulated figure is replaced in play::setup_play, starting with a seed mesh here.
    let me = sim::figure::build_figure(&sim::figure::FigureOpts::default(), &sim::figure::pose(sim::figure::AnimState::Idle, 0.0, 0.0, 0.0));
    let spawn = heihe.city.spawn;
    commands.spawn((
        Mesh3d(meshes.add(to_mesh(&me))),
        MeshMaterial3d(matte),
        Transform::from_xyz(spawn.x, 0.0, -spawn.y),
        Player { at: spawn },
    )).add_child(speck);

    let eye = commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { viewport_height: VIEW_METRES },
            near: graphics::near_plane(1.0),
            far: 2200.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(Vec3::new(spawn.x, 0.0, -spawn.y) + EYE_OFFSET).looking_at(Vec3::new(spawn.x, 1.0, -spawn.y), Vec3::Y),
        gfx.msaa,
        gfx.shadows,
        bevy::camera::Exposure { ev100: 12.0 },
        bevy::pbr::DistanceFog {
            color: Color::NONE,
            falloff: bevy::pbr::FogFalloff::Linear { start: 900.0, end: 1200.0 },
            ..default()
        },
        Eye { zoom: 1.0, cam_pan: P2::default() },
    )).id();
    if gfx.prepass {
        commands.entity(eye).insert(bevy::core_pipeline::prepass::DepthPrepass);
    }

    let font: Handle<Font> = assets.load("fonts/NotoSansSC-VF.ttf");
    commands.spawn((
        Text::new(""),
        TextFont { font: font.clone().into(), font_size: bevy::text::FontSize::Px(22.0), ..default() },
        TextColor(Color::WHITE),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, top: Val::Px(20.0), left: Val::Px(26.0), ..default() },
        ClockText,
    ));
    commands.spawn((
        Text::new("地图数据 © OpenStreetMap 贡献者（ODbL）· 建筑轮廓 CMAB（CC BY 4.0）   WASD 走 · Shift 跑 · 滚轮 缩放 · Tab / +- / [] 调速(1x-16x)"),
        TextFont { font: font.into(), font_size: bevy::text::FontSize::Px(13.0), ..default() },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
        TextShadow::default(),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(10.0), left: Val::Px(16.0), ..default() },
    ));
}

fn walk(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, heihe: Res<Heihe>, mut player: Single<(&mut Player, &mut Transform)>) {
    // Screen up is north-west on the map, screen right is north-east.
    let up = P2::new(-0.7071, 0.7071);
    let right = P2::new(0.7071, 0.7071);
    let mut wish = P2::new(0.0, 0.0);
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        wish = wish.add(up);
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        wish = wish.sub(up);
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        wish = wish.add(right);
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        wish = wish.sub(right);
    }
    if wish.len() < 0.1 {
        return;
    }
    let speed = if keys.pressed(KeyCode::ShiftLeft) { 6.0 } else { 3.4 };
    let (who, transform) = &mut *player;
    let to = who.at.add(wish.norm().mul(speed * time.delta_secs()));
    who.at = heihe.ground.step(who.at, to, 0.35);
    transform.translation = Vec3::new(who.at.x, 0.0, -who.at.y);
    let face = wish.norm();
    // The figure is built facing map south, which is +z in the world.
    transform.rotation = Quat::from_rotation_y(face.x.atan2(-face.y));
}

fn zoom(
    mut wheel: MessageReader<MouseWheel>,
    mut eye: Single<(&mut Eye, &mut Projection)>,
    menu: Option<Res<menu_ui::MenuScreen>>,
    map: Option<Res<map_ui::TacticalMap>>,
) {
    if menu.is_some_and(|m| *m != menu_ui::MenuScreen::Playing) || map.is_some_and(|m| m.open) {
        wheel.clear();
        return;
    }
    let mut turn = 0.0;
    for m in wheel.read() {
        turn += m.y;
    }
    if turn == 0.0 {
        return;
    }
    let (eye, projection) = &mut *eye;
    eye.zoom = (eye.zoom * 1.1f32.powf(turn)).clamp(0.12, 3.0);
    set_zoom(projection, eye.zoom);
}

pub fn set_zoom(projection: &mut Projection, zoom: f32) {
    if let Projection::Orthographic(ortho) = projection {
        ortho.scaling_mode = ScalingMode::FixedVertical { viewport_height: VIEW_METRES / zoom };
        ortho.near = graphics::near_plane(zoom);
    }
}

fn follow(player: Single<&Transform, (With<Player>, Without<Eye>)>, mut eye: Single<(&Eye, &mut Transform)>) {
    let (eye_c, eye_tf) = &mut *eye;
    let target = player.translation + Vec3::new(eye_c.cam_pan.x, 1.0, -eye_c.cam_pan.y);
    eye_tf.translation = target + EYE_OFFSET;
    eye_tf.look_at(target, Vec3::Y);
}

fn tick(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut clock: ResMut<Clock>,
    play: Option<Res<play::PlayState>>,
    menu: Option<Res<menu_ui::MenuScreen>>,
) {
    if menu.is_some_and(|m| *m != menu_ui::MenuScreen::Playing) {
        return;
    }
    if play.is_some() {
        return;
    }
    if clock.running {
        clock.hour += time.delta_secs() * clock.speed * 15.0 / 3600.0;
    }
    // [ and ] turn the clock by hand, to look at the city in any light.
    if keys.pressed(KeyCode::BracketRight) {
        clock.hour = (clock.hour + time.delta_secs() * 2.0).clamp(0.0, 31.0);
    }
    if keys.pressed(KeyCode::BracketLeft) {
        clock.hour = (clock.hour - time.delta_secs() * 2.0).clamp(0.0, 31.0);
    }
}

fn hud(clock: Res<Clock>, player: Single<&Player>, heihe: Res<Heihe>, mut text: Single<&mut Text, With<ClockText>>) {
    let total_h = clock.hour.max(0.0) as u32;
    let day = 17 + total_h / 24;
    let h = total_h % 24;
    let m = ((clock.hour.max(0.0) - total_h as f32) * 60.0) as u32;
    // The nearest named street, so you know where in Heihe you are.
    let mut street = "";
    let mut best = 40.0;
    for road in heihe.city.roads.iter().filter(|r| !r.name.is_empty()) {
        let d = sim::geom::dist_to_polyline(player.at, &road.line);
        if d < best {
            best = d;
            street = &road.name;
        }
    }
    let ice = if heihe.ground.on_ice(player.at) { "黑龙江冰面" } else { street };
    let spd = if clock.speed > 1.05 { format!("  [{}x加速]", clock.speed as u32) } else { String::new() };
    let line = format!("2031年1月{day}日  黑河\n{h:02}:{m:02}{spd}  {ice}    北 ↗");
    // Only when it changes: setting the text lays it out again.
    if text.0 != line {
        text.0 = line;
    }
}
