//! `northeast --bench <folder>`: what each part of drawing Heihe costs.
//!
//! At some of the --shots views it changes one thing at a time from the run's
//! graphics settings (graphics.rs) and times a run of frames with each. The
//! variants take turns, round after round; each round turns the order and every
//! other round reverses it, so no variant always follows the same one (what one
//! leaves behind once made the next read up to 15 ms high) and load from other
//! work falls on all of them. The first round only warms up (new pipelines are
//! built then) and is not counted. Variants that resize the window or swap the
//! whole town's meshes run last, after the counted rounds of the others.
//!
//! For each frame it records the frame time, the main thread's time from First to
//! Last, the render thread's time through its Render schedule (which includes
//! encoding, submitting and presenting), and the GPU's own timestamps per render
//! pass. Results go to bench.txt (a table) and bench.csv (every frame).
//!
//! NORTHEAST_BENCH_VIEWS picks views by their number ("01,07"),
//! NORTHEAST_BENCH_VARIANTS the variants (by Variant::key), and
//! NORTHEAST_BENCH_ROUNDS the number of counted rounds (3).

use crate::{graphics::Graphics, set_zoom, shots, Clock, Cut, Cutaway, Eye, Heihe, Mats, Part, Player, Speck};
use bevy::{
    anti_alias::fxaa::Fxaa,
    core_pipeline::prepass::DepthPrepass,
    diagnostic::DiagnosticsStore,
    light::{
        cluster::{ClusterConfig, ClusterZConfig},
        NotShadowCaster, ShadowFilteringMethod,
    },
    prelude::*,
    render::{renderer::RenderDevice, Render, RenderApp, RenderSystems},
    window::PrimaryWindow,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Variant {
    Base,
    MsaaOff,
    Msaa2,
    Msaa4,
    MsaaOffFxaa,
    NoSunShadow,
    HardShadow,
    SoftShadow,
    MoonAngle,
    NoLamps,
    Range18,
    NearOne,
    Z8,
    GhostLit,
    GhostUnlit,
    WindowsNoShadow,
    NoTransparent,
    Merged,
    HalfSize,
    Prepass,
    NoPrepass,
    LampPoint,
    ZeroLamps,
    PerHead,
}

/// Every variant there is. Without NORTHEAST_BENCH_VARIANTS a short default set runs.
const ALL: [Variant; 24] = [
    Variant::Base,
    Variant::MsaaOff,
    Variant::Msaa2,
    Variant::Msaa4,
    Variant::MsaaOffFxaa,
    Variant::NoSunShadow,
    Variant::HardShadow,
    Variant::SoftShadow,
    Variant::MoonAngle,
    Variant::NoLamps,
    Variant::Range18,
    Variant::NearOne,
    Variant::Z8,
    Variant::GhostLit,
    Variant::GhostUnlit,
    Variant::WindowsNoShadow,
    Variant::NoTransparent,
    Variant::Merged,
    Variant::HalfSize,
    Variant::Prepass,
    Variant::NoPrepass,
    Variant::LampPoint,
    Variant::ZeroLamps,
    Variant::PerHead,
];

impl Variant {
    fn name(self) -> &'static str {
        match self {
            Variant::Base => "as set",
            Variant::MsaaOff => "no MSAA",
            Variant::Msaa2 => "MSAA x2",
            Variant::Msaa4 => "MSAA x4",
            Variant::MsaaOffFxaa => "no MSAA, FXAA",
            Variant::NoSunShadow => "no sun shadows",
            Variant::HardShadow => "hard shadows",
            Variant::SoftShadow => "soft shadows",
            Variant::MoonAngle => "sun at moon angle",
            Variant::NoLamps => "lamps hidden",
            Variant::Range18 => "lamp range 18 m",
            Variant::NearOne => "near plane 1 m",
            Variant::Z8 => "8 cluster slices",
            Variant::GhostLit => "ghost lit",
            Variant::GhostUnlit => "ghost unlit",
            Variant::WindowsNoShadow => "windows cast none",
            Variant::NoTransparent => "no boards, speck",
            Variant::Merged => "buildings merged",
            Variant::HalfSize => "half width+height",
            Variant::Prepass => "depth prepass",
            Variant::NoPrepass => "no depth prepass",
            Variant::LampPoint => "lamps radius 0",
            Variant::ZeroLamps => "lamps at 0 by day",
            Variant::PerHead => "one light a head",
        }
    }

    fn key(self) -> String {
        format!("{self:?}").to_lowercase()
    }

    /// Changes that leave state behind (a resized window, the whole town swapped).
    fn disruptive(self) -> bool {
        matches!(self, Variant::HalfSize | Variant::Merged)
    }
}

