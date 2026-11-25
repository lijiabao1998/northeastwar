//! Buildings, drawn from their footprints. Each kind in `look::Kind` is one
//! of the kinds of building the photographs of Heihe show, and is built here
//! out of flat pieces: wall, bands, pilasters, frames, glass, a roof under
//! snow, and on Central Street a cone or a dome on a corner.

use crate::city::Building;
use crate::geom::{hash01, P2};
use crate::look::{Dress, Kind, Palette, Rgb};
use crate::soup::{world, world_dir, Soup};

/// What one building turns into.
#[derive(Default)]
pub struct Built {
    /// Walls, bands, frames, roof, snow: lit by the sun and the lamps.
    pub solid: Soup,
    /// Windows nobody is behind.
    pub glass: Soup,
    /// Windows with a light on. They glow after dark.
    pub lit: Soup,
    /// The corner crown's triangles in the world, exactly as drawn (they are in `solid`
    /// too). It stands above the roof, so the cut-away tests sight lines against them.
    pub crown: Vec<[[f32; 3]; 3]>,
}

/// Width and height of a crown's drum, by kind; None for kinds that carry no crown.
pub fn crown_drum(kind: Kind) -> Option<(f32, f32)> {
    match kind {
        Kind::CentralBaroque => Some((2.3 * 1.7, 1.6)),
        Kind::CentralSalmon => Some((2.3 * 1.6, 2.4)),
        _ => None,
    }
}

const GROUND_SHOP: f32 = 3.9;
const BAY: f32 = 3.3;

struct Edge<'a> {
    a: P2,
    dir: P2,
    out: P2,
    len: f32,
    soup: &'a mut Soup,
}

impl Edge<'_> {
    fn at(&self, along: f32, up: f32, proud: f32) -> [f32; 3] {
        world(self.a.add(self.dir.mul(along)).add(self.out.mul(proud)), up)
    }

    /// A flat piece on this wall: from `s0` to `s1` along it, `h0` to `h1` up it,
    /// standing `proud` metres off the wall so nothing fights for the same depth.
    fn piece(&mut self, s0: f32, s1: f32, h0: f32, h1: f32, proud: f32, colour: Rgb) {
        if s1 - s0 < 0.02 || h1 - h0 < 0.02 {
            return;
        }
        let corners = [self.at(s0, h0, proud), self.at(s1, h0, proud), self.at(s1, h1, proud), self.at(s0, h1, proud)];
        let normal = world_dir(self.out);
        self.soup.quad(corners, normal, colour);
    }
}

fn piece_on(soup: &mut Soup, a: P2, dir: P2, out: P2, s0: f32, s1: f32, h0: f32, h1: f32, proud: f32, colour: Rgb) {
    let at = |along: f32, up: f32| world(a.add(dir.mul(along)).add(out.mul(proud)), up);
    soup.quad([at(s0, h0), at(s1, h0), at(s1, h1), at(s0, h1)], world_dir(out), colour);
}

fn box_on(soup: &mut Soup, a: P2, dir: P2, out: P2, s0: f32, s1: f32, h0: f32, h1: f32, p0: f32, p1: f32, colour: Rgb) {
    if s1 - s0 < 0.02 || h1 - h0 < 0.02 || p1 - p0 < 0.01 {
        return;
    }
    let centre = a.add(dir.mul((s0 + s1) * 0.5)).add(out.mul((p0 + p1) * 0.5));
    let yaw = dir.y.atan2(dir.x);
    soup.boxed(centre, h0, [s1 - s0, h1 - h0, p1 - p0], yaw, colour);
}

fn ac_unit(soup: &mut Soup, pal: &Palette, a: P2, dir: P2, out: P2, cx: f32, h0: f32, p0: f32) {
    box_on(soup, a, dir, out, cx - 0.38, cx + 0.38, h0, h0 + 0.48, p0, p0 + 0.28, pal.get("ac_white"));
    piece_on(soup, a, dir, out, cx - 0.26, cx + 0.08, h0 + 0.07, h0 + 0.41, p0 + 0.29, pal.get("shopfront"));
}

/// Bays along a wall of this length: (count, first centre, spacing).
fn bays(len: f32) -> (usize, f32, f32) {
    let usable = len - 1.2;
    if usable < BAY * 0.8 {
        return (0, 0.0, 0.0);
    }
    let n = (usable / BAY).floor().max(1.0) as usize;
    let step = usable / n as f32;
    (n, 0.6 + step * 0.5, step)
}

