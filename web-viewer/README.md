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

## Controls

### Keyboard Controls
- **Arrow Keys / WASD**: Pan the map
- **+/-**: Zoom in/out
- **R**: Regenerate new world

### Mouse Controls
- **Hover**: Shows terrain and coordinate information
- **Click**: (No functionality yet - future features planned)

### UI Controls
- **🔍 Zoom In/Out**: Zoom controls
- **🔄 Reset View**: Reset to default zoom and position
- **🌍 New World**: Generate a new world with random coordinates

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