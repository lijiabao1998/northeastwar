use bevy::{
    asset::RenderAssetUsages,
    input::mouse::MouseWheel,
    picking::prelude::{Pickable, PointerButton, PointerClick, PointerOut, PointerOver},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use sim::{
    city::{Bank, City},
    forces::Forces,
    geom::{dist_to_polyline, point_in_ring, P2},
    interior::InteriorKind,
    survival::{front_depth, Faction, WorldSim},
};

use crate::{play::PlayState, Heihe};

pub const BASE_PX: usize = 720;
pub const MAP_PX: usize = 360;

#[derive(Clone, Copy, Debug)]
enum MapAction {
    ZoomIn,
    ZoomOut,
    CenterPlayer,
    ResetFull,
}

#[derive(Resource, Default)]
struct MapActionQueue(Vec<MapAction>);

#[derive(Component)]
struct MapOverlay;

#[derive(Component)]
struct MapZoomLabel;

#[derive(Resource)]
pub struct TacticalMap {
    pub open: bool,
    pub handle: Handle<Image>,
    base_rgba: Vec<u8>,
    full_bounds: [f32; 4], // [min_x, max_x, min_y, max_y]
    pub zoom: f32,
    pub center: P2,
    pub follow_player: bool,
    pub pending_militia_order: Option<P2>,
    was_open: bool,
    refresh_timer: f32,
}

impl TacticalMap {
    pub fn set_view(&mut self, center: P2, zoom: f32) {
        self.zoom = zoom.clamp(1.0, 6.0);
        let [fx0, fx1, fy0, fy1] = self.full_bounds;
        let half = 0.5 * (fx1 - fx0).max(1.0) / self.zoom;
        let cx = center.x.clamp(fx0 + half, fx1 - half);
        let cy = center.y.clamp(fy0 + half, fy1 - half);
        self.center = P2::new(cx, cy);
    }

    pub fn view_bounds(&self) -> [f32; 4] {
        let [fx0, fx1, fy0, fy1] = self.full_bounds;
        if self.zoom <= 1.01 {
            return self.full_bounds;
        }
        let half = 0.5 * (fx1 - fx0).max(1.0) / self.zoom;
        let cx = self.center.x.clamp(fx0 + half, fx1 - half);
        let cy = self.center.y.clamp(fy0 + half, fy1 - half);
        [cx - half, cx + half, cy - half, cy + half]
    }

    fn world_to_px_bounds(bounds: [f32; 4], px_size: usize, p: P2) -> (i32, i32) {
        let [x0, x1, y0, y1] = bounds;
        let u = (p.x - x0) / (x1 - x0).max(1.0);
        let v = (y1 - p.y) / (y1 - y0).max(1.0);
        (
            (u * (px_size - 1) as f32).round() as i32,
            (v * (px_size - 1) as f32).round() as i32,
        )
    }

    fn px_to_world_bounds(bounds: [f32; 4], px_size: usize, px: usize, py: usize) -> P2 {
        let [x0, x1, y0, y1] = bounds;
        let u = px as f32 / (px_size - 1) as f32;
        let v = py as f32 / (px_size - 1) as f32;
        P2::new(x0 + u * (x1 - x0), y1 - v * (y1 - y0))
    }

    fn world_to_px(&self, p: P2) -> (i32, i32) {
        Self::world_to_px_bounds(self.view_bounds(), MAP_PX, p)
    }

    pub fn redraw_now(
        &self,
        sim: &WorldSim,
        city: &City,
        forces: &Forces,
        images: &mut Assets<Image>,
    ) {
        let Some(mut img) = images.get_mut(&self.handle) else {
            return;
        };
        let bounds = self.view_bounds();
        let [fx0, fx1, fy0, fy1] = self.full_bounds;
        let fw = (fx1 - fx0).max(1.0);
        let fh = (fy1 - fy0).max(1.0);

        // Resample high-res 720x720 base terrain/road/building image cleanly into 360x360 viewport.
        let mut buf = vec![0u8; MAP_PX * MAP_PX * 4];
        for py in 0..MAP_PX {
            for px in 0..MAP_PX {
                let w = Self::px_to_world_bounds(bounds, MAP_PX, px, py);
                let su = ((w.x - fx0) / fw).clamp(0.0, 1.0);
                let sv = ((fy1 - w.y) / fh).clamp(0.0, 1.0);
                let bx = ((su * (BASE_PX - 1) as f32).round() as usize).min(BASE_PX - 1);
                let by = ((sv * (BASE_PX - 1) as f32).round() as usize).min(BASE_PX - 1);
                let src_o = (by * BASE_PX + bx) * 4;
                let dst_o = (py * MAP_PX + px) * 4;
                buf[dst_o..dst_o + 4].copy_from_slice(&self.base_rgba[src_o..src_o + 4]);
            }
        }

        // 1. Highlight explored 20m cells.
        for &(cx, cy) in &sim.visited {
            let p_nw = P2::new(cx as f32 * 20.0, (cy + 1) as f32 * 20.0);
            let p_se = P2::new((cx + 1) as f32 * 20.0, cy as f32 * 20.0);
            let (px0, py0) = self.world_to_px(p_nw);
            let (px1, py1) = self.world_to_px(p_se);
            for py in py0.max(2)..=py1.min((MAP_PX - 3) as i32) {
                for px in px0.max(2)..=px1.min((MAP_PX - 3) as i32) {
                    let o = (py as usize * MAP_PX + px as usize) * 4;
                    buf[o] = buf[o].saturating_add(16);
                    buf[o + 1] = buf[o + 1].saturating_add(20);
                    buf[o + 2] = buf[o + 2].saturating_add(22);
                }
            }
        }

        // 2. Russian front depth line across map columns.
        let depth = front_depth(sim.hour, forces);
        if depth > 0.0 {
            let [x0, x1, y0, y1] = bounds;
            for px in 2..(MAP_PX - 2) {
                let u = px as f32 / (MAP_PX - 1) as f32;
                let wx = x0 + u * (x1 - x0);
                let front_y = city.bank_y(wx) - depth;
                let py = (((y1 - front_y) / (y1 - y0).max(1.0)) * (MAP_PX - 1) as f32).round() as i32;
                let bank_py = (((y1 - city.bank_y(wx)) / (y1 - y0).max(1.0)) * (MAP_PX - 1) as f32).round() as i32;

                let span_top = bank_py.max(2);
                let span_bot = py.min((MAP_PX - 3) as i32);
                for sy in span_top..span_bot {
                    let o = (sy as usize * MAP_PX + px) * 4;
                    buf[o] = buf[o].saturating_add(26);
                }

                for dy in -1..=1 {
                    let sy = py + dy;
                    if sy >= 2 && sy < (MAP_PX - 2) as i32 && (px % 7 != 0 || dy == 0) {
                        plot(&mut buf, px as i32, sy, [225, 62, 58, 255]);
                    }
                }
            }
        }

        // 3. Shop doors.
        for d in &sim.doors {
            let (px, py) = self.world_to_px(d.at);
            let col = if d.searched {
                [108, 114, 122, 255]
            } else {
                [238, 184, 62, 255]
            };
            dot(&mut buf, px, py, 1, col);
        }

        // 3b. Enterable buildings, PlayerHome safehouse, Basement & Neighbour shelter.
        for intr in &sim.interiors {
            let (px, py) = self.world_to_px(intr.front_door.at);
            match intr.kind {
                InteriorKind::PlayerHome => {
                    ring(&mut buf, px, py, 3, [255, 224, 78, 255]);
                    dot(&mut buf, px, py, 2, [255, 212, 64, 255]);
                }
                InteriorKind::BasementShelter => {
                    ring(&mut buf, px, py, 3, [95, 238, 255, 255]);
                    dot(&mut buf, px, py, 2, [95, 238, 255, 255]);
                }
                InteriorKind::NeighbourHome => {
                    ring(&mut buf, px, py, 3, [125, 240, 145, 255]);
                    dot(&mut buf, px, py, 2, [125, 240, 145, 255]);
                }
                InteriorKind::Pharmacy => {
                    dot(&mut buf, px, py, 2, [82, 232, 224, 255]);
                }
                InteriorKind::Residential => {
                    if intr.discovered || self.zoom >= 1.8 {
                        dot(&mut buf, px, py, 1, [96, 178, 198, 255]);
                    }
                }
                _ => {
                    if intr.discovered || intr.front_door.at.dist(sim.player_at) <= 115.0 || self.zoom >= 1.8 {
                        dot(&mut buf, px, py, 1, [72, 198, 212, 255]);
                    }
                }
            }
        }

        // 4. Unlocked uncollected caches.
        for c in &sim.caches {
            if !c.collected && sim.hour >= c.unlock_hour {
                let (px, py) = self.world_to_px(c.at);
                dot(&mut buf, px, py, 2, [76, 224, 132, 255]);
            }
        }

        // 5. Barricades, craters, wrecks & fire barrels.
        for c in &sim.craters {
            let (px, py) = self.world_to_px(c.at);
            ring(&mut buf, px, py, 2, [96, 88, 82, 255]);
        }
        for w in &sim.wrecks {
            let (px, py) = self.world_to_px(w.at);
            dot(&mut buf, px, py, 2, [78, 72, 70, 255]);
            if w.smoke_ttl > 0.0 {
                plot(&mut buf, px, py, [255, 120, 40, 255]);
            }
        }
        for b in &sim.barricades {
            let (px, py) = self.world_to_px(b.at);
            let col = match b.kind {
                sim::survival::BarricadeKind::Trench => [172, 134, 86, 255],
                sim::survival::BarricadeKind::TarpDugout => [84, 178, 146, 255],
                sim::survival::BarricadeKind::BarbedWire => [164, 174, 184, 255],
                _ => [118, 202, 238, 255],
            };
            dot(&mut buf, px, py, 1, col);
        }
        for f in &sim.fire_barrels {
            let (px, py) = self.world_to_px(*f);
            dot(&mut buf, px, py, 2, [255, 148, 46, 255]);
        }

        // 6. Militia (blue).
        for m in &sim.militia {
            if m.hp > 0.0 {
                let (px, py) = self.world_to_px(m.at);
                dot(&mut buf, px, py, 2, [78, 178, 255, 255]);
            }
        }

        // 7. Nearby Russian soldiers (red).
        for sq in &sim.squads {
            for man in &sq.men {
                if man.hp <= 0.0 {
                    continue;
                }
                let near_player = man.at.dist(sim.player_at) <= 185.0;
                let near_militia = sim.militia.iter().any(|m| m.hp > 0.0 && m.at.dist(man.at) <= 85.0);
                if near_player || near_militia {
                    let (px, py) = self.world_to_px(man.at);
                    dot(&mut buf, px, py, 2, [242, 62, 56, 255]);
                }
            }
        }

        // 7b. Active PLA & Russian armored vehicles!
        for t in &sim.tanks {
            let (px, py) = self.world_to_px(t.at);
            match t.faction {
                Faction::Pla => {
                    // Emerald-teal square with bright gold center for PLA armor / PGZ-09 AA.
                    dot(&mut buf, px, py, 3, [44, 228, 148, 255]);
                    dot(&mut buf, px, py, 1, [255, 228, 84, 255]);
                }
                Faction::Russia => {
                    // Crimson-orange diamond with bright center for Russian MBTs / IFVs / TOS-1A.
                    for dy in -4i32..=4 {
                        for dx in -4i32..=4 {
                            if dx.abs() + dy.abs() <= 4 {
                                plot(&mut buf, px + dx, py + dy, [255, 68, 32, 255]);
                            }
                        }
                    }
                    plot(&mut buf, px, py, [255, 224, 96, 255]);
                }
            }
        }

        // 8. Active drones (recon, enemy FPV, Lancet-3, friendly FPV).
        if sim.drone.active {
            let (px, py) = self.world_to_px(sim.drone.at);
            ring(&mut buf, px, py, 5, [245, 84, 218, 255]);
            plot(&mut buf, px, py, [245, 84, 218, 255]);
        }
        if let Some(fpv) = &sim.fpv {
            let (px, py) = self.world_to_px(fpv.at);
            ring(&mut buf, px, py, 4, [255, 215, 70, 255]);
            dot(&mut buf, px, py, 1, [255, 65, 65, 255]);
        }
        if let Some(lan) = &sim.lancet {
            let (px, py) = self.world_to_px(lan.at);
            ring(&mut buf, px, py, 5, [255, 92, 44, 255]);
            dot(&mut buf, px, py, 1, [255, 220, 120, 255]);
        }
        if let Some(ffpv) = &sim.friendly_fpv {
            let (px, py) = self.world_to_px(ffpv.at);
            ring(&mut buf, px, py, 4, [60, 235, 255, 255]);
            dot(&mut buf, px, py, 1, [240, 255, 255, 255]);
        }

        // 9. Player crosshair + facing notch.
        let (px, py) = self.world_to_px(sim.player_at);
        for d in -6..=6 {
            plot(&mut buf, px + d, py, [255, 246, 175, 255]);
            plot(&mut buf, px, py + d, [255, 246, 175, 255]);
        }
        let f_dir = P2::new(sim.player_facing.cos(), sim.player_facing.sin());
        for s in 3..=8 {
            let fx = px + (f_dir.x * s as f32).round() as i32;
            let fy = py - (f_dir.y * s as f32).round() as i32;
            plot(&mut buf, fx, fy, [255, 212, 82, 255]);
        }

        // Crisp 2px outer border frame on viewport image.
        for i in 0..MAP_PX {
            for edge in [0, 1, MAP_PX - 2, MAP_PX - 1] {
                let h = (edge * MAP_PX + i) * 4;
                buf[h..h + 4].copy_from_slice(&[115, 148, 182, 255]);
                let v = (i * MAP_PX + edge) * 4;
                buf[v..v + 4].copy_from_slice(&[115, 148, 182, 255]);
            }
        }

        img.data = Some(buf);
    }
}

fn plot(buf: &mut [u8], x: i32, y: i32, col: [u8; 4]) {
    if x < 2 || y < 2 || x >= (MAP_PX - 2) as i32 || y >= (MAP_PX - 2) as i32 {
        return;
    }
    let o = (y as usize * MAP_PX + x as usize) * 4;
    buf[o..o + 4].copy_from_slice(&col);
}

fn dot(buf: &mut [u8], cx: i32, cy: i32, r: i32, col: [u8; 4]) {
    for dy in -r..=r {
        for dx in -r..=r {
            plot(buf, cx + dx, cy + dy, col);
        }
    }
}

fn ring(buf: &mut [u8], cx: i32, cy: i32, r: i32, col: [u8; 4]) {
    for dy in -r..=r {
        for dx in -r..=r {
            let d2 = dx * dx + dy * dy;
            if d2 >= (r - 1) * (r - 1) && d2 <= r * r + 1 {
                plot(buf, cx + dx, cy + dy, col);
            }
        }
    }
}

pub fn install(app: &mut App, enable: bool, interactive: bool) {
    if enable {
        app.init_resource::<MapActionQueue>()
            .add_systems(Startup, setup_map.after(crate::play::setup_play));
        if interactive {
            app.add_systems(Update, update_map.after(crate::play::step_play));
        } else {
            app.add_systems(Update, update_map);
        }
    }
}

fn compute_bounds(city: &City) -> [f32; 4] {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for r in &city.roads {
        for p in &r.line {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
    }
    for b in city.buildings.iter().filter(|b| b.bank == Bank::China) {
        for p in &b.footprint {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
    }

    if !min_x.is_finite() {
        return [-400.0, 400.0, -400.0, 400.0];
    }
    max_y += 95.0;
    min_y -= 25.0;
    min_x -= 25.0;
    max_x += 25.0;

    let cx = 0.5 * (min_x + max_x);
    let cy = 0.5 * (min_y + max_y);
    let half = 0.5 * (max_x - min_x).max(max_y - min_y);
    [cx - half, cx + half, cy - half, cy + half]
}

fn bake_base_map(city: &City, bounds: [f32; 4]) -> Vec<u8> {
    let mut buf = vec![0u8; BASE_PX * BASE_PX * 4];

    for py in 0..BASE_PX {
        for px in 0..BASE_PX {
            let p = TacticalMap::px_to_world_bounds(bounds, BASE_PX, px, py);
            let bank = city.bank_y(p.x);
            let ice = city.ice_y(p.x);
            let col = if p.y > ice {
                [30, 46, 68, 255]
            } else if p.y > bank {
                [50, 78, 108, 255]
            } else {
                [26, 32, 40, 255]
            };
            let o = (py * BASE_PX + px) * 4;
            buf[o..o + 4].copy_from_slice(&col);
        }
    }

    let m_per_px = (bounds[1] - bounds[0]) / BASE_PX as f32;
    for r in &city.roads {
        if r.line.len() < 2 {
            continue;
        }
        let mut rmin_x = f32::INFINITY;
        let mut rmax_x = f32::NEG_INFINITY;
        let mut rmin_y = f32::INFINITY;
        let mut rmax_y = f32::NEG_INFINITY;
        for p in &r.line {
            rmin_x = rmin_x.min(p.x);
            rmax_x = rmax_x.max(p.x);
            rmin_y = rmin_y.min(p.y);
            rmax_y = rmax_y.max(p.y);
        }
        let rad = (r.width * 0.5).max(m_per_px * 0.9);
        let (px0, py1) = TacticalMap::world_to_px_bounds(bounds, BASE_PX, P2::new(rmin_x - rad, rmin_y - rad));
        let (px1, py0) = TacticalMap::world_to_px_bounds(bounds, BASE_PX, P2::new(rmax_x + rad, rmax_y + rad));
        let col = if r.pedestrian() {
            [86, 94, 104, 255]
        } else {
            [68, 76, 88, 255]
        };
        for py in py0.max(2)..=py1.min((BASE_PX - 3) as i32) {
            for px in px0.max(2)..=px1.min((BASE_PX - 3) as i32) {
                let p = TacticalMap::px_to_world_bounds(bounds, BASE_PX, px as usize, py as usize);
                if dist_to_polyline(p, &r.line) <= rad {
                    let o = (py as usize * BASE_PX + px as usize) * 4;
                    buf[o..o + 4].copy_from_slice(&col);
                }
            }
        }
    }

    for b in city.buildings.iter().filter(|b| b.bank == Bank::China && b.footprint.len() >= 3) {
        let mut bmin_x = f32::INFINITY;
        let mut bmax_x = f32::NEG_INFINITY;
        let mut bmin_y = f32::INFINITY;
        let mut bmax_y = f32::NEG_INFINITY;
        for p in &b.footprint {
            bmin_x = bmin_x.min(p.x);
            bmax_x = bmax_x.max(p.x);
            bmin_y = bmin_y.min(p.y);
            bmax_y = bmax_y.max(p.y);
        }
        let (px0, py1) = TacticalMap::world_to_px_bounds(bounds, BASE_PX, P2::new(bmin_x, bmin_y));
        let (px1, py0) = TacticalMap::world_to_px_bounds(bounds, BASE_PX, P2::new(bmax_x, bmax_y));
        for py in py0.max(2)..=py1.min((BASE_PX - 3) as i32) {
            for px in px0.max(2)..=px1.min((BASE_PX - 3) as i32) {
                let p = TacticalMap::px_to_world_bounds(bounds, BASE_PX, px as usize, py as usize);
                if point_in_ring(p, &b.footprint) {
                    let o = (py as usize * BASE_PX + px as usize) * 4;
                    buf[o..o + 4].copy_from_slice(&[50, 58, 72, 255]);
                }
            }
        }
    }

    buf
}

fn spawn_map_btn(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, label: &str, action: MapAction) {
    let normal_bg = Color::srgba(0.08, 0.12, 0.18, 0.90);
    let hover_bg = Color::srgba(0.16, 0.24, 0.36, 0.95);
    let normal_bd = Color::srgba(0.45, 0.64, 0.84, 0.42);
    let hover_bd = Color::srgba(0.88, 0.94, 1.0, 0.88);
    let mut ec = parent.spawn((
        Node {
            height: Val::Px(28.0),
            padding: UiRect::horizontal(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(normal_bg),
        BorderColor::all(normal_bd),
    ));
    ec.observe(move |ev: On<PointerOver>, mut q: Query<(&mut BackgroundColor, &mut BorderColor)>| {
        if let Ok((mut bg, mut bd)) = q.get_mut(ev.entity) {
            bg.0 = hover_bg;
            *bd = BorderColor::all(hover_bd);
        }
    })
    .observe(move |ev: On<PointerOut>, mut q: Query<(&mut BackgroundColor, &mut BorderColor)>| {
        if let Ok((mut bg, mut bd)) = q.get_mut(ev.entity) {
            bg.0 = normal_bg;
            *bd = BorderColor::all(normal_bd);
        }
    })
    .observe(move |ev: On<PointerClick>, mut queue: ResMut<MapActionQueue>| {
        if ev.button == PointerButton::Primary {
            queue.0.push(action);
        }
    });
    ec.with_children(|b| {
        b.spawn((
            Text::new(label),
            TextFont {
                font: font.clone().into(),
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ));
    });
}

pub fn setup_map(
    mut commands: Commands,
    heihe: Res<Heihe>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let bounds = compute_bounds(&heihe.city);
    let base_rgba = bake_base_map(&heihe.city, bounds);
    let image = Image::new(
        Extent3d {
            width: MAP_PX as u32,
            height: MAP_PX as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0u8; MAP_PX * MAP_PX * 4],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let handle = images.add(image);
    let font: Handle<Font> = assets.load("fonts/NotoSansSC-VF.ttf");

    // Full-screen flexbox centering container ensures TacticalMap sits dead center at any window size.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.48)),
            Visibility::Hidden,
            MapOverlay,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.95)),
                    BorderColor::all(Color::srgba(0.45, 0.62, 0.80, 0.45)),
                    Node {
                        padding: UiRect::all(Val::Px(14.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("黑河城区战术地图   [M关闭 · 滚轮/+-缩放 · WASD平移 · C居中 · 右键派民兵]"),
                        TextFont {
                            font: font.clone().into(),
                            font_size: bevy::text::FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.96, 1.0)),
                    ));
                    p.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|bar| {
                        spawn_map_btn(bar, &font, "放大 +", MapAction::ZoomIn);
                        spawn_map_btn(bar, &font, "缩小 -", MapAction::ZoomOut);
                        spawn_map_btn(bar, &font, "居中自己 (C)", MapAction::CenterPlayer);
                        spawn_map_btn(bar, &font, "全城 (0)", MapAction::ResetFull);
                        bar.spawn((
                            Text::new("缩放 1.0x"),
                            TextFont {
                                font: font.clone().into(),
                                font_size: bevy::text::FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.88, 0.48)),
                            MapZoomLabel,
                        ));
                    });
                    p.spawn((
                        ImageNode::new(handle.clone()),
                        Node {
                            width: Val::Px(520.0),
                            height: Val::Px(520.0),
                            ..default()
                        },
                    ))
                    .observe(|ev: On<PointerClick>, mut map: ResMut<TacticalMap>| {
                        if ev.button == PointerButton::Secondary {
                            if let Some(pos) = ev.hit.position {
                                let u = (pos.x + 0.5).clamp(0.0, 1.0);
                                let v = (pos.y + 0.5).clamp(0.0, 1.0);
                                let [x0, x1, y0, y1] = map.view_bounds();
                                map.pending_militia_order = Some(P2::new(x0 + u * (x1 - x0), y1 - v * (y1 - y0)));
                            }
                        }
                    });
                    p.spawn((
                        Text::new(
                            "十字:自己  亮绿方:解放军坦克  红菱:俄军战车  蓝:民兵  红点:俄步兵  紫:无人机  青:全城可进屋",
                        ),
                        TextFont {
                            font: font.into(),
                            font_size: bevy::text::FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.78, 0.86, 0.94)),
                    ));
                });
        });

    let center = heihe.city.spawn;
    let map_res = TacticalMap {
        open: false,
        handle,
        base_rgba,
        full_bounds: bounds,
        zoom: 1.0,
        center,
        follow_player: true,
        pending_militia_order: None,
        was_open: false,
        refresh_timer: 0.0,
    };
    commands.insert_resource(map_res);
}

