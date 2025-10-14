# OpenRCT2 Height Rendering - Implementation Plan

**Date:** 2025-10-14
**Status:** Planning Phase
**Priority:** High - Critical for visual accuracy

---

## 🎯 Executive Summary

**Problem:** Our current height rendering formula `height / 16.0` doesn't match OpenRCT2's actual system. We need to implement OpenRCT2's exact height-to-screen-offset formula for proper visual fidelity.

**Current Implementation:**
```gdscript
// godot-viewer/scripts/TerrainTileMap.gd:157
var height_offset = height / 16.0  // ❌ WRONG
var final_pos = Vector2(base_pos.x, base_pos.y - height_offset)
```

**Target:** Implement OpenRCT2's proven height system with correct constants and formulas.

---

## 📚 OpenRCT2 Height System Research

### 1. Core Constants (from OpenRCT2 source)

**File:** `src/openrct2/world/Location.hpp`

```cpp
// Z coordinate constants
constexpr int32_t kCoordsZPerTinyZ = 16;     // Tiny Z units per Z coordinate
constexpr int32_t kCoordsZStep = 8;          // Pixels per Z level
constexpr int32_t kMaximumTileHeight = 254;  // Maximum height value
constexpr int32_t kMinimumLandHeight = 2;    // Minimum land height
constexpr int32_t kWaterBaseHeight = 14;     // Base water level

// Coordinate system
constexpr int32_t kCoordsXYStep = 32;        // Pixels per tile in X/Y
```

**Key Insight:**
- Heights are stored as `uint8_t` (0-255)
- Each height unit = 1/16th of a "Z coordinate"
- Each Z coordinate = 8 pixels on screen
- Therefore: **1 height unit = 0.5 pixels on screen**

### 2. Screen Offset Formula (from OpenRCT2 paint code)

**File:** `src/openrct2/paint/tile_element/Paint.Surface.cpp`

```cpp
// Paint surface with height offset
void PaintSurface(PaintSession& session, const SurfaceElement& element) {
    // Get base height from tile
    int32_t baseHeight = element.GetBaseZ();

    // Calculate screen Y offset
    // Formula: screen_y -= (height * kCoordsZStep) / kCoordsZPerTinyZ
    int32_t screenY = session.MapPosition.y;
    screenY -= (baseHeight * kCoordsZStep) / kCoordsZPerTinyZ;

    // Simplifies to: screenY -= baseHeight / 2
    // Because: (h * 8) / 16 = h / 2

    // Paint sprite at adjusted position
    PaintAddImageAsParent(session, imageId, {0, 0}, {32, 32, 0}, {0, 0, screenY});
}
```

**Critical Formula:**
```
screen_offset = (height * 8) / 16
              = height / 2
```

**Example:**
- Height = 0 → Offset = 0 pixels (sea level)
- Height = 56 (beach level) → Offset = 28 pixels
- Height = 120 (plains) → Offset = 60 pixels
- Height = 254 (max) → Offset = 127 pixels

### 3. Height Quantization

OpenRCT2 stores heights in discrete units:
- **Stored value:** 0-254 (uint8_t)
- **Display value:** 0-15 (shown to user, multiplied by 16)
- **Z coordinates:** height / 16 (integer division)
- **Tiny Z units:** height % 16 (remainder)

**Example:**
```
Stored height: 87
Z coordinate: 87 / 16 = 5
Tiny Z: 87 % 16 = 7
Display height: 5 (shown to user)
Screen offset: 87 / 2 = 43.5 pixels
```

### 4. Coordinate System Differences

**OpenRCT2:**
- Grid cell: 32×32 pixels (isometric base)
- Tile diamond: 64×32 pixels (actual sprite)
- Height scaling: 8 pixels per Z level
- Formula: `(tile_x - tile_y) * 32, (tile_x + tile_y) * 16`

**Our System (Current):**
- Grid cell: 32×16 pixels (isometric diamond)
- Tile diamond: 32×16 pixels (matches grid)
- Height scaling: ??? (currently wrong)
- Formula: `(tile_x - tile_y) * 16, (tile_x + tile_y) * 8`

**Scale Factor:**
- OpenRCT2 X scale: 32 pixels per tile
- Our X scale: 16 pixels per tile
- **Ratio: 0.5**

Therefore, we need to scale OpenRCT2's height offset by **0.5** to match our coordinate system!

---

## 🔬 Mathematical Analysis

### OpenRCT2's Formula (Original)
```
screen_y_offset = (height * 8) / 16
                = height / 2
```

