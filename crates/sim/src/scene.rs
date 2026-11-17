//! Everything in Heihe that is not a building: snow, the frozen river and its
//! embankment, streets and pavements, lamps, pollarded trees, parked cars,
//! the stone balls on Central Street, the Mother sculpture, and where each
//! shop board hangs.

use crate::city::{Bank, City};
use crate::facade;
use crate::geom::{hash01, nearest_on_segment, P2};
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
    cars(city, pal, &mut scene.props);
    central_street(city, pal, &mut scene.props);
    mother(city, pal, &mut scene.props);
    snow_piles(city, pal, &mut scene.props);
    scene.boards = boards(city, pal, &mut scene.props);
    scene
}

fn ground(city: &City, pal: &Palette, g: &mut Soup) {
    let snow = pal.get("snow_ground");
    let field = [P2::new(-1500.0, -1100.0), P2::new(1300.0, -1100.0), P2::new(1300.0, 2400.0), P2::new(-1500.0, 2400.0)];
    g.flat(&field, -0.06, snow);

    // The Heilongjiang, frozen: from the bank out past the far embankment.
    if city.bank.len() >= 2 {
        let ice = pal.get("river_ice");
        for w in city.bank.windows(2) {
            let ring = [w[0], w[1], P2::new(w[1].x, w[1].y + 760.0), P2::new(w[0].x, w[0].y + 760.0)];
            let ring: Vec<P2> = crate::geom::clean_ring(ring.to_vec());
            g.flat(&ring, -0.03, ice);
        }
        // Wind-swept lanes of bare, darker ice, and snow drifted between them.
        for (k, (off, width)) in [(70.0, 26.0), (150.0, 40.0), (255.0, 30.0), (360.0, 44.0), (500.0, 36.0)].iter().enumerate() {
            let lane = offset(&city.bank, -*off);
            g.ribbon(&lane, *width, -0.02 + k as f32 * 0.001, pal.get("river_ice_dark").mix(ice, 0.35));
        }
        for (off, width) in [(28.0, 30.0), (205.0, 34.0), (430.0, 50.0)] {
            g.ribbon(&offset(&city.bank, -off), width, -0.015, snow.mix(ice, 0.25));
        }
        // The embankment wall and the promenade behind it.
        g.ribbon(&offset(&city.bank, 5.0), 9.0, 0.02, pal.get("granite_paving"));
        g.ribbon(&offset(&city.bank, 0.4), 1.2, 0.9, pal.get("embankment"));
        g.ribbon(&offset(&city.bank, -700.0), 5.0, 2.2, pal.get("embankment"));
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
            continue;
        }
        g.ribbon(&road.line, road.width + 7.0, 0.012 + lift, brick);
        g.ribbon(&road.line, road.width, 0.04 + lift, asphalt);
        if road.width >= 8.5 {
            g.ribbon(&road.line, 0.16, 0.05 + lift, pal.get("road_line_yellow"));
        }
        // Wheel tracks worn through the snow film.
        for side in [-1.0f32, 1.0] {
            g.ribbon(&offset(&road.line, side * road.width * 0.24), 0.9, 0.045 + lift, pal.get("asphalt").mix(asphalt, 0.5));
        }
    }
    // Mother Square: stone paving around the sculpture.
    for l in city.landmarks.iter().filter(|l| l.kind == "sculpture") {
        let h = 26.0;
        let ring = [P2::new(l.at.x - h, l.at.y - h), P2::new(l.at.x + h, l.at.y - h), P2::new(l.at.x + h, l.at.y + h * 0.5), P2::new(l.at.x - h, l.at.y + h * 0.5)];
        g.flat(&ring, 0.05, pal.get("granite_paving"));
        g.flat(&[P2::new(l.at.x - 7.0, l.at.y - 7.0), P2::new(l.at.x + 7.0, l.at.y - 7.0), P2::new(l.at.x + 7.0, l.at.y + 7.0), P2::new(l.at.x - 7.0, l.at.y + 7.0)], 0.07, pal.get("granite_dark"));
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
    for (i, (at, scale)) in city.trees.iter().enumerate() {
        let id = i as i64;
        let h = 2.6 * scale;
        props.boxed(*at, 0.0, [0.42 * scale, h, 0.42 * scale], hash01(id, 1) * 1.5, bark);
        // Pollarded: a fist of stubs at the top of a thick trunk, and thin bare shoots.
        for k in 0..5 {
            let a = k as f32 * 1.256 + hash01(id, 20 + k) * 0.6;
            let reach = 0.55 + 0.35 * hash01(id, 30 + k);
            let tip = at.add(P2::new(a.cos(), a.sin()).mul(reach));
            props.boxed(tip, h - 0.2, [0.2, 0.9 + hash01(id, 40 + k), 0.2], a, bark);
            let twig = at.add(P2::new(a.cos(), a.sin()).mul(reach * 1.5));
            props.boxed(twig, h + 0.5, [0.07, 1.6 + hash01(id, 50 + k), 0.07], a, bark.mix(pal.get("snow"), 0.15));
        }
        props.boxed(*at, h + 0.02, [0.5 * scale, 0.08, 0.5 * scale], 0.0, pal.get("snow"));
    }
}

