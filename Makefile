# Bigger Item Grids and Smaller Inventory Slots
# Builds the FOMOD installer with the AMUMSS Linux port.
#
# What it does: ships modified UI MBINs that enlarge the card grid on the
# Transfer Items and Craft Product popups, and shrink the inventory slot icons.
#
# Usage:
#   make release      clean rebuild + verify + pack dist/ zip (default)
#                     the zip is a root-install archive (Binaries/ +
#                     GAMEDATA/MODS/) for Amethyst/Vortex
#   make verify       sanity-check the built outputs
#   make clean        remove build/ and dist/
#   make loader       build the nms-tweaks proxy DLL (loader/dist/)
#   make rootzip      pack the root-install archive into dist/ (no rebuild)
#   make assets       regenerate the Nexus page images (gallery + header)
#
# Override the AMUMSS install location if needed:
#   make AMUMSS_HOME=/path/to/AMUMSS release

AMUMSS_HOME ?= $(HOME)/AMUMSS
AMUMSS_LINUX ?= $(HOME)/Git/AMUMSS/linux

VERSION      := 0.0.0

MOD_SET      := Bigger Item Grids and Smaller Inventory Slots

SRC_TEMPLATE := src/mod.lua.in

BUILD := build
DIST  := dist
STAMP := $(BUILD)/.built

FILES_DIR := files

RENDERED := $(BUILD)/mod.lua

.PHONY: all release build verify clean loader rootzip assets

all: release

# Single canonical artifact: the root-install zip (same name the mod
# manager shows). The old mod-folder-only zip is gone; its contents live
# under GAMEDATA/MODS/ inside this archive.
ROOTZIP := $(DIST)/$(MOD_SET).zip

release: clean build verify rootzip
	@echo "release: $(ROOTZIP) (version $(VERSION))"

$(BUILD)/mod.lua: $(SRC_TEMPLATE)
	@mkdir -p "$(BUILD)"
	@cp "$(SRC_TEMPLATE)" "$@"

build: $(RENDERED)
	@test -d "$(AMUMSS_HOME)/MODBUILDER" || (echo "ERROR: no AMUMSS install at $(AMUMSS_HOME)" >&2; exit 1)
	@test -x "$(AMUMSS_HOME)/MODBUILDER/MBINCompiler-linux" || (echo "ERROR: MBINCompiler-linux missing" >&2; exit 1)
	@rm -rf "$(AMUMSS_HOME)/MODBUILDER/MOD" && mkdir -p "$(AMUMSS_HOME)/MODBUILDER/MOD"
	@rm -rf "$(BUILD)/stage" && mkdir -p "$(BUILD)/stage/mbin"
	@if [ -d "$(FILES_DIR)" ]; then \
	  cp -a "$(FILES_DIR)/." "$(BUILD)/stage/mbin/"; \
	  cd "$(BUILD)/stage/mbin" && find . -name '*.MBIN' -print0 | while IFS= read -r -d '' f; do \
	    "$(AMUMSS_HOME)/MODBUILDER/MBINCompiler-linux" "$$f" >/dev/null 2>&1 || echo "WARN: failed MBIN->MXML $$f" >&2; \
	  done; \
	  find . -name '*.MXML' -print0 | while IFS= read -r -d '' f; do \
	    "$(AMUMSS_HOME)/MODBUILDER/MBINCompiler-linux" "$$f" >/dev/null 2>&1 || echo "WARN: failed MXML->MBIN $$f" >&2; \
	    rm -f "$$f"; \
	  done; \
	fi
	@if [ -d "$(AMUMSS_HOME)/ModScript" ]; then mv "$(AMUMSS_HOME)/ModScript" "$(AMUMSS_HOME)/ModScript.makebak"; trap 'rm -rf "$(AMUMSS_HOME)/ModScript"; mv "$(AMUMSS_HOME)/ModScript.makebak" "$(AMUMSS_HOME)/ModScript"' EXIT; fi
	@mkdir -p "$(AMUMSS_HOME)/ModScript"
	@cp "$(RENDERED)" "$(AMUMSS_HOME)/ModScript/mod.lua"
	@cd "$(BUILD)/stage/mbin" && find . -name '*.MBIN' -print0 | while IFS= read -r -d '' f; do \
	  mkdir -p "$(AMUMSS_HOME)/ModScript/$$(dirname "$$f")"; \
	  cp "$$f" "$(AMUMSS_HOME)/ModScript/$$f"; \
	done
	@rm -rf "$(AMUMSS_HOME)/CreatedMODS/$(MOD_SET)"
	@AMUMSS_HOME="$(AMUMSS_HOME)" "$(AMUMSS_LINUX)/buildmod.sh" --run-pipeline
	@rm -rf "$(AMUMSS_HOME)/ModScript"
	@if [ -d "$(AMUMSS_HOME)/ModScript.makebak" ]; then mv "$(AMUMSS_HOME)/ModScript.makebak" "$(AMUMSS_HOME)/ModScript"; fi
	@trap - EXIT
	@rm -rf "$(BUILD)/$(MOD_SET)" && mkdir -p "$(BUILD)/$(MOD_SET)"
	@cp -a "$(AMUMSS_HOME)/CreatedMODS/$(MOD_SET)/." "$(BUILD)/$(MOD_SET)/"
	@touch "$(STAMP)"
	@echo "built: $(BUILD)/$(MOD_SET)"

