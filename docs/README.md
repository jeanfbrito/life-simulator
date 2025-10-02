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

### Quick Verification

```bash
# Run the automated movement test
./scripts/test_movement.sh
```

This script will:
1. Start the simulation
2. Track entity movement for 30 seconds
3. Analyze movement patterns
4. Report success or failure
5. Clean up processes

Expected output: Entities should move every ~3 seconds (30 ticks at 10 TPS).

### Manual Testing

```bash
# Terminal 1: Start simulation
cargo run --bin life-simulator

# Terminal 2: Monitor entities
watch -n 2 'curl -s http://127.0.0.1:54321/api/entities | jq ".entities[] | {name, position}"'
```

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

---

## 🎯 Quick Reference

### Common Commands

```bash
# Run simulation
cargo run --bin life-simulator

# Run movement test
./scripts/test_movement.sh

# Monitor API
curl http://127.0.0.1:54321/api/entities | jq

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
