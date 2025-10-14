# CLAUDE.md - Godot Isometric Viewer

This file provides guidance to Claude Code when working with the Godot isometric viewer for the Life Simulator project.

## Project Overview

The Godot viewer is an **isometric 2D client** for the Life Simulator that connects to the same backend API as the web viewer. It provides a native desktop experience with hardware-accelerated rendering.

**Architecture:** Separated client-server model
- Backend: Rust/Bevy headless simulation (port 54321)
- Godot Client: Connects via HTTP API for world data
- Data sharing: Uses identical chunk structure as web viewer

## Critical: Isometric Camera Positioning

### ⚠️ THE #1 GOTCHA: Camera Must Use Pixel Coordinates

**NEVER set camera position directly to tile coordinates!**

```gdscript
# ❌ WRONG - Camera will be thousands of pixels off-screen
camera.position = Vector2(0, 0)  # This is pixel (0,0), not tile (0,0)!

# ✅ CORRECT - Convert tile coordinates to pixel space first
var center_tile = Vector2i(0, 0)
var center_pixel = terrain_tilemap.map_to_local(center_tile)
camera.position = center_pixel  # Now camera sees tile (0,0)
```

### Why This Matters

**Isometric tiles have non-obvious pixel positions:**
- Tile size: 64×32 pixels (OpenRCT2 EXACT MATCH)
- Tile (0, 0) → Pixel (0, 0)
- Tile (-48, -48) → Pixel (-3072, -768) ⚠️
- Tile (10, 10) → Pixel (0, 320)

**Formula for isometric projection (OpenRCT2 EXACT):**
```
pixel_x = (tile_x - tile_y) × kCoordsXYStep
pixel_y = (tile_x + tile_y) × (kCoordsXYStep / 2) - height_offset

Where:
kCoordsXYStep = 32
height_offset = (height × kCoordsZStep) / kCoordsZPerTinyZ
              = (height × 8) / 16
              = height / 2
```

For OpenRCT2 64×32 tiles:
```
pixel_x = (tile_x - tile_y) × 32
pixel_y = (tile_x + tile_y) × 16 - height / 2
```

### Camera Zoom for Isometric Tiles

**Default zoom levels:**
- `zoom = 0.5`: Standard view for OpenRCT2 64×32 tiles (default)
- `zoom = 1.0`: Zoomed in for detailed inspection
- `zoom = 2.0`: Close-up view for tile-by-tile work

**Rule of thumb:** Larger tiles need smaller zoom values.

## Coordinate Systems Overview

The viewer uses **four different coordinate systems**:

### 1. World Tile Coordinates
- Integer tile positions in the world
- Example: `(0, 0)`, `(5, 10)`, `(-48, -48)`
- Used by backend API and chunk data

### 2. Chunk Coordinates
- Which 16×16 chunk a tile belongs to
- Example: Tile (32, 16) → Chunk (2, 1)
- Formula: `chunk_x = floor(tile_x / 16)`

### 3. Local Chunk Coordinates
- Tile position within its chunk (0-15)
- Example: Tile (18, 5) → Chunk (1, 0), Local (2, 5)
- Formula: `local_x = ((tile_x % 16) + 16) % 16`

### 4. Pixel/Screen Coordinates
- Actual pixel position on screen
- **Depends on tile shape and size!**
- Isometric: Use `map_to_local()` to convert
- Orthogonal: `pixel = tile × tile_size`

### Conversion Functions

```gdscript
# Tile → Pixel (use TileMap built-in)
var pixel_pos = tilemap.map_to_local(tile_pos)

# Pixel → Tile (use TileMap built-in)
var tile_pos = tilemap.local_to_map(pixel_pos)

# Tile → Chunk (use WorldDataCache)
var chunk_key = WorldDataCache.get_chunk_key(tile_x, tile_y)

# Chunk → Tile Origin (use WorldDataCache)
var tile_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)
```

## TileMap Layer Configuration

**Godot 4.x requires explicit layer setup:**

```gdscript
func _ready():
    # Ensure layer 0 exists
    if get_layers_count() == 0:
        add_layer(-1)  # Add layer at end

    # Enable layer and make visible
    set_layer_enabled(0, true)
    set_layer_modulate(0, Color(1, 1, 1, 1))
```

**Common mistake:** Forgetting to enable layers → blank screen!

