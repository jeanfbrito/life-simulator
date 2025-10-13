# Godot Viewer Current Status

**Last Updated:** 2025-01-13
**Status:** ✅ Fully Functional with Colored Tiles

---

## ✅ What's Already Working

### 1. Terrain Rendering
- **32×16 isometric diamond tiles** rendering correctly
- **12 terrain types** with distinct colors:
  - Grass (green), Forest (dark green), Sand (tan)
  - Water/ShallowWater/DeepWater (blues)
  - Dirt (brown), Stone (gray), Mountain (light gray)
  - Desert (tan), Snow (white), Swamp (olive)
- **Programmatic colored tile generation** (`create_colored_diamond_texture()`)
- **Chunk-based loading** (49 chunks = 7×7 grid)
- **Pixel-perfect rendering** (TEXTURE_FILTER_NEAREST)

### 2. Camera System
- **Isometric camera** with proper coordinate conversion
- **Controls:**
  - WASD / Arrow keys: Pan
  - +/- keys: Zoom (0.25× - 4.0×)
  - R: Reset to origin
- **Edge scrolling** (50px margin)
- **Mouse drag panning** (middle button)
- **Smooth camera interpolation**
- **Positioned correctly** at tile (0,0) using `map_to_local()`

### 3. Resource Rendering
- **Emoji-based rendering** (🌳 🪨 🌸 🫐)
- **Y-sorting** for depth perception
- **Position offsets** to keep resources in tiles
- **Type-specific scaling** and placement

### 4. Entity Rendering
- **7 species** (Rabbit, Deer, Wolf, Bear, Fox, Raccoon, Human)
- **Real-time entity polling** (200ms interval)
- **Species-specific emojis** loaded from API
- **Scaling based on species** (juvenile scales)
- **Action labels** displayed above entities
- **Y-offset of -0.2 tiles** to keep feet in grid

### 5. UI Overlays
- **Top Bar** with world info and statistics
- **Statistics HUD** (Tab key toggle)
  - FPS, memory, entity counts
  - Chunk info, camera position
- **Controls Overlay** (H key toggle)
- **Grid Overlay** (G key toggle)

### 6. Backend Integration
- **HTTP API connection** to port 54321
- **Multi-layer chunk loading** (terrain + resources)
- **Species configuration** loaded from `/api/species`
- **World info** from `/api/world/current`
- **Entity polling** from `/api/entities`

### 7. Performance
- **49 chunks** (12,544 tiles) render smoothly
- **60 FPS** maintained
- **Chunk caching** in WorldDataCache
- **Batched HTTP requests** (10 chunks per batch)
- **Rendering quadrant optimization** (16×16)

---

## 🔄 What's Ready But Not Yet Active

### Slope System (Prepared)
- **`SlopeCalculator.gd`** implemented ✅
  - 19 slope variation calculations
  - Cross-chunk boundary support
  - HEIGHT_THRESHOLD configuration
- **`SlopeDebugOverlay.gd`** created ✅
  - F3 toggle debug panel
  - Real-time slope visualization
- **Directory structure** prepared ✅
  - `assets/tiles/terrain/openrct2_placeholder/`
  - `assets/tiles/terrain/custom/`
- **Documentation** complete ✅
  - GODOT_SLOPE_RENDERING_IMPLEMENTATION.md
  - SETUP_SLOPE_RENDERING.md
  - WINDOWS_SPRITE_EXTRACTION.md

**Waiting for:**
- OpenRCT2 sprite atlases from Windows extraction
- Backend height data in API responses

---

## 🚧 What Needs to Be Done

### Phase 1: Add OpenRCT2 Sprites (Windows → macOS)

**Step 1: Extract on Windows**
Follow `WINDOWS_SPRITE_EXTRACTION.md`:
1. Download Trigger's Graphics Extractor
2. Extract sprites from g1.dat
3. Organize with PowerShell script
4. Create atlases with ImageMagick
5. Transfer to macOS

**Step 2: Integrate in Godot (macOS)**
1. Copy atlas files to `godot-viewer/assets/tiles/terrain/openrct2_placeholder/`
2. Open `scenes/World.tscn` in Godot
3. Select TerrainTileMap node
4. Configure TileSet:
   - Add atlas sources (one per terrain type)
   - Set tile size: 32×16
   - Assign source IDs: 0=Grass, 1=Sand, 2=Stone, etc.

**Step 3: Update TerrainTileMap.gd**
Replace colored tile generation with atlas-based rendering:

