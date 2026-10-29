# Bigger Item Grids and Smaller Inventory Slots
# Builds the loose-folder mod with the AMUMSS Linux port.
#
# What it does: patches the vanilla UI MBINs to enlarge the card grid on
# the Transfer Items and Craft Product popups, and ships smaller
# inventory slot and box templates. No game binaries live in this repo:
# all 19 UI files ship whole as MXML text in src/mxml, compiled to
# MBIN at build time.
#
# Usage:
#   make release      clean rebuild + verify + pack dist/ zip (default)
#                     the zip is a root-install archive (Binaries/ +
#                     GAMEDATA/MODS/) for Amethyst/Vortex
#   make verify       sanity-check the built outputs
#   make clean        remove build/ and dist/
#   make loader       build the Nozavolochen host DLL via the framework (dev testing)
#   make rootzip      pack the root-install archive into dist/ (no rebuild)
#   make assets       regenerate the Nexus page images (gallery + header)
#
# Override the AMUMSS install location if needed:
#   make AMUMSS_HOME=/path/to/AMUMSS release

AMUMSS_HOME ?= $(HOME)/AMUMSS
AMUMSS_LINUX ?= $(HOME)/Git/AMUMSS/linux

VERSION      := 0.3.0

MOD_SET      := Bigger Item Grids and Smaller Inventory Slots

SRC_SCRIPT   := src/mod.lua.in
SRC_MXML_DIR := src/mxml

BUILD := build
DIST  := dist
STAMP := $(BUILD)/.built

RENDERED := $(BUILD)/mod.lua

.PHONY: all release build verify clean loader rootzip assets

all: release

# Single canonical artifact: the root-install zip (same name the mod
# manager shows). The old mod-folder-only zip is gone; its contents live
# under GAMEDATA/MODS/ inside this archive.
ROOTZIP := $(DIST)/$(MOD_SET).zip

release: clean build verify rootzip
	@echo "release: $(ROOTZIP) (version $(VERSION))"

# The script needs no rendering; stage a copy so the build never
# reads straight from src.
$(RENDERED): $(SRC_SCRIPT)
	@mkdir -p "$(BUILD)"
	@cp "$(SRC_SCRIPT)" "$(RENDERED)"

build: $(RENDERED)
	@test -d "$(AMUMSS_HOME)/MODBUILDER" || (echo "ERROR: no AMUMSS install at $(AMUMSS_HOME)" >&2; exit 1)
	@test -x "$(AMUMSS_HOME)/MODBUILDER/MBINCompiler-linux" || (echo "ERROR: MBINCompiler-linux missing" >&2; exit 1)
	@rm -rf "$(AMUMSS_HOME)/MODBUILDER/MOD" && mkdir -p "$(AMUMSS_HOME)/MODBUILDER/MOD"
	@if [ -d "$(AMUMSS_HOME)/ModScript" ]; then mv "$(AMUMSS_HOME)/ModScript" "$(AMUMSS_HOME)/ModScript.makebak"; trap 'rm -rf "$(AMUMSS_HOME)/ModScript"; mv "$(AMUMSS_HOME)/ModScript.makebak" "$(AMUMSS_HOME)/ModScript"' EXIT; fi
	@mkdir -p "$(AMUMSS_HOME)/ModScript"
	@cp "$(RENDERED)" "$(AMUMSS_HOME)/ModScript/mod.lua"
	@mkdir -p "$(AMUMSS_HOME)/ModScript/MXML"
	@cp "$(SRC_MXML_DIR)"/*.MXML "$(AMUMSS_HOME)/ModScript/MXML/"
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
	if [ ! -d "$(BUILD)/$(MOD_SET)" ]; then echo "MISSING build output" >&2; exit 1; fi
	@python3 src/verify_build.py --build "$(BUILD)" --mod-set "$(MOD_SET)" \
		--mxml "$(SRC_MXML_DIR)" \
		--mbincompiler "$(AMUMSS_HOME)/MODBUILDER/MBINCompiler-linux"

clean:
	rm -rf "$(BUILD)" "$(DIST)"

# Nexus page header from the source screenshot in assets/. The generated
# header is a build artifact and is not tracked; the screenshot and the
# typeface are. (No gallery output: the bigger_/original_ screenshot pairs
# already cover the Nexus image slots.)
assets: assets/bigger_inventory.jpg assets/generate.py
	python3 assets/generate.py

# The Nozavolochen host DLL (xinput9_1_0.dll) is a separate required
# download (the framework Nexus file); this mod ships only its mod
# spec plus static UI. The loader target below is for local dev testing
# only. Override the framework checkout location if needed:
#   make FRAMEWORK_HOME=/path/to/nozavolochen/framework loader
FRAMEWORK_HOME ?= /home/savagecore/Modding/NMS/nozavolochen/framework
FRAMEWORK_DIST := $(FRAMEWORK_HOME)/loader/dist

DLL := $(FRAMEWORK_DIST)/xinput9_1_0.dll

loader:
	@$(MAKE) -C "$(FRAMEWORK_HOME)" loader
	@echo "loader: $(DLL)"

# Single install archive for Amethyst (default import, NOT Root Folder)
# and Vortex: the bare mod folder maps to GAMEDATA/MODS/. The host DLL
# is NOT bundled; install the Nozavolochen Framework file (required
# dependency) for Binaries/xinput9_1_0.dll.
# (A Binaries/ subdir in the zip does NOT work: Amethyst deploys it as
# Binaries/Binaries/, and a GAMEDATA/MODS prefix gets stripped and dumped
# at the game root.) The mod.toml ships inside the mod folder; the DLL
# discovers it there at boot alongside the other installed mods.

rootzip: verify
	@rm -rf "$(BUILD)/rootzip" && mkdir -p "$(BUILD)/rootzip/$(MOD_SET)"
	@cp mod.toml "$(BUILD)/rootzip/$(MOD_SET)/"
	@cp -a "$(BUILD)/$(MOD_SET)/UI" "$(BUILD)/rootzip/$(MOD_SET)/"
	@cp "$(BUILD)/$(MOD_SET)/mod.lua" "$(BUILD)/rootzip/$(MOD_SET)/"
	@mkdir -p "$(DIST)" && rm -f "$(ROOTZIP)" && cd "$(BUILD)/rootzip" && zip -qr "$(shell pwd)/$(ROOTZIP)" "$(MOD_SET)"
	@echo "packed: $(ROOTZIP) (version $(VERSION))"
	@unzip -l "$(ROOTZIP)" | tail -4
