# Heihe without snow: summer, autumn and spring colours read from photographs

Research for 東北：俄羅斯入侵. Compiled 2026-11-25. The town is drawn only in winter now. This file gives the colours for the rest of the year, read off photographs of the real place in the same way as `data/heihe/look.json`.

Nothing in `look.json` has been changed. Section 3 is a JSON block in its shape, ready to merge.

## How the colours were read

- **Where the photos came from.** Wikimedia Commons only. I searched the categories Heihe, Aihui, Amur River in Heihe, Amur River in Blagoveshchensk, Blagoveshensk-Heihe bridge, Heilongjiang bridge Park, Wang Su Park and Blagoveshchensk. I also ran full-text searches in English, Chinese and Russian, and searched Openverse (which indexes Flickr) for CC0, CC BY and CC BY-SA. Every Flickr photo that was usable had already been copied to Commons, so every source below is a Commons file page.
- **What was downloaded.** 86 photos were screened. Each one is the 1,280 px wide Commons thumbnail, and each sits in its own folder: `out/research/seasons/photos/<folder>/photo.jpg`, with a `source.json` beside it. The scripts are in `out/research/seasons/scripts/`.
- **Colour profiles.** The phone photos by HCCB3947 carry a Display P3 profile. That includes `goods-street`, which `look.json` already uses. One photo carries Adobe RGB and one Apple Wide Color. The sampler converts every embedded profile to sRGB before it reads a pixel.
- **How a colour is measured.** It is the per-channel median of a rectangle chosen by eye. The rectangle is given as fractions of width and height (x0, y0, x1, y1). Pixels with any channel at 250 or above (blown highlights) or with luma below 12 (deep shadow) are left out. The share left out is in `samples.txt`.
- **Checking the regions.** Every rectangle was drawn back onto its photo (`out/research/seasons/checks/final/<folder>.png`) and looked at before it was kept.
- **The lit half (foliage and bark only).** For these the table also gives the median of the brighter half of the pixels. A crown or a furrowed trunk is half leaf or ridge and half gap or groove, so the plain median runs dark.
- **Reproducing it.** `bash scripts/final_samples.sh > samples.txt`, run in `out/research/seasons/`, re-measures every colour in this file.

Two things I learned while measuring. Keep them in mind when using these numbers.

1. **The eye was wrong several times. The numbers were not.** Things that looked orange-tan, golden or brown measured as dull brown, olive or grey-mauve once they were sampled. Examples: the pine trunk, the autumn crowns, the red roof, the "brown" river of 2011. Every value here is the measured one.
2. **The Amur has no single colour.** Under overcast it reads neutral grey (`#76797B`). Under a clear sky it is the sky's blue (`#4D6D8F`, and `#274370` mid-river). The water itself is a silty olive-brown (`#504636`), but you see that only where light passes through it: a wave face, or a few centimetres of water over gravel. A water shader should mix the reflected sky over that body colour. A single flat fill will not match.

---

## 1. Photos

All files are on Wikimedia Commons. The id is the name used in `from`. The folder is where the downloaded copy is.

