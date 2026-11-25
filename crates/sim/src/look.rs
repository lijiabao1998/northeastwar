//! What each building in Heihe looks like. The colours come from
//! data/heihe/look.json, which says which photograph each was read from.
//! The kinds below are the kinds of building those photographs show.

use crate::city::{Bank, Building, City};
use crate::geom::{dist_to_polyline, hash01};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

    /// No colour: what is drawn in it is not drawn at all (snow in summer, ice on
    /// open water, leaves in winter). Soup skips it.
    pub const NONE: Rgb = Rgb { r: f32::NAN, g: f32::NAN, b: f32::NAN };

    pub fn is_none(self) -> bool {
        self.r.is_nan()
    }

    /// Blend towards `other`. Mixing with no colour leaves the colour as it is, so
    /// "bark with a little snow on it" is bark when there is no snow; both absent
    /// is absent.
    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        match (self.is_none(), other.is_none()) {
            (true, true) => Rgb::NONE,
            (true, false) => other,
            (false, true) => self,
            _ => Rgb { r: self.r + (other.r - self.r) * t, g: self.g + (other.g - self.g) * t, b: self.b + (other.b - self.b) * t },
        }
    }
}

/// Every colour Palette::seasonal reads from data/heihe/look.json.
pub const SEASON_COLOURS: [&str; 21] = [
    "granite_paving",
    "verge_dormant_spring",
    "grass_summer",
    "grass_late_summer",
    "grass_autumn",
    "grass_autumn_straw",
    "roof_port_light",
    "roof_metal_red",
    "roof_metal_blue",
    "far_roof_red",
    "far_roof_grey_metal",
    "road_dust_spring",
    "asphalt_summer_sun",
    "river_floes",
    "river_young_ice",
    "river_floe_gaps",
    "river_water_clear",
    "river_water_clear_deep",
    "foliage_spring",
    "foliage_summer",
    "foliage_autumn",
];

/// How much snow lies on the ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SnowCover {
    None,
    Patchy,
    Deep,
}

/// What the trees carry, and the grass under them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Foliage {
    /// Winter into spring: no leaves, the verges dead.
    Bare,
    /// The first leaves.
    Spring,
    Summer,
    /// September: the trees still in full leaf, the lawns yellowing.
    LateSummer,
    /// October: the leaves turned.
    Autumn,
    /// The leaves down, the grass straw, until the snow.
    Fallen,
}

impl Foliage {
    /// By the date. The photographs bracket the dates; the dates themselves are play
    /// (docs/research/seasons-look.md): no leaves on 11 April; the first leaves on
    /// young street trees on 21 May and willows in leaf on 26 May; still in full leaf
    /// on 7 September; half bare on 12 October and in autumn colour on 13 October.
    pub fn on(month: u32, day: u32) -> Foliage {
        match (month, day) {
            (5, 10..) | (6, ..=5) => Foliage::Spring,
            (6..=8, _) => Foliage::Summer,
            (9, _) => Foliage::LateSummer,
            (10, ..=25) => Foliage::Autumn,
            (10..=12, _) => Foliage::Fallen,
            _ => Foliage::Bare,
        }
    }
}

/// The look of the year at one moment: the snow, the river, the trees. The town
/// is drawn again when it changes, a few times a year.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Season {
    pub snow: SnowCover,
    pub river_open: bool,
    /// The river frozen, but thinly: the weeks after freeze-up in November, and the
    /// last days before the ice goes in April. Floes and young ice, not a sheet.
    pub thin_ice: bool,
    pub foliage: Foliage,
}

/// Ice thinner than this is floes and young ice to the eye (play: between the
/// freeze-up's 0 cm and the 50 cm of late December).
const THIN_ICE_CM: f32 = 20.0;

impl Season {
    /// The deep winter the town was first drawn in.
    pub const WINTER: Season = Season { snow: SnowCover::Deep, river_open: false, thin_ice: false, foliage: Foliage::Bare };

