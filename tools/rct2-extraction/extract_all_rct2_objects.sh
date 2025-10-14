#!/bin/bash
# RCT2 Complete Object Extraction and Organization Script
# Extracts ALL object DAT files and organizes by category
# Created: 2025-10-13
# This will take a while - RCT2 has hundreds of objects!

set -e  # Exit on error

# Configuration
# You can override these with environment variables:
# export RCT2_OBJDATA_DIR="/path/to/your/RCT2/ObjData"
# export RCT2_OUTPUT_DIR="/path/to/output"
OBJDATA_DIR="${RCT2_OBJDATA_DIR:-/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData}"
OUTPUT_BASE="${RCT2_OUTPUT_DIR:-./extracted-objects}"
DOTNET="/opt/homebrew/opt/dotnet@6/bin/dotnet"
OBJEXPORT_DLL="/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Category directories
CATEGORIES=(
    "trees"
    "small_scenery"
    "large_scenery"
    "walls"
    "path_banners"
    "paths"
    "path_additions"
    "scenery_groups"
    "park_entrance"
    "water"
    "rides"
    "unknown"
)

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   RCT2 COMPLETE OBJECT EXTRACTION & ORGANIZATION         ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check prerequisites
echo -e "${CYAN}Checking prerequisites...${NC}"

if [ ! -d "$OBJDATA_DIR" ]; then
    echo -e "${RED}ERROR: RCT2 ObjData directory not found at:${NC}"
    echo "$OBJDATA_DIR"
    exit 1
fi

if [ ! -f "$DOTNET" ]; then
    echo -e "${RED}ERROR: .NET 6.0 not found. Install with:${NC}"
    echo "  brew install dotnet@6"
    exit 1
fi

if [ ! -f "$OBJEXPORT_DLL" ]; then
    echo -e "${RED}ERROR: objexport tool not built. Build with:${NC}"
    echo "  cd /Users/jean/Github/objects/tools/objexport && dotnet build"
    exit 1
fi

echo -e "${GREEN}✓ All prerequisites found${NC}"
echo ""

# Count total DAT files
total_files=$(find "$OBJDATA_DIR" -name "*.DAT" | wc -l | xargs)
echo -e "${YELLOW}Found $total_files DAT files to process${NC}"
echo ""

# Create category directories
echo -e "${CYAN}Creating category directories...${NC}"
for category in "${CATEGORIES[@]}"; do
    mkdir -p "$OUTPUT_BASE/$category"
done
echo -e "${GREEN}✓ Directories created${NC}"
echo ""

# Initialize statistics
total_processed=0
total_failed=0

# Create index files for each category
for category in "${CATEGORIES[@]}"; do
    echo "# RCT2 ${category^^} Objects" > "$OUTPUT_BASE/$category/INDEX.md"
    echo "" >> "$OUTPUT_BASE/$category/INDEX.md"
    echo "Extracted on: $(date)" >> "$OUTPUT_BASE/$category/INDEX.md"
    echo "" >> "$OUTPUT_BASE/$category/INDEX.md"
    echo "## Objects in this category:" >> "$OUTPUT_BASE/$category/INDEX.md"
    echo "" >> "$OUTPUT_BASE/$category/INDEX.md"
done

# Function to determine category from object type
get_category() {
    local object_type=$1
    case "$object_type" in
        "scenery_small")
            echo "small_scenery"
            ;;
        "scenery_large")
            echo "large_scenery"
            ;;
        "scenery_wall")
            echo "walls"
            ;;
        "footpath_banner")
            echo "path_banners"
            ;;
        "footpath_surface"|"footpath_railings")
            echo "paths"
            ;;
        "footpath_addition")
            echo "path_additions"
            ;;
        "scenery_group")
            echo "scenery_groups"
            ;;
        "park_entrance")
            echo "park_entrance"
            ;;
        "water")
            echo "water"
            ;;
        "ride")
            echo "rides"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Function to check if object is a tree
is_tree() {
    local json_file=$1
    if [ -f "$json_file" ]; then
        if grep -q '"isTree".*true' "$json_file" 2>/dev/null; then
            return 0
        fi
    fi
    return 1
}

