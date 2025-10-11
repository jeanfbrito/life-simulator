# Godot Viewer - Remaining Tasks

**Last Updated:** 2025-01-11
**Status:** Phase 2 Complete, Phases 3-6 Remaining

## Executive Summary

**✅ Completed:**
- Phase 0: Foundations (workspace, config, audits)
- Phase 1: Data Pipeline (HTTP client, chunk manager, cache)
- Phase 2: Isometric Terrain Rendering (TileSet, TileMap, camera positioning)
- **BONUS:** Fixed critical camera positioning and coordinate bugs (2025-01-11)

**🔄 Current State:**
- All 49 chunks loading successfully (12,544 tiles)
- Isometric terrain rendering working perfectly
- Camera centered on tile (0,0) with proper zoom
- Complete backend integration operational

**❌ Remaining Work:**
- Phase 3: Dynamic Chunk Streaming (polish camera, lifecycle management)
- Phase 4: Resources and Entities (visual overlays, emoji sprites)
- Phase 5: UI & Statistics (HUD, controls panel)
- Phase 6: Validation & Polish (screenshots, docs, CI)

---

## Phase 3: Dynamic Chunk Streaming (Polish)

### Status: 60% Complete
**What's Done:** Camera controls, chunk loading, basic streaming
**What's Left:** Smooth camera movement, chunk lifecycle optimization

---

### Task 3.1: Enhanced Camera Controls ⚡ HIGH PRIORITY

**Goal:** Make camera movement smooth and intuitive like the web viewer

**Current State:**
- ✅ Arrow keys move camera
- ✅ +/- keys zoom
- ✅ Camera centers on tile (0,0)
- ❌ Movement is instant (no smoothing)
- ❌ No mouse drag support
- ❌ No momentum/inertia

**Sub-Tasks:**

#### 3.1.1: Implement Smooth Camera Panning
**File:** `godot-viewer/scripts/WorldRenderer.gd`

```gdscript
# Add smooth camera movement instead of instant jumps
var target_camera_position: Vector2 = Vector2.ZERO
var camera_velocity: Vector2 = Vector2.ZERO
const CAMERA_ACCELERATION = 2000.0  # pixels/sec²
const CAMERA_MAX_SPEED = 1000.0     # pixels/sec
const CAMERA_FRICTION = 0.9         # 0-1 damping

func _process(delta):
    # Smoothly interpolate camera to target position
    var direction = (target_camera_position - camera.position).normalized()
    var distance = camera.position.distance_to(target_camera_position)

    if distance > 1.0:  # Not at target yet
        camera_velocity += direction * CAMERA_ACCELERATION * delta
        camera_velocity = camera_velocity.limit_length(CAMERA_MAX_SPEED)
    else:
        camera_velocity *= CAMERA_FRICTION

    camera.position += camera_velocity * delta
```

**Verification:**
- [ ] Camera movement feels smooth, not jerky
- [ ] Arrow keys move camera at consistent speed
- [ ] Camera stops smoothly (no overshoot)

---

#### 3.1.2: Add Mouse Drag Panning
**File:** `godot-viewer/scripts/WorldRenderer.gd`

```gdscript
var dragging: bool = false
var drag_start_pos: Vector2 = Vector2.ZERO
var drag_start_camera: Vector2 = Vector2.ZERO

func _input(event):
    if event is InputEventMouseButton:
        if event.button_index == MOUSE_BUTTON_LEFT:
            if event.pressed:
                # Start drag
                dragging = true
                drag_start_pos = event.position
                drag_start_camera = camera.position
            else:
                # End drag
                dragging = false

    elif event is InputEventMouseMotion and dragging:
        # Calculate drag delta in world space
        var drag_delta = (event.position - drag_start_pos) / camera.zoom.x
        target_camera_position = drag_start_camera - drag_delta
```

**Verification:**
- [ ] Click and drag moves map smoothly
- [ ] Drag direction matches mouse movement
- [ ] Release stops dragging immediately
- [ ] Works at different zoom levels

---

#### 3.1.3: Add Mouse Wheel Zoom
**File:** `godot-viewer/scripts/WorldRenderer.gd`

