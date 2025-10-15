# OpenRCT2 Visual Parity Plan

## ✅ What's Working (Core Math Complete)

### Corner-Based Positioning
- ✅ Tiles anchor at north corner for seamless alignment
- ✅ Per-corner height calculation matches OpenRCT2's `kSlopeRelativeCornerHeights`
- ✅ Screen Y offsets use exact formula: `(tiny_z * kCoordsZStep) / kCoordsZPerTinyZ * RENDERING_SCALE`
- ✅ All 19 slope variants (0-18) supported
- ✅ Rotation handling via `SlopeCalculator.rotate_slope_index()`
- ✅ Neighboring tiles' shared corners have identical screen Y

**Validation**: The diagnostic scene shows corner math is mathematically correct.

---

## ❌ What's Missing (Visual Polish)

### 1. Edge Faces / Cliff Walls (HIGH PRIORITY)

**Problem**: When tiles have height differences, OpenRCT2 draws vertical cliff faces. Currently, we only render the top surface.

**Example**: Two adjacent tiles at heights 0 and 32 tiny-Z should show a vertical wall between them.

**Solution**:
- Implement `EdgeRenderer.paint_edge_faces()` (stub exists)
- Extract vertical cliff sprites from OpenRCT2's `g1.dat`
- Detect height differences at shared corners
- Draw edge sprites between tiles with `height_diff >= 8` tiny-Z

**Files to modify**:
- `EdgeRenderer.gd` (complete the stub)
- `TerrainTileMap.gd` (call edge renderer after base terrain)
- Extract sprites: Run `extract_rct2_sprites.py` with edge sprite indices

---

### 2. Texture Quality & Baked Lighting

**Problem**: OpenRCT2's terrain sprites have:
- Pre-rendered isometric perspective baked in
- Subtle lighting gradients on slopes
- Anti-aliased edges
- Color variations for visual interest

**Current State**: Our extracted sprites may lack these baked effects if extracted incorrectly or use placeholder textures.

**Investigation Needed**:
1. Check if `assets/tiles/terrain/openrct2_placeholder/grass/slope_*.png` has proper lighting
2. Compare with OpenRCT2 screenshots at same zoom
3. Verify sprite extraction used correct palette and transparency

**Potential Issues**:
- Sprites extracted without proper color palette
- Missing alpha channel or wrong blend mode
- Textures not rendered at OpenRCT2's native resolution (64x32 base)

---

### 3. Sprite Origin / Pivot Points

**Problem**: OpenRCT2 sprites may have specific origin points embedded in sprite metadata that affect visual alignment.

**Current Implementation**: We anchor at north corner mathematically, but sprites themselves may have different visual "hot spots."

**Solution**:
- Check OpenRCT2's `.DAT` sprite metadata for origin offsets
- Add per-slope-type origin adjustments to `paint_terrain_tile()`
- Example: Diagonal slopes (16-18) might need horizontal pivot tweaks

---

### 4. Slope Shading / Depth Cues

**Problem**: OpenRCT2 applies subtle shading overlays to slopes for visual depth.

**Solution** (lower priority):
- Add slope-specific tint modulation
- Apply darker tint to "down" corners, lighter to "up" corners
- Integrate with lighting system (if implementing day/night)

---

### 5. Tile Blending / Smooth Transitions

**Problem**: OpenRCT2 terrain types (grass → dirt) have smooth transitions, not hard edges.

**Current State**: We switch textures abruptly at tile boundaries.

**Solution** (future enhancement):
- Implement tile edge blending (shader-based or sprite variants)
- Add transition sprites (e.g., grass-to-dirt border tiles)
- Reference: OpenRCT2 uses separate "edge" sprites for transitions

---

## 🎯 Next Steps (Priority Order)

### Phase 1: Edge Faces (Critical for Terrain Depth)
1. Extract vertical cliff sprites from OpenRCT2 `g1.dat`
2. Complete `EdgeRenderer.should_draw_edge()` logic
3. Implement `EdgeRenderer.compute_edge_render_data()`
4. Integrate edge rendering into `TerrainTileMap.paint_chunk()`

