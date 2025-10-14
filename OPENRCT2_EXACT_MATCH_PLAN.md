# OpenRCT2 Exact Coordinate System Match

**Date:** 2025-10-14
**Approach:** Match OpenRCT2's system EXACTLY - no scaling, no modifications
**Status:** Planning

---

## 🎯 Core Philosophy

**WRONG APPROACH:** Scale OpenRCT2's formulas to fit our system
**RIGHT APPROACH:** Change our system to match OpenRCT2 exactly

**Why?**
- Zero translation errors
- Their formulas work as-is
- Easy to reference their code
- Future compatibility
- Community resources apply directly

---

## 📐 OpenRCT2's Exact Coordinate System

### From `src/openrct2/world/Location.hpp`

```cpp
// Coordinate constants
constexpr int32_t kCoordsXYStep = 32;        // 32 pixels per tile X/Y
constexpr int32_t kCoordsZStep = 8;          // 8 pixels per Z level
constexpr int32_t kCoordsZPerTinyZ = 16;     // 16 tiny Z per Z coordinate

// Isometric projection
// screen_x = (tile_x - tile_y) * kCoordsXYStep
// screen_y = (tile_x + tile_y) * (kCoordsXYStep / 2) - (height * kCoordsZStep) / kCoordsZPerTinyZ
//
// Simplifies to:
// screen_x = (tile_x - tile_y) * 32
// screen_y = (tile_x + tile_y) * 16 - height / 2
```

**Tile dimensions:**
- Diamond width: 64 pixels (from top-left to top-right)
- Diamond height: 32 pixels (from top to bottom)
- Grid step: 32 pixels

**Height rendering:**
- Heights: 0-254 (uint8_t)
- Screen Y offset: `height / 2` pixels upward
- Max offset: 254 / 2 = 127 pixels

---

## 🔧 Required Changes to Our System

### Change 1: Tile Size Configuration

**Current (WRONG):**
```gdscript
// Config.gd
var TILE_SIZE: int = 32  # Tile width
var TILE_HEIGHT: int = 16  # Tile height

// TerrainTileMap.gd
const TILE_WIDTH = 32
const TILE_HEIGHT = 16
```

**New (EXACT MATCH):**
```gdscript
// Config.gd
var TILE_SIZE: int = 64  # Tile width - matches OpenRCT2
var TILE_HEIGHT: int = 32  # Tile height - matches OpenRCT2

// TerrainTileMap.gd
const TILE_WIDTH = 64
const TILE_HEIGHT = 32
```

### Change 2: TileSet Configuration

**Current:**
```gdscript
coord_helper.tile_set.tile_size = Vector2i(32, 16)
```

**New:**
```gdscript
coord_helper.tile_set.tile_size = Vector2i(64, 32)
```

### Change 3: Height Formula

**Current (WRONG):**
```gdscript
var height_offset = height / 16.0
```

**New (EXACT MATCH):**
```gdscript
// Use OpenRCT2's exact constants
const COORDS_Z_STEP = 8
const COORDS_Z_PER_TINY_Z = 16

var height_offset = (height * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z
// Simplifies to: height / 2
```

### Change 4: Camera Zoom

Since tiles will be 2× larger, adjust default zoom:

**Current:**
```gdscript
camera.zoom = Vector2(1.0, 1.0)
```

**New:**
```gdscript
camera.zoom = Vector2(0.5, 0.5)  # Same visual size as before
```

### Change 5: Grid Overlay

**Current:**
```gdscript
var half_width = tile_size.x / 2.0  # 16
var half_height = tile_size.y / 2.0  # 8
```

**New:**
```gdscript
var half_width = tile_size.x / 2.0  # 32
var half_height = tile_size.y / 2.0  # 16
```

---

## 📋 Complete Implementation Checklist

### Step 1: Update Config Constants ✅

**File:** `godot-viewer/scripts/Config.gd`

```gdscript
# OpenRCT2 coordinate constants - EXACT MATCH
var TILE_SIZE: int = 64  # kCoordsXYStep * 2 = 64 (diamond width)
var TILE_HEIGHT: int = 32  # kCoordsXYStep = 32 (diamond height)
var COORDS_XY_STEP: int = 32  # OpenRCT2 constant
var COORDS_Z_STEP: int = 8  # OpenRCT2 constant
var COORDS_Z_PER_TINY_Z: int = 16  # OpenRCT2 constant

# Height limits from OpenRCT2
const MIN_LAND_HEIGHT: int = 2
const MAX_TILE_HEIGHT: int = 254
const WATER_BASE_HEIGHT: int = 14
```

