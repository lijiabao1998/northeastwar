//! Saving and loading a run. A run lasts from a month to four years
//! (docs/DESIGN.md, "Time"), so the whole world is saved, not a summary: what you
//! carry, every room's temperature and what is hidden in it, the patrols, the
//! occupation's schedule, the random generator. A loaded run goes on exactly as
//! the saved one would have.
//!
//! A save is the world as JSON, gzipped, with a format number in front.

use crate::survival::WorldSim;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

/// Bumped whenever the saved world's shape changes; older saves are refused
/// rather than loaded wrong.
pub const FORMAT: u32 = 1;

#[derive(Serialize)]
struct Out<'a> {
    format: u32,
    world: &'a WorldSim,
}

#[derive(Deserialize)]
struct In {
    format: u32,
    world: WorldSim,
}

pub fn to_bytes(sim: &WorldSim) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(&Out { format: FORMAT, world: sim }).map_err(|e| format!("save: {e}"))?;
    let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
    gz.write_all(&json).map_err(|e| format!("save: {e}"))?;
    gz.finish().map_err(|e| format!("save: {e}"))
}

pub fn from_bytes(bytes: &[u8]) -> Result<WorldSim, String> {
    let mut json = Vec::new();
    GzDecoder::new(bytes).read_to_end(&mut json).map_err(|e| format!("load: {e}"))?;
    let peek: serde_json::Value = serde_json::from_slice(&json).map_err(|e| format!("load: {e}"))?;
    let format = peek.get("format").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if format != FORMAT {
        return Err(format!("load: this save is format {format}, the game reads format {FORMAT}"));
    }
    let parsed: In = serde_json::from_value(peek).map_err(|e| format!("load: {e}"))?;
    debug_assert_eq!(parsed.format, FORMAT);
    Ok(parsed.world)
}

/// Where saves live: `saves/` beside the game's `data/` folder.
pub fn dir() -> PathBuf {
    let data = crate::data_dir();
    data.parent().map(|p| p.join("saves")).unwrap_or_else(|| PathBuf::from("saves"))
}

pub fn path(slot: &str) -> PathBuf {
    dir().join(format!("{slot}.sav"))
}

/// Write the world to `slot`, through a temporary file so a crash mid-write
/// leaves the old save whole.
pub fn write(sim: &WorldSim, slot: &str) -> Result<PathBuf, String> {
    let bytes = to_bytes(sim)?;
    std::fs::create_dir_all(dir()).map_err(|e| format!("save: {e}"))?;
    let target = path(slot);
    let tmp = target.with_extension("sav.tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("save: {e}"))?;
    std::fs::rename(&tmp, &target).map_err(|e| format!("save: {e}"))?;
    Ok(target)
}

pub fn read(slot: &str) -> Result<WorldSim, String> {
    let bytes = std::fs::read(path(slot)).map_err(|e| format!("load: {e}"))?;
    from_bytes(&bytes)
}

pub fn exists(slot: &str) -> bool {
    path(slot).is_file()
}

/// The slots a run is saved to: every morning by itself, and when you ask.
pub const SLOTS: [&str; 2] = ["auto", "manual"];

/// The newest save there is, if any.
pub fn latest() -> Option<&'static str> {
    SLOTS
        .iter()
        .filter_map(|s| std::fs::metadata(path(s)).and_then(|m| m.modified()).ok().map(|t| (*s, t)))
        .max_by_key(|(_, t)| *t)
        .map(|(s, _)| s)
}

/// One life: when you die, the run's saves go with you.
pub fn delete_all() {
    for s in SLOTS {
        let _ = std::fs::remove_file(path(s));
    }
}

/// Text the program carries as `&'static str`. Fields of this type are saved
/// through `static_str`; the alias keeps serde from taking them for text borrowed
/// from the save itself.
pub type StaticText = &'static str;

/// Text the program carries as `&'static str` (cache titles, names, messages)
/// is saved as text and read back into memory that lives as long as the game.
/// Each load keeps a few hundred bytes of these.
pub mod static_str {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(s: &&'static str, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<&'static str, D::Error> {
        let s = String::deserialize(de)?;
        Ok(if s.is_empty() { "" } else { Box::leak(s.into_boxed_str()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forces::Forces;
    use crate::geom::P2;
    use crate::survival::SimInput;
    use crate::walk::Ground;

    fn forces() -> Forces {
        Forces::from_json(&std::fs::read_to_string(crate::data_dir().join("russia/forces.json")).unwrap()).unwrap()
    }

    fn digest(sim: &WorldSim) -> String {
        serde_json::to_string(sim).unwrap()
    }

    #[test]
    fn a_loaded_run_goes_on_exactly_as_the_saved_one() {
        let (city, _) = crate::load_heihe().unwrap();
        let forces = forces();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let mut sim = WorldSim::with_seed(&city, 11);
        sim.hour = 19.5;
        let walk = |i: usize| SimInput {
            wish_move: if i % 300 < 150 { P2::new(0.4, -1.0) } else { P2::new(-1.0, 0.2) },
            clock_running: true,
            ..Default::default()
        };
        for i in 0..1200 {
            sim.tick(&city, &ground, &forces, &walk(i), 0.25);
        }
        let t0 = std::time::Instant::now();
        let bytes = to_bytes(&sim).unwrap();
        let saved_ms = t0.elapsed().as_millis();
        let t1 = std::time::Instant::now();
        let mut loaded = from_bytes(&bytes).unwrap();
        eprintln!("save: {} bytes, written in {saved_ms} ms, read in {} ms", bytes.len(), t1.elapsed().as_millis());
        assert_eq!(digest(&sim), digest(&loaded), "the save holds the whole world");
        for i in 1200..2400 {
            sim.tick(&city, &ground, &forces, &walk(i), 0.25);
            loaded.tick(&city, &ground, &forces, &walk(i), 0.25);
        }
        assert_eq!(digest(&sim), digest(&loaded), "and both go on the same, step for step");
        assert!(bytes.len() < 8_000_000, "a save is {} bytes", bytes.len());
    }

    #[test]
    fn a_save_from_another_format_is_refused() {
        let (city, _) = crate::load_heihe().unwrap();
        let sim = WorldSim::new(&city);
        let json = serde_json::to_vec(&serde_json::json!({ "format": FORMAT + 1, "world": sim })).unwrap();
        let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
        gz.write_all(&json).unwrap();
        let err = from_bytes(&gz.finish().unwrap()).err().unwrap();
        assert!(err.contains("format"), "{err}");
    }
}
