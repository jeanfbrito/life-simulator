# Life Simulator

A headless life simulation game with procedural world generation and web-based visualization, built with Bevy 0.16.

## Overview

This project demonstrates procedural terrain generation and web-based visualization using the Bevy game engine in headless mode. It creates circular island worlds with realistic terrain patterns and provides an interactive HTML viewer for exploration.

## Features

- **Bevy 0.16**: Built with the latest version of the Bevy game engine in headless mode
- **Procedural Terrain Generation**: Circular island generation with realistic beach edges and terrain zones
- **Web-Based Visualization**: Interactive HTML viewer with zoom and pan capabilities
- **HTTP API Server**: RESTful API for terrain data access
- **ECS Architecture**: Entity-Component-System based design
- **Performance Optimized**: Configured for both development and release builds

### Terrain Generation

- **Circular Islands**: Mathematical distance-based island generation
- **Realistic Beaches**: Proper water transitions (Deep Water → Shallow Water → Sand → Land)
- **Natural Variations**: Controlled irregularity using sine/cosine functions
- **Terrain Types**: Deep water, shallow water, sand, grass, forest, desert, dirt, mountains, snow, stone, swamps

## Project Structure

```
life-simulator/
├── src/
│   ├── main.rs              # Main application entry point (headless)
│   ├── lib.rs               # Library exports
│   ├── web_server_simple.rs # HTTP server and terrain generation
│   ├── tilemap/             # Chunk-based terrain system
│   └── web/                 # WebSocket and web components
├── web-viewer/
│   └── viewer.html          # Interactive terrain visualization
├── Cargo.toml               # Project configuration
└── README.md               # This file
```

## Quick Start

### Prerequisites

- Rust 1.70+ (recommended to use [rustup](https://rustup.rs/))
- Git
- A modern web browser

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd life-simulator
```

2. Run the project:
```bash
cargo run
```

3. Open the web viewer:
```bash
# The server starts on http://127.0.0.1:54321
# Open http://127.0.0.1:54321/viewer.html in your browser
```

### Web Viewer Features

- **Interactive Map**: Click and drag to pan around the island
- **Zoom**: Mouse wheel to zoom in/out
- **Terrain Display**: 12 different terrain types with distinct colors
- **Real-time Generation**: Terrain is generated procedurally as you explore
- **Dark Theme**: Optimized for comfortable viewing


## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Code Style

This project follows standard Rust formatting:
```bash
cargo fmt
cargo clippy
```

## Architecture

The project is organized around Bevy's ECS architecture in headless mode:

- **Components**: Data attached to entities
- **Systems**: Logic that operates on components
- **Resources**: Global data and configuration
- **Headless Operation**: No graphics rendering, all visualization via web interface

### Terrain Generation System

The terrain generation uses mathematical algorithms to create realistic islands:

1. **Circular Island Base**
   - Distance-based calculations from center point
   - Controlled irregularity using sine/cosine functions
   - Distinct terrain zones with smooth transitions

2. **Terrain Zones**
   - Deep Water: Outer ocean (#003366)
   - Shallow Water: Coastal transition zone (#4a7ba7)
   - Sand Beach: Island border (#f4e4bc)
   - Land Interior: Various biomes with grass center

3. **Chunk-Based Architecture**
   - 16x16 tile chunks for efficient memory usage
   - Procedural generation on-demand
   - HTTP API for terrain data access

## Dependencies

- `bevy` 0.16 - Main game engine
- `rand` 0.8 - Random number generation

## Performance Configuration

The project includes optimized build configurations:

- Development builds balance compilation speed with performance
- Release builds use LTO (Link Time Optimization) for maximum performance
- Dynamic linking is available for faster development iteration

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

## API Endpoints

The HTTP server provides the following endpoints:

- `GET /viewer.html` - Main terrain viewer interface
- `GET /api/world_info` - World metadata (center chunk, world size)
- `GET /api/chunks?coords=x1,y1&coords=x2,y2` - Terrain data for specified chunks
- `GET /api/chunks?center_x=0&center_y=0&radius=3&layers=true` - Multi-layer terrain data with batched requests

### URL Length Limitations and Batched Requests

When requesting large numbers of chunks (e.g., 7x7 grid = 49 chunks), the URL can become too long and cause `net::ERR_CONNECTION_RESET` errors. The web viewer automatically handles this by:

- **Batch Size**: Requests are split into batches of 10 chunks maximum
- **Automatic Batching**: The viewer splits large requests into multiple smaller requests
- **Data Merging**: Responses from multiple batches are merged before rendering

#### Implementation Details

The chunk request system supports two URL parameter formats:

1. **Legacy Format**: `coords=x,y&coords=x+1,y` (individual chunk coordinates)
2. **Center Format**: `center_x=0&center_y=0&radius=3` (center point and radius)

The server automatically detects and handles both formats for backward compatibility.

### Testing Checklist

Before considering map viewer functionality complete, verify the following:

#### Basic Functionality
- [ ] Server starts successfully on `http://127.0.0.1:54321`
- [ ] Web viewer loads at `http://127.0.0.1:54321/viewer.html`
- [ ] World info API returns correct center chunk and size

#### Terrain Display
- [ ] Complete 7x7 grid loads correctly (49 chunks total)
- [ ] Both terrain and resources layers display properly
- [ ] Chunk boundaries render without artifacts
- [ ] Terrain colors match expected types (water, sand, grass, forest, etc.)

#### Performance and Reliability
- [ ] Batched requests work without connection reset errors
- [ ] Map loads within reasonable time (< 5 seconds)
- [ ] No JavaScript console errors during map loading
- [ ] Edge chunks (outside saved world) show deep water correctly

#### Interactive Features
- [ ] Pan functionality works (click and drag)
- [ ] Zoom functionality works (mouse wheel)
- [ ] Layer toggle (if implemented) works correctly
- [ ] Coordinate display updates correctly during navigation

#### Data Integrity
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

## Future Development

This project serves as a foundation for:

- Advanced life simulation mechanics
- AI-driven entity behavior
- Complex ecosystem interactions
- Multi-user web-based simulation
- Real-time terrain modification

## Documentation

For detailed documentation on specific topics:

- Check inline documentation in the source code
- Refer to Bevy's official documentation for engine-specific questions
- Examine `web_server_simple.rs` for terrain generation algorithms
- Review `web-viewer/viewer.html` for visualization implementation