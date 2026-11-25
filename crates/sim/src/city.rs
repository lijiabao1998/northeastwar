//! The city as data: what tools/osm_import wrote to data/heihe/city.json,
//! read once and turned the right way up.
//!
//! The file says "X east, Z north". Map metres here are x east, y north. A
//! renderer with a right-handed, y-up world must put north at minus z, or the
//! whole city comes out as its mirror image, signs included.

use crate::geom::{clean_ring, hash01, P2};
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bank {
    /// Heihe.
    China,
    /// Blagoveshchensk, across the ice.
    Russia,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub id: i64,
    pub bank: Bank,
    pub levels: u32,
    pub height: f32,
    pub floor_h: f32,
    pub name_zh: String,
    pub name_ru: String,
    /// Counter-clockwise, no repeated closing point.
    pub footprint: Vec<P2>,
    pub centroid: P2,
    pub solid: bool,
    /// What the building is for, where a source says: CMAB's Residence, Business,
    /// Public service, Office or Other. Empty for OpenStreetMap buildings.
    pub function: String,
    /// True when nobody measured the height: the OSM importer guessed it from
    /// the footprint's area and a hash of the building's id.
    pub height_guessed: bool,
}

#[derive(Clone, Debug)]
pub struct Road {
    pub name: String,
    pub name_ru: String,
    pub kind: String,
    pub width: f32,
    pub line: Vec<P2>,
}

impl Road {
    pub fn pedestrian(&self) -> bool {
        self.kind == "pedestrian" || self.kind == "footway"
    }
}

#[derive(Clone, Debug)]
pub struct Sign {
    pub text: String,
    pub text_ru: String,
    pub at: P2,
    pub up: f32,
    pub width: f32,
    pub palette: usize,
    pub shop: bool,
}

#[derive(Clone, Debug)]
pub struct Landmark {
    pub kind: String,
    pub name_zh: String,
    pub at: P2,
}

#[derive(Clone, Debug, Default)]
pub struct City {
    pub title: String,
    pub attribution: String,
    pub spawn: P2,
    pub buildings: Vec<Building>,
    pub roads: Vec<Road>,
    /// The Chinese bank, west to east.
    pub bank: Vec<P2>,
    /// How far out on the ice you may walk, west to east.
    pub ice_limit: Vec<P2>,
    pub signs: Vec<Sign>,
    pub trees: Vec<(P2, f32)>,
    pub lamps: Vec<P2>,
    pub cars: Vec<P2>,
    pub landmarks: Vec<Landmark>,
    pub heat: Vec<(P2, f32)>,
    pub paths: Vec<Vec<P2>>,
    pub walk_x: (f32, f32),
    pub walk_south: f32,
}

#[derive(Deserialize)]
struct RawBuilding {
    id: i64,
    side: String,
    #[serde(default)]
    height_source: String,
    #[serde(default)]
    levels: f32,
    #[serde(default)]
    height: f32,
    #[serde(default)]
    floor_h: f32,
    #[serde(default)]
    name_zh: String,
    #[serde(default)]
    name_ru: String,
    #[serde(default)]
    centroid: Vec<f32>,
    #[serde(default)]
    collision: bool,
    #[serde(default)]
    footprint: Vec<Vec<f32>>,
}

fn p2(v: &Value) -> Option<P2> {
    if let Some(a) = v.as_array() {
        if a.len() >= 2 {
            return Some(P2::new(a[0].as_f64()? as f32, a[1].as_f64()? as f32));
        }
    }
    if let Some(o) = v.as_object() {
        return Some(P2::new(o.get("x")?.as_f64()? as f32, o.get("z")?.as_f64()? as f32));
    }
    None
}

fn line(v: Option<&Value>) -> Vec<P2> {
    v.and_then(|v| v.as_array()).map(|a| a.iter().filter_map(p2).collect()).unwrap_or_default()
}

fn text(v: &Value, key: &str) -> String {
    v.get(key).and_then(|s| s.as_str()).unwrap_or("").to_string()
}

fn num(v: &Value, key: &str, fallback: f32) -> f32 {
    v.get(key).and_then(|n| n.as_f64()).map(|n| n as f32).unwrap_or(fallback)
}

impl City {
    pub fn from_json(source: &str) -> Result<City, String> {
        let root: Value = serde_json::from_str(source).map_err(|e| format!("city.json: {e}"))?;
        let meta = root.get("meta").cloned().unwrap_or(Value::Null);
        let mut city = City {
            title: text(&meta, "title"),
            attribution: text(&meta, "attribution"),
            ..City::default()
        };
        if let Some(spawn) = meta.get("spawn").and_then(p2) {
            city.spawn = spawn;
        }
        let wx = line(Some(&Value::Array(vec![meta.get("walkable_x").cloned().unwrap_or(Value::Null)])));
        city.walk_x = wx.first().map(|p| (p.x, p.y)).unwrap_or((-400.0, 400.0));
        city.walk_south = meta
            .get("walkable_z")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|n| n.as_f64())
            .map(|n| n as f32)
            .unwrap_or(-400.0);

