# RCT2 Complete Object Extraction Guide

## Overview

This guide explains how to extract and organize **ALL** RollerCoaster Tycoon 2 objects from the game files.

---

## What Gets Extracted

The script processes **every DAT file** in RCT2's ObjData directory and organizes them by category:

### Categories

1. **trees** - All tree objects (small scenery with isTree=true)
   - Grass trees: Firs, pines, cedars, poplars, larches
   - Desert trees: Palms, cypress, oaks, willows
   - Snow trees: Winter variants of common trees
   - Expected: ~25-30 objects

2. **small_scenery** - Decorative objects
   - Flowers, bushes, rocks
   - Statues, fountains, clocks
   - Benches, bins, lamps
   - Expected: ~200-300 objects

3. **large_scenery** - Multi-tile structures
   - Castles, towers, buildings
   - Landmarks, monuments
   - Large decorative pieces
   - Expected: ~50-100 objects

4. **walls** - Fences and barriers
   - Wood fences, stone walls
   - Iron railings, hedges
   - Building walls
   - Expected: ~50-80 objects

5. **path_banners** - Signs and banners
   - Park signs, direction markers
   - Themed banners
   - Expected: ~20-30 objects

6. **paths** - Footpath surfaces
   - Queue lines, sidewalks
   - Dirt paths, stone paths
   - Expected: ~20-30 objects

7. **path_additions** - Path furniture
   - Benches, lamps, bins
   - Queue TVs, jumping fountains
   - Expected: ~30-50 objects

8. **scenery_groups** - Theme collections
   - Abstract, Egyptian, Roman
   - Medieval, Space, Prehistoric
   - Expected: ~20-30 groups

9. **park_entrance** - Entrance structures
   - Classic entrance, themed variants
   - Expected: ~5-10 objects

10. **water** - Water types
    - Standard water, themed water
    - Expected: ~5-10 objects

11. **rides** - Ride vehicles and tracks
    - Roller coaster trains, cars
    - Ride vehicles
    - Expected: ~200+ objects (LARGE)

12. **unknown** - Unclassified objects
    - Objects that don't fit other categories
    - Expected: ~10-20 objects

---

## Total Expected Objects

**~600-800 objects** depending on RCT2 version (base game + expansions)

---

## Running the Script

### Quick Start

```bash
/Users/jean/Downloads/extract_all_rct2_objects.sh
```

### What Happens

1. **Prerequisites Check**
   - Verifies RCT2 ObjData directory exists
   - Checks .NET 6.0 is installed
   - Confirms objexport tool is built

2. **Directory Creation**
   - Creates organized category folders
   - Initializes index files

3. **Extraction Loop** (This takes time!)
   - Processes each DAT file one by one
   - Extracts sprites and metadata
   - Categorizes based on object type
   - Detects trees vs regular small scenery
   - Updates category indexes
   - Shows progress: `[123/678] ✓ TCF → trees (Caucasian Fir Tree)`

4. **Summary Generation**
   - Creates master INDEX.md
   - Generates category-specific indexes
   - Counts objects per category
   - Creates quick reference guides

---

## Expected Runtime

- **Full extraction**: ~15-30 minutes
- **Per object**: ~1-2 seconds
- **Total**: Depends on number of DAT files

**Progress indicators** show current status:
```
[123/678] Processing: TCF...
[123/678] ✓ TCF → trees (Caucasian Fir Tree)
```

---

## Output Structure

```
~/Downloads/RCT2-Objects-Complete/
├── INDEX.md (Master index with all categories)
├── trees/
│   ├── INDEX.md (Tree-specific listing)
│   ├── TREES_QUICKREF.md (Quick reference)
│   ├── TCF/
│   │   ├── rct2.tcf/
│   │   │   ├── images.png (4 isometric views)
│   │   │   └── object.json (metadata)
│   ├── TSP/
│   │   └── rct2.tsp/
│   │       ├── images.png
│   │       └── object.json
│   └── ...
├── small_scenery/
│   ├── INDEX.md
│   └── [hundreds of objects]
├── large_scenery/
│   ├── INDEX.md
│   └── [objects]
├── walls/
├── path_banners/
├── paths/
├── path_additions/
├── scenery_groups/
├── park_entrance/
├── water/
├── rides/
└── unknown/
```

---

## Using the Extracted Objects

### Browse by Category

1. Open `~/Downloads/RCT2-Objects-Complete/INDEX.md`
2. Click category link (e.g., `trees/INDEX.md`)
3. Browse category-specific listings

### View Object Details

Each object folder contains:
- **images.png**: Sprite atlas with all views
- **object.json**: Complete metadata including:
  - Object name (multilingual)
  - Properties (height, price, flags)
  - Animation data
  - Image references

### Copy to Godot Project