fn cars(city: &City, pal: &Palette, props: &mut Soup) {
    let paints = ["car_white", "car_white", "car_silver", "car_white", "car_black", "car_silver"];
    for (i, at) in city.cars.iter().enumerate() {
        let id = i as i64;
        let (dir, _) = nearest_road_dir(city, *at);
        let yaw = dir.y.atan2(dir.x) + std::f32::consts::FRAC_PI_2;
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

fn central_street(city: &City, pal: &Palette, props: &mut Soup) {
    // Stone balls along both edges of the pedestrian street.
    for road in city.roads.iter().filter(|r| r.name == "中央街") {
        for side in [-1.0f32, 1.0] {
            let edge = offset(&road.line, side * (road.width * 0.5 - 0.6));
            along(&edge, 9.0, |p, _| {
                props.boxed(p, 0.0, [0.5, 0.18, 0.5], 0.0, pal.get("granite_dark"));
                props.cone(p, 0.18, 0.34, 0.36, 8, pal.get("bollard_stone"));
                props.cone(p, 0.5, 0.2, 0.16, 8, pal.get("snow"));
            });
        }
    }
}

fn mother(city: &City, pal: &Palette, props: &mut Soup) {
    let bronze = pal.get("mother_bronze");
    let plinth = pal.get("mother_plinth");
    for l in city.landmarks.iter().filter(|l| l.kind == "sculpture") {
        let at = l.at;
        // A tall plinth of dark red polished granite, wider at the foot.
        props.boxed(at, 0.0, [4.4, 0.5, 4.4], 0.0, pal.get("granite_dark"));
        props.boxed(at, 0.5, [3.0, 8.5, 3.0], 0.0, plinth);
        props.boxed(at, 9.0, [3.3, 0.25, 3.3], 0.0, pal.get("granite_dark"));
        // The figure faces the river: a long skirt, a body, arms straight up, and the winged child held high.
        props.cone(at, 9.25, 1.25, 4.6, 10, bronze);
        props.boxed(at, 12.6, [0.95, 1.5, 0.6], 0.0, bronze);
        props.boxed(at, 14.1, [0.5, 0.55, 0.5], 0.0, bronze);
        for side in [-1.0f32, 1.0] {
            props.boxed(at.add(P2::new(side * 0.36, 0.0)), 13.9, [0.2, 2.1, 0.2], 0.0, bronze);
        }
        props.boxed(at, 16.0, [0.5, 0.75, 0.4], 0.0, bronze);
        for side in [-1.0f32, 1.0] {
            props.boxed(at.add(P2::new(side * 0.85, 0.0)), 16.35, [1.2, 0.5, 0.12], side * 0.5, bronze);
        }
        props.boxed(at, 9.25, [3.3, 0.04, 3.3], 0.0, pal.get("snow"));
    }
    for l in city.landmarks.iter().filter(|l| l.kind == "boundary_stone") {
        props.boxed(l.at, 0.0, [0.5, 1.3, 0.25], 0.0, pal.get("embankment"));
    }
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
        let up = 3.42;
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
