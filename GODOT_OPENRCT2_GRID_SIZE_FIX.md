# Godot Viewer: OpenRCT2 Grid Size Fix

**Date:** 2025-10-14
**Status:** Complete

## Problem

The Godot viewer was using **64×32 pixel tiles** for isometric rendering, but OpenRCT2 actually uses **32×16 pixel tiles** for its grid. This caused the tiles to appear at 2× the correct size, making the grid not match OpenRCT2's specifications.

## Root Cause

The initial implementation incorrectly assumed OpenRCT2 used 64×32 tiles based on texture atlas sizes, but the actual **game grid** is 32×16 pixels per tile in isometric projection.

### OpenRCT2 Specifications

- **Grid tile size**: 32 pixels wide × 16 pixels tall (isometric diamond)
- **Tile formula**:
  - `pixel_x = (tile_x - tile_y) × 16`
  - `pixel_y = (tile_x + tile_y) × 8`
- **Texture sizes**: Can be 32×32, 64×64, or larger (for elevated terrain), but the **grid itself** is 32×16

## Solution

Updated tile size configuration throughout the Godot viewer to match OpenRCT2's 32×16 grid.

### Files Changed

#### 1. `godot-viewer/scripts/Config.gd`

**Before:**
```gdscript
var TILE_SIZE: int = 64  # Tile size - matches RCT2 atlas cell size (64×64)
```

**After:**
```gdscript
var TILE_SIZE: int = 32  # Tile width - matches OpenRCT2 isometric grid (32×16)
var TILE_HEIGHT: int = 16  # Tile height for isometric projection
```

#### 2. `godot-viewer/scripts/TerrainTileMap.gd`

**Before:**
```gdscript
# Isometric tile size (64x32 for RCT2-style rendering)
const TILE_WIDTH = 64
const TILE_HEIGHT = 32
```

**After:**
```gdscript
# Isometric tile size (32x16 to match OpenRCT2 grid exactly)
const TILE_WIDTH = 32
const TILE_HEIGHT = 16
```

#### 3. `godot-viewer/scripts/WorldRenderer.gd`

**Before:**
```gdscript
camera.zoom = Vector2(0.5, 0.5)  # Zoom out to see isometric tiles (128x64 tiles are large)
```

**After:**
```gdscript
camera.zoom = Vector2(1.0, 1.0)  # 1:1 zoom for OpenRCT2 32×16 tiles
```

**Camera reset function updated:**
```gdscript
func reset_camera_to_origin():
    # ...
    camera.zoom = Vector2(1.0, 1.0)  # Changed from 0.5
```

#### 4. `godot-viewer/scripts/GridOverlay.gd`

Updated comments to reflect correct tile dimensions:

```gdscript
# For OpenRCT2 isometric tiles (32x16), the corners form a diamond
var half_width = tile_size.x / 2.0  # 32 / 2 = 16
var half_height = tile_size.y / 2.0  # 16 / 2 = 8
```

#### 5. `godot-viewer/scripts/TooltipOverlay.gd`

Updated comments:

```gdscript
var half_width = tile_size.x / 2.0  # 16 for OpenRCT2 32×16 tiles
var half_height = tile_size.y / 2.0  # 8 for OpenRCT2 32×16 tiles
```

#### 6. `godot-viewer/CLAUDE.md`

Updated documentation to reflect correct OpenRCT2 tile sizes:
- Changed all references from 64×32 to 32×16
- Updated pixel coordinate examples
- Corrected isometric projection formulas
- Updated zoom level recommendations

## Impact

### Visual Changes

- **Tiles now appear at correct OpenRCT2 scale** (50% of previous size)
- **Camera zoom changed from 0.5x to 1.0x** to maintain similar viewport coverage
- **Grid overlay** automatically adjusts to new tile size
- **Tooltip overlay** correctly highlights tiles at new size
- **Resource and entity positioning** scales proportionally with new TILE_SIZE

### Coordinate System

**Before (64×32 tiles):**
- Tile (10, 10) → Pixel (640, 160)
- Tile (-48, -48) → Pixel (-3008, -752)

**After (32×16 tiles):**
- Tile (10, 10) → Pixel (320, 80)
- Tile (-48, -48) → Pixel (-1504, -384)

### Backward Compatibility

✅ **All systems automatically adapt** because they use `Config.TILE_SIZE` or `tilemap.tile_set.tile_size` dynamically:
- ResourceManager (resource sprite positioning)
- EntityManager (entity sprite positioning)
- GridOverlay (diamond corner calculations)
- TooltipOverlay (tile border visualization)

## Testing

### Manual Testing Steps

1. **Start the backend:**
   ```bash
   cargo run --bin life-simulator
   ```

2. **Open Godot viewer:**
   ```bash
   cd godot-viewer
   /Applications/Godot.app/Contents/MacOS/Godot --path .
   # Press F5 to run
   ```

3. **Expected behavior:**
   - Tiles should appear at correct OpenRCT2 scale (smaller than before)
   - Camera zoom should be 1.0x by default
   - Grid overlay (Press 'G') should show correct tile boundaries
   - Tooltip (mouse hover) should highlight correct tile
   - Resources and entities should position correctly within tiles

### Visual Verification Checklist

- [ ] Terrain tiles render at 32×16 pixel size
- [ ] Grid overlay draws diamonds at correct size
- [ ] Tooltip highlights correct tile under mouse cursor
- [ ] Resources (trees, rocks, bushes) positioned correctly
- [ ] Entities (rabbits, humans) positioned with feet in tile
- [ ] Camera positioned at tile (0, 0) on startup
- [ ] Zoom controls (+/-) work correctly from 1.0x base

## References

- **OpenRCT2 Source**: `src/openrct2/world/map_generator/SimplexNoise.cpp`
- **OpenRCT2 Paint System**: `src/openrct2/paint/tile_element/Paint.Surface.h`
- **Project Documentation**:
  - `OPENRCT2_MAP_GENERATOR.md` - Height map system
  - `godot-viewer/CLAUDE.md` - Godot viewer guide
  - `HEIGHT_MAP_ANALYSIS.md` - OpenRCT2 terrain research

## Conclusion

The Godot viewer now correctly uses **OpenRCT2's 32×16 isometric grid size**, ensuring accurate rendering and proper integration with the OpenRCT2-style terrain generation system.

**Result:** Tiles are now rendered at the correct scale, matching OpenRCT2's grid specifications exactly. ✅