```gdscript
# Current: Programmatic colored tiles
func render_chunk(chunk_coord: Vector2i, chunk_data: Dictionary):
    var source_id = _get_or_create_terrain_source(terrain_str, color)
    set_cell(0, tile_coord, source_id, Vector2i(0, 0))

# New: Atlas-based with slopes
func render_chunk(chunk_coord: Vector2i, chunk_data: Dictionary):
    var source_id = get_source_for_terrain(terrain_str)  # Fixed mapping
    var slope_idx = 0  # Will use height data later
    var atlas_coord = SLOPE_TO_ATLAS[slope_idx]
    set_cell(0, tile_coord, source_id, atlas_coord)
```

**Step 4: Test**
- Run Godot viewer (F5)
- Should see OpenRCT2 sprite tiles instead of colored diamonds
- All 49 chunks should render
- Press G to toggle grid overlay

---

### Phase 2: Add Height Maps (Backend)

**Location:** `src/` (Rust backend)

**Step 1: Implement HeightMapGenerator**
File: `src/tilemap/height_generator.rs` (NEW)

```rust
use noise::{Fbm, NoiseFn, Simplex};

pub struct HeightMapGenerator {
    noise: Fbm<Simplex>,
    config: HeightMapConfig,
}

pub struct HeightMapConfig {
    pub min_height: u8,      // 20 (deep ocean floor)
    pub max_height: u8,      // 200 (mountain peaks)
    pub water_level: u8,     // 50 (sea level)
    pub frequency: f64,      // 1.5
    pub octaves: usize,      // 6
    pub lacunarity: f64,     // 2.0
    pub persistence: f64,    // 0.5
    pub smoothing_passes: u8, // 3
}

impl HeightMapGenerator {
    pub fn generate_for_chunk(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<u8>> {
        // Generate 16×16 height values using simplex noise
        // Apply box blur smoothing
        // Normalize to 0-255 range
    }
}
```

**Step 2: Add Heights to SerializedChunk**
File: `src/serialization.rs`

```rust
// layers HashMap already supports any string key
// Add "heights" layer:
chunk.set_layer("heights", heights_as_strings);
```

**Step 3: Update Map Generator**
File: `src/map_generator.rs`

```rust
// Add height generation to world generation
let height_generator = HeightMapGenerator::new(config);
let heights = height_generator.generate_for_chunk(chunk_x, chunk_y);
chunk.set_layer("heights", convert_heights_to_strings(heights));
```

**Step 4: Update CachedWorld**
File: `src/cached_world.rs`

```rust
// generate_multi_layer_chunks_json() already handles any layer
// Heights will automatically be included in JSON response
```

**Step 5: Test Backend**
```bash
cargo run --bin map_generator  # Regenerate world with heights
cargo run --bin life-simulator # Start server

# Test API
curl -s "http://127.0.0.1:54321/api/chunks?center_x=0&center_y=0&radius=1&layers=true" | jq '.chunks[0].layers.heights'
# Should show 16×16 array of height values (as strings)
```

---

### Phase 3: Integrate Heights + Slopes in Godot

**Step 1: Parse Heights in ChunkManager**
File: `godot-viewer/scripts/ChunkManager.gd`

```gdscript
# Add height parsing to _parse_chunk_response()
if layers.has("heights"):
    var heights_int = []
    for row in layers["heights"]:
        var row_int = []
        for height_str in row:
            row_int.append(int(height_str))
        heights_int.append(row_int)
    chunk_data["heights"] = heights_int
```

**Step 2: Enable Slope Calculation**
File: `godot-viewer/scripts/TerrainTileMap.gd`

```gdscript
# Uncomment slope calculation in render_chunk()
func render_chunk(chunk_coord: Vector2i, chunk_data: Dictionary):
    # ... existing code ...

    # Calculate slope index from heights
    if chunk_data.has("heights"):
        var heights = chunk_data["heights"]
        var slope_idx = SlopeCalculator.calculate_slope_index(
            heights,
            Vector2i(local_x, local_y),
            chunk_coord,
            world_cache
        )
        var atlas_coord = SLOPE_TO_ATLAS[slope_idx]
        set_cell(0, tile_coord, source_id, atlas_coord)
    else:
        # Fallback to flat tile
        set_cell(0, tile_coord, source_id, Vector2i(0, 0))
```

**Step 3: Add Debug Overlay**
File: `godot-viewer/scenes/World.tscn`

1. Add SlopeDebugOverlay as CanvasLayer child
2. Assign `scripts/SlopeDebugOverlay.gd` script
3. Press F3 in-game to test

**Step 4: Test Full System**
```bash
# Terminal 1: Backend with heights
cargo run --bin life-simulator

# Terminal 2: Godot
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5 to run
# Press F3 to show slope debug overlay
```

**Expected:**
- Terrain renders with OpenRCT2 slope sprites
- Hills show raised corner sprites
- Valleys show valley sprites
- F3 overlay shows height values and slope indices
- Slopes transition smoothly across chunk boundaries

