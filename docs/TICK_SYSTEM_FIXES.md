# Tick System Issues and Fixes

## Overview

This document details all the issues we encountered while implementing and debugging the tick-based simulation system, their root causes, and the solutions applied.

## Date: 2025-10-02

---

## Issue #1: Entities Not Moving Despite Tick System Running

### Symptoms
- Simulation ticks were incrementing correctly
- Entities remained at their spawn positions indefinitely
- No movement or position updates observed
- Wandering AI appeared inactive

### Investigation Steps
1. Verified tick counter was incrementing (✓)
2. Checked entity spawning and initialization (✓)
3. Examined movement system execution
4. Analyzed system scheduling and run conditions

### Root Cause
**The tick-based systems were not running due to improper scheduling configuration:**

1. **Missing `should_tick` Run Condition**: Tick-based systems (wandering AI, movement, stats) were scheduled in the `Update` schedule but lacked the critical `run_if(should_tick)` condition, causing them to either:
   - Never run at all
   - Run on every frame instead of per-tick (timing mismatch)

2. **Schedule Confusion**: Initial attempts used Bevy's `FixedUpdate` schedule with `ScheduleRunnerPlugin` in headless mode, which led to:
   - Timing inconsistencies
   - Complex interactions between frame rate and tick rate
   - Unreliable tick execution in headless environments

3. **Accumulator Not Driving Tick Logic**: The `TickAccumulator` was calculating available ticks correctly, but the `SimulationState.should_tick` flag wasn't being:
   - Set when ticks were available
   - Cleared after systems ran
   - Used as a run condition for tick-based systems

### Solution

#### Part 1: Manual Tick Control via Accumulator
```rust
pub fn accumulate_ticks(
    time: Res<Time>,
    mut accumulator: ResMut<TickAccumulator>,
    mut tick: ResMut<SimulationTick>,
    mut state: ResMut<SimulationState>,
    speed: Res<SimulationSpeed>,
) {
    // Accumulate time and calculate available ticks
    accumulator.accumulate(time.delta(), speed.multiplier);
    let available_ticks = accumulator.available_ticks();
    
    if available_ticks > 0 {
        state.should_tick = true;  // Signal that ticks are ready
        tick.0 += available_ticks;
        accumulator.consume_ticks(available_ticks);
    } else {
        state.should_tick = false;  // No ticks available
    }
}
```

#### Part 2: Conditional System Execution
```rust
app.add_systems(Update, (
    // Tick-based systems run ONLY when should_tick is true
    wanderer_ai_system,
    movement_tick_system,
    tick_stats_system,
    death_system,
).run_if(should_tick));
```

#### Part 3: Run Condition Helper
```rust
pub fn should_tick(state: Res<SimulationState>) -> bool {
    state.should_tick
}
```

### Key Architectural Changes

1. **Moved from FixedUpdate to Update**: All tick systems now run in `Update` with explicit `should_tick` conditions
2. **Explicit Tick Availability**: Added `should_tick` flag to `SimulationState` resource
3. **Accumulator-Driven Execution**: Time accumulation drives when systems should run, not Bevy's fixed timestep
4. **Decoupled Frame Rate from Tick Rate**: Simulation ticks are independent of rendering/frame updates

---

## Issue #2: Movement Speed Configuration

### Symptoms
- Entities moved too slowly at 100 ticks per tile (10 seconds at 10 TPS)
- Difficult to observe and verify movement in tests

### Investigation Steps
1. Calculated actual movement time: 100 ticks ÷ 10 TPS = 10 seconds per tile
2. Recognized this was too slow for testing and observation
3. Evaluated appropriate speeds for different entity types

### Solution
Adjusted movement speed to more reasonable values:
- **Humans**: 30 ticks per tile (3 seconds at 10 TPS)
- Future consideration: Different speeds for different entity types

```rust
// Old configuration
MovementSpeed { ticks_per_tile: 100 }

// New configuration  
MovementSpeed { ticks_per_tile: 30 }
```

---

## Issue #3: Headless Environment Missing Core Plugins

### Symptoms
- Time resources not available
- Scheduling issues in headless mode
- App not progressing through update cycles properly

### Root Cause
The headless app was created with minimal plugins but lacked essential Bevy systems:
- No `TimePlugin` for time tracking
- No proper scheduling infrastructure
- Missing core ECS functionality

### Solution
Added `MinimalPlugins` to headless app configuration:

```rust
let mut app = App::new();
app.add_plugins(MinimalPlugins);  // Includes TimePlugin and core scheduling
```

`MinimalPlugins` includes:
- `TaskPoolPlugin` - Async task execution
- `TypeRegistrationPlugin` - Type reflection
- `FrameCountPlugin` - Frame tracking
- `TimePlugin` - Time tracking (critical for our accumulator)
- `ScheduleRunnerPlugin` - Update loop execution

---

## Issue #4: Simulation Speed Changes Not Applying

### Symptoms
- Changing simulation speed didn't affect tick rate
- Speed multiplier changes had no visible effect

