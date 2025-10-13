#!/usr/bin/env python3
"""
Extract all OpenRCT2 terrain types with their correct names
Based on SPR_TERRAIN_* definitions from Paint.Surface.h
"""

import os
import shutil
from pathlib import Path

# Terrain definitions from OpenRCT2 source
# Format: (name, start_sprite, description)
TERRAINS = [
    ("grass", 1915, "Lush fully-covered grass"),
    ("sand", 1972, "Beach/desert sand"),
    ("sand_red", 2029, "Red/martian sand"),
    ("sand_yellow", 2086, "Yellow sand variant"),
    ("ice", 2143, "Ice/snow terrain"),
    ("grass_clumps", 2200, "Natural grass with dirt patches"),
    ("martian", 2314, "Martian/alien surface"),
    ("checkerboard", 2371, "Debug checkerboard pattern"),
    ("checkerboard_inverted", 2428, "Inverted checkerboard"),
    ("dirt", 2485, "Brown dirt paths"),
    ("rock", 2542, "Rocky terrain"),
    ("grass_mowed", 2663, "Short mowed grass"),
    ("grass_mowed_90", 2701, "Mowed grass 90° variant"),
]

# Paths
SPRITE_LIBRARY = os.path.expanduser("~/RCT2-Sprites")
OUTPUT_BASE = "godot-viewer/assets/tiles/terrain/openrct2_placeholder"

def extract_terrain(name, start_sprite, description):
    """Extract one terrain type (19 slopes)"""
    output_dir = os.path.join(OUTPUT_BASE, name)
    os.makedirs(output_dir, exist_ok=True)

    print(f"\n🌍 Extracting {name} (sprites {start_sprite}-{start_sprite+18})")
    print(f"   {description}")

    success_count = 0
    for i in range(19):
        sprite_idx = start_sprite + i
        src = os.path.join(SPRITE_LIBRARY, f"{sprite_idx}.png")
        dst = os.path.join(output_dir, f"slope_{i:02d}.png")

        if os.path.exists(src):
            shutil.copy2(src, dst)
            size = os.path.getsize(dst)
            if size > 200:  # Skip nearly-empty sprites
                success_count += 1
                print(f"   ✅ slope_{i:02d}.png ({size} bytes)")
            else:
                print(f"   ⚠️  slope_{i:02d}.png (empty: {size} bytes)")
        else:
            print(f"   ❌ sprite {sprite_idx} not found")

    return success_count

def main():
    print("=" * 60)
    print("OpenRCT2 Terrain Extraction")
    print("=" * 60)
    print(f"Sprite library: {SPRITE_LIBRARY}")
    print(f"Output directory: {OUTPUT_BASE}")
    print(f"Total terrain types: {len(TERRAINS)}")

    # Check sprite library exists
    if not os.path.exists(SPRITE_LIBRARY):
        print(f"\n❌ Error: Sprite library not found at {SPRITE_LIBRARY}")
        return

    # Extract all terrains
    results = {}
    for name, start_sprite, description in TERRAINS:
        count = extract_terrain(name, start_sprite, description)
        results[name] = count

    # Summary
    print("\n" + "=" * 60)
    print("EXTRACTION SUMMARY")
    print("=" * 60)
    for name, count in results.items():
        status = "✅" if count >= 18 else "⚠️"
        print(f"{status} {name:25s} {count:2d}/19 sprites")

    print("\n✅ All terrain types extracted!")
    print(f"📁 Location: {OUTPUT_BASE}")
    print("\nNext step: Create atlases with create_terrain_atlas.py")

if __name__ == "__main__":
    main()
