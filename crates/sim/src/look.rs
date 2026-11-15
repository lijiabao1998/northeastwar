//! What each building in Heihe looks like. The colours come from
//! data/heihe/look.json, which says which photograph each was read from.
//! The kinds below are the kinds of building those photographs show.

use crate::city::{Bank, Building, City};
use crate::geom::{dist_to_polyline, hash01};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    /// "#RRGGBB" as sRGB 0..1.
    pub fn hex(code: &str) -> Option<Rgb> {
        let code = code.trim_start_matches('#');
        if code.len() != 6 {
            return None;
        }
        let part = |i: usize| u8::from_str_radix(&code[i..i + 2], 16).ok().map(|v| v as f32 / 255.0);
        Some(Rgb { r: part(0)?, g: part(2)?, b: part(4)? })
    }

    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        Rgb { r: self.r + (other.r - self.r) * t, g: self.g + (other.g - self.g) * t, b: self.b + (other.b - self.b) * t }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Palette {
    colours: HashMap<String, Rgb>,
    pub boards: Vec<(Rgb, Rgb)>,
    /// colour name -> the photograph it was read from
    pub sources: HashMap<String, String>,
}

impl Palette {
    pub fn from_json(source: &str) -> Result<Palette, String> {
        let root: Value = serde_json::from_str(source).map_err(|e| format!("look.json: {e}"))?;
        let mut out = Palette::default();
        for group in ["walls", "roofs", "ground", "props"] {
            let Some(map) = root.get(group).and_then(|v| v.as_object()) else { continue };
            for (name, entry) in map {
                let hex = entry.get("hex").and_then(|h| h.as_str()).ok_or(format!("{name} has no hex"))?;
                let rgb = Rgb::hex(hex).ok_or(format!("{name}: bad hex {hex}"))?;
                out.colours.insert(name.clone(), rgb);
                let from = entry.get("from").and_then(|h| h.as_str()).unwrap_or("");
                out.sources.insert(name.clone(), from.to_string());
            }
        }
        if let Some(boards) = root.pointer("/signs/boards").and_then(|v| v.as_array()) {
            for board in boards {
                let bg = board.get("hex").and_then(|h| h.as_str()).and_then(Rgb::hex).ok_or("board hex")?;
                let fg = board.get("text").and_then(|h| h.as_str()).and_then(Rgb::hex).ok_or("board text")?;
                out.boards.push((bg, fg));
            }
        }
        Ok(out)
    }

    /// A named colour. A name that is not in the table is a bug, and shows as magenta.
    pub fn get(&self, name: &str) -> Rgb {
        self.colours.get(name).copied().unwrap_or(Rgb { r: 1.0, g: 0.0, b: 1.0 })
    }

    pub fn has(&self, name: &str) -> bool {
        self.colours.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.colours.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colours.is_empty()
    }
}

/// The kinds of building the photographs of Heihe show.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Central Street: salmon render, cream panels and pilasters, gables, a red cone on a corner.
    CentralSalmon,
    /// Central Street: red walls, white pilasters and arches, green domes.
    CentralBaroque,
    /// Central Street: pale grey classical front with tall arched glass.
    CentralClassical,
    /// Housing: yellow walls, crimson balcony bands, white frames.
    YellowCrimson,
    /// Housing: tan tile, cream bands, glazed balconies.
    TanTile,
    /// Housing: salmon render with yellow bay bands.
    SalmonYellow,
    /// Two or three storeys: white tile over red brick, roller shutters.
    LowBrick,
    /// Blagoveshchensk across the ice: brick-orange and white, seen from afar.
    FarBank,
}

#[derive(Clone, Copy, Debug)]
pub struct Dress {
    pub kind: Kind,
    /// Shops along the ground floor.
    pub shops: bool,
    /// A cone or a dome on a corner.
    pub crown: bool,
}

/// What a building wears. The same building always gets the same answer.
pub fn dress(building: &Building, city: &City) -> Dress {
    if building.bank == Bank::Russia {
        return Dress { kind: Kind::FarBank, shops: false, crown: false };
    }
    // A building fronts Central Street if any corner of it stands near the street,
    // however deep the rest of it runs back.
    let on_central = city
        .roads
        .iter()
        .filter(|r| r.name == "中央街")
        .any(|r| building.footprint.iter().any(|p| dist_to_polyline(*p, &r.line) < 24.0));
    let near_street = city.roads.iter().filter(|r| r.width >= 8.0).any(|r| dist_to_polyline(building.centroid, &r.line) < 30.0);
    let pick = hash01(building.id, 1);
    if on_central {
        let kind = if pick < 0.62 {
            Kind::CentralSalmon
        } else if pick < 0.84 {
            Kind::CentralBaroque
        } else {
            Kind::CentralClassical
        };
        return Dress { kind, shops: true, crown: hash01(building.id, 2) < 0.6 };
    }
    if building.levels <= 3 {
        return Dress { kind: Kind::LowBrick, shops: near_street, crown: false };
    }
    let kind = if pick < 0.4 {
        Kind::YellowCrimson
    } else if pick < 0.78 {
        Kind::TanTile
    } else {
        Kind::SalmonYellow
    };
    Dress { kind, shops: near_street, crown: false }
}