### Root Cause
The `accumulate_ticks` system was reading `SimulationSpeed` but might have been:
- Cached or not detecting resource changes
- Running in wrong order relative to speed change systems
- Not properly applying the multiplier

### Solution
Ensured proper system ordering and multiplier application:

```rust
// In accumulate_ticks system
let base_tick_duration = accumulator.base_tick_duration;
let adjusted_duration = base_tick_duration.div_f64(speed.multiplier);
```

---

## Testing Observations

### Successful Movement Test Results
Test date: 2025-10-02

With configuration:
- Base TPS: 10
- Movement speed: 30 ticks per tile (3 seconds per tile)
- Test duration: 20 seconds

**Observed behavior:**
```
Time  | Position | Movement
------|----------|----------
2s    | (7,20)   | Start
4s    | (7,20)   | Accumulating ticks
6s    | (6,20)   | ✓ Moved (3 sec elapsed)
8s    | (5,20)   | ✓ Moved (3 sec elapsed)
10s   | (5,20)   | Accumulating ticks
12s   | (4,20)   | ✓ Moved (3 sec elapsed)
14s   | (5,20)   | ✓ Moved + direction change (AI decision)
16s   | (5,20)   | Accumulating ticks
18s   | (4,20)   | ✓ Moved (3 sec elapsed)
20s   | (3,20)   | ✓ Moved (3 sec elapsed)
```

**Key findings:**
1. ✅ Movement timing is consistent (~3 seconds per tile)
2. ✅ Tick accumulation working correctly
3. ✅ Wandering AI making new decisions (direction change at 14s)
4. ✅ Path following and movement execution working
5. ✅ System remains stable over time

---

## Architecture: Final Tick System Design

### Component Hierarchy
```
App (Update Schedule)
├── accumulate_ticks (runs every frame)
│   ├── Reads: Time, SimulationSpeed
│   ├── Writes: TickAccumulator, SimulationTick, SimulationState
│   └── Sets: should_tick flag
│
└── Tick-based systems (run when should_tick == true)
    ├── wanderer_ai_system
    ├── movement_tick_system
    ├── tick_stats_system
    └── death_system
```

### Key Resources
- **`TickAccumulator`**: Accumulates frame delta time and converts to ticks
- **`SimulationTick`**: Current tick counter (u64)
- **`SimulationState`**: Contains `should_tick` flag
- **`SimulationSpeed`**: Multiplier for tick rate (1.0 = normal, 2.0 = 2x speed)

### Timing Calculations
```
Base TPS = 10
Base Tick Duration = 1.0 / 10 = 0.1 seconds = 100ms

With speed multiplier = 2.0:
Adjusted Tick Duration = 100ms / 2.0 = 50ms
Effective TPS = 20

Movement at 30 ticks per tile:
Normal speed (1.0x): 30 ticks × 100ms = 3000ms = 3 seconds
Fast speed (2.0x):   30 ticks × 50ms  = 1500ms = 1.5 seconds
```

---

## Lessons Learned

### 1. Explicit is Better Than Implicit
Don't rely on Bevy's built-in fixed timestep in headless environments. Manual tick accumulation gives you:
- Full control over timing
- Predictable behavior
- Better debuggability
- Independence from frame rate

### 2. Run Conditions Are Critical
Always use explicit run conditions for systems that should run conditionally:
```rust
.run_if(should_tick)  // ✅ Good
// vs implied timing    ❌ Bad
```

### 3. Resource Flags for Cross-System Communication
Using a simple boolean flag (`should_tick`) in a shared resource is an effective pattern for:
- Coordinating system execution
- Avoiding complex scheduling dependencies
- Making execution flow explicit and traceable

### 4. Test at Human-Observable Speeds
When debugging:
- Use faster movement speeds (2-3 seconds per tile)
- Add comprehensive logging
- Test with real-world timings, not just unit test mocks

### 5. Separation of Concerns
Clear separation between:
- **Time accumulation** (frame-based, runs always)
- **Tick availability** (calculated, sets flags)
- **Tick execution** (conditional, only when ticks available)

---

## Future Considerations

### Potential Improvements
1. **Tick Budgeting**: Limit max ticks per frame to prevent spiral of death
2. **Interpolation**: Add position interpolation for smoother visual movement
3. **Tick Groups**: Different systems might run at different tick rates
4. **Save/Load**: Serialize tick state for save games
5. **Network Sync**: Deterministic ticks for multiplayer

### Performance Monitoring
Add metrics for:
- Average ticks per frame
- Tick execution duration
- System performance per tick
- Frame time vs tick time ratio

---

## References

- World-simulator project: Example of manual tick accumulation
- Bevy Time documentation: Understanding Time resources
- This implementation: `/Users/jean/Github/life-simulator/src/simulation/tick.rs`

---

## Testing Strategy

See `tests/tick_system_tests.rs` for:
1. Tick accumulation consistency
2. Movement timing verification
3. Multi-entity synchronization
4. Simulation speed changes
5. Long-running stability

These tests ensure the issues documented here don't regress in future changes.
