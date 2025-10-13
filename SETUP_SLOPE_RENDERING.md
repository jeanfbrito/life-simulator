# Setup Guide: Slope-Based Terrain Rendering

**Status:** Ready for OpenRCT2 sprite extraction
**Next Step:** Extract sprites from original RollerCoaster Tycoon 2 / OpenRCT2 game files

---

## Prerequisites Completed ✅

- [x] PRD updated with height map foundation (Phase 3)
- [x] Height map analysis complete (`HEIGHT_MAP_ANALYSIS.md`)
- [x] Sprite extraction guide created (`OPENRCT2_SPRITE_EXTRACTION_GUIDE.md`)
- [x] Implementation guide created (`GODOT_SLOPE_RENDERING_IMPLEMENTATION.md`)
- [x] Directory structure prepared (`godot-viewer/assets/tiles/terrain/`)
- [x] Core scripts created (`SlopeCalculator.gd`, `SlopeDebugOverlay.gd`)

## Current Status

**Waiting for:** Original RollerCoaster Tycoon 2 / OpenRCT2 game files for sprite extraction

Once you have the game files, follow this guide to set up slope-based rendering.

---

## Quick Start (Once You Have Game Files)

### Step 1: Extract Sprites from OpenRCT2

**Location of sprites in OpenRCT2/RCT2:**
- Game data: `<OpenRCT2_install>/data/g1.dat` (compiled sprite database)
- Extracted sprites: Use OpenRCT2's built-in sprite export tools or third-party extractors

**Extraction process:**

```bash
# Option A: Using OpenRCT2 sprite exporter
# (If OpenRCT2 has export functionality)
openrct2 --export-sprites terrain --output ./extracted_sprites

# Option B: Manual extraction from g1.dat
# Use a third-party tool like:
# - RCT2ObjectDataExtractor
# - PkWare sprite extractor
# - OpenRCT2 development tools
```

**What to extract:**
- Terrain tiles: Look for "grass", "sand", "stone", "dirt" tile groups
- Each group has 19 variations (flat + 18 slopes)
- Original size: 32×16 pixels
- Base elevation sprites only (not water or special variants)

**Expected output:**
```
extracted_sprites/
  grass/
    grass_00.png  # Flat
    grass_01.png  # N corner up
    grass_02.png  # E corner up
    ...
    grass_18.png  # Center peak
  sand/
    sand_00.png
    ...
  stone/
    stone_00.png
    ...
```

### Step 2: Organize Sprites (No Scaling Needed!)

**We use OpenRCT2 sprites at their original 32×16 size - no scaling!**

**Direct copy workflow:**

```bash
# Rename and copy sprites (run from extracted_sprites directory)
cd extracted_sprites/grass

# Rename to our convention (slope_00.png format)
for i in {00..18}; do
  cp grass_$i.png slope_$i.png
done

# Copy to Godot project
cp slope_*.png ../../godot-viewer/assets/tiles/terrain/openrct2_placeholder/grass/

# Repeat for other terrain types
cd ../sand
for i in {00..18}; do
  cp sand_$i.png slope_$i.png
done
cp slope_*.png ../../godot-viewer/assets/tiles/terrain/openrct2_placeholder/sand/
```

**That's it!** Original OpenRCT2 sprites work perfectly at 32×16 pixels.

### Step 3: Create Terrain Atlases

Combine individual slope sprites into atlases for Godot TileSet:

```bash
cd godot-viewer/assets/tiles/terrain/openrct2_placeholder/grass

# Create row 0 (slopes 0-9)
montage slope_{00..09}.png \
  -tile 10x1 \
  -geometry 32x16+0+0 \
  -background transparent \
  grass_atlas_row0.png

# Create row 1 (slopes 10-18)
montage slope_{10..18}.png \
  -tile 9x1 \
  -geometry 32x16+0+0 \
  -background transparent \
  grass_atlas_row1.png

# Combine rows into final atlas (320×32 pixels)
convert grass_atlas_row0.png grass_atlas_row1.png \
  -append \
  grass_atlas.png

# Clean up intermediate files
rm grass_atlas_row0.png grass_atlas_row1.png

# Repeat for other terrain types
```

**Expected result:**
- `grass_atlas.png` - 320×32 pixels (10 columns × 2 rows)
- `sand_atlas.png` - Same layout
- `stone_atlas.png` - Same layout
- etc.

### Step 4: Configure Godot TileSet

**Open Godot project:**

```bash
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
```

**Configure TileSet in editor:**

1. Open `scenes/World.tscn`
2. Select `TerrainTileMap` node
3. Click on TileSet in inspector
4. Add new atlas sources:

**For each terrain type:**

