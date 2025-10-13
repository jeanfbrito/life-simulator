# Godot Slope Rendering Implementation Guide

**Reference:** `OPENRCT2_SPRITE_EXTRACTION_GUIDE.md` for sprite extraction process
**Reference:** `HEIGHT_MAP_ANALYSIS.md` for height map system design
**Project:** Life Simulator - OpenRCT2-style 2D Isometric Terrain Rendering

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Data Structures](#data-structures)
4. [Slope Calculation System](#slope-calculation-system)
5. [TileMap Configuration](#tilemap-configuration)
6. [Rendering Pipeline](#rendering-pipeline)
7. [Integration with Backend](#integration-with-backend)
8. [Testing & Debugging](#testing--debugging)
9. [Performance Optimization](#performance-optimization)
10. [Complete Code Examples](#complete-code-examples)

---

## Overview

This guide implements OpenRCT2-style slope-based terrain rendering in Godot 4.3+. The system:

- ✅ Receives height data from Rust backend (0-255 per tile)
- ✅ Calculates slope indices (0-18) based on corner height differences
- ✅ Selects appropriate slope sprite from atlas
- ✅ Renders isometric tiles (32×16 pixels - OpenRCT2 original size)
- ✅ Supports all terrain types with 19 slope variations each
- ✅ Integrates with existing chunk-based rendering

**Key Principle:** Height determines slope, slope determines sprite. The Godot viewer is purely presentational - all height data comes from the backend.

---

## Architecture

```
Backend (Rust)                    Godot Viewer (GDScript)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ChunkManager                      WorldDataCache
  └─ chunks: HashMap                └─ chunks: Dictionary
       └─ layers: HashMap                 └─ "heights": [[u8; 16]; 16]
            ├─ "terrain"                  └─ "terrain": [["Grass"; 16]; 16]
            ├─ "resources"
            └─ "heights" ←──────────┐
                                     │
WorldLoader                          │
  └─ generate_chunk()                │
       └─ height_generator    ───────┘
            └─ simplex_noise          │
                                      │
HTTP API                              │
  GET /api/chunks                     │
    ?center_x=0&center_y=0            │
    &radius=3                         │
    &layers=true ─────────────────────┘

                                  WorldRenderer
                                    └─ TerrainTileMap (MODIFIED)
                                         ├─ render_chunk()
                                         │    ├─ for each tile:
                                         │    │    ├─ get height
                                         │    │    ├─ calculate_slope()
                                         │    │    └─ set_cell(slope_idx)
                                         │    └─ paint to TileMap
                                         │
                                         └─ SlopeCalculator (NEW)
                                              ├─ get_neighbor_heights()
                                              ├─ calculate_slope_index()
                                              └─ SLOPE_TO_ATLAS[] lookup
```

---

## Data Structures

### Backend: Height Data in Chunks

The Rust backend sends chunks with height layers:

```json
{
  "chunk": {
    "x": 0,
    "y": 0,
    "layers": {
      "terrain": [
        ["Grass", "Grass", "Forest", ...],
        ...
      ],
      "heights": [
        [50, 51, 52, 53, ...],
        [49, 50, 51, 52, ...],
        ...
      ]
    }
  }
}
```

**Height Format:**
- Type: `u8` (0-255)
- 0 = Lowest point (deep ocean floor)
- 255 = Highest point (mountain peak)
- Water level typically at 50

### Godot: Cached Height Data

**File:** `godot-viewer/scripts/WorldDataCache.gd`

```gdscript
# Existing cache structure - ADD heights field
var chunks = {}  # Key: "x,y", Value: chunk_data

# Example cached chunk:
chunks["0,0"] = {
    "terrain": [
        ["Grass", "Grass", "Forest", ...],
        ...
    ],
    "resources": [...],
    "heights": [      # NEW FIELD
        [50, 51, 52, 53, ...],
        [49, 50, 51, 52, ...],
        ...
    ]
}
```

### Slope Index Mapping

**19 Slope Variations** (matches OpenRCT2 exactly):

```gdscript
# godot-viewer/scripts/TerrainTileMap.gd

# Slope index to atlas coordinate mapping
# Each terrain type has 19 tiles arranged in atlas
const SLOPE_TO_ATLAS = {
    0: Vector2i(0, 0),   # Flat
    1: Vector2i(1, 0),   # N corner up
    2: Vector2i(2, 0),   # E corner up
    3: Vector2i(3, 0),   # NE side up
    4: Vector2i(4, 0),   # S corner up
    5: Vector2i(5, 0),   # NS valley
    6: Vector2i(6, 0),   # SE side up
    7: Vector2i(7, 0),   # NES corners up
    8: Vector2i(8, 0),   # W corner up
    9: Vector2i(9, 0),   # NW side up
    10: Vector2i(0, 1),  # EW valley
    11: Vector2i(1, 1),  # NEW corners up
    12: Vector2i(2, 1),  # SW side up
    13: Vector2i(3, 1),  # NWS corners up
    14: Vector2i(4, 1),  # ESW corners up
    15: Vector2i(5, 1),  # All corners up (plateau)
    16: Vector2i(6, 1),  # Diagonal NE-SW
    17: Vector2i(7, 1),  # Diagonal NW-SE
    18: Vector2i(8, 1),  # Center peak
}
```

---

## Slope Calculation System

### Core Algorithm

**File:** `godot-viewer/scripts/SlopeCalculator.gd` (NEW)

```gdscript
class_name SlopeCalculator
extends RefCounted

## Calculates slope indices from height data (OpenRCT2 style)
##
## Slope is determined by comparing a tile's height with its 4 neighbors (N/E/S/W).
## Each raised neighbor sets a bit in the slope value (4 bits = 16+ combinations).

const CORNER_N = 0b0001
const CORNER_E = 0b0010
const CORNER_S = 0b0100
const CORNER_W = 0b1000

# Threshold for considering a height difference significant
const HEIGHT_THRESHOLD = 5  # Minimum height difference to create slope

## Calculate slope index for a tile based on neighbor heights
##
## @param heights: 2D array of u8 heights (16×16 for chunk)
## @param local_pos: Vector2i position within chunk (0-15, 0-15)
## @param chunk_coord: Vector2i chunk coordinate (for boundary checks)
## @param world_cache: WorldDataCache reference (for neighbor chunk access)
## @returns: int (0-18) slope index
static func calculate_slope_index(
    heights: Array,
    local_pos: Vector2i,
    chunk_coord: Vector2i,
    world_cache: Node
) -> int:
    var current_height = heights[local_pos.y][local_pos.x]

    # Get neighbor heights (handles chunk boundaries)
    var h_n = get_neighbor_height(heights, local_pos, Vector2i(0, -1), chunk_coord, world_cache)
    var h_e = get_neighbor_height(heights, local_pos, Vector2i(1, 0), chunk_coord, world_cache)
    var h_s = get_neighbor_height(heights, local_pos, Vector2i(0, 1), chunk_coord, world_cache)
    var h_w = get_neighbor_height(heights, local_pos, Vector2i(-1, 0), chunk_coord, world_cache)

    # Build slope bitfield (OpenRCT2 style)
    var slope = 0

    if h_n > current_height + HEIGHT_THRESHOLD:
        slope |= CORNER_N
    if h_e > current_height + HEIGHT_THRESHOLD:
        slope |= CORNER_E
    if h_s > current_height + HEIGHT_THRESHOLD:
        slope |= CORNER_S
    if h_w > current_height + HEIGHT_THRESHOLD:
        slope |= CORNER_W

    # Check for special cases (valleys, diagonals)
    return slope_to_index(slope, h_n, h_e, h_s, h_w, current_height)


## Get height of neighbor tile (handles chunk boundaries)
static func get_neighbor_height(
    heights: Array,
    local_pos: Vector2i,
    offset: Vector2i,
    chunk_coord: Vector2i,
    world_cache: Node
) -> int:
    var neighbor_pos = local_pos + offset

    # Check if neighbor is within current chunk
    if neighbor_pos.x >= 0 and neighbor_pos.x < 16 and \
       neighbor_pos.y >= 0 and neighbor_pos.y < 16:
        return heights[neighbor_pos.y][neighbor_pos.x]

    # Neighbor is in adjacent chunk - fetch from cache
    var neighbor_chunk_coord = chunk_coord
    var neighbor_local_pos = neighbor_pos

    # Adjust chunk coordinate and local position
    if neighbor_pos.x < 0:
        neighbor_chunk_coord.x -= 1
        neighbor_local_pos.x = 15
    elif neighbor_pos.x >= 16:
        neighbor_chunk_coord.x += 1
        neighbor_local_pos.x = 0

    if neighbor_pos.y < 0:
        neighbor_chunk_coord.y -= 1
        neighbor_local_pos.y = 15
    elif neighbor_pos.y >= 16:
        neighbor_chunk_coord.y += 1
        neighbor_local_pos.y = 0

    # Get neighbor chunk from cache
    var chunk_key = "%d,%d" % [neighbor_chunk_coord.x, neighbor_chunk_coord.y]
    var neighbor_chunk = world_cache.get_chunk(chunk_key)

    if neighbor_chunk == null or not neighbor_chunk.has("heights"):
        # Neighbor chunk not loaded - assume same height (flat transition)
        return heights[local_pos.y][local_pos.x]

    return neighbor_chunk["heights"][neighbor_local_pos.y][neighbor_local_pos.x]


## Convert slope bitfield to atlas index (0-18)
static func slope_to_index(
    slope: int,
    h_n: int, h_e: int, h_s: int, h_w: int,
    current: int
) -> int:
    # Basic 4-bit slopes (0-15)
    match slope:
        0b0000: return 0   # Flat
        0b0001: return 1   # N up
        0b0010: return 2   # E up
        0b0011: return 3   # NE side up
        0b0100: return 4   # S up
        0b0101: return 5   # NS valley
        0b0110: return 6   # SE side up
        0b0111: return 7   # NES corners up
        0b1000: return 8   # W up
        0b1001: return 9   # NW side up
        0b1010: return 10  # EW valley
        0b1011: return 11  # NEW corners up
        0b1100: return 12  # SW side up
        0b1101: return 13  # NWS corners up
        0b1110: return 14  # ESW corners up
        0b1111: return 15  # All corners up

    # Check for diagonal slopes (16-18)
    # Diagonal NE-SW: N and E high, S and W low (or vice versa)
    if (h_n > current and h_e > current and h_s < current and h_w < current) or \
       (h_n < current and h_e < current and h_s > current and h_w > current):
        return 16

    # Diagonal NW-SE: N and W high, S and E low (or vice versa)
    if (h_n > current and h_w > current and h_s < current and h_e < current) or \
       (h_n < current and h_w < current and h_s > current and h_e > current):
        return 17

    # Center peak: tile higher than all neighbors
    if h_n < current and h_e < current and h_s < current and h_w < current:
        return 18

    # Fallback to flat
    return 0
```

---

## TileMap Configuration

### Atlas Setup

**File:** `godot-viewer/assets/tiles/terrain/terrain_atlas.tres`

Each terrain type needs a texture atlas with 19 tiles arranged in 2 rows:

```
Row 0: Slopes 0-9   (Flat, N-up, E-up, NE-up, S-up, NS-valley, SE-up, NES-up, W-up, NW-up)
Row 1: Slopes 10-18 (EW-valley, NEW-up, SW-up, NWS-up, ESW-up, All-up, Diag-NE-SW, Diag-NW-SE, Peak)
```

**Atlas Configuration:**
- Tile Size: 32×16 pixels (OpenRCT2 original size)
- Separation: 0px
- Margins: 0px
- Texture Region: 320×32 pixels (10 columns × 2 rows)

### TileSet Configuration

**File:** `godot-viewer/scenes/World.tscn` → TerrainTileMap → TileSet

```gdscript
# TileSet structure (configured in editor):
#
# Physics Layer 0: Tile-based terrain (for future collision)
#
# Terrain Type 0: "Grass"
#   - Source: grass_atlas.png (320×32)
#   - Alternative Tiles: 0-18 (auto-generated)
#
# Terrain Type 1: "Sand"
#   - Source: sand_atlas.png (320×32)
#   - Alternative Tiles: 0-18
#
# ... (repeat for all terrain types)
```

**Important:** Each terrain type is a **separate atlas texture**, not a single mega-atlas. This allows easy sprite replacement per-terrain-type.

---

## Rendering Pipeline

### Modified TerrainTileMap

**File:** `godot-viewer/scripts/TerrainTileMap.gd`

```gdscript
extends TileMap

# EXISTING CONSTANTS (keep these)
const CHUNK_SIZE = 16
const TILE_SIZE = 32  # Isometric tile width (OpenRCT2 original size)

# NEW: Slope atlas mapping
const SLOPE_TO_ATLAS = {
    0: Vector2i(0, 0),   1: Vector2i(1, 0),   2: Vector2i(2, 0),
    3: Vector2i(3, 0),   4: Vector2i(4, 0),   5: Vector2i(5, 0),
    6: Vector2i(6, 0),   7: Vector2i(7, 0),   8: Vector2i(8, 0),
    9: Vector2i(9, 0),  10: Vector2i(0, 1),  11: Vector2i(1, 1),
   12: Vector2i(2, 1),  13: Vector2i(3, 1),  14: Vector2i(4, 1),
   15: Vector2i(5, 1),  16: Vector2i(6, 1),  17: Vector2i(7, 1),
   18: Vector2i(8, 1),
}

# References
@onready var world_cache = get_node("/root/WorldDataCache")
var slope_calculator = preload("res://scripts/SlopeCalculator.gd")

# ... (existing terrain color sources setup) ...


## MODIFIED: Render chunk with slope-based tiles
func render_chunk(chunk_coord: Vector2i, chunk_data: Dictionary) -> void:
    if not chunk_data.has("terrain"):
        push_error("Chunk missing 'terrain' layer: %s" % chunk_coord)
        return

    var terrain_data = chunk_data["terrain"]
    var has_heights = chunk_data.has("heights")

    if not has_heights:
        push_warning("Chunk missing 'heights' layer, rendering flat: %s" % chunk_coord)

    # Render all tiles in chunk
    for local_y in range(CHUNK_SIZE):
        for local_x in range(CHUNK_SIZE):
            var terrain_str = terrain_data[local_y][local_x]

            # Calculate tile position in world coordinates
            var world_tile_x = chunk_coord.x * CHUNK_SIZE + local_x
            var world_tile_y = chunk_coord.y * CHUNK_SIZE + local_y
            var tile_coord = Vector2i(world_tile_x, world_tile_y)

            # Get terrain source ID
            var source_id = get_source_for_terrain(terrain_str)
            if source_id == -1:
                continue  # Skip unknown terrain

            # Calculate slope index
            var slope_index = 0  # Default: flat
            if has_heights:
                var heights = chunk_data["heights"]
                var local_pos = Vector2i(local_x, local_y)
                slope_index = slope_calculator.calculate_slope_index(
                    heights,
                    local_pos,
                    chunk_coord,
                    world_cache
                )

            # Get atlas coordinates for slope
            var atlas_coord = SLOPE_TO_ATLAS[slope_index]

            # Paint tile with specific slope variant
            set_cell(0, tile_coord, source_id, atlas_coord)

    print("✅ Rendered chunk %s with slopes (heights: %s)" % [chunk_coord, has_heights])


## Get TileSet source ID for terrain type
func get_source_for_terrain(terrain_str: String) -> int:
    # Map terrain names to source IDs (must match TileSet configuration)
    match terrain_str:
        "Grass": return 0
        "Sand": return 1
        "Stone": return 2
        "Dirt": return 3
        "Forest": return 4
        "Water": return 5
        "ShallowWater": return 6
        "DeepWater": return 7
        "Desert": return 8
        "Snow": return 9
        "Mountain": return 10
        "Swamp": return 11
        _:
            push_warning("Unknown terrain type: %s" % terrain_str)
            return -1
```

### Integration with WorldRenderer

**File:** `godot-viewer/scripts/WorldRenderer.gd`

No changes needed! The existing chunk rendering system automatically calls `TerrainTileMap.render_chunk()`, which now includes slope calculation.

```gdscript
# EXISTING CODE - No changes required
func render_chunks_around_camera():
    # ... (existing camera chunk calculation) ...

    for chunk_coord in visible_chunks:
        if chunk_coord in rendered_chunks:
            continue

        var chunk_key = "%d,%d" % [chunk_coord.x, chunk_coord.y]
        var chunk_data = world_cache.get_chunk(chunk_key)

        if chunk_data:
            terrain_tilemap.render_chunk(chunk_coord, chunk_data)  # ← Now includes slopes!
            rendered_chunks[chunk_coord] = true
```

---

## Integration with Backend

### Backend Changes Required

**File:** `src/web_server_simple.rs`

Ensure `/api/chunks` endpoint returns height data:

```rust
// EXISTING: Multi-layer chunk endpoint
// GET /api/chunks?center_x=0&center_y=0&radius=3&layers=true

// Response must include "heights" layer:
{
    "chunks": [
        {
            "x": 0,
            "y": 0,
            "layers": {
                "terrain": [["Grass", "Grass", ...]],
                "resources": [["Wood", "", ...]],
                "heights": [[50, 51, 52, ...]]  // ← CRITICAL
            }
        }
    ]
}
```

**File:** `src/serialization.rs`

Verify `SerializedChunk` includes heights:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedChunk {
    pub x: i32,
    pub y: i32,
    pub layers: HashMap<String, Vec<Vec<String>>>,  // Must include "heights"
}
```

**File:** `src/cached_world.rs`

Update `generate_multi_layer_chunks_json()` to serialize heights as strings:

```rust
// Convert heights layer (Vec<Vec<u8>>) to Vec<Vec<String>>
if let Some(heights) = chunk.layers.get("heights") {
    let heights_strings: Vec<Vec<String>> = heights
        .iter()
        .map(|row| row.iter().map(|h| h.to_string()).collect())
        .collect();

    layers_map.insert("heights".to_string(), heights_strings);
}
```

### Godot Backend Connection

**File:** `godot-viewer/scripts/ChunkManager.gd`

Update to parse height data:

```gdscript
func _parse_chunk_response(json_data: Dictionary) -> void:
    if not json_data.has("chunks"):
        return

    for chunk_dict in json_data["chunks"]:
        var chunk_coord = Vector2i(chunk_dict["x"], chunk_dict["y"])

        # Parse layers
        var chunk_data = {}
        if chunk_dict.has("layers"):
            var layers = chunk_dict["layers"]

            # Terrain layer
            if layers.has("terrain"):
                chunk_data["terrain"] = layers["terrain"]

            # Resources layer
            if layers.has("resources"):
                chunk_data["resources"] = layers["resources"]

            # Heights layer (NEW)
            if layers.has("heights"):
                # Convert string heights to integers
                var heights_int = []
                for row in layers["heights"]:
                    var row_int = []
                    for height_str in row:
                        row_int.append(int(height_str))
                    heights_int.append(row_int)
                chunk_data["heights"] = heights_int

        # Cache chunk
        world_cache.add_chunk(chunk_key, chunk_data)
```

---

## Testing & Debugging

### Debug Visualization

**File:** `godot-viewer/scripts/SlopeDebugOverlay.gd` (NEW)

```gdscript
extends CanvasLayer

## Debug overlay showing slope indices and heights
## Toggle with F3 key

@onready var label = $Label
var visible_state = false

func _ready():
    visible = false

func _input(event):
    if event is InputEventKey and event.pressed and event.keycode == KEY_F3:
        visible_state = !visible_state
        visible = visible_state

func _process(_delta):
    if not visible_state:
        return

    # Get camera position
    var camera = get_viewport().get_camera_2d()
    if not camera:
        return

    var camera_pos = camera.global_position
    var world_cache = get_node("/root/WorldDataCache")

    # Convert camera position to tile coordinate
    var tile_pos = Vector2i(
        int(camera_pos.x / 32),
        int(camera_pos.y / 16)
    )

    # Get chunk coordinate
    var chunk_coord = Vector2i(
        tile_pos.x / 16,
        tile_pos.y / 16
    )
    var local_pos = Vector2i(
        tile_pos.x % 16,
        tile_pos.y % 16
    )

    # Get chunk data
    var chunk_key = "%d,%d" % [chunk_coord.x, chunk_coord.y]
    var chunk_data = world_cache.get_chunk(chunk_key)

    if not chunk_data or not chunk_data.has("heights"):
        label.text = "No height data"
        return

    # Display height info
    var heights = chunk_data["heights"]
    var current_height = heights[local_pos.y][local_pos.x]

    # Calculate slope
    var slope_calc = preload("res://scripts/SlopeCalculator.gd")
    var slope_idx = slope_calc.calculate_slope_index(
        heights,
        local_pos,
        chunk_coord,
        world_cache
    )

    label.text = """
    Tile: (%d, %d)
    Chunk: (%d, %d) Local: (%d, %d)
    Height: %d
    Slope Index: %d
    """ % [tile_pos.x, tile_pos.y, chunk_coord.x, chunk_coord.y,
           local_pos.x, local_pos.y, current_height, slope_idx]
```

Add to `World.tscn`:
- Add SlopeDebugOverlay as CanvasLayer child
- Add Label node with custom theme (monospace font, semi-transparent background)

### Test Cases

**Test 1: Flat Terrain**
```gdscript
# All heights same = slope 0 (flat)
var test_heights = [
    [50, 50, 50, 50],
    [50, 50, 50, 50],
    [50, 50, 50, 50],
    [50, 50, 50, 50]
]
# Expected slope for (1,1): 0
```

**Test 2: North Slope**
```gdscript
# North neighbor higher = slope 1 (N corner up)
var test_heights = [
    [55, 55, 55, 55],
    [50, 50, 50, 50],
    [50, 50, 50, 50],
    [50, 50, 50, 50]
]
# Expected slope for (1,1): 1
```

**Test 3: Valley**
```gdscript
# North and South higher = slope 5 (NS valley)
var test_heights = [
    [55, 55, 55, 55],
    [50, 50, 50, 50],
    [55, 55, 55, 55],
    [50, 50, 50, 50]
]
# Expected slope for (1,1): 5 (0b0101)
```

**Test 4: Peak**
```gdscript
# Tile higher than all neighbors = slope 18 (peak)
var test_heights = [
    [50, 50, 50, 50],
    [50, 60, 50, 50],
    [50, 50, 50, 50],
    [50, 50, 50, 50]
]
# Expected slope for (1,1): 18
```

### Unit Test Script

**File:** `godot-viewer/tests/test_slope_calculation.gd`

```gdscript
extends GutTest

## Unit tests for SlopeCalculator
## Run with Gut plugin: https://github.com/bitwes/Gut

var SlopeCalculator = preload("res://scripts/SlopeCalculator.gd")
var mock_cache = MockWorldCache.new()

func test_flat_terrain():
    var heights = [
        [50, 50, 50],
        [50, 50, 50],
        [50, 50, 50]
    ]
    var slope = SlopeCalculator.calculate_slope_index(
        heights, Vector2i(1, 1), Vector2i(0, 0), mock_cache
    )
    assert_eq(slope, 0, "Flat terrain should return slope 0")

func test_north_slope():
    var heights = [
        [60, 60, 60],
        [50, 50, 50],
        [50, 50, 50]
    ]
    var slope = SlopeCalculator.calculate_slope_index(
        heights, Vector2i(1, 1), Vector2i(0, 0), mock_cache
    )
    assert_eq(slope, 1, "North higher should return slope 1")

func test_valley_ns():
    var heights = [
        [60, 60, 60],
        [50, 50, 50],
        [60, 60, 60]
    ]
    var slope = SlopeCalculator.calculate_slope_index(
        heights, Vector2i(1, 1), Vector2i(0, 0), mock_cache
    )
    assert_eq(slope, 5, "NS valley should return slope 5")

# Mock WorldDataCache for testing
class MockWorldCache:
    func get_chunk(_key: String):
        return null
```

---

## Performance Optimization

### Optimization Strategies

1. **Cache Slope Indices**
   - Calculate slopes once per chunk
   - Store in `rendered_chunks` metadata
   - Only recalculate when heights change

2. **Batch Rendering**
   - Use `set_cells_terrain_connect()` for large areas
   - Reduces TileMap update overhead

3. **Lazy Neighbor Loading**
   - Only fetch neighbor chunks when at chunk boundaries
   - Cache neighbor heights temporarily

4. **LOD System (Future)**
   - Use flat tiles (slope 0) for distant chunks
   - Calculate slopes only for visible chunks

### Profiling Points

**Critical Performance Metrics:**
- **Slope calculation time**: Should be <1ms per chunk (256 tiles)
- **TileMap update time**: Should be <5ms per chunk
- **Total render time**: Should be <100ms for 49 chunks (7×7 grid)

**Add profiling code:**

```gdscript
func render_chunk(chunk_coord: Vector2i, chunk_data: Dictionary) -> void:
    var start_time = Time.get_ticks_msec()

    # ... (existing rendering code) ...

    var elapsed = Time.get_ticks_msec() - start_time
    if elapsed > 10:
        push_warning("Slow chunk render: %d ms for %s" % [elapsed, chunk_coord])
```

---

## Complete Code Examples

### Example 1: Simple Height Map Test

**Create test world with simple heights:**

```bash
# Backend: Generate test world with height pattern
cargo run --bin map_generator -- \
    --name "test_slopes" \
    --seed 12345 \
    --radius 2 \
    --height-config "simple_hills"
```

**Expected output:**
- Center flat (height 50)
- North side elevated (height 70)
- South side low (height 30)
- East/West gradual slopes

### Example 2: Full Integration Test

**Terminal 1: Backend**
```bash
cd /Users/jean/Github/life-simulator
cargo run --bin life-simulator
```

**Terminal 2: Godot**
```bash
cd /Users/jean/Github/life-simulator/godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
```

**In Godot:**
1. Press F5 to run
2. Press F3 to toggle slope debug overlay
3. Move camera with WASD
4. Observe slope indices changing at terrain transitions

**Expected behavior:**
- Flat areas: Slope 0
- Hills: Slopes 1-8 (single corners up)
- Valleys: Slopes 5, 10 (opposite corners up)
- Peaks: Slope 18 (all neighbors lower)

### Example 3: Terrain Transition Test

**Test chunk boundary slope continuity:**

```gdscript
# godot-viewer/tests/test_chunk_boundaries.gd
extends GutTest

func test_chunk_boundary_slope():
    var world_cache = get_node("/root/WorldDataCache")

    # Load two adjacent chunks
    await load_chunks([Vector2i(0, 0), Vector2i(1, 0)])

    # Get heights at boundary
    var chunk0 = world_cache.get_chunk("0,0")
    var chunk1 = world_cache.get_chunk("1,0")

    # Tile at right edge of chunk 0
    var edge_height = chunk0["heights"][8][15]

    # Tile at left edge of chunk 1
    var neighbor_height = chunk1["heights"][8][0]

    # Calculate slope for edge tile
    var slope = SlopeCalculator.calculate_slope_index(
        chunk0["heights"],
        Vector2i(15, 8),
        Vector2i(0, 0),
        world_cache
    )

    # Slope should consider neighbor chunk height
    if neighbor_height > edge_height + 5:
        assert_true(slope & SlopeCalculator.CORNER_E != 0,
                    "East corner should be raised")
```

---

## Next Steps

### Phase 1: Foundation (Week 1)
- [ ] Extract grass terrain sprites (19 slopes) from OpenRCT2
- [ ] Create `SlopeCalculator.gd` with core algorithm
- [ ] Implement `TerrainTileMap` slope rendering
- [ ] Test with flat and simple slopes

### Phase 2: Backend Integration (Week 2)
- [ ] Verify backend sends height data in `/api/chunks`
- [ ] Update `ChunkManager.gd` to parse heights
- [ ] Update `WorldDataCache.gd` to store heights
- [ ] Test end-to-end rendering

### Phase 3: All Terrains (Week 3)
- [ ] Extract sprites for Sand, Stone, Dirt
- [ ] Extract sprites for Water variants
- [ ] Configure TileSet with all terrain atlases
- [ ] Test terrain type switching with slopes

### Phase 4: Polish & Optimization (Week 4)
- [ ] Add slope debug overlay (F3 key)
- [ ] Implement slope index caching
- [ ] Profile performance with 49-chunk grid
- [ ] Add unit tests for edge cases

### Phase 5: Custom Art Replacement (Weeks 5-8)
- [ ] Draw custom grass flat tile (replace slope 0)
- [ ] Draw custom grass slopes 1-18
- [ ] Draw custom sand terrain
- [ ] Continue replacing per `OPENRCT2_SPRITE_EXTRACTION_GUIDE.md`

---

## Troubleshooting

### Problem: All tiles render as flat (slope 0)

**Possible causes:**
1. Backend not sending height data
   - Check: `curl http://127.0.0.1:54321/api/chunks?layers=true`
   - Verify response includes `"heights"` layer
2. Heights not parsed in `ChunkManager.gd`
   - Add debug: `print(chunk_data.keys())` after parsing
3. Height threshold too high
   - Try lowering `HEIGHT_THRESHOLD` to 2-3

### Problem: Slopes don't match terrain visually

**Solution:**
- Check neighbor heights with debug overlay (F3)
- Verify atlas coordinates match actual sprite positions
- Ensure atlas tile size is exactly 32×16 (OpenRCT2 original size)

### Problem: Slopes look wrong at chunk boundaries

**Solution:**
- Verify `get_neighbor_height()` correctly fetches from adjacent chunks
- Check chunk boundary coordinates: (15, y) → (0, y) in next chunk
- Add debug logging for cross-chunk neighbor fetches

### Problem: Performance issues with slope calculation

**Solution:**
- Profile with `--profile` flag in Godot
- Cache slopes after calculation
- Only recalculate when heights change
- Use flat tiles for distant chunks (LOD)

---

## References

- **OpenRCT2 Source:** `/Users/jean/Github/OpenRCT2/src/openrct2/world/tile_element/`
- **Sprite Guide:** `OPENRCT2_SPRITE_EXTRACTION_GUIDE.md`
- **Height System:** `HEIGHT_MAP_ANALYSIS.md`
- **Godot Docs:** https://docs.godotengine.org/en/stable/classes/class_tilemap.html
- **OpenRCT2 Wiki:** https://github.com/OpenRCT2/OpenRCT2/wiki

---

**Ready to implement?** Start with Phase 1 (extracting grass sprites and `SlopeCalculator.gd`). The rendering system is now fully specified and ready for integration!