**Expected Impact**: Terrain will look 3D with vertical walls at height transitions.

### Phase 2: Verify Texture Quality
1. Compare extracted sprites with OpenRCT2 originals
2. Check palette correctness (OpenRCT2 uses specific palette indices)
3. Verify sprite dimensions (64x32 for flat, taller for slopes)
4. Ensure transparency/alpha is preserved

**Expected Impact**: Terrain textures will match OpenRCT2's visual fidelity.

### Phase 3: Sprite Origin Tweaks
1. Extract sprite origin metadata from OpenRCT2
2. Add per-slope horizontal/vertical offset adjustments
3. Test diagonal slopes (16-18) and three-corner-up slopes (7, 11, 13, 14)

**Expected Impact**: Perfect pixel alignment with OpenRCT2 reference renders.

### Phase 4: Visual Polish (Optional)
1. Implement slope shading overlays
2. Add lighting modulation
3. Tile transition blending

---

## 🔍 Debugging: Why It Doesn't Look Like OpenRCT2 Yet

### Current Diagnostic Output Analysis

From terminal logs, the corner math is **correct**:
```
Slope 4 at (16, 0) → corners(N=48,E=32,S=32,W=32) → Y_offset=24.0
```
This shows north corner is 16 tiny-Z higher (48 vs 32), producing 8px screen offset difference (24.0 vs 16.0).

**Math is working** ✅

### Visual Appearance Issues

1. **No vertical walls**: Adjacent tiles at different heights don't show cliffs
2. **Flat lighting**: Sprites may not have baked isometric lighting
3. **No depth perception**: Missing shadows/edge faces make terrain look "paper-thin"

### Comparison Checklist

To diagnose, compare side-by-side:

| Feature | OpenRCT2 | Current Godot Viewer |
|---------|----------|---------------------|
| Corner alignment | Perfect | ✅ Perfect (math correct) |
| Vertical cliffs | Yes | ❌ Missing (no EdgeRenderer) |
| Baked lighting | Yes | ❓ Check extracted sprites |
| Shadow edges | Yes | ❌ Missing |
| Tile depth | 3D appearance | ❌ Looks flat |
| Slope textures | All 19 variants | ✅ Loaded correctly |

---

## 📸 Visual Reference

To validate, capture OpenRCT2 screenshots:

1. Load a simple map with varied heights
2. Set camera to same isometric angle as Godot
3. Match zoom level (`RENDERING_SCALE = 2.0` in Godot)
4. Screenshot key features:
   - Flat tiles at height 0
   - Single raised corner (slope 1-4)
   - Valley tiles (slope 5, 10)
   - Height transitions (cliffs)
5. Compare with Godot viewer render

**Expected Findings**:
- Corner positions will match ✅
- Vertical faces will be missing in Godot ❌
- Lighting/shading may differ ❓

---

## 🛠️ Implementation Commands

### Extract Edge Sprites (Phase 1)
```bash
cd /Users/jean/Github/life-simulator
python3 extract_rct2_sprites.py --type=edges --output=godot-viewer/assets/tiles/edges/
```

### Test Edge Rendering
```gdscript
# In TerrainTileMap.paint_chunk() after base terrain:
for y in range(chunk_size):
    for x in range(chunk_size):
        var tile_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
        var tile_corners = SlopeCalculator.get_corner_heights(height_data[y][x], slope_data[y][x])
        var neighbors = _get_neighbor_corners(tile_pos)  # TODO: implement
        edge_renderer.paint_edge_faces(tile_container, tile_pos, tile_corners, neighbors)
```

---

## 📚 References

- OpenRCT2 edge rendering: `src/openrct2/paint/tile_element/Paint.Surface.cpp`
- Edge sprite indices: `src/openrct2/drawing/Drawing.h` (search for "TERRAIN_EDGE")
- Sprite metadata: `src/openrct2/drawing/ImageTable.cpp`
- Palette data: `src/openrct2/drawing/Palette.cpp`

---

**Status**: Core positioning math complete. Edge faces are the critical missing piece for OpenRCT2-like terrain depth.

**Last Updated**: 2025-01-15