```gdscript
func _input(event):
    if event is InputEventMouseButton:
        if event.button_index == MOUSE_BUTTON_WHEEL_UP:
            zoom_at_point(event.position, 1.25)  # Zoom in
        elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
            zoom_at_point(event.position, 0.8)   # Zoom out

func zoom_at_point(screen_point: Vector2, factor: float):
    # Get world position at mouse
    var world_pos_before = camera.get_global_mouse_position()

    # Apply zoom
    var new_zoom = camera.zoom * factor
    new_zoom.x = clamp(new_zoom.x, Config.min_zoom, Config.max_zoom)
    new_zoom.y = clamp(new_zoom.y, Config.min_zoom, Config.max_zoom)
    camera.zoom = new_zoom

    # Adjust camera to keep mouse position constant
    var world_pos_after = camera.get_global_mouse_position()
    camera.position += world_pos_before - world_pos_after
```

**Verification:**
- [ ] Mouse wheel zooms smoothly
- [ ] Zoom centers on mouse cursor position
- [ ] Zoom respects min/max limits (0.25x - 4.0x)
- [ ] Zoom works while dragging

---

### Task 3.2: Chunk Lifecycle Management 🔄 MEDIUM PRIORITY

**Goal:** Optimize memory usage and performance by unloading distant chunks

**Current State:**
- ✅ Chunks load within radius
- ❌ No chunk unloading (memory leak)
- ❌ No distance-based culling
- ❌ Cache grows unbounded

**Sub-Tasks:**

#### 3.2.1: Implement Chunk Unloading
**File:** `godot-viewer/scripts/WorldRenderer.gd`

```gdscript
const MAX_LOADED_CHUNKS = 100  # Configurable limit
const UNLOAD_DISTANCE = 10     # Chunks beyond this radius are unloaded

func _update_visible_chunks():
    var visible_chunks = _get_visible_chunks()

    # NEW: Check if we need to unload distant chunks
    if current_chunk_keys.size() > MAX_LOADED_CHUNKS:
        _unload_distant_chunks(visible_chunks)

    # Existing painting logic...
    var newly_painted = _add_visible_chunks(visible_chunks)

    for chunk_key in newly_painted:
        if not current_chunk_keys.has(chunk_key):
            current_chunk_keys.append(chunk_key)

func _unload_distant_chunks(keep_chunks: Array[String]):
    var center = _world_to_chunk(camera.position)
    var to_unload: Array[String] = []

    for chunk_key in current_chunk_keys:
        if not keep_chunks.has(chunk_key):
            var parts = chunk_key.split(",")
            var chunk_pos = Vector2i(int(parts[0]), int(parts[1]))
            var distance = center.distance_to(chunk_pos)

            if distance > UNLOAD_DISTANCE:
                to_unload.append(chunk_key)

    for chunk_key in to_unload:
        terrain_tilemap.clear_chunk(chunk_key)
        current_chunk_keys.erase(chunk_key)
        # Optionally clear from cache too:
        # WorldDataCache.clear_chunk(chunk_key)

    if to_unload.size() > 0:
        print("🗑️ Unloaded ", to_unload.size(), " distant chunks")
```

**Verification:**
- [ ] Memory usage stable when moving long distances
- [ ] Chunks unload when > 10 chunks away
- [ ] Performance remains smooth during unloading
- [ ] Revisiting areas re-loads chunks correctly

---

#### 3.2.2: Add Performance Monitoring
**File:** `godot-viewer/scripts/WorldRenderer.gd`

```gdscript
var performance_timer: Timer
var last_fps: float = 0.0
var last_memory: int = 0

func _ready():
    # ... existing setup ...

    # Add performance monitoring
    performance_timer = Timer.new()
    performance_timer.wait_time = 2.0
    performance_timer.timeout.connect(_print_performance)
    add_child(performance_timer)
    performance_timer.start()

func _print_performance():
    var fps = Engine.get_frames_per_second()
    var memory = Performance.get_monitor(Performance.MEMORY_STATIC)
    var chunks = current_chunk_keys.size()
    var cells = terrain_tilemap.get_used_cells(0).size()

    print("📊 Performance: FPS=%d | Chunks=%d | Cells=%d | Memory=%.1f MB"
          % [fps, chunks, cells, memory / 1024.0 / 1024.0])
```