# Process each DAT file
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Starting extraction process...${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

current=0
find "$OBJDATA_DIR" -name "*.DAT" | sort | while read -r dat_file; do
    current=$((current + 1))
    filename=$(basename "$dat_file" .DAT)

    # Progress indicator
    echo -ne "${CYAN}[$current/$total_files]${NC} Processing: ${YELLOW}$filename${NC}..."

    # Create temp extraction directory
    temp_dir="$OUTPUT_BASE/temp/$filename"
    mkdir -p "$temp_dir"

    # Extract object
    if "$DOTNET" "$OBJEXPORT_DLL" "$dat_file" "$temp_dir" --png > /dev/null 2>&1; then
        # Find the generated directory (lowercase object id)
        generated_dir=$(find "$temp_dir" -type d -name "rct2.*" | head -1)

        if [ -n "$generated_dir" ] && [ -f "$generated_dir/object.json" ]; then
            # Read object type from JSON
            object_type=$(grep -o '"objectType"[[:space:]]*:[[:space:]]*"[^"]*"' "$generated_dir/object.json" | cut -d'"' -f4)

            # Determine category
            if [ "$object_type" = "scenery_small" ] && is_tree "$generated_dir/object.json"; then
                category="trees"
            else
                category=$(get_category "$object_type")
            fi

            # Get object name from JSON
            object_name=$(grep -A 1 '"name".*{' "$generated_dir/object.json" | grep '"en-GB"' | cut -d'"' -f4 | head -1)
            if [ -z "$object_name" ]; then
                object_name="$filename"
            fi

            # Move to category directory
            dest_dir="$OUTPUT_BASE/$category/$filename"
            mv "$generated_dir" "$dest_dir"

            # Update index
            {
                echo "### $filename"
                echo "**Name**: $object_name"
                echo "**Type**: $object_type"
                if [ -f "$dest_dir/images.png" ]; then
                    size=$(ls -lh "$dest_dir/images.png" | awk '{print $5}')
                    echo "**Sprite**: images.png ($size)"
                fi
                echo ""
            } >> "$OUTPUT_BASE/$category/INDEX.md"

            echo -e "\r${CYAN}[$current/$total_files]${NC} ${GREEN}✓${NC} $filename → $category (${object_name:0:30})"
        else
            echo -e "\r${CYAN}[$current/$total_files]${NC} ${YELLOW}⚠${NC} $filename (no object.json found)"
        fi
    else
        echo -e "\r${CYAN}[$current/$total_files]${NC} ${RED}✗${NC} $filename (extraction failed)"
    fi

    # Clean up temp directory
    rm -rf "$temp_dir"
done

# Remove temp directory
rm -rf "$OUTPUT_BASE/temp"

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Generating summary reports...${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Count objects per category
for category in "${CATEGORIES[@]}"; do
    count=$(find "$OUTPUT_BASE/$category" -type d -name "rct2.*" | wc -l | xargs)
    if [ "$count" -gt 0 ]; then
        echo -e "${GREEN}$category:${NC} $count objects"
    fi
done

echo ""

# Create master index
{
    echo "# RCT2 Complete Object Library"
    echo ""
    echo "Extracted on: $(date)"
    echo ""
    echo "## Summary"
    echo ""
    for category in "${CATEGORIES[@]}"; do
        count=$(find "$OUTPUT_BASE/$category" -type d -name "rct2.*" 2>/dev/null | wc -l | xargs)
        if [ "$count" -gt 0 ]; then
            echo "- **$category**: $count objects"
        fi
    done
    echo ""
    echo "## Categories"
    echo ""
    for category in "${CATEGORIES[@]}"; do
        count=$(find "$OUTPUT_BASE/$category" -type d -name "rct2.*" 2>/dev/null | wc -l | xargs)
        if [ "$count" -gt 0 ]; then
            echo "### [$category](./$category/INDEX.md) ($count objects)"
            echo ""
        fi
    done
    echo ""
    echo "## Extraction Details"
    echo ""
    echo "- **Source**: RollerCoaster Tycoon 2 ObjData directory"
    echo "- **Tool**: OpenRCT2 objexport (F# .NET tool)"
    echo "- **Format**: PNG sprites with JSON metadata"
    echo "- **Authors**: Chris Sawyer & Simon Foster (original art)"
    echo ""
    echo "## Usage"
    echo ""
    echo "Each category directory contains:"
    echo "- Individual object folders (rct2.xxx format)"
    echo "- images.png (sprite atlas with all views)"
    echo "- object.json (metadata, properties, strings)"
    echo "- INDEX.md (category-specific listing)"
    echo ""
    echo "## Categories Explained"
    echo ""
    echo "- **trees**: Small scenery objects with isTree=true flag"
    echo "- **small_scenery**: Decorative objects (benches, statues, flowers, etc.)"
    echo "- **large_scenery**: Multi-tile structures (castles, landmarks, etc.)"
    echo "- **walls**: Fences, barriers, building walls"
    echo "- **path_banners**: Signs and banners for pathways"
    echo "- **paths**: Footpath surfaces and railings"
    echo "- **path_additions**: Benches, lamps, bins for paths"
    echo "- **scenery_groups**: Theme/style groupings"
    echo "- **park_entrance**: Park entrance structures"
    echo "- **water**: Water types and effects"
    echo "- **rides**: Ride vehicles and tracks"
    echo ""
} > "$OUTPUT_BASE/INDEX.md"

# Create tree-specific summary
tree_count=$(find "$OUTPUT_BASE/trees" -type d -name "rct2.*" 2>/dev/null | wc -l | xargs)
if [ "$tree_count" -gt 0 ]; then
    {
        echo "# RCT2 Trees - Quick Reference"
        echo ""
        echo "Total trees extracted: $tree_count"
        echo ""
        echo "## Tree Categories"
        echo ""
        echo "### Grass Trees"
        find "$OUTPUT_BASE/trees" -type d -name "rct2.*" | while read -r tree_dir; do
            tree_id=$(basename "$tree_dir")
            if [ -f "$tree_dir/object.json" ]; then
                tree_name=$(grep -A 1 '"name".*{' "$tree_dir/object.json" | grep '"en-GB"' | cut -d'"' -f4 | head -1)
                echo "- **$tree_id**: $tree_name"
            fi
        done
        echo ""
    } > "$OUTPUT_BASE/trees/TREES_QUICKREF.md"
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              EXTRACTION COMPLETE! ✅                      ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Output directory:${NC}"
echo -e "  $OUTPUT_BASE"
echo ""
echo -e "${CYAN}Master index:${NC}"
echo -e "  $OUTPUT_BASE/INDEX.md"
echo ""
echo -e "${CYAN}Browse categories:${NC}"
for category in "${CATEGORIES[@]}"; do
    count=$(find "$OUTPUT_BASE/$category" -type d -name "rct2.*" 2>/dev/null | wc -l | xargs)
    if [ "$count" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $category ($count objects)"
    fi
done
echo ""
echo -e "${YELLOW}Note: This extracted ALL RCT2 objects. You can now browse${NC}"
echo -e "${YELLOW}the complete library and copy desired sprites to your project!${NC}"
