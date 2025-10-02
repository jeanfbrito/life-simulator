# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Life Simulator is a headless life simulation game built with Bevy 0.16 that features a **separated architecture** where world generation is completely independent from the running simulation engine. Maps are generated as a separate CLI step, and the life simulator loads pre-generated worlds for consistent, reproducible simulations.

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

Rendering pattern: Fetch chunks in batches → Cache locally → Render visible area to canvas

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