/// The order of the run, the same at every view: (round, variant). Round 0 warms up.
fn plan(variants: &[Variant], rounds: usize) -> Vec<(usize, Variant)> {
    let normal: Vec<Variant> = variants.iter().copied().filter(|v| !v.disruptive()).collect();
    let n = normal.len();
    let mut out = Vec::new();
    for r in 0..=rounds {
        let mut order: Vec<Variant> = (0..n).map(|k| normal[(k + r) % n]).collect();
        if r % 2 == 1 {
            order.reverse();
        }
        out.extend(order.into_iter().map(|v| (r, v)));
    }
    for v in variants.iter().copied().filter(|v| v.disruptive()) {
        out.extend((1..=rounds).map(|r| (r, v)));
    }
    out
}

fn chosen() -> Vec<Variant> {
    let list = std::env::var("NORTHEAST_BENCH_VARIANTS").unwrap_or_else(|_| "msaaoff,msaa4,hardshadow,softshadow,nolamps,halfsize".to_string());
    let keys: Vec<String> = list.split(',').map(|s| s.trim().to_lowercase()).collect();
    ALL.iter().copied().filter(|v| *v == Variant::Base || keys.contains(&v.key())).collect()
}

const WARM_UP_SECS: f32 = 10.0;
/// Frames after a switch before timing: pipelines, shadow maps and the swap chain settle.
const SETTLE: u32 = 20;
/// Frames timed per variant per round.
const MEASURE: u32 = 40;

/// The buildings of the Chinese bank merged into one mesh per material, hidden
/// unless the Merged variant is on: the same triangles in far fewer draws.
#[derive(Component)]
struct MergedMark;

/// One light per lamp head, hidden unless PerHead is on (spawned only when the run
/// has one light per post).
#[derive(Component)]
struct HeadLamp;

/// When the main thread and the render thread began and how long they took last.
#[derive(Default)]
struct Times {
    main_start: Option<Instant>,
    main_ms: f32,
    render_start: Option<Instant>,
    render_ms: f32,
}

/// Shared by the main world and the render world.
#[derive(Resource, Clone, Default)]
struct CpuTimes(Arc<Mutex<Times>>);

#[derive(Default)]
struct Cell {
    /// Median frame time of each counted round.
    round_ms: Vec<f32>,
    frame_ms: Vec<f32>,
    main_ms: Vec<f32>,
    render_ms: Vec<f32>,
    gpu_ms: Vec<f32>,
    passes: BTreeMap<String, Vec<f32>>,
}

#[derive(Resource)]
struct Bench {
    dir: String,
    variants: Vec<Variant>,
    plan: Vec<(usize, Variant)>,
    views: Vec<shots::View>,
    rounds: usize,
    view: usize,
    step: usize,
    frames: u32,
    measuring_since: Option<Instant>,
    round_frames: Vec<f32>,
    cells: BTreeMap<(usize, Variant), Cell>,
    csv: Vec<String>,
    /// The window's own physical size, restored after HalfSize.
    physical: Option<(u32, u32)>,
    /// The sun as sky.rs placed it, while MoonAngle has moved it.
    sun_home: Option<Transform>,
    device: String,
    /// Each street lamp's intensity as the game set it, put back after ZeroLamps.
    intensities: BTreeMap<Entity, f32>,
}

