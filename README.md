# Life Simulator 🦌🐇🦝🦁🐺

Life Simulator is a rich ecosystem simulation where herbivores and predators live, hunt, and evolve on procedurally generated islands. The core sim runs in Bevy 0.16 without graphics; you peek inside through a lightweight web viewer or by calling the HTTP API.

## ✨ What makes this special

- **Dynamic Predator-Prey Ecosystems** – Wolves hunt rabbits, foxes patrol territories, while bears dominate the landscape. Herbivores exhibit realistic fear responses including fleeing, reduced feeding, and altered grazing patterns.
- **Advanced Plant System** – Real-time vegetation growth with ResourceGrid technology, biomass density visualization, and ecosystem-wide nutrient cycling.
- **Complex Animal Behaviors** – Animals form pairs, carry pregnancies, birth litters, exhibit fear responses, and adapt their behavior based on predator presence and resource availability.
- **Modular Species Architecture** – Each species (rabbits, deer, raccoons, wolves, foxes, bears) has its own module with configurable behaviors, reproduction strategies, and AI patterns.
- **Living Worlds** – Vegetation grows, gets consumed, and regenerates based on grazing pressure. Carcasses from predators enrich the soil, creating realistic ecosystem dynamics.
- **Real-time Web Visualization** – Watch the ecosystem evolve through an interactive web viewer with biomass overlays, entity tracking, and live statistics.

## Tour of the experience

| Feature | What you'll see |
| --- | --- |
| 🗺️ Living Islands | Sand-lined beaches leading into forests, mountains, and ponds, with dynamic vegetation that responds to grazing pressure. |
| 🐇 Rabbits | Fast-reproducing herbivores that graze aggressively and exhibit realistic fear responses when predators are nearby. |
| 🦌 Deer | Majestic creatures whose fawns trail their mothers until adulthood, with cautious grazing behaviors. |
| 🦝 Raccoons | Opportunistic foragers that balance thirst and hunger while staying alert to danger. |
| 🐺 Wolves | Pack hunters that patrol territories, mark scents, and create coordinated hunting strategies. |
| 🦊 Foxes | Cunning solitary hunters that use scent tracking and territory patrol techniques. |
| 🐻 Bears | Powerful apex predators that dominate territories and create fear responses across the ecosystem. |
| 🌿 Biomass Visualization | Real-time grass density overlay showing vegetation health and grazing patterns. |
| 🌐 Advanced Web Viewer | Interactive HTML/JS viewer with entity tracking, fear response visualization, and ecosystem statistics. |

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
   - Drag to pan, scroll to zoom, click entities to inspect stats
   - Toggle biomass overlay to see vegetation density
   - Watch fear responses as predators hunt herbivores

The demo spawn config introduces a complete ecosystem with herbivores and predators so you can immediately watch dynamics unfold. Toggle `CONFIG.showGrassDensity` in the viewer to see real-time biomass visualization. Tweak `config/spawn_config.ron` to customize species, names, or behavior.

## Under the hood (at a glance)

### Core Systems
- **Headless Bevy** runs the ECS, ticking stats, movement, AI planners, and reproduction systems at 10 TPS.
- **ResourceGrid Vegetation System** - High-performance vegetation storage with Level-of-Detail (LOD) rendering.
- **Fear & Predator System** - Spatial predator detection, fear propagation, and behavioral response mechanisms.
- **Event-Driven AI** - Trigger-based decision making with FearTrigger, PredatorScent, and environmental events.
- **Modular Species Architecture** - Each species has its own module with configurable behaviors and reproduction strategies.

### Key Technologies
- **Species Registry** (`src/entities/registry.rs`) - Manages spawn handlers, viewer metadata, and behaviors.
- **AI Planners** (`src/ai/`) - Event-driven planning system with fear-aware decision making.
- **Predator Toolkit** (`src/ai/predator_toolkit.rs`) - Scent marking, territory detection, prey tracking.
- **Web Viewer API** (`web_server_simple.rs`) - Real-time HTTP API serving entities, species, vegetation, and performance data.

Curious how it all works? Dive into the docs:

### 📚 Documentation
- [Species Reference Guide](docs/SPECIES_REFERENCE.md) – Complete species catalog with behaviors, stats, and reproduction
- [Plant System Parameters](docs/PLANT_SYSTEM_PARAMS.md) – Vegetation growth, ResourceGrid, and ecosystem configuration
- [Event-Driven Planner Implementation](docs/EVENT_DRIVEN_PLANNER_IMPLEMENTATION.md) – AI architecture and fear systems
- [Technical Overview & Developer Guide](docs/TECH_OVERVIEW.md) – project layout, build commands, testing tips
- `docs/` folder – ADRs, design notes, and implementation journals

### 🧪 Testing & Validation
- Comprehensive test suite covering plant system, predator-prey dynamics, and AI behaviors
- Performance benchmarking scripts for vegetation system validation
- Integration tests for fear mechanics and ecosystem interactions

