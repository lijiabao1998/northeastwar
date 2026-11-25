# N1 receipt: the night, honest

25 November 2026. Branch `claude/rust-heihe`. This is the first item of the plan the owner said to follow (按照這個做): make the night honest, with a bot that plays it. The bot is an agent's harness in `out/read/bot2` (not in the repository). It played 8 strategies × 5 seeds of the night on the rules as Gemini left them on 21 November, and ran probes for single rules. Its table is `out/read/bot2/runs/table.md`. This round fixes what it found.

## What the bot found, and what changed

1. **Lancets and FPVs struck a player nobody had seen.**
   - Cause: `under_fire_for` was set for 6 s whenever any squad anywhere was in Engage, including one fighting the militia 400 m away. The Lancet launched on that flag alone, and the FPV's "spotted now" check counted it too.
   - Probe result: 7 Lancets in 600 s at a player in the basement back room, while the militia fought 419 m away.
   - Fix: the flag now means the player is under fire. It is set when a squad in Engage sees the player that tick, or when a soldier shoots at the player.
2. **Armour saw in total darkness and into rooms.**
   - A vehicle needed only range and a clear line. A T-72 locked a player standing still in the blackout at 70 m. A BMD-4M hit a player inside the home who had never been seen; a `pd <= 45` clause counted anyone within 45 m as exposed.
   - Fix: a crew now finds a person by the same light as a soldier with a night sight: 55 m in the dark, scaled by gait, shelter and fog, and further when you fire. Indoors, it finds you only once infantry have.
   - The 55 m is play. `docs/research/ukraine-performance.md` has no figure for Russian vehicle thermals; it only says night sights are scarce at squad level. A sourced vehicle-sight table is owed.
3. **Barricades did not stop vehicles.**
   - A catch-all match arm flattened concrete, dead cars and wire between 2.8 and 4.2 m, before their own rules applied.
   - Concrete lost 55 hp per tick, not per second, because `if heavy { 55 } else { 28 } * dt` multiplies only the else branch.
   - Fix: both. Against a heavy tank, concrete now holds 4.4 s and a dead car is shoved; wire stalls and hurts light vehicles.
4. **Fires never went out.**
   - Every Molotov, every RPO hit and every knocked-over fire barrel kept warming and lighting its spot all night.
   - Fix: a Molotov burns 90 s, an RPO hit 45 s, and a barrel goes out when it is knocked over or destroyed.
5. **Blasts hit you at full strength anywhere inside their radius, and walls did nothing against them.**
   - Fix, outside the mortar table:
     - damage falls to half at the edge (the formula the militia already had);
     - indoors, the room's own protection applies (basement 0, back room 0.08, front hall 0.22) unless the blast is in the same room.
   - Found on the way: a blast left its crater before it hurt anyone, so the crater sheltered the player and the militia from that same blast, at 55%. Cover is now taken as it was before the blast.
6. **Vehicles froze for the rest of the night.**
   - Any vehicle on road 13 (a segment that cuts through a building) stopped at (-250,-259). Any vehicle behind a wreck stopped 3.4 m short of it. In every full night all 17 surviving vehicles ended frozen.
   - Fix: a vehicle that wants to move and cannot for 3 s turns round along its route.
   - Road 13 itself is unchanged.
7. **Not changed this round:**
   - The friendly 155 mm has no rule of its own. It goes through the same blast path as item 5, so it now falls off too; the bot has not measured it again yet.
   - Russian armour still finds militia by range alone.
   - Vehicle numbers in `data/russia/forces.json` (the `vehicles` block, added in R5–R6) have no `src` or `basis`, against the rule in DESIGN.md.

The bot also confirmed these hold:
- the challenge fix: 0 shots at a player standing still from 20 to 52 m;
- two-sided rifle fire;
- no sight through walls or closed doors (0 of 126,639 lines);
- the fire barrel no longer helps a drone;
- the neighbour reward is paid once.

## Checked

- `cargo test -p sim`: 107 pass, 6 of them new:
  - `a_fight_elsewhere_does_not_put_you_under_fire`: a squad in Engage 300 m away for 300 s gives no under-fire time, no Lancet at you, no harm.
  - `armour_sees_by_the_light_and_not_into_rooms`: a T-72 does not lock a still player 70 m away in the blackout, does lock at 20 m, and does not lock a player in the home 18 m from its door.
  - `concrete_holds_a_tank_for_seconds_not_one_tick`: at 3.4 m nothing is flattened; against it, two seconds take 110 hp off 240.
  - `fires_burn_down_and_a_knocked_barrel_goes_out`.
  - `a_blast_falls_off_and_the_basement_walls_take_it`: 0.5 m from the centre takes over 35 of 40; the edge less than 60% of that; a blast at the basement door takes less than 15 of 60.
  - `a_vehicle_blocked_by_a_wreck_turns_back`.
- The tests place vehicles on a line they find to be clear near Central Street. The straight line from (-274,-345) towards (-242,-210) that an earlier draft used runs through buildings.
- `cargo build -p northeast --release` builds.
- `days_bot` (3 seeds per shelter, 4 days), compared with the same run before this round:
  - Basement: woken by guns 1 time instead of 40 to 46, slept 9.8 h instead of 6.9 h; still dies of thirst at 15:00 on 19 January.
  - Home: dies of cold at 17:30 on the 18th, three hours later than before.
  - The sim step fell from 0.65–1.0 ms to 0.17–0.25 ms per step, though the machine was also quieter on this run.
- Dying of cold indoors now says so: 你冻死在屋里, instead of the street.
