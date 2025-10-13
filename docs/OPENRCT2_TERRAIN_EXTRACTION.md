# OpenRCT2 Terrain Extraction Guide

**Complete guide to extracting and organizing OpenRCT2 terrain sprites for slope-based rendering**

## Overview

This document describes how we extracted all 29,284 sprites from RollerCoaster Tycoon 2's `g1.dat` file and organized 13 terrain types with their correct OpenRCT2 names for use in the life-simulator Godot viewer.

## Table of Contents

1. [Extraction Process](#extraction-process)
2. [Finding Correct Sprite Indices](#finding-correct-sprite-indices)
3. [Terrain Types Extracted](#terrain-types-extracted)
4. [Atlas Creation](#atlas-creation)
5. [Scripts Reference](#scripts-reference)
6. [Using the Sprites](#using-the-sprites)
7. [Lessons Learned](#lessons-learned)

---

## Extraction Process

### Step 1: Extract All Sprites on Windows

We used **Trigger's Graphics Extractor** (Windows tool) to extract all sprites from `g1.dat`:

**Windows Setup:**
1. Download Trigger's Graphics Extractor
2. Open RCT2's `g1.dat` file: `Data/g1.dat` from RCT2 installation
3. Configure extraction:
   - **Format:** PNG
   - **Numbering:** Decimal (NOT hex)
   - **Extract RCT2 Images:** Checked
4. Click "Extract" → Wait ~65 seconds
5. Result: 29,284 PNG files numbered `0.png` through `29283.png`

**Why Decimal?**
- OpenRCT2 source code uses decimal sprite indices
- Makes it easy to match sprites with documentation
- Example: `SPR_TERRAIN_GRASS = 1915` (decimal)

### Step 2: Transfer to Mac

We created a permanent sprite library on Mac:

```bash
# On Windows: Zip all extracted sprites
# Transfer zip to Mac via USB/network/cloud

# On Mac: Create permanent library
mkdir -p ~/RCT2-Sprites
unzip rct2-sprites.zip -d ~/RCT2-Sprites/

# Result: 29,284 sprites in ~/RCT2-Sprites/
ls ~/RCT2-Sprites | wc -l
# Output: 29284
```

**Why permanent library?**
- No need to re-extract sprites
- Available for all projects
- Outside git (29K files too large for version control)
- Fast access: `~/RCT2-Sprites/1915.png`

---

## Finding Correct Sprite Indices

### Problem: Wrong Initial Indices

Initial documentation claimed grass was at sprites `3419-3437`, but these were actually:
- 3400-3404: Wooden floor planks (ride components)
- 3419-3424: Light green solid tiles
- 3426+: Gray stone tiles (not grass!)

### Solution: Check OpenRCT2 Source Code

We found the **correct sprite indices** in OpenRCT2's source:

```bash
cd /Users/jean/Github/OpenRCT2
grep "SPR_TERRAIN_" src/openrct2/paint/tile_element/Paint.Surface.h
```

**Key file:** `src/openrct2/paint/tile_element/Paint.Surface.h`

```cpp
// Actual terrain sprite definitions
SPR_TERRAIN_GRASS = 1915,
SPR_TERRAIN_SAND = 1972,
SPR_TERRAIN_SAND_RED = 2029,
SPR_TERRAIN_SAND_YELLOW = 2086,
SPR_TERRAIN_ICE = 2143,
SPR_TERRAIN_GRASS_CLUMPS = 2200,
SPR_TERRAIN_MARTIAN = 2314,
SPR_TERRAIN_CHECKERBOARD = 2371,
SPR_TERRAIN_CHECKERBOARD_INVERTED = 2428,
SPR_TERRAIN_DIRT = 2485,
SPR_TERRAIN_ROCK = 2542,
SPR_TERRAIN_GRASS_MOWED = 2663,
SPR_TERRAIN_GRASS_MOWED_90 = 2701,
```

### Grass Terrain Types Comparison

**We discovered three grass variants:**

| Type | Sprites | Description | Usage |
|------|---------|-------------|-------|
| **grass** | 1915-1933 | Lush fully-covered green grass | **Primary terrain** ✅ |
| **grass_clumps** | 2200-2218 | Natural grass with dirt patches | Wild/natural areas |
| **grass_mowed** | 2663-2681 | Short cut grass | Mowed lawns |

**Decision:** We chose **grass (1915-1933)** as the primary terrain because:
- Fully covered, lush appearance
- Matches typical RCT2 park grass
- Better visual quality for simulation

---

## Terrain Types Extracted

We extracted **13 complete terrain types**, each with **19 slope variations** (slope_00.png through slope_18.png):

| # | Terrain Name | Sprites | Description | Atlas |
|---|--------------|---------|-------------|-------|
| 1 | **grass** | 1915-1933 | Lush fully-covered grass | grass_atlas.png |
| 2 | sand | 1972-1990 | Beach/desert sand | sand_atlas.png |
| 3 | sand_red | 2029-2047 | Red/martian sand | sand_red_atlas.png |
| 4 | sand_yellow | 2086-2104 | Yellow sand variant | sand_yellow_atlas.png |
| 5 | ice | 2143-2161 | Ice/snow terrain | ice_atlas.png |
| 6 | grass_clumps | 2200-2218 | Natural grass with dirt | grass_clumps_atlas.png |
| 7 | martian | 2314-2332 | Martian/alien surface | martian_atlas.png |
| 8 | checkerboard | 2371-2389 | Debug checkerboard | checkerboard_atlas.png |
| 9 | checkerboard_inverted | 2428-2446 | Inverted checkerboard | checkerboard_inverted_atlas.png |
| 10 | dirt | 2485-2503 | Brown dirt paths | dirt_atlas.png |
| 11 | rock | 2542-2560 | Rocky terrain | rock_atlas.png |
| 12 | grass_mowed | 2663-2681 | Short mowed grass | grass_mowed_atlas.png |
| 13 | grass_mowed_90 | 2701-2719 | Mowed grass 90° variant | grass_mowed_90_atlas.png |

**Total Assets:**
- 13 terrain types
- 19 slopes per terrain = 247 individual sprites
- 13 atlases (640×128 pixels)
- All using **exact OpenRCT2 names**

---

## Atlas Creation

### Why Atlases?

Atlases improve performance in game engines:
- Single texture load vs. 19 individual sprites
- Faster rendering (texture switching overhead)
- Easier TileSet configuration in Godot

### Atlas Specifications

- **Size:** 640×128 pixels
- **Grid:** 10×2 (10 columns, 2 rows)
- **Cell Size:** 64×64 pixels
- **Format:** RGBA PNG with transparency
- **Alignment:** Sprites centered horizontally, bottom-aligned vertically

### Example: Grass Atlas

```
Row 0: [slope_00] [slope_01] [slope_02] ... [slope_09]
Row 1: [slope_10] [slope_11] [slope_12] ... [slope_18]

Each cell: 64×64 pixels
Sprite dimensions vary (52-64 wide, 15-63 tall)
Sprites aligned to bottom of cell for proper ground placement
```

**Visual structure:**
- Flat slopes (slope_00, slope_03, slope_04, slope_06, slope_07): Mostly diamond-shaped
- Raised slopes (slope_08, slope_09, slope_15): Taller, show elevation
- Empty sprite at slope_16: 1×1 pixel (placeholder in original RCT2 data)

---

## Scripts Reference

We created three Python scripts to automate the process:

### 1. `extract_all_terrains.py`

**Purpose:** Copy sprites from permanent library to project directories

```python
# Usage
python3 extract_all_terrains.py

# What it does:
# 1. Reads terrain definitions (name, start_sprite, description)
# 2. For each terrain:
#    - Creates output directory
#    - Copies 19 sprites from ~/RCT2-Sprites/
#    - Renames: 1915.png → slope_00.png, 1916.png → slope_01.png, etc.
# 3. Reports success/failure for each terrain

# Output:
godot-viewer/assets/tiles/terrain/openrct2_placeholder/
├── grass/
│   ├── slope_00.png
│   ├── slope_01.png
│   └── ... (19 total)
├── sand/
│   ├── slope_00.png
│   └── ...
└── ... (13 terrain types)
```

**Key features:**
- Uses exact OpenRCT2 sprite indices
- Skips nearly-empty sprites (< 200 bytes)
- Progress reporting for each sprite
- Summary table at end

### 2. `create_terrain_atlases.py`

**Purpose:** Generate 640×128 atlases for all terrain types

```python
# Usage
python3 create_terrain_atlases.py

# What it does:
# 1. For each terrain in project directory:
#    - Loads all 19 slope sprites
#    - Finds max dimensions
#    - Creates 640×128 blank atlas (RGBA, transparent)
#    - Places each sprite in 64×64 cell (10×2 grid)
#    - Centers sprite horizontally, aligns to bottom
#    - Saves as <terrain>_atlas.png
# 2. Reports success for each atlas

# Output: 13 atlas files
grass_atlas.png
sand_atlas.png
...
grass_mowed_90_atlas.png
```

**Alignment logic:**
```python
# Center sprite in 64×64 cell
paste_x = cell_x + (64 - sprite.width) // 2  # Horizontal center
paste_y = cell_y + (64 - sprite.height)      # Bottom align (for ground)
```

### 3. `create_grass_atlas.py`

**Purpose:** Legacy script for single terrain (grass only)

Now superseded by `create_terrain_atlases.py` which handles all terrains.

---

## Using the Sprites

### In Godot TileSet

**Configuration:**
```gdscript
# TileSet properties
tile_shape = TileSet.TILE_SHAPE_ISOMETRIC  # Diamond tiles
tile_layout = TileSet.TILE_LAYOUT_STACKED  # Row-by-row
tile_size = Vector2i(64, 32)  # Isometric diamond

# Load grass atlas
var atlas_source = TileSetAtlasSource.new()
atlas_source.texture = load("res://assets/tiles/terrain/openrct2_placeholder/grass/grass_atlas.png")
atlas_source.texture_region_size = Vector2i(64, 64)  # Cell size in atlas

# Create tiles for each slope (0-18)
for i in range(19):
    var atlas_coords = Vector2i(i % 10, i / 10)  # Map index to grid
    atlas_source.create_tile(atlas_coords)
```

**Painting tiles:**
```gdscript
# Get slope index from terrain calculation (0-18)
var slope_index = calculate_slope_from_neighbors(tile_pos)

# Convert slope index to atlas coordinates
var atlas_coords = Vector2i(slope_index % 10, slope_index / 10)

# Paint tile
tilemap.set_cell(0, tile_pos, source_id, atlas_coords)
```

### Adding New Terrain Types

If you want to add more terrain types:

```bash
# 1. Check OpenRCT2 source for sprite index
cd /Users/jean/Github/OpenRCT2
grep "SPR_TERRAIN_" src/openrct2/paint/tile_element/Paint.Surface.h

# 2. Add to extract_all_terrains.py TERRAINS list
("water", 2105, "Water terrain"),  # Example

# 3. Re-run extraction
python3 extract_all_terrains.py

# 4. Create atlas
python3 create_terrain_atlases.py
```

### Sprite Library Access

**Permanent location:** `~/RCT2-Sprites/`

**Quick access:**
```bash
# View any sprite
open ~/RCT2-Sprites/1915.png

# Check sprite info
identify ~/RCT2-Sprites/1915.png
# Output: 1915.png PNG 64x31 64x31+0+0 8-bit sRGB 722B

# Copy specific sprite to project
cp ~/RCT2-Sprites/1915.png /path/to/project/
```

**Search sprites:**
```bash
# Find sprites in range
for i in {1915..1933}; do
  echo "Sprite $i: $(identify ~/RCT2-Sprites/$i.png | cut -d' ' -f3)"
done
```

---

## Lessons Learned

### 1. Always Use Source Code for Truth

**Problem:** Documentation often has incorrect sprite indices.

**Solution:** Check OpenRCT2's actual source code (`Paint.Surface.h`) for definitive sprite numbers.

**Tools:**
```bash
# Find sprite definitions
grep "SPR_TERRAIN" src/openrct2/paint/tile_element/Paint.Surface.h

# Find specific sprite usage
grep -r "1915" --include="*.cpp" src/
```

### 2. Use Exact OpenRCT2 Names

**Why it matters:**
- Easy to reference OpenRCT2 documentation
- Clear which terrain variant you're using
- No confusion between grass, grass_clumps, grass_mowed

**Naming convention:**
- Folder name: `grass` (matches `SPR_TERRAIN_GRASS`)
- Atlas name: `grass_atlas.png` (consistent pattern)
- Lowercase with underscores (programming-friendly)

### 3. Decimal Numbering for Extraction

**Always extract with decimal numbering, not hex:**
- OpenRCT2 uses decimal in source
- Easier to match sprite indices
- Simpler scripting (no hex conversion)

### 4. Permanent Sprite Library

**Keep all 29K sprites in `~/RCT2-Sprites/`:**
- No re-extraction needed
- Fast access to any sprite
- Available for multiple projects
- Only commit project-specific sprites to git

### 5. Visual Verification

**Always visually check extracted sprites:**
```bash
# We did this by viewing sprites:
open ~/RCT2-Sprites/1915.png  # Grass
open ~/RCT2-Sprites/2200.png  # Grass clumps
open ~/RCT2-Sprites/3419.png  # NOT grass (wooden floor)
```

**This caught the wrong indices early!**

### 6. Atlas Cell Size

**Use 64×64 cells for RCT2 sprites:**
- Max sprite dimensions: 64×63 (from our analysis)
- 64×64 accommodates all slopes with margin
- Clean power-of-2 size for GPU
- Matches common tile sizes

### 7. Bottom Alignment

**Always align sprites to bottom of cell:**
```python
paste_y = cell_y + (cell_height - sprite.height)
```

**Why:**
- Ground level should be at cell bottom
- Matches isometric tile rendering
- Proper visual stacking of elevated tiles

### 8. Automation Scripts

**Create reusable scripts for:**
- Extraction (bulk copying from library)
- Atlas generation (for all terrains)
- Validation (check all sprites exist)

**Benefits:**
- Reproducible process
- Easy to add new terrains
- Consistent results
- Documentation via code

---

## File Organization

### Project Structure

```
life-simulator/
├── docs/
│   ├── OPENRCT2_TERRAIN_EXTRACTION.md  # This file
│   └── RCT2_SPRITE_LIBRARY.md          # Sprite indices reference
├── extract_all_terrains.py             # Bulk extraction script
├── create_terrain_atlases.py           # Atlas generation script
└── godot-viewer/
    └── assets/
        └── tiles/
            └── terrain/
                └── openrct2_placeholder/
                    ├── grass/
                    │   ├── slope_00.png ... slope_18.png
                    │   └── grass_atlas.png
                    ├── sand/
                    │   ├── slope_00.png ... slope_18.png
                    │   └── sand_atlas.png
                    └── ... (11 more terrain types)
```

### External Library

```
~/RCT2-Sprites/
├── README.md              # Library documentation
├── 0.png                  # Sprite index 0
├── 1.png                  # Sprite index 1
├── ...
├── 1915.png              # Grass slope 0
├── 1916.png              # Grass slope 1
├── ...
└── 29283.png             # Last sprite
```

---

## Quick Reference

### Common Sprite Ranges

| Terrain | Start | End | Count |
|---------|-------|-----|-------|
| Grass | 1915 | 1933 | 19 |
| Sand | 1972 | 1990 | 19 |
| Grass Clumps | 2200 | 2218 | 19 |
| Dirt | 2485 | 2503 | 19 |
| Rock | 2542 | 2560 | 19 |
| Grass Mowed | 2663 | 2681 | 19 |

### Slope Indices

| Index | Slope Type | Description |
|-------|------------|-------------|
| 0 | Flat | Completely flat |
| 1-4 | Single edge | N, E, S, W raised |
| 5-8 | Corner | NE, SE, SW, NW raised |
| 9-10 | Ridge | NS, EW ridges |
| 11-14 | 3 edges | Various combinations |
| 15 | Bowl | All edges raised |
| 16 | (Empty) | Placeholder |
| 17-18 | Double | E, S double height |

### Commands

```bash
# Extract all terrains
python3 extract_all_terrains.py

# Create all atlases
python3 create_terrain_atlases.py

# View sprite
open ~/RCT2-Sprites/1915.png

# Check sprite info
identify ~/RCT2-Sprites/1915.png
```

---

## Credits

- **RollerCoaster Tycoon 2**: Chris Sawyer, Atari
- **OpenRCT2**: Open-source RCT2 engine project
- **Trigger's Graphics Extractor**: Windows extraction tool
- **Sprite indices source**: `OpenRCT2/src/openrct2/paint/tile_element/Paint.Surface.h`

---

## Appendix: Full Terrain List

```python
TERRAINS = [
    ("grass", 1915, "Lush fully-covered grass"),
    ("sand", 1972, "Beach/desert sand"),
    ("sand_red", 2029, "Red/martian sand"),
    ("sand_yellow", 2086, "Yellow sand variant"),
    ("ice", 2143, "Ice/snow terrain"),
    ("grass_clumps", 2200, "Natural grass with dirt patches"),
    ("martian", 2314, "Martian/alien surface"),
    ("checkerboard", 2371, "Debug checkerboard pattern"),
    ("checkerboard_inverted", 2428, "Inverted checkerboard"),
    ("dirt", 2485, "Brown dirt paths"),
    ("rock", 2542, "Rocky terrain"),
    ("grass_mowed", 2663, "Short mowed grass"),
    ("grass_mowed_90", 2701, "Mowed grass 90° variant"),
]
```

Each terrain has 19 slopes numbered 0-18, except slope 16 which is a 1×1 placeholder in original RCT2 data.

---

**Last Updated:** 2025-01-13
**Status:** Complete - All 13 terrain types extracted and documented
