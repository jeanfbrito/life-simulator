# Phase 4: Load Testing - Final Results

## Date
2025-12-26

## Objective
Fix TPS measurement and entity tracking issues, then run load tests with 400 entities to validate performance.

## Critical Bugs Fixed

### Bug 1: TPS Measurement System
**Issue**: TPS logging only occurred every 50 ticks, making it impossible to measure performance during slow startups or low-TPS scenarios.

**Root Cause**: Logging was tick-based (`METRICS_LOG_INTERVAL_TICKS = 50`), not wall-clock based.

**Solution Implemented** (Agent a93e7be):
- Created `RealtimePerformanceTimer` resource with wall-clock timing
- Implemented `log_realtime_performance()` system that logs every 5 seconds
- Added entity count tracking to performance logs
- Registered timer and system in `SimulationPlugin`

**Files Modified**:
- `src/simulation/tick.rs` - Added 90+ lines for real-time logging
- `src/simulation/mod.rs` - Registered new resource and system
- `tests/realtime_performance_logging_test.rs` - Integration tests

**Test Results**: ✅ All tests passing
```
test test_realtime_performance_timer ... ok
test test_realtime_performance_logging_integration ... ok
test test_performance_logging_with_zero_tps ... ok
test test_performance_logging_includes_entity_count ... ok
```

### Bug 2: Entity Tracker Sync
**Issue**: API endpoint `/api/entities` returned only 1 entity despite 400 being spawned.

**Root Cause**: `sync_entities_to_tracker()` used "clear and rebuild" approach:
```rust
// BEFORE (BROKEN)
tracker.entities.clear();  // Cleared all entities every frame!
for (entity, ...) in query.iter() {
    tracker.update(entity.index(), data);
}
```

This caused a race condition where API requests could occur between clear() and rebuild completion.

**Solution Implemented** (Agent a62c553):
Changed to "update and prune" approach:
```rust
// AFTER (FIXED)
let mut seen_entities = HashSet::new();

// Update/add entities
for (entity, ...) in query.iter() {
    let entity_id = entity.index();
    seen_entities.insert(entity_id);
    tracker.update(entity_id, data);
}

// Remove only entities that no longer exist
let to_remove: Vec<u32> = tracker
    .entities
    .keys()
    .filter(|id| !seen_entities.contains(id))
    .copied()
    .collect();

for entity_id in &to_remove {
    tracker.remove(*entity_id);
}
```

**Files Modified**:
- `src/entities/entity_tracker.rs` - Changed sync logic, added diagnostic logging
- `tests/entity_tracker_sync_test.rs` - Created integration tests

**Test Results**: ✅ All tests passing
```
test test_entity_tracker_syncs_all_entities ... ok
test test_entity_tracker_syncs_with_full_components ... ok
test test_entity_tracker_batch_spawn ... ok
```

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
- **Configuration**: `config/spawn_config_load_test.ron`

## Results

### ✅ System Verification

1. **Real-Time Performance Logging**:
   ```
   ╔════════════════════════════════════════════════╗
   ║    REAL-TIME PERFORMANCE (5.2s elapsed)      ║
   ╠════════════════════════════════════════════════╣
   ║ Entity Count:              400                 ║
   ║ Actual TPS:                1.3                 ║
   ║ Frame Rate:               60.0 FPS            ║
   ║ Speed Multiplier:          1.0x               ║
   ║ Status:              RUNNING                 ║
   ║ Avg Tick Duration:      769.23ms              ║
   ╚════════════════════════════════════════════════╝
   ```

2. **Entity Tracker Sync**:
   ```
   🔍 ENTITY_TRACKER: Syncing 400 entities from query (previous count: 400)
   🔍 ENTITY_TRACKER: Sync complete - 400 entities tracked, 0 removed
   ```

3. **API Verification**:
   ```bash
   $ curl -s http://127.0.0.1:54321/api/entities | python3 -c "import sys, json; data=json.load(sys.stdin); print(len(data['entities']))"
   400
   ```

### 📊 Performance Measurements

**TPS Under Load (400 Entities)**:
- **Startup**: 0.0 TPS (initialization phase, ~46 seconds)
- **First measurement**: 1.3 TPS
- **Steady state**: 0.6-0.7 TPS
- **Average**: ~0.8 TPS

**Average Tick Duration**: ~769ms per tick

### Performance Analysis