        let raw: Vec<RawBuilding> = serde_json::from_value(root.get("buildings").cloned().unwrap_or(Value::Array(vec![])))
            .map_err(|e| format!("buildings: {e}"))?;
        for b in raw {
            let ring: Vec<P2> = b.footprint.iter().filter(|p| p.len() >= 2).map(|p| P2::new(p[0], p[1])).collect();
            let mut ring = clean_ring(ring);
            if ring.len() < 3 {
                // The far bank comes as centre points only. It is a skyline seen
                // across the ice, so each point becomes a plain block.
                if b.centroid.len() < 2 {
                    continue;
                }
                let c = P2::new(b.centroid[0], b.centroid[1]);
                let long = 22.0 + 34.0 * hash01(b.id, 11);
                let wide = 12.0 + 6.0 * hash01(b.id, 12);
                let turn = if hash01(b.id, 13) < 0.5 { 0.0 } else { std::f32::consts::FRAC_PI_2 } - 0.42;
                let (ax, ay) = (P2::new(turn.cos(), turn.sin()), P2::new(-turn.sin(), turn.cos()));
                ring = vec![
                    c.sub(ax.mul(long * 0.5)).sub(ay.mul(wide * 0.5)),
                    c.add(ax.mul(long * 0.5)).sub(ay.mul(wide * 0.5)),
                    c.add(ax.mul(long * 0.5)).add(ay.mul(wide * 0.5)),
                    c.sub(ax.mul(long * 0.5)).add(ay.mul(wide * 0.5)),
                ];
            }
            let centroid = if b.centroid.len() >= 2 {
                P2::new(b.centroid[0], b.centroid[1])
            } else {
                let mut c = P2::new(0.0, 0.0);
                for p in &ring {
                    c = c.add(*p);
                }
                c.mul(1.0 / ring.len() as f32)
            };
            let floor_h = if b.floor_h > 1.0 { b.floor_h } else { 3.0 };
            let levels = if b.levels >= 1.0 { b.levels.round() as u32 } else { ((b.height / floor_h).round() as u32).max(1) };
            city.buildings.push(Building {
                id: b.id,
                bank: if b.side == "cn" { Bank::China } else { Bank::Russia },
                levels,
                height: if b.height > 1.0 { b.height } else { levels as f32 * floor_h },
                floor_h,
                name_zh: b.name_zh,
                name_ru: b.name_ru,
                footprint: ring,
                centroid,
                solid: b.collision,
                function: String::new(),
                height_guessed: b.height_source == "estimated_from_footprint",
            });
        }

        for r in root.get("roads").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            let pts = line(r.get("points"));
            if pts.len() < 2 {
                continue;
            }
            city.roads.push(Road {
                name: text(&r, "name"),
                name_ru: text(&r, "name_ru"),
                kind: text(&r, "highway"),
                width: num(&r, "width", 6.0),
                line: pts,
            });
        }
        city.bank = line(root.get("bank"));
        city.ice_limit = line(root.get("ice_limit"));
        for s in root.get("signs").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            let Some(at) = p2(&s) else { continue };
            city.signs.push(Sign {
                text: text(&s, "text"),
                text_ru: text(&s, "text_ru"),
                at,
                up: num(&s, "y", 3.3),
                width: num(&s, "width", 2.4),
                palette: num(&s, "palette", 0.0) as usize,
                shop: text(&s, "kind") == "shop",
            });
        }
        for t in root.get("trees").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            if let Some(at) = p2(&t) {
                city.trees.push((at, num(&t, "scale", 1.0)));
            }
        }
        city.lamps = line(root.get("lamps"));
        city.cars = line(root.get("cars"));
        for l in root.get("landmarks").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            if let Some(at) = p2(&l) {
                city.landmarks.push(Landmark { kind: text(&l, "kind"), name_zh: text(&l, "name_zh"), at });
            }
        }
        for h in root.get("heat").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            if let Some(at) = p2(&h) {
                city.heat.push((at, num(&h, "radius", 3.2)));
            }
        }
        for p in root.get("paths").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            let pts = line(Some(&p));
            if pts.len() >= 4 {
                city.paths.push(pts);
            }
        }
        Ok(city)
    }

    /// The bank's northing at this easting.
    pub fn bank_y(&self, x: f32) -> f32 {
        sample(&self.bank, x, 40.0)
    }

    /// The furthest north you may walk on the ice at this easting.
    pub fn ice_y(&self, x: f32) -> f32 {
        sample(&self.ice_limit, x, self.bank_y(x) + 80.0)
    }

    pub fn road(&self, name: &str) -> Option<&Road> {
        self.roads.iter().find(|r| r.name == name)
    }

    /// True near a mapped shop entrance or doorway heat source.
    pub fn near_heat(&self, p: P2) -> bool {
        self.heat.iter().any(|(at, r)| p.dist(*at) <= *r)
            || self.signs.iter().any(|s| s.shop && p.dist(s.at) <= 2.8)
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
