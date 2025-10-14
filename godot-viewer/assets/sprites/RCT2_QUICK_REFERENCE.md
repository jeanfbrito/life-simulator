# RCT2 Sprites Quick Reference Guide

**Quick answers to common questions about RCT2 sprite extraction.**

---

## ❓ Are 11 Trees Enough?

### ✅ YES! 11 Grass Trees is EXCELLENT Variety

You currently have:
- **4 Fir varieties** (different shapes and colors)
- **4 Pine varieties** (including the iconic Scots Pine)
- **2 Deciduous trees** (round and layered)
- **1 Majestic cedar** (landmark tree)

**This gives you plenty of visual variety for forests!**

### When to Extract More Trees

**Extract desert trees** (4 varieties) if you need:
- Palm trees for oasis scenes
- Cypress for Mediterranean
- Willow for rivers/lakes

**Extract snow trees** (5 varieties) if you need:
- Winter/snowy biomes
- Christmas themed content

**Extract large trees** (10+ varieties) if you need:
- Landmark trees (4×4 Giant Mangrove)
- Jungle/tropical forests (3×3 tiles)
- More detailed multi-tile trees

**Bottom line**: Start with your 11 grass trees. Add more later if needed!

---

## 📚 How to Search for Sprites Instead of Manual Hunting

### ✅ Use the Index Files!

I created three index files to help you:

#### 1. **RCT2_TREES_INDEX.md** (This Directory)
- **Complete tree catalog** with 35+ varieties
- Searchable by category, name, status
- Includes DAT filenames, object IDs, extraction status
- **Use**: Search (Ctrl+F / Cmd+F) for tree names, categories

**Example Searches**:
```
"Desert Trees"        → Find all desert trees
"✅ EXTRACTED"         → See what you already have
"Not extracted"       → See what's available
"Palm"                → Find palm tree varieties
"Large Trees"         → Multi-tile trees
```

#### 2. **RCT2_OBJECT_INDEX.txt** (This Directory)
- **Full RCT2 object catalog** (started with 50 objects)
- Format: `FILENAME | OBJECT_ID | TYPE | NAME | PROPERTIES`
- Can be expanded to all 2,119 objects
- **Use**: Search for any RCT2 object by name or type

#### 3. **GRASS_TREES_COMPLETE.md** (This Directory)
- **Detailed info on the 11 extracted grass trees**
- File sizes, visual characteristics, usage examples
- Weighted random selection code
- Biome recommendations

---

## 🔍 How to Find Specific Sprites

### Method 1: Use the Trees Index (Fastest)
```bash
# Open the index
open godot-viewer/assets/sprites/vegetation/trees/RCT2_TREES_INDEX.md

# Search for what you want (Cmd+F / Ctrl+F)
# Example: Search for "Palm" to find palm trees
# You'll get: TOAS.DAT | Oasis Palm Tree
```

### Method 2: Search OpenRCT2 Source Code
```bash
# Search for object references
grep -r "tree" /Users/jean/Github/OpenRCT2/src/openrct2/world/map_generator/

# Search for specific object types
grep -r "scenery_small" /Users/jean/Github/objects/objects/rct2/
```

### Method 3: List Available DAT Files
```bash
# List all DAT files by pattern
ls "/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData/" | grep -i tree

# Or search by prefix
ls "/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData/T*.DAT"
```

### Method 4: Check Objects Repository JSON
```bash
# Find objects by ID
find /Users/jean/Github/objects -name "*.json" | xargs grep -l "isTree.*true"

# Search for specific names
grep -r "Palm Tree" /Users/jean/Github/objects/objects/
```

---

## 📖 Complete Workflow: Finding & Extracting New Sprites

### Example: "I want to extract a palm tree"

**Step 1: Search the index**
```bash
# Open RCT2_TREES_INDEX.md
# Search for "Palm"
# Result: TOAS.DAT | rct2.scenery_small.toas | Oasis Palm Tree
```

**Step 2: Extract the sprite**
```bash
DOTNET="/opt/homebrew/opt/dotnet@6/bin/dotnet"
OBJEXPORT="/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"
OBJDATA="/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData"

"$DOTNET" "$OBJEXPORT" "$OBJDATA/TOAS.DAT" ~/Downloads/RCT2-Tree-Sprites/TOAS --png
```

