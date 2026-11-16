//! Filling the town in from CMAB, the national building dataset (CC BY 4.0).
//!
//! OpenStreetMap has 198 buildings on the Chinese bank of this map and leaves
//! whole blocks empty. CMAB traces every roof in China from imagery. Its
//! outlines are good, its heights are a model's guess. The rules here:
//!
//! - CMAB is moved 2 m west and 6 m north first: that is how far its roofs
//!   sit from OpenStreetMap's buildings and streets in this layer. Two checks
//!   measured it several independent ways and put the east-west part between
//!   1.5 and 3 m; 2 m does as well as either (out/check/position, edges and
//!   their refutations).
//! - Every outer ring of a CMAB record is a roof of its own. Some records hold
//!   two separate buildings; holes (courtyards) are left filled.
//! - An OpenStreetMap building always wins, both ways: a CMAB roof with 30% or
//!   more of it under OSM buildings, or that takes in 30% or more of one, is dropped.
//! - A CMAB roof is dropped when a street or footpath runs 10 m or more through
//!   it, when more than a quarter of it is on a carriageway, when its centre is
//!   on the river, when it reaches the promenade or covers a landmark or the
//!   Mother Square, or when it is under 20 m2 or narrower than 2.8 m.
//! - Height comes from CMAB, corrected the way `height_rule` says and why.
//! - OpenStreetMap's heights on this bank were never measured: the importer
//!   guessed them from footprint area and drew towers by a hash of the id.
//!   Where one CMAB roof matches an OSM building, CMAB's height is used
//!   instead; the hashed towers that CMAB cannot confirm come down to 7 storeys.

use crate::city::{Bank, Building, City};
use crate::geom::{clean_ring, dist_to_polyline, length_inside, point_in_ring, signed_area, simplify_safely, P2};
use serde::Deserialize;

#[derive(Deserialize)]
struct Raw {
    meta: serde_json::Value,
    buildings: Vec<RawBuilding>,
}

#[derive(Deserialize)]
struct RawBuilding {
    id: i64,
    rings: Vec<Vec<[f32; 2]>>,
    height_m: Option<f32>,
    #[serde(default)]
    function: serde_json::Value,
}

/// One outer ring of a CMAB record.
struct Roof<'a> {
    id: i64,
    points: Vec<P2>,
    raw: &'a RawBuilding,
}

/// What happened to each CMAB roof, so the numbers can be checked.
#[derive(Debug, Default, Clone)]
pub struct Tally {
    pub read: usize,
    pub kept: usize,
    pub under_osm: usize,
    pub on_street: usize,
    pub on_ice: usize,
    pub too_small: usize,
    pub bad_shape: usize,
    /// On the promenade the game paves along the bank, or over a landmark or its square.
    pub on_riverfront: usize,
    /// OpenStreetMap buildings whose guessed height was corrected.
    pub osm_heights: usize,
    /// Guessed OpenStreetMap buildings one CMAB roof matches (their storeys may not change).
    pub osm_matched: usize,
    /// Hashed OpenStreetMap towers no CMAB roof confirms, brought down to 7 storeys.
    pub osm_capped: usize,
}

pub struct Cmab {
    pub source: String,
    buildings: Vec<RawBuilding>,
}

/// IDs of CMAB buildings start here, well clear of OpenStreetMap way IDs in this slice.
pub const ID_BASE: i64 = 9_000_000_000_000;

pub const MIN_AREA: f32 = 20.0;
/// Added to every CMAB point: CMAB in this layer sits about 2 m east and 6 m south of OpenStreetMap.
pub const SHIFT: P2 = P2::new(-2.0, 6.0);
/// A roof narrower than this is a sliver traced in error. The widest such
/// artefact in this slice is 2.59 m; the narrowest real roof (a row of sheds) 3.05 m.
pub const MIN_WIDTH: f32 = 2.8;
/// Separate parts of one CMAB record get IDs this far apart.
pub const PART: i64 = 1_000_000_000;
/// The game paves the promenade 0.5 to 9.5 m south of the bank line (scene.rs).
pub const PROMENADE: f32 = 9.5;
/// Outlines are simplified to this many metres (out/check/code: 0.3 m changes no
/// kept roof's area by more than 1.7%; 0.5 m by up to 4.1%).
pub const SIMPLIFY: f32 = 0.3;
/// A street centreline this long inside a roof means the roof is not a building.
pub const THROUGH: f32 = 10.0;
/// A CMAB roof this much under OpenStreetMap buildings is the same building.
pub const OSM_COVER: f32 = 0.3;

