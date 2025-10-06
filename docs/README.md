# Life Simulator Documentation

This directory contains comprehensive documentation for the life-simulator project, with a focus on the tick-based simulation system.

## 📚 Documents

### [TICK_SYSTEM_FIXES.md](./TICK_SYSTEM_FIXES.md)
**Comprehensive issue documentation and architecture guide**

Documents all the problems encountered while implementing the tick-based simulation system, their root causes, and the solutions applied.

**Key Topics:**
- Issue #1: Entities not moving despite tick system running
- Issue #2: Movement speed configuration
- Issue #3: Headless environment missing core plugins
- Issue #4: Simulation speed changes not applying
- Final architecture and design patterns
- Lessons learned and future considerations

**Read this if you want to understand:**
- Why entities weren't moving initially
- How the tick accumulator works
- The difference between `FixedUpdate` and manual tick control
- Why `should_tick` flag pattern was chosen

---

### [TESTING_TICK_SYSTEM.md](./TESTING_TICK_SYSTEM.md)
**Testing guide and verification procedures**

Explains how to test and verify the tick-based simulation system, including manual testing procedures, performance verification, and troubleshooting.

**Key Topics:**
- Manual testing procedures
- Expected behavior and timing
- Performance testing
- Troubleshooting common issues
- CI/CD integration (future)

**Read this if you want to:**
- Verify the tick system is working correctly
- Test entity movement timing
- Monitor simulation performance
- Debug tick-related issues

---

## 🧪 Testing

### Phase 5 Scenario Testing

```bash
# Run comprehensive scenario tests
cargo test --test phase5_scenario_tests -- --nocapture
```

The Phase 5 scenario testing framework validates ecosystem dynamics:

- **Ungrazed Regrowth**: Tests vegetation recovery without herbivore pressure
- **Herbivore-Only**: Tests rabbit-only scenarios for grazing balance
- **Predator-Herbivore**: Tests ecosystem stability with foxes added
- **Performance Validation**: Ensures vegetation system stays within CPU budgets

### Quick Verification

```bash
# Run the automated movement test
./scripts/test_movement.sh

# Check Phase 4 performance benchmarks
./scripts/phase4_benchmark.sh
```

These scripts will:
1. Start the simulation
2. Track entity movement for 30 seconds
3. Analyze movement patterns
4. Run vegetation performance benchmarks
5. Report success or failure
6. Clean up processes

Expected output: Entities should move every ~3 seconds (30 ticks at 10 TPS).

### Manual Testing

```bash
# Terminal 1: Start simulation
cargo run --bin life-simulator

# Terminal 2: Monitor entities
watch -n 2 'curl -s http://127.0.0.1:54321/api/entities | jq ".entities[] | {name, position}"'

# Terminal 3: Monitor vegetation metrics
watch -n 5 'curl -s http://127.0.0.1:54321/api/vegetation/metrics | jq'
```

---

### [PLANT_SYSTEM_PLAN.md](./PLANT_SYSTEM_PLAN.md)
**Comprehensive plant system architecture and Phase 5 implementation**

Documents the complete vegetation system architecture from Phase 1-5, including growth mechanics, herbivore interactions, and the new metrics dashboard.

**Key Topics:**
- Vegetation grid system and terrain-based growth
- Logistic growth model with carrying capacity
- Herbivore grazing and biomass consumption
- Phase 5 metrics dashboard and debugging tools
- Performance optimizations (Phase 4)
- Scenario testing framework

**Read this if you want to understand:**
- How plants grow and spread across the world
- Herbivore-vegetation interactions and ecosystem balance
- Phase 5 debugging and monitoring capabilities
- Performance characteristics of the vegetation system

---

### [SPECIES_REFERENCE.md](./SPECIES_REFERENCE.md)
**Species behavior guide with biomass consumption mapping**

Complete reference for all species in the simulator, including reproduction, behavior, and Phase 5 biomass consumption patterns.

**Key Topics:**
- Species-specific behavior parameters and reproduction
- Movement speeds and foraging patterns
- Biomass consumption per feeding action
- AI decision making and priorities
- Spawn configurations and viewer metadata

**Read this if you want to:**
- Understand species behavior and balance
- Add new species to the simulation
- Calculate ecosystem impact of herbivores
- Configure species spawning and behavior

