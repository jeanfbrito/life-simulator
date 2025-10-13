#!/bin/bash

# Extract tree textures from stone-kingdoms packed atlas
# Trees are much larger than grass tiles and need different handling

ATLAS_PATH="/Users/jean/Github/stone-kingdoms/assets/tiles/stronghold_assets_packed_v12-hd.png"
OUTPUT_DIR="/Users/jean/Github/life-simulator/godot-viewer/assets/tiles/trees"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to extract a tile region
extract_tile() {
    local name=$1
    local x=$2
    local y=$3
    local width=$4
    local height=$5

    local output="$OUTPUT_DIR/${name}.png"

    echo "Extracting: $name (${width}×${height})"
    convert "$ATLAS_PATH" -crop "${width}x${height}+${x}+${y}" +repage "$output"
}

echo "Extracting tree textures from stone-kingdoms..."
echo "Atlas: $ATLAS_PATH"
echo "Output: $OUTPUT_DIR"
echo ""

# Pine Trees (tall, narrow - 75×157 to 75×161)
echo "=== Pine Trees (25 variants) ==="
extract_tile "tree_pine_large_01" 2981 12264 75 157
extract_tile "tree_pine_large_02" 2342 12264 75 156
extract_tile "tree_pine_large_03" 3060 12264 76 157
extract_tile "tree_pine_large_04" 2421 12264 76 156
extract_tile "tree_pine_large_05" 3140 12264 77 157
extract_tile "tree_pine_large_06" 3221 12264 76 157
extract_tile "tree_pine_large_07" 3301 12264 76 157
extract_tile "tree_pine_large_08" 3381 12264 76 157
extract_tile "tree_pine_large_09" 3461 12264 76 157
extract_tile "tree_pine_large_10" 3684 12264 76 158
extract_tile "tree_pine_large_11" 3764 12264 75 158
extract_tile "tree_pine_large_12" 3843 12264 75 158
extract_tile "tree_pine_large_13" 4084 12264 76 159
extract_tile "tree_pine_large_14" 4164 12264 76 159
extract_tile "tree_pine_large_15" 4244 12264 76 159
extract_tile "tree_pine_large_16" 4324 12264 75 159
extract_tile "tree_pine_large_17" 4403 12264 75 159
extract_tile "tree_pine_large_18" 6053 12264 75 160
extract_tile "tree_pine_large_19" 6132 12264 75 160
extract_tile "tree_pine_large_20" 6211 12264 75 160
extract_tile "tree_pine_large_21" 6290 12264 75 160
extract_tile "tree_pine_large_22" 6927 12264 75 161
extract_tile "tree_pine_large_23" 7006 12264 75 161
extract_tile "tree_pine_large_24" 7085 12264 75 161
extract_tile "tree_pine_large_25" 7164 12264 75 161

# Birch Trees (medium - 63×122 to 69×123)
echo ""
echo "=== Birch Trees (25 variants) ==="
extract_tile "tree_birch_large_01" 2286 11399 63 123
extract_tile "tree_birch_large_02" 2353 11399 63 123
extract_tile "tree_birch_large_03" 1255 11399 64 122
extract_tile "tree_birch_large_04" 2420 11399 63 123
extract_tile "tree_birch_large_05" 1323 11399 64 122
extract_tile "tree_birch_large_06" 1391 11399 65 122
extract_tile "tree_birch_large_07" 1460 11399 65 122
extract_tile "tree_birch_large_08" 1529 11399 65 122
extract_tile "tree_birch_large_09" 1598 11399 66 122
extract_tile "tree_birch_large_10" 1183 11399 68 122
extract_tile "tree_birch_large_11" 8044 11274 68 121
extract_tile "tree_birch_large_12" 8116 11274 68 121
extract_tile "tree_birch_large_13" 2 11399 69 121
extract_tile "tree_birch_large_14" 7222 11274 68 120
extract_tile "tree_birch_large_15" 7294 11274 68 120
extract_tile "tree_birch_large_16" 5033 11274 68 119
extract_tile "tree_birch_large_17" 5105 11274 68 119
extract_tile "tree_birch_large_18" 3817 11274 68 118
extract_tile "tree_birch_large_19" 3889 11274 69 118
extract_tile "tree_birch_large_20" 3962 11274 69 118
extract_tile "tree_birch_large_21" 5177 11274 69 119
extract_tile "tree_birch_large_22" 5250 11274 69 119
extract_tile "tree_birch_large_23" 5323 11274 68 119
extract_tile "tree_birch_large_24" 5395 11274 68 119
extract_tile "tree_birch_large_25" 5467 11274 68 119

# Chestnut Trees (large, wide - 142×135 to 151×139)
echo ""
echo "=== Chestnut Trees (25 variants) ==="
extract_tile "tree_chestnut_large_01" 4399 11806 148 139
extract_tile "tree_chestnut_large_02" 4551 11806 142 139
extract_tile "tree_chestnut_large_03" 4697 11806 142 139
extract_tile "tree_chestnut_large_04" 4843 11806 142 139
extract_tile "tree_chestnut_large_05" 1890 11806 142 138
extract_tile "tree_chestnut_large_06" 2036 11806 142 138
extract_tile "tree_chestnut_large_07" 2182 11806 142 138
extract_tile "tree_chestnut_large_08" 2328 11806 143 138
extract_tile "tree_chestnut_large_09" 2475 11806 143 138
extract_tile "tree_chestnut_large_10" 1443 11806 151 138
extract_tile "tree_chestnut_large_11" 1598 11806 142 138
extract_tile "tree_chestnut_large_12" 1744 11806 142 138
extract_tile "tree_chestnut_large_13" 7207 11665 142 137
extract_tile "tree_chestnut_large_14" 7353 11665 141 137
extract_tile "tree_chestnut_large_15" 7498 11665 141 137
extract_tile "tree_chestnut_large_16" 7643 11665 142 137
extract_tile "tree_chestnut_large_17" 5805 11665 142 136
extract_tile "tree_chestnut_large_18" 7789 11665 142 137
extract_tile "tree_chestnut_large_19" 7935 11665 143 137
extract_tile "tree_chestnut_large_20" 2 11806 143 137
extract_tile "tree_chestnut_large_21" 5951 11665 143 136
extract_tile "tree_chestnut_large_22" 6098 11665 143 136
extract_tile "tree_chestnut_large_23" 5086 11665 142 135
extract_tile "tree_chestnut_large_24" 5232 11665 142 135
extract_tile "tree_chestnut_large_25" 5378 11665 142 135

echo ""
echo "✅ Tree texture extraction complete!"
echo "Extracted to: $OUTPUT_DIR"
echo ""
echo "Summary:"
echo "- 25 Pine tree variants (75×157-161 px)"
echo "- 25 Birch tree variants (63×118-123 px)"
echo "- 25 Chestnut tree variants (141×135-151 px)"
echo "Total: 75 tree texture files"
