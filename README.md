# Bigger Item Grids and Smaller Inventory Slots

See more items on screen in the inventory and Crafting/Transfer popups with optional Intro and Mod warning skip.

## What it does

**Inventory:**
Increase visible rows from 5 to **7**, Technology from
3 to **4**.

**Transfer Items and Craft Product popups:** Increases pop-up from
4 by 2 (8 items per page) to **8 by 3 (24 cards per page)**.

**Boot flow:** the startup logos are skipped,
as well as the "mods detected" warning.

> [!NOTE]  
> You can control Intro Skip and Mod Warning with the included `nms-tweaks.toml` in the mod folder under `GAMEDATA/MODS`. `inventory_grid` and `popup_pages` must be `true` for the bigger inventory to function.

![Craft Product popup with the 8 by 3 card grid](assets/bigger_crafting.jpg)

![Cargo and technology grids with smaller slots](assets/bigger_inventory.jpg)

## Install

Install `dist/Bigger Item Grids and Smaller Inventory Slots.zip` as a
**Root Folder** mod ([Amethyst](https://github.com/ChrisDKN/Amethyst-Mod-Manager)) or copy the contents into the game folder.

You should then have:

- `Binaries/xinput9_1_0.dll`: the loader
  (Windows DLL, runs under Proton with
  `WINEDLLOVERRIDES="xinput9_1_0=n,b" %command%`).
- `GAMEDATA/MODS/Bigger Item Grids and Smaller Inventory Slots/`: the
  static UI files (slot/box templates, popup windows, inventory pages),
  plus `nms-tweaks.toml` for the loader toggles.

After launch, `Binaries/nms-tweaks.log` is created to help with debugging.

## Compatibility

Static files: the popup, inventory page, slot and box UI MBINs listed in
`src/mod.lua.in`.

Loader: pattern-scanned hooks against the running NMS.exe, built for
Cosmos v7.04. A game update can shift the patterns; this will be logged and functionality disabled to prevent issues.

## Build (on Linux)

Requires a full AMUMSS install at `~/AMUMSS` (override with
`AMUMSS_HOME=...`) with `MBINCompiler-linux` fetched, plus a Rust
toolchain with the `x86_64-pc-windows-gnu` target and mingw for the
loader.

```sh
make release      # clean rebuild + verify + pack dist/ zip (default)
make verify       # sanity-check the built outputs
make loader       # build the nms-tweaks proxy DLL only
make rootzip      # pack the root-install archive (no rebuild)
make clean        # remove build/ and dist/
```

The loader lives in `loader/` (GPL-3.0-or-later, see `loader/LICENSE`):
a single `xinput9_1_0.dll` proxy that forwards the 7 XInput exports to
the system DLL and applies the enabled patches. The static UI is built
with my [AMUMSS Linux port](https://github.com/SavageCore/AMUMSS/tree/feat/linux-support).

## Game updates / versioning

The mod version is `VERSION` in the `Makefile`. The game version the
script targets is `GameVersion` in `src/mod.lua.in`; the loader pins its
patterns to the same build. When No Man's Sky updates, check
`nms-tweaks.log` first: any `NOT APPLIED` line names the feature whose
patterns drifted.
