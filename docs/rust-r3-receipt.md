# R3 receipt: frame time

17 November 2026. Branch `claude/rust-heihe`. The owner said 先處理幀時間: deal with the frame time first. R2 had left it at 12 ms at the Mother Square and 21–36 ms elsewhere, measured in a dev build.

## Result

R2's build (commit 0380ea3) and this round's took the same eleven `--shots` views in turns, three runs each, at 3200 × 1800 in release builds. Each run started with no other session's test browser running; three of the six ended with one running. The table gives each view's frame time, as the median of the three runs (each run averages 24 frames). Every run is in `docs/shots/rust-r3/before-after-runs.txt`.

| View | R2 (ms) | R3 (ms) |
|---|---|---|
| 01 Central Street, noon, zoom 1 | 34.8 | 18.8 |
| 02 Central Street, wide, zoom 0.42 | 44.4 | 16.4 |
| 03 housing, noon, zoom 0.8 | 47.4 | 16.0 |
| 04 Mother Square | 32.9 | 16.4 |
| 05 river and far bank, zoom 0.16 | 37.5 | 20.1 |
| 06 whole town, zoom 0.14 | 59.2 | 19.8 |
| 07 Central Street, dusk | 45.8 | 22.9 |
| 08 housing, night | 46.7 | 18.9 |
| 09 Central Street, night, zoom 0.6 | 39.5 | 18.7 |
| 10 shop boards, zoom 2.2 | 54.7 | 14.0 |
| 11 shop boards, night | 46.2 | 18.4 |

Within this session, frames are now 1.9 to 3.9 times faster: 14–23 ms where they were 33–59. They are not yet steadily under 16.7 ms.

R2's own build ran 1.3 to 2.9 times slower here (33–59 ms) than in R2's receipt (12–36 ms, dev build), and round 1 of the bench, also with R2's settings, agrees. The cause is not known. It is not the dev build, since here the release build was the slower one. On this machine, numbers from different sessions do not compare; only runs taken in turns within one session do. Against R2's published medians, R3's are 0.7 to 2.2 times as fast, and slower at the Mother Square (16.4 against 11.5) and at dusk (22.9 against 20.6).

## How it was measured

- **The release build.** Every R2 number came from the dev build, whose dependencies are optimised but keep their debug assertions, so Bevy and wgpu run extra checks. This round times the release build, which is what a player runs. The first release build took 42 minutes on this laptop.
- **`northeast --bench <folder>`** (`crates/game/src/bench.rs`) stands at some of the `--shots` views and changes one setting at a time. It runs one warm-up round, then three counted rounds of 40 frames per variant, each after 20 frames to settle. For every frame it records:
  - the frame time;
  - the main thread, from First to Last;
  - the render thread's Render schedule, which includes encoding, submitting and presenting;
  - the GPU's own timestamps for each render pass.

  MSAA resolves and the swap chain fall outside the timestamps.
- **Two mistakes in the bench were found in review and fixed before round 6.** That is why rounds 1–5 are kept but not trusted for everything.
  - **The order of the variants.** It only rotated from round to round, so each variant always ran right after the same one. In rounds 1 and 3 the baseline always followed the variant that resizes the window, and read up to 15 ms of GPU time high (1 to 15 ms, depending on the view). Now every other round runs in reverse, and the variants that resize the window or swap the town's meshes run last.
  - **The shadow pass was missing.** Bevy runs it twice a frame under one name, and the bench kept only the second, empty run. From round 6 it adds up both: the shadow pass takes 1–2 ms.
- **The machine is shared.** Other sessions' headless test browsers ran during parts of most rounds; `tools/bench/quiet_bench.sh` waits up to 20 minutes for none to be running before it starts a bench, and prints how many were running before and after (that line was not kept for rounds 1–6). The GPU timestamps moved with that load as well: the same variant's GPU time differs by 2–11 ms between rounds in the noisy runs. A change is taken as measured only where it holds in a clean round. Every table is in `docs/shots/rust-r3/bench-r1.txt` to `bench-r6.txt`.
- **`cargo run -p sim --bin scene_report`** counts what is drawn:
  - 867 buildings on the Chinese bank, drawn as 2,585 meshes;
  - 1.1 million triangles in all;
  - 498 lamp heads on 332 posts;
  - 90 shop boards.

## Where the time went

- **The GPU, shading pixels.** In rounds 3 to 6 the main thread took 2–6 ms. The render thread took the whole frame, waiting on the GPU. Where the run was quiet, the GPU's passes summed to within 1–3 ms of the frame. Halving the width and height cut the GPU's time about three times. Merging every building into one mesh made it slower: Bevy 0.20 already draws the buildings with multi-draw indirect, and a merged mesh can no longer be culled.
- **Street lamps that gave no light.** By day the lamps stood at intensity 0. But a light of zero intensity is still clustered, and still shaded in every pixel it reaches.
- **Shading the same pixel several times.** The opaque pass is not sorted by depth, and nothing drew depth first.
- **4x MSAA**, most of whose cost is in resolving and loading the multisampled targets, where the timestamps do not see it.
- **The cut-away glass.** It was shaded like a wall (sun, filtered shadows, ambient light and, after dark, every lamp) over a whole building's worth of screen: about 3.5 ms by day with no lamps, 4–5 ms at night.

## What changed

GPU time unless the row says otherwise, at 3200 × 1800, from the round named.

