//! cargo run -p sim --bin cmab_report -- <out.json>
//! Writes every Chinese-bank building after the CMAB fill, with where it came from,
//! and prints what each rule dropped. For checking the merge, not for the game.

use sim::city::Bank;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "out/check/merged.json".into());
    let mut city = sim::load_osm_heihe().expect("city.json");
    let cmab = sim::load_cmab_heihe().expect("cmab.json");
    let tally = cmab.fill(&mut city);
    println!("{tally:?}");
    let rows: Vec<serde_json::Value> = city
        .buildings
        .iter()
        .filter(|b| b.bank == Bank::China)
        .map(|b| {
            serde_json::json!({
                "id": b.id,
                "from": if b.id >= sim::cmab::ID_BASE { "cmab" } else { "osm" },
                "cmab_id": if b.id >= sim::cmab::ID_BASE { (b.id - sim::cmab::ID_BASE) % sim::cmab::PART } else { 0 },
                "part": if b.id >= sim::cmab::ID_BASE { (b.id - sim::cmab::ID_BASE) / sim::cmab::PART } else { 0 },
                "levels": b.levels,
                "height": b.height,
                "solid": b.solid,
                "centroid": [b.centroid.x, b.centroid.y],
                "footprint": b.footprint.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>(),
                "kind": format!("{:?}", sim::look::dress(b, &city).kind),
            })
        })
        .collect();
    if let Some(dir) = std::path::Path::new(&out).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(&out, serde_json::to_string(&serde_json::json!({"tally": format!("{tally:?}"), "buildings": rows})).unwrap()).unwrap();
    println!("wrote {} buildings to {out}", rows.len());
}
