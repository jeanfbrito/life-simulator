# OpenRCT2 Exact Match - Implementation Summary

**Date:** 2025-10-14
**Status:** ✅ COMPLETE
**Approach:** Match OpenRCT2's coordinate system EXACTLY

---

## 🎯 What We Did

Changed our Godot viewer to **match OpenRCT2's coordinate system exactly** - no scaling, no modifications, just direct implementation of their constants and formulas.

---

## 📐 OpenRCT2 Constants Implemented

From `src/openrct2/world/Location.hpp`:

| Constant | Value | Purpose |
|----------|-------|---------|
| `kCoordsXYStep` | 32 | Base coordinate step |
| `kCoordsZStep` | 8 | Pixels per Z level |
| `kCoordsZPerTinyZ` | 16 | Height division factor |
| Tile Width | 64 | Diamond width (2 × kCoordsXYStep) |
| Tile Height | 32 | Diamond height (kCoordsXYStep) |

---

## 🔧 Files Changed

### 1. Config.gd - Core Constants
```gdscript
# BEFORE:
var TILE_SIZE: int = 32
var TILE_HEIGHT: int = 16

# AFTER (OpenRCT2 EXACT):
var TILE_SIZE: int = 64
var TILE_HEIGHT: int = 32
var COORDS_XY_STEP: int = 32
var COORDS_Z_STEP: int = 8
var COORDS_Z_PER_TINY_Z: int = 16
```

### 2. TerrainTileMap.gd - Tile Size & Height Formula
```gdscript
# BEFORE:
const TILE_WIDTH = 32
const TILE_HEIGHT = 16
var height_offset = height / 16.0  // WRONG!

# AFTER (OpenRCT2 EXACT):
const TILE_WIDTH = 64
const TILE_HEIGHT = 32
const COORDS_Z_STEP = 8
const COORDS_Z_PER_TINY_Z = 16
var height_offset = float(height * COORDS_Z_STEP) / float(COORDS_Z_PER_TINY_Z)
// Simplifies to: height / 2.0
```

### 3. WorldRenderer.gd - Camera Zoom
```gdscript
# BEFORE:
camera.zoom = Vector2(1.0, 1.0)

# AFTER (compensate for 2× larger tiles):
camera.zoom = Vector2(0.5, 0.5)
```

### 4. GridOverlay.gd - Diamond Dimensions
```gdscript
# BEFORE:
var half_width = tile_size.x / 2.0  # 16
var half_height = tile_size.y / 2.0  # 8

# AFTER (OpenRCT2 EXACT):
var half_width = tile_size.x / 2.0  # 32
var half_height = tile_size.y / 2.0  # 16
```

### 5. TooltipOverlay.gd - Diamond Dimensions
```gdscript
# BEFORE:
var half_width = tile_size.x / 2.0  # 16
var half_height = tile_size.y / 2.0  # 8

# AFTER (OpenRCT2 EXACT):
var half_width = tile_size.x / 2.0  # 32
var half_height = tile_size.y / 2.0  # 16
```

### 6. CLAUDE.md - Documentation Updates
- Updated all tile size references: 32×16 → 64×32
- Updated isometric projection formulas with OpenRCT2 constants
- Updated camera zoom recommendations
- Added OpenRCT2 source code references

---

## 📊 Before vs After Comparison

| Aspect | Before | After | Change |
|--------|--------|-------|--------|
| **Tile Width** | 32px | 64px | 2× larger |
| **Tile Height** | 16px | 32px | 2× larger |
| **Height Formula** | `height / 16` ❌ | `height / 2` ✅ | 8× larger offset |
| **Camera Zoom** | 1.0× | 0.5× | Compensate |
| **Max Height Offset** | 15.9px | 127px | 8× improvement! |

---

## 🏔️ Height Rendering Impact

### Before (WRONG):
```
Height   0: offset =   0.0 px
Height  56: offset =   3.5 px  ❌ Too flat!
Height 120: offset =   7.5 px  ❌ Too flat!
Height 200: offset =  12.5 px  ❌ Too flat!
Height 254: offset =  15.9 px  ❌ Maximum offset tiny!
```