#[allow(clippy::too_many_arguments)]
fn update_map(
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    time: Res<Time>,
    heihe: Res<Heihe>,
    play: Option<Res<PlayState>>,
    menu: Option<Res<crate::menu_ui::MenuScreen>>,
    map: Option<ResMut<TacticalMap>>,
    mut actions: ResMut<MapActionQueue>,
    mut images: ResMut<Assets<Image>>,
    mut panel: Query<&mut Visibility, With<MapOverlay>>,
    mut zoom_lbl: Query<&mut Text, With<MapZoomLabel>>,
) {
    let Some(mut map) = map else {
        wheel.clear();
        return;
    };
    let Some(play) = play else {
        wheel.clear();
        return;
    };

    let playing = menu.is_none_or(|m| *m == crate::menu_ui::MenuScreen::Playing);
    if playing && play.started && keys.just_pressed(KeyCode::KeyM) {
        map.open = !map.open;
        if map.open {
            map.center = play.sim.player_at;
            map.follow_player = true;
        }
    }
    if !playing && play.started && map.open {
        map.open = false;
    }

    for mut vis in &mut panel {
        *vis = if map.open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    if !map.open {
        map.was_open = false;
        actions.0.clear();
        wheel.clear();
        return;
    }

    let mut force_redraw = false;
    for a in actions.0.drain(..) {
        match a {
            MapAction::ZoomIn => {
                let c = map.center;
                let z = map.zoom * 1.35;
                map.set_view(c, z);
            }
            MapAction::ZoomOut => {
                let c = map.center;
                let z = map.zoom / 1.35;
                map.set_view(c, z);
            }
            MapAction::CenterPlayer => {
                map.follow_player = true;
                let z = map.zoom.max(1.8);
                map.set_view(play.sim.player_at, z);
            }
            MapAction::ResetFull => {
                map.follow_player = false;
                let c = P2::new(
                    0.5 * (map.full_bounds[0] + map.full_bounds[1]),
                    0.5 * (map.full_bounds[2] + map.full_bounds[3]),
                );
                map.set_view(c, 1.0);
            }
        }
        force_redraw = true;
    }

    let mut turn = 0.0f32;
    for w in wheel.read() {
        turn += w.y;
    }
    if turn != 0.0 {
        let z = map.zoom * 1.18f32.powf(turn);
        let c = map.center;
        map.set_view(c, z);
        force_redraw = true;
    }

    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        let c = map.center;
        let z = map.zoom * 1.35;
        map.set_view(c, z);
        force_redraw = true;
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        let c = map.center;
        let z = map.zoom / 1.35;
        map.set_view(c, z);
        force_redraw = true;
    }
    if keys.just_pressed(KeyCode::KeyC) || keys.just_pressed(KeyCode::Home) {
        map.follow_player = true;
        let z = map.zoom.max(1.8);
        map.set_view(play.sim.player_at, z);
        force_redraw = true;
    }
    if keys.just_pressed(KeyCode::Digit0) {
        map.follow_player = false;
        let c = P2::new(
            0.5 * (map.full_bounds[0] + map.full_bounds[1]),
            0.5 * (map.full_bounds[2] + map.full_bounds[3]),
        );
        map.set_view(c, 1.0);
        force_redraw = true;
    }

    let mut pan = P2::default();
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        pan.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        pan.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        pan.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        pan.x -= 1.0;
    }
    if pan.len() > 0.1 {
        map.follow_player = false;
        let pan_speed = 260.0 / map.zoom.max(1.0);
        let c = map.center.add(pan.norm().mul(pan_speed * time.delta_secs()));
        let z = map.zoom;
        map.set_view(c, z);
        force_redraw = true;
    } else if map.follow_player && map.zoom > 1.01 {
        let z = map.zoom;
        map.set_view(play.sim.player_at, z);
    }

    for mut txt in &mut zoom_lbl {
        txt.0 = format!("缩放 {:.1}x", map.zoom);
    }

    map.refresh_timer -= time.delta_secs();
    if force_redraw || !map.was_open || map.refresh_timer <= 0.0 {
        map.was_open = true;
        map.refresh_timer = 0.12;
        map.redraw_now(&play.sim, &heihe.city, &play.forces, &mut images);
    }
}
