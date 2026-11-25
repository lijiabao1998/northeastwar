//! Plays the run past the first night, many times, with the one plan the tests
//! already trust for the night: take what is in the shelter, bar the door, close
//! the curtains, sit by the stove; drink, eat, keep the stove going (coal, then
//! the furniture), take medicine when ill, sleep when tired. It never goes out,
//! so it shows how long what one shelter holds lasts, and what ends it.
//!
//! cargo run --release -p sim --bin days_bot -- [seeds] [days]

use sim::forces::Forces;
use sim::interior::InteriorKind;
use sim::survival::{SimInput, WorldSim, DAWN_HOUR};
use sim::walk::Ground;
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    let days: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8.0);
    let (city, _) = sim::load_heihe().expect("Heihe data");
    let forces = Forces::from_json(&std::fs::read_to_string(sim::data_dir().join("russia/forces.json")).unwrap()).unwrap();
    let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
    let end_hour = DAWN_HOUR + 24.0 * (days - 1.0);
    // Stove time a pot of melted snow costs (play seconds), as the sim reckons it.
    let melt_s = sim::living::get().num("fuel", "melt_fuel_h") * 3600.0 / sim::forces::TIME_SCALE;

    println!("| shelter | seed | result | when | mornings | slept h | woke by | coal / wood / furniture / food / water left | room min / max °C | body min °C | illness max | ms per game hour |");
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
    for kind in [InteriorKind::BasementShelter, InteriorKind::PlayerHome] {
        for seed in 0..seeds {
            let mut sim = WorldSim::with_seed(&city, seed);
            let idx = sim.interiors.iter().position(|i| i.kind == kind).unwrap();
            let stove = sim.interiors[idx].stove_at.unwrap();
            if let Some(locker) = sim.interiors[idx].locker_at {
                sim.player_at = locker;
                sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
            }
            let loots: Vec<_> = sim.interiors[idx].loots.iter().map(|l| l.at).collect();
            for at in loots {
                sim.player_at = at;
                sim.tick(&city, &ground, &forces, &SimInput { interact_just_pressed: true, ..Default::default() }, 0.1);
            }
            sim.player_at = sim.interiors[idx].front_door.inside;
            sim.tick(&city, &ground, &forces, &SimInput { fortify_or_stove: true, ..Default::default() }, 0.1);
            sim.interiors[idx].curtains_closed = true;
            sim.player_at = stove;

            let mut woke: BTreeMap<String, u32> = BTreeMap::new();
            let mut slept = 0.0f32;
            let mut min_temp = f32::MAX;
            let mut room_lo = f32::MAX;
            let mut room_hi = f32::MIN;
            let mut ill_max = 0.0f32;
            let mut was_asleep = false;
            let t0 = Instant::now();
            let h0 = sim.hour;
            while !sim.dead && sim.hour < end_hour {
                let mut inp = SimInput { clock_running: true, ..Default::default() };
                let room = &sim.interiors[idx];
                if !sim.sleeping {
                    if sim.body.hunger < 70.0 && sim.inv.food > 0 {
                        inp.use_food = true;
                    }
                    let mut want_fire = false;
                    if sim.body.thirst < 70.0 {
                        if sim.inv.water > 0 {
                            inp.use_water = true;
                        } else if room.stove_fuel_s >= melt_s {
                            inp.interact_just_pressed = true;
                        } else {
                            // No water: light the stove to melt snow.
                            want_fire = true;
                        }
                    }
                    let has_fuel = sim.inv.coal > 0 || sim.inv.wood > 0 || room.furniture > 0;
                    if (want_fire || room.room_temp_c < 8.0) && room.stove_fuel_s < melt_s && has_fuel {
                        inp.fortify_or_stove = true;
                    }
                    if !sim.body.bleeding && sim.body.illness > 30.0 && sim.inv.medicine > 0 {
                        inp.use_bandage = true;
                    }
                    if sim.body.temp < 35.0 && sim.inv.warmer > 0 {
                        inp.use_warmer = true;
                    }
                    if sim.body.bleeding && sim.inv.bandage > 0 {
                        inp.use_bandage = true;
                    }
                    let busy = inp.use_food || inp.use_water || inp.interact_just_pressed || inp.fortify_or_stove;
                    if !busy && sim.body.fatigue > 70.0 && sim.body.temp > 35.6 {
                        inp.toggle_sleep = true;
                    }
                }
                let out = sim.tick(&city, &ground, &forces, &inp, 0.25);
                if sim.sleeping {
                    slept += 0.25 * 15.0 / 3600.0;
                }
                if was_asleep && !sim.sleeping {
                    let mut why = sim.woke.map(|w| format!("{w:?}")).unwrap_or_else(|| "-".into());
                    if why == "Noise" {
                        // Which noise, so the thresholds can be judged.
                        let near = |p: sim::geom::P2, r: f32| p.dist(sim.player_at) < r;
                        let what = [
                            (out.door_breach_thud, "door"),
                            (out.glass_break, "glass"),
                            (out.shout_stoy, "shout"),
                            (out.tank_cannon, "cannon"),
                            (out.slab_collapse, "slab"),
                            (out.mortar_whistle, "mortar"),
                            (out.explosions.iter().any(|e| near(e.at, 150.0)), "blast"),
                            (out.tracers.iter().any(|t| near(sim::geom::P2::new(t.from[0], -t.from[2]), 80.0)), "shots"),
                        ];
                        let names: Vec<&str> = what.iter().filter(|w| w.0).map(|w| w.1).collect();
                        why = format!("Noise({})", names.join("+"));
                    }
                    let night = if sim.mornings == 0 { "n0" } else { "day" };
                    *woke.entry(format!("{why}@{night}")).or_default() += 1;
                }
                was_asleep = sim.sleeping;
                min_temp = min_temp.min(sim.body.temp);
                if sim.mornings > 0 {
                    room_lo = room_lo.min(sim.interiors[idx].room_temp_c);
                    room_hi = room_hi.max(sim.interiors[idx].room_temp_c);
                }
                ill_max = ill_max.max(sim.body.illness);
            }
            let ms = t0.elapsed().as_secs_f32() * 1000.0 / (sim.hour - h0).max(0.01);
            let result = if sim.dead { format!("died: {}", sim.death_en) } else { "alive".into() };
            let woke_s: Vec<String> = woke.iter().map(|(k, v)| format!("{k} {v}")).collect();
            println!(
                "| {kind:?} | {seed} | {result} | {} | {} | {slept:.1} | {} | {} / {} / {} / {} / {} | {room_lo:.1} / {room_hi:.1} | {min_temp:.1} | {ill_max:.0} | {ms:.0} |",
                sim::clock::label(sim.hour),
                sim.mornings,
                woke_s.join(", "),
                sim.inv.coal,
                sim.inv.wood,
                sim.interiors[idx].furniture,
                sim.inv.food,
                sim.inv.water,
            );
        }
    }
}
