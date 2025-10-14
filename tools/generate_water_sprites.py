#!/usr/bin/env python3
"""
Generate RCT2-style water sprites programmatically.

OpenRCT2 water system:
- SPR_WATER_MASK (5048-5052): 5 base water sprites
- SPR_WATER_OVERLAY (5053-5057): 5 overlay sprites (animated)

Water sprite mapping based on terrain slope:
Byte97B740[] = {0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 0, 1, 4, 0}

For now, generates flat water sprites that can be enhanced later.
"""

from PIL import Image, ImageDraw
import os

# Output directory
OUTPUT_DIR = os.path.expanduser("~/Github/life-simulator/godot-viewer/assets/tiles/water/rct2")

# RCT2 isometric tile dimensions (2:1 ratio)
TILE_WIDTH = 64
TILE_HEIGHT = 31  # Actual RCT2 water sprite height

# Water colors (RGBA)
WATER_BASE = (58, 124, 165, 255)      # Base water blue
WATER_HIGHLIGHT = (82, 161, 203, 255)  # Lighter blue for highlights
WATER_SHADOW = (41, 87, 115, 255)      # Darker blue for depth

def create_diamond_mask():
    """Create isometric diamond mask for 64×31 tile."""
    mask = Image.new('L', (TILE_WIDTH, TILE_HEIGHT), 0)
    draw = ImageDraw.Draw(mask)

    # Isometric diamond points
    center_x = TILE_WIDTH // 2
    top = (center_x, 0)
    right = (TILE_WIDTH - 1, TILE_HEIGHT // 2)
    bottom = (center_x, TILE_HEIGHT - 1)
    left = (0, TILE_HEIGHT // 2)

    # Draw filled diamond
    draw.polygon([top, right, bottom, left], fill=255)

    return mask


def create_flat_water():
    """Create flat water sprite (index 0) - most common."""
    img = Image.new('RGBA', (TILE_WIDTH, TILE_HEIGHT), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Get diamond mask
    mask = create_diamond_mask()

    # Fill with base water color
    for y in range(TILE_HEIGHT):
        for x in range(TILE_WIDTH):
            if mask.getpixel((x, y)) > 0:
                # Add subtle gradient for depth
                depth_factor = y / TILE_HEIGHT
                r = int(WATER_BASE[0] + (WATER_SHADOW[0] - WATER_BASE[0]) * depth_factor)
                g = int(WATER_BASE[1] + (WATER_SHADOW[1] - WATER_BASE[1]) * depth_factor)
                b = int(WATER_BASE[2] + (WATER_SHADOW[2] - WATER_BASE[2]) * depth_factor)
                img.putpixel((x, y), (r, g, b, 255))

    # Add horizontal highlight lines (water ripples)
    for y in [8, 16, 24]:
        for x in range(TILE_WIDTH):
            if mask.getpixel((x, y)) > 0:
                img.putpixel((x, y), WATER_HIGHLIGHT)

    return img


def create_valley_water():
    """Create valley water sprite (index 1) - for NS valleys."""
    img = create_flat_water()
    draw = ImageDraw.Draw(img)
    mask = create_diamond_mask()

    # Add vertical shading for valley effect
    center_x = TILE_WIDTH // 2
    for y in range(TILE_HEIGHT):
        for x in range(center_x - 4, center_x + 4):
            if 0 <= x < TILE_WIDTH and mask.getpixel((x, y)) > 0:
                # Darken center for valley
                current = img.getpixel((x, y))
                darker = tuple(max(0, c - 20) for c in current[:3]) + (255,)
                img.putpixel((x, y), darker)

    return img


def create_slope_water_a():
    """Create slope water type A (index 2) - for specific slopes."""
    img = create_flat_water()
    draw = ImageDraw.Draw(img)
    mask = create_diamond_mask()

    # Add diagonal gradient (NE to SW)
    for y in range(TILE_HEIGHT):
        for x in range(TILE_WIDTH):
            if mask.getpixel((x, y)) > 0:
                # Diagonal factor
                diag_factor = (x + y) / (TILE_WIDTH + TILE_HEIGHT)
                current = img.getpixel((x, y))

                # Adjust brightness
                factor = 0.8 + (diag_factor * 0.4)
                adjusted = tuple(int(c * factor) for c in current[:3]) + (255,)
                img.putpixel((x, y), adjusted)

    return img


def create_slope_water_b():
    """Create slope water type B (index 3) - for specific slopes."""
    img = create_flat_water()
    draw = ImageDraw.Draw(img)
    mask = create_diamond_mask()

    # Add diagonal gradient (NW to SE)
    for y in range(TILE_HEIGHT):
        for x in range(TILE_WIDTH):
            if mask.getpixel((x, y)) > 0:
                # Opposite diagonal factor
                diag_factor = (TILE_WIDTH - x + y) / (TILE_WIDTH + TILE_HEIGHT)
                current = img.getpixel((x, y))

                # Adjust brightness
                factor = 0.8 + (diag_factor * 0.4)
                adjusted = tuple(int(c * factor) for c in current[:3]) + (255,)
                img.putpixel((x, y), adjusted)

    return img


def create_slope_water_c():
    """Create slope water type C (index 4) - for specific slopes."""
    img = create_flat_water()
    draw = ImageDraw.Draw(img)
    mask = create_diamond_mask()

    # Add EW valley shading
    for y in range(TILE_HEIGHT):
        mid_y = TILE_HEIGHT // 2
        distance = abs(y - mid_y)
        for x in range(TILE_WIDTH):
            if mask.getpixel((x, y)) > 0:
                # Darken center for EW valley
                current = img.getpixel((x, y))
                darken = int(20 * (1 - distance / mid_y))
                darker = tuple(max(0, c - darken) for c in current[:3]) + (255,)
                img.putpixel((x, y), darker)

    return img


def create_water_overlay(frame=0):
    """Create animated water overlay (transparent with ripple effect)."""
    img = Image.new('RGBA', (TILE_WIDTH, TILE_HEIGHT), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    mask = create_diamond_mask()

    # Animated ripple lines (shift based on frame)
    ripple_y = [6 + frame * 2, 14 + frame * 2, 22 + frame * 2]

    for y in ripple_y:
        if 0 <= y < TILE_HEIGHT:
            for x in range(TILE_WIDTH):
                if mask.getpixel((x, y)) > 0:
                    # Semi-transparent white ripple
                    img.putpixel((x, y), (255, 255, 255, 80))

    return img


def main():
    """Generate all water sprites."""
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    print("🌊 Generating RCT2-style water sprites...")

    # Generate 5 water mask variations
    water_sprites = [
        ("water_mask_00.png", create_flat_water()),
        ("water_mask_01.png", create_valley_water()),
        ("water_mask_02.png", create_slope_water_a()),
        ("water_mask_03.png", create_slope_water_b()),
        ("water_mask_04.png", create_slope_water_c()),
    ]

    for filename, sprite in water_sprites:
        path = os.path.join(OUTPUT_DIR, filename)
        sprite.save(path)
        print(f"  ✅ Created {filename}")

    # Generate 5 overlay frames (for animation)
    for frame in range(5):
        filename = f"water_overlay_{frame:02d}.png"
        overlay = create_water_overlay(frame)
        path = os.path.join(OUTPUT_DIR, filename)
        overlay.save(path)
        print(f"  ✅ Created {filename}")

    print(f"\n✅ Generated {len(water_sprites) + 5} water sprites in {OUTPUT_DIR}")
    print("\nWater sprite mapping (Byte97B740 lookup):")
    print("  Slope 0-6, 8-10, 12, 15: Flat water (mask 0)")
    print("  Slope 13: Valley water (mask 1)")
    print("  Slope 7: Slope water A (mask 2)")
    print("  Slope 11: Slope water B (mask 3)")
    print("  Slope 14: Slope water C (mask 4)")


if __name__ == "__main__":
    main()