**Verification:**
- [ ] Performance logs appear every 2 seconds
- [ ] FPS stays above 30 (ideally 60)
- [ ] Memory usage reported accurately
- [ ] Can track performance degradation over time

---

## Phase 4: Resources and Entities

### Status: 0% Complete
**What's Done:** Nothing (backend data loads but doesn't render)
**What's Left:** Everything - this is the most visible feature!

---

### Task 4.1: Resource Overlay Rendering ⚡⚡ CRITICAL PRIORITY

**Goal:** Show tree, bush, flower, rock emojis on top of terrain tiles

**Current State:**
- ✅ Resource data loads from backend (trees, bushes, flowers, rocks)
- ✅ Resource coordinates known (local chunk coordinates)
- ❌ No visual representation (invisible resources)
- ❌ No emoji rendering

**Sub-Tasks:**

#### 4.1.1: Create ResourceManager System
**File:** `godot-viewer/scripts/ResourceManager.gd`

```gdscript
extends Node2D

# Resource sprite pool for reuse
var resource_sprites: Dictionary = {}  # chunk_key -> Array[Label]

func paint_resources(chunk_key: String, resource_data: Array):
    # Clear existing resources for this chunk
    if resource_sprites.has(chunk_key):
        for sprite in resource_sprites[chunk_key]:
            sprite.queue_free()
        resource_sprites.erase(chunk_key)

    var sprites: Array[Label] = []
    var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

    # Iterate through 16x16 resource grid
    for y in range(resource_data.size()):
        var row = resource_data[y]
        for x in range(row.size()):
            var resource_type = row[x]
            if resource_type == "":
                continue

            # Create emoji label for this resource
            var label = Label.new()
            label.text = Config.get_resource_symbol(resource_type)

            # Get resource config for sizing and offset
            var config = Config.get_resource_config(resource_type)
            label.add_theme_font_size_override("font_size",
                int(Config.TILE_SIZE * config.size_multiplier))

            # Position at tile location (convert to pixel space)
            var tile_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
            var pixel_pos = get_parent().map_to_local(tile_pos)

            # Apply resource offset
            pixel_pos.x += Config.TILE_SIZE * config.offset_x
            pixel_pos.y += Config.TILE_SIZE * config.offset_y

            label.position = pixel_pos
            label.z_index = 1  # Above terrain

            add_child(label)
            sprites.append(label)

    resource_sprites[chunk_key] = sprites
    print("🌳 Rendered ", sprites.size(), " resources for chunk ", chunk_key)

func clear_resources(chunk_key: String):
    if resource_sprites.has(chunk_key):
        for sprite in resource_sprites[chunk_key]:
            sprite.queue_free()
        resource_sprites.erase(chunk_key)
```

**Add to Scene:**
1. Open `godot-viewer/scenes/World.tscn`
2. Add `ResourceManager` node as child of `TerrainTileMap`
3. Attach `scripts/ResourceManager.gd` script

**Integration with WorldRenderer:**
```gdscript
# In WorldRenderer.gd
@onready var resource_manager: Node2D = $TerrainTileMap/ResourceManager

func _add_visible_chunks(visible_chunks: Array[String]) -> Array[String]:
    var painted_chunks: Array[String] = []

    for chunk_key in visible_chunks:
        var terrain_data = WorldDataCache.get_terrain_chunk(chunk_key)
        if terrain_data.size() > 0 and not current_chunk_keys.has(chunk_key):
            terrain_tilemap.paint_chunk(chunk_key, terrain_data)

            # NEW: Paint resources too!
            var resource_data = WorldDataCache.get_resource_chunk(chunk_key)
            if resource_data.size() > 0:
                resource_manager.paint_resources(chunk_key, resource_data)

            painted_chunks.append(chunk_key)

    return painted_chunks
```

**Verification:**
- [ ] Trees (🌳) appear on forest tiles
- [ ] Bushes appear in correct positions
- [ ] Flowers render at appropriate size
- [ ] Rocks visible on terrain
- [ ] Resources match web viewer positions exactly
- [ ] Resources unload when chunks unload

---

#### 4.1.2: Implement Y-Sorting for Resources
**File:** `godot-viewer/scripts/ResourceManager.gd`

**Problem:** Resources need proper depth sorting so trees behind hills appear behind them.