impl Cmab {
    pub fn from_json(source: &str) -> Result<Cmab, String> {
        let mut raw: Raw = serde_json::from_str(source).map_err(|e| format!("cmab.json: {e}"))?;
        let src = raw.meta.get("source").and_then(|s| s.as_str()).unwrap_or("CMAB").to_string();
        for b in &mut raw.buildings {
            for ring in &mut b.rings {
                for p in ring.iter_mut() {
                    p[0] += SHIFT.x;
                    p[1] += SHIFT.y;
                }
            }
        }
        Ok(Cmab { source: src, buildings: raw.buildings })
    }

    pub fn len(&self) -> usize {
        self.buildings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buildings.is_empty()
    }

    /// Every roof in the file: each outer ring of each record. In the shapefile
    /// an outer ring runs clockwise and a hole counter-clockwise.
    fn roofs(&self) -> Vec<Roof<'_>> {
        let mut out = Vec::new();
        for raw in &self.buildings {
            for (k, ring) in raw.rings.iter().enumerate() {
                let points: Vec<P2> = ring.iter().map(|p| P2::new(p[0], p[1])).collect();
                if k > 0 && signed_area(&points) >= 0.0 {
                    continue;
                }
                out.push(Roof { id: raw.id + k as i64 * PART, points, raw });
            }
        }
        out
    }

    /// Add the CMAB buildings the rules keep to `city`.
    pub fn fill(&self, city: &mut City) -> Tally {
        let roofs = self.roofs();
        let mut tally = Tally { read: roofs.len(), ..Tally::default() };
        let osm: Vec<(Vec<P2>, (f32, f32, f32, f32))> = city
            .buildings
            .iter()
            .filter(|b| b.bank == Bank::China)
            .map(|b| (b.footprint.clone(), bounds(&b.footprint)))
            .collect();
        let osm_samples: Vec<Vec<P2>> = osm.iter().map(|(r, _)| inside_points(r, 2.0)).collect();
        let streets: Vec<(Vec<P2>, f32)> = city.roads.iter().filter(|r| r.width >= 4.0).map(|r| (r.line.clone(), r.width * 0.5)).collect();
        // Lines a building cannot stand across: carriageways and the paths the game draws.
        let through: Vec<Vec<P2>> = city
            .roads
            .iter()
            .filter(|r| (r.width >= 4.0 && !r.pedestrian()) || r.pedestrian())
            .map(|r| r.line.clone())
            .collect();
        let mut added = Vec::new();
        for roof in &roofs {
            let raw = roof.raw;
            let ring = clean_ring(roof.points.clone());
            if ring.len() < 3 {
                tally.bad_shape += 1;
                continue;
            }
            // CMAB traces a roof as a long chain of centimetre steps. Walls are drawn per
            // edge and edges under 0.3 m get none, so the steps would leave see-through gaps.
            // A speck that simplifies away to nothing was too small to be a building.
            let Some(ring) = simplify_safely(&ring, SIMPLIFY) else {
                tally.bad_shape += 1;
                continue;
            };
            if ring.len() < 3 || signed_area(&ring) < MIN_AREA || min_width(&ring) < MIN_WIDTH {
                tally.too_small += 1;
                continue;
            }
            let area = signed_area(&ring);
            let centre = centroid(&ring);
            if centre.y > city.bank_y(centre.x) - 3.0 {
                tally.on_ice += 1;
                continue;
            }
            let samples = inside_points(&ring, 2.0);
            if on_riverfront(&ring, &samples, city) {
                tally.on_riverfront += 1;
                continue;
            }
            if covered_by(&samples, &osm) >= OSM_COVER || swallows_osm(&ring, &osm, &osm_samples) {
                tally.under_osm += 1;
                continue;
            }
            if on_street(&samples, &streets) || through.iter().any(|line| length_inside(line, &ring) >= THROUGH) {
                tally.on_street += 1;
                continue;
            }
            let function = function_of(&raw.function);
            let (levels, floor_h) = height_rule(raw.height_m, area, &function);
            added.push(Building {
                id: ID_BASE + roof.id,
                bank: Bank::China,
                levels,
                height: levels as f32 * floor_h,
                floor_h,
                name_zh: String::new(),
                name_ru: String::new(),
                footprint: ring,
                centroid: centre,
                // Solid everywhere, like every OpenStreetMap building on this bank: a
                // roof judged by its centre alone left five you could walk into.
                solid: true,
                function,
                height_guessed: false,
            });
            tally.kept += 1;
        }
        (tally.osm_heights, tally.osm_matched, tally.osm_capped) = self.correct_osm_heights(city);
        city.buildings.extend(added);
        tally
    }

    /// Give guessed OpenStreetMap heights CMAB's reading where one CMAB roof
    /// matches the building; bring the hashed towers down where none does.
    /// Returns how many changed, how many one roof matched, and how many towers were capped.
    fn correct_osm_heights(&self, city: &mut City) -> (usize, usize, usize) {
        let roofs: Vec<(Vec<P2>, (f32, f32, f32, f32), f32, Option<f32>, String)> = self
            .roofs()
            .into_iter()
            .filter_map(|roof| {
                let ring = clean_ring(roof.points);
                if ring.len() < 3 {
                    return None;
                }
                let b = bounds(&ring);
                let area = signed_area(&ring);
                Some((ring, b, area, roof.raw.height_m, function_of(&roof.raw.function)))
            })
            .collect();
        let (mut changed, mut matched, mut capped) = (0, 0, 0);
        for b in city.buildings.iter_mut().filter(|b| b.bank == Bank::China && b.height_guessed) {
            let samples = inside_points(&b.footprint, 2.0);
            let area = signed_area(&b.footprint);
            let mut hits: Vec<usize> = Vec::new();
            for p in &samples {
                if let Some(k) = roofs.iter().position(|(r, bb, _, _, _)| p.x >= bb.0 && p.x <= bb.2 && p.y >= bb.1 && p.y <= bb.3 && point_in_ring(*p, r)) {
                    hits.push(k);
                }
            }
            let mut main = None;
            let mut best = 0;
            for &k in &hits {
                let n = hits.iter().filter(|&&j| j == k).count();
                if n > best {
                    best = n;
                    main = Some(k);
                }
            }
            let before = (b.levels, b.floor_h);
            // One CMAB roof covering at least half of the building, and not much bigger
            // than it: the same building seen twice. Then that roof's reading is used,
            // with the smaller of the two areas, so a small building keeps the small-roof rule.
            let own = best as f32 / samples.len().max(1) as f32;
            match main {
                Some(k) if own >= 0.5 && roofs[k].2 <= 2.0 * area => {
                    matched += 1;
                    let (levels, floor_h) = height_rule(roofs[k].3, area.min(roofs[k].2), &roofs[k].4);
                    b.levels = levels;
                    b.floor_h = floor_h;
                }
                _ if b.levels >= 11 => {
                    // A tower drawn by the importer's hash that no CMAB roof confirms.
                    capped += 1;
                    b.levels = 7;
                    b.floor_h = 3.0;
                }
                _ => {}
            }
            b.height = b.levels as f32 * b.floor_h;
            if (b.levels, b.floor_h) != before {
                changed += 1;
            }
        }
        (changed, matched, capped)
    }
}

