#!/bin/bash

# Extract tree textures from stone-kingdoms atlas and apply spring green ColorTable

set -e  # Exit on error

# Configuration
ATLAS_PATH="/Users/jean/Github/stone-kingdoms/assets/tiles/stronghold_assets_packed_v12-hd.png"
COLORTABLE_PINE="/Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable1.png"
COLORTABLE_BIRCH="/Users/jean/Github/stone-kingdoms/colortables/BirchTree/ColorTable1.png"
OUTPUT_DIR="godot-viewer/assets/tiles/trees"
TEMP_DIR="/tmp/tree_extraction"
VENV_DIR="/tmp/palette_env"
PALETTE_SCRIPT="scripts/apply_tree_palette.py"

echo "========================================="
echo "Tree Texture Extraction with Green Palette"
echo "========================================="
echo ""

# Check if virtual environment exists
if [ ! -d "$VENV_DIR" ]; then
    echo "❌ Virtual environment not found at $VENV_DIR"
    echo "   Please run: python3 -m venv $VENV_DIR && source $VENV_DIR/bin/activate && pip install Pillow"
    exit 1
fi

# Activate virtual environment
echo "🐍 Activating Python virtual environment..."
source "$VENV_DIR/bin/activate"

# Create temp directory
mkdir -p "$TEMP_DIR"
mkdir -p "$OUTPUT_DIR"

echo "📂 Atlas: $ATLAS_PATH"
echo "📂 Output: $OUTPUT_DIR"
echo ""

# Function to extract and apply palette
extract_and_colorize() {
    local name=$1
    local x=$2
    local y=$3
    local w=$4
    local h=$5
    local color_table=$6
    local output_file="$OUTPUT_DIR/${name}.png"
    local temp_indexed="$TEMP_DIR/${name}_indexed.png"

    echo "  ├─ Extracting: $name (${w}×${h})"

    # Step 1: Extract from atlas with preserved colormap
    magick "$ATLAS_PATH" \
        -crop ${w}x${h}+${x}+${y} +repage \
        -define png:preserve-colormap \
        "$temp_indexed" 2>/dev/null

    if [ ! -f "$temp_indexed" ]; then
        echo "  ├─ ❌ Extraction failed"
        return 1
    fi

    # Step 2: Apply ColorTable palette using Python script
    python3 "$PALETTE_SCRIPT" "$temp_indexed" "$color_table" "$output_file" > /dev/null 2>&1

    if [ ! -f "$output_file" ]; then
        echo "  ├─ ❌ Palette application failed"
        return 1
    fi

    echo "  └─ ✅ Saved: $output_file"
}

echo "🌲 Extracting PINE trees (25 variants) with green palette..."
echo ""

# Pine trees - Large variants (from object_quads.lua)
extract_and_colorize "tree_pine_large_01" 2981 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_02" 3066 12264 73 161 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_03" 3149 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_04" 3234 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_05" 3319 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_06" 3404 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_07" 3489 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_08" 3574 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_09" 3659 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_10" 3744 12264 75 161 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_11" 3829 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_12" 3914 12264 73 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_13" 3997 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_14" 4082 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_15" 4167 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_16" 4252 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_17" 4337 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_18" 4422 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_19" 4507 12264 75 157 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_20" 4592 12264 75 161 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_21" 4677 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_22" 4762 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_23" 4847 12264 73 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_24" 4930 12264 75 159 "$COLORTABLE_PINE"
extract_and_colorize "tree_pine_large_25" 5015 12264 75 159 "$COLORTABLE_PINE"

echo ""
echo "🍂 Extracting BIRCH trees (22 variants) with green palette..."
echo ""

# Birch trees - Large variants
extract_and_colorize "tree_birch_large_01" 256 12896 63 118 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_02" 329 12896 61 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_03" 400 12896 61 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_04" 471 12896 61 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_05" 542 12896 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_06" 615 12896 61 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_07" 686 12896 61 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_08" 757 12896 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_09" 830 12896 61 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_10" 901 12896 61 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_11" 972 12896 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_12" 1045 12896 63 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_13" 1118 12896 61 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_14" 1189 12896 61 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_15" 1260 12896 61 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_16" 1331 12896 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_17" 1404 12896 61 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_18" 1475 12896 61 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_19" 1546 12896 61 120 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_20" 1617 12896 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_21" 1690 12896 63 123 "$COLORTABLE_BIRCH"
extract_and_colorize "tree_birch_large_22" 1763 12896 61 120 "$COLORTABLE_BIRCH"

# Clean up temp directory
echo ""
echo "🧹 Cleaning up temporary files..."
rm -rf "$TEMP_DIR"

echo ""
echo "========================================="
echo "✅ Extraction complete!"
echo "========================================="
echo "Extracted 47 tree textures (25 pine + 22 birch)"
echo "Location: $OUTPUT_DIR"
echo "Palette: Spring Green (ColorTable1)"
echo ""
echo "Next steps:"
echo "  1. Verify trees in: $OUTPUT_DIR"
echo "  2. Test TreeTextureManager.gd in Godot viewer"
echo "  3. Test ResourceManager integration"
