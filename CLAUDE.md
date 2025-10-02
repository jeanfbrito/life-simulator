# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Life Simulator is a headless life simulation game built with Bevy 0.16 that features a **separated architecture** where world generation is completely independent from the running simulation engine. Maps are generated as a separate CLI step, and the life simulator loads pre-generated worlds for consistent, reproducible simulations.

## Prerequisites

- Rust 1.70+ (recommended to use [rustup](https://rustup.rs/))
- Git
- A modern web browser

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

### Web Viewer
```bash
# Start the simulator (web server starts automatically on port 54321)
cargo run --bin life-simulator

# Access the web viewer
# Open http://127.0.0.1:54321/viewer.html in your browser
```

#### Web Viewer Features
- **Interactive Map**: Click and drag to pan around the island
- **Zoom**: Mouse wheel to zoom in/out
- **Terrain Display**: 12 different terrain types with distinct colors
- **Dark Theme**: Optimized for comfortable viewing

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
map_generator → RON files (maps/) → WorldLoader → CachedWorld (in-memory) → web_server_simple → Web Viewer
```

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

#### Web Components
- **`src/web_server_simple.rs`**: HTTP server with multi-threaded request handling
- **`web-viewer/viewer.html`**: Interactive map viewer with pan/zoom
- **`web-viewer/js/`**: Modular JavaScript for rendering, networking, controls

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

## Web Viewer Architecture

The viewer is built with modular JavaScript:
- **`js/config.js`**: Terrain colors, tile size constants
- **`js/network.js`**: API communication and chunk fetching
- **`js/chunk-manager.js`**: Chunk caching and loading logic
- **`js/renderer.js`**: Canvas rendering with pan/zoom
- **`js/controls.js`**: Mouse input handling
- **`js/app.js`**: Main application initialization
- **`js/entity-manager.js`**: Entity polling and management (200ms interval)

Rendering pattern: Fetch chunks in batches → Cache locally → Render visible area to canvas

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
- **Missing world files**: Always run map_generator before simulator
- **Port conflicts**: Web server runs on 54321, ensure it's available
- **URL length limits**: Large chunk requests need batching (handled by viewer)
- **Chunk key format**: Always use "x,y" string format, not "(x,y)" or other variants
- **Layer access**: Use multi-layer methods when working with resources or future layers

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

Before considering map viewer functionality complete, verify:

### Basic Functionality
- [ ] Server starts successfully on `http://127.0.0.1:54321`
- [ ] Web viewer loads at `http://127.0.0.1:54321/viewer.html`
- [ ] World info API returns correct center chunk and size

### Terrain Display
- [ ] Complete 7×7 grid loads correctly (49 chunks total)
- [ ] Both terrain and resources layers display properly
- [ ] Chunk boundaries render without artifacts
- [ ] Terrain colors match expected types (water, sand, grass, forest, etc.)

### Performance and Reliability
- [ ] Batched requests work without connection reset errors
- [ ] Map loads within reasonable time (< 5 seconds)
- [ ] No JavaScript console errors during map loading
- [ ] Edge chunks (outside saved world) show deep water correctly

### Interactive Features
- [ ] Pan functionality works (click and drag)
- [ ] Zoom functionality works (mouse wheel)
- [ ] Layer toggle (if implemented) works correctly
- [ ] Coordinate display updates correctly during navigation

### Data Integrity
- [ ] Saved world data matches displayed terrain
- [ ] Resources layer data loads correctly when `layers=true` parameter is used
- [ ] Chunk coordinates are calculated correctly from center point
- [ ] No missing or corrupted chunks in the displayed area

## References and Inspiration

- [Bevy Game Engine](https://bevyengine.org/) - The game engine powering this project
- `/Users/jean/Github/world-simulator` - Terrain generation inspiration
- `/Users/jean/Github/dogoap` - AI and behavior tree reference
- `/Users/jean/Github/big-brain` - AI planning and decision-making reference
- `/Users/jean/Github/bevy_entitiles` - Tile-based entity system reference
- Procedural content generation techniques for realistic island formation

## Future Development Ideas

This project serves as a foundation for:
- Advanced life simulation mechanics
- AI-driven entity behavior
- Complex ecosystem interactions
- Multi-user web-based simulation
- Real-time terrain modification

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
- **Oxygen Not Included**: 5 TPS with sub-tick physics

## License

This project is dual-licensed under either:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
