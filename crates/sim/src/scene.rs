//! Everything in Heihe that is not a building: snow, the frozen river and its
//! embankment, streets and pavements, lamps, pollarded trees, parked cars,
//! the stone balls on Central Street, the Mother sculpture, and where each
//! shop board hangs.

use crate::city::{Bank, City};
use crate::facade;
use crate::geom::{hash01, nearest_on_segment, point_in_ring, P2};
use crate::look::{dress, Palette, Rgb};
use crate::soup::{world, world_dir, Soup};

pub struct Board {
    pub centre: [f32; 3],
    /// Unit vector along the board, left to right as you face it.
    pub right: [f32; 3],
    pub normal: [f32; 3],
    pub width: f32,
    pub height: f32,
    pub zh: String,
    pub ru: String,
    pub back: Rgb,
    pub ink: Rgb,
}

/// One building on the Chinese bank, kept apart so it can be cut away when it hides you.
pub struct Block {
    pub id: i64,
    pub footprint: Vec<P2>,
    pub height: f32,
    pub solid: Soup,
    pub glass: Soup,
    pub lit: Soup,
    /// Its corner crown's triangles in the world, as drawn (facade::Built::crown).
    pub crown: Vec<[[f32; 3]; 3]>,
}

#[derive(Default)]
pub struct Scene {
    pub ground: Soup,
    /// The far bank, all in one: it is across the ice and never in the way.
    pub buildings: Soup,
    pub blocks: Vec<Block>,
    pub glass: Soup,
    pub lit: Soup,
    pub props: Soup,
    /// The glowing heads of the street lamps (drawn unlit, bright after dark).
    pub lamp_glow: Soup,
    pub lamp_heads: Vec<[f32; 3]>,
    /// Each post's lit heads as one light: between them, and how many heads it stands
    /// for. Placed at one head instead, the pool under the other lost up to 40%.
    pub lamp_posts: Vec<([f32; 3], usize)>,
    pub boards: Vec<Board>,
}

fn along(line: &[P2], every: f32, mut each: impl FnMut(P2, P2)) {
    let mut carry = every * 0.5;
    for w in line.windows(2) {
        let len = w[0].dist(w[1]);
        let dir = w[1].sub(w[0]).norm();
        let mut s = carry;
        while s < len {
            each(w[0].add(dir.mul(s)), dir);
            s += every;
        }
        carry = s - len;
    }
}

fn offset(line: &[P2], by: f32) -> Vec<P2> {
    // Positive `by` is to the right of travel.
    let mut out = Vec::with_capacity(line.len());
    for i in 0..line.len() {
        let before = if i > 0 { line[i].sub(line[i - 1]).norm() } else { line[1].sub(line[0]).norm() };
        let after = if i + 1 < line.len() { line[i + 1].sub(line[i]).norm() } else { before };
        let dir = before.add(after).norm();
        out.push(line[i].add(dir.right().mul(by)));
    }
    out
}

pub fn build(city: &City, pal: &Palette) -> Scene {
    let mut scene = Scene::default();
    ground(city, pal, &mut scene.ground);

    for b in &city.buildings {
        let built = facade::build(b, dress(b, city), pal);
        if b.bank == Bank::China {
            scene.blocks.push(Block { id: b.id, footprint: b.footprint.clone(), height: b.height, solid: built.solid, glass: built.glass, lit: built.lit, crown: built.crown });
        } else {
            scene.buildings.append(&built.solid);
            scene.glass.append(&built.glass);
            scene.lit.append(&built.lit);
        }
    }

    lamps(city, pal, &mut scene);
    trees(city, pal, &mut scene.props);
    poles_and_wires(city, pal, &mut scene.props);
    cars(city, pal, &mut scene.props);
    trikes(city, pal, &mut scene.props);
    central_street(city, pal, &mut scene);
    goods_arch(city, pal, &mut scene);
    mother(city, pal, &mut scene);
    riverfront(city, pal, &mut scene);
    snow_piles(city, pal, &mut scene.props);
    scene.boards = boards(city, pal, &mut scene.props);
    scene
}

fn oct_prism(soup: &mut Soup, at: P2, h0: f32, h1: f32, r0: f32, r1: f32, colour: Rgb) {
    let slope = (r0 - r1) / (h1 - h0).max(0.01);
    for i in 0..8 {
        let a0 = i as f32 * std::f32::consts::TAU / 8.0;
        let a1 = (i + 1) as f32 * std::f32::consts::TAU / 8.0;
        let d0 = P2::new(a0.cos(), a0.sin());
        let d1 = P2::new(a1.cos(), a1.sin());
        let mid = P2::new(((a0 + a1) * 0.5).cos(), ((a0 + a1) * 0.5).sin());
        let l = (1.0 + slope * slope).sqrt();
        let n = [mid.x / l, slope / l, -mid.y / l];
        soup.quad(
            [world(at.add(d0.mul(r0)), h0), world(at.add(d1.mul(r0)), h0), world(at.add(d1.mul(r1)), h1), world(at.add(d0.mul(r1)), h1)],
            n,
            colour,
        );
    }
}

fn building_near(city: &City, p: P2, margin: f32) -> bool {
    for b in city.buildings.iter().filter(|b| b.bank == Bank::China) {
        if b.centroid.dist(p) > 85.0 {
            continue;
        }
        if point_in_ring(p, &b.footprint) {
            return true;
        }
        let n = b.footprint.len();
        for i in 0..n {
            if p.dist(nearest_on_segment(p, b.footprint[i], b.footprint[(i + 1) % n])) < margin {
                return true;
            }
        }
    }
    false
}

fn smooth_river_out(bank: &[P2]) -> P2 {
    if bank.len() < 2 {
        return P2::new(0.40, 0.92).norm();
    }
    let d = bank[bank.len() - 1].sub(bank[0]).norm();
    let n = P2::new(-d.y, d.x);
    if n.y > 0.0 { n } else { n.mul(-1.0) }
}

fn decimate_bank(bank: &[P2], step: usize) -> Vec<P2> {
    if bank.len() <= 2 {
        return bank.to_vec();
    }
    let mut out = Vec::with_capacity(bank.len() / step + 2);
    for (i, &p) in bank.iter().enumerate() {
        if i == 0 || i + 1 == bank.len() || i % step == 0 {
            out.push(p);
        }
    }
    out
}