**Solution:**
```gdscript
# Change Label to Node2D with Label child for Y-sorting
func paint_resources(chunk_key: String, resource_data: Array):
    # ... existing code ...

    for y in range(resource_data.size()):
        for x in range(row.size()):
            # ... existing code ...

            # Create container for Y-sorting
            var container = Node2D.new()
            container.y_sort_enabled = true

            var label = Label.new()
            label.text = Config.get_resource_symbol(resource_type)
            # ... configure label ...

            container.add_child(label)
            container.position = pixel_pos
            container.z_index = 1

            # Y-sort uses Y position for depth
            container.y_sort_origin = pixel_pos.y

            add_child(container)
            sprites.append(container)
```

**Verification:**
- [ ] Trees further north appear behind trees further south
- [ ] Resources don't overlap incorrectly
- [ ] Depth sorting matches web viewer

---

### Task 4.2: Entity Rendering ⚡⚡ CRITICAL PRIORITY

**Goal:** Show rabbits (🐇), humans (🧍‍♂️), and other entities moving around

**Current State:**
- ✅ Entity API endpoint exists (`/api/entities`)
- ✅ Entity polling implemented in web viewer (200ms)
- ❌ No entity fetching in Godot viewer
- ❌ No entity sprites/labels

**Sub-Tasks:**

#### 4.2.1: Create EntityManager System
**File:** `godot-viewer/scripts/EntityManager.gd`

```gdscript
extends Node2D

# Entity tracking
var entities: Dictionary = {}  # entity_id -> Label node
var entity_poll_timer: Timer

signal entities_updated(entity_list)

func _ready():
    # Set up polling timer (200ms like web viewer)
    entity_poll_timer = Timer.new()
    entity_poll_timer.wait_time = 0.2
    entity_poll_timer.timeout.connect(_poll_entities)
    add_child(entity_poll_timer)
    entity_poll_timer.start()

func _poll_entities():
    # Fetch entities from API
    var http = HTTPRequest.new()
    add_child(http)

    var error = http.request(Config.api_base_url + "/api/entities")
    if error != OK:
        return

    var result = await http.request_completed
    http.queue_free()

    if result[0] != HTTPRequest.RESULT_SUCCESS or result[1] != 200:
        return

    var json = JSON.new()
    if json.parse(result[3].get_string_from_utf8()) != OK:
        return

    var data = json.data
    if data.has("entities"):
        _update_entities(data.entities)

func _update_entities(entity_list: Array):
    var seen_ids = {}

    for entity_data in entity_list:
        var entity_id = entity_data.id
        seen_ids[entity_id] = true

        if not entities.has(entity_id):
            # Create new entity sprite
            _create_entity(entity_id, entity_data)
        else:
            # Update existing entity
            _update_entity_position(entity_id, entity_data)

    # Remove entities that no longer exist
    for entity_id in entities.keys():
        if not seen_ids.has(entity_id):
            entities[entity_id].queue_free()
            entities.erase(entity_id)

func _create_entity(entity_id: int, data: Dictionary):
    var label = Label.new()

    # Get entity config from API or use default
    var entity_type = data.get("entity_type", "default")
    var config = Config.get_entity_config(entity_type)

    label.text = config.emoji
    label.add_theme_font_size_override("font_size",
        int(Config.TILE_SIZE * config.size_multiplier))

    # Position entity (with -0.2 Y offset to keep feet in grid!)
    var pos = data.position
    var tile_pos = Vector2i(pos.x, pos.y)
    var pixel_pos = get_parent().map_to_local(tile_pos)
    pixel_pos.y += Config.TILE_SIZE * config.offset_y  # Apply -0.2 offset

    label.position = pixel_pos
    label.z_index = 2  # Above resources

    add_child(label)
    entities[entity_id] = label

func _update_entity_position(entity_id: int, data: Dictionary):
    var label = entities[entity_id]

    # Update position (discrete jumps like simulation)
    var pos = data.position
    var tile_pos = Vector2i(pos.x, pos.y)
    var pixel_pos = get_parent().map_to_local(tile_pos)

    var config = Config.get_entity_config(data.get("entity_type", "default"))
    pixel_pos.y += Config.TILE_SIZE * config.offset_y

    label.position = pixel_pos
```