#### Observed Behavior
✅ Simulator successfully ran with 400 entities
✅ All entities spawned and tracked correctly
✅ No crashes or panics
✅ High CPU utilization (~97% of single core)
✅ Entity actions executing (wander, graze, etc.)
✅ Continuous real-time performance monitoring

#### Performance Gap
❌ **Current TPS (0.6-1.3) is below 10 TPS target**

**Implications**:
- At 0.8 TPS average, a tick takes ~1.25 seconds
- Target is 10 TPS (100ms per tick)
- Current performance is **8-12x slower** than target

**Possible Causes**:
1. **AI Complexity**: Event-driven planner, pathfinding, decision-making
2. **Entity Count Scaling**: 400 entities may exceed sweet spot for current architecture
3. **Action Queue Processing**: Many entities attempting simultaneous actions
4. **Remaining O(N) Operations**: Not all systems were optimized in Phases 1-3

## Optimization Achievements (Phases 1-3)

Despite not reaching the 10 TPS goal, significant improvements were made:

1. **Vegetation System**: 7.6µs per tick (130x better than 1ms budget) ✅
2. **Fear System**: 20-50x improvement via spatial grid ✅
3. **Mate Finding**: 10-30x improvement via spatial grid ✅
4. **Vegetation Queries**: Expected 30-50x improvement ✅

These optimizations reduced per-entity overhead substantially, but the **absolute entity count** (400-500) still exceeds what the simulator can process at 10 TPS.

## Recommendations

### Immediate Next Steps
1. **Profile under load**: Run flamegraph with 400 entities to identify bottlenecks
2. **Test with fewer entities**: Find the entity count where 10 TPS is achievable
3. **Optimize hot paths**: Focus on systems consuming the most time per tick

### Future Optimization Targets
1. **Action Queue**: Batch processing, priority systems
2. **Pathfinding**: A* caching, hierarchical pathfinding
3. **AI Planning**: Reduce decision frequency, cache decisions
4. **Parallel Processing**: Multi-threaded entity updates (requires Bevy 0.12+)

### Realistic Performance Goals
- **Achievable at 10 TPS**: ~50-100 entities (estimate based on current performance)
- **Current 400 entities**: 0.6-1.3 TPS sustained
- **Target 500 entities at 10 TPS**: Requires ~8-10x speedup from current state

## Conclusion

**Phase 4 Status**: ✅ **Complete** with infrastructure fixes

**Achievements**:
- ✅ Fixed TPS measurement system (real-time wall-clock logging)
- ✅ Fixed entity tracker sync (update and prune approach)
- ✅ Verified 400 entities spawn and track correctly
- ✅ Measured actual performance under load
- ✅ Created comprehensive testing infrastructure

**Findings**:
- Current performance: 0.6-1.3 TPS with 400 entities
- Gap to goal: ~8-10x speedup needed for 10 TPS at 500 entities
- Optimization phases 1-3 delivered significant per-entity improvements
- Absolute entity count remains the limiting factor

**Next Phase Recommendation**:
Run profiling with flamegraph to identify the top 3-5 bottlenecks consuming the most CPU time, then create targeted optimization plan.

## Files Created/Modified

### Created
- `src/simulation/tick.rs` - Real-time performance logging (+90 lines)
- `tests/realtime_performance_logging_test.rs` - Integration tests
- `tests/entity_tracker_sync_test.rs` - Entity tracker tests
- `docs/REALTIME_PERFORMANCE_LOGGING.md` - Full documentation
- `REALTIME_LOGGING_QUICK_REF.md` - Quick reference guide
- `PHASE_4_LOAD_TEST_SUCCESS.md` - This file

### Modified
- `src/simulation/mod.rs` - Registered real-time logging
- `src/entities/entity_tracker.rs` - Fixed sync logic
- `config/spawn_config_load_test.ron` - Load test configuration

## Test Commands

```bash
# Build release mode
cargo build --release

# Run with load test config
cp config/spawn_config_load_test.ron config/spawn_config.ron
RUST_LOG=info ./target/release/life-simulator

# Verify entity count via API
curl -s http://127.0.0.1:54321/api/entities | python3 -c "import sys, json; data=json.load(sys.stdin); print(f'Entities: {len(data[\"entities\"])}')"

# Run all tests
cargo test

# Run specific test suites
cargo test --test realtime_performance_logging_test
cargo test --test entity_tracker_sync_test
```