verify: build
	@set -e; \
	if [ ! -d "$(BUILD)/$(MOD_SET)" ]; then echo "MISSING build output" >&2; exit 1; fi; \
	for f in \
	  "UI/POPUP_INVENTORY.MBIN" \
	  "UI/POPUP_TRANSFER.MBIN" \
	  "UI/POPUP_BUILD.MBIN" \
	  "UI/INVENTORYPAGE.MBIN"; do \
	  if [ ! -f "$(BUILD)/$(MOD_SET)/$$f" ]; then echo "MISSING $$f" >&2; exit 1; fi; \
	done; \
	echo "verify: OK"

clean:
	rm -rf "$(BUILD)" "$(DIST)"

# Nexus page header from the source screenshot in assets/. The generated
# header is a build artifact and is not tracked; the screenshot and the
# typeface are. (No gallery output: the bigger_/original_ screenshot pairs
# already cover the Nexus image slots.)
assets: assets/bigger_inventory.jpg assets/generate.py
	python3 assets/generate.py

# The nms-tweaks proxy DLL (xinput9_1_0.dll + default toml) for
# Binaries/. Built with the mingw cross toolchain on Linux; the same
# binary runs on Windows and under Proton.
LOADER_DIR := loader
LOADER_DIST := $(LOADER_DIR)/dist

loader:
	@cd "$(LOADER_DIR)" && cargo build --target x86_64-pc-windows-gnu --release
	@mkdir -p "$(LOADER_DIST)"
	@cp "$(LOADER_DIR)/target/x86_64-pc-windows-gnu/release/xinput9_1_0.dll" "$(LOADER_DIST)/"
	@echo "loader: $(LOADER_DIST)/xinput9_1_0.dll"

# Single install archive for Amethyst (default import, NOT Root Folder)
# and Vortex. Layout follows Amethyst's deploy rules: a flat DLL maps to
# the game Binaries, and the bare mod folder maps to GAMEDATA/MODS/.
# (A Binaries/ subdir in the zip does NOT work: Amethyst deploys it as
# Binaries/Binaries/, and a GAMEDATA/MODS prefix gets stripped and dumped
# at the game root.) The toml ships inside the mod folder; the DLL reads
# it beside itself first, then falls back to the MODS copy.
# This replaces the standalone BootSkip DLL (both provide XINPUT9_1_0.dll
# - do not run both).

rootzip: verify loader
	@rm -rf "$(BUILD)/rootzip" && mkdir -p "$(BUILD)/rootzip/$(MOD_SET)"
	@cp "$(LOADER_DIST)/xinput9_1_0.dll" "$(BUILD)/rootzip/"
	@cp "$(LOADER_DIST)/nms-tweaks.toml" "$(BUILD)/rootzip/$(MOD_SET)/"
	@cp -a "$(BUILD)/$(MOD_SET)/UI" "$(BUILD)/rootzip/$(MOD_SET)/"
	@cp "$(BUILD)/$(MOD_SET)/mod.lua" "$(BUILD)/rootzip/$(MOD_SET)/"
	@mkdir -p "$(DIST)" && rm -f "$(ROOTZIP)" && cd "$(BUILD)/rootzip" && zip -qr "$(shell pwd)/$(ROOTZIP)" xinput9_1_0.dll "$(MOD_SET)"
	@echo "packed: $(ROOTZIP) (version $(VERSION))"
	@unzip -l "$(ROOTZIP)" | tail -4