| id | folder | file (Commons page) | author | licence | date | what it shows |
|---|---|---|---|---|---|---|
| verge-2021 | bus-2021-b | [Heihe Beiya Bus Line 3,Shuchi YTK6721GH,黑N81330.jpg](https://commons.wikimedia.org/wiki/File:Heihe_Beiya_Bus_Line_3,Shuchi_YTK6721GH,%E9%BB%91N81330.jpg) | Rayista Wang | CC BY-SA 4.0 | 2021-07-13 11:12, bright haze | Bus under the red Heilongjiang bridge approach: mown verge, meadow behind a fence, dry road, stone pitching. |
| riverbank-2017-1 | riverside-island-1 | [黑龙江畔眺望大岛-1-2017.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E9%BE%99%E6%B1%9F%E7%95%94%E7%9C%BA%E6%9C%9B%E5%A4%A7%E5%B2%9B-1-2017.jpg) | Amarespeco | CC BY-SA 4.0 | 2017-08-13 09:28, clear | The Amur at low water from the Heihe bank: gravel bars and boulders, open water, the Blagoveshchensk TV mast and brick tower blocks. |
| riverbank-2017-2 | riverside-island-2 | [黑龙江畔眺望大岛-2-2017.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E9%BE%99%E6%B1%9F%E7%95%94%E7%9C%BA%E6%9C%9B%E5%A4%A7%E5%B2%9B-2-2017.jpg) | Amarespeco | CC BY-SA 4.0 | 2017-08-13 09:24, clear | New Heihe riverside works: concrete wall and apron, gravel strip, dumped earth, a lawn and path in front, the Blagoveshchensk embankment and trees behind. |
| wangsu-2017 | wangsu-monument | [王肃公园——烈士纪念碑 2017.jpg](https://commons.wikimedia.org/wiki/File:%E7%8E%8B%E8%82%83%E5%85%AC%E5%9B%AD%E2%80%94%E2%80%94%E7%83%88%E5%A3%AB%E7%BA%AA%E5%BF%B5%E7%A2%91_2017.jpg) | Amarespeco | CC BY-SA 4.0 | 2017-08-13 09:19, sun | Wang Su Park: Scots pine, spruce and broadleaf trees, whitewashed trunks, grey brick paving. |
| wangsu-tomb-2017 | wangsu-tomb | [王肃墓2017.jpg](https://commons.wikimedia.org/wiki/File:%E7%8E%8B%E8%82%83%E5%A2%932017.jpg) | Amarespeco | CC BY-SA 4.0 | 2017-08-13 09:20, sun | Wang Su's tomb: granite posts, hedges, a Scots pine trunk at the right. |
| boat-blag-2017 | boat-blag-1 | [客轮远眺布拉戈维申斯克-1-2017.jpg](https://commons.wikimedia.org/wiki/File:%E5%AE%A2%E8%BD%AE%E8%BF%9C%E7%9C%BA%E5%B8%83%E6%8B%89%E6%88%88%E7%BB%B4%E7%94%B3%E6%96%AF%E5%85%8B-1-2017.jpg) | Amarespeco | CC BY-SA 4.0 | 2017-08-12 16:19, clear | From the Heihe–Blagoveshchensk passenger boat: mid-river water, the Blagoveshchensk embankment wall and revetment, three towers, red and grey roofs. |
| mosque-2017 | mosque-2017 | [Heihe Mosque.jpg](https://commons.wikimedia.org/wiki/File:Heihe_Mosque.jpg) | Antolavoasio | CC BY-SA 4.0 | 2017-08-12 17:21, low sun | The green log mosque. Behind it, six-storey white-tile housing blocks with red and blue metal roofs. |
| promenade-2010 | rogis-2010 | [Вид на Благовещенск с набережной Хэйхэ.JPG](https://commons.wikimedia.org/wiki/File:%D0%92%D0%B8%D0%B4_%D0%BD%D0%B0_%D0%91%D0%BB%D0%B0%D0%B3%D0%BE%D0%B2%D0%B5%D1%89%D0%B5%D0%BD%D1%81%D0%BA_%D1%81_%D0%BD%D0%B0%D0%B1%D0%B5%D1%80%D0%B5%D0%B6%D0%BD%D0%BE%D0%B9_%D0%A5%D1%8D%D0%B9%D1%85%D1%8D.JPG) | 'Rogis95 | CC0 | 2010-08-24 16:24, sun | Heihe riverside promenade: cream parapet panels, stone-clad piers, grey and red pavers, a weeping willow, the river and the far bank. |
| quay-2019 | crossing-01 | [201907 Crossing the Heilong (Amur) from Heihe to Blagoveshchensk 01.jpg](https://commons.wikimedia.org/wiki/File:201907_Crossing_the_Heilong_(Amur)_from_Heihe_to_Blagoveshchensk_01.jpg) | Jonashtand | CC BY-SA 4.0 | 2019-07-07 11:38, overcast | Concrete apron of the Heihe passenger port, the white fence, boats. |
| crossing-2019-04 | crossing-04 | [… 04.jpg](https://commons.wikimedia.org/wiki/File:201907_Crossing_the_Heilong_(Amur)_from_Heihe_to_Blagoveshchensk_04.jpg) | Jonashtand | CC BY-SA 4.0 | 2019-07-07 11:46, overcast | Leaving the Heihe pontoon: a sand bar under the embankment, the embankment trees, grey water. |
| crossing-2019-07 | crossing-07 | [… 07.jpg](https://commons.wikimedia.org/wiki/File:201907_Crossing_the_Heilong_(Amur)_from_Heihe_to_Blagoveshchensk_07.jpg) | Jonashtand | CC BY-SA 4.0 | 2019-07-07 11:47, overcast | Mid-river, looking at Blagoveshchensk: water under a grey sky. |
| crossing-2019-10 | crossing-10 | [… 10.jpg](https://commons.wikimedia.org/wiki/File:201907_Crossing_the_Heilong_(Amur)_from_Heihe_to_Blagoveshchensk_10.jpg) | Jonashtand | CC BY-SA 4.0 | 2019-07-07 11:54, overcast | Mid-river, looking along the Amur: water, riprap, port cranes. |
| port-2019 | port-01 | [201907 Heihe Port 01.jpg](https://commons.wikimedia.org/wiki/File:201907_Heihe_Port_01.jpg) | Jonashtand | CC BY-SA 4.0 | 2019-07-07 10:01, overcast | Car park outside the Heihe port building: dry asphalt, granite-clad wall. |
| station-square-2019 | station-view-2019 | [201907 View from Heihe Railway Station.jpg](https://commons.wikimedia.org/wiki/File:201907_View_from_Heihe_Railway_Station.jpg) | Jonashtand | CC BY-SA 4.0 | 2019-07-20 20:06, overcast evening | Station square: grey slab paving, trees, the Sabsu statue. |
| island-shore-2023 | zhang-2023 | [Bulagoveshchensk in Heihe.jpg](https://commons.wikimedia.org/wiki/File:Bulagoveshchensk_in_Heihe.jpg) | Steve Zhang | CC BY-SA 4.0 | 2023-07-30 14:19, sun | At the waterline on Big Heihe Island, looking at Blagoveshchensk 750 m away: wavelets, the water's own colour in the wave faces and shallows, wet gravel. |
| goods-street | goods-street | [黑河市俄罗斯商品街.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E6%B2%B3%E5%B8%82%E4%BF%84%E7%BD%97%E6%96%AF%E5%95%86%E5%93%81%E8%A1%97.jpg) | HCCB3947 | CC BY-SA 4.0 | 2024-09-07, overcast | Already in `look.json`. Worn dry asphalt, street trees still in full leaf. |
| friendship-2024 | friendship-palace | [黑河市友谊宫兼老年大学.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E6%B2%B3%E5%B8%82%E5%8F%8B%E8%B0%8A%E5%AE%AB%E5%85%BC%E8%80%81%E5%B9%B4%E5%A4%A7%E5%AD%A6.jpg) | HCCB3947 | CC BY-SA 4.0 | 2024-09-07, sun | Friendship Palace forecourt: asphalt in sun and in tree shade. |
| chamber-2024 | chamber | [黑河市商会旧址.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E6%B2%B3%E5%B8%82%E5%95%86%E4%BC%9A%E6%97%A7%E5%9D%80.jpg) | HCCB3947 | CC BY-SA 4.0 | 2024-09-07, sun | Former Chamber of Commerce: a large granite slab plaza in sun. |
| bridge-park-2024 | bridge-park | [Heilongjiang bridge Park.jpg](https://commons.wikimedia.org/wiki/File:Heilongjiang_bridge_Park.jpg) | HCCB3947 | CC BY 4.0 | 2024-09-07, sun | Bridge park: mown lawn turning, tall grass, red surfaced path, sightseeing tower. |
| flag-square-2023 | venti-flag-2023-10 | [黑龙江省黑河市江畔的五星红旗.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E9%BE%99%E6%B1%9F%E7%9C%81%E9%BB%91%E6%B2%B3%E5%B8%82%E6%B1%9F%E7%95%94%E7%9A%84%E4%BA%94%E6%98%9F%E7%BA%A2%E6%97%97.jpg) | VentiHEK | CC BY-SA 4.0 | 2023-10-22 11:58, low sun | Riverside flag square: lawn with yellow tips, a dug-over bed, worn earth, pale paving. |
| station-oct-2024 | venti-station-2024-10 | [正在建设的黑河站新站房（2024.10.13）.jpg](https://commons.wikimedia.org/wiki/File:%E6%AD%A3%E5%9C%A8%E5%BB%BA%E8%AE%BE%E7%9A%84%E9%BB%91%E6%B2%B3%E7%AB%99%E6%96%B0%E7%AB%99%E6%88%BF%EF%BC%882024.10.13%EF%BC%89.jpg) | VentiHEK | CC BY-SA 4.0 | 2024-10-13 11:50, hazy sun | Station square under construction: street trees in autumn colour, spruce, asphalt with fallen leaves. |
| memorial | memorial | [黑河旅俄华侨纪念馆街景 2025-10-12.jpg](https://commons.wikimedia.org/wiki/File:%E9%BB%91%E6%B2%B3%E6%97%85%E4%BF%84%E5%8D%8E%E4%BE%A8%E7%BA%AA%E5%BF%B5%E9%A6%86%E8%A1%97%E6%99%AF_2025-10-12.jpg) | Hehua | **CC BY 4.0** on Commons (`look.json` says CC BY-SA 4.0) | 2025-10-12 15:08, street in shade | Already in `look.json`. Half-bare street trees, dead leaves in the tree pits. |
| april-street-2007 | geoalex-2007-04 | [Heihe.jpg](https://commons.wikimedia.org/wiki/File:Heihe.jpg) | Geoalex | Public domain (PD-self) | 2007-04-11 07:12, low morning sun | Heihe in April after the melt: no snow, no leaves, a dusty pale road, dead hedges and verges. |
| may-street-2013 | darlin-heihe-b | [Heihe, China (8799033151).jpg](https://commons.wikimedia.org/wiki/File:Heihe,_China_(8799033151).jpg) | Sharon Hahn Darlin (Flickr) | CC BY 2.0 | 2013-05-21 (camera clock 20:26, not local time), sun | State Grid tower. In front, rows of young street trees in their first leaves. Sky and road are overexposed. |
| may-port-2020 | oleg-2020-05 | [River Amur-Heihe.jpg](https://commons.wikimedia.org/wiki/File:River_Amur-Heihe.jpg) | Oleg fok | CC BY-SA 4.0 | 2020-05-26 13:00, sun | Heihe cargo port across the water: cranes, warehouse roofs, fresh willow scrub on the near bank. |
| from-blag-2011 | lx-2011-104 | [Благовещенск-2011 (104).JPG](https://commons.wikimedia.org/wiki/File:%D0%91%D0%BB%D0%B0%D0%B3%D0%BE%D0%B2%D0%B5%D1%89%D0%B5%D0%BD%D1%81%D0%BA-2011_(104).JPG) | LxAndrew | CC BY-SA 4.0 | 2011-08-09 10:06, hazy sun | Heihe's riverfront seen from the Blagoveshchensk side across wide, turbid water. |
| blag-elm-2020 | chursaev-a | [Nature takes its toll. Amur region. Blagoveshchensk 09.2021(…).jpg](https://commons.wikimedia.org/wiki/File:Nature_takes_its_toll._Amur_region._Blagoveshchensk_09.2021(%D0%9F%D1%80%D0%B8%D1%80%D0%BE%D0%B4%D0%B0_%D0%B1%D0%B5%D1%80%D0%B5%D1%82_%D1%81%D0%B2%D0%BE%D0%B5._%D0%90%D0%BC%D1%83%D1%80%D1%81%D0%BA%D0%B0%D1%8F_%D0%BE%D0%B1%D0%BB%D0%B0%D1%81%D1%82%D1%8C._%D0%91%D0%BB%D0%B0%D0%B3%D0%BE%D0%B2%D0%B5%D1%89%D0%B5%D0%BD%D1%81%D0%BA_09.2021).jpg) | Chursaev13 | CC BY 4.0 | EXIF 2020-08-13 18:53 (title says 09.2021) | A Blagoveshchensk street elm that has grown round a railing: furrowed bark, bare earth in the tree pit. |
| freeze-up-2023 | bukharov-2023-11-2 | [Views of Blagoveshchensk (November 2023) - 2.jpg](https://commons.wikimedia.org/wiki/File:Views_of_Blagoveshchensk_(November_2023)_-_2.jpg) | Vyacheslav Bukharov | CC BY-SA 4.0 | 2023-11-29 09:22, low sun | The Amur at freeze-up from the Blagoveshchensk embankment: jammed floes, refrozen leads, young ice, snow on the shore ice. |
| bridge-freeze-2019 | mintrans-2019-11 | [Bridge Blagoveshchensk - Heihe in winter.jpg](https://commons.wikimedia.org/wiki/File:Bridge_Blagoveshchensk_-_Heihe_in_winter.jpg) | Ministry of the Russian Federation for the Development of the Far East | Attribution (attribution-only licence, reviewed on Commons) | 2019-11-29 14:56, low sun | The Heilongjiang road bridge at freeze-up: floes and open water between the piers. |
| *(check only)* ice-check-2015 | baycrest-2015 | [Amur River and Heihe.jpg](https://commons.wikimedia.org/wiki/File:Amur_River_and_Heihe.jpg) | Baycrest | CC BY-SA 2.5 | 2015, winter | Snow-covered river ice in front of Heihe, seen from Blagoveshchensk. Used only to cross-check `look.json`'s `river_ice`. |

---

## 2. Colours

Regions are x0, y0, x1, y1 as fractions of width and height. "Lit" is the median of the brighter half of the region; it is given only for foliage, bark and floes.

### Ground: grass, soil, verges

| name | hex | photo | region | note |
|---|---|---|---|---|
| grass_summer | `#3A5621` | verge-2021 | 0,0.785,0.19,0.815: mown verge under the fence, left | July, bright haze. Use this as the base summer grass. |
| grass_riverbank_summer | `#31400C` | riverbank-2017-2 | 0.55,0.958,0.98,0.99: lawn below the riverside path | August morning sun. Longer grass, darker and yellower than the verge. |
| meadow_summer | `#324229` | verge-2021 | 0,0.63,0.16,0.70: unmown meadow behind the fence | Rough ground, waste lots. |
| grass_late_summer | `#6F7949` | bridge-park-2024 | 0.49,0.545,0.60,0.575: mown lawn behind the park sign | 7 September, sun. Already yellowing. |
| grass_autumn | `#5B6743` | flag-square-2023 | 0.73,0.80,0.97,0.88: sunlit lawn right of the pole's shadow | 22 October. Still green at the base, yellow at the tips. |
| grass_autumn_straw | `#898A68` | flag-square-2023 | 0.02,0.73,0.22,0.765: far lawn strip, left | The fully yellowed grass. |
| verge_dormant_spring | `#5A473E` | april-street-2007 | 0.45,0.655,0.95,0.675: dead verge and shrubs under the bank building | 11 April. Snow gone, nothing green yet. |
| hedge_dormant_spring | `#43322A` | april-street-2007 | 0.05,0.80,0.55,0.95: leafless clipped hedge in the median | Lit #5C4B41. |
| soil_dust | `#837460` | riverbank-2017-2 | 0.02,0.605,0.25,0.66: dry bare ground left of the new apron | Bare earth on building sites and unpaved yards in summer. |
| soil_fresh | `#645942` | riverbank-2017-2 | 0.56,0.635,0.84,0.655: freshly dumped earth mounds | Earth just dug or dumped, still holding some moisture. |
| soil_dug_autumn | `#8F8774` | flag-square-2023 | 0.20,0.745,0.44,0.82: dug-over bed | October, dry, low sun. |
| soil_worn_autumn | `#9D9681` | flag-square-2023 | 0.02,0.80,0.14,0.85: trodden earth path, left | Packed and dusty. |
| soil_tree_pit | `#675443` | blag-elm-2020 | 0.02,0.72,0.22,0.95: bare earth round a street tree | Blagoveshchensk, August. Some shade from the tree. |
| leaf_litter_autumn | `#635426` | memorial | 0.37,0.885,0.54,0.97: dead leaves in the tree pit | 12 October, street in shade. Lit #806F43. |

### Ground: roads and paving

| name | hex | photo | region | note |
|---|---|---|---|---|
| asphalt_summer_sun | `#727272` | friendship-2024 | 0.30,0.79,0.48,0.84: forecourt in sun, before the tree shadows | September. Dry. |
| asphalt_summer_shade | `#303135` | friendship-2024 | 0.02,0.84,0.25,0.95: the same surface in tree shade | |
| asphalt_summer_overcast | `#767880` | goods-street | 0.08,0.84,0.62,0.97: worn road in the foreground | September, overcast. An old, pale road with cracks. |
| asphalt_summer_overcast_dark | `#5A5652` | port-2019 | 0.05,0.86,0.95,0.99: port car park | July, overcast, dry. A newer, darker surface. |
| road_dust_spring | `#A3958C` | april-street-2007 | 0.35,0.72,0.95,0.84: wide road | 11 April, 07:12 low sun. A pale road with the winter's grit still on it. Warm light. |
| granite_paving_sun | `#A1A2A2` | chamber-2024 | 0.05,0.74,0.95,0.97: granite slab plaza | September sun. |
| granite_paving_overcast | `#8B8789` | station-square-2019 | 0.05,0.78,0.95,0.98: station square slabs | July, 20:06, overcast. |
| promenade_paving_grey | `#81817B` | promenade-2010 | 0.03,0.84,0.20,0.97: grey pavers | Riverside promenade, August, 16:24 sun. |
| promenade_paving_red | `#947979` | promenade-2010 | 0.43,0.81,0.55,0.835: red pavers in the pattern | Same light. |
| path_red_rubber | `#A35E54` | bridge-park-2024 | 0.05,0.82,0.95,0.97: red-surfaced park path | September sun. A coloured surface, not brick. |
| quay_concrete | `#998E7F` | quay-2019 | 0.05,0.78,0.95,0.97: passenger port apron | July, overcast. |

**Granite and brick against the winter values.** `look.json`'s `granite_paving` (`#908E8A`, from `central-1`) was already read from an August photo, in late-afternoon shade. The summer readings bracket it: `#A1A2A2` in sun and `#8B8789` under overcast. The stone is the same, only the light differs, so I would keep `granite_paving` as the material colour. For red brick pavement I found no clean daylight reading in summer. The memorial photo's pavement is in shade under a blue sky (`#625B6C`, too purple to use). The promenade's red pavers (`#947979`) are concrete pavers, not the brick of `pavement_brick`. `pavement_brick` (`#A35340`) comes from a January night photo under lamps. A daylight reading of the same brick is still missing.

### Ground: the river and its banks

| name | hex | photo | region | note |
|---|---|---|---|---|
| river_water_overcast | `#76797B` | crossing-2019-07 | 0.10,0.80,0.90,0.97: near water from the boat | July, grey sky. crossing-2019-10 gives `#77787A` and crossing-2019-04 gives `#818284`. Neutral grey. |
| river_water_clear | `#4D6D8F` | riverbank-2017-1 | 0.40,0.72,0.95,0.80: open water between the bars | August, clear sky. The sky's reflection. |
| river_water_clear_deep | `#274370` | boat-blag-2017 | 0.05,0.75,0.95,0.95: mid-river from the boat | August, 16:19, clear. Deeper blue at a low viewing angle. |
| river_water_hazy | `#646573` | from-blag-2011 | 0.03,0.74,0.68,0.97: wide water towards Heihe | August 2011, hazy sun. Looks brown in the photo but measures grey-mauve. |
| river_water_body | `#504636` | island-shore-2023 | 0.05,0.85,0.45,0.868: the lifted face of a wavelet | The water's own silty olive-brown, where light passes through rather than reflects. |
| river_shallows | `#514945` | island-shore-2023 | 0.30,0.89,0.60,0.92: a few cm of water over the gravel | The waterline. |
| river_gravel_bar | `#7E766A` | riverbank-2017-1 | 0.70,0.64,0.97,0.69: exposed bar, sun | August low water. |
| river_gravel_spit | `#7D7360` | riverbank-2017-1 | 0.02,0.83,0.15,0.88: near spit | Slightly browner. |
| river_gravel_strip | `#827966` | riverbank-2017-2 | 0.05,0.765,0.90,0.80: gravel and sand below the new wall | |
| river_sandbar | `#85775E` | crossing-2019-04 | 0.41,0.615,0.52,0.66: sand bar under the Heihe embankment | July, overcast. |
| embankment_top_new | `#C6C6C0` | riverbank-2017-2 | 0.25,0.60,0.50,0.635: new concrete apron on top of the embankment | Sun. Close to `look.json`'s `embankment` (`#CFCFCB`). |
| embankment_wall_shade | `#7A786B` | riverbank-2017-2 | 0.05,0.705,0.50,0.74: river face of the new wall | The Heihe wall faces north, so its river face is always in shade. |
| embankment_parapet | `#D8D2C5` | promenade-2010 | 0.02,0.68,0.22,0.76: cream parapet panels, promenade side | Sun. |

### Ground: ice on the river outside midwinter

No photo of the late-April break-up near Heihe was found. The floes below are from the **late-November freeze-up** at Blagoveshchensk, which is the same river at the same place half a year later. They are given so the river is not just "open" or "frozen".

| name | hex | photo | region | note |
|---|---|---|---|---|
| river_floes | `#92979C` | freeze-up-2023 | 0.05,0.55,0.50,0.64: floe field in sun | 29 November, 09:22. White floe tops mixed with grey hollows. Lit #B7BCC0. 12% of the region is blown white. |
| river_floe_gaps | `#6D7B86` | freeze-up-2023 | 0,0.72,0.22,0.79: refrozen leads between the floes | |
| river_young_ice | `#80888F` | freeze-up-2023 | 0.03,0.84,0.17,0.90: smooth grey new ice | |
| river_floes_low_sun | `#AB9B97` | bridge-freeze-2019 | 0,0.74,0.20,0.80: floes under the bridge | 29 November, 14:56. Pink from the low afternoon sun. |
| river_open_water_freezeup | `#878083` | bridge-freeze-2019 | 0.02,0.827,0.30,0.862: open lane among the floes | Reflects the pale sky. |

### Trees

| name | hex | photo | region | note |
|---|---|---|---|---|
| foliage_summer | `#6C8049` | wangsu-2017 | 0.585,0.30,0.665,0.42: sunlit broadleaf crown | August morning. Lit #93A970. |
| foliage_summer_overcast | `#2A3C2F` | goods-street | 0.27,0.42,0.335,0.55: street-tree crown, left of the arch | Early September, overcast, still in full leaf. Lit #384B38. |
| foliage_spruce | `#4B5C4A` | wangsu-2017 | 0.70,0.50,0.75,0.62: spruce, part sun | Lit #7D8E7B. Stays the same all year. |
| foliage_spring_first | `#85A041` | may-street-2013 | 0.02,0.775,0.48,0.83: row of young street trees in new leaf | 21 May. The photo is overexposed, though the leaves themselves are not blown (1.5% left out). Treat this as the brightest the colour gets. Lit #A3BC52. |
| foliage_spring | `#677531` | may-port-2020 | 0.74,0.80,0.97,0.97: willow scrub on the near bank | 26 May. Lit #84914D. The photo does not say which bank it was taken from. |
| foliage_autumn | `#574B40` | station-oct-2024 | 0.845,0.475,0.905,0.595: ochre crowns right of the traffic light | 13 October. Lit #675B50. A hazy, partly backlit phone photo. The crowns look golden in it but measure as a muted olive-ochre. |
| branches_bare_spring | `#5B4B40` | april-street-2007 | 0.80,0.53,0.86,0.63: leafless crowns | April, before leaf-out. |
| bark_elm_summer | `#554539` | blag-elm-2020 | 0.33,0.05,0.62,0.30: deeply furrowed trunk | Blagoveshchensk, August. Lit ridges #958071. |
| bark_pine_upper | `#4F4136` | wangsu-tomb-2017 | 0.917,0.15,0.928,0.195: upper trunk of a Scots pine in dappled sun | Lit #5B4A3A. Looks orange-tan against the dark crowns but measures dull brown. |

Summer bark is not much different from the winter `tree_bark` (`#6E5A48`, a night photo). The whitewashed lower trunks (`tree_lime`) are there all year.

### Roofs without snow

| name | hex | photo | region | note |
|---|---|---|---|---|
| roof_metal_red | `#7E2E2C` | mosque-2017 | 0.075,0.338,0.36,0.354: red metal roof edge of the six-storey block behind the mosque | Ordinary 1990s–2000s housing. Late sun. |
| roof_metal_blue | `#5376A6` | mosque-2017 | 0.83,0.365,0.99,0.375: blue metal roof edges of the blocks on the right | Same blocks. Thin strips, so read it as approximate. |
| roof_port_light | `#DEDDD3` | may-port-2020 | 0.05,0.469,0.18,0.481: flat roofs of the low blocks behind Heihe port, across the water | 16% blown and hazy with distance. Low confidence. |
| roof_port_brown | `#7E493B` | may-port-2020 | 0.452,0.533,0.472,0.543: brown pitched roof at the port | Only 244 pixels. |
| far_roof_red | `#7F3A3F` | boat-blag-2017 | 0.775,0.595,0.81,0.603: red metal pitched roof, Blagoveshchensk | 250 pixels. |
| far_roof_grey_metal | `#5A6266` | boat-blag-2017 | 0.945,0.535,0.995,0.545: grey metal roof, Blagoveshchensk | 416 pixels. |

### The far bank in summer

| name | hex | photo | region | note |
|---|---|---|---|---|
| far_bank_trees_summer | `#2E3C30` | riverbank-2017-2 | 0.40,0.475,0.70,0.515: trees along the Blagoveshchensk embankment | August morning, about 750 m away. |
| far_bank_embankment_summer | `#B4B6AF` | boat-blag-2017 | 0.05,0.628,0.44,0.648: the embankment wall | It faces south, so it is in sun. riverbank-2017-2 gives `#ACACA4`. |
| far_bank_revetment | `#898B76` | boat-blag-2017 | 0.05,0.66,0.33,0.672: stepped revetment at the waterline | |

`look.json`'s `far_bank_brick` (`#B46A4C`) and `far_bank_white` come from `mother`, which is itself an August 2017 photo, so they already are summer values. At 1,280 px the brick towers in the other far-bank photos were too small to read cleanly: the boxes mixed brick, windows and sky. I did not replace them.

---

## 3. JSON, in `look.json`'s shape

New names only. The two existing ids (`goods-street`, `memorial`) are reused and not redefined. Merge `meta.photos` into the existing object.

```json
{
  "meta": {
    "photos": {
      "verge-2021": "Heihe Beiya Bus Line 3,Shuchi YTK6721GH,黑N81330.jpg, Wikimedia Commons, Rayista Wang, CC BY-SA 4.0. 13 July 2021, bright haze. A bus under the red Heilongjiang bridge approach: mown verge, meadow behind a wire fence, dry road, stone pitching.",
      "riverbank-2017-1": "黑龙江畔眺望大岛-1-2017.jpg, Wikimedia Commons, Amarespeco, CC BY-SA 4.0. 13 August 2017 morning, clear. The Amur at low water from the Heihe bank: gravel bars, boulders, open water reflecting the sky, Blagoveshchensk across.",
      "riverbank-2017-2": "黑龙江畔眺望大岛-2-2017.jpg, Wikimedia Commons, Amarespeco, CC BY-SA 4.0. 13 August 2017 morning, clear. New riverside works in Heihe: concrete wall and apron, gravel strip, dumped earth, lawn and path; Blagoveshchensk embankment and trees across.",
      "wangsu-2017": "王肃公园——烈士纪念碑 2017.jpg, Wikimedia Commons, Amarespeco, CC BY-SA 4.0. 13 August 2017 morning, sun. Wang Su Park: Scots pine, spruce and broadleaf trees, whitewashed trunks, grey brick paving.",
      "wangsu-tomb-2017": "王肃墓2017.jpg, Wikimedia Commons, Amarespeco, CC BY-SA 4.0. 13 August 2017 morning, sun. Wang Su's tomb with granite posts, hedges and a Scots pine trunk.",
      "boat-blag-2017": "客轮远眺布拉戈维申斯克-1-2017.jpg, Wikimedia Commons, Amarespeco, CC BY-SA 4.0. 12 August 2017 afternoon, clear. From the passenger boat: mid-river water, the Blagoveshchensk embankment wall and stepped revetment, red and grey metal roofs.",
      "mosque-2017": "Heihe Mosque.jpg, Wikimedia Commons, Antolavoasio, CC BY-SA 4.0. 12 August 2017, low sun. The green log mosque; behind it six-storey white-tile housing blocks with red and blue metal roofs.",
      "promenade-2010": "Вид на Благовещенск с набережной Хэйхэ.JPG, Wikimedia Commons, 'Rogis95, CC0. 24 August 2010, 16:24, sun. Heihe riverside promenade: cream parapet panels, stone-clad piers, grey and red pavers, a weeping willow.",
      "quay-2019": "201907 Crossing the Heilong (Amur) from Heihe to Blagoveshchensk 01.jpg, Wikimedia Commons, Jonashtand, CC BY-SA 4.0. 7 July 2019, overcast. Concrete apron of the Heihe passenger port.",
      "crossing-2019-04": "201907 Crossing the Heilong (Amur) from Heihe to Blagoveshchensk 04.jpg, Wikimedia Commons, Jonashtand, CC BY-SA 4.0. 7 July 2019, overcast. Leaving the Heihe pontoon: sand bar under the embankment, grey water.",
      "crossing-2019-07": "201907 Crossing the Heilong (Amur) from Heihe to Blagoveshchensk 07.jpg, Wikimedia Commons, Jonashtand, CC BY-SA 4.0. 7 July 2019, overcast. Mid-river water under a grey sky.",
      "port-2019": "201907 Heihe Port 01.jpg, Wikimedia Commons, Jonashtand, CC BY-SA 4.0. 7 July 2019, overcast. Dry asphalt car park outside the Heihe port building.",
      "station-square-2019": "201907 View from Heihe Railway Station.jpg, Wikimedia Commons, Jonashtand, CC BY-SA 4.0. 20 July 2019, 20:06, overcast. Station square with grey slab paving.",
      "island-shore-2023": "Bulagoveshchensk in Heihe.jpg, Wikimedia Commons, Steve Zhang, CC BY-SA 4.0. 30 July 2023, sun. Waterline on Big Heihe Island: wavelets showing the silty water's own colour, shallows over gravel.",
      "friendship-2024": "黑河市友谊宫兼老年大学.jpg, Wikimedia Commons, HCCB3947, CC BY-SA 4.0 (Display P3 file, read as sRGB). 7 September 2024, sun. Friendship Palace forecourt, asphalt in sun and in tree shade.",
      "chamber-2024": "黑河市商会旧址.jpg, Wikimedia Commons, HCCB3947, CC BY-SA 4.0 (Display P3 file, read as sRGB). 7 September 2024, sun. Granite slab plaza before the former Chamber of Commerce.",
      "bridge-park-2024": "Heilongjiang bridge Park.jpg, Wikimedia Commons, HCCB3947, CC BY 4.0 (Display P3 file, read as sRGB). 7 September 2024, sun. Mown lawn starting to yellow, tall grass, red surfaced path.",
      "flag-square-2023": "黑龙江省黑河市江畔的五星红旗.jpg, Wikimedia Commons, VentiHEK, CC BY-SA 4.0. 22 October 2023, low sun. Riverside flag square: lawn with yellow tips, dug-over bed, worn earth, pale paving.",
      "station-oct-2024": "正在建设的黑河站新站房（2024.10.13）.jpg, Wikimedia Commons, VentiHEK, CC BY-SA 4.0. 13 October 2024, hazy sun. Station square with street trees in autumn colour.",
      "april-street-2007": "Heihe.jpg, Wikimedia Commons, Geoalex, public domain. 11 April 2007, 07:12. Heihe after the melt: no snow, no leaves, dusty pale road, dead hedges and verges.",
      "may-street-2013": "Heihe, China (8799033151).jpg, Wikimedia Commons (from Flickr), Sharon Hahn Darlin, CC BY 2.0. 21 May 2013, sun, overexposed. Young street trees in their first leaves.",
      "may-port-2020": "River Amur-Heihe.jpg, Wikimedia Commons, Oleg fok, CC BY-SA 4.0. 26 May 2020, sun. Heihe cargo port across the water, warehouse roofs, fresh willow scrub on the near bank.",
      "from-blag-2011": "Благовещенск-2011 (104).JPG, Wikimedia Commons, LxAndrew, CC BY-SA 4.0. 9 August 2011, hazy sun. Heihe's riverfront from the Blagoveshchensk side across wide turbid water.",
      "blag-elm-2020": "Nature takes its toll. Amur region. Blagoveshchensk 09.2021(Природа берет свое. Амурская область. Благовещенск 09.2021).jpg, Wikimedia Commons, Chursaev13, CC BY 4.0. EXIF 13 August 2020. A Blagoveshchensk street elm: furrowed bark, bare earth in the tree pit.",
      "freeze-up-2023": "Views of Blagoveshchensk (November 2023) - 2.jpg, Wikimedia Commons, Vyacheslav Bukharov, CC BY-SA 4.0. 29 November 2023, low sun. The Amur at freeze-up: jammed floes, refrozen leads, young ice.",
      "bridge-freeze-2019": "Bridge Blagoveshchensk - Heihe in winter.jpg, Wikimedia Commons, Ministry of the Russian Federation for the Development of the Far East, attribution-only licence. 29 November 2019, low sun. Floes and open water between the bridge piers."
    }
  },
  "ground": {
    "grass_summer": {"hex": "#3A5621", "from": "verge-2021"},
    "grass_riverbank_summer": {"hex": "#31400C", "from": "riverbank-2017-2"},
    "meadow_summer": {"hex": "#324229", "from": "verge-2021"},
    "grass_late_summer": {"hex": "#6F7949", "from": "bridge-park-2024"},
    "grass_autumn": {"hex": "#5B6743", "from": "flag-square-2023"},
    "grass_autumn_straw": {"hex": "#898A68", "from": "flag-square-2023"},
    "verge_dormant_spring": {"hex": "#5A473E", "from": "april-street-2007"},
    "hedge_dormant_spring": {"hex": "#43322A", "from": "april-street-2007"},
    "soil_dust": {"hex": "#837460", "from": "riverbank-2017-2"},
    "soil_fresh": {"hex": "#645942", "from": "riverbank-2017-2"},
    "soil_dug_autumn": {"hex": "#8F8774", "from": "flag-square-2023"},
    "soil_worn_autumn": {"hex": "#9D9681", "from": "flag-square-2023"},
    "soil_tree_pit": {"hex": "#675443", "from": "blag-elm-2020"},
    "leaf_litter_autumn": {"hex": "#635426", "from": "memorial"},
    "asphalt_summer_sun": {"hex": "#727272", "from": "friendship-2024"},
    "asphalt_summer_shade": {"hex": "#303135", "from": "friendship-2024"},
    "asphalt_summer_overcast": {"hex": "#767880", "from": "goods-street"},
    "asphalt_summer_overcast_dark": {"hex": "#5A5652", "from": "port-2019"},
    "road_dust_spring": {"hex": "#A3958C", "from": "april-street-2007"},
    "granite_paving_sun": {"hex": "#A1A2A2", "from": "chamber-2024"},
    "granite_paving_overcast": {"hex": "#8B8789", "from": "station-square-2019"},
    "promenade_paving_grey": {"hex": "#81817B", "from": "promenade-2010"},
    "promenade_paving_red": {"hex": "#947979", "from": "promenade-2010"},
    "path_red_rubber": {"hex": "#A35E54", "from": "bridge-park-2024"},
    "quay_concrete": {"hex": "#998E7F", "from": "quay-2019"},
    "river_water_overcast": {"hex": "#76797B", "from": "crossing-2019-07"},
    "river_water_clear": {"hex": "#4D6D8F", "from": "riverbank-2017-1"},
    "river_water_clear_deep": {"hex": "#274370", "from": "boat-blag-2017"},
    "river_water_hazy": {"hex": "#646573", "from": "from-blag-2011"},
    "river_water_body": {"hex": "#504636", "from": "island-shore-2023"},
    "river_shallows": {"hex": "#514945", "from": "island-shore-2023"},
    "river_gravel_bar": {"hex": "#7E766A", "from": "riverbank-2017-1"},
    "river_gravel_spit": {"hex": "#7D7360", "from": "riverbank-2017-1"},
    "river_gravel_strip": {"hex": "#827966", "from": "riverbank-2017-2"},
    "river_sandbar": {"hex": "#85775E", "from": "crossing-2019-04"},
    "embankment_top_new": {"hex": "#C6C6C0", "from": "riverbank-2017-2"},
    "embankment_wall_shade": {"hex": "#7A786B", "from": "riverbank-2017-2"},
    "embankment_parapet": {"hex": "#D8D2C5", "from": "promenade-2010"},
    "river_floes": {"hex": "#92979C", "from": "freeze-up-2023"},
    "river_floe_gaps": {"hex": "#6D7B86", "from": "freeze-up-2023"},
    "river_young_ice": {"hex": "#80888F", "from": "freeze-up-2023"},
    "river_floes_low_sun": {"hex": "#AB9B97", "from": "bridge-freeze-2019"},
    "river_open_water_freezeup": {"hex": "#878083", "from": "bridge-freeze-2019"}
  },
  "roofs": {
    "roof_metal_red": {"hex": "#7E2E2C", "from": "mosque-2017"},
    "roof_metal_blue": {"hex": "#5376A6", "from": "mosque-2017"},
    "roof_port_light": {"hex": "#DEDDD3", "from": "may-port-2020"},
    "roof_port_brown": {"hex": "#7E493B", "from": "may-port-2020"},
    "far_roof_red": {"hex": "#7F3A3F", "from": "boat-blag-2017"},
    "far_roof_grey_metal": {"hex": "#5A6266", "from": "boat-blag-2017"}
  },
  "props": {
    "foliage_summer": {"hex": "#6C8049", "from": "wangsu-2017"},
    "foliage_summer_overcast": {"hex": "#2A3C2F", "from": "goods-street"},
    "foliage_spruce": {"hex": "#4B5C4A", "from": "wangsu-2017"},
    "foliage_spring_first": {"hex": "#85A041", "from": "may-street-2013"},
    "foliage_spring": {"hex": "#677531", "from": "may-port-2020"},
    "foliage_autumn": {"hex": "#574B40", "from": "station-oct-2024"},
    "branches_bare_spring": {"hex": "#5B4B40", "from": "april-street-2007"},
    "bark_elm_summer": {"hex": "#554539", "from": "blag-elm-2020"},
    "bark_pine_upper": {"hex": "#4F4136", "from": "wangsu-tomb-2017"},
    "far_bank_trees_summer": {"hex": "#2E3C30", "from": "riverbank-2017-2"},
    "far_bank_embankment_summer": {"hex": "#B4B6AF", "from": "boat-blag-2017"},
    "far_bank_revetment": {"hex": "#898B76", "from": "boat-blag-2017"}
  }
}
```

---

## 4. Not found, and things to check

**Not found.** No photo under an allowed licence showed any of these:

- **Asphalt in rain.** Every summer street photo found is dry.
- **A flat bitumen roof** (`roof_flat_bitumen`), or the roofs of the Central Street blocks seen from above. No aerial or high-angle summer photo of Heihe turned up on Commons or Openverse. The roofs above come from the mosque photo, looking up at ordinary housing, and from the port seen across the water.
- **Ice floes in the late-April break-up** near Heihe. Commons has Amur ice-drift photos (Andshel, 2013–2014), but they carry no location and cannot be tied to Heihe. The freeze-up floes in section 2 stand in for them.
- **Thaw slush and dirty roadside snow in March and April.** The only spring photo with ground in it is 11 April 2007, and by then the snow had gone: dusty road, dead verges. One March 2020 snow-and-puddle photo exists ("Весенняя пора.jpg"), but its coordinates put it about 170 km north of Blagoveshchensk, in forest, so I did not use it. The existing `snow_dirty` (January kerb) remains the only dirty-snow value.
- **Spring grass and leaf-out on the ground in early May.** The May 2013 street photos have it, but they are overexposed. Their lawns read `#B6C8AF` with the paving blown out, so they were not used for grass.
- **Birch and poplar bark** specifically. Only elm (Blagoveshchensk) and the upper trunk of a Scots pine were measured.
- **Red brick pavement in clean daylight.** See the note under roads and paving.

**Things to check in `look.json` itself.** I did not change any of these. They are notes for whoever merges.

- **`river_ice` and `river_ice_dark` are attributed to `mother`, but that photo was taken on 13 August 2017.** There is no ice in it. At the size I downloaded, the river strip is not even visible behind the far embankment. The values may have been read off the Blagoveshchensk embankment's pale bands instead. A winter photo gives snow-covered ice at Heihe of `#AAB0BF` (near) and `#BCC2D5` (mid). That is from ice-check-2015, at regions 0.05,0.85,0.95,0.97 and 0.05,0.74,0.95,0.80. It is close to `#C4D3DE` but less blue.
- **Several of `look.json`'s sources are summer photos.** `central-1`, `central-2`, `pushkin` and `mother` are 12–13 August 2017. `goods-street` is 7 September 2024. `river-wheel` is 20–21 May 2013. So the walls, the granite and the far bank are already summer readings, and they work for both seasons.
- **`memorial` is CC BY 4.0 on its Commons page.** `look.json` lists it as CC BY-SA 4.0.
- **`goods-street` is a Display P3 file.** If its colours were read without converting to sRGB, saturated colours are a few units off. For example, the teal arch reads `#283E4E` raw and `#223F50` in sRGB. Neutral colours barely move.

**Things I am unsure of.**

- `foliage_autumn` comes from one hazy, partly backlit phone photo. It is the weakest value here.
- `foliage_spring_first` comes from an overexposed photo. Treat it as an upper bound on lightness.
- The roof readings are thin strips or distant buildings, with few pixels. `roof_port_light` is the least reliable.
- The far-bank and Blagoveshchensk values (`blag-elm-2020`, `freeze-up-2023`, `bridge-freeze-2019`, the far roofs) are across the river from Heihe, not in it. That follows the rule in the brief: Blagoveshchensk where Heihe has nothing.
- The licence of `bridge-freeze-2019` is shown on Commons only as "Attribution" (an attribution-only government licence, reviewed there). Check that this fits the project's licence policy before shipping.