**Add to Scene:**
1. Open `godot-viewer/scenes/World.tscn`
2. Add `EntityManager` node as child of `TerrainTileMap`
3. Attach `scripts/EntityManager.gd` script

**Verification:**
- [ ] Entities appear at correct positions
- [ ] Entities move when simulation updates (200ms polling)
- [ ] Entity emojis match web viewer (🐇 for rabbits, 🧍‍♂️ for humans)
- [ ] Entities have -0.2 Y offset (feet in grid)
- [ ] Entity scaling correct for juveniles

---

#### 4.2.2: Add Action Labels for Entities
**File:** `godot-viewer/scripts/EntityManager.gd`

```gdscript
func _create_entity(entity_id: int, data: Dictionary):
    # ... existing code ...

    # Add action label if present
    if data.has("current_action") and Config.TILE_SIZE >= 8:
        var action_label = Label.new()
        action_label.text = data.current_action
        action_label.add_theme_font_size_override("font_size",
            max(8, int(Config.TILE_SIZE * 0.5)))

        # Position above entity
        action_label.position = Vector2(0, -Config.TILE_SIZE * 0.6)
        action_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER

        # Dark background for readability
        var panel = Panel.new()
        panel.add_child(action_label)
        label.add_child(panel)
```

**Verification:**
- [ ] Action labels appear above entities
- [ ] Labels show current action (e.g., "Drinking", "Wandering")
- [ ] Labels only show when zoomed in enough
- [ ] Labels update when action changes

---

## Phase 5: UI & Statistics

### Status: 0% Complete
**What's Done:** Nothing
**What's Left:** HUD panel, control buttons, statistics display

---

### Task 5.1: Statistics HUD 🔄 MEDIUM PRIORITY

**Goal:** Show terrain statistics matching web viewer

**Sub-Tasks:**

#### 5.1.1: Create HUD Scene
**File:** `godot-viewer/scenes/HUD.tscn`

Create a `CanvasLayer` with:
- VBoxContainer for stats panel (top-right corner)
- Labels for:
  - Total tiles
  - Walkable percentage
  - Water percentage
  - Forest percentage
  - Resource count
  - Entity count
  - FPS counter

**File:** `godot-viewer/scripts/HUD.gd`

```gdscript
extends CanvasLayer

@onready var total_tiles_label = $Panel/VBox/TotalTiles
@onready var walkable_label = $Panel/VBox/Walkable
@onready var water_label = $Panel/VBox/Water
@onready var forest_label = $Panel/VBox/Forest
@onready var resource_label = $Panel/VBox/Resources
@onready var entity_label = $Panel/VBox/Entities
@onready var fps_label = $Panel/VBox/FPS

func _process(_delta):
    fps_label.text = "FPS: %d" % Engine.get_frames_per_second()

func update_stats(stats: Dictionary):
    total_tiles_label.text = "Tiles: %d" % stats.get("total_tiles", 0)
    walkable_label.text = "Walkable: %d%%" % stats.get("walkable_percent", 0)
    water_label.text = "Water: %d%%" % stats.get("water_percent", 0)
    forest_label.text = "Forest: %d%%" % stats.get("forest_percent", 0)
    resource_label.text = "Resources: %d" % stats.get("resource_count", 0)
    entity_label.text = "Entities: %d" % stats.get("entity_count", 0)
```

**Integration:**
```gdscript
# In WorldRenderer.gd
@onready var hud = $HUD

func _update_visible_chunks():
    # ... existing code ...

    # Calculate stats from painted chunks
    var stats = _calculate_terrain_stats()
    hud.update_stats(stats)

func _calculate_terrain_stats() -> Dictionary:
    var total = 0
    var walkable = 0
    var water = 0
    var forest = 0

    for chunk_key in current_chunk_keys:
        var terrain_data = WorldDataCache.get_terrain_chunk(chunk_key)
        for row in terrain_data:
            for tile in row:
                total += 1
                if tile == "Water" or tile == "DeepWater" or tile == "ShallowWater":
                    water += 1
                elif tile == "Forest":
                    forest += 1
                    walkable += 1
                elif tile != "Mountain":
                    walkable += 1

    return {
        "total_tiles": total,
        "walkable_percent": int(float(walkable) / total * 100) if total > 0 else 0,
        "water_percent": int(float(water) / total * 100) if total > 0 else 0,
        "forest_percent": int(float(forest) / total * 100) if total > 0 else 0,
        "resource_count": _count_resources(),
        "entity_count": $TerrainTileMap/EntityManager.entities.size()
    }
```

