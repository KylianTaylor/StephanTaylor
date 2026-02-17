#!/usr/bin/env python3
"""
Nimbuzyn Icon Generator
Generates a 3D orange 'N' logo on a dark background in all Android mipmap sizes.
"""

import math
from PIL import Image, ImageDraw, ImageFilter
import numpy as np

def draw_3d_N_icon(size: int, with_shadow: bool = True) -> Image.Image:
    """Draw the Nimbuzyn icon: 3D orange N on deep dark background."""
    
    scale = size / 192
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # ── Background circle / rounded square ───────────────────────────────
    pad = int(size * 0.04)
    radius = int(size * 0.22)
    
    # Deep dark gradient background
    bg_layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg_layer)
    
    # Draw gradient background using circles from center
    cx, cy = size // 2, size // 2
    for r in range(max(cx, cy), 0, -1):
        t = r / max(cx, cy)
        # Dark navy to near-black
        rc = int(10 + t * 18)
        gc = int(12 + t * 16)
        bc = int(20 + t * 28)
        bg_draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(rc, gc, bc, 255))
    
    # Clip to rounded rectangle
    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle([pad, pad, size - pad, size - pad], radius=radius, fill=255)
    
    bg_out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    bg_out.paste(bg_layer, mask=mask)
    img = Image.alpha_composite(img, bg_out)
    draw = ImageDraw.Draw(img)
    
    # Subtle inner glow ring
    ring_layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ring_draw = ImageDraw.Draw(ring_layer)
    for t_i in range(6):
        t = t_i / 5
        alpha = int(25 * (1 - t))
        offset = int(t_i * 3 * scale)
        ring_draw.rounded_rectangle(
            [pad + offset, pad + offset, size - pad - offset, size - pad - offset],
            radius=max(4, radius - offset),
            outline=(255, 140, 0, alpha),
            width=max(1, int(2 * scale)),
        )
    img = Image.alpha_composite(img, ring_layer)
    draw = ImageDraw.Draw(img)
    
    # ── 3D "N" construction ───────────────────────────────────────────────
    # Letter dimensions
    letter_w = int(size * 0.56)
    letter_h = int(size * 0.62)
    lx = (size - letter_w) // 2
    ly = (size - letter_h) // 2 - int(size * 0.02)
    
    stroke = int(letter_w * 0.22)   # stroke thickness
    depth  = int(size * 0.04)        # 3D extrusion depth
    
    # Color palette
    orange_top   = (255, 145, 20)    # bright face
    orange_mid   = (230, 110, 5)     # slightly dimmer
    orange_dark  = (180, 75, 0)      # shadow face (extrusion side)
    orange_deep  = (120, 50, 0)      # deepest shadow
    highlight    = (255, 200, 80)    # specular highlight
    
    def rect(draw, x0, y0, x1, y1, color, alpha=255):
        c = color + (alpha,) if len(color) == 3 else color
        draw.rectangle([x0, y0, x1, y1], fill=c)
    
    def poly(draw, pts, color, alpha=255):
        c = color + (alpha,) if len(color) == 3 else color
        draw.polygon(pts, fill=c)
    
    # ── Draw 3D extrusion first (back layer) ──────────────────────────────
    N_layer_back = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    nb_draw = ImageDraw.Draw(N_layer_back)
    
    dx, dy = depth, depth  # extrusion direction (bottom-right)
    
    # Left vertical bar – right extrusion face
    poly(nb_draw,
         [(lx + stroke, ly + dy),
          (lx + stroke + dx, ly),
          (lx + stroke + dx, ly + letter_h),
          (lx + stroke, ly + letter_h + dy)],
         orange_dark)
    
    # Right vertical bar – right extrusion face
    poly(nb_draw,
         [(lx + letter_w - stroke, ly + dy),
          (lx + letter_w - stroke + dx, ly),
          (lx + letter_w + dx, ly),
          (lx + letter_w, ly + dy)],
         orange_dark)
    poly(nb_draw,
         [(lx + letter_w, ly + dy),
          (lx + letter_w + dx, ly),
          (lx + letter_w + dx, ly + letter_h),
          (lx + letter_w, ly + letter_h + dy)],
         orange_dark)
    
    # Diagonal – bottom extrusion face
    diag_x0 = lx + stroke
    diag_y0 = ly + int(stroke * 0.5)
    diag_x1 = lx + letter_w - stroke
    diag_y1 = ly + letter_h - int(stroke * 0.5)
    
    poly(nb_draw,
         [(diag_x0, diag_y0 + dy),
          (diag_x0 + dx, diag_y0),
          (diag_x0 + stroke + dx, diag_y0),
          (diag_x0 + stroke, diag_y0 + dy)],
         orange_deep)
    
    poly(nb_draw,
         [(diag_x1 - stroke, diag_y1 + dy),
          (diag_x1 - stroke + dx, diag_y1),
          (diag_x1 + dx, diag_y1),
          (diag_x1, diag_y1 + dy)],
         orange_deep)
    
    # Bottom extrusion of bars
    rect(nb_draw, lx, ly + letter_h, lx + stroke + dx, ly + letter_h + dy, orange_dark)
    rect(nb_draw, lx + letter_w - stroke, ly + letter_h,
         lx + letter_w + dx, ly + letter_h + dy, orange_dark)
    
    img = Image.alpha_composite(img, N_layer_back)
    
    # ── Draw front face of the N ───────────────────────────────────────────
    N_layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    n_draw = ImageDraw.Draw(N_layer)
    
    # Left vertical bar
    rect(n_draw, lx, ly, lx + stroke, ly + letter_h, orange_mid)
    
    # Right vertical bar
    rect(n_draw, lx + letter_w - stroke, ly, lx + letter_w, ly + letter_h, orange_mid)
    
    # Diagonal stroke (top-left to bottom-right)
    # Render as a parallelogram
    diag_pts = [
        (diag_x0, diag_y0),
        (diag_x0 + stroke, diag_y0),
        (diag_x1, diag_y1),
        (diag_x1 - stroke, diag_y1),
    ]
    poly(n_draw, diag_pts, orange_mid)
    
    img = Image.alpha_composite(img, N_layer)
    draw = ImageDraw.Draw(img)
    
    # ── Highlights (specular) ─────────────────────────────────────────────
    hl_layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    hl_draw = ImageDraw.Draw(hl_layer)
    
    hl_w = max(2, int(stroke * 0.18))
    # Left bar top-left edge highlight
    hl_draw.rectangle(
        [lx, ly, lx + hl_w, ly + letter_h],
        fill=highlight + (180,)
    )
    # Left bar top edge
    hl_draw.rectangle(
        [lx, ly, lx + stroke, ly + hl_w],
        fill=highlight + (160,)
    )
    # Right bar top edge
    hl_draw.rectangle(
        [lx + letter_w - stroke, ly, lx + letter_w, ly + hl_w],
        fill=highlight + (130,)
    )
    
    img = Image.alpha_composite(img, hl_layer)
    
    # ── Soft drop shadow ──────────────────────────────────────────────────
    if with_shadow and size >= 48:
        shadow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        sh_draw = ImageDraw.Draw(shadow)
        blur_r = max(4, int(size * 0.05))
        sh_draw.rounded_rectangle(
            [pad + 3, pad + 3, size - pad + 3, size - pad + 3],
            radius=radius,
            fill=(0, 0, 0, 100)
        )
        shadow = shadow.filter(ImageFilter.GaussianBlur(blur_r))
        base = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        base = Image.alpha_composite(base, shadow)
        base = Image.alpha_composite(base, img)
        img = base
    
    return img