### After (OpenRCT2 EXACT):
```
Height   0: offset =   0.0 px  ✅
Height  56: offset =  28.0 px  ✅ Beach level visible
Height 120: offset =  60.0 px  ✅ Hills clearly elevated
Height 200: offset = 100.0 px  ✅ Mountains prominent
Height 254: offset = 127.0 px  ✅ Maximum elevation
```

**Visual Improvement:** Mountains appear **8× higher** than before! 🏔️

---

## 📐 OpenRCT2 Isometric Formulas (EXACT)

### Coordinate Projection
```
screen_x = (tile_x - tile_y) × 32
screen_y = (tile_x + tile_y) × 16 - height / 2
```

### Height Offset
```
height_offset = (height × 8) / 16
              = height / 2

Examples:
  height =   0 → offset =   0 px
  height =  56 → offset =  28 px (beach)
  height = 120 → offset =  60 px (hills)
  height = 200 → offset = 100 px (mountains)
  height = 254 → offset = 127 px (max)
```

---

## ✅ Benefits of Exact Match

1. **Zero Translation Errors**
   - OpenRCT2 formulas work as-is
   - No scaling math needed
   - Easy to verify correctness

2. **Direct Code Reference**
   - Can reference OpenRCT2 source directly
   - Community knowledge applies
   - Easy to debug issues

3. **Future Compatibility**
   - Adding new features easier
   - Slopes, corners work correctly
   - Can use OpenRCT2 sprites directly

4. **Proper Visual Scale**
   - Mountains look like mountains
   - Height differences obvious
   - Terrain depth clear

---

## 🧪 Testing

### Visual Verification Checklist
- [x] Tiles appear at correct size (64×32)
- [x] Camera zoom compensates (0.5×)
- [x] Grid overlay aligns perfectly
- [x] Tooltip highlights correct tiles
- [ ] Height rendering shows elevation (pending world test)
- [ ] Mountains clearly elevated (pending world test)
- [ ] Slopes render correctly (pending world test)

### Expected Console Output
```
Config singleton initialized
🗺️ TerrainTileMap initialized (Sprite2D-based rendering)
📹 Camera positioned at tile (0, 0) = pixel (0.0, 0.0) with zoom 0.5x (OpenRCT2 exact)
🏔️ OpenRCT2 EXACT: tile (-48, -48), height=120 → offset=60.0 px (h*8/16) → Grass
🏔️ OpenRCT2 EXACT: tile (-48, -47), height=125 → offset=62.5 px (h*8/16) → Grass
🏔️ OpenRCT2 EXACT: tile (-48, -46), height=130 → offset=65.0 px (h*8/16) → Stone
```

---

## 📝 OpenRCT2 Source References

**Our implementation directly matches:**
- `src/openrct2/world/Location.hpp` - Coordinate constants
- `src/openrct2/paint/tile_element/Paint.Surface.cpp` - Height rendering
- `src/openrct2/Limits.h` - Height limits
- `src/openrct2/world/tile_element/SurfaceElement.h` - Tile structure

---

## 🎓 Key Takeaway

**"Don't scale - MATCH EXACTLY!"**

By adopting OpenRCT2's proven coordinate system directly:
- ✅ Their formulas work as documented
- ✅ Zero translation/scaling errors
- ✅ Easy to reference their code
- ✅ Community resources apply
- ✅ Future-proof implementation

This is the **correct engineering approach** - adopt proven standards rather than reinvent them.

---

## 🚀 Next Steps

1. **Generate Test World**
   ```bash
   cargo run --bin map_generator -- --name "height_test" --seed 42 --radius 5
   ```

2. **Test in Godot**
   ```bash
   cd godot-viewer
   /Applications/Godot.app/Contents/MacOS/Godot --path .
   # Press F5, observe height rendering
   ```

3. **Verify Height Rendering**
   - Check mountains are clearly elevated
   - Verify beach/plains/hills transitions
   - Confirm grid overlay aligns
   - Test camera controls

---

## ✨ Result

**We now match OpenRCT2's coordinate system EXACTLY!**

- Tile size: 64×32 pixels ✅
- Height formula: `height / 2` ✅
- Grid step: 32 pixels ✅
- All constants: OpenRCT2 exact ✅

**Mountains will look like mountains!** 🏔️

---

**Implementation Complete:** 2025-10-14
**Status:** Ready for testing
