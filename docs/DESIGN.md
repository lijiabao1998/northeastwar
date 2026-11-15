# Design: survive it, as an ordinary person

Set by the owner on 15 November 2026:

> pz的模式 但是是换成现代战争，俄罗斯的装备完全取决于 在乌克兰战争当中的表现

So the game is Project Zomboid's loop (needs, scavenging, injuries, hiding, one life) with a modern army where the zombies would be. The army is the Russian one as it has actually fought in Ukraine since 2022.

## Two rules everything else follows

1. **You are not a soldier.** You have no weapon unless the game hands you one. Being seen is the danger, not being out-shot. A run ends when you die, and it starts again at 16:00.
2. **No brochure numbers.** Each Russian weapon, vehicle, drone and habit behaves the way open reporting says it did in Ukraine. The numbers live in one table, `data/russia/forces.json`. Each carries a `src` (which source) and a `basis`: `reported` (the source gives the figure), `derived` (worked out from reported figures) or `play` (the source gives the shape, the number was chosen so the game can be played). The findings and links are in `docs/research/ukraine-performance.md`.

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

## Next, in the order I intend to do it

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
2. More of the real town. The map today is what OpenStreetMap has: 198 buildings on the Chinese bank. The national building dataset used for the Beijing and Shanghai games (CMAB, CC BY 4.0) covers Heilongjiang and would fill in the blocks OpenStreetMap leaves empty.
3. The things a resident would miss: the teal arch of the Russian goods street, the Ferris wheel, steps up to raised shops, wires overhead, the three-wheelers, quilted door curtains.
4. Then the survival rules from the Godot build move across, one at a time, each with its test.