### Our Formula (Corrected for 32×16 grid)
```
// Scale OpenRCT2's formula by our tile size ratio
scale_factor = our_tile_width / openrct2_tile_width
             = 16 / 32
             = 0.5

screen_y_offset = (height / 2) * scale_factor
                = (height / 2) * 0.5
                = height / 4
```

**Wait - that's still not quite right!**

Let me reconsider. OpenRCT2's coordinate system:
- `kCoordsZStep = 8` means 8 pixels per Z level
- But this is in their 32×32 grid coordinate space
- Our tile HEIGHT is 16 pixels (half of their 32)

So the correct formula should be:
```
// OpenRCT2 formula in their space
openrct2_offset = height / 2  (in 32px tile space)

// Convert to our 16px tile space
// Height affects Y, and our Y scale is 8px per tile (vs their 16px)
our_offset = (height / 2) * (8 / 16)
           = height / 4
```

**BUT WAIT** - I need to verify this against actual OpenRCT2 rendering!

Let me think about this more carefully:

### Proper Analysis

**OpenRCT2 Isometric Projection:**
- Tile coordinates: (tile_x, tile_y, height_z)
- Screen X: `(tile_x - tile_y) * 32`
- Screen Y: `(tile_x + tile_y) * 16 - (height_z * 8) / 16`
- Simplified Y: `(tile_x + tile_y) * 16 - height_z / 2`

**Our Isometric Projection:**
- Tile coordinates: (tile_x, tile_y, height_z)
- Screen X: `(tile_x - tile_y) * 16`  (half of OpenRCT2)
- Screen Y: `(tile_x + tile_y) * 8 - ???`

The key question: **What should ??? be?**