**Step 3: Copy to project**
```bash
cp ~/Downloads/RCT2-Tree-Sprites/TOAS/rct2.toas/images.png \
   godot-viewer/assets/sprites/vegetation/trees/tree_palm_oasis.png
```

**Step 4: Update the index**
- Mark TOAS.DAT as "✅ EXTRACTED" in RCT2_TREES_INDEX.md

---

## 🎯 Most Common Use Cases

### Use Case 1: "I need more tree variety"
→ Extract desert trees (4) or large trees (10+)
→ See RCT2_TREES_INDEX.md "Desert Trees" section

### Use Case 2: "I need winter/snow trees"
→ Extract snow trees (5 varieties)
→ See RCT2_TREES_INDEX.md "Snow Trees" section

### Use Case 3: "I want massive landmark trees"
→ Extract large trees (4×4 Giant Mangrove, 3×3 Jungle trees)
→ See RCT2_TREES_INDEX.md "LARGE TREES" section

### Use Case 4: "I need ground cover / bushes"
→ Extract bush objects
→ See RCT2_TREES_INDEX.md "BUSHES" section

### Use Case 5: "I want to find [specific object]"
1. Search RCT2_TREES_INDEX.md (if it's a tree)
2. Search RCT2_OBJECT_INDEX.txt (for any object)
3. If not found, check OpenRCT2 source code or objects repo

---

## 📂 File Locations Quick Reference

```
DOCUMENTATION:
  godot-viewer/assets/sprites/vegetation/trees/README.md
  godot-viewer/assets/sprites/vegetation/trees/RCT2_TREES_INDEX.md         ← TREE CATALOG
  godot-viewer/assets/sprites/vegetation/trees/RCT2_QUICK_REFERENCE.md     ← THIS FILE
  godot-viewer/assets/sprites/vegetation/trees/GRASS_TREES_COMPLETE.md
  godot-viewer/assets/sprites/vegetation/trees/EXTRACTION_SUCCESS_2025-10-13.md
  godot-viewer/assets/sprites/RCT2_OBJECT_INDEX.txt                        ← FULL CATALOG

EXTRACTED SPRITES:
  godot-viewer/assets/sprites/vegetation/trees/tree_*.png (11 trees)

EXTRACTION SCRIPT:
  /Users/jean/Downloads/extract_rct2_trees.sh

RAW EXTRACTIONS:
  ~/Downloads/RCT2-Tree-Sprites/[TREE]/rct2.[tree]/images.png

SOURCE DATA:
  /Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData/*.DAT (2,119 files)
  /Users/jean/Github/objects/objects/rct2/scenery_small/*.json
  /Users/jean/Github/OpenRCT2/src/openrct2/world/map_generator/TreePlacement.cpp
```

---

## 🚀 Future Improvements

### To Create Full Searchable Database (Optional)

```bash
# Extract ALL 2,119 objects (takes ~1 hour)
# Then create CSV/JSON database with:
for dat in "$OBJDATA"/*.DAT; do
  # Extract metadata
  # Parse object.json
  # Create database entry
done

# Create searchable index:
# - By object type (ride, scenery, wall, path, etc.)
# - By name (searchable)
# - By properties (isTree, height, flags)
# - By theme (grass, desert, snow, jungle, etc.)
```

**For now, the manual index covers all trees and common objects!**

---

## ✅ Summary

### Your Question 1: Are 11 trees enough?
**Answer**: YES! You have excellent variety with 11 grass trees. Extract more only if you need specific themes (desert, snow, jungle).

### Your Question 2: Can we create an index to search instead of hunting?
**Answer**: YES! Created three index files:
1. **RCT2_TREES_INDEX.md** - Complete tree catalog (35+ varieties)
2. **RCT2_OBJECT_INDEX.txt** - General object catalog (50+ indexed, expandable to 2,119)
3. **GRASS_TREES_COMPLETE.md** - Detailed info on your 11 extracted trees

**How to use**: Open index file → Search (Ctrl+F / Cmd+F) → Find DAT filename → Extract with command

---

**Last Updated**: 2025-10-13
**Status**: Indexes created and documented ✅
**Next**: Use indexes to find and extract additional sprites as needed!
