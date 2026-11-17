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
- Native Godot 4 game for PC and laptop. The owner chose Godot over a browser build so the game can ship as a real desktop executable, without a bundled browser, and so it can use Godot's lighting. Windows is the primary export. Linux, macOS and Web presets are in the repo. The story still begins in the Northeast and still aims, later, at Harbin.

Status: Prototype (原型). The first playable slice is the evening of 17 January 2031 in central Heihe, on the blocks between 中央街 and the Heilongjiang promenade. Harbin is not in this build.

© 2026 李家寶. All rights reserved.

## The Rust build

The game is being rewritten in Rust, starting with the city of Heihe itself. `crates/sim` holds the world and the rules with no renderer in it; `crates/game` draws it with Bevy. See `docs/DESIGN.md`.

```bash
cargo run -p northeast --release
cargo run -p northeast --release -- --hour 18.5
cargo run -p northeast --release -- --shots out/shots
cargo run -p northeast --release -- --bench out/bench
cargo test -p sim
```

WASD walks, Shift runs, the wheel zooms, `[` and `]` turn the clock. A building that stands between the camera and you turns to glass. `--shots` saves a set of views of the town to a folder from a hidden window, and writes the frame time at each to `frame_times.txt`. `--bench` times the drawing at some of those views with one setting changed at a time, by frame, main thread, render thread and GPU pass, and writes `bench.txt`. Time the release build: the dev build keeps Bevy's and wgpu's debug checks on. The settings that trade looks for speed (multisampling, shadow filtering, the cut-away glass, lamps per head or per post, the depth prepass) are environment variables, listed in `crates/game/src/graphics.rs`; what each costs is in `docs/rust-r3-receipt.md`. The colours of the city are in `data/heihe/look.json`, each with the photograph it was read from.

The buildings are OpenStreetMap's, filled in from CMAB (see `CREDITS.md`). To cut the CMAB layer again: download `heilongjiang.zip` from figshare 27992417 into `data/raw/cmab-v7/`, extract `processed_1809_20230514_all.*` into `data/raw/cmab-v7/heihe/`, and run `python -I tools/cmab/slice_heihe.py data/raw/cmab-v7/heihe/processed_1809_20230514_all.shp data/heihe/cmab.json` with a Python that has `pyshp`.

The Godot project below is the earlier build. It stays until the Rust one does everything it does.

## Play the Heihe slice

You need [Godot 4.7.2 stable](https://godotengine.org/download/archive/4.7.2-stable/). Open this folder as a project and press Play. Or, with the editor binary on your PATH:

```bash
godot --path .
```

Until 17:36 it is an ordinary evening and the shops are open. Every shop sign is a door: stand under it and hold E to take what the shop has (a pharmacy has bandages, a clothes shop a padded coat). At 17:36 Russian troops cross the frozen Heilongjiang. From then on the goal is to be alive at 07:00. You have one life; a run that ends starts again at 16:00.

You are not a soldier and you have no weapon. Infantry groups of two to five patrol the streets the front has reached. A soldier has to see you: inside his reach for the light, inside the arc he faces, with nothing solid in between. Running is heard. If a group is sure of you it shouts: stand still and you are searched and robbed, run and they shoot. A small drone comes and goes; if it holds you in view, mortar rounds follow a few minutes later. A doorway hides you from it. Rifle wounds bleed until bandaged, the cold kills, and the heating stops at 21:00.

The Russian side's numbers are in `data/russia/forces.json`, each with its source. They follow how the equipment performed in Ukraine, as far as open reporting goes, and say where a number had to be chosen for play. The design is in `docs/DESIGN.md`, the findings in `docs/research/ukraine-performance.md`.

The camera is fixed above and to the northeast, looking at your feet, so north is toward the bottom left of the screen. The mouse wheel zooms between 0.3 and 2.2. A ring at your feet shows through walls. After the alert, H starts the older defend-the-shop night on 中央街 (scavenge, place barricades with the left mouse button, B to begin, hold until 06:00); only there are you handed a rifle. Sources for the fighting are in `docs/WAR_SOURCES.md`. What the 中央街 block is matched to is in `docs/HEIHE_LOOK_SOURCES.md`.

### Controls

| Action | Keyboard and mouse | Gamepad |
| --- | --- | --- |
| Move | WASD or arrows | Left stick |
| Zoom | Mouse wheel | Pinch |
| Run | Shift | Push the stick, or RB |
| Talk to someone, or hold to take from a shop | E or F | A |
| Bandage / eat / drink / hand warmer | Q / Z / X / C | |
| Start again after dying | Enter | |
| Pause, quality, credits | Esc | Start |

Click the window, or press a key, to start. Quality is Low / Medium / High from the pause menu and is remembered. Low turns off volumetric fog, SDFGI, SSAO and most real lights so an integrated GPU can hold a frame rate. High turns those back on.

Body temperature starts at 36.8°C. The street is about −24°C. Standing in a shop doorway warms you until the heating stops at 21:00; after that it only keeps the wind off. The ice is colder, and running makes it worse.

## Build

Exports use the matching 4.7.2 templates. CI pins that version.

```bash
godot --headless --export-release "Windows Desktop" build/windows/Northeast.exe
godot --headless --export-release "Linux" build/linux/Northeast.x86_64
godot --headless --export-release "macOS" build/macos/Northeast.zip
godot --headless --export-release "Web" build/web/index.html
```

GitHub Actions imports the project, runs the Python and Godot tests, and uploads Windows and Linux builds as artifacts on pull requests and on main.

## Map data

Streets, footprints, the river bank and the place names come from OpenStreetMap through the Overpass API. The importer is `tools/osm_import/import_osm.py`. It writes `data/heihe/city.json` plus a glTF mesh the game loads. The same tool is how later cities (Harbin, Changchun, Shenyang) should come in: add a region file, fetch, build.

```bash
python3 tools/osm_import/import_osm.py fetch
python3 tools/osm_import/import_osm.py build
python3 -m unittest tests/test_import.py
```

The committed extract is enough to build without calling Overpass again. Map data in `data/heihe/` is © OpenStreetMap contributors, ODbL. The game code is not. See `LICENSE`, `data/heihe/ODBL.txt` and `CREDITS.md`.

The in-game corner credit is part of that attribution. Please leave it in.