fn ground(city: &City, pal: &Palette, g: &mut Soup) {
    let snow = pal.get("snow_ground");
    let field = [P2::new(-1500.0, -1100.0), P2::new(1300.0, -1100.0), P2::new(1300.0, 2400.0), P2::new(-1500.0, 2400.0)];
    g.flat(&field, -0.06, snow);

    // The Heilongjiang, frozen: continuous frosted snow-ice base + subtle mid-river glare + organic pressure cracks.
    if city.bank.len() >= 2 {
        let ice = pal.get("river_ice");
        let base_ice = ice.mix(snow, 0.34);
        let river_out = smooth_river_out(&city.bank);
        for w in city.bank.windows(2) {
            let far0 = w[0].add(river_out.mul(780.0));
            let far1 = w[1].add(river_out.mul(780.0));
            let mut ring = vec![w[0], w[1], far1, far0];
            if crate::geom::signed_area(&ring) < 0.0 {
                ring.reverse();
            }
            if crate::geom::signed_area(&ring) > 0.5 {
                g.flat(&ring, -0.03, base_ice);
            }
        }
        let smooth = decimate_bank(&city.bank, 12);
        // Near-shore and far-shore snow-drift aprons.
        for (idx, (dist_out, width)) in [(12.0f32, 22.0f32), (710.0, 26.0)].into_iter().enumerate() {
            let apron: Vec<P2> = smooth.iter().map(|p| p.add(river_out.mul(dist_out))).collect();
            g.ribbon(&apron, width, -0.021 + idx as f32 * 0.0005, snow.mix(base_ice, 0.32));
        }
        // Broad, gentle low-contrast mid-river wind-scoured blue-ice sheen.
        let mid_channel: Vec<P2> = smooth.iter().map(|p| p.add(river_out.mul(225.0))).collect();
        g.ribbon(&mid_channel, 145.0, -0.025, ice.mix(base_ice, 0.55));
        // Three subtle organic pressure-crack / wind-ridge hairline streaks across the ice sheet.
        let crack_col = pal.get("river_ice_dark").mix(base_ice, 0.55);
        for (ci, (base_off, amp, w)) in [(48.0f32, 14.0f32, 0.8f32), (162.0, 22.0, 1.05), (335.0, 18.0, 0.72)]
            .into_iter()
            .enumerate()
        {
            let crack: Vec<P2> = smooth
                .iter()
                .enumerate()
                .map(|(i, p)| p.add(river_out.mul(base_off + ((i as f32 * 0.85 + ci as f32 * 1.9).sin()) * amp)))
                .collect();
            g.ribbon(&crack, w, -0.018 + ci as f32 * 0.0004, crack_col);
        }
        // Four staggered short wind-drifted snow patches along the river surface.
        let patch_col = snow.mix(base_ice, 0.40);
        for (pi, (i0, len, dist_out, w)) in [
            (0usize, 5usize, 78.0f32, 7.5f32),
            (3, 6, 148.0, 9.5),
            (6, 5, 195.0, 6.5),
            (8, 6, 285.0, 10.5),
        ]
        .into_iter()
        .enumerate()
        {
            let lo = i0.min(smooth.len().saturating_sub(2));
            let hi = (lo + len).min(smooth.len());
            if hi >= lo + 2 {
                let seg: Vec<P2> = smooth[lo..hi]
                    .iter()
                    .enumerate()
                    .map(|(j, p)| p.add(river_out.mul(dist_out + (j as f32 * 1.1).cos() * 4.5)))
                    .collect();
                g.ribbon(&seg, w, -0.016 + pi as f32 * 0.0005, patch_col);
            }
        }

        // The Chinese riverside promenade + inland Heilongjiang Park stone-brick walkway + far-bank embankment.
        g.ribbon(&offset(&city.bank, 5.0), 9.0, 0.02, pal.get("granite_paving"));
        let park_walk: Vec<P2> = decimate_bank(&offset(&city.bank, 22.0), 6);
        g.ribbon(&park_walk, 4.2, 0.024, pal.get("pavement_brick").mix(snow, 0.14));
        let far_shore: Vec<P2> = smooth.iter().map(|p| p.add(river_out.mul(725.0))).collect();
        g.ribbon(&far_shore, 6.5, 0.15, pal.get("embankment"));

        // Curved winter patrol / hovercraft apron trails leading out from the winter slipway.
        let along_bank = P2::new(-river_out.y, river_out.x);
        let slipway_start = P2::new(-95.0, 82.0);
        let ice_trail = [
            slipway_start,
            slipway_start.add(river_out.mul(55.0)).add(along_bank.mul(12.0)),
            slipway_start.add(river_out.mul(160.0)).add(along_bank.mul(45.0)),
            slipway_start.add(river_out.mul(340.0)).add(along_bank.mul(70.0)),
            slipway_start.add(river_out.mul(560.0)).add(along_bank.mul(30.0)),
            slipway_start.add(river_out.mul(710.0)).add(along_bank.mul(-10.0)),
        ];
        for side in [-1.4f32, 1.4] {
            g.ribbon(&offset(&ice_trail, side), 0.75, -0.010, pal.get("river_ice_dark").mix(snow, 0.3));
        }
    }

    let brick = pal.get("pavement_brick").mix(snow, 0.18);
    let asphalt = pal.get("asphalt").mix(pal.get("asphalt_snow_film"), 0.42);
    for (i, road) in city.roads.iter().enumerate() {
        let lift = i as f32 * 0.0006;
        if road.pedestrian() {
            g.ribbon(&road.line, road.width.max(3.0), 0.03 + lift, pal.get("granite_paving"));
            if road.width >= 8.0 {
                g.ribbon(&road.line, 1.6, 0.034 + lift, pal.get("granite_dark"));
            }
            if road.name == "中央街" {
                for side in [-1.0f32, 1.0] {
                    g.ribbon(&offset(&road.line, side * (road.width * 0.5 - 1.8)), 0.32, 0.036 + lift, pal.get("tactile_yellow"));
                }
            }
            continue;
        }
        g.ribbon(&road.line, road.width + 7.0, 0.012 + lift, brick);
        g.ribbon(&road.line, road.width, 0.04 + lift, asphalt);
        if road.width >= 8.5 {
            g.ribbon(&road.line, 0.16, 0.05 + lift, pal.get("road_line_yellow"));
            for side in [-1.0f32, 1.0] {
                g.ribbon(&offset(&road.line, side * (road.width * 0.5 - 0.12)), 0.14, 0.048 + lift, pal.get("road_line_yellow"));
            }
        }
        // Wheel tracks worn through the snow film.
        for side in [-1.0f32, 1.0] {
            g.ribbon(&offset(&road.line, side * road.width * 0.24), 0.9, 0.045 + lift, pal.get("asphalt").mix(asphalt, 0.5));
        }
    }
    // Zebra crosswalks at the main downtown intersections.
    let xings = [
        P2::new(-139.5, -256.1),
        P2::new(-188.3, -353.4),
        P2::new(-97.1, -167.0),
        P2::new(-378.8, -258.1),
        P2::new(-332.0, -162.4),
        P2::new(-493.2, -203.9),
        P2::new(-446.6, -106.3),
    ];
    let stripe_col = pal.get("zebra_white").mix(pal.get("asphalt_snow_film"), 0.25);
    for ix in xings {
        for road in city.roads.iter().filter(|r| !r.pedestrian() && r.width >= 6.0) {
            for k in 0..road.line.len() {
                if road.line[k].dist(ix) > 2.0 {
                    continue;
                }
                let mut dirs = Vec::new();
                if k > 0 && road.line[k - 1].dist(road.line[k]) > 8.0 {
                    dirs.push(road.line[k - 1].sub(road.line[k]).norm());
                }
                if k + 1 < road.line.len() && road.line[k + 1].dist(road.line[k]) > 8.0 {
                    dirs.push(road.line[k + 1].sub(road.line[k]).norm());
                }
                for away in dirs {
                    let side = away.right();
                    for s in 0..6 {
                        let across = (s as f32 - 2.5) * (road.width * 0.13);
                        let c = ix.add(away.mul(6.5)).add(side.mul(across));
                        let ring = [
                            c.sub(side.mul(0.225)).sub(away.mul(1.6)),
                            c.add(side.mul(0.225)).sub(away.mul(1.6)),
                            c.add(side.mul(0.225)).add(away.mul(1.6)),
                            c.sub(side.mul(0.225)).add(away.mul(1.6)),
                        ];
                        g.flat(&ring, 0.058, stripe_col);
                    }
                }
            }
        }
    }
    // Mother Square: octagonal two-tone granite plaza around the sculpture with radial stone walkways.
    for l in city.landmarks.iter().filter(|l| l.kind == "sculpture") {
        let mut oct_outer = Vec::with_capacity(8);
        let mut oct_inner = Vec::with_capacity(8);
        for k in 0..8 {
            let a = (k as f32 + 0.5) * std::f32::consts::TAU / 8.0;
            let p = P2::new(a.cos(), a.sin());
            // Keep slightly compact on north side toward the river embankment.
            let ry_out = if p.y > 0.0 { 16.0 } else { 26.5 };
            let ry_in = if p.y > 0.0 { 10.5 } else { 15.5 };
            oct_outer.push(l.at.add(P2::new(p.x * 26.5, p.y * ry_out)));
            oct_inner.push(l.at.add(P2::new(p.x * 15.5, p.y * ry_in)));
        }
        g.flat(&oct_outer, 0.05, pal.get("granite_paving"));
        g.flat(&oct_inner, 0.058, pal.get("granite_dark"));
        g.flat(
            &[
                P2::new(l.at.x - 7.0, l.at.y - 7.0),
                P2::new(l.at.x + 7.0, l.at.y - 7.0),
                P2::new(l.at.x + 7.0, l.at.y + 7.0),
                P2::new(l.at.x - 7.0, l.at.y + 7.0),
            ],
            0.07,
            pal.get("granite_paving").mix(pal.get("granite_dark"), 0.5),
        );
    }
}

fn nearest_road_dir(city: &City, p: P2) -> (P2, f32) {
    let mut best = f32::MAX;
    let mut dir = P2::new(1.0, 0.0);
    for road in &city.roads {
        for w in road.line.windows(2) {
            let d = p.dist(nearest_on_segment(p, w[0], w[1]));
            if d < best {
                best = d;
                dir = w[1].sub(w[0]).norm();
            }
        }
    }
    (dir, best)
}

fn lamps(city: &City, pal: &Palette, scene: &mut Scene) {
    let post = pal.get("lamp_post");
    for (i, at) in city.lamps.iter().enumerate() {
        let (dir, _) = nearest_road_dir(city, *at);
        let yaw = dir.y.atan2(dir.x);
        scene.props.boxed(*at, 0.0, [0.16, 8.2, 0.16], yaw, post);
        // Two arms over the road and the pavement, as on the real streets.
        let mut lit = Vec::new();
        for side in [-1.0f32, 1.0] {
            let arm = at.add(dir.right().mul(side * 0.9));
            scene.props.boxed(arm, 8.0, [0.1, 0.1, 1.9], yaw, post);
            let head = at.add(dir.right().mul(side * 1.8));
            scene.lamp_glow.boxed(head, 7.86, [0.34, 0.14, 0.7], yaw, pal.get("lamp_sodium"));
            if side > 0.0 || i % 2 == 0 {
                scene.lamp_heads.push(world(head, 7.7));
                lit.push(head);
            }
        }
        let mid = P2::new(lit.iter().map(|p| p.x).sum::<f32>() / lit.len() as f32, lit.iter().map(|p| p.y).sum::<f32>() / lit.len() as f32);
        scene.lamp_posts.push((world(mid, 7.7), lit.len()));
    }
}

fn trees(city: &City, pal: &Palette, props: &mut Soup) {
    let bark = pal.get("tree_bark");
    let lime = pal.get("tree_lime");
    let collar = pal.get("brick_red");
    let stake = pal.get("tree_stake");
    let snow = pal.get("snow");
    for (i, (at, scale)) in city.trees.iter().enumerate() {
        let id = i as i64;
        let yaw = hash01(id, 1) * 1.5;
        let h = 2.6 * scale;
        let lime_h = 1.15 * scale;
        let band_h = 0.07 * scale;
        // Whitewashed lower trunk with a red collar ring, as across Heihe's avenues.
        props.boxed(*at, 0.0, [0.42 * scale, lime_h, 0.42 * scale], yaw, lime);
        props.boxed(*at, lime_h, [0.43 * scale, band_h, 0.43 * scale], yaw, collar);
        props.boxed(*at, lime_h + band_h, [0.40 * scale, (h - lime_h - band_h).max(0.2), 0.40 * scale], yaw, bark);
        if *scale < 1.05 {
            let base_a = hash01(id, 11) * std::f32::consts::TAU;
            for k in 0..3 {
                let a = base_a + k as f32 * (std::f32::consts::TAU / 3.0);
                let d = P2::new(a.cos(), a.sin());
                props.beam(world(at.add(d.mul(0.55)), 0.0), world(at.add(d.mul(0.14)), 1.08 * scale), 0.07, stake);
            }
        }
        // Pollarded: a fist of stubs at the top of a thick trunk, and thin bare shoots.
        for k in 0..5 {
            let a = k as f32 * 1.256 + hash01(id, 20 + k) * 0.6;
            let reach = 0.55 + 0.35 * hash01(id, 30 + k);
            let tip = at.add(P2::new(a.cos(), a.sin()).mul(reach));
            props.boxed(tip, h - 0.2, [0.2, 0.9 + hash01(id, 40 + k), 0.2], a, bark);
            let twig = at.add(P2::new(a.cos(), a.sin()).mul(reach * 1.5));
            props.boxed(twig, h + 0.5, [0.07, 1.6 + hash01(id, 50 + k), 0.07], a, bark.mix(snow, 0.15));
        }
        props.boxed(*at, h + 0.02, [0.5 * scale, 0.08, 0.5 * scale], 0.0, snow);
    }
}

