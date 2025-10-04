# Life Simulator 🦌🐇🦝

Life Simulator is a cozy headless world where rabbits, deer, raccoons (and more to come) eat, drink, sleep, and raise their young on handcrafted islands. The core sim runs in Bevy 0.16 without graphics; you peek inside through a lightweight web viewer or by calling the HTTP API.

## Why you might love this project

- **Watch ecosystems evolve** – animals form pairs, carry pregnancies, birth litters, and juveniles imprint on their mothers.
- **Plug-and-play species** – reproduction, AI planners, and viewer metadata all come from per-species modules, so cloning behaviour is as easy as cloning a descriptor.
- **Data-driven bootstrapping** – demo spawns, names, and logging live in a RON config you can tweak without recompiling.
- **Shareable worlds** – generate islands once with the map CLI, then replay the same world forever or ship it to a friend.
- **Headless & scriptable** – everything streams over HTTP/websocket, perfect for dashboards, bots, or further automation.

## Tour of the experience

| Feature | What you'll see |
| --- | --- |
| 🗺️ Island worlds | Sand-lined beaches leading into forests, mountains, and ponds, all generated with deterministic maths. |
| 🐇 Rabbits | Fast-reproducing herbivores that graze aggressively and form pairs quickly. |
| 🦌 Deer | Slower, majestic creatures whose fawns trail their mothers until adulthood. |
| 🦝 Raccoons | Opportunistic foragers that balance thirst and hunger before seeking mates. |
| 🌐 Web viewer | A HTML/JS viewer (in `web-viewer/viewer.html`) that queries `127.0.0.1:54321` for terrain and entity data. |

## Quick start (5 minutes)

1. **Clone & enter the project**
   ```bash
   git clone <repository-url>
   cd life-simulator
   ```

2. **Generate an island** – this produces `maps/generated_world.ron`.
   ```bash
   cargo run --bin map_generator
   # or customise it
   cargo run --bin map_generator -- --name spring_isle --seed 123456 --radius 12
   ```

3. **Run the sim**
   ```bash
   cargo run --bin life-simulator
   ```

4. **Open the viewer**
   - Visit http://127.0.0.1:54321/viewer.html
   - Drag to pan, scroll to zoom, click an entity to inspect its stats.

The demo spawn config will introduce rabbits near the origin, a deer pair, and a raccoon couple so you can immediately watch behaviour unfold. Tweak `config/spawn_config.ron` to change names, counts, or logging.

## Under the hood (at a glance)

- **Headless Bevy** runs the ECS, ticking stats, movement, AI planners, and reproduction systems.
- **Species registry** lives in `src/entities/registry.rs`, exposing spawn handlers, viewer metadata, and default behaviours.
- **AI planners** reuse shared herbivore logic, with species contributing thresholds and follow/mate preferences.
- **Viewer API** (`web_server_simple.rs`) serves `/api/entities`, `/api/species`, and chunk endpoints for the browser client.

Curious how it all works? Dive into the docs:

- [Species Architecture](docs/SPECIES_ARCHITECTURE.md) – registry, planner hooks, viewer metadata
- [Technical Overview & Developer Guide](docs/TECH_OVERVIEW.md) – project layout, build commands, testing tips
- `docs/` folder – fix-it journals and design notes accumulated during development

## Extending the sim

Want to introduce a new animal or tweak behaviour?

1. Copy an existing module in `src/entities/types/` and adjust stats + reproduction config.
2. Register it in the species registry (spawn fn + metadata) and spawn config.
3. Add viewer styling via the metadata returned from `/api/species`.
4. Run `cargo check` + `cargo test` (see `docs/TECH_OVERVIEW.md` for details).

The modular layout keeps species-specific logic out of engine code, letting you iterate quickly without touching the core systems.

Happy simming! 🐾
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

### World Management
- `GET /api/world_info` - Current world information (name, seed, chunk count, bounds)
- `GET /api/world/current` - Current loaded world details
- `GET /api/worlds` - List all available generated worlds
- `POST /api/world/select` - Switch to a different world (JSON: `{"world_name": "my_world"}`)

### Terrain Data
- `GET /viewer.html` - Main terrain viewer interface
- `GET /api/chunks?coords=x1,y1&coords=x2,y2` - Terrain data for specified chunks
- `GET /api/chunks?center_x=0&center_y=0&radius=3&layers=true` - Multi-layer terrain data with batched requests

### World Selection Usage

```bash
# List available worlds
curl http://127.0.0.1:54321/api/worlds

# Select a different world
curl -X POST http://127.0.0.1:54321/api/world/select \
  -H "Content-Type: application/json" \
  -d '{"world_name": "my_world"}'
```

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

### General Documentation
- Check inline documentation in the source code
- Refer to Bevy's official documentation for engine-specific questions
- Examine `web_server_simple.rs` for terrain generation algorithms
- Review `web-viewer/viewer.html` for visualization implementation

### Debugging & Fix Documentation
- **`PATHFINDING_FIX.md`** - Complete pathfinding bug diagnosis and fix
  - Problem analysis with test-driven approach
  - Root cause identification (diagonal movement disabled)
  - Integration test creation for real-world validation
  - Before/after comparison with metrics
  - Recommendations for future improvements

### Key Lessons Learned

#### Pathfinding System (2025-01-02)
**Problem**: Entities couldn't reach water despite water existing in world

**Root Cause**: Diagonal movement disabled in all pathfinding requests (`allow_diagonal: false`)

**Solution**: Enable 8-directional pathfinding by setting `allow_diagonal: true`

**Impact**: Improved path success rate from 0-25% to 75-100%

**Files Modified**:
- `src/ai/action.rs` (DrinkWaterAction, WanderAction)
- `src/entities/wandering.rs` (Wanderer AI)
- `tests/pathfinding_test.rs` (Integration test)

**Testing Approach**:
1. Created integration test loading real world data
2. Built pathfinding grid matching simulation
3. Tested multiple spawn points to water sources
4. Analyzed failure points with terrain sampling
5. Validated fix with measurable improvement

**Takeaway**: Test-driven debugging with real world data revealed issues that weren't obvious from logs alone. Integration tests that mirror production environment are invaluable for complex systems.
