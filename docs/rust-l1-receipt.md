# L1 receipt, part one: the clock runs on

25 November 2026. Branch `claude/rust-heihe`. The owner, the same day: 其實我是想做長期生存的, so the invasion night becomes the opening of a run that lasts days and weeks (`docs/DESIGN.md`, "Long-term survival"). This is the first half of L1, "the days". Game states in the client, on-screen text out of the world state, and save and load are the second half.

## What changed

- **The clock does not stop at 07:00.** `hour` keeps counting from 16:00 on 17 January. The first 07:00 is a milestone. It shows「天亮了。你还活着。」and gives a new objective: keep off the streets by day, find food and water indoors, sleep with J. Every later 07:00 is counted (`WorldSim::mornings`) and announced with its date. A run ends only when you die.
- **A calendar and a real sun** (`crates/sim/src/clock.rs`):
  - The date comes from `hour`; the HUD shows 2031年1月18日 第2天.
  - The sun's position at Heihe (50.25° N, 127.5° E, Beijing time) comes from NOAA's low-precision solar equations. Sunrise is 07:23 and sunset 15:56 on 17 January; by 3 March it sets after 17:00; by late April it rises at 04:16.
  - The client's sky uses the same sun (`sky.rs` used a hand-drawn arc for 17 January only).
  - Patrol and drone vision (`light_at`) now goes by the sun's height. The thresholds are the heights at 16:30 and 17:15 on 17 January, where the night's rules were set, so the first evening is unchanged and every later dawn and dusk follows the season.
- **The night's cold** goes by the time of day, 19:00 to 09:00, instead of "after 19:00 on the first day".
- **Weather after the first night** rolls for each three-hour block from the run's seed. Clear most of the time, snow about one block in eight, river fog only at night or in the morning. The first night keeps its script. These are play numbers until L2 brings a climate table.
- **A seeded random generator.** Rolls used to come from the entity counter, so making one more entity shifted every later roll. Now they come from a splitmix64 stream; `WorldSim::with_seed(city, seed)` starts one, and the same seed plays the same run (test).
- **Tiredness and sleep:**
  - `Body::fatigue` runs from 0 to 100. It rises 4.5 an hour awake and starts at 40 at 16:00 (up since about 07:00). From 95 you move 8% slower.
  - Press J indoors to sleep. You can't sleep with a soldier within 30 m, while challenged or under fire, or when not tired (below 20).
  - Asleep, the world runs fast: 120 play seconds per real second, an hour of the clock in two seconds, in steps of 0.25 s (the longest step the sim already took).
  - Tiredness falls 14 an hour, so seven hours sleep you out. Hunger and thirst fall at 35% of the waking rate, and you lose heat 40% more slowly.
  - What wakes you:
    - a soldier within 18 m, or a vehicle within 36 m;
    - a door being broken, glass, a shout, a shot within 80 m, a blast within 150 m, or a tank gun within 300 m;
    - being hurt, body temperature below 35.2 °C, thirst, hunger;
    - being rested, or J.
  - The screen shows 睡着了, the date and the hours slept.
- **The client:**
  - Nothing treats dawn as the end any more: input, the HUD, the speed bar, Enter and Esc.
  - The speed bar no longer drops back to 1× forever after dawn: `dawn_survived` had counted as danger.
  - The clamp of the free-look clock to 0–31 hours is gone.
  - The death screen gives the date and how many days you lasted.
  - One stray `19.5` now uses `BLACKOUT_HOUR`.

## Checked

- `cargo test -p sim`: 101 pass, 7 of them new:
  - `clock`: the sun on 17 January, days lengthening into spring, the calendar;
  - `survival`: the same seed plays the same run, the clock runs past dawn and counts the next morning, light and weather follow the days, and you sleep indoors until rested (5.5–7.5 h, hunger and thirst above half) and a soldier at 6 m wakes you.
- `cargo build -p northeast --release` builds.
- `northeast --interior-shots <dir>` takes three new pictures, in `docs/shots/rust-l1/`:
  - `04_day2_noon_street`: noon on 18 January, with the date, 第2天 and the morning's objective. The subtitle below it comes from the scene being built by jumping straight to 07:00, so every event of the night fires in one step.
  - `05_basement_asleep`: asleep in the basement shelter at 22:00 on the 18th.
  - `06_march_five_pm`: 17:00 on 3 March, still light, with the low sun on the façades.
- `cargo run --release -p sim --bin days_bot -- 3 4` (new) plays the plan the tests trust for the night past dawn, sleeping when tired. Result, 3 seeds per shelter:

| shelter | lasts until | of | slept | woken by |
|---|---|---|---|---|
| basement | 15:02–15:04 on 19 January | thirst | 6.9–7.1 h | a tank gun within 300 m 27–31 times on the 18th, 12–15 times in the first night |
| home | 12:34–15:27 on 18 January | thirst (2), cold (1) | 1.7–2.8 h | gunfire in the first night, 10–11 times |

## What this shows for what comes next

- **Nothing replaces what you use up.** The water, food and coal in the shelter last into the second day, then you die of thirst. L2 has to bring water you can get again (snow on a fire you can feed, a water point, a neighbour), food that is out there to find, and heat that does not cost one coal for 26 minutes.
- **Unheated rooms fall to the outdoor −18 °C within two hours.** That is why the home fails. Real flats with the heat off lose a few degrees a day. L2's room temperatures need thermal mass.
- **The night's armour never stops.** The tank waves meant for the first night are still fighting on the 18th, and the shelter is woken thirty times by guns within 300 m. L3 gives the occupation its own days: the sweep, the order sheet, the checkpoints.
- **The HUD's outdoor temperature is a fixed −24 °C.** It goes with L2's climate table.
- **A step of the sim costs 0.65–1.0 ms** on this laptop (days_bot, with a build running beside it). That is why sleep runs at 120× and not faster. Making the step cheaper is part of the frame-time work.
- **No lying-down pose yet.** Asleep, the player figure stands.
