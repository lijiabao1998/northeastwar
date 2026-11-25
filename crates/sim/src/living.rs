//! What it takes to stay alive over days: the weather outside, how a room holds
//! its heat, fuel, water, food and illness. The numbers live in
//! data/living.json, each with the source it was set from (`src`) and whether
//! the source gives it (`reported`), it was worked out (`derived`) or it was
//! chosen for play (`play`). Code asks for them here and never hard-codes one.

use crate::clock;
use crate::geom::hash01;
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct Living {
    root: Value,
}

static LIVING: OnceLock<Living> = OnceLock::new();

/// The table, read once from data/living.json.
pub fn get() -> &'static Living {
    LIVING.get_or_init(|| {
        let path = crate::data_dir().join("living.json");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Living::from_json(&text).unwrap_or_else(|e| panic!("{e}"))
    })
}

impl Living {
    pub fn from_json(source: &str) -> Result<Living, String> {
        let root: Value = serde_json::from_str(source).map_err(|e| format!("living.json: {e}"))?;
        for part in ["climate", "building", "fuel", "body", "shops"] {
            if root.get(part).is_none() {
                return Err(format!("living.json has no {part}"));
            }
        }
        for key in ["mean_max_c", "mean_min_c", "snow_days"] {
            let n = root.pointer(&format!("/climate/{key}")).and_then(|v| v.as_array()).map_or(0, |a| a.len());
            if n != 12 {
                return Err(format!("living.json climate/{key} needs 12 months, has {n}"));
            }
        }
        Ok(Living { root })
    }

