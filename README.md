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

## Play the Heihe slice

You need [Godot 4.7.2 stable](https://godotengine.org/download/archive/4.7.2-stable/). Open this folder as a project and press Play. Or, with the editor binary on your PATH:

```bash
godot --path .
```

The opening is about eight real minutes (16:00 to 18:00 in the game, at 15×). You are a shop worker who just got off 中央街. Walk north to the river. A phone alert tells you Russian troops are crossing the frozen Heilongjiang. After you have walked a little way, small infantry patrols can meet you in the street. Civilians run for doorways. They are not targets, and nothing in the game scores a kill.

The camera matches DefendTaipei: orthographic, fixed above and to the northeast (`40, 56, 40` metres), looking at your feet. The mouse wheel zooms between 0.5 and 2.2. W and S move along the screen's up axis, A and D along its right. A touch stick does the same thing. Clicking aims and, once a patrol is near, fires. The mouse is not captured. `CombatHooks` is where drones, artillery, armour and retaking a block attach. Civilians are never a target. How much of a building can fall apart follows the same low / medium / high setting: low tracks fewer buildings and fewer storeys. Sources for the fighting are in `docs/WAR_SOURCES.md`.

### Controls

| Action | Keyboard and mouse | Gamepad |
| --- | --- | --- |
| Move | WASD or arrows | Left stick |
| Zoom | Mouse wheel | Pinch |
| Run | Shift | Push the stick, or RB |
| Talk to someone | E or F | A |
| Pause, quality, credits | Esc | Start |

Click the window, or press a key, to start. Quality is Low / Medium / High from the pause menu and is remembered. Low turns off volumetric fog, SDFGI, SSAO and most real lights so an integrated GPU can hold a frame rate. High turns those back on.

Body temperature starts at 36.8°C. The street is about −24°C. Standing in a lit shop doorway warms you. The ice is colder, and running makes it worse.

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