    /// The season at `hour`: snow from the month's normal depth, the river from its
    /// ice, the trees by the date (Foliage::on).
    pub fn at(hour: f32) -> Season {
        let l = crate::living::get();
        let depth = l.snow_depth_cm(hour);
        // The depths are monthly means, days without snow counted as nought: a mean of
        // 1 cm (October's) is ground white some days and bare on others, so patchy
        // starts there; a mean of 3 cm (March's) and more is ground under snow.
        let snow = if depth >= 3.0 {
            SnowCover::Deep
        } else if depth >= 1.0 {
            SnowCover::Patchy
        } else {
            SnowCover::None
        };
        let (m, d) = crate::clock::date_of(hour);
        let ice = l.ice_cm(hour);
        Season { snow, river_open: ice <= 0.0, thin_ice: ice > 0.0 && ice < THIN_ICE_CM, foliage: Foliage::on(m, d) }
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

    /// A named colour if the table has it, else `fallback`.
    pub fn get_or(&self, name: &str, fallback: Rgb) -> Rgb {
        self.colours.get(name).copied().unwrap_or(fallback)
    }

    pub fn set(&mut self, name: &str, c: Rgb) {
        self.colours.insert(name.to_string(), c);
    }

    /// The palette for `season`. The winter the town was drawn in is the palette as
    /// it is. Without snow, the snow lying on things is not drawn, the ground is the
    /// season's grass and the roofs are the roofs. With the river open, the ice is
    /// water. The trees carry the season's leaves. The colours for all of that come
    /// from data/heihe/look.json, read off photographs like the rest
    /// (docs/research/seasons-look.md); where the table has none, the winter colour
    /// stays. SEASON_COLOURS lists every name read here.
    pub fn seasonal(&self, s: &Season) -> Palette {
        let mut p = self.clone();
        let has = |n: &str| self.colours.contains_key(n);
        let pick = |names: &[&str]| names.iter().find(|n| has(n)).map(|n| self.get(n));
        // The open ground between the streets: the town has no other colour for it,
        // so in summer it is the mown verge.
        let bare_ground = match s.foliage {
            Foliage::Bare => pick(&["verge_dormant_spring"]),
            Foliage::Spring | Foliage::Summer => pick(&["grass_summer"]),
            Foliage::LateSummer => pick(&["grass_late_summer"]),
            Foliage::Autumn => pick(&["grass_autumn"]),
            Foliage::Fallen => pick(&["grass_autumn_straw"]),
        };
        let snow = self.get("snow");
        // Roofs: no photograph shows Heihe's roofs from above. Ordinary housing has red
        // and blue metal roofs; flat roofs across the water read pale. Each building
        // takes one of the three (facade.rs), and the far bank one of its own two.
        for (key, from) in [
            ("roof_top", "roof_port_light"),
            ("roof_top_2", "roof_metal_red"),
            ("roof_top_3", "roof_metal_blue"),
            ("far_roof_top", "far_roof_red"),
            ("far_roof_top_2", "far_roof_grey_metal"),
        ] {
            let Some(r) = pick(&[from]) else { continue };
            match s.snow {
                SnowCover::Deep => {}
                SnowCover::Patchy => p.set(key, r.mix(snow, 0.5)),
                SnowCover::None => p.set(key, r),
            }
        }
        match s.snow {
            SnowCover::Deep => {}
            SnowCover::Patchy => {
                if let Some(g) = bare_ground {
                    p.set("snow_ground", g.mix(self.get("snow_ground"), 0.45));
                }
            }
            SnowCover::None => {
                for n in ["snow", "snow_dirty", "snow_tint", "asphalt_snow_film"] {
                    p.set(n, Rgb::NONE);
                }
                if let Some(g) = bare_ground {
                    p.set("snow_ground", g);
                }
                // Dry asphalt; in April, still pale with the winter's grit.
                let road = if s.foliage == Foliage::Bare {
                    pick(&["road_dust_spring", "asphalt_summer_sun"])
                } else {
                    pick(&["asphalt_summer_sun"])
                };
                if let Some(a) = road {
                    p.set("asphalt", a);
                }
            }
        }
        // Paving round the buildings (scene.rs): the town's granite, under the snow in
        // winter.
        if let Some(f) = pick(&["granite_paving"]) {
            match s.snow {
                SnowCover::Deep => {}
                SnowCover::Patchy => p.set("forecourt", f.mix(self.get("snow_ground"), 0.45)),
                SnowCover::None => p.set("forecourt", f),
            }
        }
        if s.thin_ice {
            // Floes and young ice. No photograph of the April break-up at Heihe was
            // found, so the November freeze-up stands in for both.
            if let Some(f) = pick(&["river_floes"]) {
                p.set("river_surface", f);
            }
            if let Some(y) = pick(&["river_young_ice"]) {
                p.set("river_ice", y);
            }
            if let Some(g) = pick(&["river_floe_gaps"]) {
                p.set("river_ice_dark", g);
            }
            p.set("river_snow", Rgb::NONE);
        } else if !s.river_open && s.snow != SnowCover::Deep {
            // The ice outlasts the snow on the banks. It keeps its winter face (the
            // scene mixes it with the ground colour, which is no longer snow), and
            // with the banks bare no snow lies on it.
            let winter_face = self.get("river_ice").mix(self.get("snow_ground"), 0.34);
            p.set("river_surface", winter_face);
            p.set("river_snow", if s.snow == SnowCover::Patchy { self.get("snow_ground") } else { Rgb::NONE });
        }
        if s.river_open {
            // Open water under a clear sky is the sky's blue, deeper mid-river. Under
            // cloud it reads grey (river_water_overcast); one colour is drawn for now.
            if let Some(w) = pick(&["river_water_clear"]) {
                p.set("river_surface", w);
            }
            p.set("river_ice", pick(&["river_water_clear_deep"]).unwrap_or(Rgb::NONE));
            for n in ["river_ice_dark", "river_snow", "shore_ice"] {
                p.set(n, Rgb::NONE);
            }
        }
        let leaves = match s.foliage {
            Foliage::Bare | Foliage::Fallen => None,
            Foliage::Spring => pick(&["foliage_spring"]),
            Foliage::Summer | Foliage::LateSummer => pick(&["foliage_summer"]),
            Foliage::Autumn => pick(&["foliage_autumn"]),
        };
        p.set("foliage", leaves.unwrap_or(Rgb::NONE));
        p
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
    // Shops on the ground floor: along a main street, or where CMAB says the building is for trade or the public.
    let for_trade = matches!(building.function.as_str(), "Business" | "Public service" | "Office");
    let near_street = for_trade || city.roads.iter().filter(|r| r.width >= 8.0).any(|r| dist_to_polyline(building.centroid, &r.line) < 30.0);
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
    // Low brick shops, but not a two-storey hall with 4 m storeys.
    if building.levels <= 3 && building.floor_h < 3.5 {
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
