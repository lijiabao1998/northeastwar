# Living through a Heihe winter: weather, cold flats, fuel, the body, shops

Research for 東北：俄羅斯入侵 (long-term survival). Compiled 2026-11-25. This file is the source for the cold, fuel, water, food and illness rules. It sits next to `occupation.md`, which covers what the occupation itself does.

## How to read this file

Every number has one tag:

- **R**: the source gives it.
- **D**: derived here. The working is shown.
- **P**: no source found. A suggestion for play.

A **†** after a link means the page was not read in full. It was seen only as a search snippet, an abstract, or a secondary page that quotes the original. Pages without † were opened and read through a fetch tool, which returns a summary of the page, so the wording here is paraphrase. Check exact figures against the original before they become hard rules.

Already sourced in [`occupation.md`](occupation.md), section 4. Not redone here:

- water need, 15 L a day, 2.5–3 L of it drinking (Sphere);
- food energy, 2,100 kcal plus 100 kcal for every 5 °C below 20 °C (WFP/UNHCR);
- the 18 °C healthy indoor minimum (WHO);
- energy to melt and boil snow, about 0.75 MJ per litre;
- one occupied Mariupol flat at 3 °C (Meduza, December 2022).

Day of year assumes a non-leap year (2031 is not a leap year). 17 January is day 17.

---

## 1. Heihe climate

### 1a. Monthly normals, Heihe (Aihui) station, 1991–2020