fn poles_and_wires(city: &City, pal: &Palette, props: &mut Soup) {
    let concrete = pal.get("pole_concrete");
    let black = pal.get("car_black");
    let yellow = pal.get("road_line_yellow");
    let arm_col = pal.get("lamp_post");
    let ins_col = pal.get("tile_white");
    let wire_col = pal.get("wire_dark");
    for road in city.roads.iter().filter(|r| !r.pedestrian() && r.width >= 8.5) {
        let line = offset(&road.line, -(road.width * 0.5 + 1.1));
        let mut prev: Option<(P2, P2)> = None;
        along(&line, 42.0, |p, dir| {
            if building_near(city, p, 2.2) {
                prev = None;
                return;
            }
            let yaw = dir.y.atan2(dir.x);
            let across = dir.right();
            props.boxed(p, 0.0, [0.24, 9.0, 0.24], yaw, concrete);
            for b in 0..4 {
                let c = if b % 2 == 0 { black } else { yellow };
                props.boxed(p, 0.2 + b as f32 * 0.35, [0.27, 0.34, 0.27], yaw, c);
            }
            props.boxed(p, 8.35, [0.12, 0.12, 1.6], yaw, arm_col);
            for off in [-0.6, 0.0, 0.6] {
                props.boxed(p.add(across.mul(off)), 8.47, [0.08, 0.14, 0.08], yaw, ins_col);
            }
            if let Some((q, q_across)) = prev {
                let mid = p.lerp(q, 0.5);
                if p.dist(q) < 55.0 && !building_near(city, mid, 1.8) {
                    for s in [-0.55f32, 0.55] {
                        let a2 = q.add(q_across.mul(s));
                        let b2 = p.add(across.mul(s));
                        for seg in 0..4 {
                            let t0 = seg as f32 * 0.25;
                            let t1 = (seg + 1) as f32 * 0.25;
                            let sag0 = 0.45 * 4.0 * t0 * (1.0 - t0);
                            let sag1 = 0.45 * 4.0 * t1 * (1.0 - t1);
                            props.beam(world(a2.lerp(b2, t0), 8.56 - sag0), world(a2.lerp(b2, t1), 8.56 - sag1), 0.035, wire_col);
                        }
                    }
                }
            }
            prev = Some((p, across));
        });
    }
}

fn cars(city: &City, pal: &Palette, props: &mut Soup) {
    let paints = ["car_white", "car_white", "car_silver", "car_white", "car_black", "car_silver"];
    for (i, at) in city.cars.iter().enumerate() {
        let id = i as i64;
        let (dir, _) = nearest_road_dir(city, *at);
        let yaw = dir.y.atan2(dir.x) + std::f32::consts::FRAC_PI_2;
        if hash01(id, 9) < 0.16 {
            let cover = pal.get("camo_cover");
            props.boxed(*at, 0.18, [1.88, 0.72, 4.48], yaw, cover);
            props.boxed(*at, 0.90, [1.66, 0.58, 2.45], yaw, cover);
            props.boxed(*at, 1.48, [1.52, 0.07, 2.15], yaw, pal.get("snow"));
            props.boxed(*at, 0.0, [1.7, 0.2, 3.5], yaw, pal.get("car_black"));
            continue;
        }
        let paint = pal.get(paints[(hash01(id, 3) * paints.len() as f32) as usize % paints.len()]);
        // `boxed` takes [across, up, along] with the long side turned by yaw.
        props.boxed(*at, 0.28, [1.82, 0.62, 4.4], yaw, paint);
        props.boxed(*at, 0.9, [1.6, 0.55, 2.3], yaw, pal.get("glass_day").mix(pal.get("car_black"), 0.5));
        props.boxed(*at, 1.45, [1.62, 0.07, 2.32], yaw, paint);
        if hash01(id, 4) < 0.45 {
            props.boxed(*at, 1.52, [1.5, 0.07, 2.1], yaw, pal.get("snow"));
        }
        // Wheels as one dark slab under the body, and a blue plate at each end.
        props.boxed(*at, 0.0, [1.7, 0.3, 3.5], yaw, pal.get("car_black"));
        for end in [-1.0f32, 1.0] {
            let plate = at.add(dir.mul(end * 2.22));
            props.boxed(plate, 0.42, [0.5, 0.15, 0.04], yaw, pal.get("plate_blue"));
        }
    }
}

fn trikes(city: &City, pal: &Palette, props: &mut Soup) {
    let paints = [pal.get("trike_red"), pal.get("trike_yellow"), pal.get("trike_teal")];
    let dark = pal.get("car_black");
    let glass = pal.get("glass_day");
    let snow = pal.get("snow");
    let mut placed: Vec<P2> = Vec::new();
    let mut place_trike = |at: P2, dir: P2, idx: usize| {
        if placed.len() >= 92 {
            return;
        }
        if building_near(city, at, 1.3)
            || city.cars.iter().any(|c| c.dist(at) < 4.5)
            || placed.iter().any(|p| p.dist(at) < 6.0)
        {
            return;
        }
        placed.push(at);
        let yaw = dir.y.atan2(dir.x) + std::f32::consts::FRAC_PI_2;
        let paint = paints[idx % paints.len()];
        // Front wheel and narrow nose + enclosed Northeast winter cab behind.
        props.boxed(at.add(dir.mul(0.85)), 0.0, [0.22, 0.52, 0.52], yaw, dark);
        props.boxed(at.add(dir.mul(0.55)), 0.32, [0.78, 0.85, 0.72], yaw, paint);
        props.boxed(at.add(dir.mul(0.52)), 1.05, [0.74, 0.42, 0.46], yaw, glass);
        props.boxed(at.sub(dir.mul(0.28)), 0.28, [1.14, 1.32, 1.34], yaw, paint);
        props.boxed(at.sub(dir.mul(0.28)), 0.92, [1.18, 0.42, 0.88], yaw, glass);
        props.boxed(at.sub(dir.mul(0.28)), 1.60, [1.06, 0.07, 1.24], yaw, snow);
        props.boxed(at.sub(dir.mul(0.45)), 0.0, [1.10, 0.28, 0.6], yaw, dark);
    };
    for (i, road) in city.roads.iter().filter(|r| !r.pedestrian() && r.width >= 8.5).enumerate() {
        for side in [-1.0f32, 1.0] {
            let edge = offset(&road.line, side * (road.width * 0.5 + 1.85));
            let mut k = 0usize;
            along(&edge, 28.0, |p, dir| {
                k += 1;
                if hash01((i * 500 + k) as i64, if side > 0.0 { 31 } else { 37 }) < 0.32 {
                    place_trike(p, dir, i + k);
                }
            });
        }
    }
    for (i, sign) in city.signs.iter().filter(|s| s.shop).enumerate() {
        if hash01(i as i64, 71) < 0.45 {
            let (dir, _) = nearest_road_dir(city, sign.at);
            place_trike(sign.at.add(dir.right().mul(2.4)), dir, i);
        }
    }
}