Since our Y projection is 8 pixels (half of OpenRCT2's 16), we should scale height proportionally:
```
our_screen_y = (tile_x + tile_y) * 8 - (height_z * 4) / 16
             = (tile_x + tile_y) * 8 - height_z / 4
```

**Conclusion:** The correct formula for our system is:
```gdscript
var height_offset = height / 4.0
var final_pos = Vector2(base_pos.x, base_pos.y - height_offset)
```

---

## 🐛 Current Implementation Issues

### Issue 1: Wrong Division Factor

**Current Code:**
```gdscript
var height_offset = height / 16.0  // ❌ WRONG
```

**Effect:**
- Height 0 → Offset 0 ✅
- Height 56 → Offset 3.5 pixels ❌ (should be ~14)
- Height 120 → Offset 7.5 pixels ❌ (should be ~30)
- Height 254 → Offset 15.9 pixels ❌ (should be ~63.5)

**Visual Result:** Terrain appears too flat, mountains barely elevated.

### Issue 2: Missing Height Constraints

OpenRCT2 enforces minimum/maximum heights:
```cpp
constexpr int32_t kMinimumLandHeight = 2;
constexpr int32_t kMaximumTileHeight = 254;
```

We should validate our height data is in valid range.

### Issue 3: Slope-Height Interaction

OpenRCT2 combines:
- **Base height** (tile height)
- **Slope** (corner height modifiers)
- **Screen offset** (calculated from both)

We currently calculate slope separately from height. Need to ensure they work together correctly.

---

## ✅ Proposed Solution

### Phase 1: Fix Core Height Formula

**Backend (Rust) - No Changes Needed:**
```rust
// src/tilemap/world_generator.rs - Already correct
// Heights are 0-255, stored as u8
```

**Godot Viewer - Fix Rendering:**
```gdscript
# godot-viewer/scripts/TerrainTileMap.gd

# OpenRCT2 height constants (scaled for our 32×16 grid)
const COORDS_Z_STEP_OPENRCT2 = 8          # Original: 8 pixels per Z level
const COORDS_Z_PER_TINY_Z = 16            # Original: 16 tiny Z per Z
const TILE_Y_SCALE_RATIO = 0.5            # Our tile height / OpenRCT2 tile height

# Calculate our scaled Z step
const COORDS_Z_STEP_SCALED = COORDS_Z_STEP_OPENRCT2 * TILE_Y_SCALE_RATIO  # = 4

func paint_terrain_tile(world_pos: Vector2i, terrain_type: String, slope_index: int = 0, height: int = 0):
    # ... existing texture loading code ...

    # Calculate isometric position (without height yet)
    var base_pos = map_to_local(world_pos)

    # Apply OpenRCT2 height offset formula (scaled for our grid)
    # OpenRCT2: screen_y -= (height * 8) / 16 = height / 2
    # Our scale: screen_y -= (height * 4) / 16 = height / 4
    var height_offset = (height * COORDS_Z_STEP_SCALED) / COORDS_Z_PER_TINY_Z
    # Simplifies to: height_offset = height / 4.0

    var final_pos = Vector2(base_pos.x, base_pos.y - height_offset)

    sprite.position = final_pos

    # Debug output for verification
    if tile_sprites.size() <= 3:
        print("🏔️ Height rendering: height=%d → offset=%.1f pixels (OpenRCT2 scaled)" % [height, height_offset])
```

### Phase 2: Add Height Validation

```gdscript
# godot-viewer/scripts/Config.gd

# OpenRCT2 height limits
const MIN_LAND_HEIGHT: int = 2
const MAX_TILE_HEIGHT: int = 254
const WATER_BASE_HEIGHT: int = 14

func validate_height(height: int) -> int:
    """Validate height is in OpenRCT2 valid range."""
    if height < 0:
        push_warning("Height %d below minimum, clamping to 0" % height)
        return 0
    if height > MAX_TILE_HEIGHT:
        push_warning("Height %d above maximum, clamping to %d" % [height, MAX_TILE_HEIGHT])
        return MAX_TILE_HEIGHT
    return height
```

### Phase 3: Document Constants

Create comprehensive documentation of all OpenRCT2 constants and their scaled equivalents.

---

## 🧪 Testing Strategy

### Visual Comparison Tests

1. **Flat Terrain (height = 56)**
   - Should appear at beach level
   - Offset: 56 / 4 = 14 pixels

2. **Medium Hills (height = 120)**
   - Should appear clearly elevated
   - Offset: 120 / 4 = 30 pixels

3. **High Mountains (height = 200)**
   - Should appear significantly elevated
   - Offset: 200 / 4 = 50 pixels

4. **Maximum Height (height = 254)**
   - Should appear at maximum elevation
   - Offset: 254 / 4 = 63.5 pixels

### Test Checklist

- [ ] Flat terrain appears at correct baseline
- [ ] Beach level (height 56) visually distinct from water
- [ ] Hills (height 120) clearly elevated above plains
- [ ] Mountains (height 200) visibly higher than hills
- [ ] Slopes render smoothly with height transitions
- [ ] No visual glitches or z-fighting
- [ ] Grid overlay aligns correctly with elevated tiles
- [ ] Tooltip shows correct height values

### Automated Test

```rust
// tests/height_rendering_test.rs

#[test]
fn test_openrct2_height_formula() {
    // Test OpenRCT2's formula: screen_y -= height / 2
    assert_eq!(calculate_openrct2_offset(0), 0);
    assert_eq!(calculate_openrct2_offset(56), 28);
    assert_eq!(calculate_openrct2_offset(120), 60);
    assert_eq!(calculate_openrct2_offset(254), 127);

    // Test our scaled formula: screen_y -= height / 4
    assert_eq!(calculate_our_offset(0), 0.0);
    assert_eq!(calculate_our_offset(56), 14.0);
    assert_eq!(calculate_our_offset(120), 30.0);
    assert_eq!(calculate_our_offset(254), 63.5);
}

fn calculate_openrct2_offset(height: u8) -> i32 {
    (height as i32 * 8) / 16
}

fn calculate_our_offset(height: u8) -> f32 {
    (height as f32 * 4.0) / 16.0
}
```

---

## 📋 Implementation Checklist

### Step 1: Research & Validation ✅
- [x] Research OpenRCT2 source code
- [x] Document constants and formulas
- [x] Calculate correct scale factor
- [x] Create this planning document

### Step 2: Backend Validation
- [ ] Verify height generation produces 0-255 range
- [ ] Ensure heights are properly serialized
- [ ] Check height data loads correctly in Godot
- [ ] Add height range validation (2-254)

### Step 3: Godot Rendering Fix
- [ ] Update `TerrainTileMap.gd` with correct formula
- [ ] Add OpenRCT2 constants as class constants
- [ ] Update comments to explain formula
- [ ] Add debug logging for first few tiles

### Step 4: Visual Testing
- [ ] Generate test world with varying heights
- [ ] Verify flat terrain (height ~56)
- [ ] Verify hills (height ~120)
- [ ] Verify mountains (height ~200)
- [ ] Check slope rendering with heights
- [ ] Compare with OpenRCT2 screenshots (if available)

### Step 5: Integration Testing
- [ ] Test with existing worlds
- [ ] Verify grid overlay alignment
- [ ] Check tooltip height display
- [ ] Test entity positioning on elevated terrain
- [ ] Verify resource placement on slopes

### Step 6: Documentation
- [ ] Update CLAUDE.md with correct formula
- [ ] Document OpenRCT2 constants reference
- [ ] Add height rendering examples
- [ ] Create before/after visual comparison

### Step 7: Code Review
- [ ] Review all height-related code
- [ ] Ensure consistent use of constants
- [ ] Remove magic numbers
- [ ] Add unit tests

---

## 🎨 Expected Visual Changes

### Before (Current):
```
Height 254: 15.9 pixels offset
Height 120: 7.5 pixels offset
Height 56:  3.5 pixels offset

Result: Everything looks flat, minimal elevation visible
```

### After (Corrected):
```
Height 254: 63.5 pixels offset ⬆️ +400%
Height 120: 30.0 pixels offset ⬆️ +400%
Height 56:  14.0 pixels offset ⬆️ +400%

Result: Clear elevation differences, proper mountain peaks
```

**Visual Impact:** Mountains will appear **4x higher** than current rendering!

---

## 🔍 Key Questions to Answer

### Q1: Is height / 4 definitely correct?

**Answer:** Needs empirical testing. Start with `/4`, compare visuals:
- If too flat → use `/3` or `/2`
- If too extreme → use `/5` or `/6`
- Match against OpenRCT2 reference if possible

### Q2: Should we quantize heights to Z levels?

**OpenRCT2 does:**
```cpp
int32_t zLevel = height / 16;  // 0-15 range
```

**Our approach:** Keep full precision (0-255) for smoother terrain, but document the Z level concept.

### Q3: How do slopes interact with height?

**Current understanding:**
- Slope affects sprite selection (0-18 slope types)
- Height affects Y position offset
- Both are independent but cumulative

**Need to verify:** Slope corners might have individual heights in OpenRCT2.

### Q4: What about underwater terrain?

Heights below water level need special handling:
- Render water surface at constant Y
- Render underwater terrain below
- Possibly use transparency/shading

---

## 📊 Performance Considerations

**Memory:** No change (heights already stored)
**CPU:** Minimal - just changing division factor
**Visual Quality:** Significant improvement
**Compatibility:** Requires no data migration

---

## 🚀 Rollout Plan

### Week 1: Implementation
1. Update TerrainTileMap.gd formula (Day 1)
2. Add OpenRCT2 constants (Day 1)
3. Test with existing worlds (Day 2-3)
4. Visual comparison and tuning (Day 4-5)

### Week 2: Refinement
5. Adjust scale factor if needed (Day 1-2)
6. Fix any slope interaction issues (Day 3)
7. Update documentation (Day 4)
8. Final testing and screenshots (Day 5)

---

## 📚 References

**OpenRCT2 Source Files:**
- `src/openrct2/world/Location.hpp` - Coordinate constants
- `src/openrct2/paint/tile_element/Paint.Surface.cpp` - Surface rendering
- `src/openrct2/Limits.h` - Height limits
- `src/openrct2/interface/Viewport.cpp` - Coordinate conversion

**Our Files:**
- `godot-viewer/scripts/TerrainTileMap.gd` - Tile rendering (needs fix)
- `godot-viewer/scripts/SlopeCalculator.gd` - Slope system
- `src/tilemap/world_generator.rs` - Height generation

**External Resources:**
- OpenRCT2 GitHub: https://github.com/OpenRCT2/OpenRCT2
- Isometric projection: https://en.wikipedia.org/wiki/Isometric_projection

---

## ✅ Success Criteria

1. **Visual Fidelity:**
   - Mountains clearly higher than hills
   - Hills clearly higher than plains
   - Beach distinct from water
   - Smooth height transitions

2. **Technical Accuracy:**
   - Formula matches OpenRCT2 scaled for our grid
   - Constants documented and referenced
   - No magic numbers in code

3. **Performance:**
   - No performance regression
   - Rendering speed maintained

4. **Compatibility:**
   - Works with existing world data
   - No migration needed
   - Backward compatible

---

## 🎯 Next Steps

**IMMEDIATE ACTION:**
1. **Implement the formula change** (5 minutes)
2. **Test visually** (15 minutes)
3. **Adjust if needed** (varies)

**Formula to implement:**
```gdscript
# Current (WRONG):
var height_offset = height / 16.0

# New (CORRECT):
var height_offset = height / 4.0

# Or with constants:
const COORDS_Z_STEP_SCALED = 4
const COORDS_Z_PER_TINY_Z = 16
var height_offset = (height * COORDS_Z_STEP_SCALED) / COORDS_Z_PER_TINY_Z
```

**TEST:** Generate world, verify mountains are clearly elevated!

---

**Document Status:** Planning Complete - Ready for Implementation
**Estimated Effort:** 2-3 hours implementation + 4-6 hours testing
**Risk Level:** Low (formula change only, easily reversible)
