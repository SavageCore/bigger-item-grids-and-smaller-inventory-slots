#!/usr/bin/env python3
"""Regenerate the Nexus page header from the source screenshot.

  assets/header-1300x372.jpg    (wide crop + title overlay, banner)

The source is the Exosuit screen with the scaled grids: 7 cargo rows and
4 tech rows on the left, the exosuit on the right. The header is a band
cropped out of the middle and cut short on the right, so the inventory
sits right of the title and the title has a darkened left edge to sit
on. (No gallery output: the bigger_/original_ screenshot pairs already
cover the Nexus image slots.)

Usage: python3 assets/generate.py   (or: make assets)
Requires: Pillow (pip install Pillow). Typeface is vendored (see FONT).
"""
import os

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "bigger_inventory.jpg")
HDR = os.path.join(HERE, "header-1300x372.jpg")

# In-game heading typeface, vendored (SIL OFL 1.1):
# https://github.com/NMSCD/No-Mans-Sky-Universal-Font
FONT = os.path.join(HERE, "fonts", "GeosansLight-NMS.ttf")

TITLE = ["BIGGER ITEM GRIDS", "SMALLER INVENTORY SLOTS"]
SUBTITLE = "7 ROWS OF 10  \u2022  24 SLOTS PER PAGE"

# Header crop, in source pixels. The width is what makes the inventory land
# right of the title, so it is derived from the source rather than fixed.
HDR_X = 0
HDR_Y = 400
HDR_W = 2133

# Header size in output pixels.
HDR_OUT_W = 1300
HDR_OUT_H = 372

# Title placement in the header.
TITLE_X = 48
TITLE_Y = 48
TITLE_SIZE = 50
TITLE_GAP = 62
SUBTITLE_SIZE = 30


def main():
    img = Image.open(SRC).convert("RGB")
    assert img.size[0] * 9 == img.size[1] * 16, f"source is not 16:9: {img.size}"
    assert img.size[0] >= HDR_W, f"source too narrow for the header crop: {img.size}"

    # The band height follows from the crop width, so the crop keeps the
    # banner aspect no matter how it is retuned.
    band_h = int(HDR_W * HDR_OUT_H / HDR_OUT_W)
    band = img.crop((HDR_X, HDR_Y, HDR_X + HDR_W, HDR_Y + band_h))
    band = band.resize((HDR_OUT_W, HDR_OUT_H), Image.LANCZOS)
    d = ImageDraw.Draw(band, "RGBA")
    # left-weighted gradient: title sits on the left, Nexus overlays its own
    # title bottom-left
    for x in range(HDR_OUT_W):
        a = int(205 * max(0.0, 1 - x / (HDR_OUT_W * 0.62)) ** 1.5)
        d.rectangle([x, 0, x + 1, HDR_OUT_H], fill=(0, 0, 0, a))
    band = band.convert("RGB")
    d = ImageDraw.Draw(band)
    fb = ImageFont.truetype(FONT, TITLE_SIZE)
    fs = ImageFont.truetype(FONT, SUBTITLE_SIZE)

    def text_shadowed(xy, s, font, fill):
        x, y = xy
        for dx, dy in ((-2, 0), (2, 0), (0, -2), (0, 2), (-2, -2), (2, -2), (-2, 2), (2, 2)):
            d.text((x + dx, y + dy), s, font=font, fill=(0, 0, 0))
        d.text((x, y), s, font=font, fill=fill)

    for i, line in enumerate(TITLE):
        text_shadowed(
            (TITLE_X, TITLE_Y + i * TITLE_GAP), line, fb, (255, 255, 255)
        )
    text_shadowed(
        (TITLE_X + 2, TITLE_Y + len(TITLE) * TITLE_GAP + 8),
        SUBTITLE,
        fs,
        (255, 205, 130),
    )
    band.save(HDR, quality=92)
    print(f"wrote {HDR}")


if __name__ == "__main__":
    main()
