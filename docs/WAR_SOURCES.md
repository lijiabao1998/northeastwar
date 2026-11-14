# Sources for the combat layer

The fighting in this build is fiction set in Heihe in 2031. No real person is named. Civilians are not a scoring target: squads only shoot at the player character, and there is no points system at all.

The shape of the fighting follows open descriptions of the Russo-Ukrainian war from 2022 onward. The numbers in the game (damage, ranges, squad size) are made up so a laptop can run them. They are not a manual and not a claim about any real unit.

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

## What this file is not

It is not a recommendation to copy any of these tactics, and it is not a source for making weapons or explosives. The citations are for why the fiction looks the way it does.
