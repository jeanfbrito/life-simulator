# Terrain Sprites Directory Structure

This directory contains terrain sprites for the isometric TileMap renderer.

## Directory Organization

```
terrain/
├── openrct2_placeholder/   ← OpenRCT2 extracted sprites (TEMPORARY)
│   ├── grass/              ← 19 slope variations (slope_00.png - slope_18.png)
│   ├── sand/
│   ├── stone/
│   ├── dirt/
│   ├── forest/
│   ├── water/
│   ├── shallow_water/
│   ├── deep_water/
│   ├── desert/
│   ├── snow/
│   ├── mountain/
│   └── swamp/
│
└── custom/                 ← Custom drawn replacements (PERMANENT)
    ├── grass/              ← Your drawn sprites (same 19 slope format)
    ├── sand/
    └── ...
```

## Sprite Naming Convention

Each terrain type directory should contain 19 slope variation files:

```
slope_00.png    # Flat
slope_01.png    # N corner up
slope_02.png    # E corner up
slope_03.png    # NE side up
slope_04.png    # S corner up
slope_05.png    # NS valley
slope_06.png    # SE side up
slope_07.png    # NES corners up
slope_08.png    # W corner up
slope_09.png    # NW side up
slope_10.png    # EW valley
slope_11.png    # NEW corners up
slope_12.png    # SW side up
slope_13.png    # NWS corners up
slope_14.png    # ESW corners up
slope_15.png    # All corners up (plateau)
slope_16.png    # Diagonal NE-SW
slope_17.png    # Diagonal NW-SE
slope_18.png    # Center peak
```

## Sprite Specifications

**Original OpenRCT2 format:**
- Base size: 32×16 pixels
- Isometric diamond shape

**Godot scaled format:**
- Size: 128×64 pixels (4× scale)
- Format: PNG with transparency
- Color depth: 32-bit RGBA

## Atlas Configuration

Each terrain type will be loaded as a separate TileSet atlas:
- Layout: 10×2 grid (10 columns, 2 rows)
- Row 0: Slopes 0-9
- Row 1: Slopes 10-18
- Total atlas size: 1280×128 pixels

To create atlas from individual sprites:
```bash
# Use ImageMagick to combine sprites into atlas
montage slope_{00..09}.png -tile 10x1 -geometry 128x64+0+0 row_0.png
montage slope_{10..18}.png -tile 9x1 -geometry 128x64+0+0 row_1.png
# Then combine rows vertically
```

## Workflow

### Phase 1: Placeholder Setup
1. Extract OpenRCT2 sprites → `openrct2_placeholder/grass/`
2. Scale to 128×64 → `slope_00.png` through `slope_18.png`
3. Create atlas → `grass_atlas.png` (1280×128)
4. Import into Godot TileSet
5. Test rendering with slopes

### Phase 2: Custom Art Replacement
1. Draw custom sprite → `custom/grass/slope_00.png`
2. Replace placeholder sprite in TileSet
3. Repeat for all 19 slopes
4. Move to next terrain type

### Phase 3: Cleanup (Before Release)
1. All placeholders replaced with custom art
2. Delete `openrct2_placeholder/` directory
3. Keep only `custom/` directory
4. Update attribution in credits

## Current Status

- [x] Directory structure created
- [ ] OpenRCT2 sprites extracted (waiting for original game files)
- [ ] First atlas created (grass terrain)
- [ ] TileSet configured with atlases
- [ ] SlopeCalculator implemented
- [ ] Custom grass sprites drawn

## References

- **Extraction Guide:** `/OPENRCT2_SPRITE_EXTRACTION_GUIDE.md`
- **Implementation Guide:** `/GODOT_SLOPE_RENDERING_IMPLEMENTATION.md`
- **Height System:** `/HEIGHT_MAP_ANALYSIS.md`

## Legal Notice

Sprites in `openrct2_placeholder/` are extracted from OpenRCT2 and are:
- Used under fair use for development/testing purposes only
- TEMPORARY placeholders only
- MUST BE REPLACED before public release
- NOT to be distributed with final game

OpenRCT2 is licensed under GPLv3. Original RollerCoaster Tycoon 2 graphics are
copyright Atari/Chris Sawyer. Our custom sprites in `custom/` are original work
and licensed under this project's license (MIT/Apache 2.0).
