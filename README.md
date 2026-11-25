# Northeast: The Russian Invasion

**Winter, 2031.** Russian forces cross the frozen Amur at Heihe and Suifenhe. Within days Harbin, a city of ten million that still wears its old Russian architecture, becomes the first major city under siege.

You are not a soldier. You are an ordinary person in Harbin: a delivery rider, a nurse, a student, a retired railway worker. The heating has stopped and the temperature is -30°C. The war moves on a fixed timeline whatever you do. How you survive it is up to you.

## What the game is

- **Survival in a real city.** Explore Harbin's real streets and landmarks: Central Street (中央大街), St. Sophia Cathedral, the Songhua River and its old iron railway bridge. Later chapters follow the front south to Changchun and Shenyang.
- **Winter is the enemy too.** Cold, darkness, food, fuel and water matter as much as the shelling. The frozen Songhua becomes a road, a front line and a way out.
- **Time keeps moving.** The war advances day by day through scripted events: blackouts, evacuations, checkpoints, sieges, occupation. You cannot change how the war goes, but you can change what happens to you and the people around you.
- **Ordinary people and hard choices.** Neighbours, strangers and soldiers on both sides. Who to trust, who to help, when to leave.

## Notes

- This is a work of fiction set in the near future. All characters and events are invented. Real places are used as settings.
- Native desktop game for PC and laptop, written in Rust and drawn with Bevy. The owner wanted a real desktop executable without a bundled browser, which is why the first build was in Godot. On 15 November 2026 the owner had the whole game rewritten in Rust, and the Godot project was retired on 19 November. Windows is where it is built and tested. The story still begins in the Northeast and still aims, later, at Harbin.

Status: Prototype (原型). What you can play is central Heihe from 16:00 on 17 January 2031: an ordinary evening, the crossing of the frozen Heilongjiang at 17:36, and the night that follows. The night does not end the game. The days under occupation that come after it are being built; see `docs/DESIGN.md`, "Long-term survival". Harbin is not in this build.

© 2026 李家寶. All rights reserved.

## Play

`crates/sim` holds the world and the rules with no renderer in it. `crates/game` draws it with Bevy. The design is in `docs/DESIGN.md`. Each round of work has a receipt in `docs/` (`rust-r1-receipt.md` onward), saying what changed and how it was checked.

```bash
cargo run -p northeast --release
cargo test -p sim
```

The game is built for `x86_64-pc-windows-gnullvm` with the LLD that ships with Rust 1.99.1. `.cargo/config.toml` points at the toolchain on the machine it is made on, so elsewhere you change that path or pass `--target` for your own platform. The rules (`cargo test -p sim`) build anywhere; CI runs them on Linux.

You are not a soldier. Being seen is the danger. A soldier has to see you: inside his reach for the light, inside the arc he faces, with nothing solid in between. Running is heard. Stand still when challenged and you are searched; run and they shoot. Drones bring mortar rounds and FPVs. The heating stops at 21:00, and the cold kills. You have one life. The Russian side's numbers are in `data/russia/forces.json` and follow how the equipment performed in Ukraine (`docs/research/ukraine-performance.md`). Each says whether its source gives it or it was chosen for play, except the vehicle figures, which do not have sources yet. The occupation that follows the night is drawn from `docs/research/occupation.md`.

The clock runs 15 times faster than play, through the night and on into the days. The date and the day are at the top left. Indoors, J lies you down to sleep: time runs fast until you are rested or something wakes you.

### Controls

| Action | Keys |
| --- | --- |
| Walk / run | WASD or arrows / Shift |
| Zoom | Mouse wheel |
| Speed of time (1× to 16×) | Tab, + and −, [ and ] |
| Doors, cupboards, people | E |
| Sleep, and get up | J (indoors) |
| Up and down stairs | Space / Ctrl at a stairwell |
| Bandage / eat / drink / hand warmer | Q / Z / X / C |
| Torch / curtains | L / K |
| Bar a door, board a window, feed the stove | T |
| Rally the neighbours (after the alert) | H |
| Fire / switch weapon | Left mouse / N |
| Heavy weapon: switch / fire | U / F |
| Grenade / smoke / FPV / Molotov | Y / O / I / V |
| Call artillery by radio | R |
| Barricades | 1 to 8, B to build |
| Militia: follow or post / send | G / right mouse |
| Tactical map | M |
| Pause | P or Space |
| Menu, and the full list of keys | Esc |
| Start again after dying | Enter |

### For working on it

```bash
cargo run -p northeast --release -- --hour 18.5
cargo run -p northeast --release -- --shots out/shots
cargo run -p northeast --release -- --bench out/bench
cargo run -p northeast --release -- --interior-shots out/interior
cargo run --release -p sim --bin days_bot -- 3 4
```

- `--hour` starts at a given hour.
- `--shots` saves a set of views of the town from a hidden window, and writes the frame time at each to `frame_times.txt`.
- `--bench` times the drawing at some of those views with one setting changed at a time, by frame, main thread, render thread and GPU pass, and writes `bench.txt`.
- `--combat-shots`, `--menu-shots` and `--interior-shots` take staged pictures of the fighting, the menus and the rooms. `--interior-shots` also takes the town in April, May, July (sun and rain), October and November.
- `days_bot` plays the safest plan for the night on into the days with several seeds, and reports how each run ends.

Time the release build: the dev build keeps Bevy's and wgpu's debug checks on. The settings that trade looks for speed are environment variables, listed in `crates/game/src/graphics.rs`; what each costs is in `docs/rust-r3-receipt.md`. The colours of the city are in `data/heihe/look.json`, each with the photograph it was read from; the colours without snow (grass, leaves, open water, roofs) came from summer and autumn photographs (`docs/research/seasons-look.md`). `cargo run -p sim --release --bin scene_report -- <day>` counts what is drawn on a given day of the year.

The buildings are OpenStreetMap's, filled in from CMAB (see `CREDITS.md`). To cut the CMAB layer again:
1. Download `heilongjiang.zip` from figshare 27992417 into `data/raw/cmab-v7/`.
2. Extract `processed_1809_20230514_all.*` into `data/raw/cmab-v7/heihe/`.
3. Run `python -I tools/cmab/slice_heihe.py data/raw/cmab-v7/heihe/processed_1809_20230514_all.shp data/heihe/cmab.json` with a Python that has `pyshp`.

## Map data

Streets, footprints, the river bank and the place names come from OpenStreetMap through the Overpass API. The importer is `tools/osm_import/import_osm.py`. It writes `data/heihe/city.json`, which the game builds the town from. It was written for the Godot build and also makes a glTF mesh for it, which the Rust build does not use; it has not been run since the Godot project was retired. The same tool is how later cities (Harbin, Changchun, Shenyang) should come in: add a region file, fetch, build.

```bash
python3 tools/osm_import/import_osm.py fetch
python3 tools/osm_import/import_osm.py build
```

The committed extract is enough to build without calling Overpass again. Map data in `data/heihe/` is © OpenStreetMap contributors, ODbL. The game code is not. See `LICENSE`, `data/heihe/ODBL.txt` and `CREDITS.md`.

The in-game corner credit is part of that attribution. Please leave it in.
