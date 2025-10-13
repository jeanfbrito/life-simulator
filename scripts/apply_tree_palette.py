#!/usr/bin/env python3
"""
Apply stone-kingdoms ColorTable palette to tree textures.

This script implements the same palette lookup algorithm used by stone-kingdoms shader:
- Tree textures use red+green channels as 2D coordinates (not simple palette index)
- ColorTable is a 320×80 2D lookup table
- Algorithm: x = (red / 8) * 10, y = (green / 8) * 10, then fetch ColorTable[x, y]
"""

import sys
from PIL import Image
import os

def red_green_to_position(red, green):
    """Convert red/green values (0-255) to ColorTable position.

    Based on stone-kingdoms shader algorithm:
        int redIndex = int(floor(redValue * 255.0));
        int x = (redIndex / 8) * 10;
    """
    red_index = int(red)
    x = (red_index // 8) * 10

    green_index = int(green)
    y = (green_index // 8) * 10

    return (x, y)

def apply_palette(tree_image_path, color_table_path, output_path):
    """Apply ColorTable palette to tree texture.

    Args:
        tree_image_path: Path to tree PNG (uses red+green for coords)
        color_table_path: Path to ColorTable PNG (320×80)
        output_path: Where to save the result
    """
    print(f"Loading tree texture: {tree_image_path}")
    tree_img = Image.open(tree_image_path).convert("RGBA")

    print(f"Loading color table: {color_table_path}")
    color_table = Image.open(color_table_path).convert("RGBA")

    # Verify ColorTable dimensions
    if color_table.size[0] != 320 or color_table.size[1] != 80:
        print(f"Warning: ColorTable size is {color_table.size}, expected (320, 80)")

    # Create output image
    output_img = Image.new("RGBA", tree_img.size, (0, 0, 0, 0))

    tree_pixels = tree_img.load()
    color_table_pixels = color_table.load()
    output_pixels = output_img.load()

    width, height = tree_img.size
    pixels_processed = 0
    pixels_skipped = 0

    print(f"Processing {width}×{height} pixels...")

    for y in range(height):
        for x in range(width):
            r, g, b, a = tree_pixels[x, y]

            # If transparent, skip
            if a < 255:
                output_pixels[x, y] = (0, 0, 0, 0)
                pixels_skipped += 1
                continue

            # Calculate ColorTable coordinates from red+green
            ct_x, ct_y = red_green_to_position(r, g)

            # Check if (0, 0) → black in shader logic
            if ct_x == 0 and ct_y == 0:
                output_pixels[x, y] = (0, 0, 0, 0)
                pixels_skipped += 1
                continue

            # Clamp to ColorTable bounds
            ct_x = min(ct_x, 319)
            ct_y = min(ct_y, 79)

            # Fetch color from ColorTable
            final_r, final_g, final_b, final_a = color_table_pixels[ct_x, ct_y]

            # Write to output (preserve original alpha)
            output_pixels[x, y] = (final_r, final_g, final_b, a)
            pixels_processed += 1

    print(f"Processed: {pixels_processed} pixels")
    print(f"Skipped (transparent/black): {pixels_skipped} pixels")
    print(f"Saving to: {output_path}")

    output_img.save(output_path, "PNG")
    print("✅ Done!")

if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python3 apply_tree_palette.py <tree.png> <ColorTable.png> <output.png>")
        print()
        print("Example:")
        print("  python3 apply_tree_palette.py tree_pine_large_01.png ColorTable1.png tree_pine_green_01.png")
        sys.exit(1)

    tree_path = sys.argv[1]
    palette_path = sys.argv[2]
    output_path = sys.argv[3]

    if not os.path.exists(tree_path):
        print(f"Error: Tree file not found: {tree_path}")
        sys.exit(1)

    if not os.path.exists(palette_path):
        print(f"Error: ColorTable file not found: {palette_path}")
        sys.exit(1)

    apply_palette(tree_path, palette_path, output_path)
