# Technical Overview & Developer Guide

This document collects the nuts-and-bolts information that used to live in the README. It explains how the Life Simulator project is organised, how to build and test it, and where to look when you need deeper architecture details.

## Project Structure

```
life-simulator/
├── src/
│   ├── main.rs              # Headless Bevy application entry point
│   ├── lib.rs               # Library exports for integration tests and tooling
│   ├── map_generator.rs     # Standalone map generator binary
│   ├── world_loader.rs      # Loading and management of saved worlds
│   ├── web_server_simple.rs # HTTP server with biomass API and performance endpoints
│   ├── entities/            # ECS components, systems, species registry, fear system
│   │   ├── types/           # Species modules (rabbit, deer, wolf, fox, bear, raccoon)
│   │   ├── fear.rs          # Fear component and predator detection system
│   │   ├── carcass.rs       # Carcass system for nutrient cycling
│   │   └── auto_eat.rs      # Automated eating behavior with fear integration
│   ├── ai/                  # Event-driven AI planners and behavior systems
│   │   ├── predator_toolkit.rs  # Hunting, scent marking, territory management
│   │   ├── event_driven_planner.rs  # Fear-aware decision making
│   │   ├── trigger_emitters.rs     # Environmental and internal event triggers
│   │   └── behaviors/       # Species-specific behavior modules
│   ├── simulation/          # Tick scheduling and simulation resources
│   │   └── profiler.rs      # Performance monitoring and metrics
│   ├── tilemap/             # Chunk-based terrain data
│   ├── vegetation/          # ResourceGrid, LOD system, plant growth simulation
│   │   ├── resource_grid.rs     # High-performance vegetation storage
│   │   ├── chunk_lod.rs          # Level-of-detail rendering system
│   │   ├── constants.rs          # Plant system parameters
│   │   └── benchmark.rs          # Performance testing utilities
│   ├── resources.rs         # Resource layer generation helpers
│   ├── serialization.rs     # RON/world save and load helpers
│   └── web/                 # WebSocket prototype (future work)
├── maps/                    # Generated world files (`.ron`)
├── web-viewer/              # Static viewer with biomass overlay and entity tracking
├── docs/                    # Architecture notes, ADRs, and implementation guides
├── tests/                   # Integration tests, benchmarks, and validation suites
├── scripts/                 # Testing and benchmarking scripts
├── config/                  # Spawn configuration and system parameters
├── Cargo.toml               # Workspace manifest
└── README.md                # High-level project overview (see there first)
```

## Development Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run all tests
cargo test

# Run a specific integration test with log output
cargo test --test pathfinding_test -- --nocapture

# Run the simulator with verbose logging
RUST_LOG=info cargo run --bin life-simulator
```

## Testing Notes

### Integration Tests

The `tests/` directory contains integration tests that exercise full systems against generated worlds:

- `tests/pathfinding_test.rs` — validates the pathfinding grid against real terrain, includes diagnostics for blocked tiles.
- `tests/species_architecture_tests.rs` — confirms the species registry, viewer metadata, and spawn configuration stay in sync as new animals are added.

All integration tests use real RON map files stored under `maps/` so behaviour matches the shipping simulator.

### Debugging Approach

When hunting simulation bugs, prefer writing a focused integration test:

1. Capture a failing scenario in a dedicated test.
2. Use `--nocapture` to stream diagnostics.
3. Fix the system and confirm the test passes.
4. Keep the regression test committed for future safety.

See `PATHFINDING_FIX.md` and `VIEWER_FIX_SUMMARY.md` in `docs/` for worked examples of this workflow.

## Architecture Cheat-Sheet

The simulator is built on Bevy 0.16 running in headless mode, so everything revolves around ECS concepts:

- **Components** carry simulation data (stats, reproduction, positions).
- **Systems** advance time, pick actions, spawn offspring, and stream state to the viewer.
- **Resources** hold global configuration like tick rate and the species registry.
- **Schedules** (Update vs Startup) split once-off setup from per-frame logic.

### Terrain Generation Pipeline

Terrain files are produced by the standalone map generator and loaded by the simulator:

1. **Circular Island Base** — distance-from-centre maths with sine/cosine perturbations produce believable coastlines.
2. **Terrain Zones** — deep water → shallow water → sand → grass → forest transitions for natural beaches.
3. **Chunk Layout** — 16×16 chunks keep memory and streaming manageable while the viewer requests new data.

### Species & Behaviour Modules

Each species lives in `src/entities/types/` and registers:

- Behaviour thresholds (hunger, thirst, follow-mother logic)
- Reproduction settings (gestation, litter sizes, energy gates)
- Viewer metadata (emoji, scaling, colouring)
- Spawn handlers via the species registry (see `src/entities/registry.rs`)

The planner systems (`src/ai/`) pull those descriptors to drive shared mating/follow behaviours across species.

## Further Reading

- `docs/SPECIES_ARCHITECTURE.md` — deep dive into the new species registry + spawn pipeline.
- `docs/CLAUDE.md` — Claude’s debugging notes and historical context.
- `docs/PATHFINDING_FIX.md` — case study refactoring the pathfinding grid.

This document will grow as we modularise more systems. If you move any more technical material out of the README, add the detailed notes here.