def generate_all_icons(base_path: str):
    """Generate all required Android mipmap icon sizes."""
    sizes = {
        "mdpi":    48,
        "hdpi":    72,
        "xhdpi":   96,
        "xxhdpi":  144,
        "xxxhdpi": 192,
    }
    
    print("Generando iconos Nimbuzyn…")
    for density, size in sizes.items():
        icon = draw_3d_N_icon(size)
        path = f"{base_path}/res/mipmap-{density}/ic_launcher.png"
        icon.save(path, "PNG", optimize=True)
        
        # Round icon (circle clip)
        round_icon = icon.copy()
        mask = Image.new("L", (size, size), 0)
        mask_draw = ImageDraw.Draw(mask)
        mask_draw.ellipse([0, 0, size, size], fill=255)
        round_icon.putalpha(mask)
        round_path = f"{base_path}/res/mipmap-{density}/ic_launcher_round.png"
        round_icon.save(round_path, "PNG", optimize=True)
        
        print(f"  ✓ {density} ({size}x{size}px) → {path}")
    
    # Also generate a large version for the splash screen
    splash_icon = draw_3d_N_icon(512, with_shadow=True)
    splash_path = f"{base_path}/assets/splash_icon.png"
    splash_icon.save(splash_path, "PNG", optimize=True)
    print(f"  ✓ splash (512x512px) → {splash_path}")
    
    # Feature graphic (1024x500) for Play Store
    feature = Image.new("RGBA", (1024, 500), (11, 14, 22, 255))
    feat_draw = ImageDraw.Draw(feature)
    
    # Background gradient
    for y in range(500):
        t = y / 500
        r = int(11 + t * 8)
        g = int(14 + t * 6)
        b = int(22 + t * 12)
        feat_draw.line([(0, y), (1024, y)], fill=(r, g, b, 255))
    
    # Place centered icon
    icon_512 = draw_3d_N_icon(220)
    ix = (1024 - 220) // 2
    iy = (500 - 220) // 2 - 20
    feature.alpha_composite(icon_512, (ix, iy))
    
    # App name text (drawn as simple rectangles representing letters)
    feat_draw2 = ImageDraw.Draw(feature)
    # Title bar below icon
    text_y = iy + 240
    feat_draw2.text = None  # no font needed, use block
    
    feature.save(f"{base_path}/assets/feature_graphic.png", "PNG")
    print(f"  ✓ feature graphic → {base_path}/assets/feature_graphic.png")

    print("\n✅ Todos los iconos generados correctamente.")


if __name__ == "__main__":
    import sys
    base = sys.argv[1] if len(sys.argv) > 1 else "/home/claude/nimbuzyn"
    generate_all_icons(base)