**Verification:**
- [ ] HUD appears in top-right corner
- [ ] Statistics update when chunks load
- [ ] Numbers match web viewer (within rounding)
- [ ] FPS counter updates in real-time

---

### Task 5.2: Controls Panel 🔄 LOW PRIORITY

**Goal:** Add UI buttons for common actions

**Sub-Tasks:**

#### 5.2.1: Create Control Buttons
Add to HUD scene:
- Reset view button
- Zoom in/out buttons
- Toggle grass density button (if implementing Task 2.3)
- Reload world button

```gdscript
# In HUD.gd
signal reset_view_pressed
signal zoom_in_pressed
signal zoom_out_pressed
signal reload_world_pressed

func _on_reset_view_pressed():
    reset_view_pressed.emit()

func _on_zoom_in_pressed():
    zoom_in_pressed.emit()

func _on_zoom_out_pressed():
    zoom_out_pressed.emit()

func _on_reload_world_pressed():
    reload_world_pressed.emit()
```

**Integration:**
```gdscript
# In WorldRenderer.gd
func _ready():
    # ... existing code ...

    hud.reset_view_pressed.connect(_reset_view)
    hud.zoom_in_pressed.connect(_zoom_in)
    hud.zoom_out_pressed.connect(_zoom_out)
    hud.reload_world_pressed.connect(_reload_world)

func _reset_view():
    camera.position = terrain_tilemap.map_to_local(Vector2i(0, 0))
    camera.zoom = Vector2(0.5, 0.5)

func _zoom_in():
    camera.zoom *= 1.25
    camera.zoom = camera.zoom.clamp(Vector2(Config.min_zoom, Config.min_zoom),
                                     Vector2(Config.max_zoom, Config.max_zoom))

func _zoom_out():
    camera.zoom *= 0.8
    camera.zoom = camera.zoom.clamp(Vector2(Config.min_zoom, Config.min_zoom),
                                     Vector2(Config.max_zoom, Config.max_zoom))

func _reload_world():
    # Clear everything and reload
    for chunk_key in current_chunk_keys:
        terrain_tilemap.clear_chunk(chunk_key)
    current_chunk_keys.clear()
    WorldDataCache.clear_cache()
    ChunkManager.clear()
    start_world_loading()
```

**Verification:**
- [ ] Buttons clickable and responsive
- [ ] Reset view returns to origin
- [ ] Zoom buttons work correctly
- [ ] Reload clears and reloads world

---

## Phase 6: Validation & Polish

### Status: 0% Complete
**What's Done:** Nothing
**What's Left:** Testing, screenshots, documentation

---

### Task 6.1: Side-by-Side Comparison 📸 HIGH PRIORITY

**Goal:** Verify visual parity with web viewer

**Sub-Tasks:**

#### 6.1.1: Create Test Worlds with Known Seeds
```bash
# Generate 3 test worlds
cargo run --bin map_generator -- --name "test_seed_12345" --seed 12345
cargo run --bin map_generator -- --name "test_seed_99999" --seed 99999
cargo run --bin map_generator -- --name "test_seed_42" --seed 42
```

#### 6.1.2: Capture Screenshots
For each test world:
1. Open web viewer at `http://localhost:54321/viewer.html`
2. Open Godot viewer
3. Position both cameras at same location (e.g., tile 0,0)
4. Set same zoom level (e.g., 1.0x)
5. Screenshot both views

**Comparison Points:**
- [ ] Terrain colors match
- [ ] Resource positions identical
- [ ] Entity positions identical
- [ ] Statistics match (within rounding)
- [ ] Chunk boundaries align

**Create Comparison Document:**
**File:** `godot-viewer/docs/VISUAL_COMPARISON.md`

```markdown
# Visual Comparison: Web Viewer vs Godot Viewer

## Test Seed 12345
### Web Viewer
![Web Viewer - Seed 12345](screenshots/web_seed_12345.png)

### Godot Viewer
![Godot Viewer - Seed 12345](screenshots/godot_seed_12345.png)

### Analysis
- Terrain: ✅ Match
- Resources: ✅ Match
- Entities: ✅ Match
- Statistics: ✅ Match (within 1%)

## Test Seed 99999
...

## Summary
- ✅ All visual elements match
- ✅ Statistics within acceptable tolerance
- ⚠️ Known differences: [list any]
```

