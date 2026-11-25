# The seasons, receipt: the year in the rules and on screen

25 November 2026. Branch `claude/rust-heihe`. Third in the order the owner agreed to on 25 November (`docs/DESIGN.md`, "Order of work"), after save and load and the days that pass at a glance. The owner said 推上GitHub再做四季. The push was done first (5161a71); this is the seasons.

Two commits:
- **Rules** (e01dc06): the river's ice, snow on the ground and rain.
- **Look** (this one): the town drawn in each season's colours, from photographs.

## The rules (e01dc06)

The numbers are in `data/living.json`; each has its source in `docs/research/living.md`.

- **The Heilongjiang's ice, by day of the year:**
  - 55 cm in mid-January;
  - about 103 cm at the end of March;
  - gone at the break-up on 26 April (the hydromet centre's average);
  - back from the freeze-up on 18 November (the Chinese normal).
- **Walking on the river:**
  - Open water cannot be walked on.
  - Ice under 10 cm may give way; both numbers are play.
  - The game says when the river breaks up and when it freezes.
- **Snow on the ground, by month** (Blagoveshchensk mean depths). Without snow nobody leaves footprints: not you, not the patrols, not the vehicles.
- **Rain:**
  - Precipitation days by month (Heihe): above freezing it rains, below it snows.
  - A new `Weather::Rain`. Rain caught in pots is water for two days.
  - With no snow and no rain there is nothing to melt, so water must be fetched.

## The look

### What a season is (`crates/sim/src/look.rs`, `Season`)

- **Snow:**
  - deep from a monthly mean of 3 cm;
  - patchy from 1 cm (October's mean is ground white some days and bare on others);
  - none below that.
- **The river:**
  - open when its ice is gone;
  - "thin" under 20 cm, which is the weeks after freeze-up and the last days before break-up. 20 cm is play.
- **The trees.** No source gives Heihe's leaf dates, so the dates are play, bracketed by dated photographs:
  - bare until 10 May (no leaves on 11 April);
  - first leaves until 5 June (first leaves on 21 May, willows in leaf on 26 May);
  - summer through August;
  - full leaf with the lawns yellowing in September (full leaf on 7 September);
  - turned until 25 October (autumn colour on 13 October, half bare on 12 October);
  - down until the snow.

The town is drawn again when the season changes: a handful of times a year (the test allows 6 to 14). Redrawing takes 0.5 to 0.65 s on this laptop, and the game pauses for that long.

### Colours from photographs

- **The research:** `docs/research/seasons-look.md`, by a research agent.
  - 86 Wikimedia Commons photos screened; colours read from 26 of them.
  - Each colour is the per-channel median of a marked region, after converting the photo's colour profile to sRGB.
  - Each region was drawn back onto its photo and checked by eye.
  - `out/research/seasons/scripts/final_samples.sh` measures every colour again.
- **Merged into `data/heihe/look.json`** by `out/seasons/merge_look.py`:
  - 61 new colours and 27 new photo entries;
  - the table now has 132 colours from 38 photographs, each `from` naming one.
- **The colours the seasons use** (`SEASON_COLOURS`, checked present by a test):

  | What | Colour | Photograph |
  |---|---|---|
  | Dead verges in April | `verge_dormant_spring` | 11 April 2007 |
  | Summer grass | `grass_summer` | a mown verge, 13 July 2021 |
  | September grass | `grass_late_summer` | 7 September 2024 |
  | October grass | `grass_autumn` | 22 October 2023 |
  | Straw after leaf-fall | `grass_autumn_straw` | 22 October 2023 |
  | Dry asphalt | `asphalt_summer_sun` | 7 September 2024 |
  | The road in April, pale with the winter's grit | `road_dust_spring` | 11 April 2007 |
  | Open water: the clear sky's blue, deeper mid-river | `river_water_clear`, `river_water_clear_deep` | August 2017 |
  | Floes and young ice | `river_floes`, `river_young_ice`, `river_floe_gaps` | the November freeze-up at Blagoveshchensk; no photo of the April break-up at Heihe was found, so these stand in for both |
  | Leaves: first spring green, summer, autumn | `foliage_spring`, `foliage_summer`, `foliage_autumn` | 26 May 2020, August 2017, 13 October 2024 |
  | Roofs: red and blue metal | `roof_metal_red`, `roof_metal_blue` | ordinary six-storey housing, August 2017 |
  | Roofs: pale flat | `roof_port_light` | flat roofs across the water at the port |
  | Far-bank roofs: red and grey metal | `far_roof_red`, `far_roof_grey_metal` | Blagoveshchensk, from the passenger boat |

  No photograph shows Heihe's roofs from above, so each building takes one of the three roof colours at random by its id. That is play.
- **Two corrections the research found:**
  - **The river ice.** `river_ice` and `river_ice_dark` named `mother`, an August photo with no ice in it. They are now read from a 2015 winter photo of the snow-covered ice in front of Heihe (`#BCC2D5` and `#AAB0BF`). The winter river is slightly less blue than before.
  - **The `memorial` photo** is CC BY 4.0, not CC BY-SA.

### How the palette changes (`Palette::seasonal`)

- **Winter.** The town as it was drawn; a test checks that the winter palette draws the same triangles in the same colours.
- **"No colour".** A colour can now be absent (`Rgb::NONE`), and nothing is drawn in it. So in summer:
  - the snow on roofs, ledges, lamp heads and branches is not there;
  - the snow piles are not there;
  - the ice shoved up on the bank is not there.

  Mixing a colour with an absent one leaves it as it is.
- **Without snow:**
  - the open ground is the season's grass;
  - the roads are dry asphalt;
  - the roofs are roofs.
- **Paving.** Four metres of paving appears round every building on the Chinese bank. In winter it is under the snow and not drawn. Without it, summer had grass up to the shop doors.
- **Patchy snow.** The ground and the roofs are half grass, half snow.
- **The trees:**
  - The pollarded street trees and the park's birches carry a crown in the season's leaf.
  - The spruces stay as they are.
- **The river:**
  - **Open:** blue water.
  - **Thin ice:** grey floes.
  - **In April, before the ice thins:** the winter ice with no snow on it.

### In the game

- **The town redrawn** (`crates/game/src/main.rs`, `season_town`). When the season changes, the town's meshes are thrown away and built again from the new palette. The street lamps' lights and the shop boards stay.
- **Rain:**
  - the snowflakes become long streaks falling four times faster;
  - the sun drops to a fifth;
  - the light comes from a grey sky.

  The winter's flurries and river fog keep the light they had.
- **The weather line** says 晴 above freezing, 晴冷 below.
- **The cut-away glass.** A building that hides you used to turn to glass tinted by its own colours. Under snow-white roofs that was a pale veil. Under red and blue metal roofs it was dark enough to hide half the screen. It is now drawn from a plain white copy of the building, made the first time it hides you.

## Checked

- **`cargo test -p sim`: 142 pass.** The season tests:
  - **`the_seasons_follow_the_calendar`:**
    - 17 January is the winter as drawn;
    - July has no snow, open water and summer leaves;
    - 7 May is still bare and 17 May in first leaf;
    - 9 September is late summer, 9 October autumn and 29 October bare;
    - 23 April and 25 November are thin ice;
    - mid-December is deep snow on solid ice;
    - the look changes 6 to 14 times in a year.
  - **`the_table_has_every_season_colour`:** every name the seasons read is in `look.json` with a photograph.
  - **`winter_is_the_town_as_drawn_and_summer_has_no_snow`:**
    - winter is identical to the town as drawn;
    - summer draws nothing in an absent colour and has no snow anywhere;
    - summer has grass, open water, leaves, metal roofs and dry asphalt, and no shore ice;
    - the April thaw has floes and dead verges, and no leaves.
- **What is drawn** (`cargo run -p sim --release --bin scene_report -- <day>`):

  | | Winter | 20 April | 10 July |
  |---|---|---|---|
  | Triangles in all | 2,482,517 | 2,247,119 | 2,270,739 |
  | Ground | 2,606 | 90,654 | 90,634 |

  The paving adds about 88,000 triangles. The snow on roofs and ledges that is gone takes away about 300,000. Summer draws less than winter.
- **Pictures** (`--interior-shots`; copies in `docs/shots/rust-seasons/`):
  - 04, 18 January (winter, as before);
  - 09, 20 April on Central Street (bare, paving, dead verges, the road pale);
  - 10, 15 May on the bank (open water, first leaves on the birches);
  - 11, 10 July at noon (grass, trees in leaf, red and blue roofs);
  - 12, 12 July in the rain (overcast light, streaks);
  - 13, 5 October (turned leaves, autumn grass);
  - 14, 25 November (snow, floes on the river).

  `year_at_a_glance.jpg` puts them side by side. `05_river_winter_ice.jpg` is the winter river with the re-read ice colour.

## Not done, or not right yet

- **Open water is one colour.** Under cloud the Amur reads grey (`river_water_overcast`), under sun blue. The water stays blue in the rain.
- **Rain does not wet the roads.** No photograph of Heihe's streets in rain was found.
- **The player wears the winter coat all year.** Nothing yet makes summer heat a problem.
- **Summer days swing too far.** The day-to-day temperature swing was fitted to January and is used all year, so a July afternoon can reach +33 °C, even in rain.
- **Brick in daylight.** `pavement_brick` comes from a January night photo under sodium lamps. Without snow its full colour shows, and the park path is very orange. No daylight reading of the brick was found.
- **The redraw pause.** A season change pauses the game for about half a second.
- **Break-up and freeze-up both look the same.** Both are drawn with the November freeze-up's floes.
- **Flood and drift.** The spring ice drift and the summer floods are not in the rules.
