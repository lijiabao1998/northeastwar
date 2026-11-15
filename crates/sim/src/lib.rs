//! The game's world and rules, with no renderer in sight. Anything that can
//! be checked without a window lives here, and is checked.

pub mod city;
pub mod facade;
pub mod forces;
pub mod geom;
pub mod look;
pub mod scene;
pub mod soup;
pub mod walk;

use std::path::PathBuf;

/// The repository's data folder, wherever the program was started from.
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("NORTHEAST_DATA") {
        return PathBuf::from(dir);
    }
    let here = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let mut tries = vec![PathBuf::from("data"), PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data"))];
    if let Some(here) = here {
        tries.insert(0, here.join("data"));
    }
    for dir in &tries {
        if dir.join("heihe/city.json").exists() {
            // Absolute, so the folder beside it (assets) can be found from anywhere.
            return std::path::absolute(dir).unwrap_or_else(|_| dir.clone());
        }
    }
    tries[0].clone()
}

pub fn load_heihe() -> Result<(city::City, look::Palette), String> {
    let dir = data_dir();
    let city_text = std::fs::read_to_string(dir.join("heihe/city.json")).map_err(|e| format!("city.json: {e}"))?;
    let look_text = std::fs::read_to_string(dir.join("heihe/look.json")).map_err(|e| format!("look.json: {e}"))?;
    Ok((city::City::from_json(&city_text)?, look::Palette::from_json(&look_text)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::Bank;
    use crate::geom::{point_in_ring, signed_area, P2};
    use crate::look::Kind;

    fn heihe() -> (city::City, look::Palette) {
        load_heihe().expect("the repository's Heihe data")
    }

    #[test]
    fn heihe_loads_whole() {
        let (city, _) = heihe();
        let cn = city.buildings.iter().filter(|b| b.bank == Bank::China).count();
        assert!(cn >= 150, "{cn} Chinese buildings");
        assert!(city.roads.iter().any(|r| r.name == "中央街"));
        assert!(city.roads.iter().any(|r| r.name == "王肃街"));
        assert!(city.signs.len() >= 60);
        assert!(city.lamps.len() >= 200);
        assert!(city.landmarks.iter().any(|l| l.name_zh == "母亲"));
        assert!(city.attribution.contains("OpenStreetMap"));
        for b in &city.buildings {
            assert!(signed_area(&b.footprint) > 0.0, "footprint {} must be counter-clockwise", b.id);
            assert!(b.height > 2.0 && b.height < 200.0);
        }
    }

    #[test]
    fn the_river_is_north_of_the_city() {
        let (city, _) = heihe();
        // The Mother sculpture stands at the origin, on the bank, and Central Street is south of it.
        let central = city.road("中央街").unwrap();
        assert!(central.line.iter().all(|p| p.y < city.bank_y(p.x) - 100.0));
        assert!(city.ice_y(0.0) > city.bank_y(0.0));
        // Blagoveshchensk is across the ice, to the north.
        let far: Vec<_> = city.buildings.iter().filter(|b| b.bank == Bank::Russia).collect();
        assert!(!far.is_empty());
        assert!(far.iter().all(|b| b.centroid.y > city.bank_y(b.centroid.x)));
    }

    #[test]
    fn every_colour_names_its_photograph() {
        let (_, pal) = heihe();
        assert!(pal.len() >= 40);
        for (name, from) in &pal.sources {
            assert!(!from.is_empty(), "{name} has no photograph");
        }
        assert_eq!(pal.boards.len(), 5);
        for needed in ["central_salmon", "resi_yellow", "resi_crimson", "tile_tan", "baroque_red", "mother_bronze", "mother_plinth", "river_ice", "snow", "spire_red", "dome_green"] {
            assert!(pal.has(needed), "{needed}");
        }
    }

    #[test]
    fn central_street_is_dressed_as_central_street() {
        let (city, _) = heihe();
        let mut counts = std::collections::HashMap::new();
        for b in city.buildings.iter().filter(|b| b.bank == Bank::China) {
            let d = look::dress(b, &city);
            *counts.entry(d.kind).or_insert(0) += 1;
            // The same building is always dressed the same way.
            assert_eq!(d.kind, look::dress(b, &city).kind);
        }
        let central = counts.get(&Kind::CentralSalmon).copied().unwrap_or(0)
            + counts.get(&Kind::CentralBaroque).copied().unwrap_or(0)
            + counts.get(&Kind::CentralClassical).copied().unwrap_or(0);
        assert!(central >= 6, "{central} buildings front Central Street");
        assert!(counts.get(&Kind::YellowCrimson).copied().unwrap_or(0) >= 20);
        assert!(counts.get(&Kind::TanTile).copied().unwrap_or(0) >= 20);
        assert!(!counts.contains_key(&Kind::FarBank));
    }

    #[test]
    fn buildings_come_out_closed_and_facing_outward() {
        let (city, pal) = heihe();
        let mut triangles = 0;
        for b in city.buildings.iter().filter(|b| b.bank == Bank::China).take(60) {
            let built = facade::build(b, look::dress(b, &city), &pal);
            triangles += built.solid.triangles() + built.glass.triangles() + built.lit.triangles();
            assert!(built.solid.triangles() > 8);
            // No piece of a wall may face into the building.
            for soup in [&built.glass, &built.lit] {
                for t in soup.idx.chunks(3) {
                    let (a, bb, c) = (soup.pos[t[0] as usize], soup.pos[t[1] as usize], soup.pos[t[2] as usize]);
                    let u = [bb[0] - a[0], bb[1] - a[1], bb[2] - a[2]];
                    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                    let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                    let s = soup.nor[t[0] as usize];
                    assert!(n[0] * s[0] + n[1] * s[1] + n[2] * s[2] > 0.0);
                    // A window is outside the footprint, not inside it.
                    let mid = P2::new((a[0] + bb[0] + c[0]) / 3.0, -(a[2] + bb[2] + c[2]) / 3.0);
                    assert!(!point_in_ring(mid, &b.footprint), "a window of {} is inside the wall", b.id);
                }
            }
        }
        assert!(triangles > 20_000, "{triangles} triangles in sixty buildings");
    }

    #[test]
    fn the_scene_has_the_city_in_it() {
        let (city, pal) = heihe();
        let scene = scene::build(&city, &pal);
        assert!(scene.buildings.triangles() > 60_000, "{}", scene.buildings.triangles());
        assert!(scene.ground.triangles() > 500);
        assert!(scene.props.triangles() > 20_000);
        assert!(scene.glass.triangles() + scene.lit.triangles() > 20_000);
        assert!(scene.lamp_heads.len() >= 300);
        assert!(scene.boards.len() >= 50, "{} boards", scene.boards.len());
        for b in &scene.boards {
            // Reading left to right, then up, must come out of the wall toward you:
            // otherwise the lettering is mirror writing.
            let r = b.right;
            let n = b.normal;
            let up = [0.0, 1.0, 0.0];
            let out = [r[1] * up[2] - r[2] * up[1], r[2] * up[0] - r[0] * up[2], r[0] * up[1] - r[1] * up[0]];
            assert!(out[0] * n[0] + out[1] * n[1] + out[2] * n[2] > 0.99, "board {} reads backwards", b.zh);
            assert!(b.width >= 3.0 && b.height > 0.5);
        }
        // No two boards overlap on a wall.
        for (i, a) in scene.boards.iter().enumerate() {
            for b in &scene.boards[i + 1..] {
                let same_way = a.normal[0] * b.normal[0] + a.normal[2] * b.normal[2] > 0.9;
                let d = ((a.centre[0] - b.centre[0]).powi(2) + (a.centre[2] - b.centre[2]).powi(2)).sqrt();
                assert!(!same_way || d >= (a.width + b.width) * 0.5, "{} hangs over {}", a.zh, b.zh);
            }
        }
        // Nothing may be the colour of a missing name.
        for soup in [&scene.ground, &scene.buildings, &scene.props] {
            assert!(!soup.col.iter().any(|c| c[0] > 0.99 && c[1] < 0.01 && c[2] > 0.99));
        }
    }

    #[test]
    fn you_cannot_walk_through_a_building() {
        let (city, _) = heihe();
        let ground = walk::Ground::new(&city);
        assert!(ground.solid_count() >= 150);
        let b = city.buildings.iter().find(|b| b.solid && b.bank == Bank::China).unwrap();
        // Start outside, walk straight at the middle of the building.
        let start = b.centroid.add(P2::new(0.0, -120.0));
        let mut at = if ground.free(start, 0.35) { start } else { city.spawn };
        for _ in 0..4000 {
            let dir = b.centroid.sub(at).norm();
            at = ground.step(at, at.add(dir.mul(0.1)), 0.35);
        }
        assert!(!point_in_ring(at, &b.footprint), "walked into building {}", b.id);
        // The spawn point is somewhere you can stand.
        assert!(ground.free(city.spawn, 0.35));
        // And the main channel is out of bounds.
        let far = ground.step(P2::new(0.0, city.bank_y(0.0)), P2::new(0.0, 5000.0), 0.35);
        assert!(far.y <= city.ice_y(0.0) + 0.01);
    }
}
