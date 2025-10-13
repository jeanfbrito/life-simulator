# OpenRCT2 Terrain Extraction - Project Summary

**Date:** 2025-01-13
**Status:** ✅ Complete

## What We Accomplished

### 1. Full Sprite Library Extraction
- ✅ Extracted **all 29,284 sprites** from RCT2's `g1.dat` using Trigger's Graphics Extractor (Windows)
- ✅ Created permanent sprite library at `~/RCT2-Sprites/`
- ✅ Documented complete extraction process

### 2. Found Correct Sprite Indices
- ✅ Discovered documentation had wrong indices (3419 was NOT grass)
- ✅ Checked OpenRCT2 source code (`Paint.Surface.h`) for truth
- ✅ Found correct grass at sprites **1915-1933** (lush fully-covered)
- ✅ Identified 13 terrain types with exact OpenRCT2 names

### 3. Extracted All Terrain Types
- ✅ **13 terrain types** × 19 slopes = **247 sprites**
- ✅ Using exact OpenRCT2 names (grass, sand, rock, etc.)
- ✅ Each terrain in separate folder with slope_00.png through slope_18.png
- ✅ Cleaned up old placeholder folders

### 4. Created Atlases
- ✅ **13 atlases** (640×128 pixels, 10×2 grid of 64×64 cells)
- ✅ Each terrain has `<terrain>_atlas.png` ready for Godot
- ✅ Sprites centered horizontally, bottom-aligned for proper ground placement

### 5. Automation Scripts
- ✅ `extract_all_terrains.py` - Bulk extraction from sprite library
- ✅ `create_terrain_atlases.py` - Generate all 13 atlases at once
- ✅ Both scripts with progress reporting and validation

### 6. Complete Documentation
- ✅ `docs/OPENRCT2_TERRAIN_EXTRACTION.md` - Complete extraction guide (552 lines)
- ✅ `docs/RCT2_SPRITE_LIBRARY.md` - Sprite reference with all terrain types
- ✅ `~/RCT2-Sprites/README.md` - Library documentation
- ✅ This summary document

## Terrain Types Extracted

| # | Name | Sprites | Description | Atlas |
|---|------|---------|-------------|-------|
| 1 | grass | 1915-1933 | Lush fully-covered grass | grass_atlas.png |
| 2 | sand | 1972-1990 | Beach/desert sand | sand_atlas.png |
| 3 | sand_red | 2029-2047 | Red/martian sand | sand_red_atlas.png |
| 4 | sand_yellow | 2086-2104 | Yellow sand variant | sand_yellow_atlas.png |
| 5 | ice | 2143-2161 | Ice/snow terrain | ice_atlas.png |
| 6 | grass_clumps | 2200-2218 | Natural grass with dirt | grass_clumps_atlas.png |
| 7 | martian | 2314-2332 | Martian/alien surface | martian_atlas.png |
| 8 | checkerboard | 2371-2389 | Debug pattern | checkerboard_atlas.png |
| 9 | checkerboard_inverted | 2428-2446 | Inverted checkerboard | checkerboard_inverted_atlas.png |
| 10 | dirt | 2485-2503 | Brown dirt paths | dirt_atlas.png |
| 11 | rock | 2542-2560 | Rocky terrain | rock_atlas.png |
| 12 | grass_mowed | 2663-2681 | Short mowed grass | grass_mowed_atlas.png |
| 13 | grass_mowed_90 | 2701-2719 | Mowed grass 90° | grass_mowed_90_atlas.png |

**Total:** 247 sprites + 13 atlases

## File Locations

### Sprite Library (Permanent)
```
~/RCT2-Sprites/
├── README.md
├── 0.png ... 29283.png  (29,284 sprites)
```

### Project Assets
```
godot-viewer/assets/tiles/terrain/openrct2_placeholder/
├── grass/
│   ├── slope_00.png ... slope_18.png
│   └── grass_atlas.png
├── sand/
│   ├── slope_00.png ... slope_18.png
│   └── sand_atlas.png
└── ... (11 more terrain types)
```

### Scripts
```
extract_all_terrains.py      # Bulk extraction
create_terrain_atlases.py    # Atlas generation
create_grass_atlas.py        # Legacy single-terrain
```

### Documentation
```
docs/
├── OPENRCT2_TERRAIN_EXTRACTION.md  # Complete guide
├── RCT2_SPRITE_LIBRARY.md          # Sprite reference
└── TERRAIN_EXTRACTION_SUMMARY.md   # This file
```

