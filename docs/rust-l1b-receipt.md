# L1, second half, receipt: saves, and days that pass at a glance

25 November 2026. Branch `claude/rust-heihe`. The owner set the standard at one month, and up to four years: 我想的是甚至能以1個月 甚至是 4年為標準. A day awake is 96 minutes of play, so a month would be about 48 hours and four years about 2,300. Before more content, the game needs two things:
- a run that can be saved;
- routine days that pass at a glance.

The owner said 開始 to this order (`docs/DESIGN.md`, "Order of work").

## Saving and loading

- **What is saved.** Everything the world holds:
  - what you carry and what is hidden behind the cupboard;
  - every room's temperature, stove and furniture;
  - the patrols and the wrecks;
  - the occupation's schedule (which buildings the sweep has been through, the next visit);
  - the random generator.
- **How.** Every struct in the world derives serde. Text the program carries as `&'static str` (cache titles, names, messages) goes through `save::static_str`, behind an alias so serde does not take it for text borrowed from the file. The streets you have walked became an ordered set, so the same world saves to the same bytes.
- **The file.** `crates/sim/src/save.rs` writes the world as gzipped JSON behind a format number; a save from another format is refused, not loaded wrong. A save is about 560 KB, written in about 30 ms and read in about 110 ms (release build).
- **Where.** Saves go in `saves/` beside `data/` (not in the repository), through a temporary file so a crash mid-write keeps the old one.
- **In the game:**
  - it saves by itself every morning;
  - the pause menu has 保存进度;
  - the start menu has 继续上次的进度, with the save's date and day.
  - When you die, the run's saves are deleted: one life (DESIGN rule 1).

## Days that pass at a glance (`crates/sim/src/routine.rs`)

- **A routine:**
  - eat when the hunger meter is below 70;
  - drink below 70, a bottle or a pot of melted snow;
  - keep the stove going below 8 °C, with coal, then wood, then furniture broken up;
  - take medicine above 30 illness;
  - sleep from 22:00 to 07:00.
  These are defaults; choosing your own comes later. The routine uses the same functions as the keys (`eat_ration`, `drink_water`, `feed_stove`, `melt_snow`, `take_medicine`), now shared, so the numbers cannot drift apart.
- **`WorldSim::pass_time`** steps the world a quarter of an hour at a time with the real rules:
  - the body;
  - the rooms and the stove;
  - fires;
  - the calendar (`step_calendar`: each morning, the order sheet, the sweep and the visits, shared with real time).

  It does not run the street's patrols, which only matter to someone out in it. They come back when real time resumes.
- **What stops it, so you decide:**
  - the sweep reaching your building, ten minutes ahead;
  - a visit to your door;
  - coming back from detention;
  - a new order;
  - no water and no fire to melt snow;
  - the food gone;
  - nothing left to burn in a freezing room;
  - illness over 60;
  - hurt or bleeding;
  - a body below 35.2 °C;
  - death.
- **In the game.** Indoors and not tired, J lets the days pass: a day every two seconds, with a panel showing the date and day, what is left (food, water, medicine, coal, wood, furniture), the room's temperature and the body's meters. J again stops it. When tired, J still sleeps. When something stops the days, it says what.

## Checked

- `cargo test -p sim`: 133 pass, 4 of them new:
  - `save`:
    - **A loaded run goes on exactly as the saved one.** The test saves 1,200 steps into a walk, compares the loaded world with the original, then runs both 1,200 more steps and compares the whole state again.
    - A save from another format is refused.
  - `routine`:
    - **Days pass until something needs you.** From the morning after the first night in the basement, the first stop is the order sheet at noon. Within a month the sweep reaches the basement. The month passes in a few seconds.
    - **Days at a glance agree with real time.** Same seed, same routine, two days, patrols cleared in both. Each morning's numbers (thirst, hunger, room, body, illness, stores, fuel) agree. Day 1: thirst 80.0 against 80.0, room 15.5 °C against 15.4 °C, illness 0.95 against 1.04; the stores and fuel are equal.
- `cargo build -p northeast --release` builds. Pictures:
  - `docs/shots/rust-l1b/08_days_passing.jpg`: days passing in the basement shelter, 26 January.
  - `docs/shots/rust-l1b/start_continue.jpg`: the start menu reading a save of 07:00 on 19 January, day 3, written by `days_bot` with `DAYS_BOT_SAVE=1`.

## Not done

- **Choosing a routine:** rations, how warm to keep the room, when to sleep. The defaults are fixed for now.
- **Going out as part of a routine:** fetching water at dawn, a trip to a shop. Every trip out needs the street's risks at a glance, which come with L3's second part (checkpoints) and L4.
- **On-screen text out of the world state, and game states in the client.** These are also on L1's list. The world still holds its own subtitles; that does no harm to saves, so it was left for later.
- **A save from before a change to the world's shape is refused rather than converted.** Fields added from now on carry `#[serde(default)]` so older saves still load.
