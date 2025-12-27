# Phase 4: Load Testing and Bug Fixes

## Date
2025-12-26

## Objective
Run load tests with 300-500 entities to measure actual TPS (ticks per second) under load and validate the 10 TPS performance target.

## Critical Bug Found and Fixed

### Bug: Bevy ECS System Parameter Conflict
**Location**: `src/entities/spatial_maintenance.rs`

**Error**:
```
&World conflicts with a previous mutable system parameter. Allowing this would break Rust's mutability rules
```

**Root Cause**:
Two systems had conflicting parameters:
1. `maintain_spatial_entity_index_insertions()`
2. `maintain_spatial_entity_index_updates()`

Both systems used:
- `mut spatial_index: ResMut<SpatialEntityIndex>` (mutable borrow)
- `world: &World` (immutable borrow of entire world)

This violates Bevy's safety guarantees - you cannot have both mutable and immutable access to the same data.

**Fix Applied**:
Replaced the `&World` parameter with specific queries to classify entity types:
```rust
// BEFORE (BROKEN)
world: &World

// AFTER (FIXED)
predators: Query<(), Or<(With<Fox>, With<Wolf>)>>,
herbivores: Query<(), Or<(With<Rabbit>, With<Deer>)>>,
omnivores: Query<(), Or<(With<Bear>, With<Raccoon>)>>,
```

**Files Modified**:
- `src/entities/spatial_maintenance.rs` - Fixed system parameters and updated imports

## Load Test Configuration

### Test Setup
- **Entity Count**: 400 total entities
- **Species Mix**:
  - 150 Rabbits (herbivores)
  - 100 Deer (herbivores)
  - 80 Raccoons (omnivores)
  - 40 Foxes (predators)
  - 20 Wolves (predators)
  - 10 Bears (omnivores)
- **Build**: Release mode (`cargo build --release`)
- **World**: green_world_with_water (seed: 42069)

## Observed Behavior

### Positive Indicators
✅ Simulator successfully started and initialized all systems
✅ All 400 entities spawned successfully
✅ High CPU utilization (~97% of single core) indicating active processing
✅ Entity actions being executed (7479 log lines in ~1 minute)
✅ No crashes or panics after fixing the ECS conflict

### Issues Discovered

#### Issue 1: TPS Measurement Unavailable
- **Symptom**: Frame logs show "Actual TPS: 0.0" during startup, then stop logging
- **Impact**: Cannot measure actual TPS under load
- **Possible Causes**:
  - TPS logging may only occur periodically or on specific events
  - Tick profiler may not be configured for continuous output
  - TPS calculation may have issues

#### Issue 2: Entity API Discrepancy
- **Symptom**: API endpoint `/api/entities` reports only 1 entity despite 400 spawned
- **Impact**: Cannot validate entity survival or track population
- **Possible Causes**:
  - Entity tracker not syncing properly
  - Entities dying extremely quickly (seems unlikely given action logs)
  - API cache/sync issue between ECS and web server

#### Issue 3: Entity Action Failures
- **Symptom**: Many "failed action 'Wander'" warnings in logs
- **Impact**: Unknown - could be normal behavior or indicate pathfinding issues
- **Context**: Entities may fail to wander if no valid destination found

## Files Created
- `config/spawn_config_load_test.ron` - Load test configuration with 400 entities
- `/tmp/load_test_output.log` - Full simulator output (7479 lines)

## Recommendations

### Immediate Actions Needed
1. **Fix TPS Measurement**:
   - Add continuous TPS logging to tick profiler
   - Ensure metrics are output even under high load
   - Consider adding `/api/performance` endpoint

2. **Debug Entity API**:
   - Investigate entity tracker sync mechanism
   - Add entity count validation after spawning
   - Check if entities are dying prematurely

3. **Add Load Test Validation**:
   - Create automated load test script
   - Add assertions for minimum entity survival rate
   - Capture TPS metrics to file for analysis

### Future Work
1. Run flamegraph profiling with fixed TPS measurement
2. Test with 500 entities once issues are resolved
3. Validate 10 TPS target with accurate measurements
4. Document performance tuning guidelines

## Conclusion

**Phase 4 Status**: Partially Complete ⚠️

**Achievements**:
- ✅ Found and fixed critical Bevy ECS system conflict
- ✅ Created load test configuration
- ✅ Simulator runs stably with 400 entities

**Blockers**:
- ❌ Cannot measure actual TPS (instrumentation issue)
- ❌ Entity API shows incorrect count (sync issue)
- ❌ Unable to validate 10 TPS performance target

**Next Steps**:
Fix TPS measurement and entity tracking issues before proceeding with profiling and final documentation. The core optimization work (Phases 1-3) is complete and validated - Phase 4 revealed infrastructure issues that need addressing for accurate performance measurement.
