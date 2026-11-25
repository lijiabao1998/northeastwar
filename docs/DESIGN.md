# Design: survive it, as an ordinary person

Set by the owner on 15 November 2026:

> pz的模式 但是是换成现代战争，俄罗斯的装备完全取决于 在乌克兰战争当中的表现

So the game is Project Zomboid's loop (needs, scavenging, injuries, hiding, one life) with a modern army where the zombies would be. The army is the Russian one as it has actually fought in Ukraine since 2022.

Set by the owner on 25 November 2026:

> 其實我是想做長期生存的

So the invasion night is the opening, not the whole game. After it come days, weeks and months under occupation, and the occupation behaves the way Russian occupation has behaved in Ukraine since 2022 (see "Long-term survival" below).

And later the same day:

> 我想的是甚至能以1個月 甚至是 4年為標準

So a run is built to last from a month up to four years, about as long as the war in Ukraine has run (from February 2022). That changes how time works (below): no one can play four years at the speed of the first night.

## Two rules everything else follows

1. **You are not a soldier.** You have no weapon unless the game hands you one. Being seen is the danger, not being out-shot. A run ends when you die, and it starts again at 16:00.
2. **No brochure numbers.** Each Russian weapon, vehicle, drone and habit behaves the way open reporting says it did in Ukraine. The numbers live in one table, `data/russia/forces.json`. Each carries a `src` (which source) and a `basis`: `reported` (the source gives the figure), `derived` (worked out from reported figures) or `play` (the source gives the shape, the number was chosen so the game can be played). The findings and links are in `docs/research/ukraine-performance.md`. The occupation and civilian life follow the same rule: living (weather, rooms, fuel, water, food, illness) is in `data/living.json` from `docs/research/living.md`, and the occupation's are in `data/occupation.json` from `docs/research/occupation.md`.

## Time