fn central_street(city: &City, pal: &Palette, scene: &mut Scene) {
    let guandu = P2::new(-378.8, -258.1);
    let pushkin = P2::new(-228.0, -333.5);
    let red_bag = P2::new(-272.0, -311.5);
    let rainbow = P2::new(-316.0, -289.5);
    let mut main_dir = P2::new(-0.8944, 0.4472);

    for road in city.roads.iter().filter(|r| r.name == "中央街") {
        if road.line.len() >= 2 {
            main_dir = road.line[1].sub(road.line[0]).norm();
        }
        // Stone balls along both edges of the pedestrian street.
        for side in [-1.0f32, 1.0] {
            let edge = offset(&road.line, side * (road.width * 0.5 - 0.6));
            along(&edge, 9.0, |p, _| {
                scene.props.boxed(p, 0.0, [0.5, 0.18, 0.5], 0.0, pal.get("granite_dark"));
                scene.props.cone(p, 0.18, 0.34, 0.36, 8, pal.get("bollard_stone"));
                scene.props.cone(p, 0.5, 0.2, 0.16, 8, pal.get("snow"));
            });
            // Pedestrian lamps with paired red medallions (ref00, ref05, ref07).
            let lamp_edge = offset(&road.line, side * (road.width * 0.5 - 1.25));
            along(&lamp_edge, 18.0, |p, dir| {
                let yaw = dir.y.atan2(dir.x);
                scene.props.boxed(p, 0.0, [0.14, 5.0, 0.14], 0.0, pal.get("lamp_post"));
                scene.props.boxed(p, 5.0, [0.58, 0.10, 0.58], 0.0, pal.get("lamp_post"));
                scene.lamp_glow.boxed(p, 4.84, [0.44, 0.14, 0.44], 0.0, pal.get("lamp_sodium"));
                scene.lamp_heads.push(world(p, 4.75));
                scene.lamp_posts.push((world(p, 4.75), 1));
                scene.props.beam(world(p.sub(dir.mul(0.42)), 3.1), world(p.add(dir.mul(0.42)), 3.1), 0.06, pal.get("lamp_post"));
                for m_side in [-1.0f32, 1.0] {
                    let mp = p.add(dir.mul(m_side * 0.36));
                    scene.props.boxed(mp, 2.84, [0.54, 0.54, 0.05], yaw, pal.get("central_cream"));
                    scene.props.boxed(mp, 2.88, [0.46, 0.46, 0.07], yaw, pal.get("medallion_red"));
                }
            });
        }
        // Centreline Russian onion-dome topiary towers and wooden benches (ref07, ref08).
        along(&road.line, 38.0, |p, dir| {
            if p.dist(guandu) < 10.0 || p.dist(pushkin) < 8.0 || p.dist(red_bag) < 8.0 || p.dist(rainbow) < 8.0 {
                return;
            }
            let yaw = dir.y.atan2(dir.x);
            let topiary = pal.get("topiary_green");
            scene.props.boxed(p, 0.0, [2.1, 0.55, 2.1], yaw, pal.get("granite_dark"));
            scene.props.boxed(p, 0.55, [1.85, 0.22, 1.85], yaw, pal.get("central_cream"));
            scene.props.boxed(p, 0.77, [1.9, 0.06, 1.9], yaw, pal.get("snow"));
            scene.props.cone_turned(p, 0.8, 0.82, 2.3, 8, yaw, topiary);
            scene.props.boxed(p, 2.4, [1.05, 0.8, 1.05], yaw, topiary);
            oct_prism(&mut scene.props, p, 3.2, 3.55, 0.52, 0.78, topiary);
            scene.props.cone_turned(p, 3.55, 0.78, 1.25, 8, yaw, topiary);
            scene.props.boxed(p, 4.7, [0.08, 0.55, 0.08], yaw, pal.get("road_line_yellow"));
            scene.props.boxed(p, 4.95, [0.32, 0.08, 0.08], yaw, pal.get("road_line_yellow"));
            for end in [-1.0f32, 1.0] {
                let bp = p.add(dir.mul(end * 3.2));
                scene.props.boxed(bp, 0.42, [1.7, 0.08, 0.52], yaw, pal.get("bench_wood"));
                scene.props.boxed(bp.add(dir.right().mul(-0.22)), 0.5, [1.7, 0.36, 0.07], yaw, pal.get("bench_wood"));
                for leg in [-0.7f32, 0.7] {
                    scene.props.boxed(bp.add(dir.mul(leg)), 0.0, [0.1, 0.42, 0.48], yaw, pal.get("rail_iron"));
                }
            }
        });
    }

    // Pushkin bronze statue in front of the Pushkin / Xinhua bookstore (ref05).
    let yaw = main_dir.y.atan2(main_dir.x);
    let bronze = pal.get("mother_bronze");
    scene.props.boxed(pushkin, 0.0, [2.3, 0.35, 2.3], yaw, pal.get("granite_dark"));
    scene.props.boxed(pushkin, 0.35, [1.8, 1.4, 1.8], yaw, pal.get("mother_plinth"));
    scene.props.boxed(pushkin, 1.75, [1.95, 0.15, 1.95], yaw, pal.get("granite_dark"));
    scene.props.boxed(pushkin, 1.9, [0.42, 0.95, 0.38], yaw, bronze);
    scene.props.boxed(pushkin, 2.85, [0.68, 0.95, 0.46], yaw, bronze);
    scene.props.beam(world(pushkin.sub(main_dir.mul(0.22)), 3.65), world(pushkin.sub(main_dir.mul(0.42)), 2.55), 0.24, bronze);
    scene.props.boxed(pushkin, 3.8, [0.28, 0.34, 0.28], yaw, bronze);

    // Red hollow-cut shopping-bag sculpture and bronze shopper figures (ref08).
    let bag_red = pal.get("bag_red");
    let iron = pal.get("rail_iron");
    let across = main_dir.right();
    scene.props.boxed(red_bag, 0.0, [2.2, 0.26, 0.58], yaw, bag_red);
    scene.props.boxed(red_bag, 1.88, [2.2, 0.24, 0.58], yaw, bag_red);
    for px in [-0.88f32, -0.28, 0.32, 0.88] {
        scene.props.boxed(red_bag.add(main_dir.mul(px)), 0.26, [0.22, 1.62, 0.55], yaw, bag_red);
    }
    for h_side in [-0.18f32, 0.18] {
        let off = across.mul(h_side);
        let p0 = world(red_bag.add(off).sub(main_dir.mul(0.45)), 2.12);
        let p1 = world(red_bag.add(off).sub(main_dir.mul(0.22)), 2.75);
        let p2 = world(red_bag.add(off).add(main_dir.mul(0.22)), 2.75);
        let p3 = world(red_bag.add(off).add(main_dir.mul(0.45)), 2.12);
        scene.props.beam(p0, p1, 0.07, iron);
        scene.props.beam(p1, p2, 0.07, iron);
        scene.props.beam(p2, p3, 0.07, iron);
    }
    for (dx, dy, h) in [(1.9, 0.4, 1.72), (2.45, -0.2, 1.66)] {
        let fp = red_bag.add(across.mul(dx)).add(main_dir.mul(dy));
        scene.props.boxed(fp, 0.0, [0.34, 0.85, 0.30], yaw, bronze);
        scene.props.boxed(fp, 0.85, [0.42, 0.62, 0.26], yaw, bronze);
        scene.props.boxed(fp, h - 0.24, [0.22, 0.24, 0.22], yaw, bronze);
    }

    // Central Street decorative rainbow arch (ref08, ref09).
    let span_dir = across;
    for side in [-1.0f32, 1.0] {
        scene.props.boxed(rainbow.add(span_dir.mul(side * 4.7)), 0.0, [0.45, 2.8, 0.45], yaw, pal.get("baroque_red"));
    }
    let arch_cols = [pal.get("baroque_red"), pal.get("road_line_yellow"), pal.get("dome_green"), pal.get("arch_teal")];
    for seg in 0..8 {
        let u0 = -1.0 + seg as f32 * 0.25;
        let u1 = u0 + 0.25;
        let p0 = world(rainbow.add(span_dir.mul(u0 * 4.7)), 2.8 + 3.4 * (1.0 - u0 * u0));
        let p1 = world(rainbow.add(span_dir.mul(u1 * 4.7)), 2.8 + 3.4 * (1.0 - u1 * u1));
        scene.props.beam(p0, p1, 0.48, arch_cols[seg % arch_cols.len()]);
    }

    // Tall brick heating chimney off Central Street (ref08, central-2).
    let chimney = P2::new(-248.0, -286.0);
    scene.props.boxed(chimney, 0.0, [3.4, 4.0, 3.4], 0.0, pal.get("chimney_brick"));
    oct_prism(&mut scene.props, chimney, 4.0, 26.0, 1.65, 1.05, pal.get("chimney_brick"));
    oct_prism(&mut scene.props, chimney, 26.0, 27.6, 1.05, 1.00, pal.get("central_cream"));
    oct_prism(&mut scene.props, chimney, 27.6, 29.2, 1.00, 0.95, pal.get("chimney_brick"));
    oct_prism(&mut scene.props, chimney, 29.2, 30.2, 0.95, 0.92, pal.get("central_cream"));
    oct_prism(&mut scene.props, chimney, 30.2, 31.4, 0.92, 0.88, pal.get("chimney_brick"));
    oct_prism(&mut scene.props, chimney, 31.4, 32.0, 0.95, 0.95, pal.get("car_black"));
}

fn teal_arch(city: &City, scene: &mut Scene, pal: &Palette, centre: P2, dir: P2, span: f32) {
    let teal = pal.get("arch_teal");
    let cream = pal.get("central_cream");
    let iron = pal.get("rail_iron");
    let globe_col = pal.get("tile_white").mix(pal.get("lamp_sodium"), 0.3);
    let across = dir.right();
    let yaw = dir.y.atan2(dir.x);
    let left = centre.sub(across.mul(span * 0.5));
    let right = centre.add(across.mul(span * 0.5));

    for (idx, col_at) in [left, right].into_iter().enumerate() {
        oct_prism(&mut scene.props, col_at, 0.0, 0.45, 0.85, 0.72, teal);
        scene.props.boxed(col_at, 0.45, [1.1, 0.35, 1.1], yaw, teal);
        // Three interwoven helical struts per column (ref10).
        for s in 0..3 {
            let phase = s as f32 * std::f32::consts::TAU / 3.0;
            for step in 0..6 {
                let t0 = step as f32 / 6.0;
                let t1 = (step + 1) as f32 / 6.0;
                let a0 = phase + t0 * std::f32::consts::TAU;
                let a1 = phase + t1 * std::f32::consts::TAU;
                let p0 = col_at.add(dir.mul(a0.cos() * 0.28)).add(across.mul(a0.sin() * 0.28));
                let p1 = col_at.add(dir.mul(a1.cos() * 0.28)).add(across.mul(a1.sin() * 0.28));
                scene.props.beam(world(p0, 0.8 + t0 * 6.0), world(p1, 0.8 + t1 * 6.0), 0.14, teal);
            }
        }
        if idx == 1 {
            // Needle spire crowning the right column, matching ref10.
            scene.props.cone(col_at, 6.8, 0.24, 2.8, 6, teal);
        } else {
            scene.props.boxed(col_at, 6.8, [0.46, 0.35, 0.46], yaw, teal);
        }
        let inward = if idx == 0 { 1.0 } else { -1.0 };
        let plaque = col_at.add(across.mul(inward * 0.85));
        scene.props.boxed(plaque, 3.1, [0.16, 1.25, 1.55], yaw, teal);
        scene.props.boxed(plaque, 3.25, [0.19, 0.95, 1.25], yaw, cream);
        scene.props.beam(world(col_at, 6.3), world(col_at.add(across.mul(inward * 1.4)), 7.2), 0.11, teal);
    }

    // Double curved arch ribs + truss web peaking at 7.8 m.
    let segs = 12usize;
    for i in 0..segs {
        let u0 = -1.0 + 2.0 * (i as f32 / segs as f32);
        let u1 = -1.0 + 2.0 * ((i + 1) as f32 / segs as f32);
        let p0 = centre.add(across.mul(u0 * span * 0.5));
        let p1 = centre.add(across.mul(u1 * span * 0.5));
        let lo0 = 4.9 + 2.1 * (1.0 - u0 * u0);
        let lo1 = 4.9 + 2.1 * (1.0 - u1 * u1);
        let hi0 = 5.7 + 2.1 * (1.0 - u0 * u0);
        let hi1 = 5.7 + 2.1 * (1.0 - u1 * u1);
        scene.props.beam(world(p0, lo0), world(p1, lo1), 0.22, teal);
        scene.props.beam(world(p0, hi0), world(p1, hi1), 0.20, teal);
        scene.props.beam(world(p0, lo0), world(p0, hi0), 0.11, teal);
        scene.props.beam(world(p0, lo0), world(p1, hi1), 0.09, teal);
    }

    // Flanking black-post white-globe streetlamps (ref10).
    for along_off in [-22.0f32, -10.0, 10.0, 22.0] {
        for side in [-1.0f32, 1.0] {
            let lp = centre.add(dir.mul(along_off)).add(across.mul(side * (span * 0.5 - 0.4)));
            if building_near(city, lp, 0.8) {
                continue;
            }
            scene.props.boxed(lp, 0.0, [0.14, 4.1, 0.14], yaw, iron);
            scene.props.boxed(lp, 3.85, [0.10, 0.08, 0.62], yaw, iron);
            scene.lamp_glow.boxed(lp, 4.1, [0.40, 0.40, 0.40], 0.0, globe_col);
            scene.lamp_heads.push(world(lp, 4.1));
            scene.lamp_posts.push((world(lp, 4.1), 1));
        }
    }
}

fn goods_arch(city: &City, pal: &Palette, scene: &mut Scene) {
    // Twin teal helical-column Russian Goods Street arches across Wenhua St and Hailan St (ref10).
    for (at, span) in [(P2::new(-159.3, -246.5), 11.0), (P2::new(-116.4, -156.4), 12.0)] {
        let (dir, _) = nearest_road_dir(city, at);
        teal_arch(city, scene, pal, at, dir, span);
    }
}