## Key Decisions

### 1. Grass Terrain Choice
**Problem:** Three grass variants exist
- 1915-1933: Lush fully-covered (solid green)
- 2200-2218: Grass clumps (with dirt showing)
- 2663-2681: Mowed grass (short cut)

**Decision:** Use **1915-1933** as primary "grass" terrain
- Best visual quality
- Matches typical RCT2 park appearance
- Fully covered, no dirt patches
- Other variants available as grass_clumps, grass_mowed

### 2. Exact OpenRCT2 Names
**Why:** Using OpenRCT2's exact terrain names
- Easy to reference OpenRCT2 documentation
- No confusion about which variant
- Example: `grass` vs `grass_clumps` vs `grass_mowed`
- Professional consistency

### 3. Permanent Sprite Library
**Why:** Keep all 29K sprites in `~/RCT2-Sprites/`
- No need to re-extract
- Available for all projects
- Fast access to any sprite
- Outside git (too large for version control)

### 4. Atlas Format
**Specifications:**
- Size: 640×128 pixels (10×2 grid)
- Cell: 64×64 pixels each
- Alignment: Center horizontally, bottom-align vertically
- Format: RGBA PNG with transparency

**Why these specs:**
- 64×64 accommodates all slopes (max is 64×63)
- Power-of-2 friendly for GPU
- Bottom alignment for proper ground placement
- 10×2 grid fits 19 slopes cleanly

## Git Commits

```bash
# Commit history (most recent first)
25e2fb6 docs: update sprite library with complete terrain table
e79b429 docs: add complete OpenRCT2 terrain extraction guide
d8d5f71 feat: extract all 13 OpenRCT2 terrain types with correct names
45c14d7 feat: use fully covered grass sprites (1915-1933)
9948c2e docs: correct OpenRCT2 terrain sprite ranges
7acf237 fix: use correct OpenRCT2 grass sprites (2200-2218)
f7eab43 feat: add OpenRCT2 grass terrain sprites (3419-3437)  # Initial wrong indices
```

## Lessons Learned

### 1. Always Check Source Code
Documentation can be wrong. OpenRCT2's `Paint.Surface.h` has the definitive sprite indices.

### 2. Visual Verification is Critical
We caught wrong sprites by viewing them:
- 3419: Light green (not typical grass)
- 1915: Perfect lush grass ✅
- 2200: Grass with dirt patches

### 3. Use Decimal Numbering
Extract sprites with decimal indices, not hex. Matches OpenRCT2 source and simplifies scripting.

### 4. Automate Everything
Scripts ensure:
- Reproducible process
- Consistent results
- Easy to add new terrains
- Self-documenting

### 5. Document While Fresh
Created comprehensive docs immediately after extraction while details are fresh in memory.

## Next Steps

### Integration with Godot
Now ready to implement slope-based terrain rendering:
1. Load atlases in Godot TileSet
2. Calculate slope indices from neighbor heights
3. Paint tiles with appropriate atlas coordinates
4. Support all 13 terrain types

### Future Enhancements
- Height map backend (track tile elevations)
- Slope calculation from height differences
- Smooth terrain transitions
- Multi-terrain blending (edges)

## Quick Reference

### View Any Sprite
```bash
open ~/RCT2-Sprites/1915.png
identify ~/RCT2-Sprites/1915.png
```

### Extract New Terrain
```bash
# 1. Add to extract_all_terrains.py TERRAINS list
# 2. Run extraction
python3 extract_all_terrains.py

# 3. Create atlas
python3 create_terrain_atlases.py
```

### Documentation
- Full extraction guide: `docs/OPENRCT2_TERRAIN_EXTRACTION.md`
- Sprite reference: `docs/RCT2_SPRITE_LIBRARY.md`
- Library README: `~/RCT2-Sprites/README.md`

## Credits

- **RollerCoaster Tycoon 2**: Chris Sawyer, Atari
- **OpenRCT2**: Open-source RCT2 engine (https://openrct2.org/)
- **Trigger's Graphics Extractor**: Windows extraction tool
- **Source reference**: OpenRCT2 `src/openrct2/paint/tile_element/Paint.Surface.h`

---

**Project:** Life Simulator
**Component:** Godot Isometric Viewer
**Purpose:** Slope-based terrain rendering with authentic RCT2 graphics
**Status:** ✅ Complete - Ready for Godot integration