## Extending the sim

Want to add new species, behaviors, or ecosystem features?

### 🦁 Adding New Species
1. **Create Species Module** - Copy an existing module in `src/entities/types/` (rabbit, wolf, etc.)
2. **Define Behaviors** - Implement hunting, grazing, fear responses, and reproduction strategies
3. **Configure Stats** - Set movement speeds, reproduction rates, fear thresholds, and AI parameters
4. **Register Species** - Add to species registry with spawn functions and viewer metadata
5. **Update Configuration** - Add to `config/spawn_config.ron` for demo spawning

### 🌿 Extending Plant System
1. **Modify ResourceGrid** - Add new resource types or growth patterns in `src/vegetation/`
2. **Update Constants** - Adjust growth rates, biomass thresholds, and ecosystem parameters
3. **Add New Behaviors** - Create actions that interact with vegetation (eating, trampling, etc.)
4. **Update Visualization** - Extend biomass overlay and web viewer for new features

### 🧠 Advanced AI Features
1. **Event System** - Add new trigger types and event emitters in `src/ai/trigger_emitters.rs`
2. **Behavior Trees** - Create complex action sequences and decision trees
3. **Fear System** - Extend fear propagation and response mechanisms
4. **Performance Optimization** - Add spatial indexing and caching systems

Run `cargo check` + `cargo test` to validate changes. The modular architecture isolates species-specific logic from core systems.

Happy simming! 🐾

## 🌐 API Endpoints

The HTTP server provides comprehensive real-time data access:

### Core World Data
- `GET /api/world_info` - World metadata (name, seed, chunk count, bounds)
- `GET /api/world/current` - Current loaded world details
- `GET /api/worlds` - List all available generated worlds
- `POST /api/world/select` - Switch to different world

### Entity & Species Data
- `GET /api/entities` - Real-time entity positions, stats, and behaviors
- `GET /api/species` - Species metadata with behaviors and reproduction info

### 🌿 Vegetation & Ecosystem
- `GET /api/vegetation/biomass` - Real-time biomass density heatmap
- `GET /api/vegetation/performance` - Vegetation system performance metrics
- `GET /api/vegetation/memory` - Memory usage analysis for ResourceGrid
- `GET /api/vegetation/stats` - Ecosystem statistics and health metrics
- `GET /api/vegetation/metrics` - Comprehensive performance dashboard

### 📊 Performance & Benchmarking
- `GET /api/vegetation/benchmark/quick` - Run quick performance benchmark
- `GET /api/vegetation/benchmark/phase4` - Comprehensive system benchmark
- `GET /api/vegetation/benchmark/current` - Current performance rating
- `GET /api/vegetation/benchmark/history` - Historical performance trends

### Terrain Data
- `GET /viewer.html` - Interactive web viewer with biomass overlays
- `GET /api/chunks?coords=x1,y1&coords=x2,y2` - Terrain data for specific chunks
- `GET /api/chunks?center_x=0&center_y=0&radius=3&layers=true` - Multi-layer data with batching

## 🚀 Performance Features

### Optimized Build Configurations
- **Development**: Balanced compilation speed with performance
- **Release**: LTO (Link Time Optimization) for maximum performance
- **Dynamic Linking**: Faster development iteration

### Real-time Optimizations
- **ResourceGrid LOD**: Level-of-detail system for efficient vegetation rendering
- **Spatial Indexing**: Optimized fear detection and predator-prey calculations
- **Event-driven AI**: Efficient decision making with trigger-based updates
- **Batched API Requests**: Prevents connection timeouts for large data requests

### API Usage Examples

```bash
# List available worlds
curl http://127.0.0.1:54321/api/worlds

# Select different world
curl -X POST http://127.0.0.1:54321/api/world/select \
  -H "Content-Type: application/json" \
  -d '{"world_name": "my_world"}'

# Get real-time biomass data
curl http://127.0.0.1:54321/api/vegetation/biomass

# Run performance benchmark
curl http://127.0.0.1:54321/api/vegetation/benchmark/quick
```

## 📦 Dependencies

- `bevy` 0.16 - Main game engine with ECS and networking
- `rand` 0.8 - Random number generation for deterministic worlds
- `serde` & `ron` - Data serialization for world files
- `tokio` - Async runtime for web server functionality

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository** and create a feature branch
2. **Understand the Architecture** - Read the documentation in `docs/`
3. **Run Tests** - Ensure `cargo test` passes before submitting
4. **Add Tests** - Include tests for new features and behaviors
5. **Update Documentation** - Keep docs current with code changes
6. **Submit Pull Request** - With clear description of changes

### Development Workflow
```bash
# Generate test world
cargo run --bin map_generator -- --name dev_world

# Run with debug logging
RUST_LOG=info cargo run --bin life-simulator

# Run specific test modules
cargo test --test predator_fear_test
cargo test --test resource_grid_benchmark
```

## 📄 License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

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
