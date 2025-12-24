# Life Simulator Web Viewer

A web-based viewer for visualizing the procedurally generated terrain from the Life Simulator backend.

## Features

- **Real-time Terrain Rendering**: Displays 16x16 chunks with various terrain types
- **Interactive Controls**: Zoom, pan, and regenerate world
- **Terrain Types**: 12 different terrain types with visual representations:
  - 🌱 Grass, 💧 Water, 🌊 Deep Water, 🏖️ Sand
  - 🪨 Stone, 🌲 Forest, 🏔️ Mountain, ❄️ Snow
  - 🏜️ Desert, 🐊 Swamp, 🟫 Dirt
- **Procedural Generation**: Seed-based world generation with natural patterns
- **Live Statistics**: Real-time terrain composition analysis

## Usage

### Running the Full System

1. **Start the Rust Backend**:
   ```bash
   cd /path/to/life-simulator
   cargo run
   ```

2. **Open the Viewer**:
   Open your web browser and navigate to:
   ```
   http://localhost:8080/viewer.html
   ```

### Running with a Simple Server (for development)

If you want to test the viewer without running the full Rust backend, you can use any local HTTP server:

```bash
# Python 3
python -m http.server 8080 --directory web-viewer

# Node.js
npx http-server -p 8080 -c-1 web-viewer

# PHP (built-in)
php -S localhost:8080 -t web-viewer
```

Then open `http://localhost:8080/viewer.html` in your browser.

## Configuration

All configuration values can be found in `js/config.js` and can be customized for different performance and behavior requirements:

### Grid and Rendering
- `TILE_SIZE` (8px) - Size of each tile on screen, affected by zoom level
- `VIEW_SIZE_X` / `VIEW_SIZE_Y` (100) - Viewport size in tiles
- `CHUNK_SIZE` (16) - Tiles per chunk (16x16 tiles per chunk)

### Performance Tuning
- `ENTITY_POLL_INTERVAL_MS` (500ms) - How often to fetch entity updates from server
- `TOOLTIP_THROTTLE_MS` (100ms) - Tooltip update frequency to reduce DOM operations
- `CHUNK_LOAD_DEBOUNCE_MS` (100ms) - Delay before loading visible chunks after panning
- `BIOMASS_FETCH_INTERVAL_MS` (5000ms) - How often to fetch vegetation biomass data

### Network Settings
- `CHUNKS_PER_REQUEST` (10) - Max chunks to fetch in single API request to avoid URL length issues
- `FETCH_TIMEOUT_MS` (5s) - Default timeout for API requests
- `CHUNK_FETCH_TIMEOUT_MS` (10s) - Longer timeout for chunk loading due to larger data

### Camera and Controls
- `PAN_SMOOTHING_FACTOR` (0.2) - Camera catch-up speed while dragging (0-1, lower = faster)
- `INERTIA_FRICTION` (0.90) - Velocity decay after releasing drag (0-1, lower = faster deceleration)
- `MIN_INERTIA_SPEED` (0.15) - Minimum speed before inertia stops
- `ZOOM_MULTIPLIER` (1.25) - Zoom step size (25% per zoom level)
- `MIN_ZOOM` (0.25) - Minimum zoom level
- `MAX_ZOOM` (4.0) - Maximum zoom level

### Reliability
- `MAX_FAILURES` (5) - Number of failures before circuit breaker opens
- `MAX_BACKOFF_INTERVAL_MS` (10000ms) - Maximum retry delay during exponential backoff

## Controls

### Keyboard Controls
- **Arrow Keys / WASD**: Pan the map
- **+/-**: Zoom in/out
- **R**: Regenerate new world

### Mouse Controls
- **Hover**: Shows terrain and coordinate information
- **Middle Drag**: Pan the map with inertia
- **Click**: (No functionality yet - future features planned)

### UI Controls
- **🔍 Zoom In/Out**: Zoom controls with CONFIG.ZOOM_MULTIPLIER steps
- **🔄 Reset View**: Reset to default zoom and position
- **🌱 Show Grass Density**: Toggle biomass visualization overlay
- **📊 Stats**: Real-time rendering statistics

## API Endpoints

The backend provides REST API endpoints:

- `GET /api/world_info`: Returns world configuration and center chunk
- `GET /api/chunks?coords=x1,y1&coords=x2,y2`: Returns terrain data for specified chunks

## Terrain Generation Algorithm

The terrain is generated using:
- **Noise-based patterns**: Sine/cosine functions for natural-looking terrain
- **Distance-based features**: Deep water at origin, mountains at edges
- **Procedural seeding**: Each chunk has a unique seed based on coordinates
- **Pattern mixing**: Multiple noise patterns combined for variety

## Technical Details

- **Tile Size**: 8x8 pixels
- **Chunk Size**: 16x16 tiles (128x128 pixels)
- **View Size**: 100x100 tiles (800x800 pixels)
- **Color Mapping**: Each terrain type has a distinct color for easy identification

## Future Enhancements

- **Entity Visualization**: Show entities moving on the map
- **Real-time Updates**: WebSocket connection for live simulation data
- **Pathfinding Visualization**: Show calculated paths between points
- **Resource Indicators**: Display resource availability on terrain
- **Biome Information**: Show biome types and properties
- **Mini-map**: Overview of the entire world

## Architecture

The viewer consists of:

1. **Rust Backend** (`src/web_server.rs`): HTTP server with terrain generation
2. **HTML Frontend** (`web-viewer/viewer.html`): Interactive web interface
3. **API Layer**: RESTful API for data exchange
4. **Rendering Engine**: Canvas-based 2D rendering with zoom/pan controls