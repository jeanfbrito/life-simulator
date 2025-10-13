#!/bin/bash
# Extract grass textures from stone-kingdoms packed atlas using ImageMagick
# This is a bash alternative to the Python script for systems without Pillow

ATLAS_PATH="/Users/jean/Github/stone-kingdoms/assets/tiles/stronghold_assets_packed_v12-hd.png"
OUTPUT_DIR="godot-viewer/assets/tiles/grass"

# Check if atlas exists
if [ ! -f "$ATLAS_PATH" ]; then
    echo "❌ Error: Atlas not found at $ATLAS_PATH"
    echo "   Make sure stone-kingdoms repository is cloned to /Users/jean/Github/stone-kingdoms"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check for ImageMagick
if ! command -v convert &> /dev/null; then
    echo "❌ Error: ImageMagick not found"
    echo "   Install with: brew install imagemagick"
    exit 1
fi

echo "📂 Loading atlas from $ATLAS_PATH"
echo "   (This is a 52MB, 8192x16384 texture atlas - extractions will be fast)"
echo ""
echo "🔧 Extracting grass tile variants..."

# Extract function: extract_tile name x y width height
extract_tile() {
    local name=$1
    local x=$2
    local y=$3
    local width=$4
    local height=$5
    local output="$OUTPUT_DIR/${name}.png"

    convert "$ATLAS_PATH" -crop "${width}x${height}+${x}+${y}" +repage "$output" 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "  ✓ ${name}.png (${width}x${height})"
        return 0
    else
        echo "  ✗ Failed to extract ${name}"
        return 1
    fi
}

# Extract tiles
count=0

# 1x1 abundant grass variants (30x17 or 30x18 pixels)
echo "Extracting 1x1 tiles..."
extract_tile "abundant_grass_1x1_01" 4995 198 30 18 && ((count++))
extract_tile "abundant_grass_1x1_02" 5029 198 30 18 && ((count++))
extract_tile "abundant_grass_1x1_03" 5063 198 30 18 && ((count++))
extract_tile "abundant_grass_1x1_04" 70 156 30 17 && ((count++))
extract_tile "abundant_grass_1x1_05" 8113 135 30 17 && ((count++))
extract_tile "abundant_grass_1x1_06" 8147 135 30 17 && ((count++))
extract_tile "abundant_grass_1x1_07" 2 156 30 17 && ((count++))
extract_tile "abundant_grass_1x1_08" 104 156 30 17 && ((count++))

# 1x1 light variants
echo "Extracting 1x1 light variants..."
extract_tile "abundant_grass_1x1_light1_01" 138 156 30 17 && ((count++))
extract_tile "abundant_grass_1x1_light1_02" 5131 198 30 18 && ((count++))
extract_tile "abundant_grass_1x1_light2_01" 206 156 30 17 && ((count++))
extract_tile "abundant_grass_1x1_light2_02" 5199 198 30 18 && ((count++))

# 2x2 macro tiles
echo "Extracting 2x2 macro tiles..."
extract_tile "abundant_grass_2x2_01" 4562 952 62 34 && ((count++))
extract_tile "abundant_grass_2x2_02" 2010 1028 62 35 && ((count++))
extract_tile "abundant_grass_2x2_03" 4628 952 62 34 && ((count++))
extract_tile "abundant_grass_2x2_04" 4694 952 62 34 && ((count++))

# 3x3 macro tiles
echo "Extracting 3x3 macro tiles..."
extract_tile "abundant_grass_3x3_01" 3263 4188 94 49 && ((count++))
extract_tile "abundant_grass_3x3_02" 3361 4188 94 49 && ((count++))
extract_tile "abundant_grass_3x3_03" 3459 4188 94 49 && ((count++))
extract_tile "abundant_grass_3x3_04" 3557 4188 94 49 && ((count++))

# 4x4 macro tiles
echo "Extracting 4x4 macro tiles..."
extract_tile "abundant_grass_4x4_01" 2177 6692 126 65 && ((count++))
extract_tile "abundant_grass_4x4_02" 2307 6692 126 65 && ((count++))
extract_tile "abundant_grass_4x4_03" 2437 6692 126 65 && ((count++))
extract_tile "abundant_grass_4x4_04" 2567 6692 126 65 && ((count++))

echo ""
echo "✅ Extracted $count/28 tiles to $OUTPUT_DIR"
echo ""
echo "📊 Tile size breakdown:"
echo "   - 1x1 tiles: 30x17-18 pixels (individual tiles)"
echo "   - 2x2 tiles: 62x34-35 pixels (4 tiles merged)"
echo "   - 3x3 tiles: 94x49 pixels (9 tiles merged)"
echo "   - 4x4 tiles: 126x65 pixels (16 tiles merged)"
echo ""
echo "💡 Next steps:"
echo "   1. Import these tiles into Godot (auto-imported)"
echo "   2. Create a TileSet from the extracted tiles"
echo "   3. Use macro tiles (2x2, 3x3, 4x4) for better performance"
echo "   4. See scripts/integrate_grass_godot.md for integration guide"