### Step 2: Update TerrainTileMap ✅

**File:** `godot-viewer/scripts/TerrainTileMap.gd`

```gdscript
# OpenRCT2 tile dimensions - EXACT MATCH
const TILE_WIDTH = 64  # Diamond width
const TILE_HEIGHT = 32  # Diamond height

# OpenRCT2 height constants - EXACT MATCH
const COORDS_Z_STEP = 8
const COORDS_Z_PER_TINY_Z = 16

func _ready():
    # ...
    coord_helper.tile_set.tile_size = Vector2i(TILE_WIDTH, TILE_HEIGHT)
    # ...

func paint_terrain_tile(world_pos: Vector2i, terrain_type: String, slope_index: int = 0, height: int = 0):
    # ... texture loading code ...

    # Calculate isometric position (base, no height)
    var base_pos = map_to_local(world_pos)

    # Apply OpenRCT2 height formula - EXACT MATCH
    # Formula: screen_y -= (height * kCoordsZStep) / kCoordsZPerTinyZ
    var height_offset = (height * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z
    # Simplifies to: height / 2

    var final_pos = Vector2(base_pos.x, base_pos.y - height_offset)

    sprite.position = final_pos
    sprite.z_index = int(final_pos.y)

    if tile_sprites.size() <= 3:
        print("🏔️ OpenRCT2 height: %d → offset: %.1f pixels (exact formula)" % [height, height_offset])
```

### Step 3: Update WorldRenderer ✅

**File:** `godot-viewer/scripts/WorldRenderer.gd`

```gdscript
func _ready():
    # ...

    # Camera zoom adjusted for 64×32 tiles
    camera.zoom = Vector2(0.5, 0.5)  # Zoom out to see larger tiles
    print("📹 Camera zoom: 0.5x (adjusted for OpenRCT2 64×32 tiles)")

    # ...

func reset_camera_to_origin():
    camera.zoom = Vector2(0.5, 0.5)  # Match new tile size
```

### Step 4: Update GridOverlay ✅

**File:** `godot-viewer/scripts/GridOverlay.gd`

```gdscript
func _draw_tile_border(tile_pos: Vector2i):
    var center = tilemap.map_to_local(tile_pos)
    var tile_size = tilemap.tile_set.tile_size

    # OpenRCT2 64×32 isometric tiles
    var half_width = tile_size.x / 2.0  # 64 / 2 = 32
    var half_height = tile_size.y / 2.0  # 32 / 2 = 16

    # Diamond corners
    var visual_center = center + Vector2(0, half_height)
    var top = visual_center + Vector2(0, -half_height)
    var right = visual_center + Vector2(half_width, 0)
    var bottom = visual_center + Vector2(0, half_height)
    var left = visual_center + Vector2(-half_width, 0)

    # Draw diamond
    draw_line(top, right, grid_color, grid_thickness)
    draw_line(right, bottom, grid_color, grid_thickness)
    draw_line(bottom, left, grid_color, grid_thickness)
    draw_line(left, top, grid_color, grid_thickness)
```

### Step 5: Update TooltipOverlay ✅

**File:** `godot-viewer/scripts/TooltipOverlay.gd`

```gdscript
func _draw_debug_tile(tile_pos: Vector2i):
    var center = tilemap.map_to_local(tile_pos)
    var tile_size = tilemap.tile_set.tile_size

    # OpenRCT2 64×32 tiles
    var half_width = tile_size.x / 2.0  # 32
    var half_height = tile_size.y / 2.0  # 16

    # ... rest of diamond drawing code ...
```

### Step 6: Update CLAUDE.md ✅

**File:** `godot-viewer/CLAUDE.md`

```markdown
## OpenRCT2 Coordinate System (EXACT MATCH)

Our system now matches OpenRCT2 exactly:

- Tile size: 64×32 pixels (OpenRCT2 diamond)
- Grid step: 32 pixels (kCoordsXYStep)
- Height scaling: 8 pixels per Z level (kCoordsZStep)

### Isometric Projection

```
screen_x = (tile_x - tile_y) × 32
screen_y = (tile_x + tile_y) × 16 - height / 2
```

### Height Rendering

```
height_offset = (height × 8) / 16
              = height / 2
