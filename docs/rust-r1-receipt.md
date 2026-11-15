# R1 receipt: Heihe, in Rust

16 November 2026. Branch `claude/rust-heihe`. The first pass of the rewrite the owner asked for: all Rust, Heihe first, its real colours.

## What exists now

- A Cargo workspace. `crates/sim` is the world with no engine in it. `crates/game` (`northeast.exe`) draws it with Bevy 0.20 on Vulkan.
- Heihe from the map file: 198 buildings on the Chinese bank, 386 blocks of Blagoveshchensk across the ice, 33 streets, 332 lamp posts, 685 trees, 306 parked cars, the Mother sculpture, boundary marker 192, and the shop boards.
- The city is the right way round. The old build drew the map file's "z north" as plus z, which makes a mirror image. Here north is minus z, in one function, held by a test.
- `data/heihe/look.json`: 52 colours and 5 shop-board liveries, each naming the photograph it was read from (six photographs on Wikimedia Commons, CC0 or CC BY-SA).
- Seven kinds of building taken from those photographs: the salmon Central Street blocks with cream pilasters, gables and a red cone on a corner; the red and white fronts with green domes; the pale grey classical front; yellow housing with crimson balcony bands; tan tile blocks with cream bands; salmon blocks with yellow bays; low white tile over red brick with roller shutters.
- Shop boards in Chinese over Russian, set in Noto Sans, hung on the nearest wall and never overlapping. Names are the OpenStreetMap names.
- The light of 17 January at 50.25° north: a sun that tops out under nineteen degrees and is down before four, then sodium lamps and lit windows. Lamps and windows are in real units; only the camera's exposure changes between noon and night.
- You can walk it: WASD, Shift, the wheel, `[` and `]` for the clock. Buildings are solid and the main channel is out of bounds.
- `--shots <folder>` saves eleven views without putting a window on the screen.

## How it was checked

- `cargo test -p sim`: 16 tests, all passing. They hold, among other things, that the river is north of the city and Blagoveshchensk north of the river, that every colour names its photograph, that Central Street's buildings are dressed as Central Street, that no window faces into its wall, that no shop board reads backwards or hangs over another, that you cannot walk through a building or onto the main channel, and that the Russian numbers behave (sight falls with the light, a rifle is worse further away, drone-cued fire takes 3 to 5 war-clock minutes).
- `northeast --shots out/shots` ran to the end and wrote all eleven pictures on this laptop's Intel Arc 140V. Five are in `docs/shots/rust-r1/`. I looked at each kind of view: housing by day and night, the whole town, the Mother square, the river, boards up close.
- The windowed program was started once and ran fourteen seconds without an error before it was stopped.

## What was not checked, and what is not there

- Nobody has walked it by hand. The walking rule is tested in `sim`; the keys reaching it are not.
- Frame time was not measured.
- A building between you and the camera hides you. On Central Street the mall on its south side fills the screen. A cutaway is needed and is not built.
- The town is only what OpenStreetMap has. Whole blocks are bare snow where real Heihe has buildings.
- Not yet drawn: the teal arch of the Russian goods street, the Ferris wheel, steps up to raised shops, overhead wires, three-wheelers, people, the railing and lamps of the promenade. The Mother sculpture is a stack of blocks with the right colours and proportions. Trees are stubs. Ice is flat bands.
- The survival rules of the Godot build (being seen, the challenge, the drone and the mortar, the body, the shops) are not in the Rust build yet. Only the Russian numbers table and its sight, hit and hearing rules have moved across.
- The first build takes about an hour on this laptop.

## Toolchain, because it cost most of the night

The Rust in `C:\RUST\.toolchain` is the Windows GNU one without a full MinGW. Bevy needs more than it has:

- `windows-sys` needs a `dlltool`, and the one shipped wants an assembler that is not there. `llvm-ar` from the `llvm-tools` component is the same program as `llvm-dlltool` under another name, so it was copied to `C:\RUST\.toolchain\llvm-dlltool\`.
- Built for `x86_64-pc-windows-gnu` that way, the program linked and then died before `main`: an import (`GetCurrentThreadId`) sat in no import table the loader fills. A small test program did not show it; the full one always did.
- Built for `x86_64-pc-windows-gnullvm` with the toolchain's own LLD it works. That target and its `rust-mingw` component come from rustup. `.cargo/config.toml` sets it up.
- That component has no `libadvapi32.a`, `libcredui.a` or `libsecur32.a`. Empty ones were generated into its `self-contained` folder; nothing actually pulls a symbol from them.
- With eight jobs rustc ran out of memory on the `windows` crate while the machine was busy. `-j 4` finished.
