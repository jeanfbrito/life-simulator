# Camera and Coordinate Systems in Godot Isometric Rendering

**Last Updated:** 2025-01-11
**Author:** Debugging Session - Camera Positioning Bug Fix

## Table of Contents

1. [The Problem We Encountered](#the-problem-we-encountered)
2. [Understanding Coordinate Systems](#understanding-coordinate-systems)
3. [Isometric Projection Math](#isometric-projection-math)
4. [Camera Positioning Rules](#camera-positioning-rules)
5. [Common Pitfalls and Solutions](#common-pitfalls-and-solutions)
6. [Debugging Checklist](#debugging-checklist)
7. [Code Examples](#code-examples)

---

## The Problem We Encountered

### Symptom

- Godot viewer showed **blank gray screen**
- Console logs showed chunks loading successfully
- 2560 tiles painted, but not visible
- Web viewer worked perfectly with same data

### Investigation

**Console output revealed:**
```
🎨 Painted terrain tile at world (-48, -48) (pixel: (-6016.0, -1504.0))
📹 Camera positioned at island center (0,0) with zoom 0.5x
📹 Camera actual position: (0.0, 0.0) zoom: (0.5, 0.5)
```

**The smoking gun:**
- Camera at pixel **(0, 0)**
- Tiles rendering at pixel **(-6016, -1504)**
- Distance: **Over 6000 pixels away!**

### Root Causes

1. **Camera Position Bug:** Setting `camera.position = Vector2(0, 0)` doesn't position camera at **tile (0, 0)**, it positions it at **pixel (0, 0)**!

2. **Chunk Painting Bug:** Only 10/49 chunks painted because `current_chunk_keys` was updated prematurely, preventing subsequent batches from painting.

---

## Understanding Coordinate Systems

Godot isometric rendering uses **four distinct coordinate systems**. Confusing these is the #1 source of bugs.

### 1. World Tile Coordinates

**What:** Integer positions of tiles in the game world
**Range:** Unbounded integers (..., -2, -1, 0, 1, 2, ...)
**Used By:** Backend API, chunk data, game logic

```gdscript
# Example world tile coordinates
var player_tile = Vector2i(5, 10)
var chunk_origin_tile = Vector2i(-48, -48)
var center_tile = Vector2i(0, 0)
```

**Properties:**
- Origin (0, 0) is the "center" of the game world
- Negative coordinates are valid (west/north of origin)
- Coordinates are discrete (no fractional tiles)

### 2. Chunk Coordinates

**What:** Which 16×16 chunk grid a tile belongs to
**Range:** Chunk indices (e.g., -3 to 3 for 7×7 world)
**Used By:** Chunk loading system, caching

```gdscript
# Tile to chunk conversion
func world_to_chunk(tile_x: int, tile_y: int) -> Vector2i:
    var chunk_x = int(floor(float(tile_x) / 16.0))
    var chunk_y = int(floor(float(tile_y) / 16.0))
    return Vector2i(chunk_x, chunk_y)

# Examples:
# Tile (0, 0) → Chunk (0, 0)
# Tile (16, 16) → Chunk (1, 1)
# Tile (-1, -1) → Chunk (-1, -1)  ⚠️ Note: floor() for negatives
# Tile (-48, -48) → Chunk (-3, -3)
```

**Chunk Key Format:** `"x,y"` string (e.g., `"0,0"`, `"-3,2"`)

### 3. Local Chunk Coordinates

**What:** Tile position within its 16×16 chunk (0-15)
**Range:** 0 to 15 (inclusive)
**Used By:** Chunk data arrays (2D arrays indexed by local coords)

```gdscript
func get_local_coords(tile_x: int, tile_y: int) -> Vector2i:
    var local_x = ((tile_x % 16) + 16) % 16
    var local_y = ((tile_y % 16) % 16
    return Vector2i(local_x, local_y)

# Examples:
# Tile (0, 0) → Local (0, 0) in chunk (0, 0)
# Tile (5, 10) → Local (5, 10) in chunk (0, 0)
# Tile (16, 16) → Local (0, 0) in chunk (1, 1)
# Tile (-1, -1) → Local (15, 15) in chunk (-1, -1)
# Tile (18, 5) → Local (2, 5) in chunk (1, 0)
```

**Why the double modulo?** Handles negative numbers correctly!
```
-1 % 16 = -1  (wrong!)
((-1 % 16) + 16) % 16 = 15  (correct!)
```

### 4. Pixel/Screen Coordinates

**What:** Actual pixel positions on screen
**Range:** Floating-point pixel coordinates
**Used By:** Camera, rendering, mouse input

**CRITICAL:** Pixel coordinates depend on:
- Tile size (128×64 for isometric)
- Tile shape (isometric vs orthogonal)
- TileMap position/rotation/scale

```gdscript
# ✅ CORRECT: Use TileMap conversion functions
var pixel_pos = tilemap.map_to_local(tile_pos)
var tile_pos = tilemap.local_to_map(pixel_pos)

# ❌ WRONG: Manual calculation (doesn't account for isometric)
var pixel_x = tile_x * 128  # Wrong for isometric!
```

---

## Isometric Projection Math

### How Isometric Tiles Are Positioned

**Orthogonal (Square) Tiles:**
```
Tile size: 32×32
Tile (0, 0) → Pixel (0, 0)
Tile (1, 0) → Pixel (32, 0)
Tile (0, 1) → Pixel (0, 32)
Tile (5, 10) → Pixel (160, 320)

Formula: pixel = tile × tile_size
```

**Isometric (Diamond) Tiles:**
```
Tile size: 128×64 (width × height)
Tile (0, 0) → Pixel (0, 0)
Tile (1, 0) → Pixel (64, 32)    // Right diagonal
Tile (0, 1) → Pixel (-64, 32)   // Left diagonal
Tile (1, 1) → Pixel (0, 64)     // Down
Tile (-1, -1) → Pixel (0, -64)  // Up

Formula:
  pixel_x = (tile_x - tile_y) × (tile_width / 2)
  pixel_y = (tile_x + tile_y) × (tile_height / 2)

For 128×64 tiles:
  pixel_x = (tile_x - tile_y) × 64
  pixel_y = (tile_x + tile_y) × 32
```

### Real Examples from Our World

```gdscript
# Island center
Tile (0, 0) → Pixel (0, 0)

# Northwest corner chunk
Tile (-48, -48) → Pixel ((-48 - (-48)) × 64, (-48 + (-48)) × 32)
                → Pixel (0 × 64, -96 × 32)
                → Pixel (0, -3072)  ⚠️

# Wait, console said (-6016, -1504)?
# Let me recalculate...
# Actually: Godot's isometric layout is STACKED, not DIAMOND_RIGHT
# Formula is slightly different!

# Godot STACKED isometric (actual):
Tile (-48, -48):
  pixel_x = -48 × 128 + (-48) × 0 = -6144
  pixel_y = -48 × 0 + (-48) × 64 = -3072

# Hmm, still not matching (-6016, -1504)...
# The exact formula depends on tile_layout setting!
```

**Lesson:** Don't try to calculate pixel positions manually! Use `map_to_local()`!

### Why map_to_local() Is Essential

```gdscript
# ✅ ALWAYS use this for tile → pixel conversion
var pixel_pos = tilemap.map_to_local(Vector2i(-48, -48))
print(pixel_pos)  // Outputs: (-6016.0, -1504.0)

# ✅ ALWAYS use this for pixel → tile conversion
var tile_pos = tilemap.local_to_map(Vector2(-6016.0, -1504.0))
print(tile_pos)  // Outputs: (-48, -48)
```

**These functions account for:**
- Tile shape (isometric, orthogonal, hexagonal)
- Tile size
- Tile layout (stacked, diamond, etc.)
- TileMap transform (position, rotation, scale)

---

## Camera Positioning Rules

### Rule #1: Camera Position Is in Pixel Space

```gdscript
# Camera2D.position is ALWAYS in pixel coordinates
camera.position = Vector2(100, 200)  # Pixel (100, 200), NOT tile (100, 200)!
```

### Rule #2: To Center Camera on a Tile, Convert First

```gdscript
# ❌ WRONG - Camera at pixel (0, 0), not tile (0, 0)
camera.position = Vector2(0, 0)

# ✅ CORRECT - Camera centered on tile (0, 0)
var center_tile = Vector2i(0, 0)
var center_pixel = tilemap.map_to_local(center_tile)
camera.position = center_pixel
```

### Rule #3: Zoom Values Are Multipliers

```gdscript
# Camera2D.zoom is a scale factor
camera.zoom = Vector2(1.0, 1.0)  # 1:1 (default)
camera.zoom = Vector2(2.0, 2.0)  # 2× zoom in (objects appear 2× larger)
camera.zoom = Vector2(0.5, 0.5)  # 2× zoom out (see 2× more area)
```

**Zoom guidelines for isometric tiles:**
```
128×64 tiles:
  - zoom = 0.25: Very far (see full island)
  - zoom = 0.5: Good overview (recommended default)
  - zoom = 1.0: Normal view
  - zoom = 2.0: Very close (tiles fill screen)

32×32 tiles (orthogonal):
  - zoom = 1.0: Far
  - zoom = 2.0: Good overview (recommended default)
  - zoom = 4.0: Normal view
```

### Rule #4: Camera Viewport Is Screen-Relative

The camera shows a rectangular viewport in pixel space:
```
Viewport width in pixels = screen_width / zoom.x
Viewport height in pixels = screen_height / zoom.y

Example:
  Screen: 1920×1080
  Zoom: 0.5
  Viewport: 3840×2160 pixels
```

To see tile (0, 0) on screen:
1. Tile must be within viewport pixel bounds
2. Camera must be positioned near tile's pixel position

---

## Common Pitfalls and Solutions

### Pitfall #1: Confusing Tile and Pixel Coordinates

**Symptom:** Blank screen, objects not visible

```gdscript
# ❌ WRONG - Setting camera to tile coordinates
camera.position = Vector2(0, 0)  # Camera at pixel (0, 0), not tile (0, 0)!

# ❌ WRONG - Passing tile coords to function expecting pixels
var tile_pos = Vector2i(10, 10)
var converted = tilemap.local_to_map(Vector2(tile_pos.x, tile_pos.y))
# local_to_map expects PIXELS, not tiles!

# ✅ CORRECT
var tile_pos = Vector2i(10, 10)
var pixel_pos = tilemap.map_to_local(tile_pos)
camera.position = pixel_pos
```

### Pitfall #2: Using local_to_map() When You Already Have Tiles

**Symptom:** Tiles appear at completely wrong positions

```gdscript
# ❌ WRONG - world_pos is already a tile coordinate!
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String):
    var tile_coords = local_to_map(Vector2(world_pos.x, world_pos.y))
    set_cell(0, tile_coords, source_id, Vector2i(0, 0))
    # local_to_map converts PIXELS to TILES
    # We passed in TILES, treated them as PIXELS, converted back to wrong TILES!

# ✅ CORRECT - world_pos is already in tile coordinates
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String):
    set_cell(0, world_pos, source_id, Vector2i(0, 0))
    # set_cell expects TILE coordinates, which we already have!
```

**What happened when we used `local_to_map()` incorrectly:**
```
Input: world_pos = Vector2i(-48, -48)  (tile coordinates)
Wrong: tile_coords = local_to_map(Vector2(-48, -48))
  ↳ Treats (-48, -48) as PIXELS
  ↳ Converts pixels to tiles: (-48 pixels / 128 = ~0, -48 pixels / 64 = ~-1)
  ↳ Returns tile ~(0, -1) instead of (-48, -48)
  ↳ Tile painted in wrong location!
```

### Pitfall #3: Premature Chunk Tracking

**Symptom:** Only some chunks visible (e.g., 10/49)

```gdscript
# ❌ WRONG - Marks all chunks as "rendered" before painting
func _update_visible_chunks():
    var visible_chunks = _get_visible_chunks()
    current_chunk_keys = visible_chunks  # All 49 marked as done!
    _add_visible_chunks(visible_chunks)  # Only paints 10 (others "already done")

# ✅ CORRECT - Only mark chunks that were actually painted
func _update_visible_chunks():
    var visible_chunks = _get_visible_chunks()
    var newly_painted = _add_visible_chunks(visible_chunks)

    for chunk_key in newly_painted:
        if not current_chunk_keys.has(chunk_key):
            current_chunk_keys.append(chunk_key)
```

**Why this happens:**
1. Load 49 chunks in 5 batches (10+10+10+10+9)
2. After batch 1 loads: `_update_visible_chunks()` called
3. `visible_chunks` = all 49 (we want to show all)
4. `current_chunk_keys = visible_chunks` marks all 49 as "rendered"
5. Batches 2-5 load but skip painting (already in `current_chunk_keys`)

### Pitfall #4: Forgetting Layer Configuration

**Symptom:** TileMap has cells but nothing renders

```gdscript
# ❌ WRONG - Layer 0 might not exist or be disabled
set_cell(0, tile_pos, source_id, Vector2i(0, 0))  # Layer 0 not configured!

# ✅ CORRECT - Ensure layer exists and is enabled
func _ready():
    if get_layers_count() == 0:
        add_layer(-1)
    set_layer_enabled(0, true)
    set_layer_modulate(0, Color(1, 1, 1, 1))
```

### Pitfall #5: Wrong Zoom for Tile Size

**Symptom:** Tiles too small/large, or not visible at all

```gdscript
# For 128×64 isometric tiles
camera.zoom = Vector2(2.0, 2.0)  # Too close! Tiles are huge!
camera.zoom = Vector2(0.1, 0.1)  # Too far! Tiles are tiny pixels!

# ✅ Good starting zoom for 128×64 tiles
camera.zoom = Vector2(0.5, 0.5)  # Can see ~30×30 tiles on screen
```

---

## Debugging Checklist

When your isometric world isn't visible:

### Step 1: Verify Chunks Loaded

```gdscript
print("Cached chunks: ", WorldDataCache.terrain_cache.size())
# Should be > 0

print("Chunk keys: ", WorldDataCache.terrain_cache.keys())
# Should show ["0,0", "1,0", "-1,0", ...]

var chunk_data = WorldDataCache.get_terrain_chunk("0,0")
print("Chunk 0,0 rows: ", chunk_data.size())
# Should be 16
```

### Step 2: Verify Tiles Painted

```gdscript
print("Total cells: ", tilemap.get_used_cells(0).size())
# Should be > 0 (ideally 12544 for 49 chunks)

print("TileSet exists: ", tilemap.tile_set != null)
# Should be true

print("Layer 0 enabled: ", tilemap.is_layer_enabled(0))
# Should be true
```

### Step 3: Check Tile Positions

```gdscript
var cells = tilemap.get_used_cells(0).slice(0, 5)
print("Sample cells: ", cells)
# Should show tile coordinates like [(-48, -48), (-47, -48), ...]

for cell_pos in cells:
    var pixel_pos = tilemap.map_to_local(cell_pos)
    print("Tile ", cell_pos, " → Pixel ", pixel_pos)
# Verify pixel positions are reasonable
```

### Step 4: Check Camera Position

```gdscript
print("Camera position: ", camera.position)
print("Camera zoom: ", camera.zoom)

# Check if camera can see tile (0, 0)
var tile_center_pixel = tilemap.map_to_local(Vector2i(0, 0))
print("Tile (0,0) at pixel: ", tile_center_pixel)

var distance = camera.position.distance_to(tile_center_pixel)
print("Camera distance from tile (0,0): ", distance, " pixels")
# If > 1000 pixels, camera probably can't see it
```

### Step 5: Check Viewport Bounds

```gdscript
var viewport_size = get_viewport_rect().size
print("Viewport size: ", viewport_size)

var viewport_world_size = viewport_size / camera.zoom
print("Viewport world size: ", viewport_world_size, " pixels")

# Can camera see tile (0, 0)?
var tile_pixel = tilemap.map_to_local(Vector2i(0, 0))
var camera_min = camera.position - viewport_world_size / 2
var camera_max = camera.position + viewport_world_size / 2

print("Camera viewport: ", camera_min, " to ", camera_max)
print("Tile (0,0) at: ", tile_pixel)

if tile_pixel.x >= camera_min.x and tile_pixel.x <= camera_max.x and \
   tile_pixel.y >= camera_min.y and tile_pixel.y <= camera_max.y:
    print("✅ Tile (0,0) is in viewport!")
else:
    print("❌ Tile (0,0) is outside viewport!")
```

---

## Code Examples

### Example 1: Correct Camera Setup

```gdscript
# WorldRenderer.gd
@onready var terrain_tilemap: TileMap = $TerrainTileMap
@onready var camera: Camera2D = $TerrainTileMap/Camera2D

func _ready():
    # Initialize camera to center on tile (0, 0)
    var center_tile = Vector2i(0, 0)
    var center_pixel = terrain_tilemap.map_to_local(center_tile)

    camera.position = center_pixel
    camera.zoom = Vector2(0.5, 0.5)  # Good for 128×64 tiles

    print("📹 Camera positioned at tile ", center_tile,
          " = pixel ", center_pixel, " with zoom ", camera.zoom)
```

### Example 2: Correct Tile Painting

```gdscript
# TerrainTileMap.gd
func paint_chunk(chunk_key: String, terrain_data: Array):
    var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

    for y in range(terrain_data.size()):
        var row = terrain_data[y]
        for x in range(row.size()):
            var terrain_type = row[x]
            if terrain_type == "":
                continue

            # Calculate world tile position
            var world_pos = Vector2i(
                chunk_origin.x + x,
                chunk_origin.y + y
            )

            # Paint tile (no coordinate conversion needed!)
            paint_terrain_tile(world_pos, terrain_type)

func paint_terrain_tile(world_pos: Vector2i, terrain_type: String):
    # world_pos is already in tile coordinates!
    var terrain_color = Config.terrain_colors.get(terrain_type, Color.WHITE)
    var source_id = _get_or_create_terrain_source(terrain_type, terrain_color)

    # set_cell expects tile coordinates - use world_pos directly
    set_cell(0, world_pos, source_id, Vector2i(0, 0))
```

### Example 3: Correct Chunk Tracking

```gdscript
# WorldRenderer.gd
var current_chunk_keys: Array[String] = []

func _update_visible_chunks():
    var visible_chunks = _get_visible_chunks()

    # Paint new chunks and get list of actually painted chunks
    var newly_painted = _add_visible_chunks(visible_chunks)

    # Only mark chunks as "current" if they were actually painted
    for chunk_key in newly_painted:
        if not current_chunk_keys.has(chunk_key):
            current_chunk_keys.append(chunk_key)

    print("📊 Rendered: ", current_chunk_keys.size(),
          " / ", visible_chunks.size(), " visible")

func _add_visible_chunks(visible_chunks: Array[String]) -> Array[String]:
    var painted_chunks: Array[String] = []

    for chunk_key in visible_chunks:
        # Skip if already painted
        if current_chunk_keys.has(chunk_key):
            continue

        # Get chunk data from cache
        var terrain_data = WorldDataCache.get_terrain_chunk(chunk_key)
        if terrain_data.size() > 0:
            terrain_tilemap.paint_chunk(chunk_key, terrain_data)
            painted_chunks.append(chunk_key)

    return painted_chunks
```

### Example 4: Mouse Click to Tile Conversion

```gdscript
func _input(event):
    if event is InputEventMouseButton and event.pressed:
        # Mouse click position in screen space
        var mouse_screen = event.position

        # Convert screen to world pixel coordinates
        var mouse_world_pixel = camera.get_global_mouse_position()

        # Convert pixel to tile coordinates
        var clicked_tile = terrain_tilemap.local_to_map(mouse_world_pixel)

        print("Clicked tile: ", clicked_tile)
        print("  Screen pos: ", mouse_screen)
        print("  World pixel: ", mouse_world_pixel)
```

### Example 5: Pan Camera to Tile

```gdscript
func pan_camera_to_tile(target_tile: Vector2i, duration: float = 0.5):
    # Convert target tile to pixel position
    var target_pixel = terrain_tilemap.map_to_local(target_tile)

    # Animate camera to target
    var tween = create_tween()
    tween.tween_property(camera, "position", target_pixel, duration)

    print("Panning camera to tile ", target_tile, " (pixel ", target_pixel, ")")
```

---

## Summary: The Golden Rules

1. **Camera position is in PIXEL space** - always convert tiles to pixels first
2. **set_cell() expects TILE coordinates** - never convert tiles before calling it
3. **Use map_to_local() for tile → pixel** - don't calculate manually
4. **Use local_to_map() for pixel → tile** - for mouse clicks, not for painting
5. **Only mark chunks as rendered AFTER painting** - avoid premature tracking
6. **Configure TileMap layers explicitly** - Godot 4.x requires it
7. **Zoom affects visible area** - larger tiles need smaller zoom values

**When in doubt:** Let Godot's TileMap do the coordinate conversion. Trust `map_to_local()` and `local_to_map()`.

---

## Additional Resources

- [Godot TileMap Documentation](https://docs.godotengine.org/en/stable/classes/class_tilemap.html)
- [Godot Isometric Tutorial](https://docs.godotengine.org/en/stable/tutorials/2d/using_tilemaps.html#isometric-tiles)
- Main Project: `/Users/jean/Github/life-simulator/CLAUDE.md`
- Web Viewer: `/Users/jean/Github/life-simulator/web-viewer/js/renderer.js`

---

**Debugging Log: 2025-01-11**
- Symptom: Blank screen in Godot viewer
- Root Cause: Camera at pixel (0, 0), tiles at pixel (-6016, -1504)
- Solution: Position camera using `map_to_local(Vector2i(0, 0))`
- Result: Full island visible with 12,544 tiles rendered correctly