    pub fn num(&self, part: &str, key: &str) -> f32 {
        self.root
            .pointer(&format!("/{part}/{key}"))
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| panic!("living.json has no {part}/{key}")) as f32
    }

    fn month(&self, key: &str, m: usize) -> f32 {
        self.root
            .pointer(&format!("/climate/{key}/{m}"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32
    }

    /// A monthly normal at a fractional day of the year (0-based), read between
    /// the middles of the months on either side.
    fn normal(&self, key: &str, doy: f32) -> f32 {
        const MID: [f32; 12] = [15.5, 45.0, 74.5, 105.0, 135.5, 166.0, 196.5, 227.5, 258.0, 288.5, 319.0, 349.5];
        let d = doy.rem_euclid(365.0);
        let mut hi = MID.iter().position(|&m| m > d).unwrap_or(12);
        let lo = if hi == 0 { 11 } else { hi - 1 };
        if hi == 12 {
            hi = 0;
        }
        let (a, b) = (MID[lo], MID[hi]);
        let span = (b - a).rem_euclid(365.0);
        let t = ((d - a).rem_euclid(365.0) / span).clamp(0.0, 1.0);
        self.month(key, lo) * (1.0 - t) + self.month(key, hi) * t
    }

    /// Days with snow in the month `hour` falls in, as a share of the month.
    pub fn snow_share(&self, hour: f32) -> f32 {
        let (m, _) = clock::date_of(hour);
        const DAYS: [f32; 12] = [31.0, 28.0, 31.0, 30.0, 31.0, 30.0, 31.0, 31.0, 30.0, 31.0, 30.0, 31.0];
        (self.month("snow_days", m as usize - 1) / DAYS[m as usize - 1]).clamp(0.0, 1.0)
    }

    /// The temperature outdoors at `hour`, in °C: the month's normal daily low and
    /// high, the hour of the day, and a departure from normal that lasts several
    /// days at a time (a cold wave, a thaw), rolled from the run's seed. The first
    /// two days keep the normal, so the invasion night is the same in every run.
    pub fn outdoor_c(&self, seed: u64, hour: f32) -> f32 {
        let doy = (clock::FIRST_DAY_OF_YEAR - 1) as f32 + hour.max(0.0) / 24.0;
        let lo = self.normal("mean_min_c", doy);
        let hi = self.normal("mean_max_c", doy);
        let t = clock::time_of_day(hour);
        let min_h = self.num("climate", "min_hour");
        let max_h = self.num("climate", "max_hour");
        // 0 at the day's low, 1 at its high, along half-cosines.
        let shape = if t >= min_h && t <= max_h {
            0.5 - 0.5 * (std::f32::consts::PI * (t - min_h) / (max_h - min_h)).cos()
        } else {
            let since = (t - max_h).rem_euclid(24.0);
            let fall = 24.0 - (max_h - min_h);
            0.5 + 0.5 * (std::f32::consts::PI * since / fall).cos()
        };
        lo + (hi - lo) * shape + self.departure_c(seed, hour)
    }

    /// The departure from normal at `hour`: one value a day, each mostly the day
    /// before's (a cold wave lasts), eased between days.
    pub fn departure_c(&self, seed: u64, hour: f32) -> f32 {
        let day_f = hour.max(0.0) / 24.0;
        let day = day_f.floor() as i64;
        let keep = self.num("climate", "departure_keep");
        let sd = self.num("climate", "departure_sd_c");
        let fresh = (1.0 - keep * keep).sqrt();
        let normalish = |d: i64| -> f32 {
            // About normal, standard deviation 1: three uniform rolls added.
            let s = seed as i64 ^ d.wrapping_mul(0x5851_F42D);
            (hash01(s, 0xC11A) + hash01(s, 0xC11B) + hash01(s, 0xC11C) - 1.5) * 2.0
        };
        // Departures start on the fourth day (20 January); the night and the day
        // after it are the normal in every run.
        const FIRST: i64 = 3;
        let (mut cur, mut today, mut tomorrow) = (0.0f32, 0.0f32, 0.0f32);
        for d in FIRST..=(day + 1) {
            cur = keep * cur + fresh * sd * normalish(d);
            if d == day {
                today = cur;
            }
            if d == day + 1 {
                tomorrow = cur;
            }
        }
        let f = day_f - day as f32;
        let ease = f * f * (3.0 - 2.0 * f);
        today + (tomorrow - today) * ease
    }

    /// The share of what is left in the shops that other people take before
    /// morning `morning` (1 is the first morning, 18 January).
    pub fn shops_taken(&self, morning: u32) -> f32 {
        let curve = self.root.pointer("/shops/taken_by_morning").and_then(|v| v.as_array());
        match curve.and_then(|c| c.get(morning.saturating_sub(1) as usize)).and_then(|v| v.as_f64()) {
            Some(x) => x as f32,
            None => self.num("shops", "taken_after"),
        }
    }

    /// Food a body burns in a day at mean temperature `env_c` (kcal).
    pub fn kcal_per_day(&self, env_c: f32) -> f32 {
        let base = self.num("body", "kcal_base");
        let per = self.num("body", "kcal_per_5c_below_20");
        base + per * ((20.0 - env_c).max(0.0) / 5.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heihe_in_january_and_april() {
        let l = get();
        let at = |hour: f32| l.outdoor_c(1, hour);
        // 17 January: the afternoon near the day's high, the small hours near the low.
        let afternoon = at(16.0);
        let small_hours = at(24.0 + 5.0);
        assert!(afternoon < -10.0 && afternoon > -24.0, "{afternoon}");
        assert!(small_hours < afternoon - 6.0, "{small_hours} {afternoon}");
        // The first two days are the same in every run.
        assert_eq!(l.outdoor_c(1, 20.0), l.outdoor_c(99, 20.0));
        assert_eq!(l.outdoor_c(1, 24.0 + 20.0), l.outdoor_c(99, 24.0 + 20.0));
        assert_ne!(l.outdoor_c(1, 24.0 * 9.0), l.outdoor_c(99, 24.0 * 9.0));
        // Late April is above freezing by day.
        let april_noon = at(24.0 * 100.0 + 14.0);
        assert!(april_noon > 0.0, "{april_noon}");
    }

    #[test]
    fn departures_last_and_stay_in_bounds() {
        let l = get();
        let sd = l.num("climate", "departure_sd_c");
        let days: Vec<f32> = (2..200).map(|d| l.departure_c(7, d as f32 * 24.0)).collect();
        let max = days.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(max < sd * 4.0, "{max}");
        // Next day is closer to today than a random day is.
        let step: f32 = days.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>() / days.len() as f32;
        let spread: f32 = days.iter().map(|v| v.abs()).sum::<f32>() / days.len() as f32;
        assert!(step < spread * 1.2, "{step} {spread}");
    }

    #[test]
    fn colder_needs_more_food() {
        let l = get();
        assert!(l.kcal_per_day(0.0) > l.kcal_per_day(18.0));
        assert_eq!(l.kcal_per_day(20.0), l.num("body", "kcal_base"));
    }
}
