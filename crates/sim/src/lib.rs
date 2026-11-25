//! The game's world and rules, with no renderer in sight. Anything that can
//! be checked without a window lives here, and is checked.

pub mod arsenal;
pub mod city;
pub mod clock;
pub mod cmab;
pub mod cutaway;
pub mod facade;
pub mod figure;
pub mod forces;
pub mod geom;
pub mod interior;
pub mod living;
pub mod look;
pub mod occupation;
pub mod scene;
pub mod soup;
pub mod survival;
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

/// The OpenStreetMap city alone, as tools/osm_import wrote it.
pub fn load_osm_heihe() -> Result<city::City, String> {
    let text = std::fs::read_to_string(data_dir().join("heihe/city.json")).map_err(|e| format!("city.json: {e}"))?;
    city::City::from_json(&text)
}

pub fn load_cmab_heihe() -> Result<cmab::Cmab, String> {
    let text = std::fs::read_to_string(data_dir().join("heihe/cmab.json")).map_err(|e| format!("cmab.json: {e}"))?;
    cmab::Cmab::from_json(&text)
}

/// Heihe as the game uses it: OpenStreetMap, filled in from CMAB, and its colours.
pub fn load_heihe() -> Result<(city::City, look::Palette), String> {
    let mut city = load_osm_heihe()?;
    let cmab = load_cmab_heihe()?;
    cmab.fill(&mut city);
    let look_text = std::fs::read_to_string(data_dir().join("heihe/look.json")).map_err(|e| format!("look.json: {e}"))?;
    Ok((city, look::Palette::from_json(&look_text)?))
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
        // Sixty from each source: OpenStreetMap's and CMAB's.
        let osm = city.buildings.iter().filter(|b| b.bank == Bank::China && b.id < cmab::ID_BASE).take(60);
        let filled = city.buildings.iter().filter(|b| b.id >= cmab::ID_BASE).take(60);
        for b in osm.chain(filled) {
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
        let block_tris: usize = scene.blocks.iter().map(|b| b.solid.triangles()).sum();
        assert!(block_tris + scene.buildings.triangles() > 60_000, "{}", block_tris + scene.buildings.triangles());
        assert!(scene.blocks.len() >= 150);
        assert!(scene.ground.triangles() > 500);
        assert!(scene.props.triangles() > 20_000);
        let windows: usize = scene.blocks.iter().map(|b| b.glass.triangles() + b.lit.triangles()).sum();
        assert!(windows > 20_000);
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
        for soup in [&scene.ground, &scene.buildings, &scene.props].into_iter().chain(scene.blocks.iter().map(|b| &b.solid)) {
            assert!(!soup.col.iter().any(|c| c[0] > 0.99 && c[1] < 0.01 && c[2] > 0.99));
        }
    }

    #[test]
    fn cmab_fills_the_town_without_covering_the_map() {
        let mut city = load_osm_heihe().unwrap();
        let osm_cn = city.buildings.iter().filter(|b| b.bank == Bank::China).count();
        let osm_footprints: Vec<_> = city.buildings.iter().filter(|b| b.bank == Bank::China).map(|b| b.footprint.clone()).collect();
        let cmab = load_cmab_heihe().unwrap();
        let tally = cmab.fill(&mut city);
        let all_cn = city.buildings.iter().filter(|b| b.bank == Bank::China).count();
        assert_eq!(all_cn, osm_cn + tally.kept);
        assert_eq!(tally.read, tally.kept + tally.under_osm + tally.on_street + tally.on_ice + tally.too_small + tally.bad_shape + tally.on_riverfront);
        // The Mother sculpture stands in the open, as in the photograph.
        for l in &city.landmarks {
            assert!(!city.buildings.iter().any(|b| b.bank == Bank::China && point_in_ring(l.at, &b.footprint)), "a building stands on {}", l.name_zh);
        }
        // No street runs through a CMAB building.
        for b in city.buildings.iter().filter(|b| b.id >= cmab::ID_BASE) {
            for r in city.roads.iter().filter(|r| r.width >= 4.0 || r.pedestrian()) {
                assert!(geom::length_inside(&r.line, &b.footprint) < cmab::THROUGH, "{} runs through {}", r.name, b.id);
            }
        }
        assert!(tally.kept >= 3 * osm_cn, "only {} CMAB buildings kept: {:?}", tally.kept, tally);
        let added: Vec<_> = city.buildings.iter().filter(|b| b.id >= cmab::ID_BASE).collect();
        for b in &added {
            // Never on the ice, never mostly under an OpenStreetMap building.
            assert!(b.centroid.y < city.bank_y(b.centroid.x), "{} is on the river", b.id);
            let samples = cmab::inside_points(&b.footprint, 2.0);
            let under = samples.iter().filter(|p| osm_footprints.iter().any(|r| point_in_ring(**p, r))).count();
            assert!((under as f32) < samples.len() as f32 * cmab::OSM_COVER, "{} sits on an OSM building", b.id);
            assert!(signed_area(&b.footprint) >= cmab::MIN_AREA);
            assert!(b.height >= 3.0 && b.height <= 90.0, "{} is {} m", b.id, b.height);
        }
        // Heights: the six-to-seven-storey town of the photographs, a few towers far
        // off, and none of the importer's hashed towers left.
        let mut levels: Vec<u32> = city.buildings.iter().filter(|b| b.bank == Bank::China && b.levels > 2).map(|b| b.levels).collect();
        levels.sort();
        let median = levels[levels.len() / 2];
        assert!((5..=7).contains(&median), "median {median} storeys");
        let (wx0, wx1) = city.walk_x;
        for b in city.buildings.iter().filter(|b| b.bank == Bank::China) {
            assert!(b.levels <= 18 && (b.height - b.levels as f32 * b.floor_h).abs() < 0.01);
            if b.id < cmab::ID_BASE && b.height_guessed {
                assert!(b.levels < 11, "OSM {} kept a hashed tower of {} storeys", b.id, b.levels);
            }
            let inside = b.centroid.x > wx0 && b.centroid.x < wx1 && b.centroid.y > city.walk_south;
            assert!(!inside || b.levels <= 10, "{} is {} storeys inside the playable area", b.id, b.levels);
        }
        // 77 with the shift; without it 55 (review, 16 November 2026). The exact tally is pinned in cmab.rs.
        assert!(tally.osm_heights >= 70, "only {} OSM heights corrected", tally.osm_heights);
        // Every wall gets drawn: no CMAB outline keeps edges too short for a wall.
        for b in &added {
            let n = b.footprint.len();
            let perimeter: f32 = (0..n).map(|i| b.footprint[i].dist(b.footprint[(i + 1) % n])).sum();
            let drawn: f32 = (0..n).map(|i| b.footprint[i].dist(b.footprint[(i + 1) % n])).filter(|l| *l >= 0.3).sum();
            assert!(drawn >= perimeter * 0.995, "{} has {:.1} of {:.1} m of wall missing", b.id, perimeter - drawn, perimeter);
        }
        // Every outer ring in the file is a roof that was counted.
        let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(data_dir().join("heihe/cmab.json")).unwrap()).unwrap();
        let mut outers = 0;
        for b in raw["buildings"].as_array().unwrap() {
            for (k, r) in b["rings"].as_array().unwrap().iter().enumerate() {
                let ring: Vec<P2> = r.as_array().unwrap().iter().map(|p| P2::new(p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32)).collect();
                if k == 0 || signed_area(&ring) < 0.0 {
                    outers += 1;
                }
            }
        }
        assert_eq!(tally.read, outers);
        assert!(added.iter().any(|b| (b.id - cmab::ID_BASE) >= cmab::PART), "no second part of a record was kept");
        for b in &added {
            // Solid, like every OpenStreetMap building on this bank.
            assert!(b.solid, "{} can be walked into", b.id);
            assert!(cmab::min_width(&b.footprint) >= cmab::MIN_WIDTH, "{} is a sliver", b.id);
            // OSM wins seen from its own side too: no CMAB roof takes in most of an OSM building.
            for osm in &osm_footprints {
                let s = cmab::inside_points(osm, 2.0);
                let inside = s.iter().filter(|p| point_in_ring(**p, &b.footprint)).count();
                assert!((inside as f32) < s.len() as f32 * cmab::OSM_COVER, "{} swallows an OSM building", b.id);
            }
        }
        // Central Street stays a street: its centreline is open ground.
        let central = city.road("中央街").unwrap().clone();
        let solid: Vec<_> = city.buildings.iter().filter(|b| b.bank == Bank::China).collect();
        for w in central.line.windows(2) {
            for k in 0..20 {
                let p = w[0].lerp(w[1], k as f32 / 20.0);
                assert!(!solid.iter().any(|b| point_in_ring(p, &b.footprint)), "a building stands on Central Street at {:?}", p);
            }
        }
        // You still start somewhere you can stand.
        let ground = walk::Ground::new(&city);
        assert!(ground.free(city.spawn, 0.35));
    }

    #[test]
    fn crowns_stand_inside_their_building_and_are_what_is_drawn() {
        // Every corner of a crown's drum is inside the outline, and the crown the cut-away
        // judges is the one that is drawn (review, 16 November 2026: the drum stood 0.6 m
        // out of the facade, and the cut-away judged a 4 m column instead of the crown).
        let (city, pal) = heihe();
        let scene = scene::build(&city, &pal);
        let key = |t: &[[f32; 3]; 3]| -> [u32; 9] {
            let mut k = [0u32; 9];
            for (i, v) in t.iter().flatten().enumerate() {
                k[i] = v.to_bits();
            }
            k
        };
        let mut crowned = 0;
        for b in &scene.blocks {
            let building = city.buildings.iter().find(|o| o.id == b.id).unwrap();
            let dress = look::dress(building, &city);
            let drum = facade::crown_drum(dress.kind).filter(|_| dress.crown);
            let place = drum.and_then(|(w, _)| facade::crown_at(&b.footprint, w * 0.5));
            assert_eq!(place.is_some(), !b.crown.is_empty(), "{}: a crown is placed but not judged, or judged but not placed", b.id);
            let (Some((w, _)), Some((centre, yaw))) = (drum, place) else { continue };
            crowned += 1;
            let (ax, ay) = (P2::new(yaw.cos(), yaw.sin()), P2::new(-yaw.sin(), yaw.cos()));
            for (u, v) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let p = centre.add(ax.mul(u * w * 0.5)).add(ay.mul(v * w * 0.5));
                assert!(point_in_ring(p, &b.footprint), "the drum of {} stands out of it at {:?}", b.id, p);
            }
            let foot: Vec<P2> = b.crown.iter().flatten().filter(|p| (p[1] - b.height).abs() < 1.0e-3).map(|p| P2::new(p[0], -p[2])).collect();
            assert!(!foot.is_empty(), "the crown of {} does not stand on its roof", b.id);
            for p in foot {
                assert!(point_in_ring(p, &b.footprint), "the drum of {} is drawn out of it at {:?}", b.id, p);
            }
            let drawn: std::collections::HashSet<[u32; 9]> = b.solid.triangle_list().iter().map(key).collect();
            assert!(b.crown.iter().all(|t| drawn.contains(&key(t))), "the crown of {} is judged by triangles it does not draw", b.id);
        }
        eprintln!("{crowned} crowns");
        assert!(crowned >= 3, "{crowned} crowns");
    }

    #[test]
    fn both_crown_kinds_reach_their_tips() {
        // Heihe's data has no crowned baroque block, so its branch runs on a made-up one.
        let (_, pal) = heihe();
        let ring = vec![P2::new(0.0, 0.0), P2::new(30.0, 0.0), P2::new(30.0, 20.0), P2::new(0.0, 20.0)];
        let b = city::Building {
            id: 1,
            bank: Bank::China,
            levels: 8,
            height: 24.0,
            floor_h: 3.0,
            name_zh: String::new(),
            name_ru: String::new(),
            centroid: cmab::centroid(&ring),
            footprint: ring.clone(),
            solid: true,
            function: String::new(),
            height_guessed: false,
        };
        for (kind, tip) in [(Kind::CentralBaroque, 5.2), (Kind::CentralSalmon, 7.6)] {
            let built = facade::build(&b, look::Dress { kind, shops: true, crown: true }, &pal);
            assert!(!built.crown.is_empty(), "{kind:?} drew no crown");
            let (lo, hi) = built.crown.iter().flatten().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
            assert!((lo - 24.0).abs() < 1.0e-3 && (hi - (24.0 + tip)).abs() < 1.0e-3, "{kind:?}: {lo} to {hi}");
            for p in built.crown.iter().flatten().filter(|p| p[1] < 24.0 + 1.0e-3) {
                assert!(point_in_ring(P2::new(p[0], -p[2]), &ring), "{kind:?}: the drum stands out at {p:?}");
            }
        }
        let built = facade::build(&b, look::Dress { kind: Kind::TanTile, shops: false, crown: true }, &pal);
        assert!(built.crown.is_empty(), "a housing block drew a crown");
    }

    #[test]
    fn the_building_behind_you_is_not_cut_away() {
        // On 邮政路 by the 华夏宾馆 board: the wall behind you runs at a slant, and a check that
        // let your shoulders start inside it cut the whole hotel away (16 November 2026).
        // Judged as the game judges it: by each block's Occluder.
        let (city, pal) = heihe();
        let scene = scene::build(&city, &pal);
        let ground = walk::Ground::new(&city);
        let view = cutaway::View::from_offset(cutaway::EYE_OFFSET);
        let at = ground.step(P2::new(-139.0, -221.0), P2::new(-139.0, -221.0), 0.35);
        let hidden: Vec<i64> = scene.blocks.iter().filter(|b| cutaway::Occluder::of_block(b, &view).hides(at, &view, 2.5)).map(|b| b.id).collect();
        assert!(scene.blocks.iter().any(|b| b.id == 1079423127), "the hotel is not in the scene");
        assert!(!hidden.contains(&1079423127), "the hotel behind you was cut away");
    }

    #[test]
    fn a_crown_hides_you_where_its_roof_does_not() {
        // The crowns stand above the roof, so the cut-away judges them too, by their own
        // triangles (review, 16 November 2026). A 4 m column standing in for them cut the
        // building away on the air beside the spire; that must not come back.
        let (city, pal) = heihe();
        let scene = scene::build(&city, &pal);
        let view = cutaway::View::from_offset(cutaway::EYE_OFFSET);
        let crowned: Vec<&scene::Block> = scene.blocks.iter().filter(|b| !b.crown.is_empty()).collect();
        assert!(!crowned.is_empty(), "no crowns on Central Street");
        let (mut only_crown, mut column_on_air) = (0, 0);
        for b in crowned {
            let occluder = cutaway::Occluder::of_block(b, &view);
            let pts: Vec<P2> = b.crown.iter().flatten().map(|p| P2::new(p[0], -p[2])).collect();
            let c = P2::new(pts.iter().map(|p| p.x).sum::<f32>() / pts.len() as f32, pts.iter().map(|p| p.y).sum::<f32>() / pts.len() as f32);
            let tip = b.crown.iter().flatten().map(|p| p[1]).fold(0.0, f32::max);
            let column = cutaway::Prism::new(&[P2::new(c.x - 2.0, c.y - 2.0), P2::new(c.x + 2.0, c.y - 2.0), P2::new(c.x + 2.0, c.y + 2.0), P2::new(c.x - 2.0, c.y + 2.0)], tip, &view);
            // The camera is to the south-east: the crown hides what is to its north-west.
            for i in 0..=60 {
                for j in 0..=60 {
                    let at = c.add(P2::new(-(i as f32), j as f32));
                    let by_body = occluder.body.hides(at, &view, 2.5);
                    if occluder.hides(at, &view, 2.5) && !by_body {
                        only_crown += 1;
                    }
                    if column.hides(at, &view, 2.5) && !occluder.hides(at, &view, 2.5) {
                        column_on_air += 1;
                    }
                }
            }
        }
        assert!(only_crown > 0, "nowhere does a crown hide you that its building does not");
        assert!(column_on_air > 0, "the crown is judged no differently from a column");
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

    #[test]
    fn no_cmab_building_on_the_promenade_or_mother_square() {
        // scene.rs paves the promenade 0.5 to 9.5 m in from the bank line and the
        // square 26 m either side of the sculpture, 26 m south and 13 m north.
        let (city, _) = heihe();
        let mother = city.landmarks.iter().find(|l| l.kind == "sculpture").unwrap().at;
        let (x0, x1) = (city.bank[0].x, city.bank[city.bank.len() - 1].x);
        let mut worst = f32::MAX;
        for b in city.buildings.iter().filter(|b| b.id >= cmab::ID_BASE) {
            let mut pts = cmab::inside_points(&b.footprint, 1.0);
            pts.extend(b.footprint.iter().copied());
            for p in pts {
                if p.x >= x0 && p.x <= x1 {
                    let d = geom::dist_to_polyline(p, &city.bank);
                    worst = worst.min(d);
                    assert!(d >= cmab::PROMENADE, "{} reaches {:.1} m from the bank at {:?}", b.id, d, p);
                }
                let on_square = p.x > mother.x - 26.0 && p.x < mother.x + 26.0 && p.y > mother.y - 26.0 && p.y < mother.y + 13.0;
                assert!(!on_square, "{} stands on Mother Square at {:?}", b.id, p);
            }
        }
        eprintln!("nearest CMAB point to the bank: {worst:.2} m");
    }

    #[test]
    fn the_game_gets_the_filled_town_one_block_a_building() {
        let (city, pal) = heihe();
        let cn = city.buildings.iter().filter(|b| b.bank == Bank::China).count();
        assert!(city.buildings.iter().any(|b| b.id >= cmab::ID_BASE), "load_heihe left CMAB out");
        let scene = scene::build(&city, &pal);
        assert_eq!(scene.blocks.len(), cn);
    }

    #[test]
    fn heihe_has_its_landmarks_and_winter_props() {
        let (city, pal) = heihe();
        let expected_keys = [
            "memorial_rose",
            "curtain_green",
            "curtain_maroon",
            "ac_white",
            "awning_teal",
            "tactile_yellow",
            "tree_lime",
            "tree_stake",
            "medallion_red",
            "topiary_green",
            "bag_red",
            "bench_wood",
            "chimney_brick",
            "trike_teal",
            "camo_cover",
            "pole_concrete",
            "wire_dark",
            "wheel_white",
            "rail_iron",
        ];
        for k in expected_keys {
            assert!(pal.has(k), "missing R4 colour {k} in look.json");
        }

        let scene = scene::build(&city, &pal);
        let chunks = scene.props.chunks(120.0);
        assert!(chunks.len() >= 20, "expected >= 20 120m prop chunks, got {}", chunks.len());
        let chunk_tris: usize = chunks.iter().map(|c| c.triangles()).sum();
        assert_eq!(chunk_tris, scene.props.triangles(), "chunks must preserve every triangle");

        let tris = scene.props.triangle_list();
        let has_near = |cx: f32, cy: f32, max_r: f32, min_y: f32| -> bool {
            tris.iter().any(|t| {
                t.iter().any(|v| {
                    let dx = v[0] - cx;
                    let dy = -v[2] - cy;
                    dx * dx + dy * dy <= max_r * max_r && v[1] > min_y
                })
            })
        };

        assert!(has_near(-159.3, -246.5, 10.0, 7.0), "missing Wenhua St teal arch");
        assert!(has_near(-116.4, -156.4, 10.0, 7.0), "missing Hailan St teal arch");
        assert!(has_near(105.0, -35.0, 22.0, 40.0), "missing riverside Ferris wheel");
        assert!(has_near(75.0, -20.0, 8.0, 50.0), "missing riverside steel lattice TV tower");
        assert!(has_near(-248.0, -286.0, 6.0, 28.0), "missing Central St brick chimney");
        assert!(has_near(-48.0, 58.0, 15.0, 10.0), "missing riverside tourist boat pavilion");
        assert!(has_near(165.0, -48.0, 24.0, 18.0), "missing cross-river cable car terminal");
        assert!(
            tris.iter().any(|t| t.iter().any(|v| {
                let wy = -v[2];
                wy > city.bank_y(v[0]) + 40.0 && v[1] > 18.0
            })),
            "missing Blagoveshchensk cathedral / arch / tower silhouettes across the river"
        );

        let rail_tris = tris
            .iter()
            .filter(|t| {
                let c = geom::P2::new(
                    (t[0][0] + t[1][0] + t[2][0]) / 3.0,
                    -(t[0][2] + t[1][2] + t[2][2]) / 3.0,
                );
                let y = (t[0][1] + t[1][1] + t[2][1]) / 3.0;
                y > 0.5 && y < 1.3 && geom::dist_to_polyline(c, &city.bank) < 2.5
            })
            .count();
        assert!(rail_tris > 500, "expected continuous riverside balustrade, got {rail_tris} tris");
    }
}
