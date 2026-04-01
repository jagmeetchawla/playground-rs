#!/usr/bin/env python3
"""
Generate a 1024x1024 app icon for Rustic Playground.

Design:
  - Warm dark background (#1A1815)
  - Large rust-orange "RS" in Arial Black, centred slightly above mid
  - Isometric crate outline below, in white at low opacity
  - Subtle inner vignette for depth
"""

from PIL import Image, ImageDraw, ImageFont, ImageFilter
import math, os

SIZE   = 1024
OUT    = os.path.join(os.path.dirname(__file__), "src-tauri", "icons", "app-icon-source.png")

# ── Colours ──────────────────────────────────────────────────────────────────
BG          = (26, 24, 21)          # warm near-black
RUST        = (220, 100, 30)        # rust orange
RUST_GLOW   = (220, 100, 30, 60)    # soft behind-text glow
WHITE       = (255, 255, 255)
CRATE_CLR   = (255, 255, 255, 110)  # crate lines — white, semi-transparent

FONT_PATH   = "/System/Library/Fonts/Supplemental/Arial Black.ttf"
FONT_SM     = "/System/Library/Fonts/Supplemental/Arial.ttf"

# ── Canvas ───────────────────────────────────────────────────────────────────
img  = Image.new("RGBA", (SIZE, SIZE), BG + (255,))
draw = ImageDraw.Draw(img)

cx, cy = SIZE // 2, SIZE // 2

# ── Vignette — slightly darken corners ───────────────────────────────────────
vignette = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
vd = ImageDraw.Draw(vignette)
steps = 18
for i in range(steps):
    t       = i / steps
    alpha   = int(120 * (t ** 1.6))
    margin  = int(SIZE * 0.5 * (1 - t))
    vd.ellipse([margin, margin, SIZE - margin, SIZE - margin],
               fill=(0, 0, 0, alpha))
vignette = vignette.filter(ImageFilter.GaussianBlur(radius=80))
img = Image.alpha_composite(img, vignette)
draw = ImageDraw.Draw(img)

# ── Soft orange glow behind "RS" ─────────────────────────────────────────────
glow_layer = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
gd = ImageDraw.Draw(glow_layer)
gd.ellipse([cx - 300, cy - 340, cx + 300, cy + 100], fill=RUST_GLOW)
glow_layer = glow_layer.filter(ImageFilter.GaussianBlur(radius=90))
img = Image.alpha_composite(img, glow_layer)
draw = ImageDraw.Draw(img)

# ── "RS" text ─────────────────────────────────────────────────────────────────
font_size = 520
font = ImageFont.truetype(FONT_PATH, font_size)
text = "RS"

bbox   = draw.textbbox((0, 0), text, font=font)
tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]

# Optical centre — shift up slightly so crate has room below
text_x = cx - tw // 2 - bbox[0]
text_y = cy - th // 2 - bbox[1] - 90

draw.text((text_x, text_y), text, font=font, fill=RUST)

# ── Isometric crate (below "RS") ─────────────────────────────────────────────
# Six points of an isometric box:
#   Top face:    top, right, bottom-mid, left  (rhombus)
#   Left face:   bottom-mid, left, bottom-left, lower-left
#   Right face:  bottom-mid, right, bottom-right, lower-right

def iso(cx, cy, w, h):
    """Return (top, right, mid, left, bl, br) for an isometric box."""
    half_w = w / 2
    half_h = h / 4     # top face height
    face_h = h / 2     # side face height

    top   = (cx,              cy - half_h)
    right = (cx + half_w,     cy)
    mid   = (cx,              cy + half_h)
    left  = (cx - half_w,     cy)
    bl    = (cx - half_w,     cy + half_h + face_h)
    br    = (cx + half_w,     cy + half_h + face_h)
    bot   = (cx,              cy + half_h * 2 + face_h)
    return top, right, mid, left, bl, br, bot

# Position the crate centred below the RS letters, scaled to taste
crate_cx = cx
crate_cy = text_y + th + 105
W, H = 240, 200

top, right, mid, left, bl, br, bot = iso(crate_cx, crate_cy, W, H)

lw = 6   # line width

# Build a separate RGBA layer so we can use the alpha in CRATE_CLR
crate_layer = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
cl = ImageDraw.Draw(crate_layer)

def line(a, b):
    cl.line([a, b], fill=CRATE_CLR, width=lw)

# Top face (rhombus)
line(top, right)
line(right, mid)
line(mid, left)
line(left, top)

# Left face
line(left, bl)
line(bl, bot)
line(bot, mid)

# Right face
line(right, br)
line(br, bot)

img = Image.alpha_composite(img, crate_layer)
draw = ImageDraw.Draw(img)

# ── "rustic playground" label at the bottom ───────────────────────────────────
label_font_size = 62
try:
    lf = ImageFont.truetype(FONT_SM, label_font_size)
except Exception:
    lf = ImageFont.load_default()

label = "rustic playground"
lb = draw.textbbox((0, 0), label, font=lf)
lw2, lh2 = lb[2] - lb[0], lb[3] - lb[1]
lx = cx - lw2 // 2 - lb[0]
ly = SIZE - lh2 - 72

draw.text((lx, ly), label, font=lf,
          fill=(255, 255, 255, 120))   # subtle white

# ── Save ─────────────────────────────────────────────────────────────────────
# Convert back to RGB for the source PNG (tauri icon command prefers opaque)
final = Image.new("RGB", (SIZE, SIZE), BG)
final.paste(img.convert("RGB"), (0, 0))
final.save(OUT)
print(f"✓ saved {OUT}")