**Verification:**
- [ ] 3 test worlds documented
- [ ] Screenshots captured and committed
- [ ] Comparison analysis complete
- [ ] Stakeholder approval obtained

---

### Task 6.2: Documentation & CI 📚 HIGH PRIORITY

**Goal:** Ensure project is maintainable and testable

**Sub-Tasks:**

#### 6.2.1: Update Documentation

**File:** `godot-viewer/README.md` (update)
Add sections:
- How to run the viewer
- Keyboard controls
- Known limitations
- Troubleshooting guide

**File:** `godot-viewer/docs/ARCHITECTURE.md` (create)
Document:
- Component relationships
- Data flow diagrams
- Coordinate system explanations
- Extension points

**File:** Update main `CLAUDE.md`
Add Godot viewer section with:
- Quick start guide
- Common pitfalls
- Links to detailed docs

#### 6.2.2: Create CI Smoke Test

**File:** `.github/workflows/godot-viewer-test.yml`

```yaml
name: Godot Viewer Smoke Test

on:
  push:
    paths:
      - 'godot-viewer/**'
  pull_request:
    paths:
      - 'godot-viewer/**'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Godot
        uses: chickensoft-games/setup-godot@v1
        with:
          version: 4.5.0

      - name: Verify Project Loads
        run: |
          cd godot-viewer
          godot --headless --path . --quit-after 1 2>&1 | tee test.log
          grep -q "WorldRenderer initialized" test.log

      - name: Run Unit Tests
        run: |
          cd godot-viewer
          godot --headless --path . --script test_runner.gd
```

**Verification:**
- [ ] CI job runs successfully
- [ ] Smoke test passes on every commit
- [ ] Test logs preserved as artifacts
- [ ] README merged and reviewed

---

## Summary Checklist

### Phase 3: Dynamic Chunk Streaming
- [ ] 3.1.1: Smooth camera panning
- [ ] 3.1.2: Mouse drag support
- [ ] 3.1.3: Mouse wheel zoom
- [ ] 3.2.1: Chunk unloading
- [ ] 3.2.2: Performance monitoring

### Phase 4: Resources and Entities (MOST IMPORTANT!)
- [ ] 4.1.1: ResourceManager system
- [ ] 4.1.2: Y-sorting for resources
- [ ] 4.2.1: EntityManager system
- [ ] 4.2.2: Action labels

### Phase 5: UI & Statistics
- [ ] 5.1.1: Statistics HUD
- [ ] 5.2.1: Control buttons

### Phase 6: Validation & Polish
- [ ] 6.1.1: Test worlds created
- [ ] 6.1.2: Screenshots and comparison
- [ ] 6.2.1: Documentation updated
- [ ] 6.2.2: CI smoke test

---

## Priority Ranking

**🔥 CRITICAL (Do First):**
1. Task 4.1: Resource Overlay Rendering
2. Task 4.2: Entity Rendering
3. Task 6.1: Visual Comparison

**⚡ HIGH (Do Soon):**
1. Task 3.1: Enhanced Camera Controls
2. Task 5.1: Statistics HUD
3. Task 6.2: Documentation

**🔄 MEDIUM (Nice to Have):**
1. Task 3.2: Chunk Lifecycle Management
2. Task 5.2: Controls Panel

**💤 LOW (Optional):**
1. Task 2.3: Grass Density Overlay (from original plan)

---

## Estimated Completion Time

- **Phase 3 Polish:** 3-4 hours
- **Phase 4 Resources & Entities:** 6-8 hours ⭐ MOST VISIBLE
- **Phase 5 UI:** 2-3 hours
- **Phase 6 Validation:** 2-3 hours

**Total:** ~15-20 hours of focused development

---

## Notes

- Camera positioning bugs from 2025-01-11 are FIXED ✅
- All backend integration working perfectly ✅
- Terrain rendering operational ✅
- **Focus next on Phase 4** - most visible user-facing features!
- Resources and entities are what make the world "alive"
- Everything else is polish and optimization