The war clock runs 15 times faster than play. Anything measured in minutes in the real war (a drone-cued fire mission, a drone's time on station, the pace of the advance) is written in war-clock minutes in the table and happens that much sooner in play: a 3 to 5 minute sensor-to-shooter time is 12 to 20 seconds of play. Things a person does in seconds (noticing you, raising a rifle, a burst) are in play seconds.

## What is in the game now (P1)

- **Before 17:36** Heihe is an ordinary evening. Shops are open; every shop sign is a door you can take from once (hold E). What a door holds follows the sign: a pharmacy has bandages, a clothes shop has a padded coat.
- **17:36** the crossing. From then on the front moves south from the river at a walking pace, and infantry groups of two to five patrol the streets it has reached.
- **Being seen.** A soldier sees you only inside his reach for the light there is, inside the arc he faces, with nothing solid between you. Running is heard. Standing still in a doorway shows the least. Lamplight makes the night bright; the blackout at 19:30 makes it dark.
- **Being challenged.** A group that is sure of you shouts. Stand still and you are searched, robbed and let go. Run and they shoot. This follows what was documented in occupied towns in 2022: people outside were stopped, searched, sometimes shot at.
- **The drone and the mortar.** A quadcopter comes and goes. If it holds you in view for a few seconds, mortar rounds follow 3 to 5 war-clock minutes later, on the spot where you were seen, or on you if it can still see you. A roof (a doorway) hides you from it and takes most of a blast.
- **The body.** One life. Rifle wounds bleed until bandaged. Cold kills below 32°C; the heating stops at 21:00. Hunger and thirst run on the clock.
- **The goal** is to be alive at 07:00.
- **H after the alert** still starts the older defend-the-shop night. In that mode, and only in it, you are handed a rifle.

## Long-term survival (from 25 November 2026)

The research behind this section is `docs/research/occupation.md`: about two hundred sources on Kherson, Mariupol, Bucha, Izium, Melitopol and Berdiansk, each claim tagged reported, derived or play. What it says, in the shape the game takes:

### Time

- The clock does not stop at 07:00. Hours run on from 16:00 on 17 January (hour 16) through the days; `hour % 24` is the time of day and `hour / 24` the day. Light, weather and cold follow the time of day and the season: January's −24 °C nights, the thaw, and the river ice breaking up in late April, which ends crossing on foot.
- Awake, the clock runs 15 times faster than play, as now. Sleeping in a place you have made safe runs it much faster, and anything that comes to the door wakes you.
- A run lasts from a month to four years. At 15 times, a day awake is 96 minutes of play, a month about 48 hours, four years about 2,300. So days are lived two ways:
  - The hours that matter (going out, a search at the door, a checkpoint, a night of shelling) are played in real time.
  - Routine days pass at a glance. The player sets a routine (stay in, ration, fetch water at dawn, burn this much), and a day-level model runs it, stopping for anything that needs a decision: a knock, an order, a sound, a neighbour, the stores running low. Detention already passes time this way (L3).
- That day-level model has to agree with the real-time one, so the bots check it against the tick-by-tick sim over the same days.
- Saving is part of the game, not a convenience.
- Over years, the seasons matter as much as the occupation. The river opens and freezes. Summer is green, light and hot, and the city has to be drawn that way too, not only under snow. Prices and papers change month by month, and the war itself moves: fronts, offensives, an annexation or a liberation.

### Phases

The invasion night (17 January 16:00 to 18 January 07:00) is what the game has now. After it:

1. **Days 1 to 3, the sweep.** Power, water and the mobile network fail within a day in the districts the assault came through (the riverfront and Central Street); gas and the last heat within days. Soldiers go through every building, send people to basements with doors unlocked, search flats for weapons, radios and phones, and take some men. Moving outside is what gets people shot, men more than women, in cars more than on foot, at night more than by day.
2. **Day 2 on, the order sheet.** The commandant posts rules: curfew 20:00 to 06:00, walk alone or in pairs, stop at the first shout. Breaking them raises the odds of being shot or taken.
3. **Weeks 1 to 4.** Fixed checkpoints on the main roads and the bridge, each pass checking papers, sometimes the phone, sometimes the body. Filtration gives a paper slip that later doors ask for. A watchlist built from seized city records: officials, police, veterans, journalists first; volunteers, teachers, relatives of soldiers next; anyone reported as opposed after that. Searches come in waves through a neighbourhood and look for listed people. Prices of packaged food go up three to five times in the first month; cash runs out and comes only from private sellers at a cut; aid parcels are small and rare.
4. **Months 2 to 9.** The occupation becomes paperwork. The old mobile network is cut around month 3; new SIM cards need ID. Pensions, aid, SIM cards, utility accounts, jobs and car plates come to need the occupier's passport, one door at a time. Schools change curriculum. A "referendum" with soldiers at the door. Young men may no longer leave, and call-up papers follow. Resistance has real effect and every success brings a wave of reprisals.
5. **The end of a run** can come from outside: if the front moves, the occupier orders people across the river, cuts the power and takes what it can, and whoever comes back judges what the player did (a passport, a job, informing).

### What the player has to manage

- **The body over days.** Water in litres (15 a day to stay well, 3 to stay alive). Food in calories, more of them the colder the room (2,100 plus 100 for every 5 °C below 20). Sleep. Illness that builds in rooms below 18 °C. An optional daily medicine whose absence starts to harm after about a week.
- **The home.** Heat (stove, fuel, which rooms you can keep warm), water (snow costs fuel to melt; a water point costs a queue and a walk), stores, curtains and boarded windows, a door that holds.
- **Papers and the phone.** The phone holds evidence: chats, photos of vehicles, channels. A check copies contacts. A clean phone is a choice, and so is leaving it at home.
- **Being taken.** Detention is an event a run can survive: days to months off the map, phone, cash and weight lost, sometimes a signature or a promise to inform as the price of release. A small share do not come back.
- **People.** Named neighbours who can warn you that soldiers are on the street, share water and heat, need help (the elderly die first in the cold), get taken, or report you. Empty flats and people known for helping draw visits.
- **Money and goods.** Yuan, then roubles; barter and a flea market; prices on two tracks (packaged goods up, local produce steady).
- **Choices with no right answer.** Stay and comply, stay and resist, or try to leave through some thirty checkpoints and a crossing queue of days.

### Order of work

Each step ends with tests in `crates/sim` and with a bot that plays it many times, as `out/read/bot` does for the night now.

Order revised on 25 November, after the owner set one month to four years as the standard. Save and load and the passing of days come before more content, because no other step can be played or tested over months without them. The steps below keep their names; this is the order to do them in:

1. L1's second half: save and load, game states, on-screen text out of the world state.
2. **Days that pass at a glance:** routines, the day-level model and what stops it, checked against the real-time sim.
3. **The seasons:** the thaw and the river, spring, summer, autumn and the next winter, in the rules and on screen.
4. L3's second part, L4, then L5 stretched from months to years.
5. L6, endings, and the war's own course over four years, from new research on long occupations (Crimea and Donbas since 2014, Melitopol since 2022).

1. **L1, the days.** The clock past 07:00, day and season, sleep, a seeded random generator, game states in the client, on-screen text out of the world state, save and load. The first four were done on 25 November (`docs/rust-l1-receipt.md`); the last three are next.
2. **L2, living.** Water, food, fuel and cold over days; illness; the home as the place that keeps you alive; shops that empty. Done on 25 November (`docs/rust-l2-receipt.md`), with its numbers in `data/living.json` from `docs/research/living.md`.
3. **L3, the occupation's first weeks.** The sweep, the order sheet and curfew, searches in waves, the phone, checkpoints and filtration, detention. First part done on 25 November (`docs/rust-l3a-receipt.md`, numbers in `data/occupation.json`): the end of the assault, the sweep, searches, hiding, detention, curfew, suspicion and visits. Checkpoints, filtration, the phone and neighbourhood waves are next.
4. **L4, people and money.** Neighbours, informers, aid, cash and prices, the market.
5. **L5, months.** Papers and the passport ladder, the network cut, school, the referendum, conscription, resistance and reprisals, the thaw.
6. **L6, endings.**

## Next, from before 25 November (done by 21 November)

1. FPV strikes with the measured miss rate, fibre-optic share and 10 to 20 minute request-to-strike time already in the table.
2. Tracks in snow that a drone can follow, and fog as the weather that blinds drones (both reported).
3. Going inside: flats and basements as places to live, door-to-door searches in the first days, the phone check.
4. More than one night: sleep, the second day, an occupation that changes over weeks the way Kherson's did.
5. Vehicles as they are now used: few tanks, single dashes by motorcycles and buggies, armour kept far back.
6. Camera and map: buildings that fade when you walk behind them, a map of the streets you have walked.

## The rewrite in Rust (from 16 November 2026)

Set by the owner on 15 November 2026:

> 你完全用RUST 重写吧，以黑河为 标准，真实的黑河色彩，越像越好，先把黑河做出来吧

The whole game is being rewritten in Rust, renderer included, and the first thing to get right is Heihe itself. The Godot project stays in the repository until the Rust build does everything it does; after that it goes.

- `crates/sim` knows what Heihe is and what the rules are. It has no renderer, no window and no engine in it, so everything in it is checked by `cargo test`: the map loads, the river is north of the city, no wall faces inward, no shop board reads backwards, you cannot walk through a building, and the Russian numbers behave.
- `crates/game` is the program you run. It uses Bevy to show what `sim` built and to read the keyboard. It decides nothing about the world.
- `data/heihe/city.json` is the map (OpenStreetMap, ODbL). `data/heihe/look.json` is the colour of the city: every colour is listed with the photograph it was read from. `data/russia/forces.json` is the Russian side.

One thing the old build had wrong and the new one is built to prevent: the map file is "x east, z north", and a right-handed world with y up has north at minus z. Drawn the other way the city is its own mirror image, with the river on the wrong side of every street. `sim::soup::world` is the only place that turn is made, and a test holds it.

### Likeness comes first

Order of work on the city, most visible first:

1. Colours and the kinds of building the photographs show (Central Street salmon blocks with red cones, the red and white fronts with green domes, a pale classical front, yellow housing with crimson balcony bands, tan tile blocks, salmon blocks with yellow bays, low white-and-brick shops). Done in the first Rust pass.
2. More of the real town. OpenStreetMap has 198 buildings on the Chinese bank of this map. From 16 November 2026 the rest comes from CMAB (CC BY 4.0), the national building dataset the Beijing and Shanghai games use: its Heihe layer (imagery of 2023-05-14) is cut by `tools/cmab/slice_heihe.py` into `data/heihe/cmab.json`, and `crates/sim/src/cmab.rs` decides which roofs to add and how tall they are. An OpenStreetMap building always wins over a CMAB roof.
3. The things a resident would miss: the teal arch of the Russian goods street, the Ferris wheel, steps up to raised shops, wires overhead, the three-wheelers, quilted door curtains.
4. Then the survival rules from the Godot build move across, one at a time, each with its test.