```

Examples:
- Height 0 → Offset 0 pixels
- Height 56 → Offset 28 pixels
- Height 120 → Offset 60 pixels
- Height 254 → Offset 127 pixels
```

---

## 🧪 Testing Verification

### Visual Tests

After changes, verify:

1. **Tile Size**
   - Tiles appear larger than before (2× size)
   - Camera zoom 0.5× compensates
   - Same visual coverage as before

2. **Height Rendering**
   - Beach (height 56): 28 pixels offset
   - Hills (height 120): 60 pixels offset
   - Mountains (height 200): 100 pixels offset
   - Clear elevation differences

3. **Grid Overlay**
   - Diamond borders correct size
   - Aligns perfectly with tiles
   - No visual gaps or overlaps

4. **Interactive Elements**
   - Tooltip highlights correct tile
   - Mouse picking works correctly
   - Resources positioned properly
   - Entities positioned correctly

### Mathematical Verification

```rust
// Test in Rust or GDScript
fn test_openrct2_formulas() {
    // Coordinate projection
    assert_eq!(screen_x(5, 5), (5 - 5) * 32);  // = 0
    assert_eq!(screen_y(5, 5, 0), (5 + 5) * 16);  // = 160, no height

    // Height offset
    assert_eq!(height_offset(0), 0);
    assert_eq!(height_offset(56), 28);
    assert_eq!(height_offset(120), 60);
    assert_eq!(height_offset(254), 127);
}

fn screen_x(tile_x: i32, tile_y: i32) -> i32 {
    (tile_x - tile_y) * 32
}

fn screen_y(tile_x: i32, tile_y: i32, height: u8) -> i32 {
    (tile_x + tile_y) * 16 - (height as i32 / 2)
}

fn height_offset(height: u8) -> i32 {
    (height as i32 * 8) / 16
}
```

---

## 📊 Before vs After Comparison

| Aspect | Before (Scaled) | After (Exact Match) |
|--------|----------------|---------------------|
| Tile width | 32px | 64px |
| Tile height | 16px | 32px |
| Grid step | 16px | 32px |
| Height formula | `height / 16` ❌ | `height / 2` ✅ |
| Camera zoom | 1.0× | 0.5× |
| Max height offset | 15.9px | 127px |
| OpenRCT2 compatibility | Scaled | Exact |

**Key Benefit:** OpenRCT2 documentation and formulas apply directly!

---

## 🚀 Implementation Order

### 1. Update Constants (5 min)
- Config.gd: TILE_SIZE, TILE_HEIGHT
- TerrainTileMap.gd: TILE_WIDTH, TILE_HEIGHT

### 2. Update Height Formula (5 min)
- TerrainTileMap.gd: Change to `height / 2`
- Add OpenRCT2 constants

### 3. Update TileSet (2 min)
- coord_helper.tile_set.tile_size = Vector2i(64, 32)

### 4. Update Camera (2 min)
- WorldRenderer.gd: camera.zoom = Vector2(0.5, 0.5)

### 5. Update Overlays (5 min)
- GridOverlay.gd: Update half_width/half_height
- TooltipOverlay.gd: Update half_width/half_height

### 6. Test & Verify (30 min)
- Generate world
- Check visual appearance
- Verify height rendering
- Test all interactions

### 7. Update Documentation (15 min)
- CLAUDE.md: Document exact match
- Update formulas
- Add OpenRCT2 references

**Total Time:** ~1 hour

---

## 📝 OpenRCT2 Constants Reference

From `src/openrct2/world/Location.hpp`:

```cpp
// Coordinate system
constexpr int32_t kCoordsXYStep = 32;
constexpr int32_t kCoordsZStep = 8;
constexpr int32_t kCoordsZPerTinyZ = 16;

// Height limits
constexpr int32_t kMinimumLandHeight = 2;
constexpr int32_t kMaximumTileHeight = 254;
constexpr int32_t kWaterBaseHeight = 14;

// Tile size (isometric diamond)
// Width: 64 pixels (2 * kCoordsXYStep)
// Height: 32 pixels (kCoordsXYStep)
```

From `src/openrct2/Limits.h`:

```cpp
constexpr uint8_t kMinimumMapSizeInTiles = 15;
constexpr uint16_t kMaximumMapSizeInTiles = 1024;
constexpr uint16_t kMaximumTileHeight = 254;
```

---

## ✅ Success Criteria

