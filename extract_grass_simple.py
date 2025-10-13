#!/usr/bin/env python3
"""
Simple OpenRCT2 grass terrain sprite extractor
Based on actual OpenRCT2 RLE decompression code
"""

import struct
from PIL import Image

# RCT2 Default Palette (256 colors, RGB)
# This is the standard RCT2 palette from game files
PALETTE = [
    (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0),
    (0, 0, 0), (0, 0, 0), (35, 35, 23), (51, 51, 35), (67, 67, 47), (83, 83, 63), (99, 99, 75),
    (115, 115, 91), (131, 131, 111), (151, 151, 131), (175, 175, 159), (195, 195, 183),
    (219, 219, 211), (243, 243, 239), (0, 47, 51), (0, 59, 63), (11, 75, 79), (19, 91, 91),
    (31, 107, 107), (43, 127, 119), (59, 143, 135), (79, 155, 147), (95, 171, 159),
    (115, 187, 171), (135, 199, 183), (159, 215, 195), (183, 231, 211), (207, 247, 227),
    (227, 255, 239), (0, 47, 111), (0, 47, 159), (0, 47, 203), (0, 51, 255), (0, 87, 255),
    (0, 123, 255), (0, 155, 255), (0, 183, 255), (0, 219, 255), (0, 255, 255), (11, 67, 35),
    (15, 91, 47), (19, 111, 63), (23, 131, 75), (31, 151, 87), (39, 171, 99), (51, 187, 115),
    (63, 203, 131), (75, 219, 147), (91, 239, 167), (111, 255, 183), (43, 39, 27), (55, 51, 39),
    (67, 63, 51), (83, 75, 67), (95, 91, 83), (111, 111, 103), (127, 127, 123), (143, 147, 143),
    (159, 167, 163), (179, 187, 183), (199, 207, 203), (219, 227, 223), (0, 43, 0), (0, 63, 0),
    (7, 87, 0), (15, 107, 7), (19, 127, 15), (27, 147, 19), (35, 167, 31), (43, 187, 39),
    (55, 207, 51), (67, 227, 67), (83, 247, 87), (15, 7, 0), (27, 15, 0), (43, 23, 0), (55, 31, 0),
    (67, 43, 0), (83, 55, 0), (99, 67, 7), (115, 83, 15), (131, 99, 23), (147, 119, 35),
    (163, 135, 47), (183, 155, 63), (195, 175, 83), (207, 195, 103), (223, 215, 127),
    (239, 235, 159), (255, 255, 195), (111, 47, 0), (131, 59, 0), (151, 75, 0), (175, 91, 0),
    (191, 107, 7), (215, 127, 19), (235, 147, 35), (255, 171, 51), (255, 195, 75),
    (255, 219, 103), (255, 243, 139), (255, 255, 179), (75, 47, 11), (95, 59, 23), (115, 75, 35),
    (135, 95, 51), (159, 119, 67), (179, 139, 87), (199, 167, 111), (219, 187, 139),
    (239, 215, 167), (255, 239, 199), (255, 255, 227), (51, 31, 0), (63, 39, 0), (79, 51, 0),
    (95, 63, 7), (111, 75, 15), (131, 91, 27), (151, 111, 43), (171, 131, 59), (191, 155, 79),
    (207, 179, 99), (227, 203, 123), (247, 227, 147), (255, 255, 183), (255, 255, 219),
    (255, 255, 255), (107, 0, 0), (127, 0, 0), (151, 0, 0), (171, 0, 0), (191, 0, 0), (215, 0, 0),
    (239, 0, 0), (255, 23, 23), (255, 51, 51), (255, 83, 83), (255, 115, 115), (255, 147, 147),
    (255, 183, 183), (255, 219, 219), (255, 255, 255), (35, 0, 0), (59, 0, 0), (79, 0, 0),
    (103, 0, 0), (123, 0, 0), (143, 7, 7), (163, 23, 23), (183, 43, 43), (203, 63, 63),
    (223, 83, 83), (239, 111, 111), (255, 139, 139), (255, 171, 171), (255, 203, 203),
    (255, 235, 235), (255, 255, 255), (59, 31, 11), (75, 43, 19), (91, 55, 31), (107, 71, 43),
    (127, 87, 59), (143, 107, 75), (163, 127, 95), (179, 147, 115), (199, 167, 135),
    (215, 191, 155), (235, 215, 179), (255, 239, 207), (95, 63, 23), (115, 79, 39), (135, 99, 55),
    (155, 119, 67), (175, 139, 83), (195, 159, 99), (219, 183, 119), (239, 203, 139),
    (255, 227, 163), (255, 247, 191), (255, 255, 223), (99, 79, 43), (119, 99, 59), (143, 119, 75),
    (163, 139, 95), (187, 159, 115), (207, 183, 135), (227, 203, 159), (251, 227, 183),
    (255, 247, 207), (255, 255, 235), (111, 91, 63), (131, 111, 83), (151, 131, 103),
    (175, 151, 127), (199, 175, 147), (219, 199, 171), (243, 223, 195), (255, 247, 223),
    (255, 255, 247), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0),
    (0, 0, 0)
]