- Click "+ Add Atlas"
- **Texture:** Select `assets/tiles/terrain/openrct2_placeholder/grass/grass_atlas.png`
- **Tile Size:** 32×16 (OpenRCT2 original size)
- **Separation:** 0px
- **Margin:** 0px
- **ID:** 0 (Grass), 1 (Sand), 2 (Stone), etc.

**Configure atlas grid:**
- Godot should auto-detect 10×2 grid
- Verify: 20 tiles total (10 in row 0, 10 in row 1)
- Note: Only 19 are used (0-18), slot 19 is empty

**Set terrain IDs:**
Match the source IDs in `TerrainTileMap.gd::get_source_for_terrain()`:
- Source 0 = Grass
- Source 1 = Sand
- Source 2 = Stone
- Source 3 = Dirt
- Source 4 = Forest
- Source 5 = Water
- Source 6 = ShallowWater
- Source 7 = DeepWater
- Source 8 = Desert
- Source 9 = Snow
- Source 10 = Mountain
- Source 11 = Swamp

### Step 5: Add Debug Overlay to Scene

**Open `scenes/World.tscn` in Godot:**

1. Add `SlopeDebugOverlay` as child of root node
2. Set as CanvasLayer
3. Assign script: `scripts/SlopeDebugOverlay.gd`
4. Save scene

**Or add programmatically in WorldRenderer:**

```gdscript
# godot-viewer/scripts/WorldRenderer.gd
func _ready():
	# ... existing setup ...

	# Add debug overlay
	var debug_overlay = preload("res://scripts/SlopeDebugOverlay.gd").new()
	debug_overlay.name = "SlopeDebugOverlay"
	get_tree().root.add_child(debug_overlay)
```

### Step 6: Update Backend to Send Heights

**Verify height data in API:**

```bash
# Start backend
cargo run --bin life-simulator

# Test API
curl -s "http://127.0.0.1:54321/api/chunks?center_x=0&center_y=0&radius=1&layers=true" | jq '.chunks[0].layers | keys'
```

**Expected output:**
```json
[
  "terrain",
  "resources",
  "heights"    ← Must be present!
]
```

**If heights layer is missing:**

1. Verify `HeightMapGenerator` is integrated in `map_generator.rs`
2. Check `SerializedChunk` includes heights in `serialization.rs`
3. Ensure `generate_multi_layer_chunks_json()` serializes heights in `cached_world.rs`

**See:** `HEIGHT_MAP_ANALYSIS.md` for backend implementation details

### Step 7: Test Rendering

**Run full system:**

```bash
# Terminal 1: Backend
cd /Users/jean/Github/life-simulator
cargo run --bin life-simulator

# Terminal 2: Godot
cd /Users/jean/Github/life-simulator/godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5 to run
```

**Test checklist:**

- [ ] Terrain renders with tiles (not colored squares)
- [ ] Flat areas use slope 0 sprite
- [ ] Hills show appropriate slope sprites (1-8)
- [ ] Valleys render correctly (slopes 5, 10)
- [ ] Press F3 to show debug overlay
- [ ] Debug shows height values (0-255)
- [ ] Debug shows correct slope index (0-18)
- [ ] Debug shows neighbor heights
- [ ] Move camera with WASD
- [ ] Debug updates in real-time

**Expected visual result:**
- Isometric terrain with visible elevation changes
- Smooth slope transitions
- No black tiles (missing sprites)
- No flickering at chunk boundaries

### Step 8: Test Chunk Boundaries

**Test cross-chunk slope continuity:**

1. Navigate to chunk boundary (e.g., tile 15 → tile 16)
2. Press F3 to show debug
3. Check neighbor heights include adjacent chunk data
4. Verify slope doesn't abruptly change to flat at boundary

**Debug output should show:**
```
Chunk: (0, 0)
Local: (15, 8)

Neighbor Heights:
  N: 52  (+2)
  E: 50  (0)   ← This E neighbor is in chunk (1, 0)!
  S: 51  (+1)
  W: 50  (0)
```

If E neighbor shows as same height despite visual difference:
- Check `SlopeCalculator::get_neighbor_height()` boundary logic
- Verify `WorldDataCache::get_chunk()` returns adjacent chunks
- Ensure chunks are loaded in 7×7 grid (not just 5×5)

---

## Troubleshooting

### Problem: All tiles render as flat (slope 0)

**Possible causes:**

1. **Backend not sending heights:**
   ```bash
   curl -s "http://127.0.0.1:54321/api/chunks?layers=true" | jq '.chunks[0].layers.heights'
   # Should show 2D array of numbers, not null
   ```