```bash
# Copy a specific tree
cp ~/Downloads/RCT2-Objects-Complete/trees/TCF/rct2.tcf/images.png \
   godot-viewer/assets/sprites/vegetation/trees/tree_name.png

# Copy all trees
cp ~/Downloads/RCT2-Objects-Complete/trees/*/rct2.*/images.png \
   godot-viewer/assets/sprites/vegetation/trees/
```

---

## Category-Specific Notes

### Trees
- Always have 4 isometric views
- Include tree-specific properties (height, isTree flag)
- Organized by type (grass, desert, snow)

### Small Scenery
- Varies widely in sprite count
- Some have animations (flags, fountains)
- Check object.json for special properties

### Large Scenery
- Multi-tile objects
- More complex metadata
- Higher sprite counts

### Rides
- **WARNING**: Largest category (~200+ objects)
- Complex sprite layouts
- Vehicle and track combinations
- May want to extract separately

---

## Advanced Usage

### Extract Specific Categories Only

Edit the script and comment out unwanted categories in the loop:

```bash
# Skip rides extraction (saves lots of time)
if [ "$category" = "rides" ]; then
    continue
fi
```

### Re-extract After Updates

The script overwrites existing files, so you can re-run to update:

```bash
# Fresh extraction
rm -rf ~/Downloads/RCT2-Objects-Complete
/Users/jean/Downloads/extract_all_rct2_objects.sh
```

### Search for Specific Objects

```bash
# Find all fountain objects
grep -r "fountain" ~/Downloads/RCT2-Objects-Complete/*/INDEX.md

# Find objects by name
grep -r "Castle" ~/Downloads/RCT2-Objects-Complete/*/INDEX.md
```

---

## Performance Tips

### Speed Up Extraction

1. **SSD**: Extract to SSD for faster I/O
2. **Skip Rides**: Comment out rides category (largest)
3. **Parallel**: Could modify script for parallel extraction

### Disk Space

- **Full extraction**: ~100-200 MB
- **Per object**: ~10-50 KB average
- **Rides**: ~50-100 MB (largest category)

---

## Troubleshooting

### "No such file or directory" for DAT files

**Problem**: RCT2 path is wrong
**Solution**: Edit script and update `OBJDATA_DIR` path

### ".NET 6.0 not found"

**Problem**: .NET 6.0 not installed
**Solution**: `brew install dotnet@6`

### "objexport.dll not found"

**Problem**: objexport not built
**Solution**:
```bash
cd /Users/jean/Github/objects/tools/objexport
dotnet build
```

### Script Hangs or Slow

**Problem**: Too many objects, slow extraction
**Solution**: Let it run! Progress shows in terminal. Or comment out "rides" category.

### Extraction Failed for Specific Object

**Problem**: Some DAT files may be corrupt or non-standard
**Solution**: Script shows `✗` for failed objects and continues

---

## What to Do After Extraction

### 1. Browse the Collection

Open `INDEX.md` files in each category to see what's available.

### 2. Identify Useful Assets

Look for:
- More tree varieties for terrain diversity
- Decorative scenery for buildings/landmarks
- Walls/fences for boundaries
- Path elements for roads/trails

### 3. Copy to Project

Only copy what you need! You don't need all 600+ objects.

### 4. Update ResourceManager

Add new textures to your Godot ResourceManager:

```gdscript
var tree_textures = {
    # Add newly discovered trees
    "OasisPalm": preload("res://assets/sprites/vegetation/trees/tree_palm_oasis.png"),
    "WillowTree": preload("res://assets/sprites/vegetation/trees/tree_willow_weeping.png"),
    # etc.
}
```

---

## Quick Commands Reference

```bash
# Run full extraction
/Users/jean/Downloads/extract_all_rct2_objects.sh

# View master index
open ~/Downloads/RCT2-Objects-Complete/INDEX.md

# Count objects per category
for dir in ~/Downloads/RCT2-Objects-Complete/*/; do
    echo "$(basename $dir): $(find $dir -type d -name 'rct2.*' | wc -l) objects"
done

# Search for specific object
grep -r "YourObjectName" ~/Downloads/RCT2-Objects-Complete/*/INDEX.md

# Copy all trees to Godot
cp ~/Downloads/RCT2-Objects-Complete/trees/*/rct2.*/images.png \
   godot-viewer/assets/sprites/vegetation/trees/
```

---

## Notes

- **First run takes time**: Be patient! ~15-30 minutes for full extraction
- **Huge collection**: 600+ objects is a LOT. Browse before copying
- **Not all needed**: Most projects only need 20-50 objects max
- **Authentic pixel art**: All sprites by Chris Sawyer (RCT2, 2002)

---

## Credits

- **Original Art**: Chris Sawyer & Simon Foster (RCT2)
- **Extraction Tool**: OpenRCT2 Team (objexport)
- **Organization**: This script
- **RCT2 Community**: File format documentation

---

**Last Updated**: 2025-10-13
**Status**: Ready to extract!
**Expected Runtime**: 15-30 minutes
**Output Size**: ~100-200 MB
