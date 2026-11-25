//! The occupation after the first night: the sweep, the order sheet and curfew,
//! being searched, being taken, the watchlist. The numbers live in
//! data/occupation.json, each with its source (`src`) and whether the source
//! gives it (`reported`), it was worked out (`derived`) or it was chosen for play
//! (`play`). The research is docs/research/occupation.md.

use crate::geom::hash01;
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct Occupation {
    root: Value,
}

static OCCUPATION: OnceLock<Occupation> = OnceLock::new();

pub fn get() -> &'static Occupation {
    OCCUPATION.get_or_init(|| {
        let path = crate::data_dir().join("occupation.json");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Occupation::from_json(&text).unwrap_or_else(|e| panic!("{e}"))
    })
}

impl Occupation {
    pub fn from_json(source: &str) -> Result<Occupation, String> {
        let root: Value = serde_json::from_str(source).map_err(|e| format!("occupation.json: {e}"))?;
        for part in ["sweep", "order", "inspection", "detention", "suspicion", "visits"] {
            if root.get(part).is_none() {
                return Err(format!("occupation.json has no {part}"));
            }
        }
        Ok(Occupation { root })
    }

    pub fn num(&self, part: &str, key: &str) -> f32 {
        self.root
            .pointer(&format!("/{part}/{key}"))
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("occupation.json has no {part}/{key}")) as f32
    }

    /// When the sweep reaches building `building` (hours since 17 January 00:00):
    /// a daylight hour on one of the sweep's days, rolled from the run's seed and
    /// the building.
    pub fn sweep_hour(&self, seed: u64, building: i64) -> f32 {
        let first = self.num("sweep", "first_day");
        let last = self.num("sweep", "last_day");
        let from = self.num("sweep", "hour_from");
        let to = self.num("sweep", "hour_to");
        let s = (seed as i64) ^ building.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let days = (last - first + 1.0).max(1.0);
        let day = first + (hash01(s, 0x5EE1) * days).floor().min(days - 1.0);
        day * 24.0 + from + hash01(s, 0x5EE2) * (to - from)
    }

    /// How many days a detention lasts for a roll in [0, 1), or None when the
    /// person is moved away and never comes back.
    pub fn detention_days(&self, roll: f32, within: f32) -> Option<f32> {
        let split = self.root.pointer("/detention/length_split").and_then(|v| v.as_array()).expect("length_split");
        let mut acc = 0.0f32;
        for band in split {
            let b = band.as_array().expect("band");
            let share = b[0].as_f64().unwrap_or(0.0) as f32;
            acc += share;
            if roll < acc {
                let lo = b[1].as_f64().unwrap_or(-1.0) as f32;
                let hi = b[2].as_f64().unwrap_or(-1.0) as f32;
                if lo < 0.0 {
                    return None;
                }
                return Some(lo + (hi - lo) * within);
            }
        }
        None
    }
}

/// Who the player is, as far as the occupation is concerned.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    pub male: bool,
    pub age: u8,
}

impl Default for Profile {
    fn default() -> Self {
        // The game's figure: a man in his thirties. Choosing who you are comes later.
        Self { male: true, age: 34 }
    }
}

/// What a search is looking for: weapons and communication devices
/// (docs/research/occupation.md, 1.1 and 1.2).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contraband {
    pub primaries: u8,
    pub ammo: u32,
    pub rpg: u32,
    pub pf98: u32,
    pub hj12: u32,
    pub rpo_shmel: u32,
    pub grenades: u32,
    pub smoke_grenades: u32,
    pub molotov: u32,
    pub friendly_fpv: u32,
    pub radio: bool,
    pub radio_strikes: u32,
}

impl Contraband {
    pub fn is_empty(&self) -> bool {
        *self == Contraband::default()
    }

    /// Something that fires: a gun or an anti-tank weapon.
    pub fn armed(&self) -> bool {
        self.primaries != 0 || self.rpg + self.pf98 + self.hj12 + self.rpo_shmel > 0
    }

    pub fn add(&mut self, o: Contraband) {
        self.primaries |= o.primaries;
        self.ammo += o.ammo;
        self.rpg += o.rpg;
        self.pf98 += o.pf98;
        self.hj12 += o.hj12;
        self.rpo_shmel += o.rpo_shmel;
        self.grenades += o.grenades;
        self.smoke_grenades += o.smoke_grenades;
        self.molotov += o.molotov;
        self.friendly_fpv += o.friendly_fpv;
        self.radio |= o.radio;
        self.radio_strikes += o.radio_strikes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sweep_falls_on_its_days_by_daylight() {
        let o = get();
        for b in 0..200 {
            let h = o.sweep_hour(7, b);
            let day = (h / 24.0).floor();
            let t = h - day * 24.0;
            assert!(day >= o.num("sweep", "first_day") && day <= o.num("sweep", "last_day"), "{h}");
            assert!(t >= o.num("sweep", "hour_from") && t <= o.num("sweep", "hour_to"), "{h}");
        }
        assert_ne!(o.sweep_hour(7, 3), o.sweep_hour(8, 3));
    }

    #[test]
    fn detention_lengths_follow_the_split() {
        let o = get();
        let n = 1000;
        let mut short = 0;
        let mut never = 0;
        for i in 0..n {
            match o.detention_days((i as f32 + 0.5) / n as f32, 0.5) {
                None => never += 1,
                Some(d) if d < 4.0 => short += 1,
                Some(_) => {}
            }
        }
        assert!((380..420).contains(&short), "{short}");
        assert!((40..60).contains(&never), "{never}");
    }
}
