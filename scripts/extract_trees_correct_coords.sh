#!/bin/bash

# Extract tree textures with CORRECT coordinates from object_quads.lua and apply green palette

set -e

ATLAS_PATH="/Users/jean/Github/stone-kingdoms/assets/tiles/stronghold_assets_packed_v12-hd.png"
COLORTABLE_PINE="/Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable1.png"
COLORTABLE_BIRCH="/Users/jean/Github/stone-kingdoms/colortables/BirchTree/ColorTable1.png"
OUTPUT_DIR="godot-viewer/assets/tiles/trees"
TEMP_DIR="/tmp/tree_extraction"
VENV_DIR="/tmp/palette_env"
PALETTE_SCRIPT="scripts/apply_tree_palette.py"

echo "========================================="
echo "Tree Extraction with CORRECT Coordinates"
echo "========================================="
echo ""

source "$VENV_DIR/bin/activate"
mkdir -p "$TEMP_DIR"
mkdir -p "$OUTPUT_DIR"

extract_and_colorize() {
    local name=$1
    local x=$2
    local y=$3
    local w=$4
    local h=$5
    local color_table=$6
    local output_file="$OUTPUT_DIR/${name}.png"
    local temp_indexed="$TEMP_DIR/${name}_indexed.png"

    echo "  ├─ $name (${w}×${h} from ${x},${y})"

    magick "$ATLAS_PATH" -crop ${w}x${h}+${x}+${y} +repage -define png:preserve-colormap "$temp_indexed" 2>/dev/null
    if [ ! -f "$temp_indexed" ]; then
        echo "  └─ ❌ Extraction failed"
        return 1
    fi

    python3 "$PALETTE_SCRIPT" "$temp_indexed" "$color_table" "$output_file" > /dev/null 2>&1
    if [ ! -f "$output_file" ]; then
        echo "  └─ ❌ Palette application failed"
        return 1
    fi

    echo "  └─ ✅"
}

echo "🌲 Extracting PINE trees with CORRECT coordinates..."
echo ""

# Pine trees - coordinates from object_quads.lua
extract_and_colorize "tree_pine_large_01" 2981 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_02" 2342 12264 75 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_03" 3060 12264 76 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_04" 2421 12264 76 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_05" 3140 12264 77 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_06" 3221 12264 76 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_07" 3301 12264 76 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_08" 3381 12264 76 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_09" 3461 12264 76 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_10" 3684 12264 76 158 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_11" 2501 12264 77 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_12" 3541 12264 77 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_13" 3764 12264 76 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_14" 2582 12264 77 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_15" 3621 12264 77 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_16" 3844 12264 76 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_17" 2663 12264 77 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_18" 3924 12264 77 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_19" 2744 12264 76 156 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_20" 4005 12264 76 158 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_21" 2824 12264 77 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_22" 2905 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_23" 4085 12264 76 158 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_24" 4165 12264 76 158 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_25" 4245 12264 76 158 "$COLORTABLE_PINE"

echo ""
echo "🍂 Extracting BIRCH trees with CORRECT coordinates..."
echo ""

# Birch trees - coordinates from object_quads.lua
extract_and_colorize "tree_birch_large_01" 2286 11399 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_02" 2353 11399 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_03" 1255 11399 64 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_04" 2420 11399 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_05" 1323 11399 64 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_06" 1391 11399 65 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_07" 1460 11399 65 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_08" 1529 11399 65 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_09" 1598 11399 66 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_10" 1183 11399 68 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_11" 2490 11399 64 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_12" 1668 11399 68 122 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_13" 2558 11399 65 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_14" 1740 11399 66 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_15" 2627 11399 66 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_16" 1810 11399 68 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_17" 2697 11399 67 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_18" 1882 11399 67 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_19" 2768 11399 67 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_20" 1953 11399 68 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_21" 2025 11399 67 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_22" 2839 11399 68 123 "$COLORTABLE_BIRCH"

echo ""
rm -rf "$TEMP_DIR"
echo "✅ Extraction complete with CORRECT coordinates from object_quads.lua"