2. **Heights not parsed in Godot:**
   ```gdscript
   # Add to ChunkManager.gd
   print("Chunk data keys: ", chunk_data.keys())
   # Should include "heights"
   ```

3. **HEIGHT_THRESHOLD too high:**
   ```gdscript
   # In SlopeCalculator.gd, try:
   const HEIGHT_THRESHOLD = 2  # Lower value
   ```

### Problem: Black tiles or missing sprites

**Cause:** TileSet atlas coordinates don't match `SLOPE_TO_ATLAS` mapping

**Solution:**
1. Verify atlas has 10×2 grid layout
2. Check first tile is at (0,0), not (1,0)
3. Ensure atlas texture is imported correctly (not compressed)

### Problem: Slopes look wrong at chunk boundaries

**Cause:** Neighbor chunks not loaded when calculating slopes

**Solution:**
1. Increase chunk load radius in `WorldRenderer.gd`
2. Pre-load adjacent chunks before rendering
3. Verify `WorldDataCache::get_chunk()` handles adjacent chunks

### Problem: Performance issues (low FPS)

**Solutions:**

1. **Cache slope indices:**
   ```gdscript
   # Add to TerrainTileMap.gd
   var cached_slopes = {}  # Key: "x,y", Value: slope_idx
   ```

2. **Reduce chunk render calls:**
   - Only render visible chunks
   - Don't re-render already rendered chunks

3. **Profile with Godot:**
   - Run with `--profile` flag
   - Check `SlopeCalculator.calculate_slope_index()` time

---

## Project Structure After Setup

```
godot-viewer/
  assets/
    tiles/
      terrain/
        openrct2_placeholder/
          grass/
            slope_00.png - slope_18.png  ✅ Extracted & scaled
            grass_atlas.png              ✅ Combined atlas
          sand/
            (same structure)             ✅
          stone/
            (same structure)             ✅
          ...
        custom/
          grass/
            (empty - for future custom art)
          ...

  scripts/
    SlopeCalculator.gd                   ✅ Implemented
    SlopeDebugOverlay.gd                 ✅ Implemented
    TerrainTileMap.gd                    🔄 Needs modification
    ChunkManager.gd                      🔄 Needs height parsing
    WorldRenderer.gd                     ✅ No changes needed

  scenes/
    World.tscn                           🔄 Add SlopeDebugOverlay
                                         🔄 Configure TileSet atlases
```

**Legend:**
- ✅ Complete
- 🔄 Needs modification
- ❌ Not started

---

## Next Steps After Setup

### Phase 1: Basic Rendering (Week 1)
- [x] Extract grass sprites (19 slopes)
- [x] Create grass atlas
- [x] Configure TileSet with grass
- [x] Test basic slope rendering
- [x] Verify debug overlay works

### Phase 2: All Terrain Types (Week 2)
- [ ] Extract sand, stone, dirt sprites
- [ ] Extract water variant sprites
- [ ] Create atlases for all types
- [ ] Configure TileSet with all atlases
- [ ] Test terrain type switching

### Phase 3: Backend Integration (Week 3)
- [ ] Implement height map generation in backend
- [ ] Verify height data in API responses
- [ ] Test chunk boundary continuity
- [ ] Profile performance with full world

### Phase 4: Custom Art (Weeks 4-12)
- [ ] Draw custom grass flat tile (slope 0)
- [ ] Draw custom grass slopes 1-4
- [ ] Draw custom grass slopes 5-9
- [ ] Draw custom grass slopes 10-18
- [ ] Repeat for sand terrain
- [ ] Repeat for stone terrain
- [ ] Continue per terrain type

---

## Resources

- **Implementation Guide:** `/GODOT_SLOPE_RENDERING_IMPLEMENTATION.md`
- **Sprite Extraction:** `/OPENRCT2_SPRITE_EXTRACTION_GUIDE.md`
- **Height Maps:** `/HEIGHT_MAP_ANALYSIS.md`
- **PRD Reference:** `.taskmaster/docs/prd.txt` (Phase 3: Terrain Generation)
- **Godot Docs:** https://docs.godotengine.org/en/stable/classes/class_tilemap.html
- **OpenRCT2 Source:** `/Users/jean/Github/OpenRCT2/`

---

## Legal Reminder

Extracted OpenRCT2/RCT2 sprites are **temporary placeholders only**:
- ✅ Use for development and testing
- ✅ Learn from and reference for style
- ❌ DO NOT distribute with final game
- ❌ DO NOT use in public releases

All placeholder sprites must be replaced with original custom art before public distribution.

**Custom art in `custom/` directory is your original work and can be freely distributed under this project's license (MIT/Apache 2.0).**

---

**Ready to begin!** Once you have the OpenRCT2 game files, start with Step 1 (sprite extraction).