fn mother_characters(props: &mut Soup, at: P2, colour: Rgb) {
    // Engraved green relief strokes for 「母親」 on the south and north faces of the plinth (ref04).
    for face in [-1.0f32, 1.0] {
        let fy = at.y + face * 1.52;
        let d = [0.12f32, 0.12, 0.06];
        // Upper character「母」around up = 5.4..7.2
        props.boxed(P2::new(at.x, fy), 7.02, [1.22, 0.13, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x, fy), 5.48, [1.22, 0.13, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x - 0.54, fy), 5.48, [0.13, 1.67, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x + 0.54, fy), 5.48, [0.13, 1.67, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x, fy), 6.26, [1.56, 0.12, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x, fy), 6.56, d, 0.0, colour);
        props.boxed(P2::new(at.x, fy), 5.82, d, 0.0, colour);
        // Lower character「親」around up = 3.0..4.8
        props.boxed(P2::new(at.x - 0.36, fy), 4.52, [0.68, 0.11, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x - 0.36, fy), 4.02, [0.76, 0.11, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x - 0.36, fy), 3.62, [0.68, 0.11, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x - 0.36, fy), 3.06, [0.12, 1.66, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x + 0.36, fy), 3.06, [0.12, 1.68, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x + 0.62, fy), 3.22, [0.11, 1.48, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x + 0.42, fy), 4.62, [0.64, 0.11, 0.06], 0.0, colour);
        props.boxed(P2::new(at.x + 0.42, fy), 3.74, [0.58, 0.11, 0.06], 0.0, colour);
    }
}

fn mother(city: &City, pal: &Palette, scene: &mut Scene) {
    let bronze = pal.get("mother_bronze");
    let plinth = pal.get("mother_plinth");
    let dark = pal.get("granite_dark");
    let snow = pal.get("snow");
    let iron = pal.get("rail_iron");
    let globe_col = pal.get("tile_white").mix(pal.get("lamp_sodium"), 0.3);
    let chars_col = pal.get("dome_green").mix(bronze, 0.35);

    for l in city.landmarks.iter().filter(|l| l.kind == "sculpture") {
        let at = l.at;
        // Tall plinth of dark red polished granite with stepped stone foot and cap.
        scene.props.boxed(at, 0.0, [4.8, 0.35, 4.8], 0.0, dark);
        scene.props.boxed(at, 0.35, [3.9, 0.35, 3.9], 0.0, pal.get("granite_paving"));
        scene.props.boxed(at, 0.7, [3.0, 8.3, 3.0], 0.0, plinth);
        scene.props.boxed(at, 9.0, [3.3, 0.25, 3.3], 0.0, dark);
        mother_characters(&mut scene.props, at, chars_col);
        // The figure faces the river: a long skirt, a body, arms straight up, and the winged child held high.
        scene.props.cone(at, 9.25, 1.25, 4.6, 10, bronze);
        scene.props.boxed(at, 12.6, [0.95, 1.5, 0.6], 0.0, bronze);
        scene.props.boxed(at, 14.1, [0.5, 0.55, 0.5], 0.0, bronze);
        for side in [-1.0f32, 1.0] {
            scene.props.boxed(at.add(P2::new(side * 0.36, 0.0)), 13.9, [0.2, 2.1, 0.2], 0.0, bronze);
        }
        scene.props.boxed(at, 16.0, [0.5, 0.75, 0.4], 0.0, bronze);
        for side in [-1.0f32, 1.0] {
            scene.props.boxed(at.add(P2::new(side * 0.85, 0.0)), 16.35, [1.2, 0.5, 0.12], side * 0.5, bronze);
        }
        scene.props.boxed(at, 9.25, [3.3, 0.04, 3.3], 0.0, snow);

        // Granite square bollards and twin-globe plaza lamps framing Mother Square.
        for dx in [-11.5f32, 11.5] {
            for dy in [-11.5f32, 8.5] {
                let b = at.add(P2::new(dx, dy));
                if !building_near(city, b, 1.0) {
                    scene.props.boxed(b, 0.0, [3.4, 0.48, 0.9], 0.0, dark);
                    scene.props.boxed(b, 0.48, [3.2, 0.04, 0.75], 0.0, snow);
                }
            }
        }
        for dx in [-18.0f32, 18.0] {
            for dy in [-18.0f32, 9.0] {
                let lp = at.add(P2::new(dx, dy));
                if building_near(city, lp, 1.5) {
                    continue;
                }
                scene.props.boxed(lp, 0.0, [0.16, 4.6, 0.16], 0.0, iron);
                scene.props.boxed(lp, 4.35, [1.3, 0.09, 0.10], 0.0, iron);
                for s in [-1.0f32, 1.0] {
                    let hp = lp.add(P2::new(s * 0.58, 0.0));
                    scene.lamp_glow.boxed(hp, 4.55, [0.38, 0.38, 0.38], 0.0, globe_col);
                    scene.lamp_heads.push(world(hp, 4.55));
                }
                scene.lamp_posts.push((world(lp, 4.55), 2));
            }
        }
    }
    let flag_red = pal.get("medallion_red");
    for l in city.landmarks.iter().filter(|l| l.kind == "boundary_stone") {
        scene.props.boxed(l.at, 0.0, [2.8, 0.25, 2.0], 0.0, dark);
        scene.props.boxed(l.at, 0.25, [1.8, 0.25, 1.3], 0.0, plinth);
        scene.props.boxed(l.at, 0.50, [0.78, 2.35, 0.36], 0.0, pal.get("embankment"));
        scene.props.boxed(l.at, 2.85, [0.84, 0.14, 0.42], 0.0, snow);
        scene.props.boxed(l.at, 2.15, [0.44, 0.44, 0.40], 0.0, flag_red);
        scene.props.boxed(l.at, 1.15, [0.28, 0.85, 0.40], 0.0, flag_red);
        // Twin flanking flagpoles by 192号界碑.
        for s in [-1.0f32, 1.0] {
            let fp = l.at.add(P2::new(s * 2.1, -0.4));
            scene.props.boxed(fp, 0.0, [0.10, 6.2, 0.10], 0.0, pal.get("wheel_white"));
            scene.props.boxed(fp.add(P2::new(0.55, 0.0)), 5.3, [1.1, 0.72, 0.04], 0.0, flag_red);
        }
    }
}

fn nearest_bank_dir(bank: &[P2], p: P2) -> P2 {
    let mut best = f32::MAX;
    let mut dir = P2::new(1.0, 0.0);
    for w in bank.windows(2) {
        let d = p.dist(nearest_on_segment(p, w[0], w[1]));
        if d < best {
            best = d;
            dir = w[1].sub(w[0]).norm();
        }
    }
    dir
}

fn clear_park_spot(city: &City, approx: P2) -> P2 {
    let river_out = smooth_river_out(&city.bank);
    let mother = city
        .landmarks
        .iter()
        .find(|l| l.kind == "sculpture")
        .map(|l| l.at)
        .unwrap_or(P2::new(0.0, 0.0));
    for step in 0..8 {
        let cand = approx.add(river_out.mul(step as f32 * 5.0));
        if !building_near(city, cand, 3.6) && cand.dist(mother) > 29.0 {
            return cand;
        }
    }
    approx
}

