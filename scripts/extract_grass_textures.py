#!/usr/bin/env python3
"""
Extract grass textures from stone-kingdoms packed atlas for use in Godot viewer.

The stone-kingdoms project uses a 52MB packed texture atlas (8192x16384 pixels)
with defined quad regions for each tile variant. This script extracts specific
grass tiles that can be used in the life-simulator Godot viewer.

Usage:
    python3 scripts/extract_grass_textures.py

Requirements:
    pip install pillow
"""

from PIL import Image
import os
import sys

# Source atlas path
ATLAS_PATH = "/Users/jean/Github/stone-kingdoms/assets/tiles/stronghold_assets_packed_v12-hd.png"
OUTPUT_DIR = "/Users/jean/Github/life-simulator/godot-viewer/assets/tiles/grass"

# Grass tile definitions from object_quads.lua
# Format: (name, x, y, width, height)
GRASS_TILES = [
    # 1x1 abundant grass variants (30x17 or 30x18 pixels)
    ("abundant_grass_1x1_01", 4995, 198, 30, 18),
    ("abundant_grass_1x1_02", 5029, 198, 30, 18),
    ("abundant_grass_1x1_03", 5063, 198, 30, 18),
    ("abundant_grass_1x1_04", 70, 156, 30, 17),
    ("abundant_grass_1x1_05", 8113, 135, 30, 17),
    ("abundant_grass_1x1_06", 8147, 135, 30, 17),
    ("abundant_grass_1x1_07", 2, 156, 30, 17),
    ("abundant_grass_1x1_08", 104, 156, 30, 17),

    # 1x1 light variants (lighting variations)
    ("abundant_grass_1x1_light1_01", 138, 156, 30, 17),
    ("abundant_grass_1x1_light1_02", 5131, 198, 30, 18),
    ("abundant_grass_1x1_light2_01", 206, 156, 30, 17),
    ("abundant_grass_1x1_light2_02", 5199, 198, 30, 18),

    # 2x2 macro tiles (62x34/35 pixels)
    ("abundant_grass_2x2_01", 4562, 952, 62, 34),
    ("abundant_grass_2x2_02", 2010, 1028, 62, 35),
    ("abundant_grass_2x2_03", 4628, 952, 62, 34),
    ("abundant_grass_2x2_04", 4694, 952, 62, 34),

    # 3x3 macro tiles (94x49 pixels)
    ("abundant_grass_3x3_01", 3263, 4188, 94, 49),
    ("abundant_grass_3x3_02", 3361, 4188, 94, 49),
    ("abundant_grass_3x3_03", 3459, 4188, 94, 49),
    ("abundant_grass_3x3_04", 3557, 4188, 94, 49),

    # 4x4 macro tiles (126x65 pixels)
    ("abundant_grass_4x4_01", 2177, 6692, 126, 65),
    ("abundant_grass_4x4_02", 2307, 6692, 126, 65),
    ("abundant_grass_4x4_03", 2437, 6692, 126, 65),
    ("abundant_grass_4x4_04", 2567, 6692, 126, 65),
]

def extract_tiles():
    """Extract grass tiles from the packed atlas."""
    # Check if atlas exists
    if not os.path.exists(ATLAS_PATH):
        print(f"❌ Error: Atlas not found at {ATLAS_PATH}")
        print("   Make sure stone-kingdoms repository is cloned to /Users/jean/Github/stone-kingdoms")
        sys.exit(1)

    # Create output directory
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Load the atlas
    print(f"📂 Loading atlas from {ATLAS_PATH}")
    print(f"   (This is a 52MB, 8192x16384 texture atlas - may take a moment...)")

    try:
        atlas = Image.open(ATLAS_PATH)
        print(f"✅ Atlas loaded: {atlas.width}x{atlas.height} pixels")
    except Exception as e:
        print(f"❌ Error loading atlas: {e}")
        sys.exit(1)

    # Extract each tile
    print(f"\n🔧 Extracting {len(GRASS_TILES)} grass tile variants...")

    extracted_count = 0
    for name, x, y, width, height in GRASS_TILES:
        try:
            # Crop the region
            tile = atlas.crop((x, y, x + width, y + height))

            # Save as PNG
            output_path = os.path.join(OUTPUT_DIR, f"{name}.png")
            tile.save(output_path, "PNG")

            extracted_count += 1
            print(f"  ✓ {name}.png ({width}x{height})")

        except Exception as e:
            print(f"  ✗ Failed to extract {name}: {e}")

    print(f"\n✅ Extracted {extracted_count}/{len(GRASS_TILES)} tiles to {OUTPUT_DIR}")
    print(f"\n📊 Tile size breakdown:")
    print(f"   - 1x1 tiles: 30x17-18 pixels (individual tiles)")
    print(f"   - 2x2 tiles: 62x34-35 pixels (4 tiles merged)")
    print(f"   - 3x3 tiles: 94x49 pixels (9 tiles merged)")
    print(f"   - 4x4 tiles: 126x65 pixels (16 tiles merged)")
    print(f"\n💡 Next steps:")
    print(f"   1. Import these tiles into Godot")
    print(f"   2. Create a TileSet from the extracted tiles")
    print(f"   3. Use macro tiles (2x2, 3x3, 4x4) for better performance")
    print(f"   4. See scripts/integrate_grass_godot.md for integration guide")

if __name__ == "__main__":
    extract_tiles()
