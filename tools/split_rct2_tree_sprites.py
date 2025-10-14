#!/usr/bin/env python3
"""
Split RCT2 tree sprite sheets into individual PNG files
Each RCT2 tree has 4 isometric views (NE, SE, SW, NW) in a horizontal strip
"""

import json
import os
from PIL import Image

# RCT2 tree directories and their friendly names
TREE_MAPPING = {
    'TCF': 'caucasian_fir',      # Caucasian Fir Tree
    'TSP': 'scots_pine',          # Scots Pine Tree
    'TRF': 'red_fir',             # Red Fir Tree
    'TRF2': 'red_fir_2',          # Red Fir Tree variant 2
    'TRF3': 'red_fir_3',          # Red Fir Tree variant 3
    'TMZP': 'montezuma_pine',     # Montezuma Pine Tree
    'TAP': 'aleppo_pine',         # Aleppo Pine Tree
    'TCRP': 'corsican_pine',      # Corsican Pine Tree
    'TBP': 'black_poplar',        # Black Poplar Tree
    'TCL': 'cedar_lebanon',       # Cedar of Lebanon Tree
    'TEL': 'european_larch',      # European Larch Tree
}

VIEW_NAMES = ['ne', 'se', 'sw', 'nw']


def split_tree_sprites(input_dir, output_dir):
    """Split all RCT2 tree sprite sheets into individual PNG files

    Args:
        input_dir: Path to RCT2-Tree-Sprites directory (e.g., ~/Downloads/RCT2-Tree-Sprites/)
        output_dir: Path to output directory (e.g., godot-viewer/assets/tiles/trees/)
    """

    print("🌲 RCT2 Tree Sprite Splitter")
    print(f"📂 Input:  {input_dir}")
    print(f"📁 Output: {output_dir}")
    print()

    os.makedirs(output_dir, exist_ok=True)

    total_trees = 0
    total_sprites = 0

    # Process each tree
    for tree_code, tree_name in TREE_MAPPING.items():
        print(f"🌳 Processing {tree_name} ({tree_code})...")

        # Find the tree directory
        tree_dir = os.path.join(input_dir, tree_code)
        if not os.path.exists(tree_dir):
            print(f"  ⚠️  Tree directory not found: {tree_dir}")
            continue

        # Find the rct2.* subdirectory
        rct2_dirs = [d for d in os.listdir(tree_dir) if d.startswith('rct2.')]
        if not rct2_dirs:
            print(f"  ⚠️  No rct2.* subdirectory found in {tree_dir}")
            continue

        sprite_dir = os.path.join(tree_dir, rct2_dirs[0])

        # Load object.json for metadata
        object_json_path = os.path.join(sprite_dir, 'object.json')
        images_png_path = os.path.join(sprite_dir, 'images.png')

        if not os.path.exists(object_json_path):
            print(f"  ⚠️  object.json not found: {object_json_path}")
            continue

        if not os.path.exists(images_png_path):
            print(f"  ⚠️  images.png not found: {images_png_path}")
            continue

        # Load metadata
        with open(object_json_path, 'r') as f:
            metadata = json.load(f)

        # Load sprite sheet
        sprite_sheet = Image.open(images_png_path)

        # Extract image metadata (4 views)
        images_meta = metadata.get('images', [])

        if len(images_meta) != 4:
            print(f"  ⚠️  Expected 4 views, found {len(images_meta)}")
            continue

        # Extract each view
        for i, (view_name, img_meta) in enumerate(zip(VIEW_NAMES, images_meta)):
            # Extract crop region from sprite sheet
            src_x = img_meta['srcX']
            src_y = img_meta['srcY']
            src_width = img_meta['srcWidth']
            src_height = img_meta['srcHeight']

            # Crop the view from sprite sheet
            view_img = sprite_sheet.crop((src_x, src_y, src_x + src_width, src_y + src_height))

            # Convert to RGBA if needed (RCT2 uses indexed color)
            if view_img.mode != 'RGBA':
                view_img = view_img.convert('RGBA')

            # Save individual view
            output_filename = f"tree_{tree_name}_{view_name}.png"
            output_path = os.path.join(output_dir, output_filename)
            view_img.save(output_path)

            print(f"  ✅ {output_filename} ({src_width}×{src_height})")
            total_sprites += 1

        total_trees += 1
        print()

    print(f"✅ Complete! Extracted {total_sprites} sprites from {total_trees} trees")
    print(f"📦 Sprites saved to: {output_dir}")


if __name__ == "__main__":
    import sys

    # Default paths
    input_dir = os.path.expanduser("~/Downloads/RCT2-Tree-Sprites")
    output_dir = os.path.join(os.path.dirname(__file__), "..", "godot-viewer", "assets", "tiles", "trees", "rct2")

    # Allow command line override
    if len(sys.argv) > 1:
        input_dir = sys.argv[1]
    if len(sys.argv) > 2:
        output_dir = sys.argv[2]

    # Check if input directory exists
    if not os.path.exists(input_dir):
        print(f"❌ Error: Input directory not found: {input_dir}")
        print()
        print("Usage:")
        print(f"  python3 {sys.argv[0]} [input_dir] [output_dir]")
        print()
        print("Example:")
        print(f"  python3 {sys.argv[0]} ~/Downloads/RCT2-Tree-Sprites godot-viewer/assets/tiles/trees/rct2")
        sys.exit(1)

    split_tree_sprites(input_dir, output_dir)