## Async Chunk Loading Pattern

**CRITICAL:** Don't mark chunks as "rendered" until they're actually painted!

```gdscript
# ❌ WRONG - Marks all chunks done before painting
func _update_visible_chunks():
    var visible_chunks = _get_visible_chunks()
    current_chunk_keys = visible_chunks  // ⚠️ Premature!
    _add_visible_chunks(visible_chunks)

# ✅ CORRECT - Only mark chunks that were actually painted
func _update_visible_chunks():
    var visible_chunks = _get_visible_chunks()
    var newly_painted = _add_visible_chunks(visible_chunks)

    for chunk_key in newly_painted:
        if not current_chunk_keys.has(chunk_key):
            current_chunk_keys.append(chunk_key)
```

**Why this matters:**
- Chunks load in batches (10 at a time)
- `_update_visible_chunks()` runs after each batch
- If all 49 chunks marked "done" after batch 1, batches 2-5 won't paint!

## TileSet Configuration

### Programmatic Creation

```gdscript
var tileset = TileSet.new()
tileset.tile_shape = TileSet.TILE_SHAPE_ISOMETRIC  # Shape: 1
tileset.tile_layout = TileSet.TILE_LAYOUT_STACKED   # Layout: 1
tileset.tile_size = Vector2i(64, 32)  # OpenRCT2 isometric diamond (EXACT)

# Create atlas source with diamond texture
var source = TileSetAtlasSource.new()
source.texture = create_diamond_texture()
source.texture_region_size = Vector2i(64, 32)
source.create_tile(Vector2i(0, 0))

var source_id = tileset.add_source(source)
```

### Per-Terrain Colored Sources

**Pattern:** Create one source per terrain type with colored diamond texture

```gdscript
func _get_or_create_terrain_source(terrain_type: String, color: Color) -> int:
    if not has_meta("terrain_sources"):
        set_meta("terrain_sources", {})

    var sources = get_meta("terrain_sources")

    if sources.has(terrain_type):
        return sources[terrain_type]

    # Create new colored source
    var source = TileSetAtlasSource.new()
    source.texture = create_colored_diamond_texture(color)
    source.texture_region_size = Vector2i(64, 32)
    source.create_tile(Vector2i(0, 0))

    var source_id = self.tile_set.add_source(source)
    sources[terrain_type] = source_id

    return source_id
```

## Painting Tiles: No Coordinate Conversion Needed!

**CRITICAL:** When painting tiles, `set_cell()` expects tile coordinates, not pixel coordinates!

```gdscript
# ❌ WRONG - Converts tile coords to pixel, then back to tile (broken!)
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String):
    var tile_coords = local_to_map(Vector2(world_pos.x, world_pos.y))  # ⚠️ Wrong!
    set_cell(0, tile_coords, source_id, Vector2i(0, 0))

# ✅ CORRECT - Use tile coordinates directly
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String):
    # world_pos is already in tile coordinates!
    set_cell(0, world_pos, source_id, Vector2i(0, 0))
    # TileMap handles isometric projection automatically
```

**Why `local_to_map()` is wrong here:**
- `local_to_map()` converts **pixel coordinates → tile coordinates**
- `world_pos` is already a **tile coordinate**
- Passing tile coords to `local_to_map()` treats them as pixel coords
- Result: Tiles appear at wildly wrong positions

**When to use coordinate conversion:**
- `map_to_local()`: When positioning camera (tile → pixel) and entities (tile → pixel)
- `local_to_map()`: When converting mouse clicks (pixel → tile)
- `set_cell()`: NEVER - it expects tile coordinates directly

## Entity Positioning in Isometric Space

**Entities must be positioned in pixel coordinates using `map_to_local()`:**

```gdscript
# ✅ CORRECT - Entity positioning from backend tile coordinates
var entity_tile_pos = Vector2i(entity_data.position.x, entity_data.position.y)
var entity_pixel_pos = tilemap.map_to_local(entity_tile_pos)

# Apply Y offset to keep feet in grid (-0.2 tiles)
entity_pixel_pos.y += Config.TILE_SIZE * config.offset_y  # offset_y = -0.2

container.position = entity_pixel_pos  # Set Node2D position in pixels
```