| Change | Effect | Seen on screen |
|---|---|---|
| Lamps hidden by day, not dimmed to zero (`sky.rs`) | −2.8 to −6.7 ms by day, with the other defaults (round 6). It was larger before the depth prepass. | No |
| Depth prepass (`graphics.rs`, on by default) | −1.1 to −8.1 ms, most at night (round 5) | No |
| MSAA off by default | Frame −5.9 to −11.9 ms, GPU −1.3 to −5.5 ms (round 6) | Close up, slightly rougher long edges (`msaa_4x_2x_off.jpg`). Zoomed out to the whole town, lines under a pixel wide break into dots and will shimmer when the camera moves (`msaa_whole_town_lines.jpg`). |
| Cut-away glass unlit in full day and full night, its colour set by the hour; lit in twilight (about 06:46–07:50 and 15:34–16:38) | −3 to −5 ms where a building is cut away, outside twilight (round 2, one round) | By day the cut-away walls lose their light and dark faces and turn flat (view 02: 3.5% of the frame changes, by up to about 35–50 levels; `ghost_noon_lit_unlit.jpg`). At night it matches the lit glass (`09_night_ghost.jpg`). Twilight keeps the lit glass and its cost (`07_dusk_ghost.jpg`): an unlit tint there was a grey veil twice as bright. |
| One light per lamp post, between its heads, as bright as all of them, with no size | About −1.3 to −1.9 ms at view 08 at night; at view 09 −0.4, within the ±1 ms by which two identical rows differ at night (round 6) | The pools on the ground are the same within 10% (`lamps_heads_posts_night.jpg`). The post's own shaft is no longer lit by its lamp (`lamp_post_heads_posts_close.jpg`). |
| Camera near plane just in front of the nearest roof, following the zoom | −1.0 to −1.2 ms at night (round 4); nothing by day | No |
| Lights and materials touched only when the lamps switch; window glow set once; the HUD text set only when it changes | Avoids Bevy preparing again everything drawn with a changed material. Not measured: in `--bench` the clock stands still. | No |

Tried and not taken:

- **FXAA:** its own pass took 2.1–5.1 ms.
- **2x MSAA:** no faster than 4x on this GPU.
- **Hard shadows** (one hardware tap instead of nine): −0.7 to −1.6 ms (round 6), but every shadow edge stair-steps close up (`shadows_soft_hard.jpg`).
- **One merged mesh** for the buildings: +3 to +11 ms.
- **Lamps of radius 0.2 instead of none:** no change in cost (round 4). Radius 0 is used for the post lights because of how they look.
- **8 cluster slices instead of 24:** slower.
- **Lamp range 18 m instead of 22:** −3 to −4.5 ms at night (round 2, an unsteady round). By the falloff, it would leave darker gaps between the pools along every street; that was not looked at.
- **Stopping the windows from casting shadows:** −1.0 to +0.6 ms, no consistent saving (round 6).

Each choice can be switched with an environment variable, listed in `crates/game/src/graphics.rs`. For example: `NORTHEAST_MSAA=4`, `NORTHEAST_SHADOWS=hard`, `NORTHEAST_LAMPS=heads`.

R2 noted that noon was slower than dusk at the same spot. The GPU time was the same at both hours (round 1, rows that leave the main pass alone: 25.7 and 25.8 ms with the windows casting no shadows, 18.8 and 18.8 with the lamps hidden). The gap was in the frame times only, and why is not known. In this round's before-and-after runs, dusk was the slower of the two in both builds.

## Checks

- **Review.** Three reviewers read the round: the code, the claims, and the pictures pixel by pixel against R2's and against each single setting. A second agent for each side tried to refute its findings. What stood is fixed above: the bench order, the missing shadow pass, the twilight glass, where the post light stands, and several numbers in this receipt and in `graphics.rs`. Two more things also stood and are fixed: the bench set the glass unlit at twilight views, and its CSV split names containing commas. One claim was refuted: the noisy frame times for views 09 and 10. A last check of this rewritten receipt found nine more wrong or loose statements; all stood and are corrected above. Their work is in `out/r3review/` and `out/r3check/` (not committed).
- **Near plane.** The nearest thing drawn stays 14–16 m beyond the near plane at every zoom from 0.12 to 3, for the tallest building on the map. No roof is cut at the bottom of any view.
- **`cargo test -p sim`:** 48 tests, all passing.
- **Pictures.** `northeast --shots` writes all eleven views with the new defaults. I looked at them.

## Not done, not checked

- **Under 16.7 ms everywhere.** The main pass still takes 9–14 ms of GPU time at 3200 × 1800. The next levers:
  - Draw the town at a lower internal resolution and scale it up, keeping the HUD sharp. Bevy 0.20 has no plain switch for this. It needs the image-target path, which leaked memory in `--shots` in R2 and would have to be looked at first.
  - Split the props mesh. Its 119k triangles are drawn whole every frame, because its one bounding box always overlaps the view.
  - The sun's shadows cost 3.4–5.3 ms of GPU time in all (round 6): 1.0–1.9 ms is the shadow-map pass, and 2.3–3.2 ms is reading the shadow map in the main pass. Going from 9 taps to 1 saves only 0.7–1.6 ms of that, so most of it is reading the map at all.
- **Thin lines when zoomed out.** With MSAA off they break up, as above. Nothing is done about it yet.
- **Not timed:** frames while walking, and frames while the clock runs. Only standing still was timed.
- **Not played by hand:** nobody has played the release build yet.
