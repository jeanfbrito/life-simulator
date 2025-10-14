#!/bin/bash
# Quick test version - processes only first 20 DAT files
# Use this to verify the script works before running full catalog

set -e

# Configuration
OBJDATA_DIR="${RCT2_OBJDATA_DIR:-/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData}"
OUTPUT_FILE="${RCT2_CATALOG_OUTPUT:-./RCT2-Objects-Test-Catalog.md}"
DOTNET="/opt/homebrew/opt/dotnet@6/bin/dotnet"
OBJEXPORT_DLL="/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        RCT2 TEST CATALOG (First 20 objects only)         ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Count files
total_files=$(find "$OBJDATA_DIR" -name "*.DAT" | wc -l | xargs)
test_count=20

echo -e "${YELLOW}Testing with first $test_count of $total_files DAT files${NC}"
echo ""

# Initialize catalog
{
    echo "# RCT2 Object Catalog (TEST - First $test_count objects)"
    echo ""
    echo "Generated on: $(date)"
    echo ""
    echo "---"
    echo ""
} > "$OUTPUT_FILE"

# Category counters
count_trees=0
count_small_scenery=0
count_large_scenery=0
count_walls=0
count_path_banners=0
count_paths=0
count_path_additions=0
count_scenery_groups=0
count_park_entrance=0
count_water=0
count_rides=0
count_unknown=0

# Category sections (storing in temp files)
TEMP_DIR="/tmp/rct2_test_catalog_$$"
mkdir -p "$TEMP_DIR"
for category in trees small_scenery large_scenery walls path_banners paths path_additions scenery_groups park_entrance water rides unknown; do
    touch "$TEMP_DIR/${category}.txt"
done

echo -e "${CYAN}Processing first $test_count objects...${NC}"

current=0
find "$OBJDATA_DIR" -name "*.DAT" | sort | head -$test_count | while read -r dat_file; do
    current=$((current + 1))
    filename=$(basename "$dat_file" .DAT)

    echo -ne "\r${CYAN}[$current/$test_count]${NC} $filename...          "

    # Quick extraction to temp
    temp_dir="/tmp/rct2_test_$$_$current"
    mkdir -p "$temp_dir"

    if "$DOTNET" "$OBJEXPORT_DLL" "$dat_file" "$temp_dir" --png > /dev/null 2>&1; then
        json_file=$(find "$temp_dir" -name "object.json" | head -1)

        if [ -f "$json_file" ]; then
            # Extract metadata
            object_type=$(grep -o '"objectType"[[:space:]]*:[[:space:]]*"[^"]*"' "$json_file" | cut -d'"' -f4)
            object_name=$(grep -A 1 '"name".*{' "$json_file" | grep '"en-GB"' | cut -d'"' -f4 | head -1)
            is_tree=$(grep -q '"isTree".*true' "$json_file" 2>/dev/null && echo "yes" || echo "no")

            if [ -z "$object_name" ]; then
                object_name="(Unknown)"
            fi

            # Determine category
            if [ "$object_type" = "scenery_small" ] && [ "$is_tree" = "yes" ]; then
                category="trees"
            else
                case "$object_type" in
                    "scenery_small") category="small_scenery" ;;
                    "scenery_large") category="large_scenery" ;;
                    "scenery_wall") category="walls" ;;
                    "footpath_banner") category="path_banners" ;;
                    "footpath_surface"|"footpath_railings") category="paths" ;;
                    "footpath_addition") category="path_additions" ;;
                    "scenery_group") category="scenery_groups" ;;
                    "park_entrance") category="park_entrance" ;;
                    "water") category="water" ;;
                    "ride") category="rides" ;;
                    *) category="unknown" ;;
                esac
            fi

            # Append to category section
            echo "- **$filename**: $object_name" >> "$TEMP_DIR/${category}.txt"

            # Increment counter
            case "$category" in
                trees) count_trees=$((count_trees + 1)) ;;
                small_scenery) count_small_scenery=$((count_small_scenery + 1)) ;;
                large_scenery) count_large_scenery=$((count_large_scenery + 1)) ;;
                walls) count_walls=$((count_walls + 1)) ;;
                path_banners) count_path_banners=$((count_path_banners + 1)) ;;
                paths) count_paths=$((count_paths + 1)) ;;
                path_additions) count_path_additions=$((count_path_additions + 1)) ;;
                scenery_groups) count_scenery_groups=$((count_scenery_groups + 1)) ;;
                park_entrance) count_park_entrance=$((count_park_entrance + 1)) ;;
                water) count_water=$((count_water + 1)) ;;
                rides) count_rides=$((count_rides + 1)) ;;
                unknown) count_unknown=$((count_unknown + 1)) ;;
            esac
        fi
    fi

    rm -rf "$temp_dir"
done

echo -e "\r${GREEN}✓ Processing complete!                              ${NC}"
echo ""

# Helper function to get count for category
get_count() {
    case "$1" in
        trees) echo "$count_trees" ;;
        small_scenery) echo "$count_small_scenery" ;;
        large_scenery) echo "$count_large_scenery" ;;
        walls) echo "$count_walls" ;;
        path_banners) echo "$count_path_banners" ;;
        paths) echo "$count_paths" ;;
        path_additions) echo "$count_path_additions" ;;
        scenery_groups) echo "$count_scenery_groups" ;;
        park_entrance) echo "$count_park_entrance" ;;
        water) echo "$count_water" ;;
        rides) echo "$count_rides" ;;
        unknown) echo "$count_unknown" ;;
    esac
}

# Write catalog
{
    echo ""
    echo "## Summary by Category (First $test_count objects)"
    echo ""
    for category in trees small_scenery large_scenery walls path_banners paths path_additions scenery_groups park_entrance water rides unknown; do
        count=$(get_count "$category")
        if [ "$count" -gt 0 ]; then
            echo "- **$category**: $count objects"
        fi
    done

    echo ""
    echo "---"
    echo ""

    # Write each category
    for category in trees small_scenery large_scenery walls path_banners paths path_additions scenery_groups park_entrance water rides unknown; do
        count=$(get_count "$category")
        if [ "$count" -gt 0 ]; then
            echo "## ${category^^} ($count objects)"
            echo ""
            cat "$TEMP_DIR/${category}.txt"
            echo ""
        fi
    done

    echo "---"
    echo ""
    echo "**This is a TEST catalog. Run ./catalog_rct2_objects.sh for full catalog.**"
    echo ""

} >> "$OUTPUT_FILE"

# Display summary
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║               TEST CATALOG COMPLETE! ✅                   ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Test catalog saved to:${NC}"
echo "  $OUTPUT_FILE"
echo ""
echo -e "${CYAN}Object count in test (first $test_count):${NC}"
for category in trees small_scenery large_scenery walls path_banners paths path_additions scenery_groups park_entrance water rides unknown; do
    count=$(get_count "$category")
    if [ "$count" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $category: $count objects"
    fi
done
echo ""
echo -e "${YELLOW}Script working correctly! Run full catalog:${NC}"
echo -e "${YELLOW}  ./catalog_rct2_objects.sh${NC}"
echo ""

# Cleanup temp directory
rm -rf "$TEMP_DIR"