**Why this works:**
- Backend sends entity positions in tile coordinates (e.g., `{x: 5, y: 10}`)
- Godot Node2D positions are in pixel coordinates
- `map_to_local()` converts tile → pixel with isometric projection
- Y-offset of -0.2 tiles moves entity upward to keep feet in tile boundary

**Common mistakes:**
- ❌ Setting entity position directly to tile values: `entity.position = Vector2(5, 10)`
- ❌ Forgetting Y-offset: entities appear too low in the tile
- ❌ Using pixel coordinates for tile-based queries

## Project Structure

```
godot-viewer/
├── CLAUDE.md                  # This file - project guidance
├── README.md                  # User-facing documentation
├── project.godot              # Godot 4.5 project file
├── docs/                      # Detailed documentation
│   └── CAMERA_COORDINATES.md  # Deep dive on coordinate systems
├── scenes/
│   ├── World.tscn            # Main scene (run this)
│   ├── Main.tscn             # Alternative main scene
│   └── *Test.tscn            # Test scenes for components
├── scripts/
│   ├── WorldRenderer.gd      # Main renderer + camera control
│   ├── TerrainTileMap.gd     # Isometric tilemap rendering
│   ├── ChunkManager.gd       # HTTP chunk loading
│   ├── WorldDataCache.gd     # Chunk data caching
│   └── Config.gd             # Singleton configuration
└── resources/
    └── SimpleTerrainTileSet.tres  # Pre-built tileset (optional)
```

## Running the Viewer

### Prerequisites

1. **Start the backend first:**
   ```bash
   cd /Users/jean/Github/life-simulator
   cargo run --bin life-simulator
   ```
   Backend starts on `http://localhost:54321`

2. **Open in Godot:**
   ```bash
   cd godot-viewer
   /Applications/Godot.app/Contents/MacOS/Godot --path .
   ```

3. **Press F5 to run** (or click Play button)

### Expected Startup Sequence

```
Config singleton initialized
Loading species configuration from API...
ChunkManager initialized
WorldDataCache initialized
🌳 ResourceManager initialized
🐇 EntityManager initialized
🗺️ TerrainTileMap initialized
✅ TileSet loaded successfully
🎨 Terrain mapping setup for 12 terrain types
📊 TopBar initialized successfully
🌍 WorldRenderer initialized
📹 Camera positioned at tile (0, 0) = pixel (0.0, 0.0) with zoom 0.5x (OpenRCT2 exact)
🚀 Starting world loading...
✅ Species configuration loaded from API
Species loaded: ["default", "Bear", "Deer", "Fox", "Human", "Rabbit", "Raccoon", "Wolf"]
📦 Loading chunks around: (0, 0) (radius: 5)
🎨 Painted 10 new chunks (total visible: 49)
🌳 Rendered X resources for chunk Y
🐇 Spawned entity 0 (Rabbit) at (x, y)
🐇 Spawned entity 1 (Wolf) at (x, y)
...
📊 Total rendered chunks: 49 / 49 visible
   - Total cells rendered: 12544
✅ World loading completed - viewer should show terrain
```

### Keyboard Controls

**Camera:**
- **Arrow Keys / WASD**: Pan camera
- **+/-**: Zoom in/out
- **R**: Reset camera to origin

**UI Toggles:**
- **G**: Toggle grid overlay
- **Tab**: Toggle statistics HUD
- **H**: Toggle help/controls overlay

**System:**
- **Escape**: Quit application

## API Integration

### API Endpoints

```gdscript
# Load world info
GET http://localhost:54321/api/world/current
GET http://localhost:54321/api/world_info

# Load species configuration
GET http://localhost:54321/api/species
# Response: { "species": {...}, "default_entity": {...}, "juvenile_scales": {...} }

# Load chunk data (with layers)
GET http://localhost:54321/api/chunks?coords=0,0&coords=1,0&layers=true

# Response format
{
  "chunk_data": {
    "0,0": {
      "terrain": [[...], [...], ...],  // 16x16 array of terrain types
      "resources": [[...], [...], ...]  // 16x16 array of resource types
    }
  }
}

# Poll entities (every 200ms)
GET http://localhost:54321/api/entities
# Response: { "entities": [{id, name, entity_type, position, current_action, ...}, ...] }
```

### Chunk Data Structure

**Same as web viewer:**
- Terrain: 16×16 2D array of terrain type strings
- Resources: 16×16 2D array of resource type strings
- Chunk key: `"x,y"` format (e.g., `"0,0"`, `"-3,2"`)

