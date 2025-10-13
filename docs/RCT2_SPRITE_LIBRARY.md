# RCT2 Sprite Library Reference

**Permanent sprite library location**: `~/RCT2-Sprites/`

This directory contains all 29,284 sprites extracted from RollerCoaster Tycoon 2's `g1.dat` file.

## Quick Access

All sprites are available at `~/RCT2-Sprites/<index>.png`

Example:
```bash
# View grass terrain flat slope
open ~/RCT2-Sprites/3419.png

# Check sprite dimensions
identify ~/RCT2-Sprites/3419.png
```

## Common Terrain Ranges

**Source:** OpenRCT2 `src/openrct2/paint/tile_element/Paint.Surface.h`

| Terrain Type | Sprite Range | Count | Notes |
|--------------|--------------|-------|-------|
| **Grass (Clumps)** | **2200-2218** | 19 | **✅ In project** - Textured grass with detail |
| Grass (Flat) | 1915-1933 | 19 | Solid green, less detail |
| Grass (Mowed) | 2663-2681 | 19 | Short cut grass |
| Sand | 1972-1990 | 19 | Beach/desert terrain |
| Dirt | 1991-2009 | 19 | Brown dirt paths |
| Rock | 2010-2028 | 19 | Rocky terrain |
| Martian | 2029-2047 | 19 | Red/alien surface |
| Checkerboard | 2048-2066 | 19 | Debug pattern |
| Grass Clumps Grid | 2219-2237 | 19 | With grid overlay |

## Adding New Terrain to Project

### Step 1: Copy sprites from library

```bash
# Example: Add sand terrain
START_INDEX=1972
TERRAIN_NAME="sand"

for i in {0..18}; do
  cp ~/RCT2-Sprites/$((START_INDEX + i)).png \
     godot-viewer/assets/tiles/terrain/openrct2_placeholder/${TERRAIN_NAME}/slope_$(printf %02d $i).png
done
```

### Step 2: Create atlas (if needed)

```bash
# Modify create_grass_atlas.py for new terrain
cp create_grass_atlas.py create_${TERRAIN_NAME}_atlas.py

# Update paths in script:
# grass_dir = "godot-viewer/assets/tiles/terrain/openrct2_placeholder/sand"

# Run atlas creation
python3 create_${TERRAIN_NAME}_atlas.py
```

### Step 3: Integrate in Godot

Update `godot-viewer/scripts/Config.gd` with new terrain colors:

```gdscript
var TERRAIN_COLORS = {
    # ... existing terrain types ...
    "Sand": Color(0.96, 0.89, 0.74),  # Light sandy color
}
```

## Slope Indices Reference

Each terrain type has 19 slope variations:

| Index | Slope Type | Description |
|-------|------------|-------------|
| 0 | Flat | Completely flat tile |
| 1 | N | North edge raised |
| 2 | E | East edge raised |
| 3 | S | South edge raised |
| 4 | W | West edge raised |
| 5 | NE | Northeast corner |
| 6 | SE | Southeast corner |
| 7 | SW | Southwest corner |
| 8 | NW | Northwest corner |
| 9 | NS | North-south ridge |
| 10 | EW | East-west ridge |
| 11 | NES | Three edges up (no W) |
| 12 | ESW | Three edges up (no N) |
| 13 | NSW | Three edges up (no E) |
| 14 | NEW | Three edges up (no S) |
| 15 | NESW | All edges up (bowl) |
| 16 | N2 | North double height |
| 17 | E2 | East double height |
| 18 | S2 | South double height |

## Current Project Usage

### ✅ All Terrain Types Extracted (13 total)

All terrains include:
- 19 slope variations (slope_00 to slope_18)
- Individual PNG sprites
- 640×128 atlas (64×64 cells, 10×2 grid)

| # | Terrain | Sprites | Atlas | Status |
|---|---------|---------|-------|--------|
| 1 | **grass** | 1915-1933 | grass_atlas.png | ✅ **Primary** |
| 2 | sand | 1972-1990 | sand_atlas.png | ✅ Ready |
| 3 | sand_red | 2029-2047 | sand_red_atlas.png | ✅ Ready |
| 4 | sand_yellow | 2086-2104 | sand_yellow_atlas.png | ✅ Ready |
| 5 | ice | 2143-2161 | ice_atlas.png | ✅ Ready |
| 6 | grass_clumps | 2200-2218 | grass_clumps_atlas.png | ✅ Ready |
| 7 | martian | 2314-2332 | martian_atlas.png | ✅ Ready |
| 8 | checkerboard | 2371-2389 | checkerboard_atlas.png | ✅ Ready |
| 9 | checkerboard_inverted | 2428-2446 | checkerboard_inverted_atlas.png | ✅ Ready |
| 10 | dirt | 2485-2503 | dirt_atlas.png | ✅ Ready |
| 11 | rock | 2542-2560 | rock_atlas.png | ✅ Ready |
| 12 | grass_mowed | 2663-2681 | grass_mowed_atlas.png | ✅ Ready |
| 13 | grass_mowed_90 | 2701-2719 | grass_mowed_90_atlas.png | ✅ Ready |

**Location:** `godot-viewer/assets/tiles/terrain/openrct2_placeholder/<terrain>/`

**Total Assets:**
- 247 individual sprites (13 × 19)
- 13 atlases ready for Godot integration

### Complete Extraction Guide

For full details on how these were extracted and organized, see:
- **[OPENRCT2_TERRAIN_EXTRACTION.md](OPENRCT2_TERRAIN_EXTRACTION.md)** - Complete extraction process
- Includes Windows extraction steps
- Finding correct sprite indices from OpenRCT2 source
- Automated extraction and atlas generation scripts
- Lessons learned and best practices

## Object Sprites (Future)

Beyond terrain, the library includes:
- Trees and vegetation
- Buildings and structures
- Ride components
- Path decorations
- Water features

**TODO**: Document object sprite ranges when needed.

## Tools & Scripts

### Extraction Scripts (for reference only)
- `extract_rct2_sprites.py` - Full extraction (not needed, already extracted)
- `extract_grass_simple.py` - Grass-only extraction (superseded by Windows extraction)
- `extract_palette_from_g1.py` - Palette extraction (reference)

### Atlas Creation
- `create_terrain_atlases.py` - Creates 640×128 atlases for ALL terrain types
- `create_grass_atlas.py` - Legacy single-terrain script (superseded)

### Sprite Library Maintenance
- Location: `~/RCT2-Sprites/`
- README: `~/RCT2-Sprites/README.md`
- No git repo (too large, doesn't change)

## Tips

1. **Always copy from `~/RCT2-Sprites/`**, never modify originals
2. **Rename to `slope_XX.png`** when copying to project (for consistency)
3. **Create atlases** for better performance in game engines
4. **Document sprite ranges** when adding new features
5. **Check dimensions** with `identify` before creating atlases

## See Also

- **[OPENRCT2_TERRAIN_EXTRACTION.md](OPENRCT2_TERRAIN_EXTRACTION.md)** - Complete extraction guide
- OpenRCT2 docs: https://github.com/OpenRCT2/OpenRCT2
- `godot-viewer/assets/tiles/terrain/README.md` - Terrain asset organization
- `~/RCT2-Sprites/README.md` - Full sprite library documentation
