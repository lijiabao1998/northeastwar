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