## Common Debugging Techniques

### Check Camera Position

```gdscript
print("Camera position: ", camera.position)
print("Camera zoom: ", camera.zoom)

# Check if camera can see tile (0, 0)
var tile_pixel = terrain_tilemap.map_to_local(Vector2i(0, 0))
print("Tile (0,0) is at pixel: ", tile_pixel)
```

### Check TileMap State

```gdscript
print("Total cells: ", terrain_tilemap.get_used_cells(0).size())
print("TileSet exists: ", terrain_tilemap.tile_set != null)
print("Layer 0 enabled: ", terrain_tilemap.is_layer_enabled(0))

# Sample tile positions
var cells = terrain_tilemap.get_used_cells(0).slice(0, 5)
for cell_pos in cells:
    var pixel_pos = terrain_tilemap.map_to_local(cell_pos)
    print("Tile ", cell_pos, " → Pixel ", pixel_pos)
```

### Check Chunk Loading

```gdscript
# In WorldDataCache
print("Cached chunks: ", terrain_cache.size())
print("Chunk keys: ", terrain_cache.keys())

# Check specific chunk
var chunk_data = get_terrain_chunk("0,0")
print("Chunk 0,0 size: ", chunk_data.size())
```

## Performance Considerations

### Chunk Batching

- Load 10 chunks per batch (avoid URL length limits)
- Add 100ms delay between chunk requests
- Total load time for 49 chunks: ~5 seconds

### TileMap Optimization

- Use cached colored sources (don't recreate textures)
- Layer 0 only (single rendering layer)
- Diamond textures: 64×32 pixels = 2KB each (OpenRCT2 EXACT)

### Memory Usage

- 49 chunks × 256 tiles = 12,544 rendered cells
- Each cell: 1 texture reference + tile data
- Estimated: ~2-3 MB for full island

## Implemented Features (2025-01-11)

1. ✅ **Resource Rendering** - Trees, bushes, rocks, flowers with emoji symbols
2. ✅ **Entity Rendering** - All 7 species with proper emojis and scaling
3. ✅ **Entity Polling** - Real-time updates every 200ms from backend
4. ✅ **Species Configuration** - Loaded from `/api/species` endpoint
5. ✅ **Action Labels** - Entity current actions displayed above sprites
6. ✅ **Y-Sorting** - Proper depth sorting for entities and resources
7. ✅ **Top Bar UI** - Statistics, FPS, entity counts, action buttons

## Known Limitations

1. **Camera controls** - Arrow keys only, no mouse drag/wheel zoom yet
2. **Discrete movement** - Entities jump between tiles (matches backend simulation)
3. **Static camera** - No smooth scrolling or interpolation

## Future Enhancements

- [ ] Implement smooth camera pan/zoom with mouse
- [ ] Add minimap in corner
- [ ] Mouse click to inspect tiles
- [ ] Show tile coordinates on hover
- [ ] Add more detailed entity tooltips
- [ ] Implement camera edge scrolling

## References

- Main project: `/Users/jean/Github/life-simulator/CLAUDE.md`
- Godot Isometric Docs: https://docs.godotengine.org/en/stable/tutorials/2d/using_tilemaps.html#isometric-tiles
- API Backend: `src/web_server_simple.rs`
- Web Viewer: `web-viewer/js/renderer.js`

## Key Lessons Learned

### 2025-01-11: Camera Positioning Bug

**Symptom:** Blank gray screen despite chunks loading successfully

**Root Causes:**
1. Camera at pixel (0, 0) but tiles at pixel (-6016, -1504)
2. Only 10/49 chunks painted due to premature marking

**Solutions:**
1. Use `map_to_local()` to position camera on tile (0, 0)
2. Only mark chunks as rendered after actual painting

**Files Fixed:**
- `scripts/WorldRenderer.gd`: Camera positioning, chunk tracking
- `scripts/TerrainTileMap.gd`: Removed incorrect `local_to_map()` calls

**Documentation Created:**
- `godot-viewer/CLAUDE.md`: Project guidance
- `godot-viewer/docs/CAMERA_COORDINATES.md`: Detailed coordinate system guide

See `docs/CAMERA_COORDINATES.md` for comprehensive explanation of coordinate systems and common pitfalls.
