#!/usr/bin/env python3
"""
Generate the diagonal-slice composite hero image from three theme screenshots.

Takes three screenshots of the SAME app window (same size, same content) in
different themes, and composites them into a single image where each theme
occupies a diagonal slice — left=Dark, middle=Rust, right=Light. The slices
blend seamlessly so it looks like one window transitioning across themes.

Matches the style of docs/images/screenshot-themes.png.

Input (in docs/images/hero/):
  hero-dark.png   — full app window in Dark theme
  hero-rust.png   — full app window in Rust theme
  hero-light.png  — full app window in Light theme

Output:
  docs/images/hero-composite.png

Usage: python3 scripts/build-hero-composite.py
"""

import os
import sys
from PIL import Image

IMAGES_DIR = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "docs", "images"
)
HERO_DIR = os.path.join(IMAGES_DIR, "hero")

# Diagonal slice angle: how many pixels the cut shifts vertically across
# the full image width. Higher = steeper diagonal. 0 = vertical slices.
SKEW = 0.15  # 15% of height


def main():
    names = ["hero-dark.png", "hero-rust.png", "hero-light.png"]
    paths = [os.path.join(HERO_DIR, n) for n in names]

    for p in paths:
        if not os.path.exists(p):
            print(f"Missing: {p}")
            print("Take screenshots first: hero-dark.png, hero-rust.png, hero-light.png")
            print("Place them in docs/images/hero/")
            sys.exit(1)

    imgs = [Image.open(p).convert("RGBA") for p in paths]

    # All images must be the same size
    w, h = imgs[0].size
    for img in imgs[1:]:
        if img.size != (w, h):
            print(f"Error: all images must be the same size. Got {img.size}, expected ({w}, {h})")
            sys.exit(1)

    # Create output image
    out = Image.new("RGBA", (w, h))

    # Calculate diagonal cut positions
    # Two cuts divide the image into three slices (left, middle, right)
    # Each cut is a diagonal line from top to bottom
    skew_px = int(h * SKEW)  # horizontal shift from top to bottom

    # Cut 1: ~1/3 from left (top) to ~1/3 - skew (bottom)
    cut1_top = w // 3
    cut1_bot = cut1_top - skew_px

    # Cut 2: ~2/3 from left (top) to ~2/3 - skew (bottom)
    cut2_top = 2 * w // 3
    cut2_bot = cut2_top - skew_px

    # For each pixel row, determine which image to use based on x position
    dark_px = imgs[0].load()
    rust_px = imgs[1].load()
    light_px = imgs[2].load()
    out_px = out.load()

    for y in range(h):
        # Interpolate cut positions for this row
        t = y / max(h - 1, 1)  # 0 at top, 1 at bottom
        c1 = int(cut1_top * (1 - t) + cut1_bot * t)
        c2 = int(cut2_top * (1 - t) + cut2_bot * t)

        for x in range(w):
            if x < c1:
                out_px[x, y] = dark_px[x, y]
            elif x < c2:
                out_px[x, y] = rust_px[x, y]
            else:
                out_px[x, y] = light_px[x, y]

    out_path = os.path.join(IMAGES_DIR, "hero-composite.png")
    out.save(out_path, "PNG", optimize=True)
    size_kb = os.path.getsize(out_path) // 1024
    print(f"Wrote {out_path} ({w}x{h}, {size_kb}KB)")


if __name__ == "__main__":
    main()
