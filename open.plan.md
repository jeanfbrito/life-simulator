<!-- fc199cc9-c72c-4977-81b2-69fd8cf16637 e7b79beb-3cdf-42fb-aa2d-76a9e1d9ac37 -->
# Mirror OpenRCT2 Terrain Offsets in Godot Viewer

### 1) Map current data flow (read-only)

- Inspect how tiles are painted and where inputs arrive:
- `godot-viewer/scripts/TerrainTileMap.gd` → `paint_chunk(...)`, `paint_terrain_tile(...)`, `map_to_local(...)`
- `godot-viewer/scripts/WorldDataCache.gd` → `chunk_key_to_world_origin(...)`, `get_height_chunk(...)`, `get_slope_chunk(...)`
- `godot-viewer/scripts/RCT2TerrainTextureManager.gd` → `get_terrain_texture(terrain_type, slope_index)`
- `godot-viewer/scripts/SlopeCalculator.gd` → `rotate_slope_index(...)`
- Confirm constants used at render time inside `TerrainTileMap.gd` (`COORDS_XY_STEP`, `COORDS_Z_STEP`, `COORDS_Z_PER_TINY_Z`, `RENDERING_SCALE`).
- Verify height units from `height_data` (u8 0–255 tiny-Z) and current screen Y offset formula in `paint_terrain_tile()` is `height * kCoordsZStep / kCoordsZPerTinyZ`.

### 2) Extract OpenRCT2 corner-height logic (reference only)

- Constants and coordinate system (no code changes):
- `OpenRCT2/src/openrct2/world/Location.hpp` → `kCoordsXYStep=32`, `kCoordsZStep=8`, `kCoordsZPerTinyZ=16`, tile screen sizes 64×32.
- Corner-height tables:
- `OpenRCT2/src/openrct2/world/tile_element/Slope.h` → `SlopeRelativeCornerHeights`, `GetSlopeRelativeCornerHeights(...)`, `kTileSlopeDiagonalFlag`.
- `OpenRCT2/src/openrct2/world/tile_element/Slope.cpp` → table for relative corner heights and `GetSlopeCornerHeights(...)`.
- Paint consumption (screen offsets, edges):
- `OpenRCT2/src/openrct2/paint/tile_element/Paint.Surface.cpp` → surface placement, `viewport_surface_paint_data`, edge/side drawing, diagonal handling.

### 3) Design Godot-side corner math (new helpers in GDScript)

- Extend `godot-viewer/scripts/SlopeCalculator.gd`:
- `get_relative_corner_heights(slope_index: int) -> Dictionary` returning `{top, right, bottom, left}` (values 0–2) mirroring OpenRCT2's `SlopeRelativeCornerHeights`.
- `get_corner_heights(base_height: int, slope_index: int) -> Dictionary` returning per-corner tiny-Z: `base_height + rel*LAND_HEIGHT_STEP` where `LAND_HEIGHT_STEP = Config.COORDS_Z_PER_TINY_Z` (16).
- Screen Y per-corner: `corner_screen_y = (corner_tiny_z * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z * RENDERING_SCALE`.
- Anchor/pivot convention:
- Anchor sprite at the tile's north/top corner (OpenRCT2 'north') so that sprite Y = `base_pos.y - north_corner_screen_y`.
- For each rotation, use `rotate_slope_index(...)` before lookup so cardinal corners map consistently.
- Add optional small horizontal pivot tweaks (if needed) guided by `viewport_surface_paint_data`.

### 4) Refactor renderer to use corner math (surgical edits)

- In `TerrainTileMap.gd`:
- Add helper: `compute_corner_offsets_screen(base_height: int, slope_index: int) -> Dictionary` returning `{top, right, bottom, left}` in pixels.
- Update `paint_terrain_tile(...)` to:
  - Rotate slope index via `SlopeCalculator.rotate_slope_index`.
  - Fetch corner heights via new helper.
  - Compute `final_pos` from `map_to_local(world_pos)` and subtract the anchor (north/top) corner Y offset — remove the ad‑hoc `+ 0.5 * COORDS_Z_STEP` fudge.
  - Keep Z-sorting based on `final_pos.y`.
