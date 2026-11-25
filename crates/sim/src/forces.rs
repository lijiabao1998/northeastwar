//! The Russian side's numbers. They live in data/russia/forces.json, each
//! with the source it was set from, and follow how the equipment performed in
//! Ukraine. Code asks for them here and never hard-codes one.

use serde_json::Value;

/// The war clock runs this many times faster than play.
pub const TIME_SCALE: f32 = 15.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Light {
    Day,
    Dusk,
    NightLit,
    NightDark,
}

impl Light {
    fn key(self) -> &'static str {
        match self {
            Light::Day => "day",
            Light::Dusk => "dusk",
            Light::NightLit => "night_lit",
            Light::NightDark => "night_dark",
        }
    }
    pub fn is_night(self) -> bool {
        matches!(self, Light::NightLit | Light::NightDark)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gait {
    Still,
    Walk,
    Run,
}

#[derive(Clone, Debug)]
pub struct Forces {
    root: Value,
}

impl Forces {
    pub fn from_json(source: &str) -> Result<Forces, String> {
        let root: Value = serde_json::from_str(source).map_err(|e| format!("forces.json: {e}"))?;
        for part in ["infantry_group", "rifle", "grenade", "recon_drone", "mortar_82", "fpv"] {
            if root.get(part).is_none() {
                return Err(format!("forces.json has no {part}"));
            }
        }
        Ok(Forces { root })
    }

    pub fn num(&self, part: &str, key: &str) -> f32 {
        self.root.pointer(&format!("/{part}/{key}")).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32
    }

    pub fn num_or(&self, part: &str, key: &str, default: f32) -> f32 {
        self.root
            .pointer(&format!("/{part}/{key}"))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(default)
    }

    pub fn vehicle_num(&self, vehicle_key: &str, field: &str, default: f32) -> f32 {
        self.root
            .pointer(&format!("/vehicles/{vehicle_key}/{field}"))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(default)
    }

    /// A [low, high] pair.
    pub fn range(&self, part: &str, key: &str) -> (f32, f32) {
        let pair = self.root.pointer(&format!("/{part}/{key}")).and_then(|v| v.as_array());
        match pair {
            Some(a) if a.len() >= 2 => (a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32),
            _ => (0.0, 0.0),
        }
    }

    /// Minutes on the war clock, as seconds of play.
    pub fn war_minutes(minutes: f32) -> f32 {
        minutes * 60.0 / TIME_SCALE
    }

    /// How far a soldier sees a person in this light.
    pub fn sight(&self, light: Light, optics: bool) -> f32 {
        let reach = self.root.pointer(&format!("/infantry_group/sight_m/{}", light.key())).and_then(|v| v.as_f64()).unwrap_or(30.0) as f32;
        if optics && light.is_night() {
            reach.max(self.num("infantry_group", "night_optics_sight_m"))
        } else {
            reach
        }
    }

    /// The chance one burst hits a standing person at this range.
    pub fn hit_chance(&self, metres: f32) -> f32 {
        let Some(rows) = self.root.pointer("/rifle/hit_p_by_range_m").and_then(|v| v.as_array()) else { return 0.0 };
        let row = |i: usize| -> (f32, f32) {
            let r = rows[i].as_array();
            match r {
                Some(r) if r.len() >= 2 => (r[0].as_f64().unwrap_or(0.0) as f32, r[1].as_f64().unwrap_or(0.0) as f32),
                _ => (0.0, 0.0),
            }
        };
        if rows.is_empty() {
            return 0.0;
        }
        if metres <= row(0).0 {
            return row(0).1;
        }
        for i in 1..rows.len() {
            let (a, b) = (row(i - 1), row(i));
            if metres <= b.0 {
                let t = (metres - a.0) / (b.0 - a.0).max(0.001);
                return a.1 + (b.1 - a.1) * t;
            }
        }
        0.0
    }

    /// Seconds of unbroken sight before a soldier is sure of what he sees.
    pub fn notice_time(&self, metres: f32, reach: f32) -> f32 {
        let near = self.num("infantry_group", "notice_s/near");
        let far = self.num("infantry_group", "notice_s/far");
        let t = (metres / reach.max(1.0)).clamp(0.0, 1.0);
        near + (far - near) * t
    }

    pub fn hearing(&self, gait: Gait) -> f32 {
        let key = match gait {
            Gait::Still => "still",
            Gait::Walk => "walk",
            Gait::Run => "run",
        };
        self.num("infantry_group", &format!("hearing_m/{key}"))
    }
}

/// How much of a watcher's reach you fill. Still in a doorway is the least.
pub fn exposure(gait: Gait, sheltered: bool) -> f32 {
    let base = match gait {
        Gait::Run => 1.15,
        Gait::Walk => 1.0,
        Gait::Still => 0.8,
    };
    if sheltered {
        base * 0.55
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forces() -> Forces {
        let path = crate::data_dir().join("russia/forces.json");
        Forces::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn a_group_is_two_to_five_and_sight_falls_with_the_light() {
        let f = forces();
        assert_eq!(f.range("infantry_group", "size"), (2.0, 5.0));
        assert!(f.sight(Light::Day, false) > f.sight(Light::Dusk, false));
        assert!(f.sight(Light::Dusk, false) > f.sight(Light::NightLit, false));
        assert!(f.sight(Light::NightLit, false) > f.sight(Light::NightDark, false));
        assert!(f.sight(Light::NightDark, true) > f.sight(Light::NightDark, false));
        assert_eq!(f.sight(Light::Day, true), f.sight(Light::Day, false));
    }

    #[test]
    fn a_rifle_is_worse_further_away_and_never_sure() {
        let f = forces();
        let mut last = 1.0;
        for metres in [5.0, 20.0, 40.0, 70.0, 84.0] {
            let p = f.hit_chance(metres);
            assert!(p <= last && p < 0.95);
            last = p;
        }
        assert_eq!(f.hit_chance(300.0), 0.0);
        assert!(f.notice_time(5.0, 60.0) < f.notice_time(55.0, 60.0));
    }

    #[test]
    fn drone_cued_fire_takes_three_to_five_minutes_on_the_war_clock() {
        let f = forces();
        let (low, high) = f.range("mortar_82", "sensor_to_shooter_min");
        assert_eq!((low, high), (3.0, 5.0));
        assert!((Forces::war_minutes(low) - 12.0).abs() < 0.01);
        assert!((Forces::war_minutes(high) - 20.0).abs() < 0.01);
        // An FPV takes longer to arrive than a mortar round, as reported.
        assert!(f.range("fpv", "spotted_to_strike_min").0 > high);
    }

    #[test]
    fn standing_still_in_a_doorway_shows_least() {
        assert!(exposure(Gait::Still, true) < exposure(Gait::Walk, true));
        assert!(exposure(Gait::Walk, true) < exposure(Gait::Walk, false));
        assert!(exposure(Gait::Walk, false) < exposure(Gait::Run, false));
        let f = forces();
        assert!(f.hearing(Gait::Run) > f.hearing(Gait::Walk));
        assert_eq!(f.hearing(Gait::Still), 0.0);
    }
}
