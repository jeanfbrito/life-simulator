#!/usr/bin/env python3
"""
Extract OpenRCT2 terrain sprites from g1.dat file
Based on OpenRCT2 g1.dat format documentation
"""

import struct
import os
from PIL import Image

# RCT2 Default Palette (256 colors, RGB)
# This is the standard RCT2 palette extracted from game files
RCT2_PALETTE = [
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

class G1DATExtractor:
    def __init__(self, g1dat_path):
        self.g1dat_path = g1dat_path
        self.file = None
        self.num_entries = 0
        self.entries = []

    def open(self):
        """Open g1.dat file and read header"""
        self.file = open(self.g1dat_path, 'rb')

        # Read number of entries (4 bytes, little endian)
        self.num_entries = struct.unpack('<I', self.file.read(4))[0]
        print(f"📊 G1.DAT contains {self.num_entries} sprite entries")

        # Initialize entries list without reading all headers
        # We'll read headers on-demand for better performance
        self.entries = [None] * self.num_entries

    def _read_entry_header(self):
        """Read a single sprite entry header"""
        offset = struct.unpack('<I', self.file.read(4))[0]
        width = struct.unpack('<H', self.file.read(2))[0]
        height = struct.unpack('<H', self.file.read(2))[0]
        x_offset = struct.unpack('<h', self.file.read(2))[0]  # signed
        y_offset = struct.unpack('<h', self.file.read(2))[0]  # signed
        flags = struct.unpack('<H', self.file.read(2))[0]

        return {
            'offset': offset,
            'width': width,
            'height': height,
            'x_offset': x_offset,
            'y_offset': y_offset,
            'flags': flags
        }

    def extract_sprite(self, index):
        """Extract a single sprite by index"""
        if index >= self.num_entries:
            print(f"❌ Sprite index {index} out of range")
            return None

        # Read entry header on-demand if not cached
        if self.entries[index] is None:
            # Seek to entry header position
            header_offset = 4 + (index * 16)  # 4 bytes for count, 16 bytes per entry
            self.file.seek(header_offset)
            self.entries[index] = self._read_entry_header()

        entry = self.entries[index]

        # Skip empty sprites
        if entry['width'] == 0 or entry['height'] == 0:
            return None

        # Seek to sprite data
        self.file.seek(entry['offset'])

        # Create image with transparency
        img = Image.new('RGBA', (entry['width'], entry['height']), (0, 0, 0, 0))
        pixels = img.load()

        # Decode RLE compressed sprite data
        x, y = 0, 0

        try:
            for row in range(entry['height']):
                x = 0

                # Read row offset (2 bytes)
                row_offset = struct.unpack('<H', self.file.read(2))[0]

                if row_offset == 0xFFFF:
                    # Empty row
                    continue

                # Save current position
                saved_pos = self.file.tell()

                # Seek to row data
                self.file.seek(entry['offset'] + row_offset)

                # Read row data
                while x < entry['width']:
                    # Read chunk info
                    chunk_info = self.file.read(1)[0]

                    if chunk_info & 0x80:  # Transparent pixels
                        num_pixels = chunk_info & 0x7F
                        x += num_pixels
                    else:  # Opaque pixels
                        num_pixels = chunk_info
                        for _ in range(num_pixels):
                            if x < entry['width']:
                                palette_index = self.file.read(1)[0]
                                color = RCT2_PALETTE[palette_index]
                                pixels[x, row] = (*color, 255)
                                x += 1

                # Restore position for next row offset
                self.file.seek(saved_pos)
        except Exception as e:
            print(f"⚠️ Error decoding sprite {index}: {e}")
            return None

        return img

    def close(self):
        """Close the g1.dat file"""
        if self.file:
            self.file.close()


def extract_terrain_sprites(g1dat_path, output_dir):
    """Extract all terrain sprites from g1.dat"""

    # Terrain sprite ranges
    terrain_ranges = {
        'grass': (3419, 3437),       # Standard grass (19 slopes)
        'sand': (3438, 3456),         # Beach sand
        'dirt': (3457, 3475),         # Brown dirt
        'stone': (3476, 3494),        # Gray rock/stone
        'grass_dark': (3495, 3513),   # Dark green grass (forest floor)
        'grass_light': (3514, 3532),  # Light grass (dried)
    }

    print("🎨 Starting RCT2 terrain sprite extraction...")
    print(f"📂 Source: {g1dat_path}")
    print(f"📁 Output: {output_dir}")
    print()

    # Create output directories
    for terrain in terrain_ranges.keys():
        terrain_dir = os.path.join(output_dir, terrain)
        os.makedirs(terrain_dir, exist_ok=True)

    # Open g1.dat
    extractor = G1DATExtractor(g1dat_path)
    extractor.open()

    # Extract each terrain type
    for terrain, (start, end) in terrain_ranges.items():
        print(f"🌍 Extracting {terrain}...")
        slope_index = 0

        for sprite_index in range(start, end + 1):
            img = extractor.extract_sprite(sprite_index)

            if img:
                # Save as slope_XX.png
                filename = f"slope_{slope_index:02d}.png"
                filepath = os.path.join(output_dir, terrain, filename)
                img.save(filepath)
                print(f"  ✅ Saved {filename} ({img.width}×{img.height})")
            else:
                print(f"  ⚠️ Failed to extract sprite {sprite_index}")

            slope_index += 1

    extractor.close()
    print()
    print("✅ Extraction complete!")
    print(f"📦 Sprites saved to: {output_dir}")


if __name__ == "__main__":
    import sys

    # Default paths
    g1dat_path = "/Users/jean/Library/Application Support/Steam/steamapps/common/RollerCoaster Tycoon Classic/RCT Classic.app/Contents/Resources/g1.dat"
    output_dir = "/Users/jean/Github/life-simulator/godot-viewer/assets/tiles/terrain/openrct2_placeholder"

    # Allow command line override
    if len(sys.argv) > 1:
        g1dat_path = sys.argv[1]
    if len(sys.argv) > 2:
        output_dir = sys.argv[2]

    # Check if g1.dat exists
    if not os.path.exists(g1dat_path):
        print(f"❌ Error: g1.dat not found at: {g1dat_path}")
        print()
        print("Please provide the correct path:")
        print(f"  python3 {sys.argv[0]} /path/to/g1.dat [output_dir]")
        sys.exit(1)

    # Extract sprites
    extract_terrain_sprites(g1dat_path, output_dir)
