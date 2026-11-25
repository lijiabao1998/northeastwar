//! Days that pass at a glance. A run lasts from a month to four years
//! (docs/DESIGN.md, "Time"), so routine days are not played minute by minute: the
//! player sets a routine, and the world steps a quarter of an hour at a time
//! with the same rules as real time (body, rooms, stove, the calendar, the
//! sweep and the visits) but without the street's patrols, which only matter to
//! someone out in it. Anything that needs a decision stops it.

use crate::city::City;
use crate::clock;
use crate::forces::{Forces, TIME_SCALE};
use crate::survival::{TickOutput, WorldSim, SLEEP_MIN_FATIGUE};
use crate::walk::Ground;

/// How the player keeps themselves while days pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Routine {
    /// Eat a ration when the hunger meter falls below this.
    pub eat_below: f32,
    /// Drink (a bottle, or melted snow) when the thirst meter falls below this.
    pub drink_below: f32,
    /// Keep the stove going when the room falls below this (°C).
    pub warm_below_c: f32,
    /// Take medicine above this much illness.
    pub medicine_above: f32,
    /// Sleep from this hour of the day to that one.
    pub sleep_from: f32,
    pub sleep_to: f32,
}

impl Default for Routine {
    fn default() -> Self {
        Self { eat_below: 70.0, drink_below: 70.0, warm_below_c: 8.0, medicine_above: 30.0, sleep_from: 22.0, sleep_to: 7.0 }
    }
}

/// What stopped the days passing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    /// As long as was asked.
    Done,
    /// Can't rest here (outdoors, someone close).
    NotSafe,
    /// Boots on the stairs: the sweep has reached your building.
    SweepComing,
    /// They came to your door.
    Visit,
    /// You were taken, and are back.
    Taken,
    /// The commandant's order sheet is up.
    Order,
    /// No water left, and no fire to melt snow on.
    NoWater,
    /// No food left.
    NoFood,
    /// Nothing left to burn and the room is freezing.
    NoFuel,
    /// Ill enough to need care.
    Ill,
    /// Hurt or bleeding.
    Hurt,
    /// Cold to the core.
    Cold,
    Dead,
}

impl Stop {
    pub fn text(self) -> (&'static str, &'static str) {
        match self {
            Stop::Done => ("日子过去了。", "The days have passed."),
            Stop::NotSafe => ("这里静不下来。", "You can't settle here."),
            Stop::SweepComing => ("楼道里有靴子声：清查到你这栋楼了。", "Boots on the stairs: the sweep has reached your building."),
            Stop::Visit => ("有人来过你家门口。", "Someone came to your door."),
            Stop::Taken => ("你被带走过，现在回来了。", "You were taken, and you are back."),
            Stop::Order => ("贴出了新的布告。", "A new order has been posted."),
            Stop::NoWater => ("没有水了，炉子也没火化雪。", "No water left, and no fire to melt snow."),
            Stop::NoFood => ("吃的没了。", "The food is gone."),
            Stop::NoFuel => ("没有能烧的了，屋里在结冰。", "Nothing left to burn, and the room is freezing."),
            Stop::Ill => ("你病得不轻。", "You are badly ill."),
            Stop::Hurt => ("你受了伤。", "You are hurt."),
            Stop::Cold => ("你冷透了。", "You are chilled to the core."),
            Stop::Dead => ("", ""),
        }
    }
}

/// How long days passed, and why they stopped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Passed {
    pub hours: f32,
    pub stop: Stop,
}

/// Game hours in one step of days that pass.
pub const STEP_H: f32 = 0.25;

impl WorldSim {
    /// Sleep at night by the routine, or when worn out; wake rested in the morning.
    pub fn routine_sleep(&mut self, r: &Routine) {
        let t = clock::time_of_day(self.hour);
        let night = if r.sleep_from > r.sleep_to { t >= r.sleep_from || t < r.sleep_to } else { t >= r.sleep_from && t < r.sleep_to };
        if !self.sleeping && ((night && self.body.fatigue >= SLEEP_MIN_FATIGUE) || self.body.fatigue >= 95.0) {
            self.sleeping = true;
        } else if self.sleeping && !night && self.body.fatigue <= 0.0 {
            self.sleeping = false;
        }
        self.body.asleep = self.sleeping;
    }

    /// One thing the routine does now, if anything needs doing: eat, drink (a
    /// bottle or melted snow), keep the stove going, take medicine. The same
    /// actions as the keys, so the numbers are the same.
    pub fn apply_routine(&mut self, r: &Routine, out: &mut TickOutput) -> bool {
        let Some(i) = self.active_interior else { return false };
        if self.sleeping {
            return false;
        }
        if self.body.hunger < r.eat_below && self.eat_ration() {
            return true;
        }
        let melt_s = self.melt_fuel_s();
        let mut want_fire = false;
        if self.body.thirst < r.drink_below {
            if self.drink_water() {
                return true;
            }
            if self.interiors[i].stove_at.is_some() && self.melt_snow(i) {
                return true;
            }
            want_fire = true;
        }
        let room = &self.interiors[i];
        if room.stove_at.is_some() && (want_fire || room.room_temp_c < r.warm_below_c) && room.stove_fuel_s < melt_s {
            let has_fuel = self.inv.coal > 0 || self.inv.wood > 0 || room.furniture > 0;
            if has_fuel && self.feed_stove(i, out) {
                return true;
            }
        }
        if !self.body.bleeding && self.body.illness > r.medicine_above && self.take_medicine() {
            return true;
        }
        false
    }

