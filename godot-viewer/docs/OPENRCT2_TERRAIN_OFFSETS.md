# OpenRCT2 Terrain Offset Implementation

## Overview

This document describes how the Godot viewer mirrors OpenRCT2's exact terrain rendering formulas, particularly concerning height and slope offsets. The implementation ensures that terrain tiles align seamlessly at shared corners by using OpenRCT2's corner-based positioning logic.

## Core Constants

From OpenRCT2 `src/openrct2/world/Location.hpp`:

```cpp
constexpr int32_t kCoordsXYStep = 32;      // Horizontal tile size in world units
constexpr int32_t kCoordsZStep = 8;        // Vertical step size
constexpr int32_t kCoordsZPerTinyZ = 16;   // Tiny-Z units per full height step
```

Godot equivalent in `Config.gd` and `TerrainTileMap.gd`:

```gdscript
const COORDS_XY_STEP = 32
const COORDS_Z_STEP = 8
const COORDS_Z_PER_TINY_Z = 16
const RENDERING_SCALE = 2.0  # Display multiplier
```

## Height Units

- **Tiny-Z**: Base height unit (0-255) stored in `height_data`
- **Screen pixels**: Derived via formula `(tiny_z * kCoordsZStep) / kCoordsZPerTinyZ`
- **LAND_HEIGHT_STEP**: 16 tiny-Z units = one terrain slope step

## Corner-Based Positioning

### OpenRCT2 Reference

From `src/openrct2/world/tile_element/Slope.cpp`:

- `kSlopeRelativeCornerHeights`: Table mapping slope index (0-31) to corner heights (0-2 units per corner)
- `GetSlopeCornerHeights()`: Returns absolute tiny-Z for each corner: `base_height + relative * LAND_HEIGHT_STEP`

### Godot Implementation

**`SlopeCalculator.gd`**:

- `get_relative_corner_heights(slope_index)`: Returns `{top, right, bottom, left}` with relative heights (0-2)
- `get_corner_heights(base_height, slope_index)`: Returns absolute tiny-Z per corner

**`TerrainTileMap.gd`**:

- `compute_corner_offsets_screen(base_height, slope_index)`: Converts corner heights to screen pixel offsets
- `paint_terrain_tile()`: Anchors sprite at **north corner** (top) for seamless alignment

### Positioning Formula

For each tile:

1. **Get corner heights**: `corner_heights = SlopeCalculator.get_corner_heights(base_height, slope_index)`
2. **Convert to screen offsets**: `screen_y = (corner_tiny_z * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z * RENDERING_SCALE`
3. **Anchor at north corner**: `sprite.position.y = base_pos.y - north_corner_screen_offset`

This ensures tiles sharing a corner have identical screen Y at that corner, eliminating gaps or overlaps.

## Slope Rotation

Before rendering, all slope indices are rotated via `SlopeCalculator.rotate_slope_index()` according to `Config.slope_rotation`. This allows the same corner math to work consistently across all rotations.

## Coordinate System Convention

- **North (top)**: Primary anchor corner
- **East (right)**: +X direction
- **South (bottom)**: -Y direction (isometric)
- **West (left)**: -X direction

Corners are named relative to isometric visual layout, matching OpenRCT2's internal conventions.

## Texture Selection

Textures are pre-baked per slope index (0-18) and stored in `assets/tiles/terrain/openrct2_placeholder/`. The `RCT2TerrainTextureManager` loads all 19 variants per terrain type. After rotation, the correct texture is fetched and positioned using the corner math.

## Known Gaps & Follow-Up Tasks

### Edge Faces

OpenRCT2 draws vertical side faces when adjacent tiles' shared corners differ in height. This requires:

- **Edge detection**: Compare corner heights between neighbors
- **Side sprites**: Load vertical cliff/edge textures from OpenRCT2's sprite library
- **Render order**: Draw edges after base terrain, before entities

Planned implementation: `EdgeRenderer.gd` helper module (see plan step 5).

### Smoothing & Shading

OpenRCT2 applies subtle shading to slopes for visual depth. Current implementation uses flat textures. Future enhancement could:

- Apply slope-specific tint overlays
- Add lighting/shadow modulation based on corner heights
- Integrate with time-of-day lighting system

### Diagonal Slopes (16-18)

Current implementation treats diagonals like standard slopes. OpenRCT2 uses special rendering for these:

- **Slope 16/17**: Diagonal ridges (NE-SW)
- **Slope 18**: Peak (all corners raised)

Consider adding specialized positioning tweaks if visual drift is detected.

## Testing & Validation

### Diagnostic Scene

Create `tests/SlopeDiagnostics.tscn` with:

- Grid of all 19 slope variants at various heights
- Toggle to print per-corner tiny-Z and screen Y
- Visual overlays showing corner positions (`SlopeDebugOverlay.gd`)

### Verification Checklist

- [ ] Neighboring tiles' shared corners have identical screen Y
- [ ] Flat tiles (slope 0) render consistently regardless of height
- [ ] Valley slopes (5, 10) show correct depression
- [ ] Three-corner-up slopes (7, 11, 13, 14) align with neighbors
- [ ] Diagonal slopes (16-18) render without gaps
- [ ] All rotations (0-3) produce visually consistent results

### OpenRCT2 Reference Renders

For visual comparison:

1. Load identical map in OpenRCT2
2. Set same viewport position/zoom
3. Screenshot key terrain features (valleys, peaks, transitions)
4. Compare with Godot renders to identify drift

## Implementation History

- **Phase 1**: Direct constant replication (`COORDS_XY_STEP`, `COORDS_Z_STEP`, `COORDS_Z_PER_TINY_Z`)
- **Phase 2**: Corner-height table extraction from OpenRCT2 source
- **Phase 3**: GDScript helpers in `SlopeCalculator` for corner math
- **Phase 4**: Refactor `paint_terrain_tile()` to anchor at north corner
- **Phase 5 (pending)**: Edge face rendering

## References

- OpenRCT2 source: `src/openrct2/world/Location.hpp`
- Slope logic: `src/openrct2/world/tile_element/Slope.cpp`
- Surface paint: `src/openrct2/paint/tile_element/Paint.Surface.cpp`
- Godot scripts: `godot-viewer/scripts/TerrainTileMap.gd`, `SlopeCalculator.gd`

---

**Last Updated**: 2025-01-15  
**Status**: Core positioning complete; edge faces pending

