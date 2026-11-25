# Credits and sources

The game is a work of fiction. Characters, dialogue and the 2031 crossing are invented. No real politician is depicted. Real places are used as the setting.

Game code, original audio, shaders and story text: © 2026 李家宝. All rights reserved. See `LICENSE`.

## OpenStreetMap

© OpenStreetMap contributors, [Open Database License 1.0](https://opendatacommons.org/licenses/odbl/1-0/).

- Downloaded 2026-11-14 from the public Overpass API (`https://overpass-api.de/api/interpreter`).
- OSM base timestamp is in `data/heihe/city.json` under `meta.osm_timestamp` (2026-11-14T12:02:56Z).
- Query: `tools/osm_import/regions/heihe_riverfront.overpassql`.
- Origin of the local metre grid: the Mother sculpture, OSM node 7779151765, 50.2510885 N, 127.4960436 E.
- Derived files: `data/heihe/overpass.json`, `city.json`, `city.gltf`, `city.bin`, `import_report.json`. These stay under ODbL. See `data/heihe/ODBL.txt`.

The slice is the Chinese riverfront around 中央街, 王肃街, 文化街, 海兰街, 兴安街, 官渡路 and 邮政路, plus the Amur (黑龙江) and the near skyline of Blagoveshchensk.

## CMAB building outlines

Zhang, Yecheng; Zhao, Huimin; Long, Ying (2024). *CMAB-The World's First National-Scale Multi-Attribute Building Dataset*, v7. figshare. Dataset. [doi:10.6084/m9.figshare.27992417.v7](https://doi.org/10.6084/m9.figshare.27992417.v7). [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

- Downloaded 2026-11-16: `heilongjiang.zip` (figshare file 53667566, 414,319,581 bytes, MD5 `1f21f2de9813547fef849bb26c13176b`, the same as published). The layer covering central Heihe is `processed_1809_20230514_all` (imagery of 2023-05-14). The download and the extracted layer stay in `data/raw/` and are not committed.
- Changes made: cut to 127.4780-127.5110 E, 50.2420-50.2580 N (centre inside); reprojected from Web Mercator to the local metre grid above; rounded to centimetres (`tools/cmab/slice_heihe.py` writes `data/heihe/cmab.json`). Then, in the game (`crates/sim/src/cmab.rs`): every outline is moved 2 m west and 6 m north to sit on the OpenStreetMap map and simplified at 0.3 m; each outer ring becomes its own building and inner rings (yards, light wells) are filled; roofs that overlap OpenStreetMap buildings (30% either way), that a street or footpath runs through, on the river or the promenade, over a landmark, under 20 m2 or narrower than 2.8 m are left out; heights are turned into storeys as that file explains. CMAB's heights are also used to replace the guessed heights of OpenStreetMap buildings that one CMAB roof matches. See `docs/rust-r2-receipt.md`.
- CMAB's outlines are traced from imagery; its heights and attributes are model estimates, not surveys.

## What is measured, what is estimated, what is dressing

Measured from OSM, and kept:

- Building footprints and the road centrelines, including the pedestrian 中央街 and 王肃街 along the park.
- The Amur water polygon (relation 5756550, 黑龙江 / Амур) and the bank line derived from it. At the Mother sculpture's longitude the south bank is about 56 m north of the sculpture and the river is on the order of 750 m wide.
- Place names in Chinese and Russian on shops, hotels and streets (肯德基, 蜜雪冰城, 黑河国际饭店, 瑞幸咖啡, and Cyrillic names such as Снеговик where OSM has `name:ru`).
- The Mother sculpture node, boundary stone 192 (node 13246224095), and 旅俄华侨纪念馆 (node 12133250372) inside building way 1079423094.
- 黑河市热电有限责任公司, OSM way 229876678: `power=plant`, `plant:source=coal`, `plant:output:electricity=119 MW`. It sits about 2.3 km south of the promenade, outside the walked blocks. The smoke column is placed on that coordinate. Two `man_made=chimney` nodes are included the same way; the nearer one is on the Russian bank, not in the walked streets.
- Rooftop characters use real hotel and museum names from OSM nodes, hung on the building that contains the node.

Estimated, because the tags were not there:

- Building height in the Chinese core. A count of the core bbox found building footprints and essentially no `building:levels` or `height`. Heights are guessed from footprint area: mostly 6–8 storey slabs, with a few mid-rises. The guess is tagged `height_source: estimated_from_footprint` in `city.json`. Where OSM did have levels (mostly Blagoveshchensk), those values are used. `city.json` itself is unchanged, but in the Rust build (from R2) the OSM buildings that one CMAB roof matches take CMAB's reading instead, the hashed towers no CMAB roof confirms are capped at 7 storeys, and the rest of the town comes from CMAB (see the CMAB section).
- The memorial footprint had no levels. It is forced to two storeys because the 12 October 2025 street photo shows a two-storey facade. Tagged `poi_plus_reference_photo`.

Dressing, not from a survey of each object:

- Ceramic tile colours. The mix of beige, yellow, red and pink is taken from the January 2026 street photos, not from a per-building OSM colour tag.
- Enclosed balcony bands on Chinese slabs of four floors or more. The type is typical. Individual balconies are not in OSM.
- Bare pollarded trees and sodium lamps. The extract contained zero `natural=tree` nodes and zero `highway=street_lamp` nodes, so both are generated along the mapped roads and tagged `placed_along_osm_road`.
- Snow piles, parked cars, the brick sidewalk material and the promenade pavers.
- The Mother sculpture mesh is an abstract stand-in at the real coordinate. It is not a copy of the artwork.
- Boundary stone 192 is a simple dark obelisk at the real node, not a measured model of the stone.
- All spoken lines, the friend 赵小北, and the wording of the municipal alert. The alert is fiction.

## Photographs used as reference

None of these files are committed. They were looked at while choosing colours, the balcony/window pattern, the pollarded trees, the brick sidewalks and the pink memorial. Licences below are as recorded from Wikimedia Commons on 14 November 2026.

| File | Licence | What it informed |
| --- | --- | --- |
| [Streets of Heihe 04.01.2026.jpg](https://commons.wikimedia.org/wiki/File:Streets_of_Heihe_04.01.2026.jpg) and (2), (3), (4) | CC0, SmallSonMarex, 4 January 2026, about 18:48 | Night streets: coloured tile, white uPVC windows, enclosed balconies, brick sidewalks, curb snow, sodium lamps, hooded black coats, overhead wires, bilingual shop boards |
| [黑河旅俄华侨纪念馆街景 2025-10-12.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E6%B2%B3%E6%97%85%E4%BF%84%E5%8D%8E%E4%BE%A8%E7%BA%AA%E5%BF%B5%E9%A6%86%E8%A1%97%E6%99%AF_2025-10-12.jpg) | CC BY 4.0, Hehua, 12 October 2025 | The pink arched memorial on 王肃街, and the two-storey height override |
| [黑河市俄罗斯商品街.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E6%B2%B3%E5%B8%82%E4%BF%84%E7%BD%97%E6%96%AF%E5%95%86%E5%93%81%E8%A1%97.jpg) | CC BY-SA 4.0, HCCB3947, 7 September 2024 | Bilingual shopfronts and the enclosed-balcony apartment type in daylight |
| [黑河市，母亲雕塑2017.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E6%B2%B3%E5%B8%82%EF%BC%8C%E6%AF%8D%E4%BA%B2%E9%9B%95%E5%A1%912017.jpg) | CC BY-SA 4.0, Amarespeco, 13 August 2017 | That the Mother sculpture is the riverfront landmark. The in-game mesh is not a copy of it |
| [Blagoveshchensk, Russia --- Heihe, China (8789415981)](https://commons.wikimedia.org/wiki/File:Blagoveshchensk,_Russia_---_Heihe,_China_(8789415981).jpg) and [China river Russia (8788808173)](https://commons.wikimedia.org/wiki/File:China_river_Russia_(8788808173).jpg) | CC BY 2.0, Sharon Hahn Darlin, May 2013 | How close the two skylines sit, and the long paved embankment. These are summer daylight views from the Russian bank, so they were not used for winter colour |

Sunset time used for the sky (17 January, about 15:58 CST, sun around 238°) is from public sunrise tables for Heihe, not from OSM.

## Fonts

- Noto Sans SC (variable) and Noto Sans Regular, SIL Open Font License 1.1. See `assets/fonts/OFL.txt`.
- Chinese sign text is drawn with Noto Sans SC. Cyrillic falls through to Noto Sans.

## Audio

Wind, snow crunch, the phone tone and the distant booms are synthesized in `tools/audio/gen_audio.py`. They are original. No recordings are included.

## CC0 textures and characters

These files are in the repo and are not covered by the proprietary code licence.

- Textures from [Poly Haven](https://polyhaven.com) (CC0): `snow_02`, `snow_04`, `asphalt_02`, `rectangular_facade_tiles`, `yellow_plaster`, `white_plaster_02`, `red_brick_03`, `granite_tile`, `concrete_wall_001`. 1K JPG, diffuse and normal (and roughness for snow and asphalt). Files live in `assets/textures/polyhaven/`.
- Character files from [Quaternius](https://quaternius.com) (CC0): Casual Male, Casual Female and Worker Male, kept in `assets/models/quaternius/`. The street figures in the current build are original low-poly coats and fur hats, and white winter camouflage with a helmet and rifle, because those CC0 meshes read as casual clothes rather than a Northeast winter street.

## Engine

[Godot Engine 4.7.2 stable](https://godotengine.org/download/archive/4.7.2-stable/) (commit `ed1daf0bf`, 18 August 2026), MIT License. This project is not affiliated with Godot.