    /// Let up to `hours` pass by the routine, a quarter of an hour at a time,
    /// until something needs a decision. The street's patrols are not run: they
    /// move on while you stay in, and come back when real time resumes.
    pub fn pass_time(&mut self, city: &City, ground: &Ground, forces: &Forces, r: &Routine, hours: f32) -> Passed {
        let _ = (ground, forces);
        if self.can_rest().is_err() {
            return Passed { hours: 0.0, stop: if self.dead { Stop::Dead } else { Stop::NotSafe } };
        }
        let dt = STEP_H * 3600.0 / TIME_SCALE;
        let mut out = TickOutput::default();
        self.squads.clear();
        self.grenades.clear();
        let start = (self.detentions, self.order_posted, self.body.illness > 60.0, self.body.hp < 50.0 || self.body.bleeding);
        let warned_before = self.sweep_warned_here();
        let visit_before = self.visit_at;
        let mut passed = 0.0;
        let mut stop = Stop::Done;
        while passed + STEP_H * 0.5 < hours {
            self.routine_sleep(r);
            self.apply_routine(r, &mut out);

            self.hour += STEP_H;
            self.hours_alive += STEP_H;
            self.sim_time += dt;
            passed += STEP_H;
            let (w, fog) = crate::survival::weather_at(self.seed, self.hour);
            if self.weather_override.is_none() {
                self.weather = w;
                self.fog_density = fog;
            }

            self.step_calendar(&mut out);
            if self.dead {
                stop = Stop::Dead;
                break;
            }
            if self.detentions != start.0 {
                stop = Stop::Taken;
                break;
            }
            if self.order_posted && !start.1 {
                stop = Stop::Order;
                break;
            }
            if self.sweep_warned_here() && !warned_before {
                stop = Stop::SweepComing;
                break;
            }
            if visit_before.is_some() && self.visit_at.is_none() {
                stop = Stop::Visit;
                break;
            }

            self.step_rooms(dt, &mut out);
            if self.step_body_resting(city, dt) {
                stop = Stop::Dead;
                break;
            }
            self.step_fires(dt);

            let i = self.active_interior;
            let room_c = i.map(|i| self.interiors[i].room_temp_c).unwrap_or(-30.0);
            let fuel_left = self.inv.coal + self.inv.wood + i.map(|i| self.interiors[i].furniture).unwrap_or(0);
            let fire = i.is_some_and(|i| self.interiors[i].stove_fuel_s >= self.melt_fuel_s());
            if self.body.thirst < r.drink_below - 20.0 && self.inv.water == 0 && !fire && fuel_left == 0 {
                stop = Stop::NoWater;
                break;
            }
            if self.body.hunger < r.eat_below - 10.0 && self.inv.food == 0 {
                stop = Stop::NoFood;
                break;
            }
            if fuel_left == 0 && room_c < 2.0 && !fire {
                stop = Stop::NoFuel;
                break;
            }
            if self.body.illness > 60.0 && !start.2 {
                stop = Stop::Ill;
                break;
            }
            if (self.body.hp < 50.0 || self.body.bleeding) && !start.3 {
                stop = Stop::Hurt;
                break;
            }
            if self.body.temp < 35.2 {
                stop = Stop::Cold;
                break;
            }
        }
        if stop != Stop::Done && stop != Stop::Dead {
            self.sleeping = false;
            self.body.asleep = false;
        }
        Passed { hours: passed, stop }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interior::InteriorKind;
    use crate::survival::{SimInput, DAWN_HOUR};

    fn world() -> (City, Ground, Forces) {
        let (city, _) = crate::load_heihe().unwrap();
        let ground = Ground::new_with_interiors(&city, &WorldSim::new(&city).interiors);
        let forces = Forces::from_json(&std::fs::read_to_string(crate::data_dir().join("russia/forces.json")).unwrap()).unwrap();
        (city, ground, forces)
    }

    /// The morning after the first night, in the basement shelter with what it holds.
    fn sheltered(city: &City, ground: &Ground, forces: &Forces, seed: u64) -> WorldSim {
        let mut sim = WorldSim::with_seed(city, seed);
        let idx = sim.interiors.iter().position(|i| i.kind == InteriorKind::BasementShelter).unwrap();
        sim.player_at = sim.interiors[idx].stove_at.unwrap();
        sim.hour = DAWN_HOUR + 0.001;
        sim.tick(city, ground, forces, &SimInput { clock_running: true, ..Default::default() }, 0.25);
        sim.tick(city, ground, forces, &SimInput { clock_running: true, ..Default::default() }, 0.25);
        assert_eq!(sim.active_interior, Some(idx));
        sim.inv.food = 6;
        sim.inv.water = 4;
        sim.inv.coal = 4;
        sim.squads.clear();
        sim
    }

    #[test]
    fn days_pass_until_something_needs_you() {
        let (city, ground, forces) = world();
        let mut sim = sheltered(&city, &ground, &forces, 3);
        let r = Routine::default();
        let t0 = std::time::Instant::now();
        let p = sim.pass_time(&city, &ground, &forces, &r, 24.0 * 10.0);
        assert_ne!(p.stop, Stop::Done, "ten days do not pass without something happening: {:?}", p);
        assert!(p.hours > 2.0, "{:?}", p);
        // The order sheet comes at noon on the 18th: that is a stop.
        assert_eq!(p.stop, Stop::Order, "{:?} at {}", p, clock::label(sim.hour));
        let mut stops = vec![p.stop];
        let mut total = p.hours;
        while total < 24.0 * 30.0 && !sim.dead {
            let p = sim.pass_time(&city, &ground, &forces, &r, 24.0 * 30.0 - total);
            total += p.hours;
            stops.push(p.stop);
            if p.hours == 0.0 {
                break;
            }
        }
        assert!(stops.contains(&Stop::SweepComing), "the sweep reaches the basement: {stops:?}");
        assert!(total > 24.0 * 5.0 || sim.dead, "{stops:?} {total}");
        assert!(t0.elapsed().as_secs_f32() < 30.0, "a month at a glance is quick: {:?}", t0.elapsed());
    }

    /// Days passed at a glance must agree with the same days played step by
    /// step: same routine, same seed, the patrols cleared in both.
    #[test]
    fn days_at_a_glance_agree_with_real_time() {
        let (city, ground, forces) = world();
        let r = Routine::default();
        let mut quick = sheltered(&city, &ground, &forces, 5);
        let mut slow = quick.clone();
        // No sweep or visit in this comparison: only the body, the room and the fire.
        for sim in [&mut quick, &mut slow] {
            sim.order_posted = true;
            sim.assault_over = true;
            for i in 0..sim.interiors.len() {
                sim.mark_swept(i);
            }
        }
        let days = 2;
        let mut quick_mornings = Vec::new();
        for _ in 0..days {
            let p = quick.pass_time(&city, &ground, &forces, &r, 24.0);
            assert_eq!(p.stop, Stop::Done, "{p:?}");
            quick_mornings.push(snapshot(&quick));
        }
        let mut slow_mornings = Vec::new();
        let end = slow.hour + 24.0 * days as f32;
        let mut next = slow.hour + 24.0;
        let mut out = TickOutput::default();
        while slow.hour < end - 1e-3 {
            slow.squads.clear();
            slow.spawn_wait = 1.0e6;
            slow.routine_sleep(&r);
            slow.apply_routine(&r, &mut out);
            slow.tick(&city, &ground, &forces, &SimInput { clock_running: true, ..Default::default() }, 0.25);
            if slow.hour >= next - 1e-3 {
                slow_mornings.push(snapshot(&slow));
                next += 24.0;
            }
        }
        for (d, (q, s)) in quick_mornings.iter().zip(&slow_mornings).enumerate() {
            eprintln!("day {}: at a glance {:?}", d + 1, q);
            eprintln!("day {}: real time   {:?}", d + 1, s);
            assert!((q.thirst - s.thirst).abs() < 6.0, "thirst, day {}: {q:?} vs {s:?}", d + 1);
            assert!((q.hunger - s.hunger).abs() < 3.0, "hunger, day {}: {q:?} vs {s:?}", d + 1);
            assert!((q.room - s.room).abs() < 3.0, "room, day {}: {q:?} vs {s:?}", d + 1);
            assert!((q.temp - s.temp).abs() < 0.5, "body, day {}: {q:?} vs {s:?}", d + 1);
            assert!((q.illness - s.illness).abs() < 3.0, "illness, day {}: {q:?} vs {s:?}", d + 1);
            assert!(q.fuel.abs_diff(s.fuel) <= 1, "fuel, day {}: {q:?} vs {s:?}", d + 1);
            assert!(q.food.abs_diff(s.food) <= 1 && q.water.abs_diff(s.water) <= 1, "stores, day {}: {q:?} vs {s:?}", d + 1);
        }
    }

    #[derive(Debug)]
    struct Snap {
        thirst: f32,
        hunger: f32,
        room: f32,
        temp: f32,
        illness: f32,
        fuel: u32,
        food: u32,
        water: u32,
    }

    fn snapshot(sim: &WorldSim) -> Snap {
        let i = sim.active_interior.unwrap();
        Snap {
            thirst: sim.body.thirst,
            hunger: sim.body.hunger,
            room: sim.interiors[i].room_temp_c,
            temp: sim.body.temp,
            illness: sim.body.illness,
            fuel: sim.inv.coal + sim.inv.wood + sim.interiors[i].furniture,
            food: sim.inv.food,
            water: sim.inv.water,
        }
    }
}
