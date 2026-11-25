//! The calendar and the sun over Heihe for the whole of a run.
//!
//! A run's clock is one number, `hour`: hours since 00:00 on 17 January 2031,
//! Beijing time. The invasion starts at hour 16. Day 0 is 17 January, day 1 is
//! the 18th, and so on; `time_of_day` is the hour on the wall clock.
//!
//! The sun is worked out, not drawn by hand, so the days lengthen as the run goes
//! on: sunrise 07:23 and sunset 15:56 on 17 January, 06:42 and 16:47 a month
//! later, 04:16 and 18:41 by late April. The formulas are NOAA's low-precision
//! solar position equations (Spencer's series for declination and the equation
//! of time), good to a fraction of a degree, far finer than anything they drive.

/// Heihe, the city centre (Central Street at the river).
pub const LAT_DEG: f64 = 50.25;
pub const LON_DEG: f64 = 127.5;
/// Beijing time, UTC+8, which Heihe keeps although the sun is half an hour ahead of it.
pub const UTC_OFFSET_H: f64 = 8.0;
/// Day of the year (1-based) of hour 0: 17 January.
pub const FIRST_DAY_OF_YEAR: u32 = 17;
pub const YEAR: u32 = 2031;

const MONTH_DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Days since 17 January (day 0).
pub fn day_index(hour: f32) -> u32 {
    (hour.max(0.0) / 24.0).floor() as u32
}

/// The hour on the wall clock, 0 to 24.
pub fn time_of_day(hour: f32) -> f32 {
    hour.rem_euclid(24.0)
}

/// Month (1 to 12) and day of month of `hour`. 2031 is not a leap year; a run
/// that outlasts it rolls into 2032 as if it were 2031 again.
pub fn date_of(hour: f32) -> (u32, u32) {
    let mut doy = (FIRST_DAY_OF_YEAR - 1 + day_index(hour)) % 365;
    for (i, &len) in MONTH_DAYS.iter().enumerate() {
        if doy < len {
            return (i as u32 + 1, doy + 1);
        }
        doy -= len;
    }
    (12, 31)
}

/// "1月18日 06:40", the way the HUD shows the time.
pub fn label(hour: f32) -> String {
    let (m, d) = date_of(hour);
    let tod = time_of_day(hour);
    // A hair over, so 06:40 built from hours does not print as 06:39.
    let mins = (tod * 60.0 + 1.0e-3).floor() as u32;
    format!("{m}月{d}日 {:02}:{:02}", mins / 60, mins % 60)
}

/// Elevation and azimuth of the sun in degrees at `hour`. Azimuth is clockwise
/// from north.
pub fn sun_at(hour: f32) -> (f32, f32) {
    let hour = hour as f64;
    let day = (hour / 24.0).floor();
    let tod = hour - day * 24.0;
    let doy = FIRST_DAY_OF_YEAR as f64 + day;
    let g = std::f64::consts::TAU / 365.0 * (doy - 1.0 + (tod - UTC_OFFSET_H - 12.0) / 24.0);
    let eq_time_min = 229.18
        * (0.000075 + 0.001868 * g.cos() - 0.032077 * g.sin() - 0.014615 * (2.0 * g).cos() - 0.040849 * (2.0 * g).sin());
    let decl = 0.006918 - 0.399912 * g.cos() + 0.070257 * g.sin() - 0.006758 * (2.0 * g).cos() + 0.000907 * (2.0 * g).sin()
        - 0.002697 * (3.0 * g).cos()
        + 0.00148 * (3.0 * g).sin();
    let solar_min = tod * 60.0 + eq_time_min + 4.0 * LON_DEG - 60.0 * UTC_OFFSET_H;
    let ha = (solar_min / 4.0 - 180.0).to_radians();
    let phi = LAT_DEG.to_radians();
    let cos_zen = phi.sin() * decl.sin() + phi.cos() * decl.cos() * ha.cos();
    let elevation = cos_zen.clamp(-1.0, 1.0).asin().to_degrees();
    let azimuth = ha.sin().atan2(ha.cos() * phi.sin() - decl.tan() * phi.cos()).to_degrees() + 180.0;
    (elevation as f32, azimuth as f32)
}

/// While the sun is higher than this, it is day for anyone looking. It is the sun's
/// height at 16:30 on 17 January, where the night's light rules were set
/// (patrols and drones see by day until then).
pub const DAY_SUN_DEG: f32 = -5.56;
/// Below this it is full night. The sun's height at 17:15 on 17 January.
pub const NIGHT_SUN_DEG: f32 = -12.21;

/// True from 19:00 to 09:00, when the cold is the night's: the sun is down or too
/// low to warm anything, and the coldest hour is just after sunrise.
pub fn night_cold(hour: f32) -> bool {
    let t = time_of_day(hour);
    t > 19.0 || t < 9.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hm(h: u32, m: u32) -> f32 {
        h as f32 + m as f32 / 60.0
    }

    #[test]
    fn heihe_sun_on_17_january() {
        let (noon, az) = sun_at(hm(11, 40));
        assert!((noon - 18.8).abs() < 0.3, "noon elevation {noon}");
        assert!((az - 180.0).abs() < 2.0, "noon azimuth {az}");
        // Sunrise about 07:23, sunset about 15:56 (refraction: -0.83 degrees).
        assert!(sun_at(hm(7, 18)).0 < -0.83 && sun_at(hm(7, 28)).0 > -0.83);
        assert!(sun_at(hm(15, 51)).0 > -0.83 && sun_at(hm(16, 1)).0 < -0.83);
        // The thresholds are the sun's heights at 16:30 and 17:15.
        assert!(sun_at(hm(16, 29)).0 > DAY_SUN_DEG && sun_at(hm(16, 31)).0 < DAY_SUN_DEG);
        assert!(sun_at(hm(17, 14)).0 > NIGHT_SUN_DEG && sun_at(hm(17, 16)).0 < NIGHT_SUN_DEG);
    }

    #[test]
    fn days_lengthen_into_spring() {
        let day_len = |d: f32| (0..24 * 60).filter(|m| sun_at(d * 24.0 + *m as f32 / 60.0).0 > -0.83).count();
        let jan = day_len(0.0);
        let feb = day_len(30.0);
        let apr = day_len(100.0);
        assert!(jan < feb && feb < apr, "{jan} {feb} {apr}");
        assert!((500..540).contains(&jan), "17 January has about 8 h 33 min of sun: {jan}");
    }

    #[test]
    fn calendar() {
        assert_eq!(date_of(16.0), (1, 17));
        assert_eq!(date_of(31.0), (1, 18));
        assert_eq!(date_of(24.0 * 15.0), (2, 1));
        assert_eq!(date_of(24.0 * 43.0), (3, 1));
        assert_eq!(day_index(31.0), 1);
        assert_eq!(label(hm(30, 40)), "1月18日 06:40");
        assert!((time_of_day(31.5) - 7.5).abs() < 1e-4);
    }
}
