# RCT2 Object Extraction Scripts

**Location**: `tools/rct2-extraction/`
**Created**: October 13, 2025
**Purpose**: Extract and organize ALL RollerCoaster Tycoon 2 objects for the Life Simulator project

---

## 🚀 Quick Start

```bash
# Run from project root
cd tools/rct2-extraction

# Option 1: Quick catalog (30 seconds - RECOMMENDED FIRST)
./catalog_rct2_objects.sh

# Option 2: Full extraction (15-30 minutes)
./extract_all_rct2_objects.sh

# Option 3: Trees only (already done - 11 grass trees)
./extract_rct2_trees.sh
```

### 📍 Output Locations

All scripts output to the current directory by default:
- **Catalog**: `RCT2-Objects-Catalog.md`
- **Full extraction**: `extracted-objects/` directory
- **Trees only**: `extracted-trees/` directory

**Custom Paths**: You can override with environment variables:
```bash
export RCT2_OBJDATA_DIR="/path/to/your/RCT2/ObjData"
export RCT2_OUTPUT_DIR="/custom/output/path"
./extract_all_rct2_objects.sh
```

---

## 📋 Available Scripts

### 1. Quick Catalog (Recommended First!)
**File**: `catalog_rct2_objects.sh`
**Runtime**: ~30 seconds
**Output**: Single markdown file with complete listing

**What it does**:
- Scans all DAT files
- Extracts metadata only (no sprites)
- Creates organized catalog by category
- Shows object names and types
- Very fast overview

**Run with**:
```bash
cd tools/rct2-extraction
./catalog_rct2_objects.sh
```

**Output**:
- `tools/rct2-extraction/RCT2-Objects-Catalog.md` - Complete listing of all objects

**When to use**:
- ✅ Want to see what's available quickly
- ✅ Need to browse before extracting
- ✅ Don't need sprites yet
- ✅ Want fast results

---

### 2. Full Extraction (Complete Archive)
**File**: `extract_all_rct2_objects.sh`
**Runtime**: ~15-30 minutes
**Output**: Organized directory structure with all sprites

**What it does**:
- Extracts ALL objects with sprites
- Organizes by category (trees, scenery, walls, etc.)
- Creates comprehensive indexes
- Generates category-specific listings
- Preserves all metadata

**Run with**:
```bash
cd tools/rct2-extraction
./extract_all_rct2_objects.sh
```

**Output**:
- `tools/rct2-extraction/extracted-objects/` - Full archive with organized folders

**When to use**:
- ✅ Want complete sprite library
- ✅ Need to browse visually
- ✅ Planning to use many objects
- ✅ Want permanent archive

---

## 🚀 Recommended Workflow

### Step 1: Quick Catalog (30 seconds)

```bash
cd tools/rct2-extraction
./catalog_rct2_objects.sh
```

This creates `RCT2-Objects-Catalog.md` in the current directory showing:
- All available objects
- Names and categories
- Object counts per category

**Example output**:
```
## TREES (28 objects)
- **TCF**: Caucasian Fir Tree
- **TSP**: Scots Pine Tree
- **TRF**: Red Fir Tree
...

## SMALL_SCENERY (287 objects)
- **FLOWER1**: Red Flowers
- **BENCH1**: Wood Bench
...
```

### Step 2: Review Catalog

Open the catalog and identify what you want:
```bash
open tools/rct2-extraction/RCT2-Objects-Catalog.md
# Or if you're in the tools/rct2-extraction directory:
open RCT2-Objects-Catalog.md
```

Browse by category:
- Trees: See all ~28 tree varieties
- Small Scenery: Hundreds of decorations
- Walls: All fence/barrier types
- etc.

### Step 3A: Extract Specific Objects (Fast)

If you only need a few objects, extract individually:

```bash
DOTNET="/opt/homebrew/opt/dotnet@6/bin/dotnet"
OBJEXPORT="/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"
OBJDATA="/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData"

# Extract specific tree
"$DOTNET" "$OBJEXPORT" "$OBJDATA/TOAS.DAT" ~/Downloads/RCT2-Trees/TOAS --png

# Extract specific scenery
"$DOTNET" "$OBJEXPORT" "$OBJDATA/FLOWER1.DAT" ~/Downloads/RCT2-Scenery/FLOWER1 --png
```