- Keep texture selection unchanged (uses pre-baked slope textures per index).
- Ensure the same math works for all rotations (the only rotation entry point stays `rotate_slope_index`).

### 5) Plan edge/transition integration (follow-up after baseline)

- Identify vertical edges when adjacent tiles' matching corners differ.
- Add an `EdgeRenderer.gd` (or helper functions) that, given neighbor corner deltas, draws side faces or overlays per OpenRCT2's `Paint.Surface.cpp` logic (e.g., `TileSurfaceBoundaryData`, edge images). Defer detailed sprite set until baseline alignment is validated.

### 6) Testing & validation

- Add a diagnostic scene `godot-viewer/tests/SlopeDiagnostics.tscn`:
- Grid of representative slopes: single raised corner, side slopes, NS/WE valleys, diagonals (16/17), peak (18).
- Toggle to print: base height, slope index (rotated/raw), per-corner tiny‑Z, per-corner screen Y.
- Use/extend `SlopeDebugOverlay.gd` and `GridOverlay.gd` to overlay labels at the four corners.
- Verification:
- Numeric cross-check: neighboring tiles' shared corners have identical screen Y.
- Optional visual reference: capture OpenRCT2 screenshots with the same configs to spot drift.

### 7) Documentation & rollout

- Add a short note (new `docs/OPENRCT2_TERRAIN_OFFSETS.md` or extend `GODOT_SLOPE_RENDERING_IMPLEMENTATION.md`) covering:
- Constants used, units (tiny‑Z, pixel mapping), corner anchoring convention, rotation flow.
- Known gaps (edge faces, smoothing/shading) and follow-up tasks.

### To-dos

- [x] Trace terrain/height/slope into `TerrainTileMap.paint_*` and record height units
- [x] Copy references for slope corner tables and constants from OpenRCT2
- [x] Add GDScript helpers to return relative and absolute corner heights
- [x] Update `paint_terrain_tile` to anchor at north corner using new math
- [x] Sketch edge/transition stub mirroring OpenRCT2 sides; defer art assets
- [x] Create slope diagnostics scene and toggles; extend overlays
- [x] Write offsets doc with units and math; list follow-ups

---

## ✅ Implementation Complete (2025-01-15)

All 7 steps of the plan have been implemented:

### Files Modified
- **`godot-viewer/scripts/SlopeCalculator.gd`**: Added `get_relative_corner_heights()` and `get_corner_heights()` helpers
- **`godot-viewer/scripts/TerrainTileMap.gd`**: Added `compute_corner_offsets_screen()` and refactored `paint_terrain_tile()` to use north-corner anchoring

### Files Created
- **`godot-viewer/scripts/EdgeRenderer.gd`**: Stub for future edge face rendering
- **`godot-viewer/docs/OPENRCT2_TERRAIN_OFFSETS.md`**: Comprehensive documentation
- **`godot-viewer/tests/SlopeDiagnostics.tscn`**: Diagnostic scene with UI controls
- **`godot-viewer/tests/SlopeDiagnostics.gd`**: Test script displaying all 19 slope variants
- **`godot-viewer/tests/CornerMarker.gd`**: Visual marker helper for corners

### Key Changes
1. Corner math now mirrors OpenRCT2's `kSlopeRelativeCornerHeights` table (32 entries including steep diagonals)
2. Tiles anchor at north corner for seamless alignment
3. Removed ad-hoc `+0.5 * COORDS_Z_STEP` fudge factor
4. Debug output shows per-corner heights and screen offsets

### Next Steps (Follow-up Work)
- Test diagnostic scene in Godot editor
- Validate alignment with neighboring tiles
- Compare visual output with OpenRCT2 screenshots
- Implement edge face rendering (use `EdgeRenderer.gd` stub)
- Extract cliff/edge sprites from OpenRCT2's g1.dat