pub fn build(b: &Building, dress: Dress, pal: &Palette) -> Built {
    let mut out = Built::default();
    let ring = &b.footprint;
    let n = ring.len();
    let parapet = if dress.kind == Kind::FarBank { 0.0 } else { crate::cutaway::PARAPET };
    let top = b.height;
    let ground = if dress.shops { GROUND_SHOP.min(top) } else { 0.0 };
    let floors_above = (((top - ground) / b.floor_h).round() as i32).max(if ground > 0.0 { 0 } else { 1 }) as usize;
    let floor_h = if floors_above > 0 { (top - ground) / floors_above as f32 } else { b.floor_h };

    let (wall, trim, band) = match dress.kind {
        Kind::CentralSalmon => (pal.get("central_salmon"), pal.get("central_pilaster"), pal.get("central_cream")),
        Kind::CentralBaroque => (pal.get("baroque_red"), pal.get("baroque_white"), pal.get("baroque_white")),
        Kind::CentralClassical => (pal.get("classical_grey"), pal.get("baroque_white"), pal.get("tile_white")),
        Kind::YellowCrimson => (pal.get("resi_yellow"), pal.get("resi_cream"), pal.get("resi_crimson")),
        Kind::TanTile => (pal.get("tile_tan"), pal.get("tile_cream"), pal.get("tile_cream")),
        Kind::SalmonYellow => (pal.get("salmon_block"), pal.get("tile_cream"), pal.get("bay_yellow")),
        Kind::LowBrick => (pal.get("tile_white"), pal.get("tile_white"), pal.get("brick_red")),
        Kind::FarBank => {
            let brick = hash01(b.id, 3) < 0.55;
            let c = if brick { pal.get("far_bank_brick") } else { pal.get("far_bank_white") };
            (c, c, c)
        }
    };
    let frame = pal.get("window_frame");
    let glass = pal.get("glass_day");

    for i in 0..n {
        let a = ring[i];
        let c = ring[(i + 1) % n];
        let len = a.dist(c);
        if len < 0.3 {
            continue;
        }
        let dir = c.sub(a).norm();
        let outw = dir.right();
        {
            let mut e = Edge { a, dir, out: outw, len, soup: &mut out.solid };
            // The wall itself, up past the roof as a parapet.
            e.piece(0.0, len, 0.0, top + parapet, 0.0, wall);
            if dress.kind == Kind::FarBank {
                continue;
            }
            if dress.kind == Kind::LowBrick {
                e.piece(0.0, len, 0.0, (top * 0.45).min(3.2), 0.02, band);
            }
            // Cornice.
            if matches!(dress.kind, Kind::CentralSalmon | Kind::CentralBaroque | Kind::CentralClassical) {
                e.piece(0.0, len, top - 0.35, top + parapet, 0.18, trim);
                e.piece(0.0, len, ground - 0.25, ground + 0.15, 0.14, trim);
            }
            if dress.kind == Kind::TanTile {
                for f in 0..=floors_above {
                    let h = ground + f as f32 * floor_h;
                    e.piece(0.0, len, h - 0.18, h + 0.12, 0.03, trim);
                }
            }
            let _ = e.len;
        }

        let (count, first, step) = bays(len);
        // Ground floor: shops.
        if ground > 0.0 && count > 0 {
            for k in 0..count {
                let cx = first + k as f32 * step;
                let half = step * 0.5 - 0.25;
                let bay_key = (i * 37 + k) as u32;
                let shut = dress.kind == Kind::LowBrick && hash01(b.id, 100 + bay_key) < 0.35;
                if shut {
                    piece_on(&mut out.solid, a, dir, outw, cx - half, cx + half, 0.15, 2.9, 0.05, pal.get("shutter"));
                    for slat in 0..7 {
                        let sh = 0.35 + slat as f32 * 0.36;
                        piece_on(&mut out.solid, a, dir, outw, cx - half + 0.04, cx + half - 0.04, sh, sh + 0.05, 0.06, pal.get("granite_dark"));
                    }
                } else {
                    piece_on(&mut out.solid, a, dir, outw, cx - half, cx + half, 0.0, 3.0, 0.04, pal.get("shopfront"));
                    let lit = hash01(b.id, 200 + bay_key) < 0.7;
                    let target = if lit { &mut out.lit } else { &mut out.glass };
                    piece_on(target, a, dir, outw, cx - half + 0.15, cx + half - 0.15, 0.45, 2.62, 0.07, glass);

                    if k % 2 == 0 || count == 1 {
                        let door_w = (half * 0.48).clamp(0.42, 0.62);
                        let curtain = if hash01(b.id, 300 + bay_key) < 0.55 {
                            pal.get("curtain_green")
                        } else {
                            pal.get("curtain_maroon")
                        };
                        piece_on(&mut out.solid, a, dir, outw, cx - door_w, cx + door_w, 0.22, 2.35, 0.08, curtain);
                        let stitch = curtain.mix(pal.get("shopfront"), 0.35);
                        piece_on(&mut out.solid, a, dir, outw, cx - door_w * 0.38 - 0.02, cx - door_w * 0.38 + 0.02, 0.24, 2.33, 0.095, stitch);
                        piece_on(&mut out.solid, a, dir, outw, cx + door_w * 0.38 - 0.02, cx + door_w * 0.38 + 0.02, 0.24, 2.33, 0.095, stitch);
                        piece_on(&mut out.glass, a, dir, outw, cx - 0.16, cx + 0.16, 1.45, 1.85, 0.10, glass);
                        let couplet = pal.get("baroque_red");
                        piece_on(&mut out.solid, a, dir, outw, cx - door_w - 0.11, cx - door_w - 0.01, 0.35, 2.32, 0.085, couplet);
                        piece_on(&mut out.solid, a, dir, outw, cx + door_w + 0.01, cx + door_w + 0.11, 0.35, 2.32, 0.085, couplet);
                        piece_on(&mut out.solid, a, dir, outw, cx - door_w - 0.11, cx + door_w + 0.11, 2.34, 2.52, 0.085, couplet);

                        let s0 = cx - door_w - 0.25;
                        let s1 = cx + door_w + 0.25;
                        box_on(&mut out.solid, a, dir, outw, s0, s1, 0.0, 0.14, 0.0, 0.52, pal.get("granite_paving"));
                        box_on(&mut out.solid, a, dir, outw, s0 + 0.06, s1 - 0.06, 0.14, 0.26, 0.0, 0.28, pal.get("granite_dark"));
                        if matches!(dress.kind, Kind::TanTile | Kind::LowBrick) {
                            let iron = pal.get("rail_iron");
                            for rs in [s0 + 0.06, s1 - 0.06] {
                                let p_bot = world(a.add(dir.mul(rs)).add(outw.mul(0.48)), 0.14);
                                let p_top = world(a.add(dir.mul(rs)).add(outw.mul(0.06)), 0.26);
                                let r_bot = world(a.add(dir.mul(rs)).add(outw.mul(0.48)), 0.82);
                                let r_top = world(a.add(dir.mul(rs)).add(outw.mul(0.06)), 1.02);
                                out.solid.beam(p_bot, r_bot, 0.045, iron);
                                out.solid.beam(p_top, r_top, 0.045, iron);
                                out.solid.beam(r_bot, r_top, 0.045, iron);
                            }
                        }
                    }
                    if matches!(dress.kind, Kind::TanTile | Kind::LowBrick) {
                        let canopy = if dress.kind == Kind::TanTile { pal.get("awning_teal") } else { pal.get("baroque_red") };
                        box_on(&mut out.solid, a, dir, outw, cx - half + 0.05, cx + half - 0.05, 2.62, 2.78, 0.0, 0.34, canopy);
                        box_on(&mut out.solid, a, dir, outw, cx - half + 0.07, cx + half - 0.07, 2.78, 2.82, 0.02, 0.32, pal.get("snow"));
                    }
                    if hash01(b.id, 400 + bay_key) < 0.42 {
                        ac_unit(&mut out.solid, pal, a, dir, outw, cx + half - 0.42, 2.12, 0.0);
                    }
                }
            }
        }
        // Upper floors.
        for f in 0..floors_above {
            let base = ground + f as f32 * floor_h;
            let sill = base + 0.95;
            let head = base + floor_h - 0.55;
            match dress.kind {
                Kind::YellowCrimson => {
                    // The crimson apron under every row of windows: the thing you remember.
                    piece_on(&mut out.solid, a, dir, outw, 0.0, len, base + 0.05, sill - 0.05, 0.06, band);
                }
                Kind::CentralSalmon => {
                    piece_on(&mut out.solid, a, dir, outw, 0.0, len, base + 0.1, sill - 0.1, 0.03, band);
                }
                _ => {}
            }
            for k in 0..count {
                let cx = first + k as f32 * step;
                let key = (f * 1009 + i * 37 + k) as u32;
                let stair = k % 4 == 3 && matches!(dress.kind, Kind::YellowCrimson | Kind::TanTile | Kind::SalmonYellow);
                let mut w = match dress.kind {
                    Kind::TanTile => step * 0.36,
                    Kind::YellowCrimson => step * 0.34,
                    Kind::CentralClassical => step * 0.3,
                    _ => step * 0.27,
                };
                if stair {
                    w = 0.5;
                }
                let (mut s, mut h) = (sill, head);
                if stair {
                    s = base + 1.3;
                    h = base + floor_h - 0.3;
                }
                if dress.kind == Kind::SalmonYellow && k % 3 == 1 {
                    // The yellow bay: a column of bay windows running up the block.
                    piece_on(&mut out.solid, a, dir, outw, cx - step * 0.42, cx + step * 0.42, base, base + floor_h, 0.1, band);
                    w = step * 0.36;
                }
                if dress.kind == Kind::LowBrick && f == 0 && ground == 0.0 {
                    s = base + 1.0;
                }
                piece_on(&mut out.solid, a, dir, outw, cx - w - 0.09, cx + w + 0.09, s - 0.09, h + 0.09, 0.12, frame);
                if dress.kind == Kind::CentralBaroque || dress.kind == Kind::CentralClassical {
                    // An arched head, as three short pieces.
                    piece_on(&mut out.solid, a, dir, outw, cx - w * 0.8, cx + w * 0.8, h + 0.09, h + 0.34, 0.12, frame);
                    piece_on(&mut out.solid, a, dir, outw, cx - w * 0.45, cx + w * 0.45, h + 0.34, h + 0.5, 0.12, frame);
                }
                let lit = hash01(b.id, 5000 + key) < 0.42;
                let target = if lit { &mut out.lit } else { &mut out.glass };
                piece_on(target, a, dir, outw, cx - w, cx + w, s, h, 0.15, glass);
                // Glazing bars and snow-capped sill.
                piece_on(&mut out.solid, a, dir, outw, cx - 0.035, cx + 0.035, s, h, 0.17, frame);
                let trans_h = s + (h - s) * 0.65;
                piece_on(&mut out.solid, a, dir, outw, cx - w, cx + w, trans_h - 0.03, trans_h + 0.03, 0.17, frame);
                piece_on(&mut out.solid, a, dir, outw, cx - w - 0.10, cx + w + 0.10, s - 0.14, s - 0.05, 0.16, pal.get("snow"));
                if !stair && s - base >= 0.72 && w >= 0.6 && hash01(b.id, 6000 + key) < 0.09 {
                    ac_unit(&mut out.solid, pal, a, dir, outw, cx + w * 0.45, s - 0.62, 0.10);
                }
            }
            // Pilasters between bays on Central Street.
            if matches!(dress.kind, Kind::CentralSalmon | Kind::CentralBaroque) && count > 1 {
                for k in 0..=count {
                    let px = first - step * 0.5 + k as f32 * step;
                    piece_on(&mut out.solid, a, dir, outw, px - 0.22, px + 0.22, base, base + floor_h, 0.1, trim);
                }
            }
        }
        // Gables standing on the cornice of the long salmon blocks.
        if dress.kind == Kind::CentralSalmon && count >= 4 {
            let mut k = 2;
            while k < count {
                let cx = first + k as f32 * step;
                let p = |along: f32, up: f32| world(a.add(dir.mul(along)).add(outw.mul(0.16)), up);
                out.solid.tri(
                    [p(cx - step * 0.9, top + parapet), p(cx + step * 0.9, top + parapet), p(cx, top + parapet + 1.7)],
                    world_dir(outw),
                    trim,
                );
                k += 5;
            }
        }
    }

    // Roof, under snow, and the inner face of the parapet.
    if dress.kind == Kind::FarBank {
        out.solid.flat(ring, top, pal.get("snow"));
    } else {
        out.solid.flat(ring, top, pal.get("snow"));
        for i in 0..n {
            let a = ring[i];
            let c = ring[(i + 1) % n];
            let dir = a.sub(c).norm();
            if a.dist(c) < 0.3 {
                continue;
            }
            // Seen from inside the roof: the wall's back.
            let inw = dir.right();
            out.solid.quad(
                [world(c, top), world(a, top), world(a, top + parapet), world(c, top + parapet)],
                world_dir(inw),
                pal.get("parapet"),
            );
        }
    }

    // A crown on one corner: the red cone of the salmon blocks, the green dome of the red ones.
    // The drum is turned with the wall and the facets of the cone face the walls, so neither
    // stands out past the facade more than an eave does.
    let place = crown_drum(dress.kind).filter(|_| dress.crown).and_then(|(w, h)| crown_at(ring, w * 0.5).map(|at| (at, w, h)));
    if let Some(((centre, yaw), w, h)) = place {
        let drum = 2.3;
        let mut c = Soup::default();
        match dress.kind {
            Kind::CentralBaroque => {
                c.boxed(centre, top, [w, h, w], yaw, trim);
                c.cone_turned(centre, top + h, drum * 1.05, 2.6, 12, yaw + std::f32::consts::PI / 12.0, pal.get("dome_green"));
                c.cone_turned(centre, top + h + 2.0, 0.25, 1.6, 6, yaw + std::f32::consts::PI / 6.0, pal.get("dome_green"));
            }
            _ => {
                c.boxed(centre, top, [w, h, w], yaw, band);
                c.cone_turned(centre, top + h, drum * 1.05, 5.2, 8, yaw + std::f32::consts::PI / 8.0, pal.get("spire_red"));
            }
        }
        out.crown = c.triangle_list();
        out.solid.append(&c);
    }
    out
}

