//! Seamless cutaway ground-floor interiors, safehouses, and basement shelter.
//!
//! Fits clean local rectangular room layouts strictly inside real Heihe building
//! footprints along Central Street, Mother Square, and nearby residential blocks,
//! supporting walk collision, window line-of-sight, thermal inertia, door
//! barricading/breaching, curtains, coal stoves, and procedural 3D furniture.

use crate::city::{Bank, City};
use crate::geom::{nearest_on_segment, point_in_ring, spans_inside, P2};
use crate::look::{Palette, Rgb};
use crate::soup::Soup;
use crate::survival::ShopKind;
use crate::walk::Ground;

/// Vector from world center toward the orthographic SE camera (+east, -north).
const CAM_DIR: P2 = P2::new(0.70710678, -0.70710678);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteriorKind {
    PlayerHome,
    NeighbourHome,
    BasementShelter,
    Pharmacy,
    Food,
    Drink,
    Clothes,
    Hardware,
    Hotel,
    Shop,
    Residential,
}

impl InteriorKind {
    pub fn from_shop(k: ShopKind) -> Self {
        match k {
            ShopKind::Pharmacy => Self::Pharmacy,
            ShopKind::Food => Self::Food,
            ShopKind::Drink => Self::Drink,
            ShopKind::Clothes => Self::Clothes,
            ShopKind::Hardware => Self::Hardware,
            ShopKind::Hotel => Self::Hotel,
            ShopKind::Office | ShopKind::Shop => Self::Shop,
        }
    }

