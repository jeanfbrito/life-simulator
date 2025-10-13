#!/usr/bin/env python3
"""
Extract RCT2 palette from g1.dat
Based on OpenRCT2's LoadPalette() function
"""

import struct

g1dat_path = "/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/Data/g1.dat"

print("🎨 Extracting RCT2 palette from g1.dat...")

with open(g1dat_path, 'rb') as f:
    # Read header
    num_entries, total_size = struct.unpack('<II', f.read(8))
    print(f"Found {num_entries} sprites")

    # Palette sprites are at indices 761-772 (12 palette sprites)
    # Each sprite contains a portion of the 256-color palette
    palette_sprite_indices = range(761, 773)  # 761 through 772
    palette = [(0, 0, 0)] * 256  # Initialize 256 colors

    print("\n🎨 Extracting from palette sprites...")
    for palette_sprite_idx in palette_sprite_indices:
        # Seek to palette sprite header
        header_offset = 8 + (palette_sprite_idx * 16)
        f.seek(header_offset)

        # Read entry
        offset = struct.unpack('<I', f.read(4))[0]
        width, height = struct.unpack('<hh', f.read(4))
        x_offset, y_offset = struct.unpack('<hh', f.read(4))

        if width <= 0:
            continue

        # Seek to palette data
        f.seek(offset)

        # Read RGB triplets
        for i in range(width):
            r, g, b = struct.unpack('BBB', f.read(3))
            palette_idx = x_offset + i
            if 0 <= palette_idx < 256:
                palette[palette_idx] = (r, g, b)

        print(f"  Sprite {palette_sprite_idx}: {width} colors at index {x_offset}")

    # Print first 20 colors
    print("\n📊 First 20 palette colors:")
    for i in range(20):
        print(f"  {i:3d}: RGB({palette[i][0]:3d}, {palette[i][1]:3d}, {palette[i][2]:3d})")

    # Save as Python list
    output_path = "rct2_palette.py"
    with open(output_path, 'w') as out:
        out.write("# RCT2 Color Palette extracted from g1.dat\n")
        out.write("# 256 RGB colors (R, G, B)\n\n")
        out.write("RCT2_PALETTE = [\n")
        for i in range(0, 256, 8):
            colors = [f"({palette[j][0]}, {palette[j][1]}, {palette[j][2]})" for j in range(i, min(i+8, 256))]
            out.write("    " + ", ".join(colors) + ",\n")
        out.write("]\n")

    print(f"\n✅ Palette saved to {output_path}")
    print("Now run: python3 extract_grass_with_palette.py")