def read_g1_header(f):
    """Read g1.dat header"""
    num_entries, total_size = struct.unpack('<II', f.read(8))
    return num_entries, total_size

def read_entry_header(f):
    """Read sprite entry header (16 bytes)"""
    offset = struct.unpack('<I', f.read(4))[0]
    width, height = struct.unpack('<hh', f.read(4))
    x_offset, y_offset = struct.unpack('<hh', f.read(4))
    flags = struct.unpack('<H', f.read(2))[0]
    zoomed_offset = struct.unpack('<H', f.read(2))[0]
    return {
        'offset': offset,
        'width': width,
        'height': height,
        'x_offset': x_offset,
        'y_offset': y_offset,
        'flags': flags
    }

def extract_sprite_simple(f, entry):
    """Extract sprite using OpenRCT2's RLE format"""
    if entry['width'] <= 0 or entry['height'] <= 0:
        return None

    # Create transparent image
    img = Image.new('RGBA', (entry['width'], entry['height']), (0, 0, 0, 0))
    pixels = img.load()

    # Seek to sprite data
    f.seek(entry['offset'])

    # Read row offsets (2 bytes per row)
    row_offsets = []
    for _ in range(entry['height']):
        offset = struct.unpack('<H', f.read(2))[0]
        row_offsets.append(offset)

    # Decode each row
    for row_num, row_offset in enumerate(row_offsets):
        if row_offset == 0 or row_offset == 0xFFFF:
            continue  # Empty row

        # Seek to row data
        f.seek(entry['offset'] + row_offset)

        # Decode chunks in this row
        while True:
            try:
                data_size = f.read(1)[0]
                first_pixel_x = f.read(1)[0]

                is_end_of_line = (data_size & 0x80) != 0
                data_size &= 0x7F

                # Read pixel data
                for i in range(data_size):
                    x_pos = first_pixel_x + i
                    if 0 <= x_pos < entry['width']:
                        palette_idx = f.read(1)[0]
                        color = PALETTE[palette_idx]
                        pixels[x_pos, row_num] = (*color, 255)

                if is_end_of_line:
                    break
            except:
                break

    return img

# Main extraction
g1dat_path = "/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/Data/g1.dat"
output_dir = "godot-viewer/assets/tiles/terrain/openrct2_placeholder/grass"

print("🌿 Extracting grass terrain sprites...")

with open(g1dat_path, 'rb') as f:
    num_entries, total_size = read_g1_header(f)
    print(f"📊 Found {num_entries} sprites")

    # Grass terrain: sprites 3419-3437 (19 slopes)
    grass_start = 3419
    grass_end = 3437

    for slope_idx in range(19):
        sprite_idx = grass_start + slope_idx

        # Seek to entry header
        header_offset = 8 + (sprite_idx * 16)
        f.seek(header_offset)
        entry = read_entry_header(f)

        # Extract sprite
        img = extract_sprite_simple(f, entry)

        if img:
            filename = f"{output_dir}/slope_{slope_idx:02d}.png"
            img.save(filename)
            print(f"✅ slope_{slope_idx:02d}.png ({img.width}×{img.height})")
        else:
            print(f"⚠️ slope_{slope_idx:02d} failed")

print("\n✅ Grass extraction complete!")