    pub fn to_shop_kind(self) -> ShopKind {
        match self {
            Self::Pharmacy => ShopKind::Pharmacy,
            Self::Food => ShopKind::Food,
            Self::Drink => ShopKind::Drink,
            Self::Clothes => ShopKind::Clothes,
            Self::Hardware => ShopKind::Hardware,
            Self::Hotel => ShopKind::Hotel,
            Self::PlayerHome | Self::NeighbourHome | Self::BasementShelter | Self::Residential => {
                ShopKind::Shop
            }
            Self::Shop => ShopKind::Shop,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::PlayerHome => "自己家·安全屋",
            Self::NeighbourHome => "邻居家·避难室",
            Self::BasementShelter => "人防地下室",
            Self::Pharmacy => "药店室内",
            Self::Food => "餐馆大堂",
            Self::Drink => "饮品店室内",
            Self::Clothes => "服装店室内",
            Self::Hardware => "五金杂货间",
            Self::Hotel => "旅馆前厅",
            Self::Shop => "临街商铺室内",
            Self::Residential => "居民单元楼",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomZone {
    FrontHall,
    BackRoom,
    Basement,
}

/// Room-local axis-aligned box in `(u, v)` space.
#[derive(Clone, Copy, Debug)]
pub struct LocalBox {
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hd: f32,
    pub h: f32,
}

impl LocalBox {
    pub fn contains(&self, u: f32, v: f32, pad: f32) -> bool {
        (u - self.cx).abs() <= self.hw + pad && (v - self.cy).abs() <= self.hd + pad
    }

    pub fn push_out(&self, u: f32, v: f32, radius: f32) -> Option<(f32, f32)> {
        let dx = u - self.cx;
        let dy = v - self.cy;
        let ox = (self.hw + radius) - dx.abs();
        let oy = (self.hd + radius) - dy.abs();
        if ox > 0.0 && oy > 0.0 {
            if ox < oy {
                Some((self.cx + if dx >= 0.0 { self.hw + radius } else { -(self.hw + radius) }, v))
            } else {
                Some((u, self.cy + if dy >= 0.0 { self.hd + radius } else { -(self.hd + radius) }))
            }
        } else {
            None
        }
    }

    /// 2D slab intersection test against local segment `(u0,v0) -> (u1,v1)`.
    pub fn segment_hits(&self, u0: f32, v0: f32, u1: f32, v1: f32) -> bool {
        let min_x = self.cx - self.hw;
        let max_x = self.cx + self.hw;
        let min_y = self.cy - self.hd;
        let max_y = self.cy + self.hd;

        let du = u1 - u0;
        let dv = v1 - v0;
        let mut tmin = 0.0f32;
        let mut tmax = 1.0f32;

        if du.abs() < 1.0e-5 {
            if u0 < min_x || u0 > max_x {
                return false;
            }
        } else {
            let inv = 1.0 / du;
            let mut t1 = (min_x - u0) * inv;
            let mut t2 = (max_x - u0) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }

        if dv.abs() < 1.0e-5 {
            if v0 < min_y || v0 > max_y {
                return false;
            }
        } else {
            let inv = 1.0 / dv;
            let mut t1 = (min_y - v0) * inv;
            let mut t2 = (max_y - v0) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }

        true
    }
}

/// Vertical height per story in metres (`1F..5F`).
pub const FLOOR_H: f32 = 3.1;

/// Physical door archetype determining visuals and opening/curtain behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorStyle {
    CottonCurtain,
    ShopGlass,
    SteelSecurity,
    WoodFlat,
}

#[derive(Clone, Debug)]
pub struct RoomDoor {
    pub at: P2,
    pub outside: P2,
    pub inside: P2,
    pub u_center: f32,
    pub v_center: f32,
    pub dir: P2,
    pub out: P2,
    pub width: f32,
    pub is_back: bool,
    pub is_security: bool,
    pub style: DoorStyle,
    pub open: bool,
    pub hp: f32,
    pub destroyed: bool,
    pub barricade_hp: f32,
    pub max_barricade_hp: f32,
    pub has_curtain: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SlabCell {
    pub hp: f32,
    pub intact: bool,
}

#[derive(Clone, Debug)]
pub struct FloorState {
    pub slabs: [[SlabCell; 3]; 2],
    pub front_wall_hp: f32,
    pub front_wall_breached: bool,
    pub loots: Vec<RoomLoot>,
    pub rubble: Vec<P2>,
}

impl FloorState {
    pub fn intact_default(slab_hp: f32, wall_hp: f32, loots: Vec<RoomLoot>) -> Self {
        Self {
            slabs: [[SlabCell { hp: slab_hp, intact: true }; 3]; 2],
            front_wall_hp: wall_hp,
            front_wall_breached: false,
            loots,
            rubble: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoomWindow {
    pub at: P2,
    pub u_a: f32,
    pub u_b: f32,
    pub a: P2,
    pub b: P2,
    pub out: P2,
    pub broken: bool,
    pub boarded: bool,
}

#[derive(Clone, Debug)]
pub struct RoomLoot {
    pub id: u32,
    pub at: P2,
    pub label: &'static str,
    pub kind: ShopKind,
    pub in_backroom: bool,
    pub searched: bool,
}

#[derive(Clone, Debug)]
pub struct RoomNpc {
    pub id: u32,
    pub at: P2,
    pub facing: f32,
    pub label: &'static str,
    pub line_calm: &'static str,
    pub line_war: &'static str,
    pub coat: Rgb,
}

#[derive(Clone, Debug, Default)]
pub struct HomeStash {
    pub food: u32,
    pub water: u32,
    pub bandage: u32,
    pub warmer: u32,
    pub ammo: u32,
    pub scrap: u32,
    pub coal: u32,
}

#[derive(Clone, Debug)]
pub struct Interior {
    pub id: usize,
    pub building_id: i64,
    pub name: String,
    pub kind: InteriorKind,
    pub origin: P2,
    pub u: P2,
    pub v: P2,
    pub half_w: f32,
    pub depth: f32,
    pub split_v: f32,
    pub wall_h: f32,
    pub num_floors: u8,
    pub stairwell_u: f32,
    pub stairwell_v: f32,
    pub front_door: RoomDoor,
    pub back_door: Option<RoomDoor>,
    pub inner_door: Option<RoomDoor>,
    pub windows: Vec<RoomWindow>,
    pub walk_walls: Vec<LocalBox>,
    pub sight_walls: Vec<LocalBox>,
    pub colliders: Vec<LocalBox>,
    pub firing_spots: Vec<P2>,
    pub stove_at: Option<P2>,
    pub stove_fuel_s: f32,
    pub locker_at: Option<P2>,
    pub stash: HomeStash,
    pub loots: Vec<RoomLoot>,
    pub floors: Vec<FloorState>,
    pub npcs: Vec<RoomNpc>,
    pub room_temp_c: f32,
    pub curtains_closed: bool,
    pub discovered: bool,
    pub breach_alert: f32,
    pub revision: u32,
}

/// Visual meshes built once per enterable building and toggled in Bevy when entered/approached.
#[derive(Default)]
pub struct InteriorMesh {
    pub static_solid: Soup,
    pub window_glass: Soup,
    pub window_boards: Soup,
    pub curtains: Soup,
    pub barricade_front: Soup,
    pub barricade_back: Soup,
    pub exterior_portal: Soup,
}

impl Interior {
    pub fn to_local(&self, p: P2) -> (f32, f32) {
        let d = p.sub(self.origin);
        (d.dot(self.u), d.dot(self.v))
    }

    pub fn to_world(&self, lu: f32, lv: f32) -> P2 {
        self.origin.add(self.u.mul(lu)).add(self.v.mul(lv))
    }

    pub fn contains(&self, p: P2) -> bool {
        let (lu, lv) = self.to_local(p);
        lu.abs() <= self.half_w - 0.02 && lv >= 0.02 && lv <= self.depth - 0.02
    }

    pub fn contains_padded(&self, p: P2, pad: f32) -> bool {
        let (lu, lv) = self.to_local(p);
        lu.abs() <= self.half_w + pad && lv >= -pad && lv <= self.depth + pad
    }

    pub fn zone(&self, p: P2) -> Option<RoomZone> {
        if !self.contains(p) {
            return None;
        }
        if self.kind == InteriorKind::BasementShelter {
            return Some(RoomZone::Basement);
        }
        let (_, lv) = self.to_local(p);
        if lv >= self.split_v {
            Some(RoomZone::BackRoom)
        } else {
            Some(RoomZone::FrontHall)
        }
    }

    /// True if `p` sits inside the front/back doorway opening corridor so exterior ring collision is bypassed.
    pub fn door_passage(&self, p: P2, radius: f32, ignore_barricade: bool) -> bool {
        let (lu, lv) = self.to_local(p);
        let fd = &self.front_door;
        if (ignore_barricade || fd.barricade_hp <= 0.0)
            && (lu - fd.u_center).abs() <= fd.width * 0.5 + radius + 0.14
            && (-1.25..=1.25).contains(&lv)
        {
            return true;
        }
        if let Some(bd) = &self.back_door {
            if (ignore_barricade || bd.barricade_hp <= 0.0)
                && (lu - bd.u_center).abs() <= bd.width * 0.5 + radius + 0.14
                && (self.depth - 1.25..=self.depth + 1.65).contains(&lv)
            {
                return true;
            }
        }
        false
    }

    /// Stairwell pad world coordinates inside this interior.
    pub fn stairwell_world(&self) -> P2 {
        self.to_world(self.stairwell_u, self.stairwell_v)
    }

    pub fn slab_cell_at(&self, p: P2) -> Option<(usize, usize)> {
        let (lu, lv) = self.to_local(p);
        if lu.abs() > self.half_w + 0.15 || lv < -0.15 || lv > self.depth + 0.15 {
            return None;
        }
        let col = (((lu + self.half_w) / (self.half_w * 2.0).max(0.1)) * 3.0)
            .floor()
            .clamp(0.0, 2.0) as usize;
        let row = ((lv / self.depth.max(0.1)) * 2.0)
            .floor()
            .clamp(0.0, 1.0) as usize;
        Some((row, col))
    }

    pub fn slab_center_world(&self, row: usize, col: usize) -> P2 {
        let cw = (self.half_w * 2.0) / 3.0;
        let cd = self.depth * 0.5;
        let lu = -self.half_w + (col as f32 + 0.5) * cw;
        let lv = (row as f32 + 0.5) * cd;
        self.to_world(lu, lv)
    }

    pub fn is_slab_intact(&self, floor: u8, p: P2) -> bool {
        if floor == 0 {
            return true;
        }
        if self.num_floors > 1 && self.stairwell_world().dist(p) <= 0.95 {
            return true;
        }
        let Some(fs) = self.floors.get(floor as usize) else {
            return true;
        };
        let Some((r, c)) = self.slab_cell_at(p) else {
            return true;
        };
        fs.slabs[r][c].intact
    }

    pub fn any_structural_damage(&self) -> bool {
        self.floors
            .iter()
            .any(|f| f.front_wall_breached || f.slabs.iter().flatten().any(|c| !c.intact))
    }

    /// Apply explosive/tank structural blast onto floor slabs, front wall & doors around `at` at `target_floor`.
    pub fn blast_at(&mut self, at: P2, target_floor: u8, radius: f32, structural_dmg: f32) -> (u32, Vec<P2>) {
        let mut destroyed_slabs = 0u32;
        let mut damaged = false;
        let mut add_rubble: Vec<(usize, P2)> = Vec::new();
        let w_left = self.to_world(-self.half_w, 0.0);
        let w_right = self.to_world(self.half_w, 0.0);
        let d_wall = at.dist(nearest_on_segment(at, w_left, w_right));

        let mut centers = [[P2::default(); 3]; 2];
        for r in 0..2 {
            for c in 0..3 {
                centers[r][c] = self.slab_center_world(r, c);
            }
        }

        let f_count = self.floors.len();
        for f in 0..f_count {
            let df = (f as i32 - target_floor as i32).abs();
            if df > 1 {
                continue;
            }
            let mul = if df == 0 { 1.0 } else { 0.42 };
            let fs = &mut self.floors[f];
            for r in 0..2 {
                for c in 0..3 {
                    let d = centers[r][c].dist(at);
                    if d <= radius {
                        let falloff = (1.0 - d / (radius * 1.15)).clamp(0.25, 1.0);
                        let cell = &mut fs.slabs[r][c];
                        if cell.intact {
                            cell.hp -= structural_dmg * mul * falloff;
                            damaged = true;
                            if cell.hp <= 0.0 {
                                cell.hp = 0.0;
                                cell.intact = false;
                                destroyed_slabs += 1;
                                add_rubble.push((f.saturating_sub(1), centers[r][c]));
                            }
                        }
                    }
                }
            }
            if d_wall <= radius {
                fs.front_wall_hp -= structural_dmg * mul;
                damaged = true;
                if fs.front_wall_hp <= 0.0 && !fs.front_wall_breached {
                    fs.front_wall_hp = 0.0;
                    fs.front_wall_breached = true;
                    if f == 0 {
                        for w in &mut self.windows {
                            w.broken = true;
                        }
                    }
                }
            }
        }

        let mut out_rubble = Vec::with_capacity(add_rubble.len());
        for (dst_f, pos) in add_rubble {
            if let Some(fs) = self.floors.get_mut(dst_f) {
                fs.rubble.push(pos);
            }
            out_rubble.push(pos);
        }

        if target_floor <= 1 {
            for door in std::iter::once(&mut self.front_door)
                .chain(self.back_door.iter_mut())
                .chain(self.inner_door.iter_mut())
            {
                if door.at.dist(at) <= radius {
                    door.hp -= structural_dmg;
                    door.barricade_hp = (door.barricade_hp - structural_dmg).max(0.0);
                    damaged = true;
                    if door.hp <= 0.0 {
                        door.hp = 0.0;
                        door.destroyed = true;
                        door.open = true;
                        door.barricade_hp = 0.0;
                    }
                }
            }
        }

        if damaged {
            self.revision = self.revision.wrapping_add(1);
        }
        (destroyed_slabs, out_rubble)
    }

    /// Resolve player/soldier collision inside `floor == 0` (doorway funnel) or upper enclosed stories (`floor >= 1`).
    pub fn step_inside_floor(&self, p: P2, radius: f32, floor: u8) -> P2 {
        if floor == 0 {
            return self.step_inside(p, radius);
        }
        let (mut lu, mut lv) = self.to_local(p);
        let two_room = self.depth >= 5.2 && self.half_w >= 2.5 && self.split_v < self.depth - 0.5;
        let front_solid = LocalBox { cx: 0.0, cy: 0.0, hw: self.half_w + 0.16, hd: 0.16, h: FLOOR_H };
        let back_solid = LocalBox { cx: 0.0, cy: self.depth, hw: self.half_w + 0.16, hd: 0.16, h: FLOOR_H };
        let left_solid = LocalBox { cx: -self.half_w, cy: self.depth * 0.5, hw: 0.16, hd: self.depth * 0.5 + 0.16, h: FLOOR_H };
        let right_solid = LocalBox { cx: self.half_w, cy: self.depth * 0.5, hw: 0.16, hd: self.depth * 0.5 + 0.16, h: FLOOR_H };
        let mut part_boxes = [left_solid, right_solid, front_solid, back_solid];
        for _ in 0..3 {
            let mut moved = false;
            for b in &mut part_boxes {
                if let Some((nu, nv)) = b.push_out(lu, lv, radius) {
                    lu = nu;
                    lv = nv;
                    moved = true;
                }
            }
            if two_room {
                for w in &self.walk_walls {
                    if (w.cy - self.split_v).abs() < 0.25 && w.hd < 0.25 {
                        if let Some((nu, nv)) = w.push_out(lu, lv, radius) {
                            lu = nu;
                            lv = nv;
                            moved = true;
                        }
                    }
                }
            }
            if !moved {
                break;
            }
        }
        self.to_world(lu, lv)
    }

    /// Resolve player/soldier collision against walls, furniture, and barricaded doors in local space.
    pub fn step_inside(&self, p: P2, radius: f32) -> P2 {
        let (mut lu, mut lv) = self.to_local(p);
        // Funnel grazing threshold approaches smoothly through unlocked doorways instead of bouncing outside.
        if self.front_door.barricade_hp <= 0.0 && (-0.65..=0.35).contains(&lv) {
            let clear = (self.front_door.width * 0.5 - radius - 0.02).max(0.18);
            let off = lu - self.front_door.u_center;
            if off.abs() <= self.front_door.width * 0.5 + radius * 0.65 && off.abs() > clear {
                lu = self.front_door.u_center + off.clamp(-clear, clear);
            }
        }
        if let Some(bd) = &self.back_door {
            if bd.barricade_hp <= 0.0 && (self.depth - 0.35..=self.depth + 0.65).contains(&lv) {
                let clear = (bd.width * 0.5 - radius - 0.02).max(0.18);
                let off = lu - bd.u_center;
                if off.abs() <= bd.width * 0.5 + radius * 0.65 && off.abs() > clear {
                    lu = bd.u_center + off.clamp(-clear, clear);
                }
            }
        }

        let door_hw = self.front_door.width * 0.5;
        let front_block = (self.front_door.barricade_hp > 0.0).then_some(LocalBox {
            cx: self.front_door.u_center,
            cy: 0.0,
            hw: door_hw,
            hd: 0.22,
            h: 2.0,
        });
        let back_block = self.back_door.as_ref().and_then(|bd| {
            (bd.barricade_hp > 0.0).then_some(LocalBox {
                cx: bd.u_center,
                cy: self.depth,
                hw: bd.width * 0.5,
                hd: 0.22,
                h: 2.0,
            })
        });

        for _ in 0..3 {
            let mut moved = false;
            for b in self.walk_walls.iter().chain(self.colliders.iter()).chain(front_block.iter()).chain(back_block.iter()) {
                if let Some((nu, nv)) = b.push_out(lu, lv, radius) {
                    lu = nu;
                    lv = nv;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        self.to_world(lu, lv)
    }

    /// True if segment `a -> b` passes unobstructed inside this interior's sight walls (and curtains when `!ignore_curtains`).
    pub fn clear_line_inside(&self, a: P2, b: P2, ignore_curtains: bool) -> bool {
        let (u0, v0) = self.to_local(a);
        let (u1, v1) = self.to_local(b);
        for w in &self.sight_walls {
            if w.segment_hits(u0, v0, u1, v1) {
                return false;
            }
        }
        if !ignore_curtains && self.curtains_closed {
            for win in &self.windows {
                let wb = LocalBox {
                    cx: (win.u_a + win.u_b) * 0.5,
                    cy: 0.0,
                    hw: ((win.u_b - win.u_a).abs() * 0.5).max(0.2),
                    hd: 0.14,
                    h: 2.4,
                };
                if wb.segment_hits(u0, v0, u1, v1) {
                    return false;
                }
            }
        }
        if !self.front_door.destroyed && (self.front_door.barricade_hp > 0.0 || (!ignore_curtains && !self.front_door.open)) {
            let fd = LocalBox {
                cx: self.front_door.u_center,
                cy: 0.0,
                hw: self.front_door.width * 0.5,
                hd: 0.16,
                h: 2.2,
            };
            if fd.segment_hits(u0, v0, u1, v1) {
                return false;
            }
        }
        if let Some(bd) = &self.back_door {
            if !bd.destroyed && (bd.barricade_hp > 0.0 || (!ignore_curtains && !bd.open)) {
                let bb = LocalBox {
                    cx: bd.u_center,
                    cy: self.depth,
                    hw: bd.width * 0.5,
                    hd: 0.16,
                    h: 2.2,
                };
                if bb.segment_hits(u0, v0, u1, v1) {
                    return false;
                }
            }
        }
        if let Some(id) = &self.inner_door {
            if !id.destroyed && (id.barricade_hp > 0.0 || (!ignore_curtains && !id.open)) {
                let ib = LocalBox {
                    cx: id.u_center,
                    cy: self.split_v,
                    hw: id.width * 0.5,
                    hd: 0.14,
                    h: 2.2,
                };
                if ib.segment_hits(u0, v0, u1, v1) {
                    return false;
                }
            }
        }
        true
    }

    pub fn has_unboarded_broken_window(&self) -> bool {
        self.windows.iter().any(|w| w.broken && !w.boarded)
    }

    pub fn open_doors_count(&self) -> usize {
        let mut n = usize::from(self.front_door.open && self.front_door.barricade_hp <= 0.0);
        if let Some(bd) = &self.back_door {
            if bd.open && bd.barricade_hp <= 0.0 {
                n += 1;
            }
        }
        n
    }

    pub fn blocks_mortar(&self, p: P2) -> f32 {
        match self.zone(p) {
            Some(RoomZone::Basement) => 0.0,
            Some(RoomZone::BackRoom) => 0.08,
            Some(RoomZone::FrontHall) => 0.22,
            None => 1.0,
        }
    }

    /// Build ground-floor cutaway room meshes (delegates to `build_mesh_for_floor(0)`).
    pub fn build_mesh(&self, _pal: &Palette) -> InteriorMesh {
        self.build_mesh_for_floor(0)
    }

    /// Build sliced multi-story cutaway meshes (`0..=max_floor`) including `2x3` destructible floor slabs,
    /// concrete rubble, exposed rebar holes, staircases, front wall breaches, and ground furniture.
    pub fn build_mesh_for_floor(&self, max_floor: u8) -> InteriorMesh {
        let mut m = InteriorMesh::default();
        let yaw = self.u.y.atan2(self.u.x);
        let top_f = (max_floor as usize).min(self.floors.len().saturating_sub(1));

        let (floor_col, back_col) = match self.kind {
            InteriorKind::Pharmacy => (Rgb { r: 0.78, g: 0.84, b: 0.82 }, Rgb { r: 0.66, g: 0.74, b: 0.72 }),
            InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::Residential => {
                (Rgb { r: 0.54, g: 0.39, b: 0.26 }, Rgb { r: 0.45, g: 0.33, b: 0.22 })
            }
            InteriorKind::BasementShelter => (Rgb { r: 0.36, g: 0.38, b: 0.41 }, Rgb { r: 0.30, g: 0.32, b: 0.35 }),
            InteriorKind::Food | InteriorKind::Drink => {
                (Rgb { r: 0.62, g: 0.52, b: 0.40 }, Rgb { r: 0.52, g: 0.42, b: 0.32 })
            }
            InteriorKind::Hardware => (Rgb { r: 0.46, g: 0.47, b: 0.49 }, Rgb { r: 0.38, g: 0.39, b: 0.41 }),
            _ => (Rgb { r: 0.64, g: 0.64, b: 0.66 }, Rgb { r: 0.52, g: 0.53, b: 0.56 }),
        };
        let upper_floor_col = Rgb { r: 0.60, g: 0.48, b: 0.34 };
        let scorch_col = Rgb { r: 0.24, g: 0.22, b: 0.20 };
        let rebar_col = Rgb { r: 0.44, g: 0.28, b: 0.20 };
        let concrete_edge = Rgb { r: 0.32, g: 0.32, b: 0.34 };

        let wall_col = match self.kind {
            InteriorKind::BasementShelter => Rgb { r: 0.42, g: 0.44, b: 0.46 },
            InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::Residential => {
                Rgb { r: 0.78, g: 0.75, b: 0.69 }
            }
            InteriorKind::Pharmacy => Rgb { r: 0.84, g: 0.88, b: 0.86 },
            _ => Rgb { r: 0.76, g: 0.75, b: 0.73 },
        };
        let wainscot = Rgb { r: 0.36, g: 0.42, b: 0.40 };
        let cap_col = Rgb { r: 0.22, g: 0.24, b: 0.28 };
        let step_col = Rgb { r: 0.62, g: 0.60, b: 0.58 };
        let iron_dark = Rgb { r: 0.18, g: 0.19, b: 0.21 };
        let rail_amber = Rgb { r: 0.76, g: 0.54, b: 0.22 };

        let cw = (self.half_w * 2.0) / 3.0;
        let cd = self.depth * 0.5;
        let h_for = |out_dir: P2, story_h: f32| -> f32 {
            if out_dir.dot(CAM_DIR) > 0.12 { 0.64 } else { story_h }
        };
        let two_room = self.depth >= 5.2 && self.half_w >= 2.5 && self.split_v < self.depth - 0.5;

        for f in 0..=top_f {
            let base_y = f as f32 * FLOOR_H;
            let floor_top = base_y + 0.08;
            let story_h = if f < top_f { FLOOR_H } else { self.wall_h };
            let front_h = h_for(self.u.right(), story_h);
            let back_h = h_for(self.v, story_h);
            let left_h = h_for(self.u.mul(-1.0), story_h);
            let right_h = h_for(self.u, story_h);

            // 2x3 destructible concrete floor slabs per story.
            if let Some(fs) = self.floors.get(f) {
                for r in 0..2 {
                    for c in 0..3 {
                        let lu = -self.half_w + (c as f32 + 0.5) * cw;
                        let lv = (r as f32 + 0.5) * cd;
                        let cell = fs.slabs[r][c];
                        let w_pos = self.to_world(lu, lv);
                        if cell.intact {
                            let base_c = if f == 0 {
                                if r == 0 { floor_col } else { back_col }
                            } else {
                                upper_floor_col
                            };
                            let col = if cell.hp < 65.0 { base_c.mix(scorch_col, 0.55) } else { base_c };
                            m.static_solid
                                .boxed(w_pos, base_y + 0.01, [cw - 0.03, 0.11, cd - 0.03], yaw, col);
                        } else if f >= 1 {
                            // Open structural hole through to floor below with broken concrete lip + exposed rebar stubs!
                            m.static_solid.boxed(
                                self.to_world(lu, lv - cd * 0.45),
                                base_y + 0.01,
                                [cw, 0.13, 0.18],
                                yaw,
                                concrete_edge,
                            );
                            m.static_solid.boxed(
                                self.to_world(lu, lv + cd * 0.45),
                                base_y + 0.01,
                                [cw, 0.13, 0.18],
                                yaw,
                                concrete_edge,
                            );
                            for k in [-0.28f32, 0.0, 0.28] {
                                m.static_solid.boxed(
                                    self.to_world(lu + k * cw, lv - cd * 0.30),
                                    base_y + 0.05,
                                    [0.05, 0.04, 0.42],
                                    yaw,
                                    rebar_col,
                                );
                                m.static_solid.boxed(
                                    self.to_world(lu + k * cw, lv + cd * 0.30),
                                    base_y + 0.05,
                                    [0.05, 0.04, 0.42],
                                    yaw,
                                    rebar_col,
                                );
                            }
                        } else {
                            // Ground floor crater scar.
                            m.static_solid
                                .boxed(w_pos, base_y + 0.01, [cw - 0.03, 0.04, cd - 0.03], yaw, scorch_col);
                        }
                    }
                }
                for rb in &fs.rubble {
                    for (ri, (ox, oy, sc)) in [(-0.45f32, -0.30f32, 0.42f32), (0.35, 0.25, 0.36), (0.05, -0.42, 0.30)]
                        .into_iter()
                        .enumerate()
                    {
                        m.static_solid.boxed(
                            rb.add(P2::new(ox, oy)),
                            floor_top,
                            [sc * 1.2, 0.22 + ri as f32 * 0.07, sc],
                            yaw + ri as f32 * 0.8,
                            concrete_edge,
                        );
                    }
                }
            }

            // Stairwell climbing treads & handrails when multi-story.
            if self.num_floors > 1 {
                let sw = self.stairwell_world();
                m.static_solid.boxed(sw, floor_top, [1.28, 0.04, 1.28], yaw, rail_amber);
                if (f as u8) + 1 < self.num_floors {
                    for st in 0..7 {
                        let t = st as f32 / 7.0;
                        let st_p = self.to_world(self.stairwell_u, self.stairwell_v - 0.55 + t * 1.10);
                        let sy = floor_top + (st + 1) as f32 * (FLOOR_H / 7.5);
                        m.static_solid.boxed(st_p, sy - 0.14, [0.95, 0.14, 0.24], yaw, step_col);
                    }
                    m.static_solid.boxed(
                        self.to_world(self.stairwell_u - 0.52, self.stairwell_v),
                        floor_top,
                        [0.08, FLOOR_H * 0.65, 1.20],
                        yaw,
                        iron_dark,
                    );
                }
            }

            let emit_wall = |soup: &mut Soup, cx: f32, cy: f32, w: f32, d: f32, h: f32, col: Rgb| {
                soup.boxed(self.to_world(cx, cy), floor_top, [w, h, d], yaw, col);
                if h < 1.0 {
                    soup.boxed(self.to_world(cx, cy), floor_top + h, [w, 0.05, d], yaw, cap_col);
                } else {
                    soup.boxed(self.to_world(cx, cy), floor_top, [w + 0.01, 0.72, d + 0.01], yaw, wainscot);
                }
            };

            let breached = self.floors.get(f).is_some_and(|fs| fs.front_wall_breached);
            if f == 0 {
                // Ground floor walls with window sills & front/back door cutouts.
                for b in &self.walk_walls {
                    let is_front = (b.cy - 0.0).abs() < 0.25 && b.hd < 0.35;
                    if is_front {
                        continue;
                    }
                    let is_back = (b.cy - self.depth).abs() < 0.25 && b.hd < 0.35;
                    let is_left = (b.cx + self.half_w).abs() < 0.25 && b.hw < 0.35;
                    let is_right = (b.cx - self.half_w).abs() < 0.25 && b.hw < 0.35;
                    let is_part = (b.cy - self.split_v).abs() < 0.25 && b.hd < 0.35;
                    let h = if is_back {
                        back_h
                    } else if is_left {
                        left_h
                    } else if is_right {
                        right_h
                    } else if is_part {
                        1.48
                    } else {
                        story_h
                    };
                    emit_wall(&mut m.static_solid, b.cx, b.cy, b.hw * 2.0, b.hd * 2.0, h, wall_col);
                }

                let door_hw = self.front_door.width * 0.5;
                let sill_h = front_h.min(0.78);
                if !breached {
                    for (s0, s1) in [(-self.half_w, -door_hw), (door_hw, self.half_w)] {
                        if s1 - s0 > 0.2 {
                            let cx = (s0 + s1) * 0.5;
                            emit_wall(&mut m.static_solid, cx, 0.0, s1 - s0, 0.24, sill_h, wall_col);
                            if front_h > 1.5 {
                                m.static_solid.boxed(self.to_world(cx, 0.0), floor_top + 2.20, [s1 - s0, front_h - 2.20, 0.24], yaw, wall_col);
                                m.static_solid.boxed(self.to_world(s0 + 0.14, 0.0), floor_top + 0.78, [0.24, 1.44, 0.26], yaw, wall_col);
                                m.static_solid.boxed(self.to_world(s1 - 0.14, 0.0), floor_top + 0.78, [0.24, 1.44, 0.26], yaw, wall_col);
                            }
                        }
                    }
                } else {
                    // jagged blast hole on front wall!
                    for side_u in [-self.half_w + 0.45, self.half_w - 0.45] {
                        emit_wall(&mut m.static_solid, side_u, 0.0, 0.90, 0.26, sill_h * 0.75, scorch_col);
                    }
                }
            } else {
                // Upper floor outer perimeter + partition divider + optional front blast breach.
                emit_wall(&mut m.static_solid, -self.half_w, self.depth * 0.5, 0.26, self.depth + 0.24, left_h, wall_col);
                emit_wall(&mut m.static_solid, self.half_w, self.depth * 0.5, 0.26, self.depth + 0.24, right_h, wall_col);
                emit_wall(&mut m.static_solid, 0.0, self.depth, self.half_w * 2.0 + 0.24, 0.26, back_h, wall_col);
                if breached {
                    let wing = (self.half_w - 1.2).max(0.4);
                    emit_wall(&mut m.static_solid, -self.half_w + wing * 0.5, 0.0, wing, 0.26, front_h * 0.65, scorch_col);
                    emit_wall(&mut m.static_solid, self.half_w - wing * 0.5, 0.0, wing, 0.26, front_h * 0.65, scorch_col);
                } else {
                    emit_wall(&mut m.static_solid, 0.0, 0.0, self.half_w * 2.0 + 0.24, 0.26, front_h.min(0.68), wall_col);
                }
                if two_room {
                    for b in &self.walk_walls {
                        if (b.cy - self.split_v).abs() < 0.25 && b.hd < 0.25 {
                            emit_wall(&mut m.static_solid, b.cx, b.cy, b.hw * 2.0, b.hd * 2.0, 1.42, wall_col);
                        }
                    }
                }
                // Simple upper-floor bed / apartment table away from stairwell.
                if self.half_w >= 2.3 && self.depth >= 4.2 {
                    let fx = if self.stairwell_u < 0.0 { self.half_w - 0.78 } else { -self.half_w + 0.78 };
                    m.static_solid.boxed(
                        self.to_world(fx, self.depth * 0.36),
                        floor_top,
                        [0.95, 0.52, 1.75],
                        yaw,
                        Rgb { r: 0.56, g: 0.24, b: 0.22 },
                    );
                }
            }
        }

        // Ground-floor (f == 0) street windows, radiators, boards, curtains, barricades, stove, locker & furniture.
        let floor_h = 0.08;
        let sill_h = h_for(self.u.right(), self.wall_h).min(0.78);
        let rad_col = Rgb { r: 0.72, g: 0.74, b: 0.76 };
        let glass_col = Rgb { r: 0.45, g: 0.62, b: 0.75 };
        let board_col = Rgb { r: 0.48, g: 0.35, b: 0.22 };
        let curtain_col = if self.kind == InteriorKind::PlayerHome {
            Rgb { r: 0.54, g: 0.24, b: 0.20 }
        } else {
            Rgb { r: 0.20, g: 0.30, b: 0.36 }
        };
        for win in &self.windows {
            let cx = (win.u_a + win.u_b) * 0.5;
            let span = (win.u_b - win.u_a).abs().max(0.8);
            m.static_solid.boxed(self.to_world(cx, 0.28), floor_h, [span * 0.68, 0.56, 0.14], yaw, rad_col);
            m.window_glass.boxed(self.to_world(cx, 0.0), floor_h + sill_h * 0.65, [span, 0.85, 0.04], yaw, glass_col);
            for p in 0..3 {
                m.window_boards.boxed(
                    self.to_world(cx, 0.12),
                    floor_h + 0.52 + p as f32 * 0.32,
                    [span + 0.15, 0.16, 0.08],
                    yaw,
                    board_col,
                );
            }
            m.curtains.boxed(self.to_world(cx, 0.20), floor_h + 0.18, [span + 0.12, 1.45, 0.07], yaw, curtain_col);
        }

        // Permanent doorframe & threshold doormat (door leaves/curtains render dynamically via ActorPools).
        self.append_exterior_portal(&Palette::default(), &mut m.exterior_portal, false);

        // Door barricades (drawn only when barricade_hp > 0).
        let bar_col = Rgb { r: 0.44, g: 0.30, b: 0.17 };
        for p in 0..3 {
            m.barricade_front.boxed(
                self.to_world(self.front_door.u_center, 0.20),
                floor_h + 0.35 + p as f32 * 0.55,
                [self.front_door.width + 0.25, 0.22, 0.22],
                yaw,
                bar_col,
            );
        }
        if let Some(bd) = &self.back_door {
            for p in 0..3 {
                m.barricade_back.boxed(
                    self.to_world(bd.u_center, self.depth - 0.20),
                    floor_h + 0.35 + p as f32 * 0.55,
                    [bd.width + 0.25, 0.22, 0.22],
                    yaw,
                    bar_col,
                );
            }
        }

        let wood_dark = Rgb { r: 0.38, g: 0.25, b: 0.15 };
        let wood_light = Rgb { r: 0.62, g: 0.47, b: 0.30 };
        let white_med = Rgb { r: 0.88, g: 0.92, b: 0.90 };
        let fridge_blue = Rgb { r: 0.72, g: 0.82, b: 0.88 };
        let bed_red = Rgb { r: 0.66, g: 0.22, b: 0.20 };
        let crate_col = Rgb { r: 0.50, g: 0.39, b: 0.24 };

        for (ci, c) in self.colliders.iter().enumerate() {
            let p = self.to_world(c.cx, c.cy);
            let col = match self.kind {
                InteriorKind::Pharmacy => if ci == 0 { white_med } else { Rgb { r: 0.34, g: 0.64, b: 0.52 } },
                InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::Residential => {
                    if ci == 0 { bed_red } else if ci == 1 { wood_dark } else { iron_dark }
                }
                InteriorKind::Clothes => if ci == 0 { wood_light } else { Rgb { r: 0.58, g: 0.28, b: 0.32 } },
                InteriorKind::Drink | InteriorKind::Shop => if ci == 1 { fridge_blue } else { wood_light },
                InteriorKind::Food => if ci == 0 { wood_dark } else { crate_col },
                InteriorKind::Hardware | InteriorKind::BasementShelter => {
                    if ci == 0 { iron_dark } else { crate_col }
                }
                InteriorKind::Hotel => if ci == 0 { wood_dark } else { bed_red },
            };
            m.static_solid.boxed(p, floor_h, [c.hw * 2.0, c.h, c.hd * 2.0], yaw, col);
        }

        if let Some(st) = self.stove_at {
            m.static_solid.boxed(st, floor_h, [0.68, 0.85, 0.68], yaw, iron_dark);
            m.static_solid.boxed(st, floor_h + 0.85, [0.16, 1.35, 0.16], yaw, iron_dark);
            m.static_solid.boxed(st, floor_h + 0.18, [0.36, 0.24, 0.72], yaw, Rgb { r: 0.85, g: 0.34, b: 0.12 });
        }
        if let Some(lk) = self.locker_at {
            m.static_solid.boxed(lk, floor_h, [0.95, 1.55, 0.58], yaw, Rgb { r: 0.30, g: 0.40, b: 0.34 });
        }

        m
    }

    /// Build permanent exterior doorframe jambs, lintel, and sidewalk threshold doormat.
    /// When `include_leaf` is `true` (static `--shots`/`--bench`), also bakes parted cotton curtains or ajar door leaves.
    pub fn append_exterior_portal(&self, _pal: &Palette, soup: &mut Soup, include_leaf: bool) {
        let yaw = self.u.y.atan2(self.u.x);
        let door_hw = self.front_door.width * 0.5;
        let trim_wood = Rgb { r: 0.34, g: 0.22, b: 0.13 };
        let door_green = Rgb { r: 0.14, g: 0.28, b: 0.19 };
        let steel_door = Rgb { r: 0.32, g: 0.36, b: 0.42 };
        let glass_door = Rgb { r: 0.26, g: 0.40, b: 0.52 };
        let mat_base = Rgb { r: 0.16, g: 0.17, b: 0.19 };
        let threshold_col = if self.front_door.is_security {
            Rgb { r: 0.72, g: 0.58, b: 0.24 }
        } else if matches!(self.kind, InteriorKind::Residential | InteriorKind::NeighbourHome) {
            Rgb { r: 0.64, g: 0.44, b: 0.22 }
        } else {
            Rgb { r: 0.26, g: 0.62, b: 0.64 }
        };

        soup.boxed(self.to_world(0.0, -0.36), 0.005, [self.front_door.width + 0.24, 0.035, 0.76], yaw, mat_base);
        soup.boxed(self.to_world(0.0, -0.34), 0.015, [self.front_door.width + 0.06, 0.05, 0.54], yaw, threshold_col);
        soup.boxed(self.to_world(-door_hw - 0.11, -0.22), 0.0, [0.20, 2.40, 0.24], yaw, trim_wood);
        soup.boxed(self.to_world(door_hw + 0.11, -0.22), 0.0, [0.20, 2.40, 0.24], yaw, trim_wood);
        soup.boxed(self.to_world(0.0, -0.22), 2.30, [self.front_door.width + 0.44, 0.24, 0.28], yaw, trim_wood);

        if include_leaf {
            match self.front_door.style {
                DoorStyle::CottonCurtain => {
                    let half_w = 0.36;
                    let inset_u = (door_hw - half_w * 0.5).max(0.45);
                    soup.boxed(self.to_world(-inset_u, -0.18), 0.08, [half_w, 2.18, 0.11], yaw, door_green);
                    soup.boxed(self.to_world(inset_u, -0.18), 0.08, [half_w, 2.18, 0.11], yaw, door_green);
                }
                DoorStyle::SteelSecurity => {
                    soup.boxed(self.to_world(-door_hw + 0.32, -0.36), 0.02, [self.front_door.width * 0.56, 2.24, 0.09], yaw + 0.58, steel_door);
                }
                DoorStyle::ShopGlass => {
                    soup.boxed(self.to_world(-door_hw + 0.26, -0.34), 0.02, [self.front_door.width * 0.48, 2.20, 0.07], yaw + 0.45, glass_door);
                }
                DoorStyle::WoodFlat => {
                    soup.boxed(self.to_world(0.0, -0.16), 0.02, [self.front_door.width * 0.94, 2.22, 0.08], yaw, Rgb { r: 0.42, g: 0.26, b: 0.15 });
                }
            }
        }

        if let Some(bd) = &self.back_door {
            soup.boxed(self.to_world(bd.u_center, self.depth + 0.34), 0.015, [bd.width + 0.12, 0.045, 0.58], yaw, threshold_col);
            soup.boxed(self.to_world(bd.u_center - bd.width * 0.5 - 0.09, self.depth + 0.16), 0.0, [0.16, 2.30, 0.20], yaw, trim_wood);
            soup.boxed(self.to_world(bd.u_center + bd.width * 0.5 + 0.09, self.depth + 0.16), 0.0, [0.16, 2.30, 0.20], yaw, trim_wood);
        }
    }
}

/// Hinged door panel mesh (`x = 0` is left jamb hinge pivot, stretches `0..width` along `+X`).
pub fn build_door_leaf_soup(style: DoorStyle, width: f32) -> Soup {
    let mut s = Soup::default();
    let w = width.clamp(0.8, 2.2);
    let cx = w * 0.5;
    match style {
        DoorStyle::SteelSecurity => {
            let steel = Rgb { r: 0.30, g: 0.34, b: 0.40 };
            let brass = Rgb { r: 0.78, g: 0.62, b: 0.22 };
            s.boxed(P2::new(cx, 0.0), 0.02, [w * 0.96, 2.24, 0.08], 0.0, steel);
            s.boxed(P2::new(w * 0.84, 0.0), 1.02, [0.14, 0.24, 0.13], 0.0, brass);
        }
        DoorStyle::ShopGlass => {
            let frame = Rgb { r: 0.22, g: 0.25, b: 0.28 };
            let glass = Rgb { r: 0.38, g: 0.58, b: 0.72 };
            let handle = Rgb { r: 0.82, g: 0.84, b: 0.86 };
            s.boxed(P2::new(cx, 0.0), 0.02, [w * 0.96, 0.48, 0.07], 0.0, frame);
            s.boxed(P2::new(cx, 0.0), 1.98, [w * 0.96, 0.26, 0.07], 0.0, frame);
            s.boxed(P2::new(0.08, 0.0), 0.50, [0.14, 1.50, 0.07], 0.0, frame);
            s.boxed(P2::new(w - 0.08, 0.0), 0.50, [0.14, 1.50, 0.07], 0.0, frame);
            s.boxed(P2::new(cx, 0.0), 0.50, [w * 0.74, 1.48, 0.04], 0.0, glass);
            s.boxed(P2::new(w * 0.82, 0.0), 0.96, [0.06, 0.46, 0.14], 0.0, handle);
        }
        _ => {
            let wood = Rgb { r: 0.44, g: 0.28, b: 0.16 };
            let trim = Rgb { r: 0.32, g: 0.20, b: 0.12 };
            let brass = Rgb { r: 0.76, g: 0.60, b: 0.24 };
            s.boxed(P2::new(cx, 0.0), 0.02, [w * 0.96, 2.22, 0.07], 0.0, wood);
            s.boxed(P2::new(cx, 0.0), 0.32, [w * 0.70, 0.72, 0.09], 0.0, trim);
            s.boxed(P2::new(cx, 0.0), 1.24, [w * 0.70, 0.74, 0.09], 0.0, trim);
            s.boxed(P2::new(w * 0.85, 0.0), 1.05, [0.10, 0.14, 0.12], 0.0, brass);
        }
    }
    s
}

/// Northeast padded cotton doorway curtain (`棉门帘`), centered at local `(0, 0)`.
pub fn build_curtain_soup(open: bool, width: f32) -> Soup {
    let mut s = Soup::default();
    let green = Rgb { r: 0.15, g: 0.30, b: 0.20 };
    let stitch = Rgb { r: 0.22, g: 0.40, b: 0.28 };
    let hw = width.clamp(0.8, 2.2) * 0.5;
    if open {
        let flap_w = 0.32;
        let off = (hw - flap_w * 0.45).max(0.38);
        s.boxed(P2::new(-off, 0.0), 0.08, [flap_w, 2.18, 0.10], 0.0, green);
        s.boxed(P2::new(off, 0.0), 0.08, [flap_w, 2.18, 0.10], 0.0, green);
    } else {
        let panel_w = hw * 0.96;
        for &side in &[-1.0f32, 1.0] {
            let cx = side * panel_w * 0.52;
            s.boxed(P2::new(cx, 0.0), 0.08, [panel_w, 2.16, 0.10], 0.0, green);
            s.boxed(P2::new(cx, 0.0), 1.15, [panel_w * 0.75, 0.06, 0.12], 0.0, stitch);
        }
    }
    s
}

fn edge_is_inside_footprint(ring: &[P2], origin: P2, u: P2, v: P2, hw: f32, depth: f32, front_edge_idx: usize) -> bool {
    let to_w = |lu: f32, lv: f32| origin.add(u.mul(lu)).add(v.mul(lv));
    // Sample grid + inset corners inside the rectangle `[ -hw..hw ] x [ 0.22..depth-0.14 ]`.
    for &lu in &[-hw + 0.14, -hw * 0.5, 0.0, hw * 0.5, hw - 0.14] {
        for &lv in &[0.22, depth * 0.35, depth * 0.7, depth - 0.14] {
            let p = to_w(lu, lv);
            if !point_in_ring(p, ring) {
                return false;
            }
            for i in 0..ring.len() {
                if i == front_edge_idx {
                    continue;
                }
                let q = nearest_on_segment(p, ring[i], ring[(i + 1) % ring.len()]);
                if p.dist(q) < 0.12 {
                    return false;
                }
            }
        }
    }
    true
}

fn building_thick_at(ring: &[P2], origin: P2, v: P2) -> Option<f32> {
    let a = origin.add(v.mul(0.25));
    let b = origin.add(v.mul(28.0));
    let spans = spans_inside(a, b, ring);
    spans.first().map(|&(t0, t1)| (t1 - t0) * 27.75 + 0.25)
}

fn nearest_sign_kind(city: &City, origin: P2, p0: P2, p1: P2, fallback: ShopKind) -> (String, ShopKind, bool) {
    let mut best_score = (false, false, f32::NEG_INFINITY);
    let mut best = None;
    for s in &city.signs {
        let q = nearest_on_segment(s.at, p0, p1);
        let d_seg = s.at.dist(q);
        let d_ori = s.at.dist(origin);
        if d_seg <= 14.0 {
            let k = crate::survival::shop_kind(&s.text, &s.text_ru);
            let is_pharm = k == ShopKind::Pharmacy;
            let near_ori = d_ori <= 10.0;
            let score = (is_pharm, near_ori, -d_ori);
            if score > best_score {
                best_score = score;
                best = Some((s, k, near_ori));
            }
        }
    }
    if let Some((s, k, near_ori)) = best {
        let label = if !s.text.is_empty() { s.text.clone() } else { s.text_ru.clone() };
        (label, k, near_ori)
    } else {
        ("沿街商铺".into(), fallback, false)
    }
}

struct RoadGrid {
    cells: std::collections::HashMap<(i32, i32), Vec<(P2, P2)>>,
}

impl RoadGrid {
    fn new(city: &City) -> Self {
        let mut cells: std::collections::HashMap<(i32, i32), Vec<(P2, P2)>> = std::collections::HashMap::new();
        let rc = |v: f32| (v / 32.0).floor() as i32;
        for r in &city.roads {
            for w in r.line.windows(2) {
                let (a, b) = (w[0], w[1]);
                let x0 = rc(a.x.min(b.x) - 1.0);
                let x1 = rc(a.x.max(b.x) + 1.0);
                let y0 = rc(a.y.min(b.y) - 1.0);
                let y1 = rc(a.y.max(b.y) + 1.0);
                for cx in x0..=x1 {
                    for cy in y0..=y1 {
                        cells.entry((cx, cy)).or_default().push((a, b));
                    }
                }
            }
        }
        Self { cells }
    }

    fn dist(&self, p: P2) -> f32 {
        let cx = (p.x / 32.0).floor() as i32;
        let cy = (p.y / 32.0).floor() as i32;
        let mut best = 48.0f32;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(segs) = self.cells.get(&(cx + dx, cy + dy)) {
                    for &(a, b) in segs {
                        best = best.min(p.dist(nearest_on_segment(p, a, b)));
                    }
                }
            }
        }
        best
    }
}

struct FitCandidate {
    b_idx: usize,
    building_id: i64,
    origin: P2,
    u: P2,
    v: P2,
    hw: f32,
    depth: f32,
    back_u: Option<f32>,
    sign_name: String,
    has_near_sign: bool,
    shop_kind: ShopKind,
    b_name: String,
    b_levels: u8,
    is_residence: bool,
    on_street: bool,
    faces_cam: bool,
    road_dist: f32,
    dist_spawn: f32,
    dist_sos: f32,
    dist_north: f32,
}

/// Deterministically fits multi-unit multi-story enterable interiors across ALL eligible Chinese-bank
/// buildings in Heihe (shops, residences, Home, Neighbour SOS, & Basement bunker), prioritizing
/// street-facing and shop-sign aligned doorways with verified sidewalk clearances.
pub fn build_interiors(city: &City) -> Vec<Interior> {
    let ground = Ground::new(city);
    let road_grid = RoadGrid::new(city);
    let sos_anchor = P2::new(-260.0, -230.0);
    let north_anchor = P2::new(-30.0, -45.0);
    let in_playable_bounds = |p: P2| -> bool {
        p.x >= city.walk_x.0 + 6.2
            && p.x <= city.walk_x.1 - 6.2
            && p.y >= city.walk_south + 4.2
            && p.y <= city.bank_y(p.x) - 0.8
    };

    let mut cands: Vec<FitCandidate> = Vec::new();
    for (b_idx, b) in city.buildings.iter().enumerate() {
        if b.bank != Bank::China || !b.solid || b.footprint.len() < 3 {
            continue;
        }
        let d_spawn = b.centroid.dist(city.spawn);
        let d_sos = b.centroid.dist(sos_anchor);
        let d_north = b.centroid.dist(north_anchor);
        let is_residence = b.function == "Residence" || b.name_zh.contains("小区") || b.name_zh.contains("楼");
        let b_levels = b.levels.clamp(1, 5) as u8;

        let n = b.footprint.len();
        let mut edge_fits: Vec<FitCandidate> = Vec::new();
        for i in 0..n {
            let p0 = b.footprint[i];
            let p1 = b.footprint[(i + 1) % n];
            let edge_len = p0.dist(p1);
            if edge_len < 3.2 {
                continue;
            }
            let u = p1.sub(p0).norm();
            let v = P2::new(-u.y, u.x); // Inward CCW normal
            let faces_cam = (-v.x * CAM_DIR.x - v.y * CAM_DIR.y) > -0.15;

            // Collect candidate doorway offsets along this wall: matching signs first, then regular 14.5m storefront bays.
            let mut t_cands: Vec<f32> = Vec::with_capacity(8);
            let margin = (1.6 / edge_len).min(0.42);
            for s in &city.signs {
                let q = nearest_on_segment(s.at, p0, p1);
                if s.at.dist(q) <= 10.0 {
                    let raw_t = q.sub(p0).dot(u) / edge_len;
                    if raw_t >= -0.05 && raw_t <= 1.05 {
                        t_cands.push(raw_t.clamp(margin, 1.0 - margin));
                    }
                }
            }
            t_cands.push(0.5);
            if edge_len >= 18.0 {
                let bay_n = ((edge_len / 14.5).floor() as usize).clamp(2, 5);
                for bi in 0..bay_n {
                    let frac = (bi as f32 + 0.5) / bay_n as f32;
                    t_cands.push(frac.clamp(margin, 1.0 - margin));
                }
            } else if edge_len >= 12.0 {
                t_cands.push(0.30);
                t_cands.push(0.70);
            }

            for t in t_cands {
                let origin = p0.lerp(p1, t);
                let out_1 = origin.sub(v.mul(0.85));
                let out_2 = origin.sub(v.mul(1.45));
                if !in_playable_bounds(out_1)
                    || !in_playable_bounds(out_2)
                    || !ground.free(out_1, 0.34)
                    || !ground.free(out_2, 0.34)
                {
                    continue;
                }

                let road_dist = road_grid.dist(out_1);
                let on_street = road_dist <= 16.0;
                let span_min = (t * edge_len).min((1.0 - t) * edge_len);
                let hw_cap = (span_min - 0.25).min(6.0);
                if hw_cap < 1.35 {
                    continue;
                }
                let thick = building_thick_at(&b.footprint, origin, v).unwrap_or(8.0);
                let mut found_dim = None;

                let try_depths = [
                    (thick - 0.20).clamp(2.9, 10.8),
                    9.2,
                    7.6,
                    6.2,
                    5.2,
                    4.2,
                    3.2,
                ];
                for &dep in &try_depths {
                    if dep < 2.8 || dep > thick - 0.12 {
                        continue;
                    }
                    if !in_playable_bounds(origin.add(v.mul(dep))) {
                        continue;
                    }
                    for &hw_try in &[hw_cap, 4.6, 3.6, 2.8, 2.2, 1.5] {
                        let hw = hw_try.min(hw_cap);
                        if hw < 1.35 {
                            continue;
                        }
                        if edge_is_inside_footprint(&b.footprint, origin, u, v, hw, dep, i) {
                            found_dim = Some((hw, dep));
                            break;
                        }
                    }
                    if found_dim.is_some() {
                        break;
                    }
                }
                let Some((hw, dep)) = found_dim else { continue };

                let mut back_u = None;
                if (thick - dep).abs() < 1.2 {
                    for cand_u in [0.0, -hw * 0.45, hw * 0.45] {
                        let b_out1 = origin.add(u.mul(cand_u)).add(v.mul(dep + 1.05));
                        let b_out2 = origin.add(u.mul(cand_u)).add(v.mul(dep + 1.85));
                        if in_playable_bounds(b_out2)
                            && !point_in_ring(b_out1, &b.footprint)
                            && ground.free(b_out1, 0.34)
                            && ground.free(b_out2, 0.34)
                        {
                            back_u = Some(cand_u);
                            break;
                        }
                    }
                }

                let (sign_name, shop_kind, has_near_sign) = nearest_sign_kind(city, origin, p0, p1, ShopKind::Shop);
                edge_fits.push(FitCandidate {
                    b_idx,
                    building_id: b.id,
                    origin,
                    u,
                    v,
                    hw,
                    depth: dep,
                    back_u,
                    sign_name,
                    has_near_sign,
                    shop_kind,
                    b_name: b.name_zh.clone(),
                    b_levels,
                    is_residence,
                    on_street,
                    faces_cam,
                    road_dist,
                    dist_spawn: d_spawn,
                    dist_sos: d_sos,
                    dist_north: d_north,
                });
            }
        }

        // Prioritize best-oriented / shop-signed bays, keeping up to 5 disjoint bays per building!
        edge_fits.sort_by(|a, b_fit| {
            let a_ph = a.has_near_sign && a.shop_kind == ShopKind::Pharmacy;
            let b_ph = b_fit.has_near_sign && b_fit.shop_kind == ShopKind::Pharmacy;
            let ka = (a_ph, a.has_near_sign, a.on_street, a.faces_cam, a.back_u.is_some(), -a.road_dist, a.hw * a.depth);
            let kb = (b_ph, b_fit.has_near_sign, b_fit.on_street, b_fit.faces_cam, b_fit.back_u.is_some(), -b_fit.road_dist, b_fit.hw * b_fit.depth);
            kb.partial_cmp(&ka).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut accepted: Vec<FitCandidate> = Vec::new();
        for fit in edge_fits {
            if accepted.len() >= 5 {
                break;
            }
            let clashes = accepted.iter().any(|prev| {
                let min_sep = (fit.hw + prev.hw) * 0.92 + 0.4;
                fit.origin.dist(prev.origin) < min_sep
                    || fit.to_center().dist(prev.to_center()) < min_sep.min(fit.depth.min(prev.depth))
            });
            if !clashes {
                accepted.push(fit);
            }
        }
        // Fallback single-bay pass: cover compact sheds, alleys, diagonal/irregular footprints across playable Heihe.
        if accepted.is_empty() && (in_playable_bounds(b.centroid) || b.footprint.iter().any(|&p| in_playable_bounds(p))) {
            let mut best_fb: Option<(f32, FitCandidate)> = None;
            let try_span = |p0: P2, p1: P2, edge_idx: usize, fallback_edge: bool, best: &mut Option<(f32, FitCandidate)>| {
                let edge_len = p0.dist(p1);
                if edge_len < 1.35 {
                    return;
                }
                let u0 = p1.sub(p0).norm();
                let v_ccw = P2::new(-u0.y, u0.x);
                let mid = p0.lerp(p1, 0.5);
                let (u, v) = if point_in_ring(mid.add(v_ccw.mul(0.25)), &b.footprint) {
                    (u0, v_ccw)
                } else {
                    (u0.mul(-1.0), v_ccw.mul(-1.0))
                };
                let faces_cam = (-v.x * CAM_DIR.x - v.y * CAM_DIR.y) > -0.15;
                let fractions = [0.5f32, 0.30, 0.70, 0.18, 0.82];
                for t in fractions {
                    let origin = mid.lerp(if t < 0.5 { p0 } else { p1 }, (t - 0.5).abs() * 2.0);
                    let out_1 = origin.sub(v.mul(0.75));
                    if !in_playable_bounds(out_1) || point_in_ring(out_1, &b.footprint) || !ground.free(out_1, 0.24) {
                        continue;
                    }
                    let road_dist = road_grid.dist(out_1);
                    let span_min = (t * edge_len).min((1.0 - t) * edge_len);
                    let hw_cap = (span_min - 0.14).min(5.0);
                    if hw_cap < 0.72 {
                        continue;
                    }
                    let thick = building_thick_at(&b.footprint, origin, v).unwrap_or(4.5);
                    let try_deps = [(thick - 0.15).clamp(1.4, 9.0), 6.0, 4.2, 3.0, 2.2, 1.5];
                    for dep in try_deps {
                        if dep < 1.4 || dep > thick + 0.15 || !in_playable_bounds(origin.add(v.mul(dep * 0.5))) {
                            continue;
                        }
                        let mut found = false;
                        for hw_try in [hw_cap, 3.2, 2.2, 1.5, 1.1, 0.8] {
                            let hw = hw_try.min(hw_cap);
                            if hw < 0.72 {
                                continue;
                            }
                            let sample_ok = if !fallback_edge && hw >= 1.2 && dep >= 2.2 {
                                edge_is_inside_footprint(&b.footprint, origin, u, v, hw, dep, edge_idx)
                            } else {
                                point_in_ring(origin.add(v.mul(0.45)), &b.footprint)
                                    && point_in_ring(origin.add(v.mul(dep * 0.65)), &b.footprint)
                                    && point_in_ring(origin.add(u.mul(hw * 0.6)).add(v.mul(dep * 0.45)), &b.footprint)
                                    && point_in_ring(origin.sub(u.mul(hw * 0.6)).add(v.mul(dep * 0.45)), &b.footprint)
                            };
                            if sample_ok {
                                let score = (if road_dist <= 16.0 { 100.0 } else { 0.0 })
                                    + (if faces_cam { 25.0 } else { 0.0 })
                                    + hw * dep
                                    - road_dist * 0.4;
                                if best.as_ref().is_none_or(|(bs, _)| score > *bs) {
                                    let (sign_name, shop_kind, has_near_sign) =
                                        nearest_sign_kind(city, origin, p0, p1, ShopKind::Shop);
                                    *best = Some((
                                        score,
                                        FitCandidate {
                                            b_idx,
                                            building_id: b.id,
                                            origin,
                                            u,
                                            v,
                                            hw,
                                            depth: dep,
                                            back_u: None,
                                            sign_name,
                                            has_near_sign,
                                            shop_kind,
                                            b_name: b.name_zh.clone(),
                                            b_levels,
                                            is_residence,
                                            on_street: road_dist <= 16.0,
                                            faces_cam,
                                            road_dist,
                                            dist_spawn: d_spawn,
                                            dist_sos: d_sos,
                                            dist_north: d_north,
                                        },
                                    ));
                                }
                                found = true;
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
            };
            for i in 0..n {
                try_span(b.footprint[i], b.footprint[(i + 1) % n], i, false, &mut best_fb);
            }
            if best_fb.is_none() && n >= 4 {
                for i in 0..n {
                    try_span(b.footprint[i], b.footprint[(i + 2) % n], i, true, &mut best_fb);
                }
            }
            if let Some((_, fit)) = best_fb {
                accepted.push(fit);
            }
        }
        cands.extend(accepted);
    }

    // Rank and assign special safehouse / shelter roles, then keep EVERY eligible storefront/unit!
    let mut chosen: Vec<(FitCandidate, InteriorKind, String)> = Vec::new();
    let mut used_hero_b = std::collections::HashSet::new();

    // 1. Guarantee 1 full-sized Pharmacy near spawn (by matched sign or closest to any pharmacy sign).
    let pharm_sign_pos = city
        .signs
        .iter()
        .filter(|s| crate::survival::shop_kind(&s.text, &s.text_ru) == ShopKind::Pharmacy)
        .min_by(|a, b| a.at.dist(city.spawn).total_cmp(&b.at.dist(city.spawn)))
        .map(|s| s.at)
        .unwrap_or(city.spawn);
    if let Some(idx) = cands
        .iter()
        .enumerate()
        .filter(|(_, c)| c.hw >= 2.6 && c.depth >= 5.2 && c.shop_kind == ShopKind::Pharmacy)
        .min_by(|a, b| a.1.dist_spawn.total_cmp(&b.1.dist_spawn))
        .or_else(|| {
            cands
                .iter()
                .enumerate()
                .filter(|(_, c)| c.hw >= 2.6 && c.depth >= 5.2)
                .min_by(|a, b| a.1.origin.dist(pharm_sign_pos).total_cmp(&b.1.origin.dist(pharm_sign_pos)))
        })
        .or_else(|| {
            cands
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.origin.dist(pharm_sign_pos).total_cmp(&b.1.origin.dist(pharm_sign_pos)))
        })
        .map(|(i, _)| i)
    {
        let c = cands.remove(idx);
        used_hero_b.insert(c.b_idx);
        let nm = if c.shop_kind == ShopKind::Pharmacy && c.has_near_sign {
            c.sign_name.clone()
        } else {
            "中央大药房".into()
        };
        chosen.push((c, InteriorKind::Pharmacy, nm));
    }

    // 2. Player's Home safehouse (non-shop or residential block 40..220m from spawn, 2-room capable).
    if let Some(idx) = cands
        .iter()
        .enumerate()
        .filter(|(_, c)| !used_hero_b.contains(&c.b_idx) && c.hw >= 2.6 && c.depth >= 5.2 && c.dist_spawn >= 40.0)
        .min_by(|a, b| {
            let a_pref = !a.1.has_near_sign || a.1.is_residence;
            let b_pref = !b.1.has_near_sign || b.1.is_residence;
            (!a_pref).cmp(&(!b_pref)).then_with(|| a.1.dist_spawn.total_cmp(&b.1.dist_spawn))
        })
        .or_else(|| {
            cands
                .iter()
                .enumerate()
                .filter(|(_, c)| !used_hero_b.contains(&c.b_idx) && c.dist_spawn >= 40.0)
                .min_by(|a, b| a.1.dist_spawn.total_cmp(&b.1.dist_spawn))
        })
        .map(|(i, _)| i)
    {
        let c = cands.remove(idx);
        used_hero_b.insert(c.b_idx);
        chosen.push((c, InteriorKind::PlayerHome, "自己家·老式单元楼".into()));
    }

    // 3. Trapped Neighbours Safehouse near SOS point (-260, -230).
    if let Some(idx) = cands
        .iter()
        .enumerate()
        .filter(|(_, c)| !used_hero_b.contains(&c.b_idx) && c.hw >= 2.6 && c.depth >= 5.2)
        .min_by(|a, b| a.1.dist_sos.total_cmp(&b.1.dist_sos))
        .or_else(|| cands.iter().enumerate().min_by(|a, b| a.1.dist_sos.total_cmp(&b.1.dist_sos)))
        .map(|(i, _)| i)
    {
        let c = cands.remove(idx);
        used_hero_b.insert(c.b_idx);
        chosen.push((c, InteriorKind::NeighbourHome, "邻居家单元避难点".into()));
    }

    // 4. Basement Bunker near north promenade / Mother Square.
    if let Some(idx) = cands
        .iter()
        .enumerate()
        .filter(|(_, c)| !used_hero_b.contains(&c.b_idx) && c.hw >= 2.6 && c.depth >= 5.2)
        .min_by(|a, b| a.1.dist_north.total_cmp(&b.1.dist_north))
        .or_else(|| cands.iter().enumerate().min_by(|a, b| a.1.dist_north.total_cmp(&b.1.dist_north)))
        .map(|(i, _)| i)
    {
        let c = cands.remove(idx);
        used_hero_b.insert(c.b_idx);
        chosen.push((c, InteriorKind::BasementShelter, "北街人防地下室".into()));
    }

    // 5. ALL remaining multi-bay units across Central St & residential Heihe!
    cands.sort_by(|a, b| {
        b.has_near_sign
            .cmp(&a.has_near_sign)
            .then_with(|| b.back_u.is_some().cmp(&a.back_u.is_some()))
            .then_with(|| a.dist_spawn.total_cmp(&b.dist_spawn))
    });
    for c in cands {
        let (kind, nm) = if c.has_near_sign {
            (InteriorKind::from_shop(c.shop_kind), c.sign_name.clone())
        } else if c.is_residence {
            let nm = if !c.b_name.is_empty() { c.b_name.clone() } else { "居民单元楼".into() };
            (InteriorKind::Residential, nm)
        } else {
            let nm = if !c.b_name.is_empty() { c.b_name.clone() } else { "临街商铺".into() };
            (InteriorKind::Shop, nm)
        };
        chosen.push((c, kind, nm));
    }

    let mut interiors = Vec::with_capacity(chosen.len());
    let mut loot_id = 1u32;
    let mut npc_id = 1u32;
    for (id, (c, kind, name)) in chosen.into_iter().enumerate() {
        interiors.push(assemble_interior(id, c, kind, name, &mut loot_id, &mut npc_id));
    }
    interiors
}

impl FitCandidate {
    fn to_center(&self) -> P2 {
        self.origin.add(self.v.mul(self.depth * 0.5))
    }
}

fn assemble_interior(
    id: usize,
    c: FitCandidate,
    kind: InteriorKind,
    name: String,
    loot_id: &mut u32,
    npc_id: &mut u32,
) -> Interior {
    let hw = c.hw;
    let dep = c.depth;
    let two_room = dep >= 5.2 && hw >= 2.5;
    let split_v = if two_room { dep * 0.56 } else { dep };
    let door_hw = (hw - 0.16).clamp(0.48, 1.05);
    let to_w = |lu: f32, lv: f32| c.origin.add(c.u.mul(lu)).add(c.v.mul(lv));

    let front_style = match kind {
        InteriorKind::PlayerHome | InteriorKind::BasementShelter => DoorStyle::SteelSecurity,
        InteriorKind::NeighbourHome | InteriorKind::Residential | InteriorKind::Hotel => DoorStyle::WoodFlat,
        InteriorKind::Drink | InteriorKind::Clothes | InteriorKind::Pharmacy if id % 3 == 0 => DoorStyle::ShopGlass,
        _ => DoorStyle::CottonCurtain,
    };

    let out_step = if hw < 1.35 || dep < 2.6 { 0.72 } else { 1.05 };
    let in_step = (dep * 0.42).clamp(0.48, 0.95);

    let front_door = RoomDoor {
        at: c.origin,
        outside: to_w(0.0, -out_step),
        inside: to_w(0.0, in_step),
        u_center: 0.0,
        v_center: 0.0,
        dir: c.u,
        out: c.u.right(),
        width: door_hw * 2.0,
        is_back: false,
        is_security: front_style == DoorStyle::SteelSecurity,
        style: front_style,
        open: false,
        hp: if front_style == DoorStyle::SteelSecurity { 260.0 } else { 110.0 },
        destroyed: false,
        barricade_hp: 0.0,
        max_barricade_hp: if front_style == DoorStyle::SteelSecurity { 120.0 } else { 65.0 },
        has_curtain: front_style == DoorStyle::CottonCurtain,
    };

    let back_door = c.back_u.map(|bu| RoomDoor {
        at: to_w(bu, dep),
        outside: to_w(bu, dep + 1.1),
        inside: to_w(bu, dep - 0.9),
        u_center: bu,
        v_center: dep,
        dir: c.u,
        out: c.v,
        width: (door_hw * 1.8).min(hw * 1.4),
        is_back: true,
        is_security: false,
        style: DoorStyle::WoodFlat,
        open: false,
        hp: 110.0,
        destroyed: false,
        barricade_hp: 0.0,
        max_barricade_hp: 65.0,
        has_curtain: false,
    });

    let inner_door_u = if two_room {
        Some(if hw > 3.6 { 0.85 } else { 0.45 })
    } else {
        None
    };
    let inner_door = inner_door_u.map(|iu| RoomDoor {
        at: to_w(iu, split_v),
        outside: to_w(iu, split_v - 0.85),
        inside: to_w(iu, split_v + 0.85),
        u_center: iu,
        v_center: split_v,
        dir: c.u,
        out: c.v,
        width: 1.40,
        is_back: false,
        is_security: false,
        style: DoorStyle::WoodFlat,
        open: true,
        hp: 90.0,
        destroyed: false,
        barricade_hp: 0.0,
        max_barricade_hp: 50.0,
        has_curtain: false,
    });

    let num_floors = if matches!(kind, InteriorKind::PlayerHome | InteriorKind::NeighbourHome) {
        c.b_levels.clamp(3, 5)
    } else if kind == InteriorKind::BasementShelter {
        1
    } else {
        c.b_levels.clamp(1, 5)
    };

    // Stairwell tucked cleanly into back corner away from front/back doorways.
    let stairwell_v = if two_room {
        (dep - 1.10).max(split_v + 0.85)
    } else {
        (dep * 0.65).clamp(0.65, (dep - 0.38).max(0.65))
    };
    let stairwell_u = if hw < 1.30 {
        0.0
    } else if back_door.as_ref().is_some_and(|b| b.u_center > 0.1) || two_room {
        (-hw + 0.85).clamp(-hw + 0.40, -0.40)
    } else {
        (hw - 0.85).clamp(0.40, hw - 0.40)
    };

    // Street window spans left & right of front door (only when wall wings are wide enough).
    let mut windows = Vec::new();
    for (s0, s1) in [(-hw + 0.35, -door_hw - 0.14), (door_hw + 0.14, hw - 0.35)] {
        if s1 - s0 >= 1.0 {
            windows.push(RoomWindow {
                at: to_w((s0 + s1) * 0.5, 0.0),
                u_a: s0,
                u_b: s1,
                a: to_w(s0, 0.0),
                b: to_w(s1, 0.0),
                out: c.u.right(),
                broken: false,
                boarded: false,
            });
        }
    }

    // Perimeter walk & sight walls.
    let mut walk_walls = Vec::new();
    let mut sight_walls = Vec::new();

    for sx in [-hw, hw] {
        let w = LocalBox { cx: sx, cy: dep * 0.5, hw: 0.16, hd: dep * 0.5 + 0.16, h: 2.85 };
        walk_walls.push(w);
        sight_walls.push(w);
    }

    for (s0, s1) in [(-hw - 0.16, -door_hw), (door_hw, hw + 0.16)] {
        if s1 - s0 > 0.05 {
            walk_walls.push(LocalBox {
                cx: (s0 + s1) * 0.5,
                cy: 0.0,
                hw: (s1 - s0) * 0.5,
                hd: 0.16,
                h: 2.85,
            });
        }
    }

    let mut front_solid_u = vec![-hw - 0.16];
    for win in &windows {
        front_solid_u.push(win.u_a);
        front_solid_u.push(win.u_b);
    }
    front_solid_u.push(hw + 0.16);
    front_solid_u.sort_by(|a, b| a.total_cmp(b));
    for idx in (0..front_solid_u.len()).step_by(2) {
        if idx + 1 >= front_solid_u.len() {
            break;
        }
        let a = front_solid_u[idx];
        let b = front_solid_u[idx + 1];
        for (sa, sb) in [(-door_hw, door_hw)] {
            if a < sa && b > sb {
                if sa - a > 0.05 {
                    sight_walls.push(LocalBox { cx: (a + sa) * 0.5, cy: 0.0, hw: (sa - a) * 0.5, hd: 0.16, h: 2.85 });
                }
                if b - sb > 0.05 {
                    sight_walls.push(LocalBox { cx: (sb + b) * 0.5, cy: 0.0, hw: (b - sb) * 0.5, hd: 0.16, h: 2.85 });
                }
            } else if a < b - 0.05 && !(a >= sa - 0.02 && b <= sb + 0.02) {
                sight_walls.push(LocalBox { cx: (a + b) * 0.5, cy: 0.0, hw: (b - a) * 0.5, hd: 0.16, h: 2.85 });
            }
        }
    }

    if let Some(bd) = &back_door {
        let bu = bd.u_center;
        let b_hw = bd.width * 0.5;
        for (s0, s1) in [(-hw - 0.16, bu - b_hw), (bu + b_hw, hw + 0.16)] {
            if s1 - s0 > 0.05 {
                let w = LocalBox { cx: (s0 + s1) * 0.5, cy: dep, hw: (s1 - s0) * 0.5, hd: 0.16, h: 2.85 };
                walk_walls.push(w);
                sight_walls.push(w);
            }
        }
    } else {
        let w = LocalBox { cx: 0.0, cy: dep, hw: hw + 0.16, hd: 0.16, h: 2.85 };
        walk_walls.push(w);
        sight_walls.push(w);
    }

    if let Some(iu) = inner_door_u {
        for (s0, s1) in [(-hw, iu - 0.72), (iu + 0.72, hw)] {
            if s1 - s0 > 0.15 {
                let w = LocalBox { cx: (s0 + s1) * 0.5, cy: split_v, hw: (s1 - s0) * 0.5, hd: 0.12, h: 2.35 };
                walk_walls.push(w);
                sight_walls.push(w);
            }
        }
    }

    // Furniture colliders strictly outside doorway corridor & stairwell landing.
    let mut colliders = Vec::new();
    let corridor_edge = door_hw + 0.36;
    let push_safe = |colliders: &mut Vec<LocalBox>, b: LocalBox| {
        if ((b.cx - stairwell_u).abs() > 1.15 || (b.cy - stairwell_v).abs() > 1.15) || num_floors <= 1 {
            colliders.push(b);
        }
    };
    if two_room {
        let front_cy = (split_v * 0.58).clamp(1.75, split_v - 0.75);
        let front_hd = 0.72f32.min((split_v - 1.95) * 0.35).max(0.28);
        if (-corridor_edge) - (-hw + 0.24) >= 0.48 {
            let strip_w = (-corridor_edge) - (-hw + 0.24);
            let f_hw = (strip_w * 0.42).min(0.45);
            push_safe(&mut colliders, LocalBox { cx: -hw + 0.24 + strip_w * 0.45, cy: front_cy, hw: f_hw, hd: front_hd, h: 0.95 });
        }
        if (hw - 0.24) - corridor_edge >= 0.48 {
            let strip_w = (hw - 0.24) - corridor_edge;
            let f_hw = (strip_w * 0.42).min(0.42);
            push_safe(&mut colliders, LocalBox { cx: hw - 0.24 - strip_w * 0.45, cy: front_cy, hw: f_hw, hd: front_hd, h: 1.60 });
        }
        let back_cy = (split_v + dep) * 0.54;
        let candidate_back_x = if stairwell_u < 0.0 { hw - 0.78 } else { -hw + 0.78 };
        if candidate_back_x.abs() > 0.65 && (dep - split_v) >= 2.2 {
            push_safe(&mut colliders, LocalBox { cx: candidate_back_x, cy: back_cy, hw: 0.42_f32.min(hw * 0.22), hd: 0.40, h: 0.85 });
        }
    } else if dep >= 3.2 {
        let cy = (dep - 0.68).max(1.85);
        if (-corridor_edge) - (-hw + 0.20) >= 0.42 {
            let strip_w = (-corridor_edge) - (-hw + 0.20);
            push_safe(&mut colliders, LocalBox { cx: -hw + 0.20 + strip_w * 0.45, cy, hw: (strip_w * 0.40).min(0.36), hd: 0.30, h: 0.90 });
        }
        if hw >= 2.2 && (hw - 0.20) - corridor_edge >= 0.42 && back_door.as_ref().is_none_or(|b| b.u_center < 0.3) {
            let strip_w = (hw - 0.20) - corridor_edge;
            push_safe(&mut colliders, LocalBox { cx: hw - 0.20 - strip_w * 0.45, cy, hw: (strip_w * 0.40).min(0.34), hd: 0.30, h: 1.15 });
        }
    }

    let right_u = (hw - 0.85).max(1.25);
    let left_u = (-hw + 0.85).min(-1.25);

    let stove_at = matches!(
        kind,
        InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::BasementShelter | InteriorKind::Hardware
    )
    .then(|| to_w(right_u.min(hw - 0.65), (split_v + dep) * 0.5));
    let locker_at = matches!(
        kind,
        InteriorKind::PlayerHome | InteriorKind::NeighbourHome | InteriorKind::BasementShelter
    )
    .then(|| to_w((-hw + 0.65).clamp(-hw + 0.40, -0.25), (split_v + 0.65).min(dep - 0.55)));

    let mut firing_spots = if hw >= 2.6 {
        vec![to_w(left_u * 0.62, 1.15), to_w(right_u * 0.62, 1.15)]
    } else {
        vec![to_w(0.0, 1.25)]
    };
    if two_room && hw > 4.2 {
        firing_spots.push(to_w(0.0, split_v - 1.0));
    }

    // Ground loots (`intr.loots`).
    let mut loots = Vec::new();
    let mut push_loot = |dst: &mut Vec<RoomLoot>, at: P2, label: &'static str, lk: ShopKind, back: bool| {
        let lid = *loot_id;
        *loot_id += 1;
        dst.push(RoomLoot { id: lid, at, label, kind: lk, in_backroom: back, searched: false });
    };
    match kind {
        InteriorKind::PlayerHome => {
            push_loot(&mut loots, to_w(-hw + 1.2, dep - 1.1), "衣柜棉衣与应急盒", ShopKind::Clothes, true);
        }
        InteriorKind::NeighbourHome => {
            push_loot(&mut loots, to_w(right_u, dep - 1.1), "邻居客厅应急包", ShopKind::Pharmacy, true);
        }
        InteriorKind::BasementShelter => {
            push_loot(&mut loots, to_w(-hw + 1.2, dep - 1.1), "人防应急物资箱", ShopKind::Hardware, true);
            push_loot(&mut loots, to_w(right_u, split_v * 0.65), "储备饮水与压缩饼干", ShopKind::Food, false);
        }
        InteriorKind::Residential => {
            if id % 3 == 0 {
                let ly = if two_room { dep - 1.0 } else { (dep - 0.85).max(1.65) };
                push_loot(&mut loots, to_w(0.0, ly), "楼道住户杂物柜", ShopKind::Shop, two_room);
            }
        }
        _ => {
            let sk = kind.to_shop_kind();
            if two_room {
                push_loot(&mut loots, to_w(left_u * 0.65, split_v * 0.52), "前厅柜台/货架", sk, false);
                push_loot(&mut loots, to_w((-hw + 1.05).min(0.0), dep - 1.05), "后屋库房货箱", sk, true);
            } else {
                let ly = (dep * 0.62).clamp(0.75, (dep - 0.42).max(0.75));
                push_loot(&mut loots, to_w(0.0, ly), "店内柜台货架", sk, false);
            }
        }
    }

    // Multi-story FloorStates (ground `0` + upper `1..num_floors`, with upper-floor loot & 100/180 HP slabs).
    let mut floors = Vec::with_capacity(num_floors as usize);
    for f in 0..num_floors {
        let mut f_loots = Vec::new();
        if f >= 1 && (f % 2 == 1 || f + 1 == num_floors || matches!(kind, InteriorKind::PlayerHome | InteriorKind::NeighbourHome)) {
            let (lbl, lk) = match (id + f as usize) % 4 {
                0 => ("楼上衣柜过冬衣物", ShopKind::Clothes),
                1 => ("楼上客厅应急食品", ShopKind::Food),
                2 => ("高楼储物间废铁工具", ShopKind::Hardware),
                _ => ("楼上家用小药箱", ShopKind::Pharmacy),
            };
            let fy = (dep * 0.48).clamp(0.70, (dep - 0.42).max(0.70));
            push_loot(&mut f_loots, to_w(0.0, fy), lbl, lk, false);
        }
        let slab_hp = if f == 0 { 180.0 } else { 100.0 };
        let wall_hp = if matches!(kind, InteriorKind::PlayerHome | InteriorKind::BasementShelter) {
            240.0
        } else {
            135.0
        };
        floors.push(FloorState::intact_default(slab_hp, wall_hp, f_loots));
    }

    let mut npcs = Vec::new();
    let mut push_npc = |npcs: &mut Vec<RoomNpc>, at: P2, label: &'static str, calm: &'static str, war: &'static str, coat: Rgb| {
        let id = *npc_id;
        *npc_id += 1;
        npcs.push(RoomNpc {
            id,
            at,
            facing: 0.0,
            label,
            line_calm: calm,
            line_war: war,
            coat,
        });
    };
    match kind {
        InteriorKind::Pharmacy => push_npc(
            &mut npcs,
            to_w((left_u + 0.65).min(-0.8), (split_v * 0.50).max(1.9)),
            "药店老板",
            "柜台里的绷带纱布拿去用吧，外头真乱起来了。",
            "快进里屋躲躲，门后有门栓（按 T 顶门）！",
            Rgb { r: 0.86, g: 0.88, b: 0.90 },
        ),
        InteriorKind::NeighbourHome => push_npc(
            &mut npcs,
            to_w(0.0, split_v + 1.1),
            "被困邻居王婶",
            "小伙子你来了！咱家屋里有半袋煤，先凑合暖和。",
            "听见炮响可别往外跑，20点之后带我们一起突出这一片！",
            Rgb { r: 0.46, g: 0.25, b: 0.24 },
        ),
        InteriorKind::BasementShelter => push_npc(
            &mut npcs,
            to_w(-0.8, split_v + 1.0),
            "社区防空值守",
            "地下室厚实的，无人机和迫击炮打不进来。",
            "玻璃要是碎了冷风倒灌，按 T 用废铁把铁门焊死！",
            Rgb { r: 0.24, g: 0.30, b: 0.38 },
        ),
        InteriorKind::PlayerHome => {}
        InteriorKind::Residential => {
            if id % 7 == 0 {
                push_npc(
                    &mut npcs,
                    to_w(0.0, (dep * 0.55).max(1.85)),
                    "楼道邻居",
                    "外头天黑了，快进门洞里避避风，楼梯间可以上楼（按 Space）。",
                    "坦克上街了就上楼扔燃烧瓶/用火箭筒（F）攻顶！别站临街低楼层窗边！",
                    Rgb { r: 0.24, g: 0.26, b: 0.30 },
                );
            }
        }
        _ => {
            if id % 2 == 1 {
                let ny = if two_room { split_v * 0.55 } else { (dep * 0.60).max(1.85) };
                push_npc(
                    &mut npcs,
                    to_w((right_u - 0.55).max(0.75).min(hw - 0.55), ny),
                    "店主",
                    "冷了吧？推门进屋暖和会儿，架上吃的还能拿。",
                    "俄军顺着中央街搜过来了，快从后屋或楼梯撤离！",
                    Rgb { r: 0.26, g: 0.30, b: 0.34 },
                );
            }
        }
    }

    let stash = match kind {
        InteriorKind::PlayerHome => HomeStash { food: 2, water: 2, bandage: 1, warmer: 2, ammo: 0, scrap: 2, coal: 3 },
        InteriorKind::BasementShelter => HomeStash { food: 2, water: 3, bandage: 2, warmer: 1, ammo: 0, scrap: 2, coal: 3 },
        _ => HomeStash::default(),
    };

    Interior {
        id,
        building_id: c.building_id,
        name,
        kind,
        origin: c.origin,
        u: c.u,
        v: c.v,
        half_w: hw,
        depth: dep,
        split_v,
        wall_h: 2.85,
        num_floors,
        stairwell_u,
        stairwell_v,
        front_door,
        back_door,
        inner_door,
        windows,
        walk_walls,
        sight_walls,
        colliders,
        firing_spots,
        stove_at,
        stove_fuel_s: 0.0,
        locker_at,
        stash,
        loots,
        floors,
        npcs,
        room_temp_c: match kind {
            InteriorKind::BasementShelter => 4.0,
            _ => 17.5,
        },
        curtains_closed: matches!(kind, InteriorKind::PlayerHome | InteriorKind::NeighbourHome),
        discovered: kind == InteriorKind::PlayerHome,
        breach_alert: 0.0,
        revision: 0,
    }
}
