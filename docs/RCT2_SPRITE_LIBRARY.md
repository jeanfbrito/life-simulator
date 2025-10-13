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

### Implemented
- ✅ **Grass terrain - Clumps** (2200-2218)
  - Location: `godot-viewer/assets/tiles/terrain/openrct2_placeholder/grass/`
  - Atlas: `grass_atlas.png` (640×128, 64×64 tiles)
  - Textured grass with detail and natural appearance
  - Status: Ready for integration

### Planned
- ⏳ Sand terrain (beach areas)
- ⏳ Dirt terrain (paths)
- ⏳ Stone terrain (mountains)

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
- `create_grass_atlas.py` - Creates 640×128 atlas from 19 slope sprites
- Can be adapted for other terrain types

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

- OpenRCT2 docs: https://github.com/OpenRCT2/OpenRCT2
- `godot-viewer/assets/tiles/terrain/README.md` - Terrain asset organization
- `docs/OPENRCT2_SPRITE_EXTRACTION_GUIDE.md` - Original extraction guide
- `~/RCT2-Sprites/README.md` - Full sprite library documentation
