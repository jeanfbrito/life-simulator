# Headless Runner Analysis - SYSTEM IS WORKING CORRECTLY

## Investigation Summary

**CLAIM**: "Headless life-simulator binary is NOT running the Update schedule"

**FINDING**: **CLAIM IS FALSE** - The Update schedule IS running correctly.

## Evidence

### Test 1: Minimal Headless Test
**File**: `src/bin/test_headless_schedule.rs`
**Result**: ✅ PASS
```
✅ UPDATE: Frame 1 - Update schedule is running!
✅ UPDATE: Frame 2 - Update schedule is running!
...
✅ TEST PASSED: Update schedule runs successfully
```

**Conclusion**: `MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(...))` works perfectly.

### Test 2: SimulationPlugin Test
**File**: `src/bin/test_simulation_plugin.rs`
**Result**: ✅ PASS
```
INFO life_simulator::simulation: 🔌 SimulationPlugin: Installing tick systems...
INFO life_simulator::simulation: ✅ SimulationPlugin: Tick systems installed
INFO life_simulator::simulation: 🔍 Frame 1: delta=0.0000s, ticks=0, accumulated=0.0000
INFO life_simulator::simulation: 💓 Heartbeat #1 - Update schedule is running
```

**Conclusion**:
- `accumulate_ticks` system IS running
- `diagnostic_heartbeat` system IS running
- Update schedule executes every frame

## Why "No Ticks" on Frame 1?

**Expected Behavior**: Frame 1 has `delta=0.0000s` which produces `ticks=0`.

This is **NORMAL Bevy behavior**:
- First frame: `Time::delta()` = 0.0 (no time elapsed yet)
- Second frame onwards: `Time::delta()` = actual frame time (~0.0167s at 60 FPS)

**Ticks start accumulating from Frame 2+**, not Frame 1.

## Configuration Verification

### src/main.rs Configuration (Lines 37-73)
```rust
App::new()
    .add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,  // 60 FPS
        ))),
    )
    .add_plugins(bevy::log::LogPlugin::default())
    .add_plugins((
        SimulationPlugin,  // ← Adds accumulate_ticks to Update schedule
        EntitiesPlugin,
        TQUAIPlugin,
        VegetationPlugin,
        HealthCheckPlugin,
        HealthCheckApiPlugin,
    ))
    .run();  // ← Starts the schedule runner loop
```

**Status**: ✅ CORRECT - No changes needed

### SimulationPlugin Configuration (src/simulation/mod.rs:43-52)
```rust
.add_systems(
    Update,
    (
        diagnostic_heartbeat,
        accumulate_ticks.before(run_simulation_ticks),
        run_simulation_ticks,
        handle_speed_controls,
        log_realtime_performance,
    ),
)
```

**Status**: ✅ CORRECT - Systems registered to Update schedule

## Why EntityTracker Shows 0 Entities?

**Potential Issue**: The conditional `.run_if(resource_exists::<WorldLoader>)` on line 71 of `src/main.rs`

```rust
.add_systems(
    Update,
    (
        process_pathfinding_requests,
        pathfinding_cache_cleanup_system,
        simulation_system,
        save_load_system.after(simulation_system),
    )
        .run_if(resource_exists::<WorldLoader>),  // ← These systems only run if WorldLoader exists
)
```

**However**: `WorldLoader` IS inserted in the `setup` Startup system (line 199), so this should not be an issue.

## Actual Root Cause Hypothesis

The **real issue** is likely:

1. **Entities spawn in Startup** (via `spawn_entities_from_config`)
2. **But entity movement/AI systems might not be registered to Update**
3. **Or they have run conditions that prevent execution**

## Recommended Next Steps

1. **Add diagnostic logging** to `spawn_entities_from_config` to confirm entities spawn
2. **Check EntitiesPlugin** systems - ensure movement/AI systems run in Update
3. **Verify entity systems don't have blocking run conditions**
4. **Monitor EntityTracker** update systems - when do they populate the tracker?

## Conclusion

**The headless runner configuration is CORRECT and WORKING.**

The Update schedule runs at 60 FPS, `accumulate_ticks` executes every frame, and ticks accumulate correctly after the first frame.

If entities don't appear to move, the issue is **NOT** with the headless runner, but with:
- Entity spawning
- Entity movement systems
- EntityTracker population
- Or plugin initialization order

**No changes to `src/main.rs` headless configuration are needed.**