fn riverfront(city: &City, pal: &Palette, scene: &mut Scene) {
    if city.bank.len() < 2 {
        return;
    }
    let stone = pal.get("embankment");
    let dark = pal.get("granite_dark");
    let snow = pal.get("snow");
    let iron = pal.get("rail_iron");
    let red_med = pal.get("medallion_red");
    let globe_col = pal.get("tile_white").mix(pal.get("lamp_sodium"), 0.3);
    let river_out = smooth_river_out(&city.bank);

    let near_ice_gate = |p: P2| -> bool {
        let g0 = P2::new(-95.0, 42.0);
        let g1 = P2::new(18.0, 6.0);
        let g2 = P2::new(195.0, -68.0);
        p.dist(g0) < 9.0 || p.dist(g1) < 8.5 || p.dist(g2) < 9.0
    };

    // 1. Solid stepped granite floodwall & balustrade along the Heilongjiang promenade + 3 ice-access stair gates.
    let rail_line = offset(&city.bank, 0.9);
    let mut bay = 0usize;
    along(&rail_line, 3.2, |p, dir| {
        bay += 1;
        let yaw = dir.y.atan2(dir.x);
        if near_ice_gate(p) {
            // Wide stepped granite landing leading down to the frozen river ice.
            for st in 0..3 {
                let sp = p.sub(dir.right().mul(st as f32 * 0.65));
                scene.props.boxed(sp, 0.0, [3.2, 0.36 - st as f32 * 0.10, 0.65], yaw, dark);
            }
            return;
        }
        scene.props.boxed(p, 0.0, [3.26, 0.38, 0.44], yaw, dark);
        scene.props.boxed(p, 0.0, [0.24, 1.05, 0.24], yaw, stone);
        scene.props.boxed(p, 1.05, [0.26, 0.04, 0.26], yaw, snow);
        scene.props.boxed(p, 0.94, [3.22, 0.14, 0.26], yaw, stone);
        scene.props.boxed(p, 1.08, [3.20, 0.03, 0.22], yaw, snow);
        for off in [-0.9f32, 0.9] {
            scene.props.boxed(p.add(dir.mul(off)), 0.38, [0.08, 0.56, 0.08], yaw, iron);
        }
        if bay % 4 == 0 {
            scene.props.boxed(p, 0.46, [0.55, 0.34, 0.28], yaw, red_med);
        }
    });

    // 2. Promenade twin-globe streetlamps facing the river.
    along(&offset(&city.bank, 8.2), 26.0, |p, dir| {
        if building_near(city, p, 1.5) {
            return;
        }
        let yaw = dir.y.atan2(dir.x);
        let across = dir.right();
        scene.props.boxed(p, 0.0, [0.16, 4.6, 0.16], yaw, iron);
        scene.props.boxed(p, 4.35, [0.10, 0.09, 1.3], yaw, iron);
        for s in [-1.0f32, 1.0] {
            let hp = p.add(across.mul(s * 0.58));
            scene.lamp_glow.boxed(hp, 4.55, [0.38, 0.38, 0.38], 0.0, globe_col);
            scene.lamp_heads.push(world(hp, 4.55));
        }
        scene.lamp_posts.push((world(p, 4.55), 2));
    });

    // 3. Clean pressure-ice blocks shoved gently up against the riverbank wall.
    let ice = pal.get("river_ice");
    let mut idx = 0i64;
    along(&offset(&city.bank, -5.5), 14.0, |p, _| {
        idx += 1;
        let h = hash01(idx * 31, 3);
        if h < 0.62 {
            let yaw = hash01(idx, 9) * std::f32::consts::TAU;
            let w = 1.1 + hash01(idx, 2) * 1.4;
            let ht = 0.24 + hash01(idx, 4) * 0.36;
            scene.props.boxed(p, -0.02, [w, ht, w * 0.55], yaw, ice.mix(snow, 0.35));
        }
    });

    // 4. Heihe Riverside Ferris wheel at (105, -35) (ref13).
    let wheel_at = P2::new(105.0, -35.0);
    let wdir = nearest_bank_dir(&city.bank, wheel_at);
    let wperp = wdir.right();
    let wyaw = wdir.y.atan2(wdir.x);
    let white = pal.get("wheel_white");
    let hub_up = 29.0f32;
    let r_outer = 21.0f32;
    let r_inner = 17.5f32;
    for sx in [-1.0f32, 1.0] {
        for sy in [-1.0f32, 1.0] {
            let foot = wheel_at.add(wdir.mul(sx * 11.0)).add(wperp.mul(sy * 4.5));
            let top = wheel_at.add(wperp.mul(sy * 2.2));
            scene.props.boxed(foot, 0.0, [1.6, 0.55, 1.6], wyaw, dark);
            scene.props.beam(world(foot, 0.5), world(top, hub_up), 0.38, white);
        }
    }
    scene.props.beam(world(wheel_at.sub(wperp.mul(2.5)), hub_up), world(wheel_at.add(wperp.mul(2.5)), hub_up), 0.52, white);
    let ring_segs = 20usize;
    let cabin_cols = [pal.get("trike_red"), white, pal.get("arch_teal"), pal.get("trike_yellow")];
    for i in 0..ring_segs {
        let a0 = i as f32 * std::f32::consts::TAU / ring_segs as f32;
        let a1 = (i + 1) as f32 * std::f32::consts::TAU / ring_segs as f32;
        let rim0 = wheel_at.add(wdir.mul(a0.cos() * r_outer));
        let rim1 = wheel_at.add(wdir.mul(a1.cos() * r_outer));
        let in0 = wheel_at.add(wdir.mul(a0.cos() * r_inner));
        let in1 = wheel_at.add(wdir.mul(a1.cos() * r_inner));
        let y0 = hub_up + a0.sin() * r_outer;
        let y1 = hub_up + a1.sin() * r_outer;
        let iy0 = hub_up + a0.sin() * r_inner;
        let iy1 = hub_up + a1.sin() * r_inner;
        for s in [-1.4f32, 1.4] {
            let off = wperp.mul(s);
            scene.props.beam(world(rim0.add(off), y0), world(rim1.add(off), y1), 0.24, white);
            scene.props.beam(world(in0.add(off), iy0), world(in1.add(off), iy1), 0.18, white);
            scene.props.beam(world(in0.add(off), iy0), world(rim0.add(off), y0), 0.12, white);
            if i % 2 == 0 {
                scene.props.beam(world(wheel_at.add(off), hub_up), world(rim0.add(off), y0), 0.14, white);
            }
        }
        scene.props.beam(world(rim0.sub(wperp.mul(1.4)), y0), world(rim0.add(wperp.mul(1.4)), y0), 0.14, white);
        scene.props.beam(world(rim0, y0), world(rim0, y0 - 0.8), 0.08, iron);
        scene.props.boxed(rim0, y0 - 2.35, [1.5, 1.55, 1.25], wyaw, cabin_cols[i % cabin_cols.len()]);
        scene.lamp_glow.boxed(rim0, y0, [0.28, 0.28, 0.28], 0.0, globe_col);
    }

    // 5. Steel lattice TV tower at (75, -20), 58 m tall with observation deck (ref13).
    let tc = P2::new(75.0, -20.0);
    let bred = pal.get("baroque_red");
    scene.props.boxed(tc, 0.0, [8.4, 0.55, 8.4], 0.0, dark);
    let tiers = [(0.55f32, 3.8f32), (10.0, 2.9), (20.0, 2.1), (30.0, 1.45), (38.0, 1.1)];
    for w in tiers.windows(2) {
        let ((z0, r0), (z1, r1)) = (w[0], w[1]);
        let corners = [P2::new(-1.0, -1.0), P2::new(1.0, -1.0), P2::new(1.0, 1.0), P2::new(-1.0, 1.0)];
        for k in 0..4 {
            let c0 = corners[k];
            let c1 = corners[(k + 1) % 4];
            let a0 = tc.add(c0.mul(r0));
            let a1 = tc.add(c0.mul(r1));
            let b0 = tc.add(c1.mul(r0));
            let b1 = tc.add(c1.mul(r1));
            scene.props.beam(world(a0, z0), world(a1, z1), 0.24, white);
            scene.props.beam(world(a1, z1), world(b1, z1), 0.16, white);
            scene.props.beam(world(a0, z0), world(b1, z1), 0.12, white);
            scene.props.beam(world(b0, z0), world(a1, z1), 0.12, white);
        }
    }
    oct_prism(&mut scene.props, tc, 38.0, 39.2, 1.25, 2.7, white);
    oct_prism(&mut scene.props, tc, 39.2, 40.6, 2.7, 2.7, pal.get("glass_day"));
    oct_prism(&mut scene.props, tc, 40.6, 41.5, 2.7, 1.4, white);
    oct_prism(&mut scene.props, tc, 41.5, 46.0, 1.05, 0.72, bred);
    oct_prism(&mut scene.props, tc, 46.0, 50.5, 0.72, 0.45, white);
    oct_prism(&mut scene.props, tc, 50.5, 54.5, 0.42, 0.22, bred);
    oct_prism(&mut scene.props, tc, 54.5, 58.0, 0.20, 0.06, white);
    scene.lamp_glow.boxed(tc, 58.0, [0.35, 0.35, 0.35], 0.0, bred);

    // 6. Red-spired tourist boat pavilion moored off the riverfront at (-48, 58) (ref13).
    let bc = P2::new(-48.0, 58.0);
    let bdir = nearest_bank_dir(&city.bank, bc);
    let byaw = bdir.y.atan2(bdir.x);
    let hull_white = pal.get("tile_white");
    let roof_red = pal.get("spire_red");
    scene.props.boxed(bc, 0.0, [26.0, 0.9, 8.2], byaw, dark);
    scene.props.boxed(bc, 0.9, [25.0, 0.7, 8.4], byaw, hull_white);
    scene.props.boxed(bc, 1.6, [19.5, 2.9, 6.8], byaw, hull_white);
    scene.props.boxed(bc, 2.3, [16.5, 1.35, 6.9], byaw, pal.get("glass_day"));
    scene.lit.boxed(bc, 2.45, [15.5, 0.95, 6.94], byaw, pal.get("glass_lit"));
    scene.props.boxed(bc, 4.5, [20.6, 0.35, 7.6], byaw, roof_red);
    scene.props.boxed(bc, 4.85, [12.5, 2.5, 5.4], byaw, hull_white);
    scene.props.boxed(bc, 5.4, [10.0, 1.15, 5.5], byaw, pal.get("glass_day"));
    scene.props.boxed(bc, 7.35, [13.4, 0.45, 6.1], byaw, roof_red);
    scene.props.boxed(bc, 7.80, [13.0, 0.05, 5.8], byaw, snow);
    oct_prism(&mut scene.props, bc, 7.8, 10.2, 2.4, 2.1, hull_white);
    scene.props.cone(bc, 10.2, 2.75, 4.4, 8, roof_red);
    let gangway = bc.add(bdir.right().mul(5.8));
    scene.props.boxed(gangway, 0.45, [2.4, 0.35, 7.6], byaw, hull_white);

    // 7. Heilongjiang Riverside Park (`黑龙江公园`): white-bark birches, snowy pines, benches & 2 traditional pavilions.
    let birch_col = Rgb { r: 0.85, g: 0.85, b: 0.86 };
    let knot_col = Rgb { r: 0.18, g: 0.18, b: 0.20 };
    let pine_col = pal.get("topiary_green");
    let bark_col = pal.get("tree_bark");
    let bench_col = pal.get("bench_wood");
    let mother = city
        .landmarks
        .iter()
        .find(|l| l.kind == "sculpture")
        .map(|l| l.at)
        .unwrap_or(P2::new(0.0, 0.0));

    for (band_i, (park_off, spacing)) in [(15.0f32, 16.0f32), (30.0, 18.0), (44.0, 22.0)].into_iter().enumerate() {
        let mut t_i = 0i64;
        along(&offset(&city.bank, park_off), spacing, |p, dir| {
            t_i += 1;
            if p.dist(mother) < 28.5 || building_near(city, p, 3.2) {
                return;
            }
            let on_road = city.roads.iter().any(|r| dist_to_poly_local(p, &r.line) < r.width * 0.5 + 1.8);
            if on_road {
                return;
            }
            let seed = t_i * 73 + band_i as i64 * 131;
            if hash01(seed, 1) < 0.55 {
                // Northern winter birch (`白桦树`): pale ringed trunk + wispy branches.
                let h = 4.2 + hash01(seed, 2) * 2.2;
                scene.props.boxed(p, 0.0, [0.24, h, 0.24], 0.0, birch_col);
                for k in 0..3 {
                    scene.props.boxed(p, 0.9 + k as f32 * 0.95, [0.26, 0.14, 0.26], 0.0, knot_col);
                }
                for b in 0..4 {
                    let a = b as f32 * 1.57 + hash01(seed, 10 + b) * 0.8;
                    let bp = p.add(P2::new(a.cos(), a.sin()).mul(0.85));
                    scene.props.beam(world(p, h * 0.65), world(bp, h + 1.1), 0.055, bark_col);
                }
                scene.props.boxed(p, h, [0.55, 0.08, 0.55], 0.0, snow);
            } else {
                // Dark snow-laden spruce / pine (`雪松`).
                let sc = 0.85 + hash01(seed, 3) * 0.45;
                scene.props.boxed(p, 0.0, [0.28, 1.3 * sc, 0.28], 0.0, bark_col);
                for tier in 0..3 {
                    let y0 = 1.0 * sc + tier as f32 * 1.25 * sc;
                    let r = (1.65 - tier as f32 * 0.40) * sc;
                    scene.props.cone(p, y0, r, 1.75 * sc, 7, pine_col);
                    scene.props.cone(p, y0 + 0.18 * sc, r * 0.92, 1.62 * sc, 7, snow.mix(pine_col, 0.22));
                }
            }
            if band_i == 0 && t_i % 3 == 0 {
                let yaw = dir.y.atan2(dir.x);
                scene.props.boxed(p.add(dir.mul(2.5)), 0.0, [1.8, 0.48, 0.65], yaw, bench_col);
            }
        });
    }

    // 2 Traditional Chinese Octagonal Riverside Pavilions (`江畔观景亭`).
    for approx in [P2::new(-118.0, 26.0), P2::new(160.0, -92.0)] {
        let pv = clear_park_spot(city, approx);
        oct_prism(&mut scene.props, pv, 0.0, 0.42, 4.4, 4.4, dark);
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.0;
            let col_p = pv.add(P2::new(a.cos(), a.sin()).mul(3.4));
            scene.props.boxed(col_p, 0.42, [0.24, 3.1, 0.24], 0.0, bred);
        }
        scene.props.cone(pv, 3.52, 4.8, 1.9, 8, roof_red);
        scene.props.cone(pv, 5.15, 3.1, 2.2, 8, roof_red);
        scene.props.cone(pv, 5.25, 2.95, 2.15, 8, snow.mix(roof_red, 0.2));
        scene.props.boxed(pv, 7.35, [0.22, 0.85, 0.22], 0.0, pal.get("road_line_yellow"));
    }

    // 8. Chinese Border Defense (`中国边防`): Stepped Riverside Monument, Barbed-Wire Y-Fence & Watchtowers.
    let bs = P2::new(-22.0, 18.0);
    scene.props.boxed(bs, 0.0, [3.6, 0.30, 2.6], 0.0, dark);
    scene.props.boxed(bs, 0.30, [2.4, 0.30, 1.7], 0.0, stone);
    scene.props.boxed(bs, 0.60, [0.85, 2.75, 0.42], 0.0, white);
    scene.props.boxed(bs, 2.45, [0.46, 0.46, 0.46], 0.0, red_med);
    scene.props.boxed(bs, 1.35, [0.30, 0.90, 0.46], 0.0, red_med);
    for s in [-1.0f32, 1.0] {
        let fp = bs.add(P2::new(s * 2.5, -0.5));
        scene.props.boxed(fp, 0.0, [0.10, 7.0, 0.10], 0.0, white);
        scene.props.boxed(fp.add(P2::new(0.65, 0.0)), 6.0, [1.3, 0.82, 0.04], 0.0, red_med);
    }

    let wire_steel = Rgb { r: 0.34, g: 0.38, b: 0.35 };
    let warn_yellow = pal.get("road_line_yellow");
    let mut f_idx = 0usize;
    along(&offset(&city.bank, -1.6), 4.0, |p, dir| {
        f_idx += 1;
        if near_ice_gate(p) {
            return;
        }
        let yaw = dir.y.atan2(dir.x);
        let out_n = P2::new(-dir.y, dir.x);
        // Upright olive steel border fence post & angled barbed-wire Y-arm.
        scene.props.boxed(p, 0.0, [0.10, 2.25, 0.10], yaw, wire_steel);
        scene.props.boxed(p, 0.90, [4.04, 0.06, 0.04], yaw, wire_steel);
        scene.props.boxed(p, 1.65, [4.04, 0.06, 0.04], yaw, wire_steel);
        scene.props.beam(world(p, 2.15), world(p.add(out_n.mul(0.45)), 2.58), 0.045, iron);
        scene.props.beam(world(p, 2.15), world(p.sub(out_n.mul(0.45)), 2.58), 0.045, iron);
        scene.props.boxed(p.add(out_n.mul(0.32)), 2.48, [4.02, 0.04, 0.04], yaw, iron);
        if f_idx % 9 == 0 {
            // Border warning board (`中俄边境·严禁越界`).
            scene.props.boxed(p.sub(out_n.mul(0.08)), 1.35, [0.95, 0.55, 0.06], yaw, warn_yellow);
            scene.props.boxed(p.sub(out_n.mul(0.09)), 1.50, [0.85, 0.16, 0.07], yaw, red_med);
        }
    });

    // 2 Chinese Border Guard Watchtowers (`边防瞭望哨`).
    let olive_cab = Rgb { r: 0.24, g: 0.30, b: 0.22 };
    for approx in [P2::new(-145.0, 58.0), P2::new(215.0, -82.0)] {
        let wt = clear_park_spot(city, approx);
        let dir = nearest_bank_dir(&city.bank, wt);
        let yaw = dir.y.atan2(dir.x);
        let perp = dir.right();
        scene.props.boxed(wt, 0.0, [3.6, 0.35, 3.6], yaw, dark);
        for sx in [-1.3f32, 1.3] {
            for sz in [-1.3f32, 1.3] {
                let leg = wt.add(dir.mul(sx)).add(perp.mul(sz));
                scene.props.beam(world(leg, 0.3), world(wt.add(dir.mul(sx * 0.85)).add(perp.mul(sz * 0.85)), 5.4), 0.18, wire_steel);
            }
        }
        scene.props.boxed(wt, 5.4, [2.9, 0.25, 2.9], yaw, dark);
        scene.props.boxed(wt, 5.65, [2.5, 2.2, 2.5], yaw, olive_cab);
        scene.props.boxed(wt, 6.45, [2.58, 0.85, 2.58], yaw, pal.get("glass_day"));
        scene.lit.boxed(wt, 6.55, [2.42, 0.55, 2.62], yaw, pal.get("glass_lit"));
        scene.props.boxed(wt, 7.85, [2.9, 0.28, 2.9], yaw, white);
        scene.props.boxed(wt, 8.13, [2.8, 0.06, 2.8], yaw, snow);
        scene.props.boxed(wt.add(dir.mul(0.8)), 8.2, [0.08, 3.2, 0.08], yaw, iron);
    }

    // 9. 2 Moored Winter Hovercraft (`黑河气垫船`) parked on the Heilongjiang ice by the slipway.
    let rubber_skirt = Rgb { r: 0.13, g: 0.14, b: 0.16 };
    let navy_trim = Rgb { r: 0.14, g: 0.30, b: 0.56 };
    for (hi, hc) in [P2::new(-95.0, 86.0), P2::new(-115.0, 96.0)].into_iter().enumerate() {
        let hyaw = 0.55 + hi as f32 * 0.18;
        let f = P2::new(hyaw.cos(), hyaw.sin());
        scene.props.boxed(hc, 0.0, [11.2, 0.72, 4.6], hyaw, rubber_skirt);
        scene.props.boxed(hc, 0.72, [9.8, 0.55, 4.1], hyaw, white);
        scene.props.boxed(hc, 1.05, [9.4, 0.26, 4.14], hyaw, navy_trim);
        scene.props.boxed(hc.add(f.mul(0.6)), 1.27, [6.8, 1.45, 3.4], hyaw, white);
        scene.props.boxed(hc.add(f.mul(0.9)), 1.65, [5.8, 0.68, 3.46], hyaw, pal.get("glass_day"));
        scene.props.boxed(hc.add(f.mul(0.6)), 2.72, [6.6, 0.06, 3.3], hyaw, snow);
        // Twin stern ducted propeller shrouds & red rudders.
        for s in [-1.05f32, 1.05] {
            let perp = P2::new(-f.y, f.x);
            let rear = hc.sub(f.mul(4.1)).add(perp.mul(s));
            scene.props.boxed(rear, 1.25, [0.95, 1.65, 1.55], hyaw, navy_trim);
            scene.props.boxed(rear.sub(f.mul(0.5)), 1.35, [0.42, 1.45, 0.14], hyaw, bred);
        }
    }

    // 10. Heihe–Blagoveshchensk Cross-River Cable Car (`中俄跨江索道`) with suspended gondolas!
    let c_south = P2::new(165.0, -48.0);
    let c_north = c_south.add(river_out.mul(295.0));
    let c_dir = c_north.sub(c_south).norm();
    let c_across = P2::new(-c_dir.y, c_dir.x);
    let c_yaw = c_dir.y.atan2(c_dir.x);
    for (base, top_h) in [(c_south, 38.0f32), (c_north, 42.0f32)] {
        scene.props.boxed(base, 0.0, [9.0, 0.8, 9.0], c_yaw, dark);
        for sx in [-3.4f32, 3.4] {
            for sz in [-2.6f32, 2.6] {
                let foot = base.add(c_across.mul(sx)).add(c_dir.mul(sz));
                let head = base.add(c_across.mul(sx * 0.45));
                scene.props.beam(world(foot, 0.6), world(head, top_h), 0.38, white);
            }
        }
        scene.props.boxed(base, top_h - 1.2, [10.4, 1.1, 2.4], c_yaw, steel_arm_col(pal));
        scene.lamp_glow.boxed(base, top_h + 0.5, [0.4, 0.4, 0.4], 0.0, bred);
    }
    for s in [-3.6f32, 3.6] {
        let a = c_south.add(c_across.mul(s));
        let b = c_north.add(c_across.mul(s));
        for seg in 0..8 {
            let t0 = seg as f32 / 8.0;
            let t1 = (seg + 1) as f32 / 8.0;
            let h0 = 37.5 * (1.0 - t0) + 41.5 * t0 - 11.0 * 4.0 * t0 * (1.0 - t0);
            let h1 = 37.5 * (1.0 - t1) + 41.5 * t1 - 11.0 * 4.0 * t1 * (1.0 - t1);
            scene.props.beam(world(a.lerp(b, t0), h0), world(a.lerp(b, t1), h1), 0.09, iron);
        }
    }
    for (gt, gs) in [(0.34f32, -3.6f32), (0.72, 3.6)] {
        let gp = c_south.lerp(c_north, gt).add(c_across.mul(gs));
        let cable_y = 37.5 * (1.0 - gt) + 41.5 * gt - 11.0 * 4.0 * gt * (1.0 - gt);
        scene.props.beam(world(gp, cable_y), world(gp, cable_y - 2.6), 0.09, iron);
        scene.props.boxed(gp, cable_y - 5.2, [3.8, 2.6, 2.6], c_yaw, bred);
        scene.props.boxed(gp, cable_y - 4.5, [3.2, 1.3, 2.66], c_yaw, pal.get("glass_day"));
    }

    // 11. Blagoveshchensk Far-Bank Waterfront Landmarks (`布拉戈维申斯克天际线`).
    let gold = Rgb { r: 0.88, g: 0.72, b: 0.22 };
    let ru_blue = Rgb { r: 0.16, g: 0.30, b: 0.62 };
    let cream_wall = pal.get("far_bank_white");
    let terra_brick = pal.get("far_bank_brick");

    // 11a. Russian Embankment White Rotunda Colonnade & Tricolor Flag.
    let rot = P2::new(25.0, 0.0).add(river_out.mul(725.0));
    oct_prism(&mut scene.props, rot, 0.0, 1.2, 7.2, 7.2, dark);
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::TAU / 8.0;
        let cp = rot.add(P2::new(a.cos(), a.sin()).mul(5.4));
        scene.props.boxed(cp, 1.2, [0.45, 5.2, 0.45], 0.0, cream_wall);
    }
    scene.props.cone(rot, 6.4, 6.5, 3.2, 8, pal.get("dome_green"));
    let rfp = rot.add(P2::new(14.0, 0.0));
    scene.props.boxed(rfp, 0.0, [0.16, 14.0, 0.16], 0.0, white);
    scene.props.boxed(rfp.add(P2::new(1.5, 0.0)), 12.8, [3.0, 0.55, 0.06], 0.0, white);
    scene.props.boxed(rfp.add(P2::new(1.5, 0.0)), 12.25, [3.0, 0.55, 0.06], 0.0, ru_blue);
    scene.props.boxed(rfp.add(P2::new(1.5, 0.0)), 11.7, [3.0, 0.55, 0.06], 0.0, red_med);

    // 11b. Blagoveshchensk Orthodox Cathedral (`Благовещенский собор`: 5 golden/blue onion domes).
    let cath = P2::new(0.0, 0.0).add(river_out.mul(775.0));
    scene.props.boxed(cath, 0.0, [26.0, 18.0, 16.0], 0.0, cream_wall);
    scene.props.boxed(cath, 0.0, [16.0, 20.0, 24.0], 0.0, cream_wall);
    let mut emit_onion_dome = |at: P2, base_y: f32, r: f32, drum_h: f32, col: Rgb| {
        oct_prism(&mut scene.props, at, base_y, base_y + drum_h, r * 0.9, r * 0.9, cream_wall);
        oct_prism(&mut scene.props, at, base_y + drum_h, base_y + drum_h + r * 0.75, r * 0.95, r * 1.18, col);
        scene.props.cone(at, base_y + drum_h + r * 0.75, r * 1.18, r * 1.55, 8, col);
        scene.props.boxed(at, base_y + drum_h + r * 2.3, [0.22, 2.4, 0.22], 0.0, gold);
        scene.props.boxed(at, base_y + drum_h + r * 2.3 + 1.5, [1.1, 0.20, 0.20], 0.0, gold);
    };
    emit_onion_dome(cath, 20.0, 3.6, 8.5, gold);
    for (dx, dy) in [(-8.0f32, -6.5f32), (8.0, -6.5), (-8.0, 6.5), (8.0, 6.5)] {
        emit_onion_dome(cath.add(P2::new(dx, dy)), 18.0, 2.3, 5.5, ru_blue);
    }

    // 11c. Blagoveshchensk Triumphal Arch (`Триумфальная арка`).
    let arch = P2::new(-85.0, 45.0).add(river_out.mul(745.0));
    for sx in [-5.8f32, 5.8] {
        scene.props.boxed(arch.add(P2::new(sx, 0.0)), 0.0, [4.6, 13.5, 4.6], 0.0, terra_brick);
        scene.props.boxed(arch.add(P2::new(sx, 0.0)), 13.5, [5.0, 0.5, 5.0], 0.0, cream_wall);
    }
    scene.props.boxed(arch, 9.5, [16.5, 4.5, 4.8], 0.0, terra_brick);
    scene.props.cone(arch, 14.0, 6.2, 6.8, 4, pal.get("dome_green"));
    scene.props.boxed(arch, 20.8, [0.3, 2.5, 0.3], 0.0, gold);

    // 11d. Blagoveshchensk 160m Red-and-White TV Tower on the far bank skyline.
    let rtv = P2::new(120.0, -40.0).add(river_out.mul(860.0));
    scene.props.boxed(rtv, 0.0, [14.0, 1.0, 14.0], 0.0, dark);
    for seg in 0..8 {
        let z0 = seg as f32 * 20.0;
        let z1 = z0 + 20.0;
        let r0 = (7.2 - seg as f32 * 0.78).max(1.2);
        let r1 = (7.2 - (seg + 1) as f32 * 0.78).max(0.6);
        let col = if seg % 2 == 0 { bred } else { white };
        let corners = [P2::new(-1.0, -1.0), P2::new(1.0, -1.0), P2::new(1.0, 1.0), P2::new(-1.0, 1.0)];
        for k in 0..4 {
            let c0 = corners[k];
            let c1 = corners[(k + 1) % 4];
            scene.props.beam(world(rtv.add(c0.mul(r0)), z0), world(rtv.add(c1.mul(r1)), z1), 0.35, col);
            scene.props.beam(world(rtv.add(c0.mul(r1)), z1), world(rtv.add(c1.mul(r1)), z1), 0.25, col);
        }
    }
    scene.lamp_glow.boxed(rtv, 160.0, [0.9, 0.9, 0.9], 0.0, red_med);
}