### Step 3B: Extract Everything (Complete Archive)

If you want the full library:

```bash
cd tools/rct2-extraction
./extract_all_rct2_objects.sh
```

Wait ~15-30 minutes, then you'll have:
- Complete organized archive in `extracted-objects/`
- All sprites extracted
- All metadata preserved
- Easy browsing by category

---

## 📊 What You'll Get

### From Catalog Script

**File**: `~/Downloads/RCT2-Objects-Catalog.md`

```markdown
# RCT2 Object Catalog

Generated on: 2025-10-13

**Total Objects**: 678

## Summary by Category
- **trees**: 28 objects
- **small_scenery**: 287 objects
- **large_scenery**: 89 objects
- **walls**: 67 objects
- **paths**: 24 objects
- **path_additions**: 43 objects
- **rides**: 124 objects
...

## TREES (28 objects)
- **TCF**: Caucasian Fir Tree
- **TSP**: Scots Pine Tree
- **TOAS**: Oasis Palm Tree
...
```

### From Full Extraction

**Directory**: `tools/rct2-extraction/extracted-objects/`

```
extracted-objects/
├── INDEX.md (Master index)
├── trees/
│   ├── INDEX.md
│   ├── TREES_QUICKREF.md
│   ├── TCF/rct2.tcf/
│   │   ├── images.png (4 views, 4.5 KB)
│   │   └── object.json (metadata)
│   ├── TSP/rct2.tsp/
│   ├── TOAS/rct2.toas/
│   └── ... (25 more trees)
├── small_scenery/ (287 objects!)
├── large_scenery/ (89 objects)
├── walls/ (67 objects)
├── path_banners/
├── paths/
├── path_additions/
├── scenery_groups/
├── park_entrance/
├── water/
├── rides/ (124 objects - LARGE!)
└── unknown/
```

---

## 📦 Expected Object Counts

Based on RCT2 base game + expansions:

| Category | Expected Count | Description |
|----------|---------------|-------------|
| **trees** | ~25-30 | Grass, desert, snow variants |
| **small_scenery** | ~250-300 | Flowers, statues, fountains, etc. |
| **large_scenery** | ~80-100 | Castles, landmarks, buildings |
| **walls** | ~60-80 | Fences, barriers, hedges |
| **path_banners** | ~20-30 | Signs and banners |
| **paths** | ~20-30 | Footpath surfaces |
| **path_additions** | ~30-50 | Benches, lamps, bins |
| **scenery_groups** | ~20-30 | Theme collections |
| **park_entrance** | ~5-10 | Entrance structures |
| **water** | ~5-10 | Water types |
| **rides** | ~100-150 | Ride vehicles/tracks |
| **unknown** | ~10-20 | Miscellaneous |

**Total**: ~600-800 objects

---

## 💡 Use Cases

### Use Case 1: "I just want more tree variety"

```bash
# Step 1: Run catalog (30 sec)
cd tools/rct2-extraction
./catalog_rct2_objects.sh

# Step 2: Find trees in catalog
open RCT2-Objects-Catalog.md
# Scroll to "TREES" section

# Step 3: Extract specific trees you want
DOTNET="/opt/homebrew/opt/dotnet@6/bin/dotnet"
OBJEXPORT="/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"
OBJDATA="/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData"

"$DOTNET" "$OBJEXPORT" "$OBJDATA/TOAS.DAT" ~/Downloads/TOAS --png  # Palm tree
"$DOTNET" "$OBJEXPORT" "$OBJDATA/TWW.DAT" ~/Downloads/TWW --png   # Willow

# Step 4: Copy to Godot
cp ~/Downloads/TOAS/rct2.toas/images.png godot-viewer/assets/sprites/vegetation/trees/tree_palm_oasis.png
cp ~/Downloads/TWW/rct2.tww/images.png godot-viewer/assets/sprites/vegetation/trees/tree_willow_weeping.png
```