pub fn install(app: &mut App, dir: String) {
    let _ = std::fs::create_dir_all(&dir);
    let wanted: Option<Vec<String>> = std::env::var("NORTHEAST_BENCH_VIEWS").ok().map(|v| v.split(',').map(|s| s.trim().to_string()).collect());
    let wanted = wanted.unwrap_or_else(|| ["01", "03", "06", "08", "09", "10"].iter().map(|s| s.to_string()).collect());
    let views: Vec<shots::View> = shots::views().into_iter().filter(|v| wanted.iter().any(|w| v.name.starts_with(w.as_str()))).collect();
    let rounds = std::env::var("NORTHEAST_BENCH_ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
    let times = CpuTimes::default();
    let variants = chosen();
    app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .insert_resource(times.clone())
        .insert_resource(Bench {
            dir,
            plan: plan(&variants, rounds),
            variants,
            views,
            rounds,
            view: 0,
            step: 0,
            frames: 0,
            measuring_since: None,
            round_frames: Vec::new(),
            cells: BTreeMap::new(),
            csv: vec!["view,variant,round,frame_ms,main_ms,render_ms,gpu_ms".to_string()],
            intensities: BTreeMap::new(),
            physical: None,
            sun_home: None,
            device: String::new(),
        })
        .add_systems(Startup, merge_buildings.after(crate::setup))
        .add_systems(First, main_begin)
        .add_systems(Last, main_end)
        .add_systems(Update, run.after(crate::cut_away).after(crate::sky::apply));
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render
            .insert_resource(times)
            .add_systems(Render, render_begin.before(RenderSystems::ExtractCommands))
            .add_systems(Render, render_end.after(RenderSystems::PostCleanup));
    }
}

fn main_begin(t: Res<CpuTimes>) {
    t.0.lock().unwrap().main_start = Some(Instant::now());
}

fn main_end(t: Res<CpuTimes>) {
    let mut t = t.0.lock().unwrap();
    if let Some(s) = t.main_start {
        t.main_ms = s.elapsed().as_secs_f32() * 1000.0;
    }
}

fn render_begin(t: Res<CpuTimes>) {
    t.0.lock().unwrap().render_start = Some(Instant::now());
}

fn render_end(t: Res<CpuTimes>) {
    let mut t = t.0.lock().unwrap();
    if let Some(s) = t.render_start {
        t.render_ms = s.elapsed().as_secs_f32() * 1000.0;
    }
}

