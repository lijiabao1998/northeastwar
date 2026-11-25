# Sources for the combat layer

The fighting in this build is fiction set in Heihe in 2031. No real person is named. Civilians are not a scoring target: squads only shoot at the player character, and there is no points system at all.

The shape of the fighting follows open descriptions of the Russo-Ukrainian war from 2022 onward. The numbers in the game are kept in one table with a source and a basis for each (see "The table and its keys" below). They are not a manual and not a claim about any real unit.

## What was taken from open reporting

- Infantry in small groups, often around a handful of soldiers up to about eight, rather than a full company walking in line. RUSI's *Meatgrinder* (Jack Watling and Nick Reynolds, 2023) describes Russian infantry split by role into line, assault, specialist and expendable troops, used in small task groups. https://www.rusi.org/explore-our-research/publications/special-resources/meatgrinder-russian-tactics-second-year-its-invasion-ukraine
- Fire and movement, suppression, and flanking inside towns: one part of the group shoots while another moves. That is ordinary infantry practice, and it matches how urban fights in Ukraine are described once the artillery lifts. Anthony King's "Urban Warfare in Ukraine" (Canadian Army Journal, 2024) is the overview used here. https://www.canada.ca/en/army/services/canadian-army-journal/articles/2024/21-1-king-urban-warfare-in-ukraine.html
- First-person-view drones used as one-way weapons, cued by a recon quadcopter, and grenades dropped from small drones. RUSI's *Tactical Developments During the Third Year of the Russo-Ukrainian War* (Watling and Reynolds, February 2025) describes FPV drones, Lancet-type munitions and artillery as one side of an "offensive triangle" with infantry and glide bombs, and notes wire-guided FPVs with a short practical range. https://www.rusi.org/explore-our-research/publications/special-resources/tactical-developments-during-third-year-russo-ukrainian-war
- Artillery and mortars still decide streets. The same paper, and RUSI's *Stormbreak* / *Preliminary Lessons from Ukraine’s Offensive Operations, 2022–23*, describe guns firing, then moving, and drones finding targets for them. A whistle then a blast is the game's shorthand for that, not a sound recording. https://static.rusi.org/lessons-learned-ukraine-offensive-2022-23.pdf
- Armour that advances with infantry, not alone. *Meatgrinder* and the offensive-operations paper both describe dismounted infantry with a few infantry fighting vehicles. The game uses one vehicle and a squad, not a battalion.
- Casualty evacuation under drone observation. The third-year RUSI paper says persistent drones made resupply and evacuation dangerous. In the game, bleeding stops only if you stand in a shop doorway. That is a simplification, not a medical guide.
- Civilians in basements and doorways. Urban-warfare reporting from 2022–2024 describes people sheltering below street level. Here they run for the buildings. Soldiers are not ordered to shoot them, and the game never awards anything for it.

## What was invented

- Every soldier, the timing of patrols, hit points, the rifle's damage, and the idea that one person on 中央街 is fighting a squad.
- The look of the drones, the vehicle and the rubble. No photograph of a wreck was copied.
- Sounds. `tools/audio/gen_audio.py` synthesises the whistle and the drone buzz. They are not field recordings.
- Building collapse as three or four vertical slices cut out of the OpenStreetMap footprint. Real structures fail in more complicated ways. The slices exist so a laptop can drop a storey without simulating every brick.

## The table and its keys

From 15 November 2026 the enemy's numbers are no longer made up in code. They are in `data/russia/forces.json`, and the findings they were set from are in `docs/research/ukraine-performance.md` (twelve topics, each row with its period, source and how far to trust it). The `src` keys in the table mean:

| Key | Source | Used for |
|---|---|---|
| `rusi-meatgrinder` | RUSI, *Meatgrinder* (Watling and Reynolds, 2023). https://rusi.org/explore-our-research/publications/special-resources/meatgrinder-russian-tactics-second-year-its-invasion-ukraine | Drone-cued fire arrives in about 3 to 5 minutes. |
| `rusi-third-year` | RUSI, *Tactical Developments During the Third Year* (February 2025). https://www.rusi.org/explore-our-research/publications/special-resources/tactical-developments-during-third-year-russo-ukrainian-war | 60 to 80 percent of FPVs miss; about a dozen FPVs per tank; drone orbits kept over an axis. |
| `rusi-emergent-2025` | RUSI, *Emergent Approaches to Combined Arms Manoeuvre in Ukraine* (October 2025). https://www.rusi.org/explore-our-research/publications/insights-papers/emergent-approaches-combined-arms-manoeuvre-ukraine | Infiltration groups of 2 to 5 men. |
| `uk-mod-2023` | UK Defence Intelligence update, 12 December 2023, as relayed in the press. | Night-vision gear is short at squad level. No share is given; the 0.2 in the table is derived. |
| `wotr-fpv-2025` | War on the Rocks, "I Fought in Ukraine and Here's Why FPV Drones Kind of Suck" (June 2025). https://warontherocks.com/2025/06/i-fought-in-ukraine-and-heres-why-fpv-drones-kind-of-suck/ | About 15 minutes from request to FPV strike; a mortar answers in under 5. Ukrainian-side figures. |
| `fpri-2026` | FPRI, "Russia's Drone Line Experiment" (April 2026). https://www.fpri.org/article/2026/04/russias-drone-line-experiment/ | About a fifth of attack drones are fibre-optic. |
| `hrw-bucha` | Human Rights Watch, "Ukraine: Russian Forces' Trail of Death in Bucha" (April 2022). https://www.hrw.org/news/2022/04/21/ukraine-russian-forces-trail-death-bucha | People outside were stopped, searched and sometimes shot at; the game's challenge and search. |
| `king-urban` | Anthony King, "Urban Warfare in Ukraine" (2024), listed above. | Grenades before entry. |

What the research could not find is as important as what it found, and the table says so with `basis: "play"`: there is no open figure for rifle hit rates in street fighting, for how far a soldier or a drone sees a person at night, or for anything at all below about minus 20°C. Those numbers were chosen for play and should be read that way.

## What this file is not

It is not a recommendation to copy any of these tactics, and it is not a source for making weapons or explosives. The citations are for why the fiction looks the way it does.