### Use Case 2: "I want a complete RCT2 asset library"

```bash
# Just run full extraction
cd tools/rct2-extraction
./extract_all_rct2_objects.sh

# Wait 15-30 minutes
# Browse complete archive:
open extracted-objects/INDEX.md
```

### Use Case 3: "I need decorative scenery for buildings"

```bash
# Step 1: Run catalog
cd tools/rct2-extraction
./catalog_rct2_objects.sh

# Step 2: Browse small_scenery and large_scenery sections
open RCT2-Objects-Catalog.md

# Step 3: Extract full archive (or specific objects)
./extract_all_rct2_objects.sh

# Step 4: Browse extracted sprites
open extracted-objects/small_scenery/INDEX.md
open extracted-objects/large_scenery/INDEX.md
```

---

## 🎯 Quick Start (Recommended)

### For Beginners

1. **Run the catalog first** (30 seconds):
   ```bash
   cd tools/rct2-extraction
   ./catalog_rct2_objects.sh
   ```

2. **Browse what's available**:
   ```bash
   open RCT2-Objects-Catalog.md
   ```

3. **Decide**: Do I need everything or just specific objects?

4. **If specific**: Extract individually (fast)
   **If everything**: Run full extraction script

### For Power Users

Just run the full extraction and have everything:
```bash
cd tools/rct2-extraction
./extract_all_rct2_objects.sh
# Go make coffee, come back in 20 minutes
```

---

## 🔧 Technical Details

### Prerequisites

Both scripts require:
- ✅ RCT2 installation with ObjData directory
- ✅ .NET 6.0 SDK (`brew install dotnet@6`)
- ✅ OpenRCT2 objexport tool (built from source)

### Performance

**Catalog Script**:
- Runtime: ~30 seconds
- Disk space: <1 MB (single markdown file)
- Memory: Minimal

**Full Extraction**:
- Runtime: ~15-30 minutes (depends on object count)
- Disk space: ~100-200 MB
- Memory: Moderate
- CPU: One object at a time (could parallelize)

### Output Formats

**Sprites**: PNG with transparency, 8-bit colormap
**Metadata**: JSON with complete object properties
**Indexes**: Markdown for easy browsing

---

## 📝 Files Created

### Scripts
- ✅ `extract_all_rct2_objects.sh` - Full extraction
- ✅ `catalog_rct2_objects.sh` - Quick catalog
- ✅ `extract_rct2_trees.sh` - Trees only (already have)
- ✅ `EXTRACTION_GUIDE.md` - Detailed guide
- ✅ `RCT2_EXTRACTION_SCRIPTS_README.md` - This file

### Output Files
- `~/Downloads/RCT2-Objects-Catalog.md` - Catalog listing
- `~/Downloads/RCT2-Objects-Complete/` - Full archive
- Individual extractions as needed

---

## ❤️ What Makes This Special

### Organized by Purpose
Not just a dump - intelligently categorized by object type and use case.

### Complete Metadata
Every object includes:
- Name (multilingual)
- Type and properties
- Height, cost, flags
- Animation data
- Image count

### Easy to Browse
- Category indexes
- Quick reference guides
- Search-friendly markdown
- Visual sprite atlases

### Authentic Art
Original Chris Sawyer pixel art from 2002! 🎮

---

## 🎉 Summary

**Two approaches**:

1. **Quick Browse** → Catalog script (30 sec) → Extract specific objects
2. **Complete Archive** → Full extraction (20 min) → Have everything

**Recommendation**: Start with catalog, then decide!

```bash
# Start here:
cd tools/rct2-extraction
./catalog_rct2_objects.sh

# Then either:
# Option A: Extract what you need individually
# Option B: Run full extraction for complete library
./extract_all_rct2_objects.sh
```

---

**Created**: 2025-10-13
**Status**: Ready to run!
**Expected**: ~600-800 objects available
**Nostalgia**: Maximum! 🌲🎢🎮