1. **Exact Match Achieved:**
   - [ ] Tile size = 64×32 pixels
   - [ ] Height formula = `height / 2`
   - [ ] Grid step = 32 pixels
   - [ ] All constants match OpenRCT2

2. **Visual Quality:**
   - [ ] Mountains clearly elevated
   - [ ] Height differences obvious
   - [ ] No visual glitches
   - [ ] Smooth rendering

3. **Code Quality:**
   - [ ] No magic numbers
   - [ ] OpenRCT2 constants documented
   - [ ] Comments reference source files
   - [ ] Easy to understand

4. **Compatibility:**
   - [ ] Existing worlds still load
   - [ ] All features work correctly
   - [ ] Performance maintained
   - [ ] No regressions

---

## 🎯 Final Implementation Code

### Complete TerrainTileMap.gd Update

```gdscript
# TerrainTileMap.gd - OpenRCT2 EXACT MATCH
extends Node2D

const SlopeCalculator = preload("res://scripts/SlopeCalculator.gd")

# OpenRCT2 coordinate constants - EXACT MATCH
# From: src/openrct2/world/Location.hpp
const COORDS_XY_STEP = 32          # kCoordsXYStep
const COORDS_Z_STEP = 8            # kCoordsZStep
const COORDS_Z_PER_TINY_Z = 16     # kCoordsZPerTinyZ

# Tile dimensions (isometric diamond)
const TILE_WIDTH = 64   # 2 * COORDS_XY_STEP
const TILE_HEIGHT = 32  # COORDS_XY_STEP

# Height limits from OpenRCT2
const MIN_LAND_HEIGHT = 2
const MAX_TILE_HEIGHT = 254
const WATER_BASE_HEIGHT = 14

# ... rest of existing code ...

func paint_terrain_tile(world_pos: Vector2i, terrain_type: String, slope_index: int = 0, height: int = 0):
    # Validate height
    if height < 0 or height > MAX_TILE_HEIGHT:
        push_warning("Invalid height %d at %s, clamping" % [height, world_pos])
        height = clamp(height, 0, MAX_TILE_HEIGHT)

    # Get texture
    var texture: Texture2D = null
    if _should_use_rct2_texture(terrain_type) and rct2_terrain_manager and rct2_terrain_manager.has_textures():
        if _is_water_terrain(terrain_type) and water_texture_manager:
            texture = water_texture_manager
        else:
            texture = rct2_terrain_manager.get_terrain_texture(terrain_type, slope_index)

    if not texture:
        push_warning("No texture for terrain type: %s" % terrain_type)
        return

    # Create or get sprite
    var tile_key = "%d,%d" % [world_pos.x, world_pos.y]
    var sprite: Sprite2D = null

    if tile_sprites.has(tile_key):
        sprite = tile_sprites[tile_key]
    else:
        sprite = Sprite2D.new()
        sprite.name = "Tile_%d_%d" % [world_pos.x, world_pos.y]
        sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
        tile_container.add_child(sprite)
        tile_sprites[tile_key] = sprite

    sprite.texture = texture

    # Calculate isometric position (base, no height)
    var base_pos = map_to_local(world_pos)

    # Apply OpenRCT2 height formula - EXACT MATCH
    # From: src/openrct2/paint/tile_element/Paint.Surface.cpp
    # Formula: screen_y -= (height * kCoordsZStep) / kCoordsZPerTinyZ
    var height_offset = (height * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z
    # Simplifies to: height / 2

    var final_pos = Vector2(base_pos.x, base_pos.y - height_offset)

    sprite.position = final_pos
    sprite.z_index = int(final_pos.y)

    # Debug output for first few tiles
    if tile_sprites.size() <= 3:
        print("🏔️ OpenRCT2 EXACT: height=%d → offset=%.1f px (formula: h*%d/%d)" %
              [height, height_offset, COORDS_Z_STEP, COORDS_Z_PER_TINY_Z])
```

---

## 🎓 Key Takeaway

**Don't scale - MATCH EXACTLY!**

By using OpenRCT2's constants directly:
- ✅ Their formulas work as-is
- ✅ Their documentation applies
- ✅ Zero translation errors
- ✅ Easy to verify correctness
- ✅ Future-proof compatibility

This is the **correct engineering approach** - adopt proven standards rather than reinvent them.

---

**Ready to implement?** This is a straightforward system-wide change that brings us into perfect alignment with OpenRCT2's coordinate system.
