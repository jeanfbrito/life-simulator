# CLAUDE.md

This file provides guidance to AI assistants when working with code in this repository.

## Project Overview

Life Simulator is a headless life simulation game built with Bevy 0.16 that features a **separated architecture** where world generation is completely independent from the running simulation engine. Maps are generated as a separate CLI step, and the life simulator loads pre-generated worlds for consistent, reproducible simulations.

The project includes **two visualization clients**:
- **Web Viewer**: Browser-based HTML5 canvas renderer (orthogonal view)
- **Godot Viewer**: Native desktop application with hardware-accelerated isometric rendering

## Prerequisites

- Rust 1.70+ (recommended to use [rustup](https://rustup.rs/))
- Git
- A modern web browser (for web viewer)
- Godot 4.3+ (for Godot viewer) - [Download](https://godotengine.org/)

## Common Commands

### Development Build & Run
```bash
# Build the project
cargo build

# Generate a world (REQUIRED before running the simulator)
cargo run --bin map_generator

# Generate world with custom parameters
cargo run --bin map_generator -- --name "my_world" --seed 12345 --radius 10 --verbose

# Run the life simulator (requires a pre-generated world)
cargo run --bin life-simulator

# Release build (optimized with LTO)
cargo build --release --bin life-simulator
```

### Testing & Quality
```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Run with debug logging
RUST_LOG=debug cargo run --bin life-simulator
```

### Running a Single Test
```bash
# Run a specific test by name
cargo test test_name

# Run tests in a specific module
cargo test module_name::

# Run tests matching a pattern
cargo test terrain
```

### Visualization Clients

#### Web Viewer (Browser-Based)
```bash
# Start the simulator (web server starts automatically on port 54321)
cargo run --bin life-simulator

# Access the web viewer
# Open http://127.0.0.1:54321/viewer.html in your browser
```

**Web Viewer Features:**
- **Interactive Map**: Click and drag to pan around the island
- **Zoom**: Mouse wheel to zoom in/out
- **Terrain Display**: 12 different terrain types with distinct colors
- **Resource Rendering**: Trees, bushes, flowers, rocks
- **Entity Display**: Live animals with behavior labels
- **Dark Theme**: Optimized for comfortable viewing

#### Godot Viewer (Native Desktop)
```bash
# Terminal 1: Start the backend simulator
cargo run --bin life-simulator

# Terminal 2: Launch Godot viewer
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5 to run (or use Play button in Godot Editor)
```

**Godot Viewer Features:**
- **Isometric Rendering**: Beautiful diamond-shaped tiles (128×64 pixels)
- **Advanced Camera**: Mouse wheel zoom, middle-click pan, edge scrolling, WASD movement
- **Resource Display**: Emoji-based rendering with Y-sorting for depth
- **Entity Rendering**: Live animals with scaling and action labels
- **Real-time Statistics**: HUD with world info, entity counts, performance metrics
- **Hardware Acceleration**: Native OpenGL rendering for smooth performance
- **Professional UI**: Toggle-able controls overlay and statistics panel

**For detailed Godot viewer documentation, see:** `godot-viewer/CLAUDE.md`

### API Testing
```bash
# Get current world info
curl http://127.0.0.1:54321/api/world_info

# List available worlds
curl http://127.0.0.1:54321/api/worlds

# Switch to a different world
curl -X POST http://127.0.0.1:54321/api/world/select \
  -H "Content-Type: application/json" \
  -d '{"world_name": "my_world"}'

# Get chunk data
curl http://127.0.0.1:54321/api/chunks?coords=0,0&coords=1,0

# Get multi-layer chunk data
curl "http://127.0.0.1:54321/api/chunks?center_x=0&center_y=0&radius=3&layers=true"
```

## Architecture Overview

### Separated Architecture Pattern

The project uses a **fundamental separation between world generation and simulation**:

1. **Map Generator** (`src/map_generator.rs`): Standalone CLI tool that generates complete worlds and saves them as RON files in `maps/`. Generates both terrain and resources layers in a single pass.

2. **Life Simulator** (`src/main.rs`): Headless Bevy ECS application that loads pre-generated worlds via `WorldLoader`. Never generates terrain at runtime.

3. **Web Server** (`src/web_server_simple.rs`): Simple HTTP server (port 54321) that serves world data through a RESTful API and hosts the interactive web viewer.

This separation ensures:
- **Reproducible worlds**: Same seed always generates identical worlds
- **World versioning**: Worlds can be saved, shared, and version controlled
- **Performance**: No runtime terrain generation overhead
- **Consistency**: All systems see the exact same world state

### Key Architectural Components

#### World Data Flow
```
map_generator → RON files (maps/) → WorldLoader → CachedWorld (in-memory) → web_server_simple → Visualization Clients
                                                                                                    ├→ Web Viewer (HTML/Canvas)
                                                                                                    └→ Godot Viewer (GDScript/Isometric)
```

Both viewers connect to the same HTTP API on port 54321 and receive identical chunk data.

#### Multi-Layer System
Worlds are stored with multiple layers in a single structure:
- **terrain**: Base terrain types (Grass, Forest, Sand, Water, etc.)
- **resources**: Resource deposits (Wood, Stone, Iron, etc.)
- Extensible to add more layers in the future

All layers for a chunk are stored together in `SerializedChunk.layers` (a `HashMap<String, Vec<Vec<String>>>`). The `CachedWorld` loads all layers at startup for fast API access.

#### Chunk-Based World Structure
- **Chunk Size**: Fixed 16×16 tiles (`CHUNK_SIZE = 16`)
- **Chunk Coordinates**: Integer coordinates (chunk_x, chunk_y), not world coordinates
- **Chunk Key Format**: String format `"x,y"` used as HashMap keys in serialized data
- **World Tiles**: World coordinates = chunk_coordinates × 16 + tile_offset

#### Terrain Generation Algorithm
Located in `tilemap/world_generator.rs`, implements:
- **Circular Island Generation**: Distance-based calculations from center point
- **Beach Transitions**: Deep Water → Shallow Water → Sand → Land
- **Natural Variations**: Controlled irregularity using sine/cosine functions
- **Biome System**: Different terrain zones based on distance and randomness

#### ECS Architecture (Bevy)
- **Headless Mode**: No rendering, runs with `ScheduleRunnerPlugin` at 60 FPS
- **Components**: `PositionComponent`, `Chunk`, `ChunkCoordinate`
- **Resources**: `WorldLoader`, `WorldConfig`, `ChunkManager`, `CachedWorld`, `WorldGenerator`
- **Systems**: 
  - `simulation_system`: Main simulation loop
  - `chunk_loading_system` / `chunk_unloading_system`: Chunk memory management
  - `terrain_query_api_system`: Terrain queries for external systems

### Critical Files & Modules

#### Core Systems
- **`src/main.rs`**: Application entry point, sets up Bevy ECS in headless mode
- **`src/lib.rs`**: Public API exports for library usage
- **`src/map_generator.rs`**: Standalone world generation binary

#### World Management
- **`src/world_loader.rs`**: Loads pre-generated worlds from RON files, never generates terrain
- **`src/serialization.rs`**: RON serialization/deserialization with multi-layer support
- **`src/cached_world.rs`**: Global in-memory cache for fast chunk access by web server

#### Tilemap System (`src/tilemap/`)
- **`mod.rs`**: Plugin registration and initialization
- **`world_generator.rs`**: Procedural terrain generation algorithms (used by map_generator)
- **`chunk.rs`**: Chunk data structure and CHUNK_SIZE constant
- **`chunk_manager.rs`**: Chunk loading/unloading, memory management
- **`terrain.rs`**: TerrainType enum and properties
- **`biome.rs`**: BiomeType enum and biome generation
- **`terrain_query.rs`**: Pathfinding and terrain analysis

#### Resources System
- **`src/resources.rs`**: Resource type definitions and generation logic for resources layer

#### Visualization Components

**Backend API:**
- **`src/web_server_simple.rs`**: HTTP server with multi-threaded request handling

**Web Viewer (Browser):**
- **`web-viewer/viewer.html`**: Interactive map viewer with pan/zoom
- **`web-viewer/js/`**: Modular JavaScript for rendering, networking, controls
- **`web-viewer/js/entity-manager.js`**: Entity polling and management (200ms interval)

**Godot Viewer (Native Desktop):**
- **`godot-viewer/`**: Complete Godot 4.3+ project
- **`godot-viewer/scenes/World.tscn`**: Main scene with isometric rendering
- **`godot-viewer/scripts/`**: GDScript modules for rendering, networking, camera
- **`godot-viewer/CLAUDE.md`**: Comprehensive Godot-specific documentation
- **`godot-viewer/docs/`**: Detailed guides on coordinate systems, camera positioning

### Data Persistence

#### World Files
- **Format**: RON (Rusty Object Notation)
- **Location**: `maps/` directory for generated worlds
- **Structure**: `SerializedWorld` containing name, seed, config, and all chunks with layers
- **Version**: v0.2.0 with multi-layer support

#### Save Files
- **Location**: `saves/` directory (currently unused by main flow)
- **Format**: Same as world files, but represents simulation state snapshots

### HTTP API Endpoints

The web server provides these endpoints:
- `GET /viewer.html` - Main viewer interface
- `GET /api/world_info` - Current world metadata
- `GET /api/world/current` - Current loaded world details
- `GET /api/worlds` - List all available worlds
- `POST /api/world/select` - Switch to different world
- `GET /api/chunks?coords=x,y` - Get chunk data (legacy terrain-only)
- `GET /api/chunks?center_x=0&center_y=0&radius=3&layers=true` - Get multi-layer chunks

#### URL Length & Batching
For large chunk requests (e.g., 7×7 grid = 49 chunks), URLs can exceed limits causing connection resets. The web viewer automatically batches requests into groups of 10 chunks maximum.

### Important Design Patterns

#### Never Generate Terrain at Runtime
The simulator **never** calls `WorldGenerator.generate_chunk()` during normal operation. All terrain must be pre-generated by `map_generator` and loaded via `WorldLoader`.

#### CachedWorld Global State
`CachedWorld` uses a global `RwLock<Option<CachedWorld>>` for thread-safe access from the web server. Updated when worlds are loaded or switched via the API.

#### Multi-Layer Chunk Access
Always use `WorldLoader.get_chunk_layers()` to get all layers. For backward compatibility, `get_chunk_data()` returns terrain-only. The web API supports both formats via the `layers=true` parameter.

#### Chunk Coordinate Parsing
When parsing chunk keys from strings (format `"x,y"`), always handle the comma split and parse errors carefully. See `parse_chunk_key()` in `web_server_simple.rs`.

## Dependencies & Build Configuration

### Key Dependencies
- **bevy 0.16**: Game engine with `multi_threaded`, `serialize`, `bevy_log` features (no rendering)
- **rand / rand_pcg**: Deterministic random number generation for world gen
- **serde / ron**: Data serialization to/from RON format
- **tokio**: Async runtime (currently unused, legacy from WebSocket attempt)
- **clap**: CLI argument parsing for map_generator

### Build Profiles
- **Dev**: `opt-level = 1` for project code, `opt-level = 3` for dependencies (fast iteration)
- **Release**: `lto = "thin"` for link-time optimization (maximum performance)
- **Dev Dependencies**: Bevy with `dynamic_linking` feature for faster compilation during development

## Visualization Architecture

### Web Viewer Architecture (Browser-Based)

The web viewer is built with modular JavaScript:
- **`js/config.js`**: Terrain colors, tile size constants
- **`js/network.js`**: API communication and chunk fetching
- **`js/chunk-manager.js`**: Chunk caching and loading logic
- **`js/renderer.js`**: Canvas rendering with pan/zoom
- **`js/controls.js`**: Mouse input handling
- **`js/app.js`**: Main application initialization
- **`js/entity-manager.js`**: Entity polling and management (200ms interval)

**Rendering pattern:** Fetch chunks in batches → Cache locally → Render visible area to canvas

### Godot Viewer Architecture (Native Desktop)

The Godot viewer is built with GDScript and Godot's scene system:

**Core Systems:**
- **`WorldRenderer.gd`**: Main renderer, camera management, chunk orchestration
- **`TerrainTileMap.gd`**: Isometric TileMap rendering with dynamic colored sources
- **`ChunkManager.gd`**: HTTP chunk loading (autoload singleton)
- **`WorldDataCache.gd`**: Client-side chunk caching (autoload singleton)
- **`Config.gd`**: Terrain colors, resource emojis, API endpoints (autoload singleton)

**Advanced Features:**
- **`CameraController.gd`**: Multi-input camera system (mouse, keyboard, edge scrolling)
- **`ResourceManager.gd`**: Resource sprite rendering with Y-sorting
- **`EntityManager.gd`**: Entity polling and display (200ms interval)
- **`StatisticsHUD.gd`**: Real-time performance and world statistics overlay
- **`ControlsOverlay.gd`**: Toggle-able help panel

**Rendering pattern:** Load chunks via HTTP → Build TileMap sources → Paint isometric tiles → Update resources/entities

**Key Design Principle:** Camera positioning MUST use `map_to_local()` to convert tile coordinates to pixel space. See `godot-viewer/docs/CAMERA_COORDINATES.md` for critical coordinate system information.

### Entity Rendering Standards

**IMPORTANT**: All entities (emojis, sprites) must be rendered with a **Y offset of -0.2 tiles** (upward) to keep the feet/base inside the grid square. This ensures:
- Entity feet remain within the tile boundaries visually
- Proper alignment with grid-based pathfinding
- Consistent appearance across all entity types

**Example in `renderer.js`:**
```javascript
// Render the emoji with Y offset to position feet above
const entityY = screenY + (CONFIG.TILE_SIZE * -0.2); // Move up 0.2 tiles
this.ctx.fillText('🧍‍♂️', screenX, entityY);
```

**Default for new entities:**
- All future entity types (animals, NPCs, etc.) should use this same -0.2 Y offset
- Entity emojis render at 1.2× tile size
- Current entity polling rate: 200ms (5 times per second)

## Development Notes

### Before You Start
Always generate a world before running the simulator:
```bash
cargo run --bin map_generator
cargo run --bin life-simulator
```

### Performance Optimization
- Chunks are loaded on-demand by `ChunkManager` with configurable `VIEW_DISTANCE` and `MAX_LOADED_CHUNKS`
- Web server runs in separate thread to avoid blocking simulation
- Canvas uses `putImageData()` for fast pixel rendering

### Testing the Web Viewer
Complete testing checklist in README.md, but key items:
- Verify 7×7 chunk grid loads (49 chunks total)
- Check both terrain and resources layers display
- Confirm batched requests work without connection errors
- Test pan/zoom functionality

### Extending the System
To add new layers:
1. Update `ResourceGenerator` or create new generator in `map_generator.rs`
2. Add layer to chunk in `SerializedChunk` via `set_layer()`
3. Update `CachedWorld.generate_multi_layer_chunks_json()` to include new layer
4. Add rendering logic in `web-viewer/js/renderer.js`

### Common Pitfalls

**Backend:**
- **Missing world files**: Always run map_generator before simulator
- **Port conflicts**: Web server runs on 54321, ensure it's available
- **URL length limits**: Large chunk requests need batching (handled automatically by viewers)
- **Chunk key format**: Always use "x,y" string format, not "(x,y)" or other variants
- **Layer access**: Use multi-layer methods when working with resources or future layers
- **❌ CRITICAL: Entity spawning**: NEVER manually spawn entities with component tuples. ALWAYS use spawn helper functions (`spawn_rabbit()`, `spawn_human()`, etc.) to ensure `BehaviorConfig` is attached. Missing `BehaviorConfig` = AI won't work. See "Entity System and AI Configuration" section below.

**Web Viewer:**
- **Entity Y offset**: Always render entities with -0.2Y offset to keep feet in tile
- **Polling rate**: 200ms entity polling is a balance between freshness and server load
- **Canvas performance**: Large worlds may require viewport culling

**Godot Viewer:**
- **❌ CRITICAL: Camera positioning**: Camera.position MUST use pixel coordinates from `map_to_local()`, NEVER tile coordinates directly. Tile (0,0) is NOT at pixel (0,0)!
- **TileMap painting**: `set_cell()` expects tile coordinates directly - do NOT use `local_to_map()` conversion
- **Chunk tracking**: Only mark chunks as rendered AFTER actual painting, not before
- **Layer configuration**: Godot 4.x requires explicit layer setup with `add_layer()` and `set_layer_enabled()`
- **Coordinate confusion**: Four coordinate systems (world tiles, chunks, local chunk, pixels) - see `godot-viewer/docs/CAMERA_COORDINATES.md`
- **Missing Godot**: Viewer requires Godot 4.3+ - [Download here](https://godotengine.org/)

## Terrain Types & Features

### Available Terrain Types
- **Deep Water**: Outer ocean (#003366)
- **Shallow Water**: Coastal transition zone (#4a7ba7)
- **Sand**: Beach areas (#f4e4bc)
- **Grass**: Common land terrain
- **Forest**: Wooded areas
- **Desert**: Dry, sandy terrain
- **Dirt**: Barren land
- **Mountains**: Elevated terrain
- **Snow**: Cold, snowy peaks
- **Stone**: Rocky areas
- **Swamps**: Wetland areas

### Terrain Generation Features
- **Circular Islands**: Mathematical distance-based island generation
- **Realistic Beaches**: Proper water transitions (Deep Water → Shallow Water → Sand → Land)
- **Natural Variations**: Controlled irregularity using sine/cosine functions
- **Biome Diversity**: Multiple terrain types with smooth transitions

### Terrain Zones
1. **Circular Island Base**
   - Distance-based calculations from center point
   - Controlled irregularity using sine/cosine functions
   - Distinct terrain zones with smooth transitions

2. **Terrain Transition Layers**
   - Deep Water: Outer ocean
   - Shallow Water: Coastal transition zone
   - Sand Beach: Island border
   - Land Interior: Various biomes with grass center

3. **Chunk-Based Architecture**
   - 16×16 tile chunks for efficient memory usage
   - Procedural generation on-demand by map_generator
   - HTTP API for terrain data access

## Map Generator Options

The standalone map generator provides these options:

```bash
cargo run --bin map_generator -- --help

Options:
  -n, --name <NAME>         World name [default: generated_world]
  -s, --seed <SEED>         World generation seed (random if not specified)
  -r, --radius <RADIUS>     World size in chunks radius [default: 5]
  -o, --output-dir <DIR>    Output directory [default: maps]
  -p, --preview             Generate HTML preview
  -v, --verbose             Verbose output
```

### Example Map Generation Commands
```bash
# Generate a default world (radius 5, random seed)
cargo run --bin map_generator

# Generate with custom parameters
cargo run --bin map_generator -- --name "my_world" --seed 12345 --radius 10 --verbose

# Generate with preview HTML
cargo run --bin map_generator -- --name "test_world" --preview
```

## Testing Checklist

### Backend & API
- [ ] Server starts successfully on `http://127.0.0.1:54321`
- [ ] World info API returns correct center chunk and size
- [ ] Multi-layer chunk endpoint works with `layers=true` parameter
- [ ] Entity API endpoint returns live entity data

### Web Viewer Testing
**Basic Functionality:**
- [ ] Web viewer loads at `http://127.0.0.1:54321/viewer.html`
- [ ] Complete 7×7 grid loads correctly (49 chunks total)
- [ ] Both terrain and resources layers display properly
- [ ] Entities appear and update every 200ms

**Visual Quality:**
- [ ] Chunk boundaries render without artifacts
- [ ] Terrain colors match expected types (water, sand, grass, forest, etc.)
- [ ] Resources display with correct sprites/emojis
- [ ] Entities render with -0.2Y offset (feet inside tile)

**Interactivity:**
- [ ] Pan functionality works (click and drag)
- [ ] Zoom functionality works (mouse wheel)
- [ ] No JavaScript console errors during operation

**Performance:**
- [ ] Batched requests work without connection reset errors
- [ ] Map loads within reasonable time (< 5 seconds)
- [ ] 60 FPS rendering maintained

### Godot Viewer Testing
**Basic Functionality:**
- [ ] Godot project opens without errors
- [ ] Scene runs (F5) and connects to backend
- [ ] All 49 chunks load and paint correctly
- [ ] Isometric rendering displays properly

**Camera Controls:**
- [ ] Mouse wheel zoom (0.2x - 5.0x range)
- [ ] Middle mouse drag panning
- [ ] Edge scrolling (50px margin)
- [ ] Keyboard movement (WASD + Arrow keys)
- [ ] +/- keys for zoom
- [ ] Camera positioned correctly on tile (0,0)

**Visual Features:**
- [ ] Terrain tiles render in isometric projection (128×64 diamonds)
- [ ] Resources display with emoji symbols
- [ ] Entities appear with proper scaling and labels
- [ ] Y-sorting works correctly for depth

**UI Overlays:**
- [ ] Controls overlay visible and toggle-able (H key)
- [ ] Statistics HUD displays correctly (Tab key)
- [ ] Real-time statistics update every second
- [ ] FPS and memory metrics show correctly

**Validation:**
- [ ] Camera controls validation passes (8/8 tests)
- [ ] Statistics HUD validation passes (7/7 tests)
- [ ] No Godot errors in output console

### Data Integrity (Both Viewers)
- [ ] Saved world data matches displayed terrain
- [ ] Resources layer data matches between viewers
- [ ] Entity positions match between viewers (accounting for polling delay)
- [ ] Chunk coordinates calculated correctly from center point
- [ ] No missing or corrupted chunks in displayed area

## References and Inspiration

**Core Engine:**
- [Bevy Game Engine](https://bevyengine.org/) - The game engine powering the simulation backend

**Visualization:**
- **`godot-viewer/`** - Native desktop isometric viewer (Godot 4.3+)
  - See `godot-viewer/CLAUDE.md` for comprehensive documentation
  - See `godot-viewer/docs/CAMERA_COORDINATES.md` for coordinate system deep dive
- **`web-viewer/`** - Browser-based orthogonal viewer (HTML5 Canvas)

**Terrain & World Generation:**
- `/Users/jean/Github/world-simulator` - Terrain generation inspiration
- Procedural content generation techniques for realistic island formation

**AI & Behavior:**
- `/Users/jean/Github/dogoap` - AI and behavior tree reference
- `/Users/jean/Github/big-brain` - AI planning and decision-making reference

**Pathfinding:**
- `/Users/jean/Github/bevy_entitiles` - Tile-based entity system and **pathfinding algorithm** reference
  - Key file: `src/algorithm/pathfinding.rs` - Original A* implementation
  - Our implementation simplified for single-threaded discrete tick simulation
  - Max steps increased to 5000 (vs original 1000) due to resource-fragmented terrain

## Viewer Comparison & Selection Guide

### When to Use Web Viewer
**Best for:**
- ✅ Universal accessibility (no installation required)
- ✅ Quick testing and debugging
- ✅ Remote access scenarios
- ✅ Platform-independent deployment
- ✅ Lightweight solution

**Limitations:**
- ⚠️ Limited by browser Canvas performance
- ⚠️ Orthogonal view only (no isometric)
- ⚠️ Basic camera controls
- ⚠️ No real-time statistics overlay

### When to Use Godot Viewer
**Best for:**
- ✅ Native desktop experience
- ✅ Hardware-accelerated performance
- ✅ Beautiful isometric rendering
- ✅ Advanced camera controls (pan, zoom, edge scrolling)
- ✅ Professional UI with statistics overlay
- ✅ Local development and testing

**Limitations:**
- ⚠️ Requires Godot 4.3+ installation
- ⚠️ Platform-specific builds for distribution
- ⚠️ Larger initial setup

### Feature Parity Matrix

| Feature | Web Viewer | Godot Viewer |
|---------|------------|--------------|
| Terrain Rendering | ✅ Orthogonal | ✅ Isometric (128×64) |
| Resource Display | ✅ Basic | ✅ Emoji with Y-sort |
| Entity Rendering | ✅ Basic | ✅ Scaling + labels |
| Camera Pan | ✅ Drag | ✅ Drag + WASD + Edge |
| Camera Zoom | ✅ Wheel | ✅ Wheel + Keys (0.2-5x) |
| Statistics Overlay | ❌ | ✅ Real-time HUD |
| Performance Metrics | ❌ | ✅ FPS + Memory |
| Installation | None | Godot 4.3+ |
| Platform Support | Universal Browser | Desktop (exportable) |

**Recommendation:** Use Godot viewer for development and rich visualization. Use web viewer for quick checks and universal access.

See `godot-viewer/VIEWER_COMPARISON_REPORT.md` for detailed feature analysis.

## Future Development Ideas

This project serves as a foundation for:
- Advanced life simulation mechanics
- AI-driven entity behavior
- Complex ecosystem interactions
- Multi-user web-based simulation
- Real-time terrain modification
- Godot web export (combining native performance with browser accessibility)

## Pathfinding and Movement System

### Architecture Overview

The life simulator uses a **tick-based discrete movement system** with A* pathfinding:

- **Pathfinding**: Runs asynchronously every frame (not tick-synced) for responsive path calculation
- **Movement**: Discrete tile-by-tile movement that executes only on simulation ticks
- **No smooth interpolation**: Entities jump from tile to tile (discrete simulation)

### Core Components

#### Pathfinding Module (`src/pathfinding.rs`)

**Key Types:**
- `PathfindingGrid`: Resource storing movement costs for all tiles (built from terrain)
- `PathRequest`: Component requesting path calculation (origin → destination)
- `Path`: Component containing computed waypoints
- `PathNode`: Internal A* algorithm node (g_cost, h_cost, parent)

**Algorithm:**
- Pure A* with Manhattan distance heuristic
- BinaryHeap priority queue for O(E log V) performance
- Respects terrain costs (Grass=1, Forest=3, DeepWater=impassable)
- Supports diagonal movement (optional)
- max_steps limit to prevent infinite searches

**Systems (Non-Tick):**
- `process_pathfinding_requests()`: Computes paths using A* algorithm

#### Movement Module (`src/entities/movement.rs`)

**Key Components:**
- `TilePosition`: Entity's discrete tile position (IVec2)
- `MoveOrder`: High-level movement command (destination)
- `MovementSpeed`: Ticks per tile (1=fast, 2=normal, 4=slow)
- `MovementState`: Internal tick counter for speed control

**Systems:**
- **Non-Tick**: `initiate_pathfinding()` - converts MoveOrder to PathRequest
- **Non-Tick**: `initialize_movement_state()` - prepares entity for movement
- **TICK-SYNCED**: `tick_movement_system()` - executes discrete movement steps

### Movement Flow

```
1. Entity gets MoveOrder(destination)         [User/AI decision]
2. → PathRequest(origin, dest)               [initiate_pathfinding]
3. → Path(waypoints)                          [process_pathfinding_requests]
4. → MovementState initialized                [initialize_movement_state]
5. Each tick: advance one waypoint            [tick_movement_system]
6. Path complete → components removed         [tick_movement_system]
```

### Integration Example

```rust
fn main() {
    App::new()
        .init_resource::<PathfindingGrid>()
        .add_plugins(EntitiesPlugin)
        .add_systems(Startup, setup_pathfinding_grid)
        .add_systems(Update, process_pathfinding_requests)  // Every frame
        .add_systems(FixedUpdate, tick_movement_system)     // On ticks only
        .run();
}

fn setup_pathfinding_grid(
    world_loader: Res<WorldLoader>,
    mut grid: ResMut<PathfindingGrid>,
) {
    *grid = build_pathfinding_grid_from_world(&world_loader);
}
```

### Usage Example

```rust
use life_simulator::entities::{spawn_creature, issue_move_order, MovementSpeed};

fn spawn_and_move(mut commands: Commands) {
    let entity = spawn_creature(
        &mut commands,
        "Bob",
        "Human",
        IVec2::new(0, 0),
        MovementSpeed::normal(),  // 1 tile per 2 ticks
    );
    
    issue_move_order(&mut commands, entity, IVec2::new(10, 10));
}
```

### Terrain Movement Costs

Defined in `TerrainType::movement_cost()`:
- Grass: 1 (easy)
- Sand/Dirt: 2 (normal)
- Forest/Stone: 3 (trees/rocks)
- Desert: 4 (hot, tiring)
- ShallowWater: 5 (wading)
- Snow: 6 (cold, deep)
- Mountain: 8 (very difficult)
- Swamp: 10 (very slow)
- DeepWater: u32::MAX (impassable)

### Performance Notes

- **PathfindingGrid**: ~8 bytes per tile, ~80KB for 100×100 grid
- **A* complexity**: O(E log V), typically <1ms for 100×100 with obstacles
- **Memory efficient**: Chunked storage for world data
- **Pathfinding not tick-synced**: Can take multiple frames without blocking ticks

### Key Files

- `src/pathfinding.rs`: A* algorithm, PathfindingGrid, terrain costs
- `src/entities/mod.rs`: Entity types, plugin, spawn helpers
- `src/entities/movement.rs`: Tick-based movement execution
- `docs/MOVEMENT_INTEGRATION.md`: Complete integration guide

### Important Design Decisions

1. **Discrete movement**: Entities don't interpolate - they teleport tile-by-tile on ticks
2. **Tick-synced execution**: Only `tick_movement_system` runs on simulation ticks
3. **Async pathfinding**: Path calculation happens off-tick for responsiveness
4. **Terrain-based costs**: Movement cost derived from TerrainType, not separate data
5. **Speed via tick budgets**: MovementSpeed controls ticks-per-tile, not tiles-per-second

### Testing

```bash
# Run pathfinding unit tests
cargo test --lib pathfinding

# Enable debug logging
RUST_LOG=life_simulator=debug cargo run --bin life-simulator
```

### Source Inspiration

Extracted and adapted from `/Users/jean/Github/bevy_entitiles/src/algorithm/pathfinding.rs` but simplified:
- Removed bevy_entitiles dependency
- Removed multi-threaded task system
- Removed tilemap entity associations
- Added discrete tick-based execution
- Integrated directly with our terrain system

## Tick System (Simulation Heartbeat)

### Architecture Overview

The life simulator uses a **discrete tick-based system** inspired by Dwarf Fortress, Factorio, and RimWorld:

- **Base rate**: 10 TPS (ticks per second) = 100ms per tick
- **Fixed timestep**: Uses Bevy's FixedUpdate schedule for determinism
- **Multi-rate updates**: Different systems run at different frequencies
- **Pause/Speed controls**: Built-in speed multiplier and pause functionality
- **Performance monitoring**: Real-time TPS tracking and metrics

### Core Components

#### SimulationTick Resource
```rust
#[derive(Resource)]
pub struct SimulationTick(pub u64);  // Current tick counter
```

#### SimulationSpeed Resource
```rust
#[derive(Resource)]
pub struct SimulationSpeed {
    pub multiplier: f32,  // 0.5x, 1.0x, 2.0x, 3.0x
    paused: bool,
}
```

#### TickMetrics Resource
- Tracks tick durations (last 60 ticks)
- Calculates actual TPS
- Reports min/max/average tick times

### System Categories by Update Frequency

**Fast Systems (Every Tick - 10 TPS)**
- Movement execution
- Combat resolution
- Physics updates
- Immediate interactions

**Medium Systems (Every 5-10 Ticks - 1-2 TPS)**
- AI decision making (every 5 ticks)
- Job assignment (every 10 ticks)
- Needs updates (hunger, thirst - every 10 ticks)
- Social interactions (every 10 ticks)

**Slow Systems (Every 100+ Ticks - 0.1 TPS)**
- Plant growth (every 100 ticks)
- Aging (every 1000 ticks)
- Weather changes (every 1000 ticks)
- World events (every 1000+ ticks)

**Async Systems (Not Tick-Bound)**
- Pathfinding calculation (runs at 60fps)
- Terrain generation
- Save/load operations

### Integration Example

```rust
use life_simulator::simulation::{SimulationPlugin, every_n_ticks};

fn main() {
    App::new()
        .add_plugins(SimulationPlugin)  // Adds tick system
        
        // Tick systems (run on fixed timestep)
        .add_systems(FixedUpdate, (
            // Fast: every tick
            tick_movement_system,
            
            // Medium: conditional
            ai_system.run_if(every_n_ticks(5)),
            
            // Slow: conditional
            plant_growth.run_if(every_n_ticks(100)),
        ))
        .run();
}
```

### Run Conditions

- `every_n_ticks(n)` - Execute system every N ticks
- `when_paused()` - Execute only when simulation is paused
- `when_not_paused()` - Execute only when simulation is running
- `on_tick(n)` - Execute on specific tick number
- `after_tick(n)` - Execute after N ticks have passed

### Keyboard Controls

- **Space**: Pause/Unpause
- **1**: 0.5x speed
- **2**: 1.0x normal speed
- **3**: 2.0x fast
- **4**: 3.0x ultra

### Performance Metrics

Automatic logging every 100 ticks (10 seconds):
```
╔══════════════════════════════════════════╗
║       TICK METRICS - Tick 100            ║
╠══════════════════════════════════════════╣
║ Actual TPS:        10.1                  ║
║ Speed:             1.0x                  ║
║ Status:            RUNNING               ║
║ Tick Duration:                           ║
║   Average:          1.23ms               ║
║   Min:              0.98ms               ║
║   Max:              2.45ms               ║
║ Total Ticks:           100               ║
║ Uptime:            00:00:10              ║
╚══════════════════════════════════════════╝
```

### Key Design Decisions

1. **10 TPS base rate**: Good balance between responsiveness and CPU usage
2. **Deterministic ticks**: Fixed timestep ensures reproducible simulation
3. **Decoupled rendering**: Logic at 10 TPS, rendering at 60 FPS
4. **Multi-rate updates**: Expensive systems run less frequently
5. **Budget-based pathfinding**: Async calculation, tick-synced application

## CRITICAL: Simulation vs. Viewer Separation

### Pure Discrete Simulation Philosophy

This simulator follows a **strict separation between simulation logic and visualization**:

#### Simulation Side (Rust/Bevy)
- ✅ **DISCRETE, TICK-BASED ONLY**: All movement and logic happens on discrete ticks
- ✅ **NO INTERPOLATION**: Entities teleport from tile A to tile B instantly
- ✅ **GRID-LOCKED**: Positions are always exact tile coordinates (IVec2)
- ✅ **DETERMINISTIC**: Same inputs always produce same outputs
- ✅ **ROGUELIKE STYLE**: Think Dwarf Fortress, NetHack, CDDA - pure grid movement

**Example:** Entity at tile (5, 5) waits 15 ticks, then **instantly jumps** to tile (5, 6).

#### Viewer Side (JavaScript/Web)
- ✅ **POLLS STATE**: Fetches entity positions via HTTP every 200ms
- ✅ **DISPLAYS CURRENT STATE**: Shows entities at their exact tile positions
- ✅ **NO CLIENT-SIDE INTERPOLATION**: Entities jump from tile to tile in the viewer too
- ✅ **OPTIONAL SMOOTHING**: Could add interpolation in future, but NOT required

**Example:** Viewer polls every 200ms, sees entity at (5,5) three times, then sees it at (5,6).

### Why This Matters

**NEVER try to add interpolation or smooth movement to the simulation engine.**
- Movement interpolation belongs ONLY in the viewer (if desired at all)
- The simulation must remain pure, discrete, and deterministic
- This is a feature, not a bug - it's how roguelikes and simulation games work

### Current Movement Behavior (AS DESIGNED)

```
Tick:    0    1    2    3 ... 14   15   16   17 ... 29   30
Pos:     A    A    A    A     A    B    B    B      B    C
         └─── Counting ticks ──┘    └─── Counting ──┘
                               ▲                      ▲
                            JUMP                   JUMP
```

With `MovementSpeed::custom(15)`:
- Entity stays at tile A for 15 ticks (1.5 seconds at 10 TPS)
- On tick 15, entity **instantly teleports** to tile B
- Entity stays at tile B for 15 ticks
- On tick 30, entity **instantly teleports** to tile C

### Web Viewer Polling

With 200ms polling (5 times per second):
- Viewer polls at 0ms, 200ms, 400ms, 600ms, etc.
- Sees entity position at exact moment of poll
- Entity appears to "jump" because it does jump - **this is correct behavior**

### If You Want Smoother Visuals

**Option 1: Faster polling (already at 200ms)**
- More frequent updates show more of the jumps
- Still discrete, just more frequent samples

**Option 2: Client-side interpolation (NOT IMPLEMENTED, not required)**
- JavaScript could interpolate between known positions
- **Never add this to the simulation engine**
- Optional future enhancement for viewer only

**Option 3: Faster movement (adjust ticks_per_move)**
- Lower `ticks_per_move` = more frequent jumps
- Still discrete, just more frequent

### What NOT To Do

❌ **Do NOT add smooth position interpolation to the Rust simulation**
❌ **Do NOT use f32 positions in the simulation (always IVec2)**
❌ **Do NOT try to "fix" the jumping - it's intentional**
❌ **Do NOT add delta-time based movement to the core simulation**

### What TO Do

✅ **Keep simulation discrete and tick-based**
✅ **Keep entity positions as exact tile coordinates**
✅ **All movement happens instantly on specific ticks**
✅ **Add interpolation only to viewer if truly needed**
✅ **Embrace the discrete, roguelike style**

### Key Files

- `src/simulation/mod.rs`: SimulationPlugin and speed controls
- `src/simulation/tick.rs`: Core tick resources and systems
- `docs/TICK_SYSTEM_ANALYSIS.md`: Deep analysis of tick architectures (816 lines)
- `docs/TICK_SYSTEM_QUICKSTART.md`: Quick start guide

### Integration with Pathfinding & Movement

```
Frame (60fps)                     Tick (10 TPS)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                                  
Pathfinding calculation           
  (process_pathfinding_requests)  
        ↓                         
  Path computed                        ↓
                                  Movement execution
                                  (tick_movement_system)
                                       ↓
                                  TilePosition updated
```

### Testing

```bash
# Run tick system tests
cargo test --lib simulation::tick::tests

# Expected: 4 tests passing
# - test_tick_increment
# - test_speed_control
# - test_update_frequency
# - test_tick_metrics
```

### References

- **Dwarf Fortress**: Variable tick rate, priority queues
- **Factorio**: Fixed 60 UPS, deterministic lockstep
- **RimWorld**: Multi-rate "ticker" system (Normal/Rare/Long)
- **Oxygen Not Included**: 5 TPS sub-tick physics

## Entity Movement Verification Test

### Critical Test: Verifying Entities Are Alive and Moving

This is the **fundamental test** to verify the simulation is working correctly. Run this test after any changes to the tick system, movement system, or entity spawning to ensure entities are alive and actively moving.

```bash
# Start simulation in background
cargo run --bin life-simulator > /tmp/life-simulator.log 2>&1 &
sleep 6  # Wait for startup

# Track human and rabbit movement for 20 seconds
echo "=== Comparing Movement Speeds ==="
echo ""
printf "%-8s | %-12s | %-12s | %s\n" "Time" "Human_0" "Rabbit_0" "Notes"
echo "---------|--------------|--------------|---------------------------"

for i in {1..10}; do
  timestamp=$(date +"%H:%M:%S")
  entities=$(curl -s http://127.0.0.1:54321/api/entities 2>/dev/null)
  human_pos=$(echo "$entities" | jq -r '.entities[] | select(.name == "Human_0") | "\(.position.x),\(.position.y)"' 2>/dev/null)
  rabbit_pos=$(echo "$entities" | jq -r '.entities[] | select(.name == "Rabbit_0") | "\(.position.x),\(.position.y)"' 2>/dev/null)
  printf "%-8s | %-12s | %-12s |\n" "$timestamp" "$human_pos" "$rabbit_pos"
  sleep 2
done

echo ""
echo "Expected: Rabbits move every ~2s (20 ticks), Humans every ~3s (30 ticks)"

# Cleanup
pkill -f "target/debug/life-simulator" 2>/dev/null || true
```

### What This Test Verifies

1. **✅ Tick System Working**: Entities only move if ticks are accumulating and executing
2. **✅ Movement System Active**: Entities advance through their movement ticks
3. **✅ Wandering AI Functional**: Entities generate paths and make decisions
4. **✅ Pathfinding Working**: Entities find valid paths to destinations
5. **✅ Speed Configuration Correct**: Different entity types move at their configured speeds

### Expected Output

```
=== Comparing Movement Speeds ===

Time     | Human_0      | Rabbit_0     | Notes
---------|--------------|--------------|---------------------------
13:45:21 | -7,-2        | -10,-15      |
13:45:23 | -7,-2        | -10,-14      |  <-- Rabbit moved!
13:45:25 | -7,-3        | -9,-14       |  <-- Both moved!
13:45:27 | -7,-3        | -9,-15       |  <-- Rabbit moved!
13:45:29 | -8,-3        | -10,-15      |  <-- Both moved!
13:45:31 | -7,-3        | -10,-16      |  <-- Rabbit moved!
13:45:33 | -7,-3        | -11,-16      |  <-- Rabbit moved!
13:45:35 | -8,-3        | -10,-16      |  <-- Human moved!
13:45:38 | -8,-4        | -10,-15      |  <-- Both moved!
13:45:40 | -9,-4        | -9,-15       |  <-- Both moved!

Expected: Rabbits move every ~2s (20 ticks), Humans every ~3s (30 ticks)
```

### Current Entity Movement Speeds

| Entity Type | Ticks per Tile | Time per Tile | Status |
|-------------|----------------|---------------|---------|
| **Rabbit** 🐇 | 20 | 2.0 seconds | Active |
| **Human** 🧍‍♂️ | 30 | 3.0 seconds | Active |
| **Deer** 🦌 | 10 | 1.0 seconds | Future |
| **Wolf** 🐺 | 6 | 0.6 seconds | Future |

### When to Run This Test

- After modifying the tick accumulation system
- After changing movement speed configurations  
- After updates to the wandering AI
- After pathfinding algorithm changes
- Before committing major simulation changes
- When debugging "entities not moving" issues

### Troubleshooting

If entities aren't moving:
1. Check `/tmp/life-simulator.log` for errors
2. Verify tick counter is incrementing: `grep "Tick #" /tmp/life-simulator.log`
3. Ensure `should_tick` flag is being set
4. Check that movement systems are registered with `.run_if(should_tick)`
5. Verify pathfinding grid is properly initialized

### Automated Test Script

The full test script with analysis is available at `scripts/test_movement.sh`:

```bash
./scripts/test_movement.sh
```

This will:
- Start the simulation
- Track movement for 30 seconds  
- Analyze movement patterns
- Report success/failure
- Clean up processes

See also:
- `docs/TICK_SYSTEM_FIXES.md` - Complete tick system architecture
- `docs/TESTING_TICK_SYSTEM.md` - Comprehensive testing guide
- `docs/MOVEMENT_SPEEDS.md` - Entity speed reference

## Pathfinding System Debugging

### Integration Testing Framework

The project includes a comprehensive integration testing framework for debugging pathfinding issues against real world data:

**Test: `tests/pathfinding_test.rs`**
- Loads actual generated world from `maps/` directory
- Builds pathfinding grid using same logic as main simulation
- Tests pathfinding from multiple spawn points to water sources
- Validates terrain accessibility and resource blocking
- Provides detailed diagnostics for path failures

```bash
# Run with verbose diagnostic output
cargo test --test pathfinding_test -- --nocapture
```

**Expected Output:**
```
🧪 Testing rabbit pathfinding to water
📂 Loading world...
✅ World loaded: full_world
   Chunks: 121
🗺️  Building pathfinding grid...
✅ Grid built: 43681 tiles processed, 19077 walkable, 554 blocked

🐇 Testing from North spawn: IVec2(10, 15)
   💧 Found water at (21, 30) - distance: 18.6 tiles
   🚶 Adjacent walkable: IVec2(21, 29)
   🧭 Finding path from IVec2(10, 15) to IVec2(21, 29)...
   ✅ PATH FOUND! 14 waypoints
```

### Critical Pathfinding Requirements

**ALWAYS enable diagonal movement** in pathfinding requests:
```rust
commands.entity(entity).insert(MoveOrder {
    destination: target_pos,
    allow_diagonal: true,  // ← CRITICAL: Must be true
});
```

**Why Diagonal Movement Matters:**
- With only 4-directional movement (N/S/E/W), even small obstacles create impassable barriers
- 8-directional movement allows navigation around single-tile obstacles
- Essential for realistic pathfinding with resources (trees, bushes) blocking tiles
- Without diagonals, pathfinding success rate drops from 75-100% to 0-25%

### Common Pathfinding Issues

#### Issue: Entities can't reach destinations
**Symptoms:**
- Pathfinding failures in logs: `"PATH FAILED for entity X to Y"`
- Actions planned but never completed
- Entities dying before reaching critical resources (water, food)

**Debugging Steps:**
1. Run integration test: `cargo test --test pathfinding_test -- --nocapture`
2. Check if paths are found from typical spawn points
3. Look for blocked terrain diagnostics in test output
4. Verify `allow_diagonal: true` in all MoveOrder creations
5. Check if resources are blocking too many tiles

**Common Causes:**
- ❌ Diagonal movement disabled (`allow_diagonal: false`)
- ❌ Too many resources blocking paths
- ❌ Spawn points isolated by impassable terrain
- ❌ Water sources too far from spawn (increase `MAX_SEARCH_RADIUS`)

#### Issue: PathfindingFailed component stuck on entities
**Symptoms:**
- Entities stop moving permanently
- Actions report failures but keep trying

**Solution:**
Actions must detect and remove PathfindingFailed component:
```rust
if world.get::<PathfindingFailed>(entity).is_some() {
    warn!("Pathfinding failed for entity {:?}", entity);
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        entity_mut.remove::<PathfindingFailed>();
    }
    return ActionResult::Failed;
}
```

### Pathfinding Documentation References

- **`PATHFINDING_FIX.md`** - Complete pathfinding bug diagnosis and fix
  - Problem analysis with test-driven approach
  - Root cause identification (diagonal movement disabled)
  - Integration test creation for real-world validation
  - Before/after comparison with metrics
  - Recommendations for future improvements

- **`SESSION_2025-01-02.md`** - Complete debugging session summary
  - Investigation methodology
  - Solutions implemented (3 commits)
  - Testing infrastructure created
  - Key takeaways and lessons learned

### Test-Driven Debugging Methodology

When debugging gameplay issues:

1. **Create Integration Test**
   - Load real world data (not mocks)
   - Replicate exact simulation setup
   - Test multiple scenarios

2. **Use Diagnostic Output**
   - Run with `--nocapture` to see all diagnostics
   - Analyze failure points systematically
   - Sample terrain along failed paths

3. **Measure Impact**
   - Track success rates before/after
   - Document metrics in commit messages
   - Validate fixes across multiple test cases

4. **Document Lessons**
   - Create detailed fix documentation
   - Add to README "Key Lessons Learned"
   - Include in session summaries

**Example from PATHFINDING_FIX.md:**
- Problem: Entities couldn't reach water (0% success)
- Root Cause: Diagonal movement disabled
- Fix: Enable `allow_diagonal: true`
- Result: 75-100% success rate
- Test: Integration test validates fix
- Documentation: Complete analysis preserved

### Pathfinding Grid Building

Pathfinding grid must be built from terrain and resources:

```rust
let terrain_cost = if let Some(terrain) = TerrainType::from_str(&terrain_str) {
    let cost = terrain.movement_cost();
    if cost >= 1000.0 {
        u32::MAX  // Impassable
    } else {
        cost as u32
    }
} else {
    u32::MAX
};

let has_resource = world_loader.get_resource_at(x, y)
    .map(|r| !r.is_empty())
    .unwrap_or(false);

let final_cost = if has_resource && terrain_cost != u32::MAX {
    u32::MAX  // Resources block movement
} else {
    terrain_cost
};

pathfinding_grid.set_cost(pos, final_cost);
```

**Note:** Currently ALL resources block movement. Future improvement could differentiate between blocking (trees) and passable (flowers) resources.

### AI Planner Logging

Enhanced logging helps debug AI decisions:

```rust
RUST_LOG=info cargo run --bin life-simulator
```

**Look for:**
- `🧠 Entity X - Evaluated N actions` (action planning)
- `✅ Entity X queuing action Y with utility Z` (action selected)
- `❌ Entity X - No actions above threshold` (no valid actions)
- `🐇 Entity X drank water from Y! Thirst: Z%` (successful behavior)

**Utility Scoring:**
- Drink water: 70% thirst, 30% distance (weighted sum)
- Wander: 0.01 (lowest priority)
- Threshold: 0.05 (lowered to allow early water seeking)

## Entity System and AI Configuration

### Modular Entity Configuration Architecture

The simulator uses a **component-based behavior configuration system** where each entity type has its own modular configuration. This allows easy addition of new entity types and tuning of behavior parameters.

#### File Structure
```
src/entities/
├── mod.rs                  # Entity plugin, exports
├── entity_types.rs         # Spawn functions, entity markers
├── movement.rs             # Movement components and systems
├── stats.rs                # Health, thirst, hunger, energy
└── types/                  # Behavior configurations
    ├── mod.rs              # BehaviorConfig component definition
    └── rabbit.rs           # RabbitBehavior preset
```

#### BehaviorConfig Component

Every AI-driven entity MUST have a `BehaviorConfig` component attached:

```rust
#[derive(Component, Debug, Clone)]
pub struct BehaviorConfig {
    pub thirst_threshold: f32,      // 0.0-1.0, when to seek water
    pub hunger_threshold: f32,      // 0.0-1.0, when to seek food
    pub graze_range: (i32, i32),    // (min, max) tiles for foraging
    pub water_search_radius: i32,   // Max tiles to search for water
    pub food_search_radius: i32,    // Max tiles to search for food
    pub wander_radius: i32,         // Idle wandering range
}
```

**Why it's required:**
- The AI planner queries: `Query<(Entity, &TilePosition, &Thirst, &BehaviorConfig), With<EntityType>>`
- Without `BehaviorConfig`, entities won't be processed by the AI system
- Each behavior module (drinking, grazing, etc.) uses these parameters

#### Entity Behavior Presets

Each entity type has a preset with optimized default values:

**Rabbit** (`src/entities/types/rabbit.rs`):
```rust
pub struct RabbitBehavior;

impl RabbitBehavior {
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new(
            0.15,       // thirst_threshold: Drink at 15% thirsty
            0.4,        // hunger_threshold: Eat at 40% hungry
            (3, 8),     // graze_range: Short-range grazing
            100,        // water_search_radius: Wide search
            100,        // food_search_radius: Wide search
            15,         // wander_radius: Small territory
        )
    }
}
```

**Future entities** can follow the same pattern:
- `src/entities/types/deer.rs` → `DeerBehavior::config()`
- `src/entities/types/wolf.rs` → `WolfBehavior::config()`

### CRITICAL: Entity Spawning Rules

**❌ NEVER manually spawn entities with component tuples:**

```rust
// ❌ WRONG - Missing BehaviorConfig, will break AI!
let rabbit = commands.spawn((
    Creature { name: "Rabbit".to_string(), species: "Rabbit".to_string() },
    Rabbit,
    TilePosition::from_tile(spawn_pos),
    MovementSpeed::custom(20),
    EntityStatsBundle::default(),
    // Missing BehaviorConfig!
)).id();
```

**✅ ALWAYS use the spawn helper functions:**

```rust
// ✅ CORRECT - Uses spawn_rabbit() which attaches BehaviorConfig
use entities::spawn_rabbit;
let rabbit = spawn_rabbit(&mut commands, "TestRabbit", spawn_pos);
```

**Why this matters:**
- Spawn helper functions ensure ALL required components are attached
- They attach the correct `BehaviorConfig` preset for that entity type
- Manual spawning can easily forget components, breaking systems
- This is a common ECS "component dependency" bug

### Available Spawn Functions

Defined in `src/entities/entity_types.rs`:

```rust
// Individual entity spawning
pub fn spawn_human(commands: &mut Commands, name: impl Into<String>, position: IVec2) -> Entity;
pub fn spawn_rabbit(commands: &mut Commands, name: impl Into<String>, position: IVec2) -> Entity;

// Batch spawning with pathfinding validation
pub fn spawn_humans(commands: &mut Commands, count: usize, center: IVec2, 
                    spawn_radius: i32, grid: &PathfindingGrid) -> Vec<Entity>;
pub fn spawn_rabbits(commands: &mut Commands, count: usize, center: IVec2,
                     spawn_radius: i32, grid: &PathfindingGrid) -> Vec<Entity>;
```

**Integration example:**
```rust
fn spawn_entities(
    mut commands: Commands,
    pathfinding_grid: Res<PathfindingGrid>,
) {
    // Spawn single rabbit at specific position
    let rabbit = spawn_rabbit(&mut commands, "Fluffy", IVec2::new(5, 10));
    
    // Spawn multiple rabbits around a point
    let rabbits = spawn_rabbits(&mut commands, 5, IVec2::new(0, 0), 20, &pathfinding_grid);
}
```

### Debugging Missing BehaviorConfig

If entities aren't responding to AI:

1. **Check AI planner logs** (`RUST_LOG=info`):
   ```bash
   # Should see logs like:
   🧠 Entity X at (5, 10) - Thirst: 15.3% - Evaluated 2 actions
   ✅ Entity X queuing action DrinkWater { target_tile: (20, 15) } with utility 0.28
   ```

2. **If no logs appear**, entity likely missing `BehaviorConfig`:
   - Verify entity was spawned using spawn helper function
   - Check that `BehaviorConfig` component is attached
   - Ensure entity has the correct marker component (`Rabbit`, `Human`, etc.)

3. **Verify component attachment** in Bevy inspector or logs:
   ```rust
   // Query to check components
   fn debug_entities(query: Query<(Entity, Option<&BehaviorConfig>), With<Rabbit>>) {
       for (entity, config) in query.iter() {
           if config.is_none() {
               error!("❌ Entity {:?} missing BehaviorConfig!", entity);
           }
       }
   }
   ```

### Adding New Entity Types

To add a new entity type (e.g., Deer):

1. **Create behavior preset** (`src/entities/types/deer.rs`):
   ```rust
   pub struct DeerBehavior;
   
   impl DeerBehavior {
       pub fn config() -> BehaviorConfig {
           BehaviorConfig::new(
               0.2,        // Different thresholds for deer
               0.3,
               (5, 15),    // Wider grazing range
               150,        // Longer water search
               150,
               40,         // Larger territory
           )
       }
   }
   ```

2. **Export from mod.rs** (`src/entities/types/mod.rs`):
   ```rust
   pub mod rabbit;
   pub mod deer;  // Add this
   ```

3. **Create spawn function** (`src/entities/entity_types.rs`):
   ```rust
   pub fn spawn_deer(
       commands: &mut Commands,
       name: impl Into<String>,
       position: IVec2,
   ) -> Entity {
       let template = EntityTemplate::DEER;
       
       commands.spawn((
           Creature {
               name: name.into(),
               species: template.species.to_string(),
           },
           Deer,
           TilePosition::from_tile(position),
           MovementSpeed::custom(template.movement_speed),
           EntityStatsBundle::default(),
           DeerBehavior::config(), // Attach behavior config
       )).id()
   }
   ```

4. **Update AI planner** (`src/ai/planner.rs`):
   ```rust
   // Add deer query
   deer_query: Query<(Entity, &TilePosition, &Thirst, &BehaviorConfig), With<Deer>>,
   
   // Process deer entities
   for (entity, position, thirst, behavior_config) in deer_query.iter() {
       // ... same planning logic
   }
   ```

### Key Lessons Learned

**Component Dependencies in ECS:**
- Systems query specific component combinations
- Missing ONE component = entity won't be processed
- Use spawn helpers to ensure consistency
- Document required components clearly

**AI System Requirements:**
- Every AI-driven entity needs: `BehaviorConfig`, `TilePosition`, `Thirst`, `Hunger`, `Energy`
- Stats components come from `EntityStatsBundle::default()`
- Marker components (`Rabbit`, `Deer`, etc.) differentiate entity types
- Movement requires: `TilePosition`, `MovementSpeed`, `MovementState` (auto-added)

**Debugging Pattern:**
1. No AI logs? → Check `BehaviorConfig` component
2. Entity exists but inactive? → Verify all required components
3. Spawn not working? → Use spawn helper, don't manually construct
4. New entity type? → Follow the 4-step pattern above

## License

This project is dual-licensed under either:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

## Task Master AI Instructions
**Import Task Master's development workflow commands and guidelines, treat as if import is in the main CLAUDE.md file.**
@./.taskmaster/CLAUDE.md
