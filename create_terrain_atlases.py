#!/usr/bin/env python3
"""
Create atlases for all OpenRCT2 terrain types
"""

from PIL import Image
import os

# All terrain types
TERRAINS = [
    "grass", "sand", "sand_red", "sand_yellow", "ice",
    "grass_clumps", "martian", "checkerboard", "checkerboard_inverted",
    "dirt", "rock", "grass_mowed", "grass_mowed_90"
]

BASE_DIR = "godot-viewer/assets/tiles/terrain/openrct2_placeholder"
TILE_SIZE = 64  # Atlas cell size

def create_atlas_for_terrain(terrain_name):
    """Create 640×128 atlas for one terrain type"""
    terrain_dir = os.path.join(BASE_DIR, terrain_name)

    if not os.path.exists(terrain_dir):
        print(f"⚠️  Skipping {terrain_name}: directory not found")
        return False

    print(f"\n🎨 Creating atlas for {terrain_name}...")

    # Load all sprites
    sprites = []
    max_w, max_h = 0, 0

    for i in range(19):
        filename = os.path.join(terrain_dir, f"slope_{i:02d}.png")
        if os.path.exists(filename):
            img = Image.open(filename)
            sprites.append(img)
            max_w = max(max_w, img.width)
            max_h = max(max_h, img.height)
        else:
            print(f"   ⚠️  Missing slope_{i:02d}.png")
            return False

    print(f"   📐 Max dimensions: {max_w}×{max_h}")

    # Create atlas: 10×2 grid of 64×64 tiles
    atlas_width = TILE_SIZE * 10
    atlas_height = TILE_SIZE * 2
    atlas = Image.new('RGBA', (atlas_width, atlas_height), (0, 0, 0, 0))

    # Place sprites
    for i, sprite in enumerate(sprites):
        col = i % 10
        row = i // 10
        x = col * TILE_SIZE
        y = row * TILE_SIZE

        # Center sprite in tile, align to bottom
        paste_x = x + (TILE_SIZE - sprite.width) // 2
        paste_y = y + (TILE_SIZE - sprite.height)  # Bottom align

        atlas.paste(sprite, (paste_x, paste_y), sprite)

    # Save atlas
    output_path = os.path.join(terrain_dir, f"{terrain_name}_atlas.png")
    atlas.save(output_path)

    print(f"   ✅ {terrain_name}_atlas.png ({atlas_width}×{atlas_height})")
    return True

def main():
    print("=" * 60)
    print("OpenRCT2 Terrain Atlas Creation")
    print("=" * 60)
    print(f"Creating {len(TERRAINS)} atlases...")

    success = 0
    failed = []

    for terrain in TERRAINS:
        if create_atlas_for_terrain(terrain):
            success += 1
        else:
            failed.append(terrain)

    # Summary
    print("\n" + "=" * 60)
    print("ATLAS CREATION SUMMARY")
    print("=" * 60)
    print(f"✅ Created: {success}/{len(TERRAINS)} atlases")

    if failed:
        print(f"❌ Failed: {', '.join(failed)}")

    print("\n✅ All terrain atlases ready!")
    print(f"📁 Location: {BASE_DIR}/<terrain>/<terrain>_atlas.png")

if __name__ == "__main__":
    main()
