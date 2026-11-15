//! Walking: a person is a circle on the map, buildings are solid, the main
//! channel of the river is out of bounds. No physics engine.

use crate::city::City;
use crate::geom::{nearest_on_segment, point_in_ring, P2};
use std::collections::HashMap;

const CELL: f32 = 24.0;

pub struct Ground {
    rings: Vec<Vec<P2>>,
    /// cell -> rings whose bounding box touches it
    grid: HashMap<(i32, i32), Vec<u32>>,
    bank: Vec<P2>,
    ice: Vec<P2>,
    x_range: (f32, f32),
    south: f32,
}

fn cell(v: f32) -> i32 {
    (v / CELL).floor() as i32
}

impl Ground {
    pub fn new(city: &City) -> Ground {
        let mut rings = Vec::new();
        let mut grid: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for b in city.buildings.iter().filter(|b| b.solid) {
            let id = rings.len() as u32;
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for p in &b.footprint {
                x0 = x0.min(p.x);
                y0 = y0.min(p.y);
                x1 = x1.max(p.x);
                y1 = y1.max(p.y);
            }
            for cx in cell(x0 - 1.0)..=cell(x1 + 1.0) {
                for cy in cell(y0 - 1.0)..=cell(y1 + 1.0) {
                    grid.entry((cx, cy)).or_default().push(id);
                }
            }
            rings.push(b.footprint.clone());
        }
        Ground {
            rings,
            grid,
            bank: city.bank.clone(),
            ice: city.ice_limit.clone(),
            x_range: (city.walk_x.0 + 6.0, city.walk_x.1 - 6.0),
            south: city.walk_south + 4.0,
        }
    }

    pub fn solid_count(&self) -> usize {
        self.rings.len()
    }

    /// True when a circle of this radius at `p` overlaps no building.
    pub fn free(&self, p: P2, radius: f32) -> bool {
        self.push_out(p, radius).dist(p) < 1.0e-4
    }

    /// Where you end up if you try to go from `from` to `to`. You slide along walls.
    pub fn step(&self, from: P2, to: P2, radius: f32) -> P2 {
        // Short hops, so a fast mover cannot tunnel through a thin wall.
        let span = to.dist(from);
        let hops = (span / (radius * 0.8)).ceil().max(1.0) as usize;
        let mut at = from;
        for i in 1..=hops {
            let want = from.lerp(to, i as f32 / hops as f32);
            let delta = want.sub(from.lerp(to, (i - 1) as f32 / hops as f32));
            at = self.push_out(at.add(delta), radius);
        }
        self.keep_in(at)
    }

    fn push_out(&self, mut p: P2, radius: f32) -> P2 {
        for _ in 0..4 {
            let mut moved = false;
            let Some(ids) = self.grid.get(&(cell(p.x), cell(p.y))) else { break };
            for &id in ids {
                let ring = &self.rings[id as usize];
                let inside = point_in_ring(p, ring);
                let mut best = f32::MAX;
                let mut best_q = p;
                for i in 0..ring.len() {
                    let q = nearest_on_segment(p, ring[i], ring[(i + 1) % ring.len()]);
                    let d = p.dist(q);
                    if d < best {
                        best = d;
                        best_q = q;
                    }
                }
                if inside {
                    // Out through the nearest wall, and a radius beyond it.
                    let dir = best_q.sub(p).norm();
                    p = best_q.add(dir.mul(radius + 0.01));
                    moved = true;
                } else if best < radius {
                    let dir = p.sub(best_q).norm();
                    p = best_q.add(dir.mul(radius + 0.001));
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
        p
    }

    fn keep_in(&self, mut p: P2) -> P2 {
        p.x = p.x.clamp(self.x_range.0, self.x_range.1);
        if p.y < self.south {
            p.y = self.south;
        }
        let limit = sample(&self.ice, p.x, sample(&self.bank, p.x, 40.0) + 80.0);
        if p.y > limit {
            p.y = limit;
        }
        p
    }

    pub fn on_ice(&self, p: P2) -> bool {
        p.y > sample(&self.bank, p.x, 40.0) + 2.0
    }
}

fn sample(line: &[P2], x: f32, fallback: f32) -> f32 {
    if line.len() < 2 {
        return fallback;
    }
    if x <= line[0].x {
        return line[0].y;
    }
    let last = line[line.len() - 1];
    if x >= last.x {
        return last.y;
    }
    for w in line.windows(2) {
        if w[0].x <= x && x <= w[1].x {
            let t = (x - w[0].x) / (w[1].x - w[0].x).max(0.001);
            return w[0].y + (w[1].y - w[0].y) * t;
        }
    }
    fallback
}