/// Where a corner crown's drum stands and how it is turned: on the corner
/// `crown_corner` picks, turned with the wall that arrives there, and moved in along
/// the corner's bisector from 2.6 m until all four corners of a drum `half` metres
/// from centre to side are inside the outline. None when nothing within 6 m fits.
pub fn crown_at(ring: &[P2], half: f32) -> Option<(P2, f32)> {
    let (corner, inward, along) = crown_corner(ring)?;
    let yaw = along.y.atan2(along.x);
    let (ax, ay) = (P2::new(yaw.cos(), yaw.sin()), P2::new(-yaw.sin(), yaw.cos()));
    let mut d = 2.6;
    while d <= 6.0 {
        let c = corner.add(inward.mul(d));
        let fits = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .iter()
            .all(|&(u, v)| crate::geom::point_in_ring(c.add(ax.mul(u * half)).add(ay.mul(v * half)), ring));
        if fits {
            return Some((c, yaw));
        }
        d += 0.05;
    }
    None
}

/// The corner a tower goes on: a real convex corner (turning at least 45
/// degrees) with long walls either side, the way into the building from it, and
/// the direction of the wall that arrives at it. None when the outline has no such corner.
pub fn crown_corner(ring: &[P2]) -> Option<(P2, P2, P2)> {
    let n = ring.len();
    if n < 3 {
        return None;
    }
    let mut best: Option<(f32, P2, P2, P2)> = None;
    for i in 0..n {
        let (prev, here, next) = (ring[(i + n - 1) % n], ring[i], ring[(i + 1) % n]);
        let (din, dout) = (here.sub(prev).norm(), next.sub(here).norm());
        // Counter-clockwise ring: a left turn is a convex corner.
        let turn = din.cross(dout).atan2(din.dot(dout)).to_degrees();
        if turn < 45.0 {
            continue;
        }
        let score = here.dist(prev).min(here.dist(next)).min(20.0);
        // The building is on the left of each wall; halfway between the two is inward.
        let inward = din.right().mul(-1.0).add(dout.right().mul(-1.0)).norm();
        // The tower stands 2.6 m in from the corner; on a narrow wing that can be outside.
        if !crate::geom::point_in_ring(here.add(inward.mul(2.6)), ring) {
            continue;
        }
        if best.map_or(true, |(s, _, _, _)| score > s) {
            best = Some((score, here, inward, din));
        }
    }
    best.map(|(_, c, d, w)| (c, d, w))
}