fn merge_buildings(mut commands: Commands, heihe: Res<Heihe>, mats: Res<Mats>, gfx: Res<Graphics>, mut meshes: ResMut<Assets<Mesh>>) {
    let scene = sim::scene::build(&heihe.city, &heihe.palette);
    let (mut solid, mut glass, mut lit) = (sim::soup::Soup::default(), sim::soup::Soup::default(), sim::soup::Soup::default());
    for b in &scene.blocks {
        solid.append(&b.solid);
        glass.append(&b.glass);
        lit.append(&b.lit);
    }
    for (soup, material) in [(solid, mats.matte.clone()), (glass, mats.glass.clone()), (lit, mats.glow.clone())] {
        if soup.triangles() > 0 {
            commands.spawn((Mesh3d(meshes.add(crate::to_mesh(&soup))), MeshMaterial3d(material), Visibility::Hidden, MergedMark));
        }
    }
    if !gfx.lamp_per_head {
        let sodium = heihe.palette.get("lamp_sodium");
        for head in &scene.lamp_heads {
            commands.spawn((
                PointLight {
                    color: Color::srgb(sodium.r, sodium.g, sodium.b),
                    intensity: 62_000.0,
                    range: 22.0,
                    radius: 0.2,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_xyz(head[0], head[1], head[2]),
                Visibility::Hidden,
                HeadLamp,
            ));
        }
    }
}

/// What the GPU and its driver offer that matters for these numbers.
fn describe(device: &RenderDevice) -> String {
    use bevy::render::render_resource::WgpuFeatures as F;
    let f = device.features();
    let has = |x: F| if f.contains(x) { "yes" } else { "no" };
    format!(
        "GPU features: timestamps {}, timestamps inside passes {}, multi-draw indirect count {}, depth clip control {}",
        has(F::TIMESTAMP_QUERY),
        has(F::TIMESTAMP_QUERY_INSIDE_PASSES),
        has(F::MULTI_DRAW_INDIRECT_COUNT),
        has(F::DEPTH_CLIP_CONTROL)
    )
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn run(
    mut commands: Commands,
    mut bench: ResMut<Bench>,
    // Grouped: a system takes at most sixteen parameters.
    (time, store, device, cpu, gfx): (Res<Time<Real>>, Res<DiagnosticsStore>, Option<Res<RenderDevice>>, Res<CpuTimes>, Res<Graphics>),
    mut clock: ResMut<Clock>,
    heihe: Res<Heihe>,
    mut player: Single<(&mut Player, &mut Transform), Without<crate::sky::Sun>>,
    mut eye: Single<(Entity, &mut Eye, &mut Projection, &mut Msaa)>,
    mut sun: Single<(&mut DirectionalLight, &mut Transform), (With<crate::sky::Sun>, Without<Player>)>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    windows: Query<(Entity, &Cut)>,
    cutaway: Res<Cutaway>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shown: ParamSet<(
        Query<(Entity, &mut Visibility, &mut PointLight), With<crate::sky::StreetLamp>>,
        Query<&mut Visibility, Or<(With<crate::boards::BoardMark>, With<Speck>)>>,
        Query<(&Cut, &mut Visibility)>,
        Query<&mut Visibility, With<MergedMark>>,
        Query<&mut Visibility, With<HeadLamp>>,
    )>,
    mut exit: MessageWriter<AppExit>,
    mut warm: Local<u32>,
) {
    *warm += 1;
    if time.elapsed_secs() < WARM_UP_SECS || *warm < 60 {
        return;
    }
    let bench = &mut *bench;
    if bench.device.is_empty() {
        if let Some(d) = device {
            bench.device = describe(&d);
        }
    }
    if bench.physical.is_none() {
        bench.physical = Some((window.resolution.physical_width(), window.resolution.physical_height()));
    }
    if bench.view >= bench.views.len() {
        let text = report(bench, &gfx);
        println!("{text}");
        let _ = std::fs::write(format!("{}/bench.txt", bench.dir), text);
        let _ = std::fs::write(format!("{}/bench.csv", bench.dir), bench.csv.join("\n") + "\n");
        exit.write(AppExit::Success);
        return;
    }
    let (round, variant) = bench.plan[bench.step];

    if bench.frames == 0 {
        if bench.step == 0 {
            let v = &bench.views[bench.view];
            let (who, transform) = &mut *player;
            let at = heihe.ground.step(v.at, v.at, 0.35);
            who.at = at;
            transform.translation = Vec3::new(at.x, 0.0, -at.y);
            clock.hour = v.hour;
            let (_, e, projection, _) = &mut *eye;
            e.zoom = v.zoom;
            set_zoom(projection, v.zoom);
        }
        // Everything back as the run's settings have it, then the one change.
        *eye.3 = match variant {
            Variant::MsaaOff | Variant::MsaaOffFxaa => Msaa::Off,
            Variant::Msaa2 => Msaa::Sample2,
            Variant::Msaa4 => Msaa::Sample4,
            _ => gfx.msaa,
        };
        if variant == Variant::MsaaOffFxaa {
            commands.entity(eye.0).insert(Fxaa::default());
        } else {
            commands.entity(eye.0).remove::<Fxaa>();
        }
        commands.entity(eye.0).insert(match variant {
            Variant::HardShadow => ShadowFilteringMethod::Hardware2x2,
            Variant::SoftShadow => ShadowFilteringMethod::Gaussian,
            _ => gfx.shadows,
        });
        let zoom = eye.1.zoom;
        set_zoom(&mut eye.2, zoom);
        if variant == Variant::NearOne {
            if let Projection::Orthographic(ortho) = &mut *eye.2 {
                ortho.near = 1.0;
            }
        }
        let prepass = match variant {
            Variant::Prepass => true,
            Variant::NoPrepass => false,
            _ => gfx.prepass,
        };
        if prepass {
            commands.entity(eye.0).insert(DepthPrepass);
        } else {
            commands.entity(eye.0).remove::<DepthPrepass>();
        }
        commands.entity(eye.0).insert(if variant == Variant::Z8 {
            ClusterConfig::FixedZ { total: 4096, z_slices: 8, z_config: ClusterZConfig::default(), dynamic_resizing: true }
        } else {
            ClusterConfig::default()
        });
        let unlit = match variant {
            Variant::GhostLit => false,
            Variant::GhostUnlit => true,
            _ => !gfx.ghost_lit && crate::sky::ghost_unlit(clock.hour),
        };
        if materials.get(&cutaway.ghost).is_some_and(|g| g.unlit != unlit) {
            if let Some(mut ghost) = materials.get_mut(&cutaway.ghost) {
                ghost.unlit = unlit;
            }
        }
        sun.0.shadow_maps_enabled = variant != Variant::NoSunShadow;
        let (w, h) = bench.physical.unwrap();
        if variant == Variant::HalfSize {
            window.resolution.set_physical_resolution(w / 2, h / 2);
        } else if window.resolution.physical_width() != w {
            window.resolution.set_physical_resolution(w, h);
        }
        for (entity, cut) in &windows {
            if cut.part == Part::Window {
                if variant == Variant::WindowsNoShadow {
                    commands.entity(entity).insert(NotShadowCaster);
                } else {
                    commands.entity(entity).remove::<NotShadowCaster>();
                }
            }
        }
        let vis = |off: bool| if off { Visibility::Hidden } else { Visibility::Inherited };
        // Lamps as sky.rs has them (hidden by day), unless the variant is about lamps.
        // ZeroLamps is how they were before: shown by day, at intensity 0.
        let lit = crate::sky::lamps_on(clock.hour);
        let base_radius = crate::graphics::lamp_radius(&gfx);
        let mut p0 = shown.p0();
        if bench.intensities.is_empty() {
            bench.intensities = p0.iter().map(|(e, _, l)| (e, l.intensity)).collect();
        }
        for (e, mut v, mut light) in &mut p0 {
            let shown_now = match variant {
                Variant::NoLamps | Variant::PerHead => false,
                Variant::ZeroLamps => true,
                _ => lit,
            };
            *v = vis(!shown_now);
            light.intensity = if variant == Variant::ZeroLamps && !lit { 0.0 } else { bench.intensities[&e] };
            light.range = if variant == Variant::Range18 { 18.0 } else { 22.0 };
            light.radius = if variant == Variant::LampPoint { 0.0 } else { base_radius };
        }
        for mut v in &mut shown.p4() {
            *v = vis(!(lit && variant == Variant::PerHead));
        }
        for mut v in &mut shown.p1() {
            *v = vis(variant == Variant::NoTransparent);
        }
        let merged = variant == Variant::Merged;
        for (cut, mut v) in &mut shown.p2() {
            *v = vis(merged || (cut.part == Part::Window && cut.hidden));
        }
        for mut v in &mut shown.p3() {
            *v = if merged { Visibility::Inherited } else { Visibility::Hidden };
        }
        bench.measuring_since = None;
        bench.round_frames.clear();
    }
    // The sun moved to where sky.rs puts the moon (48 degrees up, from 150 degrees),
    // light and exposure unchanged: does the angle alone change the cost?
    if variant == Variant::MoonAngle {
        if bench.sun_home.is_none() {
            bench.sun_home = Some(*sun.1);
        }
        let (el, az) = (48.0f32.to_radians(), 150.0f32.to_radians());
        let toward = Vec3::new(az.sin() * el.cos(), el.sin(), -az.cos() * el.cos());
        *sun.1 = Transform::from_translation(toward * 400.0).looking_at(Vec3::ZERO, Vec3::Y);
    } else if let Some(home) = bench.sun_home.take() {
        *sun.1 = home;
    }

    bench.frames += 1;
    if bench.frames == SETTLE {
        bench.measuring_since = Some(Instant::now());
    }
    if bench.frames > SETTLE && round > 0 {
        let since = bench.measuring_since.unwrap();
        let frame_ms = time.delta_secs() * 1000.0;
        let (main_ms, render_ms) = {
            let t = cpu.0.lock().unwrap();
            (t.main_ms, t.render_ms)
        };
        // The GPU's time per pass, from timestamps that arrived since timing began.
        // Only the top-level passes are summed; nested ones are listed under their parent.
        let mut gpu = 0.0f32;
        let mut passes: Vec<(String, f32)> = Vec::new();
        for d in store.iter() {
            let path = d.path().as_str();
            if !path.ends_with("/elapsed_gpu") {
                continue;
            }
            let Some(m) = d.measurement() else { continue };
            if m.time < since {
                continue;
            }
            // A pass that runs more than once a frame under one name (Bevy's shadow pass
            // runs early and late) leaves one measurement each, all with the sync's time.
            let t = m.time;
            let ms = d.measurements().filter(|x| x.time == t).map(|x| x.value).sum::<f64>() as f32;
            if path.split('/').count() == 3 {
                gpu += ms;
            }
            passes.push((path.trim_start_matches("render/").trim_end_matches("/elapsed_gpu").to_string(), ms));
        }
        let view = bench.view;
        let name = bench.views[view].name;
        let gpu_field = if passes.is_empty() { String::new() } else { format!("{gpu:.3}") };
        bench.csv.push(format!("{name},{},{round},{frame_ms:.3},{main_ms:.3},{render_ms:.3},{gpu_field}", variant.key()));
        bench.round_frames.push(frame_ms);
        let cell = bench.cells.entry((view, variant)).or_default();
        cell.frame_ms.push(frame_ms);
        cell.main_ms.push(main_ms);
        cell.render_ms.push(render_ms);
        if !passes.is_empty() {
            cell.gpu_ms.push(gpu);
        }
        for (p, ms) in passes {
            cell.passes.entry(p).or_default().push(ms);
        }
    }
    if bench.frames >= SETTLE + MEASURE {
        if round > 0 {
            let m = median(&bench.round_frames);
            let view = bench.view;
            bench.cells.entry((view, variant)).or_default().round_ms.push(m);
        }
        bench.frames = 0;
        bench.step += 1;
        if bench.step >= bench.plan.len() {
            bench.step = 0;
            bench.view += 1;
        }
    }
}

fn median(v: &[f32]) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    s[s.len() / 2]
}

fn report(bench: &Bench, gfx: &Graphics) -> String {
    let (w, h) = bench.physical.unwrap_or((0, 0));
    let build = if cfg!(debug_assertions) { "dev build (debug assertions on)" } else { "release build" };
    let mut out = vec![
        format!("northeast --bench, {build}, {w} x {h} physical pixels."),
        format!("Settings (graphics.rs): {}.", gfx.describe()),
        bench.device.clone(),
        format!(
            "{} counted rounds of {MEASURE} frames per variant, after {SETTLE} to settle. Medians in ms. main: First to Last on the main thread; render: the render thread's Render schedule (encoding, submitting, presenting); GPU: the sum of top-level passes by GPU timestamps (MSAA resolves and the swap chain are outside them).",
            bench.rounds
        ),
    ];
    for (i, v) in bench.views.iter().enumerate() {
        out.push(String::new());
        out.push(format!("{} (zoom {}, {:.1} h)", v.name, v.zoom, v.hour));
        out.push(format!("  {:<19} {:>6} {:>18} {:>6} {:>6} {:>6}   {}", "variant", "frame", "per round", "main", "render", "GPU", "GPU passes"));
        for &variant in &bench.variants {
            let Some(c) = bench.cells.get(&(i, variant)) else { continue };
            let rounds: Vec<String> = c.round_ms.iter().map(|m| format!("{m:.1}")).collect();
            let mut passes: Vec<(f32, &String)> = c.passes.iter().map(|(p, ms)| (median(ms), p)).collect();
            passes.sort_by(|a, b| b.0.total_cmp(&a.0));
            let top: Vec<String> = passes.iter().filter(|(ms, _)| *ms >= 0.1).take(8).map(|(ms, p)| format!("{p} {ms:.2}")).collect();
            out.push(format!(
                "  {:<19} {:>6.1} {:>18} {:>6.1} {:>6.1} {:>6.1}   {}",
                variant.name(),
                median(&c.frame_ms),
                rounds.join(" "),
                median(&c.main_ms),
                median(&c.render_ms),
                median(&c.gpu_ms),
                top.join(", ")
            ));
        }
    }
    out.join("\n") + "\n"
}