---

## 📋 Integration Checklist

### Current Phase: Sprite Extraction

- [ ] **Windows: Extract sprites** (WINDOWS_SPRITE_EXTRACTION.md)
  - [ ] Download Trigger's Graphics Extractor
  - [ ] Locate g1.dat file
  - [ ] Run extraction (5000+ sprites)
  - [ ] Run PowerShell organize script
  - [ ] Install ImageMagick
  - [ ] Run atlas batch file
  - [ ] Verify 6 atlases created (320×32 pixels)
  - [ ] Transfer to macOS

- [ ] **macOS: Integrate sprites in Godot**
  - [ ] Copy atlases to `godot-viewer/assets/tiles/terrain/openrct2_placeholder/`
  - [ ] Open World.tscn in Godot editor
  - [ ] Configure TileSet (add 6 atlas sources)
  - [ ] Update TerrainTileMap.gd to use atlases
  - [ ] Test: Run viewer, verify OpenRCT2 tiles render

### Next Phase: Backend Heights

- [ ] **Implement HeightMapGenerator**
  - [ ] Create `src/tilemap/height_generator.rs`
  - [ ] Add noise-rs dependency
  - [ ] Implement simplex noise generation
  - [ ] Add box blur smoothing
  - [ ] Add configuration struct

- [ ] **Integrate in Map Generator**
  - [ ] Update `src/map_generator.rs`
  - [ ] Generate heights for each chunk
  - [ ] Add heights to SerializedChunk
  - [ ] Test: Verify heights in saved world files

- [ ] **Update API**
  - [ ] Verify CachedWorld sends heights
  - [ ] Test: curl API, check heights in response
  - [ ] Verify JSON format correct

### Final Phase: Slopes in Godot

- [ ] **Parse Heights**
  - [ ] Update ChunkManager.gd
  - [ ] Convert string heights to ints
  - [ ] Cache in WorldDataCache

- [ ] **Enable Slope Rendering**
  - [ ] Update TerrainTileMap.gd
  - [ ] Call SlopeCalculator for each tile
  - [ ] Map slope index to atlas coordinate
  - [ ] Handle missing heights gracefully

- [ ] **Add Debug Tools**
  - [ ] Add SlopeDebugOverlay to World.tscn
  - [ ] Test F3 toggle
  - [ ] Verify slope indices shown correctly

- [ ] **End-to-End Testing**
  - [ ] Generate world with varied terrain
  - [ ] Load in Godot viewer
  - [ ] Verify slopes render correctly
  - [ ] Test chunk boundary transitions
  - [ ] Check performance (60 FPS maintained)

---

## 🎯 Current Working State

**You can run the viewer RIGHT NOW:**

```bash
# Terminal 1: Start backend
cd /Users/jean/Github/life-simulator
cargo run --bin life-simulator

# Terminal 2: Start Godot viewer
cd /Users/jean/Github/life-simulator/godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5 to run
```

**What you'll see:**
- ✅ Isometric map with 49 chunks (7×7 grid)
- ✅ 12 terrain types with distinct colors
- ✅ Trees, rocks, bushes (emoji-based)
- ✅ Live animals moving around
- ✅ Smooth camera controls
- ✅ Real-time statistics HUD

**What's missing:**
- ❌ OpenRCT2 sprite tiles (using colored diamonds instead)
- ❌ Height-based slopes (all tiles are flat)
- ❌ Slope debug overlay (F3 won't show anything yet)

---

## 📚 Documentation Reference

**Setup Guides:**
- `WINDOWS_SPRITE_EXTRACTION.md` - Extract sprites on Windows
- `SETUP_SLOPE_RENDERING.md` - Integration steps for slopes
- `GODOT_SLOPE_RENDERING_IMPLEMENTATION.md` - Technical implementation details

**Analysis Documents:**
- `HEIGHT_MAP_ANALYSIS.md` - Backend height system design
- `OPENRCT2_SPRITE_EXTRACTION_GUIDE.md` - Sprite format and organization

**Project Docs:**
- `CLAUDE.md` - Main project documentation
- `godot-viewer/CLAUDE.md` - Godot-specific guidance
- `.taskmaster/docs/prd.txt` - Product requirements document

---

## 🚀 Next Immediate Action

**Your next step:** Follow `WINDOWS_SPRITE_EXTRACTION.md` on Windows machine to extract OpenRCT2 terrain sprites. Once you have the atlas files, bring them back to macOS and we'll integrate them into Godot.

**Estimated time:** 30-45 minutes on Windows + 15 minutes integration on macOS = ~1 hour total for Phase 1.

After sprites are working, we can add backend height maps (Phase 2) and connect everything together (Phase 3).
