//! Walking: a person is a circle on the map, buildings are solid, the main
//! channel of the river is out of bounds. No physics engine.

use crate::city::City;
use crate::geom::{nearest_on_segment, point_in_ring, spans_inside, P2};
use crate::interior::Interior;
use std::collections::{HashMap, HashSet};

const CELL: f32 = 24.0;

pub struct Ground {
    rings: Vec<Vec<P2>>,
    ring_bids: Vec<i64>,
    /// cell -> rings whose bounding box touches it
    grid: HashMap<(i32, i32), Vec<u32>>,
    enterable: HashMap<u32, Vec<usize>>,
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
        Self::new_with_interiors(city, &[])
    }

    pub fn new_with_interiors(city: &City, interiors: &[Interior]) -> Ground {
        let mut by_bid: HashMap<i64, Vec<usize>> = HashMap::new();
        for (idx, intr) in interiors.iter().enumerate() {
            by_bid.entry(intr.building_id).or_default().push(idx);
        }

        let mut rings = Vec::new();
        let mut ring_bids = Vec::new();
        let mut grid: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        let mut enterable = HashMap::new();
        for b in city.buildings.iter().filter(|b| b.solid) {
            let id = rings.len() as u32;
            if let Some(list) = by_bid.get(&b.id) {
                enterable.insert(id, list.clone());
            }
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
            ring_bids.push(b.id);
        }
        Ground {
            rings,
            ring_bids,
            grid,
            enterable,
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
        self.push_out(&[], p, radius, 0, false).dist(p) < 1.0e-4
    }

    /// Where you end up if you try to go from `from` to `to`. You slide along walls.
    pub fn step(&self, from: P2, to: P2, radius: f32) -> P2 {
        self.step_with_interiors_on_floor(&[], from, to, radius, 0, false)
    }

    /// Walk resolution on ground floor (level 0) supporting unlocked enterable building doorways and rooms.
    pub fn step_with_interiors(&self, interiors: &[Interior], from: P2, to: P2, radius: f32, is_enemy: bool) -> P2 {
        self.step_with_interiors_on_floor(interiors, from, to, radius, 0, is_enemy)
    }

    /// Multi-story walk resolution across ground-floor doorways and upper enclosed stories (`floor >= 1`).
    pub fn step_with_interiors_on_floor(
        &self,
        interiors: &[Interior],
        from: P2,
        to: P2,
        radius: f32,
        floor: u8,
        is_enemy: bool,
    ) -> P2 {
        let span = to.dist(from);
        let hops = (span / (radius * 0.8)).ceil().max(1.0) as usize;
        let mut at = from;
        for i in 1..=hops {
            let want = from.lerp(to, i as f32 / hops as f32);
            let delta = want.sub(from.lerp(to, (i - 1) as f32 / hops as f32));
            at = self.push_out(interiors, at.add(delta), radius, floor, is_enemy);
        }
        self.keep_in(at)
    }

    fn push_out(&self, interiors: &[Interior], mut p: P2, radius: f32, floor: u8, _is_enemy: bool) -> P2 {
        for _ in 0..4 {
            let mut moved = false;
            let Some(ids) = self.grid.get(&(cell(p.x), cell(p.y))) else { break };
            let active_intr_ring: Option<(u32, usize)> = ids.iter().find_map(|&id| {
                self.enterable.get(&id).and_then(|list| {
                    list.iter().copied().find_map(|idx| {
                        interiors.get(idx).and_then(|intr| {
                            let ok = if floor == 0 {
                                intr.contains(p) || intr.door_passage(p, radius, false)
                            } else {
                                intr.num_floors > floor && intr.contains_padded(p, 0.55)
                            };
                            ok.then_some((id, idx))
                        })
                    })
                })
            });
            for &id in ids {
                if active_intr_ring.is_some_and(|(aid, _)| aid != id) {
                    // Do not let adjacent row-house footprints overlap/block an enterable room or doorway.
                    continue;
                }
                if let Some(list) = self.enterable.get(&id) {
                    let ring = &self.rings[id as usize];
                    let chosen_idx = active_intr_ring.filter(|&(aid, _)| aid == id).map(|(_, i)| i).or_else(|| {
                        if floor == 0 {
                            list.iter().copied().find(|&idx| {
                                interiors.get(idx).is_some_and(|intr| {
                                    intr.contains(p)
                                        || intr.door_passage(p, radius, false)
                                        || (point_in_ring(p, ring) && intr.contains_padded(p, 0.45))
                                })
                            })
                        } else {
                            list.iter().copied().find(|&idx| {
                                interiors
                                    .get(idx)
                                    .is_some_and(|intr| intr.num_floors > floor && intr.contains_padded(p, 0.55))
                            })
                        }
                    });
                    if let Some(intr_idx) = chosen_idx {
                        if let Some(intr) = interiors.get(intr_idx) {
                            let resolved = intr.step_inside_floor(p, radius, floor);
                            if resolved.dist(p) > 1.0e-4 {
                                p = resolved;
                                moved = true;
                            }
                            continue;
                        }
                    }
                }

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

    /// True when point `p` lies inside the playable Chinese landmass bounds (south of the frozen river bank).
    pub fn in_playable_land(&self, p: P2) -> bool {
        p.x >= self.x_range.0 + 0.5
            && p.x <= self.x_range.1 - 0.5
            && p.y >= self.south + 0.5
            && p.y <= sample(&self.bank, p.x, 40.0) - 1.5
    }

    /// Snap `p` onto the closest walkable playable street/courtyard spot within `max_search` metres.
    pub fn snap_walkable(&self, p: P2, radius: f32, max_search: f32) -> Option<P2> {
        if self.in_playable_land(p) && self.free(p, radius) {
            return Some(p);
        }
        let mut r = 0.6f32;
        while r <= max_search {
            for k in 0..16 {
                let ang = k as f32 * (std::f32::consts::TAU / 16.0);
                let q = P2::new(p.x + ang.cos() * r, p.y + ang.sin() * r);
                if self.in_playable_land(q) && self.free(q, radius) {
                    return Some(q);
                }
            }
            r += 0.6;
        }
        None
    }

    /// True when no solid building stands between `a` and `b`.
    pub fn clear_line(&self, a: P2, b: P2) -> bool {
        self.clear_line_with_interiors(&[], a, b, true)
    }

    /// Line-of-sight & gunfire trace accounting for enterable windows, partition walls, closed doors, and curtains.
    pub fn clear_line_with_interiors(&self, interiors: &[Interior], a: P2, b: P2, ignore_curtains: bool) -> bool {
        self.trace_clear(interiors, None, a, b, ignore_curtains)
    }

    /// Fire trace allowing shots fired from inside `shooter_intr` toward any outdoor target
    /// to pass cleanly out of the shooter's building (windows/balconies/rooftops) while still
    /// colliding against all other solid buildings along the street.
    pub fn clear_shot_from_interior(
        &self,
        interiors: &[Interior],
        shooter_intr: Option<usize>,
        from: P2,
        to: P2,
    ) -> bool {
        let skip_bid = shooter_intr
            .and_then(|i| interiors.get(i))
            .filter(|intr| !intr.contains(to))
            .map(|intr| intr.building_id);
        self.trace_clear(interiors, skip_bid, from, to, true)
    }

    fn trace_clear(
        &self,
        interiors: &[Interior],
        skip_bid: Option<i64>,
        a: P2,
        b: P2,
        ignore_curtains: bool,
    ) -> bool {
        let len = a.dist(b);
        if len < 0.05 {
            return true;
        }
        let steps = (len / 8.0).ceil().max(1.0) as usize;
        let mut checked: HashSet<u32> = HashSet::new();
        for s in 0..=steps {
            let p = a.lerp(b, s as f32 / steps as f32);
            let (cx, cy) = (cell(p.x), cell(p.y));
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let Some(ids) = self.grid.get(&(cx + dx, cy + dy)) else { continue };
                    for &id in ids {
                        if !checked.insert(id) {
                            continue;
                        }
                        if skip_bid == Some(self.ring_bids[id as usize]) {
                            continue;
                        }
                        let spans = spans_inside(a, b, &self.rings[id as usize]);
                        if spans.is_empty() {
                            continue;
                        }
                        if spans.len() == 1 && !interiors.is_empty() {
                            if let Some(list) = self.enterable.get(&id) {
                                let p0 = a.lerp(b, spans[0].0);
                                let p1 = a.lerp(b, spans[0].1);
                                let mut pass = false;
                                for &intr_idx in list {
                                    if let Some(intr) = interiors.get(intr_idx) {
                                        if (intr.contains(a) || intr.contains(b))
                                            && intr.contains_padded(p0, 0.55)
                                            && intr.contains_padded(p1, 0.55)
                                            && intr.clear_line_inside(a, b, ignore_curtains)
                                        {
                                            pass = true;
                                            break;
                                        }
                                    }
                                }
                                if pass {
                                    continue;
                                }
                            }
                        }
                        return false;
                    }
                }
            }
        }
        true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_building_blocks_sight_and_an_open_street_does_not() {
        let (city, _) = crate::load_heihe().unwrap();
        assert!(!city.heat.is_empty(), "city.json heat points missing");
        assert!(!city.paths.is_empty(), "city.json pedestrian paths missing");
        let ground = Ground::new(&city);
        let b = city.buildings.iter().find(|b| b.solid && b.footprint.len() >= 4).unwrap();
        let c = b.centroid;
        // Pick an axis across the building's centroid out to 60 m beyond it on both sides.
        let mut dir = b.footprint[1].sub(b.footprint[0]).right().norm();
        if dir.len() < 0.5 {
            dir = P2::new(1.0, 0.0);
        }
        let a = c.sub(dir.mul(40.0));
        let d = c.add(dir.mul(40.0));
        assert!(!ground.clear_line(a, d), "line straight through building {} must be blocked", b.id);

        // Open Central Street near spawn should have clear sight along the street.
        let central = city.road("中央街").unwrap();
        let p0 = central.line[0];
        let p1 = p0.add(central.line[1].sub(p0).norm().mul(30.0));
        assert!(ground.clear_line(p0, p1), "open stretch of Central Street must have clear sight");
    }
}