Station 50468, 50.25° N, 127.45° E, 166 m. Normals 1991–2020; record lows 1971–2010. Source: China Meteorological Administration, as tabulated on [Wikipedia, "Heihe", climate box](https://en.wikipedia.org/wiki/Heihe)†. The CMA's own pages (data.cma.cn) need a login and were not read. Tag R for every cell.

| Month | Mean daily max °C | Mean daily min °C | Mean °C | Record low °C | Precip. mm | Precip. days | Snow days |
|---|---|---|---|---|---|---|---|
| Jan | −15.6 | −26.2 | −21.6 | −44.5 | 5.0 | 7.4 | 10.4 |
| Feb | −9.9 | −22.6 | −16.9 | −40.9 | 5.1 | 4.6 | 7.3 |
| Mar | −0.8 | −13.3 | −7.2 | −32.8 | 7.0 | 4.9 | 7.4 |
| Apr | 10.7 | −1.8 | 4.5 | −19.1 | 23.2 | 6.6 | 4.6 |
| May | 19.4 | 6.0 | 12.8 | −8.4 | 55.8 | 10.9 | 0.4 |
| Jun | 24.9 | 12.7 | 18.9 | 1.7 | 87.5 | 12.8 | 0 |
| Jul | 27.1 | 16.6 | 21.6 | 7.9 | 147.0 | 14.3 | 0 |
| Aug | 24.9 | 14.3 | 19.2 | 4.0 | 112.9 | 14.1 | 0 |
| Sep | 18.9 | 6.8 | 12.4 | −5.3 | 66.4 | 11.0 | 0.2 |
| Oct | 8.8 | −2.4 | 2.8 | −19.1 | 27.6 | 7.7 | 4.9 |
| Nov | −5.2 | −15.3 | −10.7 | −32.8 | 11.1 | 7.3 | 9.7 |
| Dec | −15.3 | −24.8 | −20.6 | −38.9 | 8.1 | 7.9 | 11.2 |
| Year | 7.3 | −4.2 | 1.3 | −44.5 | 556.7 | 109.5 | 56.1 |

### 1b. Blagoveshchensk across the river, for cross-checks and for what Heihe's table lacks

Station 31510. Source: [Погода и климат, «Климат Благовещенска»](http://www.pogodaiklimat.ru/climate/31510.htm) ("Climate of Blagoveshchensk"), compiled from Roshydromet station data. The page does not say which years the normals cover. [Russian Wikipedia](https://ru.wikipedia.org/wiki/Благовещенск)† prints the same temperature and precipitation figures as the 1991–2020 norm, citing the Hydrometcentre of Russia. Tag R.

| Month | Mean max °C | Mean min °C | Mean °C | Abs. min °C (year) | Precip. mm | Days with snow | Snow cover days | Mean snow depth cm | Blizzard days |
|---|---|---|---|---|---|---|---|---|---|
| Jan | −15.1 | −25.6 | −21.0 | −44.5 (1980) | 7 | 12 | 31 | 12 | 3 |
| Feb | −9.4 | −21.8 | −16.1 | −45.4 (1920) | 7 | 7 | 26 | 10 | 1 |
| Mar | −0.2 | −12.2 | −6.4 | −35.7 (1922) | 10 | 8 | 14 | 3 | 2 |
| Apr | 11.2 | −1.1 | 4.9 | −17.7 (1988) | 25 | 6 | 2 | 0 | 0 |
| May | 19.9 | 6.9 | 13.2 | −7.5 (1910) | 55 | 1 | 0.1 | 0 | 0 |
| Jun | 25.5 | 13.8 | 19.4 | 0.1 (1922) | 91 | 0 | 0 | 0 | 0 |
| Jul | 27.7 | 17.5 | 22.2 | 8.2 (1914) | 141 | 0 | 0 | 0 | 0 |
| Aug | 25.4 | 15.4 | 19.9 | 4.4 (1949) | 112 | 0 | 0 | 0 | 0 |
| Sep | 19.4 | 7.9 | 13.0 | −4.3 (1964) | 68 | 0.2 | 0 | 0 | 0 |
| Oct | 9.3 | −1.1 | 3.5 | −24.8 (1902) | 30 | 5 | 4 | 1 | 1 |
| Nov | −4.6 | −13.9 | −9.8 | −32.9 (1944) | 13 | 12 | 22 | 4 | 2 |
| Dec | −14.7 | −23.9 | −19.8 | −41.2 (1912) | 11 | 14 | 30 | 8 | 3 |
| Year | 7.9 | −3.2 | 1.9 | −45.4 | 569 | 65 | 128 | — | 12 |

Blagoveshchensk wind blows mostly from the west and north-west in winter: in January, W 28% and NW 30% of observations, calm 15% (same page).

### 1c. Other quantities

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Thermal zone | 严寒A区 (zone 1A, severe cold A) | R | [GB 50176-2016 民用建筑热工设计规范](https://zh.wikisource.org/wiki/GB_50176-2016_%E6%B0%91%E7%94%A8%E5%BB%BA%E7%AD%91%E7%83%AD%E5%B7%A5%E8%AE%BE%E8%AE%A1%E8%A7%84%E8%8C%83) ("Thermal design code for civil buildings"), Table A.0.1, Heihe row |
| Heating degree-days, base 18 °C | 6,310 K·d | R | same |
| Heating design outdoor temperature (采暖室外计算温度) | −28.9 °C | R | same |
| Lowest daily mean over the record (累年最低日平均温度) | −37.3 °C | R | same |
| Heating season | 193 days at a mean of −11.6 °C, mean relative humidity 60% | R | same |
| Coldest-month mean | −21.9 °C | R | same |
| Blagoveshchensk: coldest day, probability 0.98 / 0.92 | −37 / −35 °C | R | [СП 131.13330.2020 «Строительная климатология»](https://meganorm.ru/mega_doc/norm/pravila/0/sp_131_13330_2020_svod_pravil_stroitelnaya_klimatologiya.html) ("Building climatology"), Table 3.1, Благовещенск row. Data period not stated in the part read |
| Blagoveshchensk: coldest 5 days, 0.98 / 0.92 | −34 / −33 °C | R | same |
| Blagoveshchensk: absolute minimum | −45 °C | R | same |
| Blagoveshchensk: mean daily range, coldest month | 10.4 °C | R | same |
| Blagoveshchensk: days with daily mean ≤ 0 °C | 165 days at a mean of −14.7 °C | R | same |
| Blagoveshchensk: precipitation November–March | 47 mm | R | same |
| Blagoveshchensk: prevailing wind December–February | north-west | R | same |
| Blagoveshchensk: mean wind speed, period with daily mean ≤ 8 °C | 2.1 m/s | R | same |
| Blagoveshchensk: highest of the January mean wind speeds by direction | 2.6 m/s | R | same |
| Blagoveshchensk: annual mean wind speed | 2.0 m/s | R | [Russian Wikipedia](https://ru.wikipedia.org/wiki/Благовещенск)† (no period or source given for this figure) |
| Mean wind speed, January sample | 1.8 m/s; calm in 9% of observations; highest single reading 7 m/s | D | 224 three-hourly observations at Blagoveshchensk, 28 January days in 2022–2025, read from [pogodaiklimat.ru weather archive](http://www.pogodaiklimat.ru/weather.php?id=31510&bday=15&fday=21&amonth=1&ayear=2024&bot=2) (one of four weekly pages). Working files: `out/research/living/climate/obs.csv`, `diurnal.mjs` |
| Mean wind speed for each month, Heihe or Blagoveshchensk, from an authoritative source | not found. The pogodaiklimat wind table is empty; CMA monthly wind normals need a login. A commercial site gives Jan 2.2, Apr 3.7, May 3.2 m/s, but the same table prints January as +21.4 °C, so it is not used | — | — |

### 1d. How much January varies from day to day

No published figure was found (the Soviet climate handbook's interdiurnal-variability tables are not online). Derived from 11 Januaries of daily data at Blagoveshchensk, 2016–2026: 341 days of minimum, mean and maximum, and the anomaly against the station's daily norm, read from the [pogodaiklimat.ru monthly monitor pages](http://www.pogodaiklimat.ru/monitor.php?id=31510&month=1&year=2024) (one page per January; change `year=`). The monthly means computed from the transcribed days match the means the pages print, to 0.1 °C, in every year. Working files: `out/research/living/climate/jan-*.csv`, `jan_stats.mjs`, `jan_stats.txt`.

| Quantity | Value | Tag | Working |
|---|---|---|---|
| Mean of the 11 January means | −20.2 °C (norm −21.0; the last decade ran about 0.8 °C warm) | D | |
| Spread of January means between years (SD) | 2.3 °C; warmest −16.3 (2019), coldest −24.0 (2021, 2023) | D | |
| Spread of the daily mean about the daily norm (SD) | 4.7 °C | D | all 341 anomalies |
| Same, with each year's own mean removed | 4.1 °C | D | |
| Day-to-day persistence of the anomaly (lag-1 correlation) | 0.74 | D | |
| Random daily kick that reproduces the two lines above | 2.8 °C SD | D | 4.1 × √(1 − 0.74²) |
| Change in daily mean from one day to the next | SD 3.0 °C; mean absolute 2.4 °C; largest +8.8 and −8.2 °C | D | 330 day pairs |
| Share of days changing by 3 / 5 / 8 °C or more | 35% / 8% / 0.6% | D | |
| Mean daily range (max − min) | 10.2 °C (SD 3.2; 2.7 to 19.4) | D | agrees with СП 131's 10.4 °C (R) |
| Days with minimum ≤ −30 / −35 / −40 °C | 18% / 3.5% / 0.9% of January days | D | 60, 12 and 3 of 341 |
| Days with maximum ≥ −10 / −5 °C | 16% / 0.6% | D | 54 and 2 of 341; no January day reached 0 °C |
| Cold spells (anomaly −5 °C or colder) | 13 spells; mean 2.9 days; longest 15 days | D | the longest: 16–30 Jan 2023, daily means −24 to −35 °C, minimum −41.2 °C on 21 Jan |
| Mild spells (anomaly +5 °C or warmer) | 27 spells; mean 2.6 days; longest 8 days | D | |
| Highest January temperature on record | −0.4 °C (Heihe); +0.2 °C (Blagoveshchensk, 1983) | R | tables 1a and 1b. A true thaw does not happen in January |

### 1e. Hour of the daily minimum and maximum in winter

No published figure was found. Derived from the same 224 three-hourly January observations (28 days, four weeks in 2022–2025) and 56 March observations (one week in 2024), converted to Beijing time (UTC+8). Heihe's solar noon is about 11:40 Beijing time in mid-January; sunrise about 07:25, sunset about 15:55 (D: 127.5° E, 50.25° N, declination −21°).

| Beijing time | Jan mean °C | Jan offset from daily mean | Coldest reading of the day, Jan (of 24 complete days) | Warmest reading of the day, Jan | Mar offset |
|---|---|---|---|---|---|
| 02:00 | −22.1 | −2.2 | 2 | 0 | −3.0 |
| 05:00 | −22.7 | −2.9 | 5 | 0 | −5.4 |
| 08:00 | −22.6 | −2.7 | 10 | 1 | −2.1 |
| 11:00 | −17.0 | +2.9 | 0 | 2 | +3.1 |
| 14:00 | −15.3 | +4.6 | 0 | 20 | +5.1 |
| 17:00 | −18.1 | +1.7 | 0 | 0 | +2.9 |
| 20:00 | −19.9 | −0.1 | 1 | 1 | +0.4 |
| 23:00 | −21.2 | −1.3 | 6 | 0 | −1.0 |

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Hour of the daily minimum, January | around sunrise, 05:00–08:00 Beijing time (15 of 24 days); on a quarter of days the coldest reading is late evening, when a cold front arrives | D | table above |
| Hour of the daily maximum, January | about 14:00 Beijing time (20 of 24 days) | D | table above |
| Hour of the minimum, March | about 05:00 Beijing time | D | one week only |

**How sure.** The Heihe monthly table is the CMA 1991–2020 normal, read through Wikipedia, so it is marked †. Blagoveshchensk, across the river, agrees within about 1 °C in every winter month, which supports it. The day-to-day statistics are derived from one recent decade at Blagoveshchensk, which ran warmer than the 1991–2020 norm; the spread should hold, the level is about 0.8 °C warm. The diurnal table rests on four January weeks, at three-hour resolution, so the true minimum falls somewhere in 05:00–08:00 and the true maximum in 13:00–15:00. Winter wind is light, about 2 m/s from the west and north-west (R and D). Monthly wind speeds for the other months were not found from an authoritative source.

---

## 2. The Amur at Heihe and Blagoveshchensk

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Mean freeze-up date, Blagoveshchensk | 15 November (day 319) | R | [Amur.Life, 7 Dec 2021, «Амурские гидрологи объяснили, почему на Амуре и Зее в черте Благовещенска до сих пор не встал лёд»](https://www.amur.life/news/2021/12/07/amurskie-gidrologi-obyasnili-pochemu-na-amure-i-zee-v-cherte-blagoveschenska-do-sih-por-ne-vstal-led) ("Amur hydrologists explain why the ice has not yet formed"), quoting the deputy head of the Amur regional hydromet centre (Амурский ЦГМС) |
| Latest freeze-up on record, Blagoveshchensk | 18 December 2013 (day 352) | R | same |
| "Normal" freeze-up, Heihe reach (Chinese side) | about 18 November (day 322) | D | 2022: froze 28 Nov, "10 days later than average" ([中国新闻网, 28 Nov 2022, 《黑龙江黑河段封江 较历年平均封江日期晚十天》](https://www.chinanews.com.cn/sh/2022/11-28/9904102.shtml), "Heihe reach freezes ten days later than average"). 2023: froze 24 Nov, "6 days later than normal" ([黑龙江省人民政府, 24 Nov 2023](https://www.hlj.gov.cn/hlj/c107856/202311/c00_31687967.shtml), citing the provincial hydrology centre). Both give 18 Nov |
| Mean freeze-up, 1962–2020, four Heilongjiang stations including Heihe | between 12 and 22 November; Heihe about 8 days later over 59 years | R | [陈霞等, 《黑龙江省主要河流封冻日期变化特征及预测》, 冰川冻土 46(3), 2024](https://search.napstic.cn/literature/periodical/010bcdt202403022)† ("Freeze-up dates of Heilongjiang's main rivers", abstract) |
| Freeze-up criterion (Chinese) | open water less than 20% of river width | R | 中国新闻网 2022, above |
| Mean break-up (start of ice drift), Blagoveshchensk | 26 April (day 116); earliest 18 April (day 108), latest 10 May (day 130) | R | [Amur.Life, 20 Apr 2026, «На Амуре в Благовещенске начался ледоход»](https://www.amur.life/news/2026/04/20/na-amure-v-blagoveschenske-nachalsya-ledohod-poka-neintensivnyy) ("Ice drift has begun at Blagoveshchensk"), same hydromet official |
| Break-up, cross-check 1 | 19 Apr 2023, "7 days earlier than usual", so norm 26 April | D | [Амурский ЦГМС, «Обзор погодных условий в 2023 г.»](https://amurmeteo.ru/news/416-obzor-pogodnyh-uslovij-v-2023-g.html) ("Review of weather in 2023") |
| Break-up, cross-check 2 (Chinese side) | 20 April, "7 days earlier than normal", so norm 27 April | D | [环球网, Heihe reach breaks up](https://china.huanqiu.com/article/4MMn451uLnV) (year not printed; probably 2025) |
| Break-up, cross-check 3 | 18 Apr 2026, "about 9 days earlier than the norm", so 27 April | D | [Tourdom, «Российские туристы застряли в Китае из-за начала ледохода на Амуре»](https://www.tourdom.ru/news/rossiyskie-turisty-zastryali-v-kitae-iz-za-nachala-ledokhoda-na-amure.html) ("Russian tourists stuck in China as ice drift starts") |
| Length of ice cover | about 5 months; 15 Nov to 26 Apr is 162 days | R/D | 中国新闻网 2022; 46 + 116 = 162 |
| Ice thickness, late December | 57–83 cm on the Amur in the oblast, 3rd ten-day period of Dec 2023 | R | Амурский ЦГМС 2023 review, above |
| Ice thickness, late December, Blagoveshchensk | minimum 40 cm, 27 Dec 2024 | R | [Администрация Благовещенска, «На реке Амур провели первые замеры толщины льда»](https://xn--80abedla9acxg1b7f.xn--p1ai/news/na_reke_amur_proveli_pervye_zamery_tolshchiny_lda_/) ("First ice measurements on the Amur") |
| Ice thickness, mid-January | 50 cm (11 Jan 2017); 51 cm (12 Jan 2010, ice-road site); 60 cm (12 Jan 2026) | R/D | [Амурская правда, 10 Jan 2017](https://ampravda.ru/2017/01/10/72145.html); [Телепорт 2001, 13 Jan 2010](https://www.teleport2001.ru/news/2010-01-13/22754-bereg-levyy-bereg-pravyy.html) ("10 cm short of the 61 cm norm", so 51); [Амурская правда, 12 Jan 2026](https://ampravda.ru/2026/01/12/v-blagoveshhenske-spasateli-izmerili-tolshhinu-lda-na-meste-budushhei-ploshhadki-zimfestamura) |
| Ice thickness, late January | minimum 110 cm, 30 Jan 2024 (a thick year) | R | [Amur.Life, 30 Jan 2024](https://www.amur.life/news/2024/01/30/na-amure-v-blagoveschenske-zameryayut-tolschinu-lda-dlya-chego) |
| Ice thickness, February | not found | — | — |
| Ice thickness, late March | 118 cm on 25 Mar 2024, "15 cm above norm", so norm about 103 cm; 86–104 cm on 23 Mar 2026; 81–119 cm at end of March 2023 (8–39 cm below norm) | R/D | [ГТРК «Амур», 25 Mar 2024](https://gtrkamur.ru/news/2024/03/25/368267); [ГТРК «Амур», 23 Mar 2026](https://gtrkamur.ru/news/2026/03/23/510772); Амурский ЦГМС 2023 review |
| Maximum ice, 2026 winter, Heilongjiang main stem and other rivers | 0.83–1.35 m, 0.04–0.37 m thinner than average; 115–149 days frozen | R | [中国水利 via 兵团水利局, 6 May 2026](http://btslj.xjbt.gov.cn/c/2026-05-06/8482822.shtml) |
| Ice growth in March, middle and lower Amur | 0.1–0.2 cm a day; melting starts in April | R | [Махинов А., «Ледяной покров Амура», Природа 2019 № 3](https://elementy.ru/nauchno-populyarnaya_biblioteka/435141/Ledyanoy_pokrov_Amura) ("Ice cover of the Amur") |
| Minimum ice for an ice road, Amur oblast | 61 cm | R | Телепорт 2001, 2010, above |
| Load against ice thickness | 55 cm: 20 t; 110 cm: 30 t (festival site, Blagoveshchensk); 74 cm allowed 30 t (Poyarkovo–Xunke crossing, opened 19 Jan 2016) | R | Администрация Благовещенска, above; [EastRussia, «Ледовая переправа через Амур в Китай откроется 19 января»](https://www.eastrussia.ru/news/ledovaya-pereprava-cherez-amur-v-kitay-otkroetsya-19-yanvarya-/) |
| Heihe floating-box ice crossing | opened 4 Dec 2019; 13 m wide, 650 m long, 10 barges | R | [中国日报, 5 Dec 2019](https://cn.chinadaily.com.cn/a/201912/05/WS5de838a0a31099ab995efad4.html) |

**How sure.** Break-up is firm: the hydromet centre's mean of 26 April and three "days earlier than normal" reports agree within a day. Freeze-up differs by three days between the Russian mean (15 Nov) and the Chinese normal (18 Nov); the definitions or reference years may differ, and freeze-up is getting later. No source says which years these averages cover, except the 1962–2020 paper. Ice thickness comes from single measurements at festival or crossing sites, not a station norm table; years differ two-fold (50 cm in mid-January 2017, 110 cm in late January 2024). The monthly norm table would be in «Ресурсы поверхностных вод СССР», which is not online. Nothing was found for February. The two load figures for 30 t conflict.

---

## 3. How fast a heated flat cools when the heating stops

### 3a. Real heating failures

All indoor figures are residents' reports. Almost all were taken with gas stoves or electric heaters running.

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Alchevsk, Ukraine, Jan 2006: flat temperature | never above +5 °C even with the gas stove and oven on full; outdoor −25 to −30 °C (one claim −40 °C); the town was without heat about two months | R | [«Ледниковый период Украины: как на Донбассе вымерз целый город» (MK, via Rambler, 2016)](https://news.rambler.ru/world/34986938/) ("Ice age in Ukraine: how a whole Donbas town froze") |
| Alchevsk, one flat | −5 °C about a week after the 22 January failure; power off for 3 days; maternity delivery room at most +12 °C | R | [Обозреватель, 30 Jan 2006, «Уже неделю Алчевск похож на вымерзшую обратную сторону Луны»](https://www.obozrevatel.com/news/2006/1/30/84038/amp.htm) ("For a week Alchevsk has looked like the frozen far side of the Moon") |
| Ekibastuz, Kazakhstan, after the 27 Nov 2022 power-plant failure | about +3 °C in the days after; outdoor about −30 °C; one block 6 days without heat | R | [NUR.KZ, «Как возвращали тепло в дома Экибастуза»](https://www.nur.kz/incident/emergency/2000931-kak-vozvraschali-teplo-v-doma-ekibastuza-istoriya-komandirovannogo-spetsialista/) ("How heat was returned to Ekibastuz homes") |
| Ekibastuz, another block (report of Feb 2023) | living rooms not above +15 °C after 10 days without heat; its substation froze; outdoor −30 °C | R | [Informburo, 24 Feb 2023](https://informburo.kz/novosti/rabota-vedyotsya-ne-perezivaite-cto-otvecayut-v-pravitelstve-zamerzayushhim-ekibastuzcam) |
| Klimovsk, Moscow region, Jan 2024 | flats down to about +6 °C by day 3 (7 Jan); +8 to +19 °C reported by day 4; outdoor about −24 °C; about 20,000 people; a brick block stayed warm for the first few days; residents used gas and electric heaters | R | [RBC, 8 Jan 2024](https://www.rbc.ru/rbcfreenews/659bbc049a794736a27b6346); [News.ru, 10 Jan 2024, «С 1965-го не меняли трубы: как живут люди в Климовске без отопления»](https://news.ru/society/s-1965-go-ne-menyali-truby-kak-zhivut-lyudi-v-klimovske-bez-otopleniya) ("Pipes unchanged since 1965: how Klimovsk lives without heating") |
| Kyiv, Jan 2026 | about +4 °C (40 °F) about 14 days after the 9 January strike; nights down to about −20 °C; power intermittent | R | [NBC News, 23 Jan 2026](https://www.nbcnews.com/world/ukraine/kyiv-freezing-dark-russian-strikes-leave-ukraines-capital-powerless-rcna254359) |
| Kyiv, Jan 2026, a flat in a block unheated for 9 days | about +13 °C | R | [UNICEF Ukraine, 19 Jan 2026](https://www.unicef.org/ukraine/en/stories/hot-water-bottles-sleeping-bags-and-camping-gas) |
| Khrushchyovka at −5 °C outdoors | holds at most about 24 hours | R (weak: the article's own claim) | [Обозреватель, 17 Jan 2023](https://news.obozrevatel.com/ukr/economics/communalka/skilki-mozhe-vitrimati-kvartira-bez-opalennya-ekspert-rozpoviv-yak-zberegti-teplo.htm) (Ukrainian; on how long a flat lasts without heating) |

### 3b. Norms and the building time constant

Soviet and Russian practice treats a building as one heat store that cools exponentially:

t(τ) = t_out + (t_start − t_out) · e^(−τ/β)

β is the building's heat-accumulation coefficient in hours, which is its time constant.

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Allowed fall during repair of a heat main | residential buildings to +12 °C, for at most 54 hours | R | [СП 124.13330.2012 «Тепловые сети»](https://files.stroyinf.ru/Data2/1/4293788/4293788792.htm) ("Heat networks"), clause 4.2. The β formula is **not** in this code |
| Repair times allowed, by pipe diameter | 300 mm: 15 h; 500 mm: 22 h; 800–1,000 mm: 40 h; 1,200–1,400 mm: up to 54 h | R | same, clause 6.10, Table 2 |
| Cooling formula in heat-supply schemes | z = β · ln[(t_in − t_out) / (t_limit − t_out)], with t_in 20 °C, t_limit 12 °C, β = 40 h | R | [Схема теплоснабжения г. Зеленогорска, гл. 11](https://www.zeladmin.ru/download_file/force/68294/366) (Zelenogorsk heat-supply scheme, ch. 11 "Reliability"), section 11.3 |
| β by building type and position | panel series 1-605A: top-floor corner flat 42 h, corner on a middle or first floor 46 h, middle 77 h. Panel K7-3 (Khrushchyovka type): 32 / 40 / 51 h. Brick, 2.5-brick (about 640 mm) walls: middle 65–100 h, corner 60–65 h. Industrial: 14–25 h | R | [«Указания по повышению надёжности систем коммунального теплоснабжения»](https://meganorm.ru/mega_doc/norm/vypiska/0/ukazaniya_po_povysheniyu_nadezhnosti_sistem_kommunalnogo.html) ("Guidelines for improving the reliability of municipal heat supply"), approved by Роскоммунэнерго 26 Jun 1989, published by АКХ им. К.Д. Памфилова 1990; clause 60, Table 3. [Copy](https://www.baurum.ru/_library/?cat=enginworks&id=4058). The online text is OCR-garbled in the brick and industrial rows; those ranges should be checked against a scan |
| Cooling rate in the first hours, from 18 °C | β 20 h: 0.8 / 1.4 / 1.8 / 2.4 °C per hour at outdoor 0 / −10 / −20 / −30 °C. β 40 h: 0.5 / 0.8 / 1.1 / 1.5. β 60 h: 0.4 / 0.6 / 0.8 / 1.0 | R | same, Table 2. Worked example in the text: β 40 h at −20 °C, 18 °C to 8 °C in about 9 hours. Clause 60 names 8 °C as the point where basements and stairwells risk freezing. β is best measured in a corner room with the heating off, and the building with the lowest β governs |
| Modern 31-storey Moscow block, by window share of the façade f | β 100 h at f = 0.19, down to 42 h at f = 0.84; air after 16 h at design cold 13.2 °C down to 5.5 °C | R (model) | [Васильев и др., «Расчётная оценка влияния степени остеклённости и климатических условий на остывание здания», Энергосбережение № 8-2025](https://www.abok.ru/for_spec/articles/46/9232/9232.pdf) ("Effect of glazing share and climate on building cooling"), Table 3 |
| Same block in Krasnoyarsk or Vorkuta | after 16 h: about 11 / 7.5 / 3 °C at f = 0.25 / 0.51 / 0.84 | R (model) | same, Table 4 |
| Exponential form, β = heat capacity × resistance | as above; traced to Sokolov's textbook | R | [Васильев и др., «Тепловая инерция зданий…», Энергосбережение № 7-2025](https://www.abok.ru/for_spec/articles/45/9190/9190.pdf) ("Thermal inertia of buildings"), formulas 3–6 |
| Russian consumer rule | limits heating breaks by indoor temperature; the article quotes 12 °C and 16 hours. The exact clause was not checked | R† | Government Resolution 354 (ПП РФ № 354), as quoted in the article above |
| First hours fall faster than the exponential | middle-floor room from +20 °C to +11 °C in 11.1 h at a steady 0 °C outdoors, 6.7 h if outdoors falls 20 K over 12 h; the fall follows exp(−k√τ) early on | R (model) | [Самарин О.Д., «Расчёт остывания помещений здания в аварийных режимах при переменной температуре наружного воздуха», Вестник МГСУ 19(1), 2024](https://journals.rcsi.science/1997-0935/article/download/254470/232802) ("Cooling of rooms in emergencies with changing outdoor temperature") |
| Measured whole-building heat loss, 5-storey 1980s concrete-panel block, Ryazan, 60 flats | UA = 7,739 and 7,056 W/K; average occupant heat 22.3 and 28.4 kW; gas stoves 31.3 kW; electricity 9.9 kW; coldest daily mean −27.5 °C | R (measured) | [Armstrong et al., "Russian Apartment Building Thermal Response Models", ACEEE Summer Study 2000](https://aceee.org/files/proceedings/2000/data/papers/SS00_Panel3_Paper01.pdf), Tables 5–6 |
| Whole-building time constant from that model | about 96 h and 116 h | D (low confidence: the model is a regression, not a cool-down test) | daily decay with no heat input 15.52/19.93 = 0.779, so τ = 24 / −ln 0.779 ≈ 96 h; second block 27.11/33.31 gives ≈ 116 h |
| High-performance condo, Toronto, January (simulated) | held heat more than 2 days longer than a standard unit; above 17 °C on day 1 | R | [O'Brien & Kesik, "Condominiums and Resilience", Canadian Consulting Engineer, Dec 2016](https://carleton.ca/hbilab/wp-content/uploads/Condominiums-and-Resilience-Canadian-Consulting-Engineer.pdf) |
| Time to +12 °C at Heihe, β 40 h | 8.5 h at −22 °C; 7.0 h at −30 °C | D | 40 · ln(42/34) = 8.5; 40 · ln(50/42) = 7.0 |
| Time to 0 °C at Heihe, β 60 h | 39 h at −22 °C; 31 h at −30 °C | D | 60 · ln(42/22) = 39; 60 · ln(50/30) = 31 |

### 3c. Where an unheated, occupied flat settles

Steady state: the indoor excess over outdoors is internal heat ÷ heat-loss conductance.

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Whole block unheated, body heat only | +2.9 K above outdoors | D | Armstrong 2000: 22,330 W ÷ 7,739 W/K |
| Body heat plus normal electricity | +4.2 K | D | (22.3 + 9.9) kW ÷ 7.74 kW/K |
| Body heat, electricity and normal gas cooking | +8.2 K | D | (22.3 + 9.9 + 31.3) kW ÷ 7.74 kW/K. Tenants burned more than twice the usual gas during heat shortages, so stoves used as heaters push this higher |
| Observed with gas stoves or heaters running | +3 to +10 °C indoors at −24 to −30 °C outdoors, i.e. about 30 K above outdoors | R | the failures in 3a |

### 3d. Broken windows

No measured air-change rate through a broken window was found. The figures are derived from a standard formula.

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Flow through one opening, driven by the temperature difference | Q = (1/3) · Cd · A · √(ΔT · g · H / T̄) | R | [Larsen et al., "Calculation methods for single-sided natural ventilation", Energy and Buildings 177 (2018)](https://vbn.aau.dk/ws/files/284948171/Calculation_methods_for_single_sided_natural_ventilation_now_and_ahead.pdf), eq. 4 (after Warren and Parkins 1985). Closing internal doors cuts the flow by 41–72% |
| Whole 1.5 × 1.5 m window gone, ΔT 30 K | about 0.58 m³/s, about 17 air changes an hour in a 125 m³ flat; heat-loss conductance 700–790 W/K, against about 129 W/K for a whole intact flat | D | (1/3) · 0.6 · 2.25 · √(30 · 9.81 · 1.5 / 263) = 0.58 m³/s; 0.58 × 1.2–1.35 kg/m³ × 1,005 J/kg·K; flat UA = 7,739 ÷ 60 |
| Effect on the flat | time constant about 0.15 of intact; settled excess drops to about 0.4 K, i.e. the flat sits at outdoor temperature | D | 129 ÷ (129 + 750) ≈ 0.15 |
| One 0.5 × 0.5 m pane broken | about 1.1 air changes an hour, about 50 W/K; adds about 40% to the flat's loss; time constant × 0.72 | D | same formula, A = 0.25 m² |
| Reported | flats in occupied Mariupol became "wind tunnels" after blast damage; people moved to stairwells and basements | R | `occupation.md` 3.11 (Meduza, 6 Dec 2022) |

### 3e. Top-floor and corner flats

| Quantity | Value | Tag | Source |
|---|---|---|---|
| β, top-floor corner ÷ middle flat | 0.55 (series 1-605A, 42/77); 0.63 (K7-3, 32/51); brick corner 60–65 h against middle 65–100 h | R | АКХ 1989, Table 3, above |
| Gap after 24 h, start +20 °C, outdoor −25 °C | top corner +0.4 °C, middle +7.9 °C: about 7.5 K colder | D | −25 + 45·e^(−24/42) = 0.4; −25 + 45·e^(−24/77) = 7.9 |
| Gap after 48 h | −10.6 against −0.9 °C: about 9.7 K colder | D | same, τ = 48 h |

### 3f. Walls and windows in Heilongjiang

| Quantity | Value | Tag | Source |
|---|---|---|---|
| JGJ 26-2018 limits, zone 1A, buildings of 4 storeys or more | wall 0.35, roof 0.15, window 1.6 W/m²K (1.4 for 3 storeys or fewer; wall 0.25) | R | [JGJ 26-2018 严寒和寒冷地区居住建筑节能设计标准, Table 4.2.1-1, reposted on Sohu](https://www.sohu.com/a/288315193_722349)† ("Energy-efficiency design standard for residential buildings in severe-cold and cold zones"). The repost says it is a partial copy of the draft for comment |
| JGJ 26-2018 calculation conditions | indoor 18 °C; 0.5 air changes an hour | R | [JGJ 26-2018 §4.3.6, reposted](https://www.yuanlin8.com/guifan/nuantong/8760fcd1.html) |
| Heilongjiang's own standard | stricter than JGJ 26-2018 for 1A roofs; zone B treated as zone A | R | [DB23/1270-2019 黑龙江省居住建筑节能设计标准](https://zfcxjst.hlj.gov.cn/zfcxjst/c114948/202001/31215436/files/%E9%BB%91%E9%BE%99%E6%B1%9F%E7%9C%81%E5%B1%85%E4%BD%8F%E5%BB%BA%E7%AD%91%E8%8A%82%E8%83%BD%E8%AE%BE%E8%AE%A1%E6%A0%87%E5%87%86.pdf)† (preface; no numbers read) |
| Clay brick masonry conductivity | 0.81 W/m·K (1,800 kg/m³); cement mortar 0.93; reinforced concrete 1.74 | R | GB 50176-2016, Table B.1 (link in 1c) |
| Surface resistances, winter | inside 0.11, outside 0.04 m²K/W | R | GB 50176-2016, Table B.4.1 |
| Old solid-brick wall with 20 mm plaster | 370 mm: 1.58; 490 mm: 1.28; 620 mm: 1.06 W/m²K | D | 490 mm: R = 0.11 + 0.49/0.81 + 0.02/0.81 + 0.04 = 0.78, U = 1.28 |
| Single-glazed metal window | 6.6 W/m²K | R/D | GB 50176-2016, Table C.5.3-1 (3 mm glass; whole-window values 6.6 / 5.8 / 5.0 by frame). Matching 6.6 to a metal frame without a thermal break is this file's reading |
| Old double-sash window (two panes, about 90 mm gap) | about 2.5 (wood frames) to 3.0 (steel frames) W/m²K | D | air gap R ≈ 0.20 (GB 50176 Table B.3-2); R = 0.15 + 2 × 0.022 + 0.20 = 0.39, U ≈ 2.5 for the glazing |
| JGJ 26-86 and JGJ 26-95 limits | not found | — | — |

**How sure.** The β table and the 40 h practice value are primary data and agree with the 2025 models (42–100 h). The real failures are residents' reports, untimed and nearly always with stoves or heaters on, so they stay warmer than a pure exponential would. Outdoor temperatures for Alchevsk conflict (−20 to −40 °C). Not found: Chinese heating-failure cases, Leningrad or Sarajevo indoor temperatures, a measured air-change rate through a broken pane, and official Kyiv engineering guidance. If a Heihe block has post-2010 external insulation, it sits near the 100 h end; 1980s brick and panel blocks sit at 40–77 h.

---

## 4. Basements in winter

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Design air temperature in a technical basement (техподполье) under a heated block | at least +2 °C at design conditions; assumes heating pipes run through it | R | [СП 23-101-2004 «Проектирование тепловой защиты зданий»](https://meganorm.ru/Data2/1/4294813/4294813064.htm) ("Design of thermal protection of buildings"), §9.3.2 |
| Air change assumed in that basement | 0.5 an hour; 1.0 if gas pipes run there | R | same, §9.3.5 |
| Resistance of floor plus buried walls, by basement width | 4 / 8 / 12 / 16 m: 2.15 / 2.86 / 3.69 / 4.52 m²K/W | R | same, Table 13 |
| Kyiv basement shelter, block unheated 9 days | "barely" +5 °C; flat above about +13 °C; outdoor lows to about −18 °C nationally | R | UNICEF Ukraine, 19 Jan 2026 (link in 3a) |
| Unheated underground garages, Beijing, outdoor −19.6 °C | first level above 0 °C over more than half its area; second level above +5 °C | R | [李雨婷, 《极端低温条件下北京市无供暖地下车库温度的实测分析》, 建筑科学 40(6), 2024](https://search.napstic.cn/literature/periodical/010jzkx202406032)† ("Measured temperatures in unheated Beijing garages in extreme cold", abstract). Whether the buildings above were heated is not stated |
| Ground near Heihe, Bei'an (48.6° N) | maximum frost depth 1.75 m by mid-March 2024; annual mean ground temperature about 6 °C at 2–3 m; January mean −7.6 °C at 0.1 m | R | [Liu et al., Frontiers in Earth Science 13 (2025), doi 10.3389/feart.2025.1566593](https://www.frontiersin.org/journals/earth-science/articles/10.3389/feart.2025.1566593/full) |
| Frost-depth formula | d = d0 · √Mt; d0 = 0.23 m (clay, loam) to 0.34 m (coarse); Mt = sum of negative monthly means | R | [СП 22.13330.2016 «Основания зданий и сооружений»](https://meganorm.ru/Data2/1/4293747/4293747631.htm) ("Foundations of buildings"), §5.5.3 |
| Heihe frost depth, bare ground | 2.0 m (loam) to 3.0 m (coarse) | D | Mt = 21.6 + 16.9 + 7.2 + 10.7 + 20.6 = 77.0; √77 = 8.77; × d0 |
| Basement under a heated block, −25 °C outside | +7.5 °C with vents at 0.5 air changes an hour; +10.5 °C with vents mostly shut (0.2) | D | per metre of building: slab to flats 26.7 W/K at +20 °C; walls above ground 1.54, ground 3.54, air 5.23 W/K to −25 °C; (534 − 258) ÷ 37.0 = 7.5 |
| Basement after weeks with no heat in the block, flats above at +5 / 0 / −5 / −10 °C | +1.7 / −2.3 / −6.2 / −10.1 °C | D (rough) | same model, vents shut, ground path to deep ground at +4 °C (the +4 °C is P). Run against Kyiv inputs it gives +9.6 °C against the +5 °C reported, so it reads about 4 K warm |

**How sure.** No measured basement temperature at −25 °C outside was found. Three data points bracket it: the +2 °C design minimum, Kyiv's +5 °C, and the Beijing garages at 0–5 °C. The design method counts ground loss against outdoor air, which is too pessimistic for the first weeks because the ground stores heat; over months that store runs down. GB 50007's standard frost depth for Heihe was not found.

---

## 5. Fuel and heat

### 5a. Stoves

| Quantity | Value | Tag | Source |
|---|---|---|---|
| GB 16154-2018 民用水暖煤炉通用技术条件 (household water-heating coal stoves) | replaces the 2005 edition; in force 2019-08-01. Output and efficiency limits not read (paywalled) | R | [chinesestandard.net listing](https://www.chinesestandard.net/PDF.aspx/GB16154-2018)† |
| Small cast-iron "burzhuika" stove, TST025L | 4.5 kW, 39 kg, 115 mm flue | R | [Epicentr listing](https://epicentrk.ua/shop/burzhuyka-tst025l.html)† |
| Larger cast-iron stove, Termofor | 13 kW for 200 m³ (wood); a 16 kW model also listed | R | [Onliner catalogue](https://catalog.onliner.by/stove/termofor/term_ob9)† |
| Stove with a stated burn rate | 5.9 kW from 1.9 kg of wood an hour, efficiency over 80% | R | [Spartherm Ambiente A1 manual](https://manuals.online/spartherm/stove/ambiente-a1/user-manual/1xt4n)† |
| Efficiency floor, closed solid-fuel room heaters (EU) | at least 65%; open-front at least 30% | R | [EU Regulation 2015/1185, Annex II](https://www.legislation.gov.uk/eur/2015/1185/annex/II) |
| Default efficiency, certified wood stoves (US) | 63% non-catalytic, 72% catalytic | R | [US EPA certified-stove list](https://19january2017snapshot.epa.gov/sites/production/files/2015-11/documents/pre2015nsps-certifiedwood.pdf)† |
| Share of heat into the room from a simple iron stove | 50% | P | between the 30% and 65% floors above |
| Coal burn rate of a named household coal stove | not found | — | — |

### 5b. Heating values

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Raw coal, Chinese national conversion figure | 20,908 kJ/kg (5,000 kcal/kg); washed coal 26,344; coke 28,435 | R | [coal.in-en conversion table](https://coal.in-en.com/html/coal-2467032.shtml), citing the national energy statistics system; the same figures in a [Beijing NDRC annex](https://fgw.beijing.gov.cn/fgwzwgk/2024zcwj/bwqtwj/202611/W020261103622212733624.pdf)†. GB/T 2589-2020 itself was not opened |
| Standard coal | 29,271 kJ/kg (7,000 kcal/kg) | R | [National Bureau of Statistics Q&A](https://www.stats.gov.cn/hd/lyzx/zxgk/202210/t20221027_1889755.html) |
| Russian coals, net heating value | brown coal 13.7; Chita 14.2; Primorye 14.8; hard coal 22.5; anthracite 26.7 MJ/kg | R | [Приказ Минприроды России № 371 от 27.05.2022, прил. 2, табл. 1.1](https://legalacts.ru/doc/prikaz-minprirody-rossii-ot-27052022-n-371-ob-utverzhdenii/) (Natural Resources Ministry order 371, Annex 2) |
| Blagoveshchensk power station fuel | brown coal from the Erkovtsy mine; its heating value not found | R | [EastRussia, on the Blagoveshchensk power station](https://www.eastrussia.ru/material/dym-nad-amurom-budet-pozhizhe_/) |
| Heilongjiang mines (Hegang, Shuangyashan, Jixi, Qitaihe) | not found | — | — |
| Wood, oven-dry | 19 MJ/kg | R | [Forest Research (UK), "Typical calorific values of fuels"](https://www.forestresearch.gov.uk/tools-and-resources/biomass-energy-resources/reference-biomass/facts-figures/typical-calorific-values-of-fuels/) |
| Wood, air-dry logs at 20% moisture | 14.7 MJ/kg (net) | R | same |
| Wood at 15% moisture | 15.8 MJ/kg | D | 19 · (1 − 0.15) − 2.44 · 0.15 = 15.8 (the same formula gives 14.7 at 20%) |
| Particleboard, melamine-faced, waste | about 16.5 MJ/kg (MDF 17.4, HDF 19.4) | R | [PMC3918744, Table 5](https://pmc.ncbi.nlm.nih.gov/articles/PMC3918744/table/tab5)† |
| Densities | pine or spruce 500, oak 700, plywood 600 kg/m³ | R | GB 50176-2016, Table B.1 |

### 5c. Furniture masses

No residential fire-load survey with weighed items could be read. Masses are derived from dimensions and checked against listings.

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Wooden chair | about 5 kg (pine), 7 kg (oak or birch) | D | about 0.010 m³ of wood × 500–700 kg/m³. Cross-checks: IKEA INGOLF solid-wood chair, package 5.6–8.5 kg ([IKEA](https://www.ikea.com/cz/en/p/ingolf-chair-brown-black-60217822)†); a Belarus fire-load example uses 5 kg chairs ([Polotsk State University paper](https://elib.psu.by/bitstream/123456789/26642/5/46-51.pdf)†) |
| Dining table, 4 seats, solid pine, 120 × 75 cm | about 21 kg | D | top 0.0225 + legs 0.0141 + aprons 0.0051 = 0.042 m³ × 500. The Belarus example uses a 30 kg particleboard desk† |
| Two-door particleboard wardrobe, 120 × 60 × 240 cm | about 93 kg | D | 7.9 m² of 16 mm board × 680 kg/m³ (P) + 7 kg hardboard back. The Belarus example uses about 100 kg† |
| Interior door leaf, 2.0 × 0.8 m | hollow core about 14 kg; solid pine 40 mm about 32 kg | D | hollow: 2 × 1.6 m² × 4 mm × 850 kg/m³ + 3.5 kg frame; solid: 0.064 m³ × 500. Construction types from [ГОСТ 6629-88 «Двери деревянные внутренние»](https://files.stroyinf.ru/Data2/1/4294853/4294853218.htm) ("Interior wooden doors"), which gives no masses |

### 5d. Heat to hold a room

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Heat-load index, housing without energy saving | 58–64 W/m² (includes 5% network loss) | R | CJJ 34 城镇供热管网设计规范 ("Urban heating network design code"), as quoted by [Baidu Baike, 采暖热指标](https://baike.baidu.com/item/%E9%87%87%E6%9A%96%E7%83%AD%E6%8C%87%E6%A0%87/10291905)†. CJJ 34-2010 was replaced by CJJ/T 34-2022 ([listing](https://codeofchina.com/standard/CJJT34-2022.html)); its table was not read |
| Heat-load index, housing (design handbook) | 47–70 W/m² (40–60 kcal/m²·h) | R | 民用建筑采暖通风设计技术措施, via [Kepu China](https://www.kepuchina.cn/article/articleinfo?business_type=100&ar_id=212023)† |
| Heat-load index, energy-saving housing | 40–45 W/m² | R (weak: standard not named) | [gas-industry article](https://www.chinacqpgx.com/nbnews/shownews?id=11010)† |
| 18 m² room held at 15 °C, −25 °C outside, old stock | about 1.44 kW (80 W/m²) | D | room 4.5 × 4.0 × 2.7 m, one outside wall with a 1.8 × 1.5 m window. Wall 9.45 m² × 1.28 × 40 K = 484 W; window 2.7 m² × 2.8 × 40 = 302 W; air 0.335 Wh/m³K × 48.6 m³ × 1.0 /h × 40 = 651 W. Neighbouring rooms at the same temperature |
| Same, variants | 0.5 air changes: 1.11 kW; single steel window: 1.85 kW; corner room: +0.55 kW | D | same method |
| Same room, JGJ 26-2018 flat | about 0.63 kW (35 W/m²) | D | wall 0.35, window 1.6, 0.5 air changes: 132 + 173 + 326 W |
| Cross-check with the index | 18 m² × 58–64 W/m² = 1.04–1.15 kW | D | the index averages whole buildings, inner rooms included, and was set for milder northern cities, so the room figure is higher |
| One heated room in an unheated block (neighbours at about 0 °C) | about +2.1 kW on top: about 3.5 kW in all | D (upper bound) | two inner brick walls 21.4 m² × 1.78 × 15 K = 571 W; floor 18 × 2.3 × 15 = 621 W; ceiling 18 × 3.2 × 15 = 864 W. Neighbours would warm, so this is a steady-state ceiling |

**How sure.** The heating values are good: Chinese national raw coal, the Russian ministry order and Forest Research. No heating value was found for a named Heilongjiang or Amur mine; Amur brown coal is probably near Russia's 13.7 MJ/kg, unconfirmed. No named coal stove gives a burn rate, and the GB 16154 numbers were not read. Furniture masses are derived, with only snippet cross-checks. The biggest unknown in the room heat is whether the flats around are heated; it can double the need.

---

## 6. The body

### 6a. Water

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Minimum daily water loss, sedentary adult | breath 250–350, urine 500–1,000, faeces 100–200, skin 450–1,900 mL; metabolic water made 250–350 mL; net loss at least 1,050 mL a day | R | [Institute of Medicine, *Dietary Reference Intakes for Water, Potassium, Sodium, Chloride, and Sulfate* (2005), ch. 4, Table 4-2](https://www.nationalacademies.org/read/10925/chapter/6) |
| Minimum urine | about 500 mL a day; older adults about 700–1,100 | R | same |
| Cold raises urine output | direction only, no size given | R | same |
| Breath water at rest | about 10 mL/h at 25 °C, 13 at 0 °C, 15 at −20 °C | R | [Freund and Sawka, ch. 9 in IOM, *Nutritional Needs in Cold and in High-Altitude Environments* (1996), Table 9-2](https://www.nationalacademies.org/read/5197/chapter/14) |
| Soldiers in cold field work | often dehydrated by 3–8% of body weight; thirst is blunted in cold | R | same |
| Minimum drink at rest, comfortable temperature | about 1 L a day | R | [Tipton and Montgomery, "The physiology of survival: fluid and food", *Experimental Physiology* 2025, doi 10.1113/EP093194](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC13140192/fullTextXML) |
| 2% of body weight lost | thirst; mental performance declines | R | IOM 2005, above |
| About 5% | headache, irritability, light-headedness | R | Tipton and Montgomery 2025 |
| 8–10% | performance badly impaired; weakness, dizziness, fainting, weak fast pulse, fast shallow breathing | R | same |
| 10% | Adolph's subjects lost 10% with little added risk of death, but needed help beyond it | R | IOM 2005, p. 120, citing Adolph 1947 |
| Over 10% | delirium common, senses fail | R | [Allen, "Thirst", *Natural History* 1956](https://nhmag.com/picks-from-the-past/181950/thirst?page=3) (a summary of Adolph) |
| Lethal | 15–20% of body water (Tipton and Montgomery 2025); 25% of body weight "probably fatal at any temperature" (Adolph via Allen 1956) | R | as above |
| 15–20% of body water as share of body mass | about 9–12% | D | body water ≈ 60% of mass (textbook value, not sourced here) |
| Survival with no water, cool, at rest | about 10 days below 21 °C, inactive, in shade, eating little; the longest among WWII shipwreck survivors was 11 days | R | [Allen 1956](https://nhmag.com/picks-from-the-past/181950/thirst?page=2), from Adolph's calculation |
| Survival, comfortable surroundings | a week or more | R | [Packer, *Scientific American* 2002](https://www.scientificamerican.com/article/how-long-can-the-average/) |
| People trapped after earthquakes | longest reliable 14 days; average longest across 18 quakes 6.8 days; whether they had water is not recorded | R | [Macintyre, Barbera and Smith, *Prehospital and Disaster Medicine* 2006](https://www.cambridge.org/core/journals/prehospital-and-disaster-medicine/article/surviving-collapsed-structure-entrapment-after-earthquakes-a-timetorescue-analysis/D24F04BFBA9EDA264EA11010418B09B5)† (abstract) |
| Ill elderly who stop food and drink | 85% died within 15 days | R | [Ganzini et al., *NEJM* 2003](https://pubmed.ncbi.nlm.nih.gov/12878744/)† (abstract) |
| No food and no water together | not compatible with life for more than a few days | R | [Gétaz et al., "Hunger strike", *Swiss Medical Weekly* 2012](https://smw.ch/index.php/smw/article/download/1579/2043?inline=1) |
| Days to each stage, 70 kg adult, no intake, net loss 1.05–1.5 L a day | 2%: 0.9–1.3 d; 5%: 2.3–3.3 d; 10%: 4.7–6.7 d; 15%: 7–10 d | D | 70 kg × share ÷ daily loss, e.g. 10.5 L ÷ 1.05 = 10 d, matching Adolph's 10 days |

### 6b. Food (with water)

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Death in complete starvation | usually within 40–60 days | R | Tipton and Montgomery 2025, above |
| Serious complications and death | especially from day 40; people of normal weight survive up to 2–2.5 months | R | Gétaz et al. 2012, above |
| Serious medical problems | begin at about 18% loss of starting weight | R | same |
| Hospital criteria in hunger strikes (no consensus) | weight loss over 10%; BMI below 16.5–18; core temperature below 35.5 °C; pulse below 35 | R | same, Table 2 |
| Performance | declines once 10% or more of body weight is lost; work capacity holds for less than 10 days of severe restriction | R | Tipton and Montgomery 2025 |
| Irish hunger strikers, 1981 | ten died after 46–73 days | R | [Wikipedia, "1981 Irish hunger strike"](https://en.wikipedia.org/wiki/1981_Irish_hunger_strike)† (secondary) |
| 43-day water-only hunger strike | average weight loss 17.9%; no deaths | R | [Faintuch et al., *Revista do Hospital das Clínicas* 2000](https://scielo.br/j/rhc/a/cBdyRngyB9rQT4Tgbpdj3pG/?lang=en) |
| First-week weight loss | about 0.9 kg a day, mostly water and salt | R | [Kerndt et al., *Western Journal of Medicine* 1982](https://pubmed.ncbi.nlm.nih.gov/6758355/)† |
| BMI at death | about 13 for men, 11 for women | R | [ENN *Field Exchange* 15 (2002), summarising Henry 2001](https://ennonline.net/fex/15/limits)† |
| BMI 13 as weight lost, man 70 kg, 1.75 m | about 43% | D | 1 − 13 ÷ 22.9 |
| Minnesota Starvation Experiment | about 1,570 kcal a day for 24 weeks; most lost over 25% of weight; anaemia, fatigue, apathy, extreme weakness, leg swelling | R | [Kalm and Semba, *Journal of Nutrition* 2005](https://pubmed.ncbi.nlm.nih.gov/15930436/)† (abstract) |
| Leningrad siege rations | from 20 Nov 1941: 250 g bread a day for manual workers, 125 g for other civilians | R | [BMJ correspondence on Sparén et al. 2004](https://pmc.ncbi.nlm.nih.gov/articles/PMC313894)† |
| Starvation and shivering | liver glycogen can be gone in about 24 hours | R | Tipton and Montgomery 2025 |

### 6c. Hypothermia

| Core temperature | Stage and signs | Tag | Source |
|---|---|---|---|
| above 35 °C | cold-stressed, normal function | R | [AAFP 2020 summary of the Wilderness Medical Society 2019 guideline (Dow et al.)](https://www.aafp.org/pubs/afp/issues/2020/1101/p571.html), Table 1 |
| 35–32 °C | mild (Swiss HT I): conscious, shivering, clumsy | R | same; [Deslarzes et al., *Scand J Trauma Resusc Emerg Med* 2016](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC4756507/fullTextXML), Table 1 |
| 32–28 °C | moderate (HT II): consciousness impaired; shivering slows near 32 °C and stops by 30 °C; dangerous heart rhythms below 32 °C | R | same |
| 28–24 °C | severe (HT III): unconscious, vital signs present; cardiac arrest risk rises steeply below 28 °C | R | Deslarzes 2016 |
| below 24 °C | HT IV: no vital signs | R | same |
| Model survival in still air, end point core 30 °C | naked: 9 h at 0 °C, over 24 h at 10 °C; two thin clothing layers: over 24 h at −10 °C | R | [Tikuisis, *International Journal of Biometeorology* 1995](https://pubmed.ncbi.nlm.nih.gov/8530210/)† (abstract) |
| Implied cooling rate | naked at 0 °C about 0.8 °C an hour; clothed at −10 °C or warmer under 0.3 °C an hour | D | (37 − 30) ÷ 9; 7 ÷ 24 |
| Elderly at rest indoors | core temperature does not usually fall until a resting, clothed elderly person has spent 2 hours or more at 9 °C or below | R | [Collins, *Age and Ageing* 1986](https://pubmed.ncbi.nlm.nih.gov/3751747/)† (abstract) |
| Elderly at home, UK winter 1972 | about 10% had a core temperature below 35.5 °C | R | [Fox et al., *BMJ* 1973](https://pubmed.ncbi.nlm.nih.gov/4686555/)† (abstract) |

### 6d. Indoor cold and respiratory illness

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Per 1 °C fall in **outdoor** temperature, 892 Finnish conscripts | upper-airway infection +4.3%; common cold +2.1%; sore throat +2.8%; lower-airway infection +2.1% | R | [Mäkinen et al., *Respiratory Medicine* 2009](https://pubmed.ncbi.nlm.nih.gov/18977127/)† (abstract) |
| Indoor temperature and COPD symptoms, Shanghai, 82 patients | odds ratio 0.977 at moderate humidity (if per °C, about +2.4% per °C colder); keep indoors at 18.2 °C or above | R/D | [Mu et al., *International Journal of Biometeorology* 2017](https://pubmed.ncbi.nlm.nih.gov/28567499/)† (abstract; the unit is not stated) |
| Below 16 °C; below 12 °C | below 16 °C resistance to respiratory infection may fall; below 12 °C short rises in blood pressure | R | Collins 1986, above |
| UK cold-home thresholds | 18 °C minimal risk for a sedentary, dressed person; under 16 °C respiratory resistance down; 9–12 °C blood pressure and heart risk up; 5 °C high hypothermia risk | R | [Public Health England, *Cold Weather Plan: Making the Case*, Table 1.1](https://derbyshire.gov.uk/site-elements/documents/pdf/social-health/health-and-wellbeing/your-health/cold-weather-plan-making-the-case.pdf)† |
| Insulation trial, New Zealand, 4,407 people | bedrooms only 0.5 °C warmer, yet wheeze odds 0.57, GP visits 0.73 | R | [Howden-Chapman et al., *BMJ* 2007](https://pmc.ncbi.nlm.nih.gov/articles/PMC1808149)† (abstract) |
| Heating trial, children with asthma | living room 1.1 °C warmer; wheeze disturbing sleep odds 0.55, night cough 0.52 (new heaters also cut indoor pollution) | R | [Howden-Chapman et al., *BMJ* 2008](https://pmc.ncbi.nlm.nih.gov/articles/PMC2658826)† (abstract) |
| Acute chilling of the feet | cold symptoms within 4–5 days in 14.4% of chilled people against 5.6% of controls: × 2.6 | R/D | [Johnson and Eccles, *Family Practice* 2005](https://pubmed.ncbi.nlm.nih.gov/16286463/)† (abstract); 14.4 ÷ 5.6 |
| COPD and home warmth | worse health with fewer days the living room reached 21 °C for 9 hours | R | [Osman et al., *European Journal of Public Health* 2008](https://pubmed.ncbi.nlm.nih.gov/18367496/)† (abstract; no effect size) |
| Review | 17 of 20 studies linked homes below 18 °C to worse health | R | [Janssen et al., *Public Health* 2023](https://pubmed.ncbi.nlm.nih.gov/37820536/)† (abstract) |
| UK excess winter deaths | about a third respiratory, about 40% cardiovascular | R | [Marmot Review Team 2011, as reported in the BMJ](https://www.bmj.com/content/342/bmj.d2910)† |

**How sure.** The water budget (IOM) and impairment bands are solid. The lethal point is where sources differ: about 9–12% of body mass (Tipton), 10% "little risk" in Adolph's subjects, 25% "probably fatal". None of the lethal figures comes from cold conditions, and no figure for cold diuresis per day was found. The 40–60 day death window without food rests on three kinds of source and is firm. Hypothermia bands agree across the WMS, Swiss and MSD stagings. No measured cooling rate for a dressed adult indoors at 0–10 °C was found. Cold homes clearly harm the lungs, but no source gives a clean indoor per-°C rate: Mäkinen's 4.3% is for outdoor temperature, and Mu's odds ratios have no stated unit. Leningrad's clinical stages of alimentary dystrophy and the ICRC hunger-strike guidance could not be read.

---

## 7. Shops in the first days of an occupation or siege

Day 1 is the first 24 hours.

| Quantity | Value | Tag | Source |
|---|---|---|---|
| Kharkiv, day 1 (24 Feb 2022) | bread already short; supermarket shelves emptying fast; small shops closing after 14:00; dairy sold out by 15:00 | R (diary) | [Yatsenko, "A researcher's diary during full-scale war", Baltic Worlds](https://balticworlds.com/?p=10342) |
| Kharkiv, days 2 and 4 | day 2: most shops shut; day 4: 3.5-hour supermarket queue, all alcohol sold out | R (diary) | same |
| Lviv, day 1, by early afternoon | grains, pasta, bottled water and toilet paper shelves empty; queues at ATMs, pharmacies and supermarkets | R | [ZAXID.NET, «Макаронів хватить всім»](https://zaxid.net/makaroniv_hvatit_vsim_n1536722) ("There's enough pasta for everyone") |
| Kyiv, day 1 | by afternoon, cereal, spaghetti and canned meat almost gone; ATMs out of cash | R | [IWPR](https://iwpr.net/hz07cmfs)† |
| Kyiv, about days 18–28, supply lines open | shelves "more than 60%" stocked after a large delivery | R | [Kyiv Independent, 23 Mar 2022](https://kyivindependent.com/kyiv-keeps-supply-chains-open-amid-defense/) |
| Mariupol, days 1–4 | day 1: rush to shopping centres; day 2: empty shelves, bread up to 100 UAH/kg; day 3: residents looted the Metro and PortCity stores, food first, then TVs and phones; day 4: shops empty | R (chronicle) | [Тексти, «Війна. Початок. Маріуполь»](https://texty.org.ua/d/2023/mariupol_chronicles/) ("War. The beginning. Mariupol") |
| Mariupol, day 11, after the blackout | residents broke into dark stores; food running out | R | [AP via BreakingNews.ie, "'Why? Why? Why?' Ukraine's Mariupol descends into despair"](https://www.breakingnews.ie/ukraine/why-why-why-ukraines-mariupol-descends-into-despair-1275521.html) |
| Melitopol, days 4–5 (occupied about 25–26 Feb) | mass looting of chain supermarkets and electronics stores by residents; night patrols of up to 150 people | R | [Радіо Свобода, 28 Feb 2022](https://www.radiosvoboda.org/a/news-zaporizhya-vyizd-v-enerhodar/31728279.html) |
| Kherson, day 8 (occupied 1–3 Mar) | most shelves empty; shops open a few hours; long queues | R | [NBC News, 3 Mar 2022](https://www.nbcnews.com/news/world/ukraine-kherson-siege-russia-invasion-putin-rcna18505) |
| Kherson, day 8 | Russian soldiers looting local shops | R (Ukrainian-side report) | [Ukrainska Pravda, 3 Mar 2022](https://www.pravda.com.ua/eng/news/2022/03/03/7327859/) |
| Kherson, first days | many looters for a few days until residents organised; people with money stockpiled first | R (testimony) | [Meduza, 9 Mar 2022, "The poorer ones are already starving"](https://meduza.io/en/feature/2022/03/09/the-poorer-ones-are-already-starving) |
| Kherson, first week | insulin and vital drugs gone; pharmacy shelves empty | R | [AFP via IBTimes, 31 Mar 2022](https://www.ibtimes.com/medicine-shortages-russian-army-searches-life-occupied-kherson-3456486) |
| Sumy, about day 12 (encircled) | no meat, dairy, grains or fresh produce; only chocolate and juice | R (NGO testimony) | [HRW, 9 Mar 2022, "Trying to Survive in a Besieged Ukrainian City"](https://www.hrw.org/news/2022/03/09/trying-survive-besieged-ukrainian-city) |
| Nova Kakhovka, day 13 (occupied day 1) | supermarket shelves and petrol stations empty | R | [Euronews, 9 Mar 2022](https://www.euronews.com/2022/03/09/empty-supermarket-shelves-in-russia-occupied-city-of-nova-kakhovka) |
| Gaza, day 11 (17 Oct 2023) | 4–5 days of food left in shops; 5 of 23 WFP bakeries working | R | [Gulf News, quoting WFP](https://gulfnews.com/world/mena/israel-gaza-war-four-or-five-days-of-food-left-in-gaza-shops-world-food-programme-says-1.98801997) |
| Gaza, day 23 | days of stock left: rice 1, oil and pulses 3, sugar 8, flour 23 | R | [WFP Market Monitor Flash Update 3](https://docs.wfp.org/api/documents/WFP-0000153560/download)† |
| Wuhan, lockdown morning, 23 Jan 2020 | vegetables sold out within hours; queues for petrol | R | [Fortune, 23 Jan 2020](https://fortune.com/2020/01/23/china-wuhan-lockdown-flights-suspended-virus-outbreak) |
| COVID panic buying, 54 countries | panic is sudden and strong and fades in a week to ten days | R (peer-reviewed) | [Keane and Neal, "Consumer panic in the COVID-19 pandemic", *Journal of Econometrics* 220 (2020)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7447232) |
| Sverdlovsk, 22 Jun 1941 | counters emptied of soap, salt and matches in 2 hours | R (memoir and police report) | [E1.ru, 22 Jun 2024, «Война-то всего месяца на два!»](https://www.e1.ru/text/culture/2024/06/22/73725392/) ("The war will only last a couple of months!") |
| New Orleans, Hurricane Katrina | looting reported day 2; "grows exponentially" day 3 | R (secondary timeline) | [Fox News, Katrina timeline](https://www.foxnews.com/story/fast-facts-hurricane-katrina-timeline) |
| Sarajevo, 1992 | by May, stores looted, empty or closed for weeks | R | [Christian Science Monitor, 12 May 1992](https://www.csmonitor.com/1992/0512/12012.html)† |
| Normal shop stock (UK) | about 10 days of food held by grocers | R | [Full Fact, 2013, citing Defra/IGD](https://fullfact.org/economy/uks-food-supply-hanging-balance/) |
| Looting in Bucha and Kherson by soldiers | supermarkets first, then smaller shops; see `occupation.md` 1.6 | R | `occupation.md` 1.6 |

**How sure.** The order is consistent across cities. On day 1, residents buy out staples, cash, medicine and fuel. Residents start looting on days 3–5, when shops are left unguarded and the lights or police go (Mariupol, Melitopol). Soldiers loot shops in occupied towns (Kherson; Bucha in `occupation.md`). By days 8–13, occupied or encircled towns have empty shelves except for sweets, juice and expensive goods. Where deliveries continued (Kyiv, Wuhan, COVID), shelves recovered in one to three weeks. Almost everything is testimony or news; no war case has store-level stock data. Nothing usable was found for Leningrad 1941 shops, Grozny or Aleppo.

---

## Suggested game numbers

### Outdoor temperature by day of year and hour

T(day, hour) = norm(day) + winter offset + anomaly(day) + diurnal(hour, month)

| Part | Value | Tag |
|---|---|---|
| norm(day) | straight line between the Heihe 1991–2020 monthly means placed mid-month: day 16 −21.6; 45 −16.9; 75 −7.2; 105 4.5; 136 12.8; 166 18.9; 197 21.6; 228 19.2; 258 12.4; 289 2.8; 319 −10.7; 350 −20.6 °C. Game start, day 17: −21.6 °C | R values (CMA via Wikipedia†); D interpolation |
| winter offset | one draw per game, normal with SD 1.3 °C | D, fitted (see check below) |
| anomaly(day) | AR(1), one value per day placed at 12:00 and interpolated linearly through the day: anomaly = 0.66 × yesterday's + normal(0, 3.8 °C) | D, fitted. The raw daily statistics are lag-1 0.74 and SD 4.1 °C (1d), but smoothing through the day damps them, so the fitted values differ. Using the same model in February–May is P |
| diurnal(hour, month) | January offsets from the daily mean, Beijing time: 02:00 −2.2; 05:00 −2.9; 08:00 −2.7; 11:00 +2.9; 14:00 +4.6; 17:00 +1.7; 20:00 −0.1; 23:00 −1.3 °C; smooth between them. Multiply by 1.35 × (1 − 0.03 × anomaly), so cold days have a wider range (observed: 11.5 °C on days with mean ≤ −24 °C, 8.9 °C on days ≥ −17 °C). For later months also multiply by the month's mean range ÷ 10.6: Feb × 1.20, Mar × 1.18, Apr × 1.18, May × 1.26, and move the cold point earlier with sunrise (about 05:00 Beijing time by mid-March) | offsets D (1e); 1.35 and 0.03 D, fitted; month factors D from 1a; the sunrise shift is P, backed by the one March week |
| Check of the model against the 11 observed Januaries, at the same mean level | mean daily range 10.4 (observed 10.2) °C; SD of January means 2.3 (2.3); SD of the daily mean 4.8 (4.7); day-to-day change SD 3.0 (3.0); nights at −30 °C or colder 14% (18%); cold spells of 10 days or more in 1.5% of Januaries (1 in 11 observed) | D. Simulation of 3,000 Januaries: `out/research/living/sim_grid2.mjs 0.66 3.8 1.3 1.35 0.03` |
| Known shortfall | the model makes too few long cold waves and slightly too few very cold nights. A cold-wave event (for example once every few winters, 7–15 days at about −8 °C below norm, like 16–30 Jan 2023) would cover it | P |
| Hard limits | January maximum never above 0 °C; minimum floor −45 °C | R (record highs −0.4 and +0.2 °C; records −44.5 and −45.4 °C) |

| Other weather | Value | Tag |
|---|---|---|
| Wind, December–March | mean 2 m/s, mostly west and north-west; calm about 10–15% of the time | R (СП 131; pogodaiklimat) / D (sample) |
| Days with snow falling, Jan / Feb / Mar / Apr | about 10 / 7 / 7 / 5 (Heihe); 12 / 7 / 8 / 6 (Blagoveshchensk) | R |
| Snow on the ground | mean depth 12 cm in January, 10 in February, 3 in March, gone in April; deepest on record 37 cm | R (Blagoveshchensk) |
| Blizzard days | 3 in January, 1 in February, 2 in March | R (Blagoveshchensk) |
| Precipitation | about 5 mm water a month in January and February | R |

### River

| Number | Value | Tag |
|---|---|---|
| Freeze-up | day 322 (18 Nov); random range day 316–352 | D (Chinese normal); range: Chinese means 12–22 Nov and the latest on record (R) |
| Break-up, start of ice drift | day 116 (26 Apr); random range day 108–130 | R |
| Ice thickness, mid-channel | 17 Jan 55 cm; 31 Jan 65 cm; 15 Feb 80 cm; 15 Mar 100 cm; 31 Mar 103 cm (peak); thins through April | 17 Jan D (mean of 50, 51, 60); 15 Mar and 31 Mar D (norm from the 2024 report); 31 Jan and 15 Feb P (interpolated) |
| Year-to-year spread of ice | ± 25 cm | D (81–119 cm across stations, 110 cm in a thick January) |
| Who can cross | on foot once frozen; trucks need 61 cm or more; 20 t at 55 cm | R |

### Flats

| Number | Value | Tag |
|---|---|---|
| Cooling law | t = t_out + (t_now − t_out) · e^(−Δt/β), applied step by step toward the settled temperature below | R (form: Soviet practice; Vasiliev 2025) |
| β, middle flat in a 1980s brick or panel block | 60 h | D/P: inside the 40–100 h of the АКХ 1989 table; 40 h is the norm's conservative value |
| β multipliers | top floor or corner × 0.6; post-2010 insulated block × 1.6; one pane broken × 0.7; a whole window gone × 0.15 | D (АКХ 1989 table; Vasiliev 2025; Larsen 2018 with Armstrong 2000) |
| Settled temperature, block unheated, people only | outdoor + 3 K | D (Armstrong 2000) |
| Settled temperature, people and cooking | outdoor + 8 K | D (Armstrong 2000) |
| A whole window gone | the flat sits at outdoor temperature (+0.5 K) | D |
| Fall in the first 12 hours | faster than the exponential: use β × 0.7 for the first 12 h | P, shaped by Samarin 2024 |

### Basements

| Number | Value | Tag |
|---|---|---|
| Under a heated block, or in the first days after the heat stops, at −25 °C outside | +5 °C | D, bracketed by +2 °C design minimum, Kyiv +5 °C and Beijing 0–5 °C (R) |
| After 2–4 weeks with no heat in the block | drift to 0 °C; about −5 °C if the flats above freeze | D/P |

### Fuel

Treat fuel as stored energy, released at the stove's rate. Room heat need: 1.4 kW for an 18 m² room at 15 °C in an old block at −25 °C (D), 0.65 kW in a JGJ 26-2018 flat (D), about 3.5 kW for a lone heated room in a cold block (D, upper bound). Stove efficiency 50% (P).

| Fuel | Useful heat | Hours at 1.4 kW | Tag |
|---|---|---|---|
| Raw coal, 1 kg (20.9 MJ/kg) | 2.9 kWh | 2.1 h | D |
| Raw coal, 25 kg sack | 73 kWh | 52 h | D |
| Amur brown coal, 1 kg (13.7 MJ/kg) | 1.9 kWh | 1.4 h | D |
| Wooden chair, 5 kg (14.7 MJ/kg) | 10 kWh | 7 h | D |
| Dining table, 21 kg | 43 kWh | 31 h | D |
| Particleboard wardrobe, 93 kg (16.5 MJ/kg) | 213 kWh | 152 h (about 6 days) | D |
| Hollow interior door, 14 kg (16.5 MJ/kg) | 32 kWh | 23 h | D |
| Solid pine door, 32 kg | 65 kWh | 47 h | D |

A 4.5 kW cast-iron stove at 50% burns about 2.2 kg of wood an hour (D: 9 kW input ÷ 4.08 kWh/kg). A chair is then about 2.3 hours of fire, and the room coasts on its time constant after that.

### Body

| Number | Value | Tag |
|---|---|---|
| Water lost with no intake, resting adult in a cold flat | 1.3 L a day | D: IOM net 1.05 L + 0.2 L for older adults' higher minimum urine; rounding up for cold diuresis is P |
| No water: impaired (5%) | day 3 | D |
| No water: collapse, delirium (10%) | day 5 | D |
| No water: death (about 15%) | day 8, range 7–11 | D/P, inside Adolph's 10 days and the 11-day shipwreck maximum (R) |
| No food: marked weakness (about 10% weight lost) | day 12 | D/P |
| No food: collapse, serious illness (about 18%) | day 40 | R/D (Gétaz 2012; Faintuch 2000) |
| No food: death | day 60, range 46–73 | R (Tipton 2025; 1981 hunger strikers) |
| Partial rations: death | at BMI 13 (men) or 11 (women) | R |
| Cold flat shortens the food numbers | days × 0.8 | P |
| Hypothermia bands | >35 °C fine; 35–32 shivering, clumsy; 32–28 confused, shivering stops near 30; 28–24 unconscious, heart may stop; <24 no vital signs | R |
| Core cooling | awake, dressed, fed, dry: none; still or asleep at 9 °C or below for over 2 h: −0.1 °C/h; under-dressed or wet: −0.3 °C/h; naked at 0 °C: −0.8 °C/h | P inside D bounds (Tikuisis 1995; Collins 1986) |
| Respiratory infection risk | × 1.04 per °C below 18 °C indoors, compounding: × 1.4 at 10 °C, × 1.7 at 5 °C, × 2.0 at 0 °C | D/P (Mäkinen 2009 outdoor 4.3%/°C; Mu 2017 about 2.4%/°C) |
| Chest infection (pneumonia-type) risk | × 1.02 per °C below 18 °C | D/P (Mäkinen 2009, lower airway, outdoor) |
| After a soaking or chilling | × 2.6 for the next 4–5 days | R/D (Johnson and Eccles 2005) |

### Shops

Share of pre-war stock left on shelves when the occupation cuts resupply. The timing is R (section 7); the fractions are P.

| End of day | All goods | Staples (bread, water, grains, tins, baby food, insulin and common drugs, fuel) | What drives it |
|---|---|---|---|
| 1 | 0.45 | 0.10 | residents buying |
| 2 | 0.30 | 0.05 | most shops shut |
| 3 | 0.15 | 0.02 | residents start looting: food, then alcohol, electronics, phones |
| 7 | 0.05 | 0 | sweets, juice and dear goods left; soldiers loot shops |

If deliveries resume, shelves climb back to about 0.6 within about 3 weeks (R, Kyiv 2022).