---

## 🏗️ Architecture Overview

### Tick System Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Update Schedule (Every Frame)            │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  accumulate_ticks (runs every frame)                 │  │
│  │  ├─ Reads: Time, SimulationSpeed                     │  │
│  │  ├─ Accumulates delta time                           │  │
│  │  ├─ Calculates available ticks                       │  │
│  │  └─ Sets: should_tick flag                           │  │
│  └──────────────────────────────────────────────────────┘  │
│                            ↓                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Tick-based Systems (run_if should_tick == true)     │  │
│  │  ├─ wanderer_ai_system                               │  │
│  │  ├─ movement_tick_system                             │  │
│  │  ├─ tick_stats_system                                │  │
│  │  └─ death_system                                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Key Resources

- **`SimulationTick`**: Current tick counter (u64)
- **`TickAccumulator`**: Accumulates frame time into ticks
- **`SimulationState`**: Contains `should_tick` flag
- **`SimulationSpeed`**: Multiplier for tick rate (1.0 = normal, 2.0 = 2x)

### Timing

```
Base TPS = 10
Base Tick Duration = 100ms

Movement at 30 ticks per tile:
Normal speed (1.0x): 30 ticks × 100ms = 3000ms = 3 seconds
Fast speed (2.0x):   30 ticks × 50ms  = 1500ms = 1.5 seconds
```

---

## 🌱 Vegetation System ✅ **COMPLETED**

The vegetation system has been fully rewritten from a dense tile-by-tile system to a sparse, event-driven architecture.

### New Architecture (Phases 1-6 Complete)

**🔄 Event-Driven ResourceGrid**: Sparse storage that only tracks cells with biomass
**📊 Level-of-Detail Management**: Chunk-based proximity optimization (Hot/Warm/Cold)
**🌡️ On-Demand Heatmaps**: Refresh only when data changes, with dirty flag pattern
**📈 Performance Monitoring**: Real-time metrics dashboard and API endpoints

### Key Components

- **ResourceGrid**: Sparse hash map storing only active vegetation cells
- **ChunkLODManager**: Proximity-based chunk activation and aggregation
- **HeatmapRefreshManager**: On-demand heatmap generation with caching
- **VegetationScheduler**: Event-driven regrowth and consumption scheduling

### Performance Characteristics

- **Scalability**: CPU usage scales with grazing activity, not world size
- **Memory Efficiency**: Only stores vegetation cells that contain biomass
- **API Performance**: Heatmap generation <5ms, metrics <1ms
- **LOD Optimization**: Agent proximity reduces processing by 70%+ in clustered scenarios

### Documentation

See: [Vegetation System Rewrite Plan](VEGETATION_REWRITE_PLAN.md) for complete implementation details and Phase 6 cleanup results.

---

## 🐛 Troubleshooting

### Entities not moving?

1. Check if simulation is paused (press Space)
2. Verify ticks incrementing: `tail -f /tmp/life-simulator.log`
3. Check API: `curl http://127.0.0.1:54321/api/entities`
4. Run test script: `./scripts/test_movement.sh`

### Ticks not accumulating?

1. Verify `MinimalPlugins` added to app
2. Check `should_tick` flag being set
3. Ensure `accumulate_ticks` runs every frame
4. Look for errors in `/tmp/life-simulator.log`

### Inconsistent tick rate?

1. Check CPU usage (might be overloaded)
2. Verify no infinite loops in systems
3. Check tick duration metrics in logs
4. Reduce entity count if needed

---

## 📈 Performance Metrics

### Simulation Performance

Monitor these in the simulation logs:

- **TPS (Ticks Per Second)**: Should stay near 10.0
- **Avg Tick Duration**: Should be under 10ms
- **Frame Time**: Independent of tick time
- **Entity Count**: Track active entities

Example log output:
```
🎯 Tick #100 | TPS: 10.0 | Avg duration: 2.5ms
🎯 Tick #200 | TPS: 10.1 | Avg duration: 2.3ms
```

### Vegetation System Metrics (Phase 5)

Monitor vegetation system via API endpoint `/api/vegetation/metrics`:

- **Total Biomass**: Current vegetation biomass across all tiles
- **Active Tiles**: Number of tiles undergoing growth updates
- **Depleted Tiles**: Tiles below biomass threshold
- **Average Biomass %**: Percentage of carrying capacity
- **Growth vs Consumption**: Net biomass change balance
- **Trend Analysis**: Increasing/Decreasing/Stable biomass trends

Example metrics output:
```json
{
  "total_suitable_tiles": 15000,
  "active_tiles": 2450,
  "depleted_tiles": 180,
  "total_biomass": 875432.5,
  "average_biomass_pct": 73.2,
  "total_consumed": 1234.8,
  "total_grown": 1456.3,
  "trend": "Increasing"
}
```

---

## 🎯 Quick Reference

### Common Commands

```bash
# Run simulation
cargo run --bin life-simulator

# Run Phase 5 scenario tests
cargo test --test phase5_scenario_tests -- --nocapture

# Run movement test
./scripts/test_movement.sh

# Run Phase 4 vegetation benchmarks
./scripts/phase4_benchmark.sh

# Monitor entities API
curl http://127.0.0.1:54321/api/entities | jq

# Monitor vegetation metrics (Phase 5)
curl http://127.0.0.1:54321/api/vegetation/metrics | jq

# Monitor vegetation performance
curl http://127.0.0.1:54321/api/vegetation/performance | jq

# Watch logs
tail -f /tmp/life-simulator.log | grep "Tick #"

# Check tick rate
curl http://127.0.0.1:54321/api/simulation | jq '.current_tick'
```

### Speed Controls (while running)

- `Space` = Pause/Resume
- `1` = 0.5x speed
- `2` = 1.0x speed (normal)
- `3` = 2.0x speed
- `4` = 3.0x speed

---

## 🔗 Related Files

### Core Implementation
- `src/simulation/tick.rs` - Tick system implementation
- `src/simulation/mod.rs` - Simulation plugin and tick accumulation
- `src/entities/movement.rs` - Entity movement system
- `src/entities/wandering.rs` - Wandering AI

### Configuration
- `src/main.rs` - App setup and entity spawning
- `src/entities/entity_types.rs` - Entity templates and speeds

### Testing
- `scripts/test_movement.sh` - Automated movement verification
- `tests/test_utils.rs` - Test utilities (WIP)
- `tests/tick_system_tests.rs` - Integration tests (WIP)

---

## 📝 Change Log

### 2025-10-05: Phase 5 Implementation - Scenario Tuning & Regression Suite
- **Scenario Testing Framework**: Comprehensive ecosystem validation with ungrazed regrowth, herbivore-only, and predator-herbivore scenarios
- **Metrics Dashboard**: Real-time vegetation monitoring with biomass tracking, depleted tile analysis, and trend identification
- **Performance Integration**: Phase 5 metrics integrated with existing Phase 4 performance optimizations
- **Documentation Updates**: Added biomass consumption mapping to species reference and comprehensive plant system overview
- **API Enhancements**: New `/api/vegetation/metrics` endpoint for real-time monitoring
- **Testing Infrastructure**: Automated scenario tests with configurable validation criteria

### 2025-10-02: Tick System Fixes
- Fixed entities not moving (missing `should_tick` condition)
- Implemented manual tick accumulation
- Moved from `FixedUpdate` to `Update` with explicit run conditions
- Added `should_tick` flag to `SimulationState`
- Adjusted human movement speed to 30 ticks per tile (3 seconds)
- Added comprehensive documentation
- Created automated test scripts

---

## 🚀 Future Improvements

1. **Tick Budgeting**: Limit max ticks per frame to prevent spiral of death
2. **Position Interpolation**: Smooth visual movement between tiles
3. **Tick Groups**: Different systems at different tick rates
4. **Save/Load**: Serialize tick state for save games
5. **Network Sync**: Deterministic ticks for multiplayer
6. **Integration Tests**: Complete test suite with proper API exports

---

## ❓ Questions?

If something isn't working as expected:

1. Review `TICK_SYSTEM_FIXES.md` for architecture details
2. Check `TESTING_TICK_SYSTEM.md` for testing procedures  
3. Run `./scripts/test_movement.sh` for automated diagnosis
4. Check logs at `/tmp/life-simulator.log`
5. Monitor tick metrics in simulation output

---

**Last Updated**: 2025-10-02  
**Project**: life-simulator v0.1.0  
**Author**: Tick System Fixes & Documentation
