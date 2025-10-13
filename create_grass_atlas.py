#!/usr/bin/env python3
"""
Create normalized grass terrain atlas from extracted sprites
"""

from PIL import Image
import os

grass_dir = "godot-viewer/assets/tiles/terrain/openrct2_placeholder/grass"

# Load all sprites
sprites = []
max_w, max_h = 0, 0

print("📏 Analyzing sprite dimensions...")
for i in range(19):
    filename = f"{grass_dir}/slope_{i:02d}.png"
    img = Image.open(filename)
    sprites.append(img)
    max_w = max(max_w, img.width)
    max_h = max(max_h, img.height)
    print(f"  slope_{i:02d}: {img.width}×{img.height}")

print(f"\n📐 Max dimensions: {max_w}×{max_h}")

# Normalize to 64×64 to accommodate all sprites
tile_size = 64

# Create atlas: 10×2 grid
atlas_width = tile_size * 10
atlas_height = tile_size * 2
atlas = Image.new('RGBA', (atlas_width, atlas_height), (0, 0, 0, 0))

print(f"\n🎨 Creating atlas ({atlas_width}×{atlas_height})...")

for i, sprite in enumerate(sprites):
    # Calculate position in atlas
    col = i % 10
    row = i // 10
    x = col * tile_size
    y = row * tile_size

    # Center the sprite in the tile (align to bottom)
    paste_x = x + (tile_size - sprite.width) // 2
    paste_y = y + (tile_size - sprite.height)  # Align to bottom

    # Paste sprite
    atlas.paste(sprite, (paste_x, paste_y), sprite)
    print(f"  ✅ Placed slope_{i:02d} at ({col}, {row})")

# Save atlas
output_path = f"{grass_dir}/grass_atlas.png"
atlas.save(output_path)

print(f"\n✅ Atlas created: {output_path}")
print(f"📦 Size: {atlas_width}×{atlas_height} pixels")
print(f"🎯 Tile size: {tile_size}×{tile_size} pixels")
print("\nNext: Configure this atlas in Godot TileSet with tile size {tile_size}×{tile_size}")