/// What CMAB says a building is for. Kept as text: the dataset's labels.
pub fn function_of(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// Storeys and storey height for a building CMAB reads as `height_m` metres.
///
/// Worked out by the data check of 16 November 2026 (out/check/heights, and
/// its refutation in out/check/refute-heights), against the Beijing and
/// Shanghai buildings where both CMAB and a measured height exist:
/// - CMAB's median here, 20.7 m, is the six-to-seven-storey block the
///   photographs show, at 3 m a storey.
/// - Small roofs get ordinary heights from CMAB (under 20 m2: median 23 m), so
///   below 120 m2 the area decides: one storey, then two.
/// - Between 13 and 33 m CMAB cannot tell four storeys from seven, so those
///   become the town's 5 to 7, less 2 m for the parapet and a taller shop floor.
/// - 33 to 47 m: in Shanghai's measured pairs the median is 8 storeys, but only
///   about a third are 7 to 10 and about four in ten are 11 or more; Beijing has
///   8 pairs, median 5. Capping the band at 7 to 10 lowered the mean error against
///   those pairs (Shanghai 8.87 to 8.25 storeys, out/check/refute-heights/r5b.json)
///   and keeps the town low. It is a choice, not a reading. From 47 m it is a tower.
/// - Roofs of 120 to 150 m2 that CMAB reads at 13 m or more stay at 5 to 7
///   storeys: a roof that small is not a tower here. Below 13 m they follow the
///   low rule like any other roof.
/// - Big shops and public buildings have taller storeys: 4 m.
pub fn height_rule(height_m: Option<f32>, area: f32, function: &str) -> (u32, f32) {
    if area < 60.0 {
        return (1, 3.0);
    }
    if area < 120.0 {
        return (2, 3.0);
    }
    let Some(h) = height_m.filter(|h| *h > 1.0) else {
        return (6, 3.0);
    };
    if (function == "Business" || function == "Public service") && area >= 1500.0 {
        return ((((h - 2.0) / 4.0).round() as u32).clamp(2, 6), 4.0);
    }
    let levels = if h < 13.0 {
        ((h / 3.0).round() as u32).max(2)
    } else if h < 33.0 || area < 150.0 {
        (((h - 2.0) / 3.0).round() as u32).clamp(5, 7)
    } else if h < 47.0 {
        (7 + ((h - 33.0) / 4.0).round() as u32).clamp(7, 10)
    } else {
        (((h + 1.0) / 3.0).round() as u32).clamp(11, 18)
    };
    (levels, 3.0)
}

fn bounds(ring: &[P2]) -> (f32, f32, f32, f32) {
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in ring {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    (x0, y0, x1, y1)
}

pub fn centroid(ring: &[P2]) -> P2 {
    let mut a = 0.0;
    let mut cx = 0.0;
    let mut cy = 0.0;
    for i in 0..ring.len() {
        let p = ring[i];
        let q = ring[(i + 1) % ring.len()];
        let k = p.cross(q);
        a += k;
        cx += (p.x + q.x) * k;
        cy += (p.y + q.y) * k;
    }
    if a.abs() < 1.0e-6 {
        return ring[0];
    }
    P2::new(cx / (3.0 * a), cy / (3.0 * a))
}

/// Points on a grid inside the ring, at least one.
pub fn inside_points(ring: &[P2], step: f32) -> Vec<P2> {
    let (x0, y0, x1, y1) = bounds(ring);
    let mut out = Vec::new();
    let mut y = y0 + step * 0.5;
    while y < y1 {
        let mut x = x0 + step * 0.5;
        while x < x1 {
            let p = P2::new(x, y);
            if point_in_ring(p, ring) {
                out.push(p);
            }
            x += step;
        }
        y += step;
    }
    if out.is_empty() {
        out.push(centroid(ring));
    }
    out
}

/// The share of `samples` that fall inside any of `rings`.
fn covered_by(samples: &[P2], rings: &[(Vec<P2>, (f32, f32, f32, f32))]) -> f32 {
    let mut hit = 0;
    for p in samples {
        if rings.iter().any(|(r, b)| p.x >= b.0 && p.x <= b.2 && p.y >= b.1 && p.y <= b.3 && point_in_ring(*p, r)) {
            hit += 1;
        }
    }
    hit as f32 / samples.len().max(1) as f32
}

/// True when the roof stands on the promenade or the ice along the bank, or
/// over a landmark or the square around the Mother sculpture.
fn on_riverfront(ring: &[P2], samples: &[P2], city: &City) -> bool {
    if city.landmarks.iter().any(|l| point_in_ring(l.at, ring)) {
        return true;
    }
    for l in city.landmarks.iter().filter(|l| l.kind == "sculpture") {
        // The paving scene.rs lays around it.
        if samples.iter().any(|p| p.x > l.at.x - 26.0 && p.x < l.at.x + 26.0 && p.y > l.at.y - 26.0 && p.y < l.at.y + 13.0) {
            return true;
        }
    }
    let (Some(first), Some(last)) = (city.bank.first(), city.bank.last()) else { return false };
    // The outline's own corners as well as the 2 m samples, so no wall reaches the paving.
    samples.iter().chain(ring).any(|p| p.x >= first.x && p.x <= last.x && (p.y > city.bank_y(p.x) || dist_to_polyline(*p, &city.bank) < PROMENADE))
}

/// The width of a ring: the least extent across it, over its own edge directions.
pub fn min_width(ring: &[P2]) -> f32 {
    let n = ring.len();
    let mut best = f32::MAX;
    for i in 0..n {
        let d = ring[(i + 1) % n].sub(ring[i]);
        if d.len() < 1.0e-6 {
            continue;
        }
        let across = d.norm().right();
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for p in ring {
            let t = p.sub(ring[i]).dot(across);
            lo = lo.min(t);
            hi = hi.max(t);
        }
        best = best.min(hi - lo);
    }
    best
}

/// True when the roof takes in most of an OpenStreetMap building: the same block
/// drawn twice, seen from the OSM side.
fn swallows_osm(ring: &[P2], osm: &[(Vec<P2>, (f32, f32, f32, f32))], osm_samples: &[Vec<P2>]) -> bool {
    let rb = bounds(ring);
    osm.iter().zip(osm_samples).any(|((_, b), samples)| {
        if b.2 < rb.0 || b.0 > rb.2 || b.3 < rb.1 || b.1 > rb.3 {
            return false;
        }
        let inside = samples.iter().filter(|p| point_in_ring(**p, ring)).count();
        inside as f32 / samples.len().max(1) as f32 >= OSM_COVER
    })
}

/// True when a real share of the roof sits on a street's carriageway.
fn on_street(samples: &[P2], streets: &[(Vec<P2>, f32)]) -> bool {
    let mut hit = 0;
    for p in samples {
        if streets.iter().any(|(line, half)| dist_to_polyline(*p, line) < *half) {
            hit += 1;
        }
    }
    hit as f32 / samples.len().max(1) as f32 > 0.25
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_roofs_go_by_area_and_the_middle_becomes_the_town() {
        assert_eq!(height_rule(Some(41.8), 66.0, "Residence"), (2, 3.0));
        assert_eq!(height_rule(Some(23.0), 40.0, "Residence"), (1, 3.0));
        // 13 to 33 m: five to seven storeys.
        assert_eq!(height_rule(Some(13.5), 600.0, "Residence"), (5, 3.0));
        assert_eq!(height_rule(Some(20.7), 600.0, "Residence"), (6, 3.0));
        assert_eq!(height_rule(Some(32.9), 600.0, "Residence"), (7, 3.0));
        // No cliff at 33 m: 32.96 and 33.23 m are a storey apart at most.
        let (a, _) = height_rule(Some(32.96), 600.0, "Residence");
        let (b, _) = height_rule(Some(33.23), 600.0, "Residence");
        assert!(b - a <= 1);
        // 33 to 47 m: seven to ten. From 47 m: towers.
        assert_eq!(height_rule(Some(41.3), 600.0, "Residence"), (9, 3.0));
        assert_eq!(height_rule(Some(51.2), 289.0, "Residence"), (17, 3.0));
        // A big shop has taller storeys.
        assert_eq!(height_rule(Some(38.1), 8068.0, "Business"), (6, 4.0));
        // Low readings stay low.
        assert_eq!(height_rule(Some(8.1), 500.0, "Residence"), (3, 3.0));
        assert_eq!(height_rule(None, 500.0, ""), (6, 3.0));
    }

    // Each rule on a small made-up town, and the real numbers pinned (review, 16 November 2026).
    use crate::city::{Landmark, Road};

    /// A rectangle as the shapefile has it: clockwise, closed.
    fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<[f32; 2]> {
        vec![[x0, y0], [x0, y1], [x1, y1], [x1, y0], [x0, y0]]
    }

    /// CMAB records given where their roofs should end up, after the shift.
    fn cmab_at(records: Vec<(i64, Vec<Vec<[f32; 2]>>, f32, &str)>) -> Cmab {
        let buildings: Vec<serde_json::Value> = records
            .into_iter()
            .map(|(id, rings, h, f)| {
                let rings: Vec<Vec<[f32; 2]>> = rings.into_iter().map(|r| r.into_iter().map(|p| [p[0] - SHIFT.x, p[1] - SHIFT.y]).collect()).collect();
                serde_json::json!({"id": id, "rings": rings, "height_m": h, "function": f})
            })
            .collect();
        Cmab::from_json(&serde_json::json!({"meta": {}, "buildings": buildings}).to_string()).unwrap()
    }

    /// The bank along y = 0, the town to the south.
    fn town() -> City {
        City { bank: vec![P2::new(-1000.0, 0.0), P2::new(1000.0, 0.0)], walk_x: (-1000.0, 1000.0), walk_south: -1000.0, ..City::default() }
    }

    fn kept_ids(city: &City) -> Vec<i64> {
        let mut v: Vec<i64> = city.buildings.iter().filter(|b| b.id >= ID_BASE).map(|b| b.id - ID_BASE).collect();
        v.sort();
        v
    }

    fn sq(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<P2> {
        vec![P2::new(x0, y0), P2::new(x1, y0), P2::new(x1, y1), P2::new(x0, y1)]
    }

    fn osm(id: i64, ring: Vec<P2>, levels: u32, guessed: bool) -> Building {
        Building {
            id,
            bank: Bank::China,
            levels,
            height: levels as f32 * 3.0,
            floor_h: 3.0,
            name_zh: String::new(),
            name_ru: String::new(),
            centroid: centroid(&ring),
            footprint: ring,
            solid: !false,
            function: String::new(),
            height_guessed: guessed,
        }
    }

    #[test]
    fn cmab_moves_two_west_and_six_north() {
        let raw = r#"{"meta":{},"buildings":[{"id":1,"rings":[[[0,-100],[0,-90],[10,-90],[10,-100],[0,-100]]],"height_m":20}]}"#;
        let mut city = town();
        Cmab::from_json(raw).unwrap().fill(&mut city);
        let b = &city.buildings[0];
        let x0 = b.footprint.iter().map(|p| p.x).fold(f32::MAX, f32::min);
        let y0 = b.footprint.iter().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!((x0 + 2.0).abs() < 1e-3 && (y0 + 94.0).abs() < 1e-3, "{x0} {y0}");
    }

    #[test]
    fn slivers_and_specks_go_and_sheds_stay() {
        let mut city = town();
        let cmab = cmab_at(vec![
            (1, vec![rect(0.0, -100.0, 30.0, -97.4)], 20.0, ""),
            (2, vec![rect(50.0, -100.0, 60.0, -96.95)], 20.0, ""),
            (3, vec![rect(100.0, -100.0, 104.0, -95.5)], 20.0, ""),
            (4, vec![rect(150.0, -100.0, 155.0, -95.0)], 20.0, ""),
        ]);
        let t = cmab.fill(&mut city);
        assert_eq!(kept_ids(&city), vec![2, 4], "{t:?}");
    }

    #[test]
    fn the_promenade_landmarks_and_mother_square_stay_open() {
        let mut city = town();
        city.landmarks.push(Landmark { kind: "sculpture".into(), name_zh: "母亲".into(), at: P2::new(0.0, -150.0) });
        city.landmarks.push(Landmark { kind: "church".into(), name_zh: "x".into(), at: P2::new(500.0, -300.0) });
        let cmab = cmab_at(vec![
            (1, vec![rect(100.0, -30.0, 130.0, -8.0)], 20.0, ""),
            // 9.7 m from the bank: kept at 9.5, dropped if the promenade grew to 9.8.
            (2, vec![rect(200.0, -30.8, 230.0, -9.7)], 20.0, ""),
            (3, vec![rect(20.0, -170.0, 40.0, -150.0)], 20.0, ""),
            (4, vec![rect(30.0, -170.0, 50.0, -150.0)], 20.0, ""),
            (5, vec![rect(490.0, -310.0, 510.0, -290.0)], 20.0, ""),
        ]);
        let t = cmab.fill(&mut city);
        assert_eq!(kept_ids(&city), vec![2, 4], "{t:?}");
    }

    #[test]
    fn a_roof_on_the_river_goes_beyond_the_bank_line_too() {
        // Past the end of the bank line only the river rule applies: centre within 3 m of the
        // bank (clamped to its last point) is on the ice (review, 16 November 2026).
        let mut city = town();
        let cmab = cmab_at(vec![
            (1, vec![rect(1100.0, -6.0, 1110.0, 2.0)], 20.0, ""),
            (2, vec![rect(1200.0, -8.0, 1210.0, 0.0)], 20.0, ""),
        ]);
        let t = cmab.fill(&mut city);
        assert_eq!(kept_ids(&city), vec![2], "{t:?}");
        assert_eq!(t.on_ice, 1, "{t:?}");
    }

    #[test]
    fn one_roof_not_a_blend_and_the_smaller_area() {
        // The rules the review changed: one roof must cover half of a building, not several
        // together, and the smaller of the two areas decides (review, 16 November 2026).
        let mut city = town();
        // 144 m2, a third under one tall roof and more under another: half covered, by no one roof.
        city.buildings.push(osm(1, sq(0.0, -112.0, 12.0, -100.0), 2, true));
        // 220 m2 under a 148.5 m2 roof: the roof's area puts it in the 120-150 band.
        city.buildings.push(osm(2, sq(100.0, -111.0, 120.0, -100.0), 5, true));
        // 96 m2 under a 170 m2 roof: the building's area keeps it small.
        city.buildings.push(osm(3, sq(200.0, -112.0, 208.0, -100.0), 3, true));
        let cmab = cmab_at(vec![
            (1, vec![rect(0.0, -112.0, 4.9, -100.0)], 41.3, ""),
            (2, vec![rect(4.9, -112.0, 8.5, -100.0)], 41.3, ""),
            (3, vec![rect(100.0, -111.0, 113.5, -100.0)], 35.05, ""),
            (4, vec![rect(200.0, -112.0, 214.2, -100.0)], 22.7, ""),
        ]);
        let t = cmab.fill(&mut city);
        let levels: Vec<u32> = city.buildings.iter().filter(|b| b.id < ID_BASE).map(|b| b.levels).collect();
        assert_eq!(levels, vec![2, 7, 2], "{t:?}");
        assert_eq!(t.osm_matched, 2, "{t:?}");
    }

    #[test]
    fn a_street_through_a_roof_or_under_it_drops_the_roof() {
        let mut city = town();
        let road = |kind: &str, w: f32, a: P2, b: P2| Road { name: String::new(), name_ru: String::new(), kind: kind.into(), width: w, line: vec![a, b] };
        city.roads.push(road("residential", 6.0, P2::new(-50.0, -115.0), P2::new(50.0, -115.0)));
        city.roads.push(road("footway", 3.0, P2::new(150.0, -150.0), P2::new(150.0, -50.0)));
        city.roads.push(road("primary", 12.0, P2::new(250.0, -200.0), P2::new(350.0, -200.0)));
        let cmab = cmab_at(vec![
            (1, vec![rect(-15.0, -130.0, 15.0, -100.0)], 20.0, ""),
            (2, vec![rect(135.0, -115.0, 165.0, -85.0)], 20.0, ""),
            (3, vec![rect(290.0, -211.0, 310.0, -201.0)], 20.0, ""),
            (4, vec![rect(400.0, -130.0, 430.0, -100.0)], 20.0, ""),
        ]);
        let t = cmab.fill(&mut city);
        assert_eq!(kept_ids(&city), vec![4], "{t:?}");
    }

    #[test]
    fn openstreetmap_wins_both_ways() {
        let mut city = town();
        city.buildings.push(osm(1, sq(0.0, -120.0, 20.0, -100.0), 6, false));
        city.buildings.push(osm(2, sq(100.0, -120.0, 120.0, -100.0), 6, false));
        let cmab = cmab_at(vec![
            (1, vec![rect(2.0, -118.0, 22.0, -98.0)], 20.0, ""),
            (2, vec![rect(80.0, -150.0, 140.0, -90.0)], 20.0, ""),
            (3, vec![rect(200.0, -120.0, 220.0, -100.0)], 20.0, ""),
        ]);
        let t = cmab.fill(&mut city);
        assert_eq!(kept_ids(&city), vec![3], "{t:?}");
    }

    #[test]
    fn guessed_osm_heights_take_cmab_or_come_down() {
        let mut city = town();
        city.buildings.push(osm(1, sq(0.0, -120.0, 20.0, -100.0), 12, true));
        city.buildings.push(osm(2, sq(100.0, -120.0, 120.0, -100.0), 14, true));
        city.buildings.push(osm(3, sq(200.0, -120.0, 220.0, -100.0), 14, false));
        city.buildings.push(osm(4, sq(300.0, -120.0, 320.0, -100.0), 6, true));
        city.buildings.push(osm(5, sq(400.0, -120.0, 420.0, -100.0), 12, true));
        let cmab = cmab_at(vec![
            (1, vec![rect(0.0, -120.0, 20.0, -100.0)], 20.7, "Residence"),
            (2, vec![rect(200.0, -120.0, 220.0, -100.0)], 20.7, ""),
            (3, vec![rect(290.0, -130.0, 340.0, -90.0)], 45.0, ""),
            (4, vec![rect(400.0, -120.0, 406.0, -100.0)], 20.7, ""),
        ]);
        let t = cmab.fill(&mut city);
        let levels: Vec<u32> = city.buildings.iter().filter(|b| b.id < ID_BASE).map(|b| b.levels).collect();
        assert_eq!(levels, vec![6, 7, 14, 6, 7], "{t:?}");
        assert_eq!((t.osm_heights, t.osm_matched, t.osm_capped), (3, 1, 2));
    }

    #[test]
    fn a_record_with_a_hole_and_a_second_building_gives_two_roofs() {
        let mut city = town();
        // Outer (clockwise), a courtyard (counter-clockwise), a second building (clockwise).
        let hole: Vec<[f32; 2]> = rect(5.0, -115.0, 10.0, -110.0).into_iter().rev().collect();
        let cmab = cmab_at(vec![(7, vec![rect(0.0, -120.0, 20.0, -100.0), hole, rect(40.0, -120.0, 60.0, -100.0)], 20.0, "")]);
        let t = cmab.fill(&mut city);
        assert_eq!(t.read, 2);
        assert_eq!(kept_ids(&city), vec![7, 7 + 2 * PART]);
    }

    #[test]
    fn cmab_lines_up_with_openstreetmap() {
        let city = crate::load_osm_heihe().unwrap();
        let cmab = crate::load_cmab_heihe().unwrap();
        let roofs: Vec<Vec<P2>> = cmab.roofs().into_iter().map(|r| clean_ring(r.points)).collect();
        let (mut dx, mut dy) = (Vec::new(), Vec::new());
        for b in city.buildings.iter().filter(|b| b.bank == Bank::China) {
            let area = signed_area(&b.footprint);
            if area < 100.0 {
                continue;
            }
            let c = centroid(&b.footprint);
            for r in &roofs {
                let ra = signed_area(r);
                if ra < 0.8 * area || ra > 1.25 * area {
                    continue;
                }
                let d = centroid(r).sub(c);
                if d.len() < 12.0 {
                    dx.push(d.x);
                    dy.push(d.y);
                }
            }
        }
        dx.sort_by(f32::total_cmp);
        dy.sort_by(f32::total_cmp);
        let (mx, my) = (dx[dx.len() / 2], dy[dy.len() / 2]);
        assert!(dx.len() >= 30, "{} pairs", dx.len());
        assert!(mx.abs() < 1.5 && my.abs() < 1.5, "CMAB sits {mx:.2}, {my:.2} m off OpenStreetMap over {} pairs", dx.len());
    }

    #[test]
    fn simplifying_changes_no_kept_roof_by_more_than_two_percent() {
        let mut city = crate::load_osm_heihe().unwrap();
        let cmab = crate::load_cmab_heihe().unwrap();
        let before: std::collections::HashMap<i64, f32> = cmab.roofs().into_iter().map(|r| (r.id, signed_area(&clean_ring(r.points)))).collect();
        cmab.fill(&mut city);
        let mut worst = 0.0f32;
        for b in city.buildings.iter().filter(|b| b.id >= ID_BASE) {
            let a0 = before[&(b.id - ID_BASE)];
            worst = worst.max((signed_area(&b.footprint) - a0).abs() / a0);
        }
        assert!(worst <= 0.02, "worst area change {:.2}%", worst * 100.0);
    }

    #[test]
    fn height_rule_edges() {
        // 120 to 150 m2 that CMAB reads tall stay in the town's five to seven.
        assert_eq!(height_rule(Some(41.3), 140.0, "Residence"), (7, 3.0));
        // A shop under 1500 m2 has ordinary storeys.
        assert_eq!(height_rule(Some(20.7), 1000.0, "Business"), (6, 3.0));
    }

    #[test]
    fn the_tally_is_the_receipt() {
        let mut city = crate::load_osm_heihe().unwrap();
        let t = crate::load_cmab_heihe().unwrap().fill(&mut city);
        assert_eq!(
            (t.read, t.kept, t.under_osm, t.on_street, t.on_ice, t.too_small, t.bad_shape, t.on_riverfront, t.osm_heights, t.osm_matched, t.osm_capped),
            (877, 669, 138, 4, 29, 31, 0, 6, 77, 85, 4)
        );
    }
}
