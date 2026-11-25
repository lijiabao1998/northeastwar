# L3, first part, receipt: the occupation's first weeks

25 November 2026. Branch `claude/rust-heihe`. The owner said 開始 L3. L2 left a careful player safe indoors for weeks; what wore people down in occupied cities was the occupation. This is the first part of `docs/DESIGN.md`'s step L3:
- the end of the assault;
- the sweep;
- the order sheet and curfew;
- being searched;
- hiding things;
- being taken;
- the watchlist and visits to your home.

Checkpoints, filtration and its slip, the phone's contents, and search waves through a neighbourhood are the second part.

## Where the numbers come from

Every L3 number is in `data/occupation.json`. Each has a `_src` naming its section of `docs/research/occupation.md` (about two hundred sources on Bucha, Hostomel, Kherson, Mariupol, Melitopol, Berdiansk and Izium), and a `_basis`:
- reported: the source gives it;
- derived: worked out from reported figures;
- play: the source gives the shape but not the number.

The code reads them through `crates/sim/src/occupation.rs`.

## What changed

**The assault ends at the first dawn.** At 07:00 on 18 January:
- the front has moved south and the PLA has left Heihe;
- the night's armour, mortars, FPVs and Lancets stop;
- the militia goes to ground;
- the radio has nobody left to answer.

A drone that spots you now calls the nearest patrol over instead of mortars. This also ends what L1's bot saw, the night's tanks still fighting on the 18th.

**The sweep (occupation.md 1.1, 1.2).** Soldiers go through every building in the first 72 hours (Bucha, Hostomel). Each building's turn falls on a daylight hour between 18 and 20 January, rolled from the run's seed. Ten minutes before, you hear boots on the stairs if you are inside.
- **If you are there:** everyone goes to the basement and is searched. A barred door is broken in, and that makes it go worse.
- **If you are not:** your home is turned over. Half the food in the cupboard goes, and what you hid may be found. You find out when you come home.

**Being searched**, by the sweep, a patrol's stop, or a visit (1.2, 1.5, 2.3):
- **What it looks for:** weapons and communication devices: any gun, rounds, anti-tank weapon, grenade, Molotov, FPV, the radio. Found, they are taken, and you are:
  - shot on the spot (15%);
  - taken (85%);
  - otherwise cursed and kicked.
  The shot and taken shares are play; the sources give the shape, not a rate.
- **Clean:** a man is taken 8% of the time, a woman 1.1%. That follows the 88% of the UN's detainees who were men; the 8% itself is play. Being known raises it, and so does a door they had to force.
- **Let go:** they take half of the food, water and medicine you carry.
- **Over 150 seeded runs** (test `a_search_finds_what_you_carry`), shot or taken:
  - carrying a gun, 87%;
  - clean, a man 10% and a woman 1%;
  - the gun hidden at home, 34%.

**Hiding.** At the cupboard in your home, E puts the gun, the rounds, the anti-tank weapons and the radio in the space behind it; E again takes them out. A sweep finds what is hidden 30% of the time, a visit 50% (play).

**The order sheet (1.4).** At noon on 18 January the commandant's Order No. 1 comes as a message. These are the Kherson day-2 conditions:
- curfew 20:00 to 06:00;
- walk alone or in pairs;
- stop at the first order;
- vehicles only by day.

From then on, a patrol that sees you outdoors during the curfew opens fire without a challenge. By day, a challenge ends in a search instead of the night's robbery. The HUD shows 宵禁中.

**Being taken (2.3).** A detention takes you off the map. The lengths follow the split in occupation.md 6.8, fitted to the testimonies (1 day to about 4 months):
- 40% under 4 days;
- 35% 4–30 days;
- 20% 1–4 months;
- 5% moved north and never heard of again, which ends the run.

About 2% die in custody (5 of 226 in Kherson). In custody:
- the food is about 600 kcal a day;
- there is no medical care, so illness builds;
- a quarter are beaten.

Release comes at ten in the morning, at your door. In three cases of ten you signed a paper promising to cooperate, which halves what they hold against you; otherwise it grows. Your weapons and radio are gone.

The world goes on without you:
- rooms cool and stoves go out;
- each morning missed empties the shops;
- the sweep goes through your home if its turn came.

**What they know about you (2.2).** A suspicion score from 0 to 100 builds from:
- calling artillery on the radio (+15);
- launching an FPV (+10);
- rallying the militia (+20);
- being found with contraband (+30);
- being caught firing at troops (+5).

The weights are play; the lists' tiers in 2.2 give the shape. From the fourth morning on, each morning may put a visit to your home on for between 06:00 and 10:00. In Kherson armed men came to activists' homes, sometimes at 6 a.m. The chance is 30% a day for someone fully known. At 40 you are on the list, and a visit that finds you takes you 70% of the time.

**The client.** 宵禁中 on the HUD during the curfew. The death screen says how many times you were taken and for how many days. The key list says what E does at the cupboard.

## Checked

- `cargo test -p sim`: 129 pass, 9 of them new:
  - `occupation`:
    - the sweep falls on its days by daylight;
    - detention lengths follow the split.
  - `survival`:
    - the assault ends at first dawn, even jumping straight to it from the afternoon;
    - the order sheet brings a curfew;
    - in the curfew a patrol shoots instead of challenging;
    - a search finds what you carry, with the rates above over 150 runs;
    - detention takes days and the world goes on: released at 10:00 at your door, every morning missed counted, the sweep through while you were away, at least 30 of 40 come back;
    - you can hide things behind the cupboard;
    - being known brings them to your door: a visit on about one morning in three at full suspicion, none at zero, and most listed people at home are taken.
- `cargo build -p northeast --release` builds. `--interior-shots` has a new picture, `docs/shots/rust-l3a/07_curfew_order.jpg`: 20:10 on 18 January, with 宵禁中, the order's message, and the new objective.
- `days_bot`, 4 seeds per shelter, 10 days, the indoor plan with upkeep, in two variants:
  - `hide`: once the occupation starts, the gun and the radio go behind the cupboard;
  - `keep`: they stay on the player.

  The tables are in `docs/shots/rust-l3a/`.

| variant | untouched for 10 days | taken, came back | shot when searched | died in custody |
|---|---|---|---|---|
| hide | 7 of 8 | 1 (29 days, at home: a clean man's 8%) | 0 | 0 |
| keep | 0 of 8 | 6 (5, 19, 27, 30, 73 and 109 days) | 1 | 1 |

Hiding is the choice the sources point to (occupation.md 1.3: people hid spare phones, families left phones at home), and here it is what keeps a careful player out of the cells. Six of eight who kept the gun were taken, for between five days and three and a half months. The one held 109 days came back on 7 May, at illness 90.

## Not done

- **Checkpoints, filtration and the slip, the phone, search waves through a neighbourhood.** These are the second part of L3.
- **Character choice.** The player is a man of 34. Choosing who you are (and so the odds of being taken) comes with L4's people.
- **What signing to cooperate leads to.** Informing and its price come with L4.
- **Many shares are play, because the sources give the shape but not a rate:** being taken in a sweep, being shot when armed, a cache being found, a visit's odds, and suspicion's weights.
- **The sweep is not seen.** It happens as text and a forced door, not as soldiers going from flat to flat.
- **Detention leaves no new trace on the town.** You come home to the sweep's mess, but not to neighbours who know you were taken.