fn steel_arm_col(pal: &Palette) -> Rgb {
    pal.get("lamp_post")
}

fn dist_to_poly_local(p: P2, line: &[P2]) -> f32 {
    let mut best = f32::MAX;
    for w in line.windows(2) {
        best = best.min(p.dist(nearest_on_segment(p, w[0], w[1])));
    }
    best
}

fn snow_piles(city: &City, pal: &Palette, props: &mut Soup) {
    // Shovelled snow stays where it was thrown, along the kerb, getting dirtier.
    for (r, road) in city.roads.iter().enumerate() {
        if road.pedestrian() || road.width < 6.0 {
            continue;
        }
        for side in [-1.0f32, 1.0] {
            let edge = offset(&road.line, side * (road.width * 0.5 + 0.5));
            let mut k = 0;
            along(&edge, 13.0, |p, _| {
                k += 1;
                let id = (r * 1000 + k) as i64;
                if hash01(id, if side > 0.0 { 8 } else { 7 }) < 0.55 {
                    let size = 0.7 + hash01(id, 2);
                    props.cone(p, 0.0, size, 0.35 + 0.3 * size, 7, pal.get("snow_dirty").mix(pal.get("snow_ground"), hash01(id, 5)));
                }
            });
        }
    }
}

fn boards(city: &City, pal: &Palette, props: &mut Soup) -> Vec<Board> {
    let mut out = Vec::new();
    let mut hung: Vec<(P2, f32, String, String, P2)> = Vec::new();
    for (i, sign) in city.signs.iter().enumerate() {
        if sign.text.is_empty() && sign.text_ru.is_empty() {
            continue;
        }
        // The wall this board belongs on: the nearest one.
        let mut best = 14.0;
        let mut place: Option<(P2, P2)> = None;
        for b in city.buildings.iter().filter(|b| b.bank == Bank::China) {
            if b.centroid.dist(sign.at) > 140.0 {
                continue;
            }
            let n = b.footprint.len();
            for k in 0..n {
                let (a, c) = (b.footprint[k], b.footprint[(k + 1) % n]);
                let q = nearest_on_segment(sign.at, a, c);
                let d = q.dist(sign.at);
                if d < best && a.dist(c) > 3.0 {
                    best = d;
                    place = Some((q, c.sub(a).norm()));
                }
            }
        }
        let (back, ink) = pal.boards[(sign.palette + i) % pal.boards.len().max(1)];
        let letters = sign.text.chars().count().max(sign.text_ru.chars().count() / 2) as f32;
        let width = (letters * 0.95 + 1.0).clamp(3.2, 9.0);
        let height = if sign.text.is_empty() || sign.text_ru.is_empty() { 0.9 } else { 1.25 };
        let on_wall = place.is_some();
        let (centre, dir, outw) = match place {
            Some((q, dir)) => (q.add(dir.right().mul(0.24)), dir, dir.right()),
            None => {
                // No wall near: a board on two posts by the road.
                let (dir, _) = nearest_road_dir(city, sign.at);
                for end in [-1.0f32, 1.0] {
                    props.boxed(sign.at.add(dir.mul(end * width * 0.45)), 0.0, [0.12, 3.0, 0.12], 0.0, pal.get("lamp_post"));
                }
                (sign.at, dir, dir.right())
            }
        };
        // Two boards never share a stretch of wall. The same shop listed twice is hung once;
        // a neighbour is slid along the wall until it is clear.
        let half = width * 0.5;
        let mut centre = centre;
        let mut twice = false;
        for _ in 0..6 {
            let mut clash: Option<(P2, f32)> = None;
            for (at, other_half, zh, ru, facing) in &hung {
                // Only boards facing the same way can be in each other's way.
                if facing.dot(outw) > 0.9 && centre.dist(*at) < half + other_half + 0.3 {
                    if *zh == sign.text && *ru == sign.text_ru {
                        twice = true;
                    }
                    clash = Some((*at, *other_half));
                    break;
                }
            }
            let Some((at, other_half)) = clash else { break };
            if twice {
                break;
            }
            let side = if centre.sub(at).dot(dir) >= 0.0 { 1.0 } else { -1.0 };
            centre = at.add(dir.mul(side * (half + other_half + 0.35)));
        }
        // Still in someone's way after all that sliding: the wall is full, leave it off.
        let crowded = hung.iter().any(|(at, other_half, _, _, facing)| facing.dot(outw) > 0.9 && centre.dist(*at) < half + other_half);
        if twice || crowded {
            continue;
        }
        hung.push((centre, half, sign.text.clone(), sign.text_ru.clone(), outw));
        let up = if on_wall { 3.55 } else { 3.42 };
        // The board itself, a little proud of the wall. The text goes on a second quad in front.
        let a = centre.sub(dir.mul(width * 0.5));
        let c = centre.add(dir.mul(width * 0.5));
        props.quad(
            [world(a, up - height * 0.5), world(c, up - height * 0.5), world(c, up + height * 0.5), world(a, up + height * 0.5)],
            world_dir(outw),
            back,
        );
        // Facing the board from the street, the wall's direction runs left to right.
        out.push(Board {
            centre: world(centre.add(outw.mul(0.03)), up),
            right: world_dir(dir),
            normal: world_dir(outw),
            width,
            height,
            zh: sign.text.clone(),
            ru: sign.text_ru.clone(),
            back,
            ink,
        });
    }
    out
}
