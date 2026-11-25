# L2 receipt: living

25 November 2026. Branch `claude/rust-heihe`. The owner said 開始 L2. This is `docs/DESIGN.md`'s step L2: water, food, fuel and cold over days; illness; the home as the place that keeps you alive; shops that empty. Before it, `days_bot` showed that the most careful plan died of thirst on the second day, because nothing in the world could be had again.

## Where the numbers come from

- Every L2 number is in `data/living.json`. Each has a `_src` (where it comes from) and a `_basis`: reported, derived (the working is in the research file) or play.
- The research is `docs/research/living.md`, written for this round: about 120 sources on Heihe's climate, the Amur's ice, how heated blocks cool when the heat stops, basements, stove fuel, the body without water or food, cold and illness, and how fast shops emptied in Kherson, Bucha and Mariupol.
- `docs/research/occupation.md` already had water need, calories by temperature, the WHO 18 °C minimum and the energy to melt snow.
- Code reads the table through `crates/sim/src/living.rs` and hard-codes none of it.

## What changed

**The weather is Heihe's.**
- The temperature outdoors comes from the CMA 1991–2020 monthly normals for Heihe, eased between months.
- A daily curve puts the low at 06:30 and the high at 14:00, from 3-hourly January observations across the river.
- A day-to-day departure follows an AR(1) process: 0.66 of yesterday's plus a fresh roll, standing spread 5 °C, fitted to eleven Januaries. It is rolled from the run's seed and starts on the fourth day, so the invasion night is the same in every run.
- On 17 January that is about −16 °C at 16:00 and −25 °C before dawn.
- Snow falls on the month's snow days (10 in January).
- The HUD shows the real temperature instead of a fixed −24 °C.

**The body.**
- Cold outdoors scales with how far the air is below comfort, `(12 − T) / 30`. This keeps the night's old rule at −18 °C and replaces the fixed ×1.3 after 19:00.
- Indoors, out of the wind and wrapped up, the body loses heat only below about 2 °C. People lived for weeks in 3 °C flats in occupied Mariupol.
- Hypothermia follows the clinical bands: from 32 °C you are slow and clumsy, at 28 °C unconscious, which alone in the cold is death. It was 32 °C.
- **Water:**
  - A resting adult with nothing to drink loses about 1.3 L a day; more walking, more running.
  - The thirst meter is the 6.5 L (10% of body weight) you can lose before collapse, about day 5.
  - Death comes about day 8.
  - A bottle is 1 L; a pot of melted snow is 2 L.
- **Food:**
  - A body burns 2,100 kcal a day plus 100 for every 5 °C below 20.
  - The hunger meter is the 84,000 kcal it can go without before collapse, about day 40.
  - Weakness, colder and slower, comes from about day 12 (the meter at 70). Death comes about day 60.
  - A ration is 1,000 kcal.
- **Illness:**
  - It builds in air below 18 °C, compounding about 4% for every degree below, as the risk of a respiratory infection does. For four and a half days after a chilling (body below 35 °C), it builds 2.6 times as fast.
  - It goes when you are warm.
  - Above 40 you lose heat faster; above 70 you are slow and losing health; at 100 it kills.
  - Pharmacies have medicine. Q takes it when you are not bleeding.

**The rooms.**
- A room heads for a balance temperature with a time constant:
  - Radiators (until 21:00 on the first night): 18 °C.
  - A burning stove: 19 °C, within the hour.
  - A basement: +5 °C, over about ten days.
  - Otherwise: 3 °C above the street, over 60 hours, the time constant of a middle flat in a 1980s block.
- A broken window cuts that time constant to 0.15 of itself and leaves the room at the street's temperature. An open door cuts it too.
- The home on the first night is still about 12 °C at dawn, not −18 °C, and goes below freezing on the third day.

**Fuel.**
- A unit of coal (2 kg) is 4.1 hours of fire. A unit of wood (a chair, 5 kg) is 7.3 hours.
- With neither, T at the stove breaks up a piece of furniture. That is loud: patrols hear it within 25 m for three seconds.
- Furniture is counted in chair-sized pieces: a home's ground floor has 25, a shop's 8, each floor above 15.
- Melting a pot of snow costs 0.63 hours of fire.
- The stove's prompt says how many hours of fire are left and how much furniture.

**Shops.** Each morning other people take part of what is left on the shop shelves: 55% the first morning, a third the second, half the third, then about a quarter a day, then a tenth. That follows the timing of Kherson, Bucha and Mariupol (staples gone on day 1, looting from day 3, shelves empty by days 8–13). Homes and upper floors are left alone until L3.

**The HUD.** Outdoor temperature; 柴 (wood) beside 煤; 病 when you are ill; 药 beside 绷带. The menu's key list explains Q and T's new uses.

## Checked

- `cargo test -p sim`: 118 pass, 8 of them new:
  - `living`: Heihe in January and April; departures last and stay in bounds; colder needs more food.
  - `survival`:
    - a flat cools over days, not hours: still above 8 °C the morning after, below freezing but above the street on the third day, the basement near +5 °C;
    - the stove burns coal, then wood, then the furniture, loudly;
    - water and food last days, not hours: alive on day 4, near collapse, alive on day 7, dead by day 9; a cold room burns more food;
    - cold rooms make you ill and medicine helps;
    - others empty the shops but not your home.
  - Two older tests changed with the rules: a ration now feeds by calories, and the thirst timeline is the new one.
- Two mistakes the night bot's probe found (below), fixed with tests in the same files: a room of 10 °C or more counted as a fire for illness too, so nobody fell ill in a 12–16 °C room; and the shops on the street, searched under their signs, never emptied (0 of 65 after three mornings).
- `cargo build -p northeast --release` builds. `--interior-shots` pictures are in `docs/shots/rust-l2/`:
  - the stove's new prompt;
  - noon on 18 January at −18 °C from the climate;
  - 17:00 on 3 March at −1 °C, a warm spell from this seed.
- `days_bot`, 3 seeds per shelter, 14 days, the indoor plan with upkeep (it never goes out). The table is `docs/shots/rust-l2/days_bot_14.md`.

| shelter | lasted | room after the first night | body low | illness high | fuel used | left at the end |
|---|---|---|---|---|---|---|
| basement | all 14 days | 12–17 °C (stove) | 36.8 °C | 15 | all the coal and wood; furniture down to 5 | no food or water; melting snow |
| home | all 14 days | −12 to +9 °C (windows broken in the night) | 35.2–35.5 °C | 65–67 | about 40 pieces of furniture | no food or water; melting snow; 45 pieces of furniture |

Before this round the same plan died on the second day, of thirst. Now what wears it down is slower:
- fuel, which runs out;
- a cold flat, which makes you ill;
- the food a shelter holds, about two weeks.

Nothing in the plan goes out for more; L3's sweeps and searches will come to it.

## Not done

- The climate fit's per-winter offset and wider daily range on cold days are not used. The model also makes too few long cold waves (living.md, "Known shortfall").
- The faster fall in a room's first 12 hours is not modelled. Neither is the basement drifting towards 0 °C after weeks without heat.
- The Amur's freeze-up and break-up are in the table, but nothing uses them until L5.
- The body still cools outdoors faster than living.md 6c estimates for a dressed adult. That is kept, so the night plays as designed. Indoors it now matches the estimates.
- These are play numbers with no source:
  - the stove's 50% efficiency;
  - how much furniture a shop or an upper floor holds;
  - how fast the illness meter fills and empties;
  - what medicine does;
  - what share of the shops' last goods others take after the first week.
- Shops refill if deliveries resume (living.md 7), but nothing does that yet.
