#!/bin/bash
# RCT2 Tree Sprite Extraction Script
# Extracts tree sprites from RCT2 object DAT files using objexport tool
# Created: 2025-10-13
# DO NOT LOSE THIS SCRIPT!

set -e  # Exit on error

# Configuration - can be overridden with environment variables
# export RCT2_OBJDATA_DIR="/path/to/your/RCT2/ObjData"
# export RCT2_TREES_OUTPUT="/path/to/output"
OBJDATA_DIR="${RCT2_OBJDATA_DIR:-/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData}"
OUTPUT_DIR="${RCT2_TREES_OUTPUT:-./extracted-trees}"
OBJEXPORT="/opt/homebrew/opt/dotnet@6/bin/dotnet /Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"
GODOT_TREES_DIR="godot-viewer/assets/sprites/vegetation/trees"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}RCT2 Tree Sprite Extraction Script${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"

if [ ! -d "$OBJDATA_DIR" ]; then
    echo -e "${RED}ERROR: RCT2 ObjData directory not found at:${NC}"
    echo "$OBJDATA_DIR"
    exit 1
fi

if [ ! -f "/opt/homebrew/opt/dotnet@6/bin/dotnet" ]; then
    echo -e "${RED}ERROR: .NET 6.0 not found. Install with:${NC}"
    echo "  brew install dotnet@6"
    exit 1
fi

if [ ! -f "/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll" ]; then
    echo -e "${RED}ERROR: objexport tool not built. Build with:${NC}"
    echo "  cd /Users/jean/Github/objects/tools/objexport && dotnet build"
    exit 1
fi

echo -e "${GREEN}✓ All prerequisites found${NC}"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to extract a tree
extract_tree() {
    local dat_file=$1
    local tree_name=$2
    local output_name=$3

    echo -e "${BLUE}Extracting $tree_name...${NC}"

    if [ ! -f "$OBJDATA_DIR/$dat_file" ]; then
        echo -e "${RED}  ERROR: $dat_file not found${NC}"
        return 1
    fi

    $OBJEXPORT "$OBJDATA_DIR/$dat_file" "$OUTPUT_DIR/${dat_file%.DAT}" --png > /dev/null 2>&1

    local dat_lower=$(echo "${dat_file%.DAT}" | tr '[:upper:]' '[:lower:]')
    local source_file="$OUTPUT_DIR/${dat_file%.DAT}/rct2.$dat_lower/images.png"

    if [ -f "$source_file" ]; then
        if [ -n "$output_name" ]; then
            cp "$source_file" "$GODOT_TREES_DIR/$output_name"
            echo -e "${GREEN}  ✓ Extracted and copied to: $output_name${NC}"
        else
            echo -e "${GREEN}  ✓ Extracted to: $source_file${NC}"
        fi
    else
        echo -e "${RED}  ERROR: Extraction failed${NC}"
        return 1
    fi
}

# Extract trees
echo -e "${BLUE}Extracting tree sprites...${NC}"
echo ""

# Common trees (already extracted)
extract_tree "TCF.DAT" "Caucasian Fir Tree" "tree_fir_caucasian.png"
extract_tree "TSP.DAT" "Scots Pine Tree" "tree_pine_scots.png"
extract_tree "TRF.DAT" "Red Fir Tree" "tree_fir_red.png"

# Additional popular trees (optional - uncomment to extract)
# extract_tree "TRF2.DAT" "Red Fir Tree (variant 2)" "tree_fir_red2.png"
# extract_tree "TRF3.DAT" "Red Fir Tree (variant 3)" "tree_fir_red3.png"
# extract_tree "TMZP.DAT" "Montezuma Pine Tree" "tree_pine_montezuma.png"
# extract_tree "TAP.DAT" "Aleppo Pine Tree" "tree_pine_aleppo.png"
# extract_tree "TCRP.DAT" "Corsican Pine Tree" "tree_pine_corsican.png"
# extract_tree "TBP.DAT" "Black Poplar Tree" "tree_poplar_black.png"
# extract_tree "TCL.DAT" "Cedar of Lebanon Tree" "tree_cedar_lebanon.png"
# extract_tree "TEL.DAT" "European Larch Tree" "tree_larch_european.png"

# Desert trees
# extract_tree "TOAS.DAT" "Oasis Palm Tree" "tree_palm_oasis.png"
# extract_tree "TLC.DAT" "Lombardy Cypress Tree" "tree_cypress_lombardy.png"
# extract_tree "TMO.DAT" "Mediterranean Oak Tree" "tree_oak_mediterranean.png"
# extract_tree "TWW.DAT" "Weeping Willow Tree" "tree_willow_weeping.png"

# Snow trees
# extract_tree "TCFS.DAT" "Caucasian Fir Tree (Snow)" "tree_fir_caucasian_snow.png"
# extract_tree "TRFS.DAT" "Red Fir Tree (Snow)" "tree_fir_red_snow.png"
# extract_tree "TSP1.DAT" "Scots Pine Tree (Snow 1)" "tree_pine_scots_snow1.png"
# extract_tree "TSP2.DAT" "Scots Pine Tree (Snow 2)" "tree_pine_scots_snow2.png"
# extract_tree "TSPH.DAT" "Scots Pine Tree (Snow Heavy)" "tree_pine_scots_snow_heavy.png"

echo ""
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}Extraction Complete!${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""
echo -e "Tree sprites saved to:"
echo -e "  ${BLUE}$GODOT_TREES_DIR${NC}"
echo ""
echo -e "Raw extractions in:"
echo -e "  ${BLUE}$OUTPUT_DIR${NC}"
echo ""
echo -e "${BLUE}To extract additional trees, edit this script and uncomment the desired trees.${NC}"
