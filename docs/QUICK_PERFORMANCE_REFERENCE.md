# Quick Performance Reference

## 🚀 Running Benchmarks

### Entity State Benchmarks
```bash
# Run all entity state benchmarks
cargo test --test entity_state_benchmark -- --nocapture

# Run specific benchmarks
cargo test --test entity_state_benchmark benchmark_entity_query_overhead -- --nocapture
cargo test --test entity_state_benchmark benchmark_position_lookup -- --nocapture
cargo test --test entity_state_benchmark benchmark_planning_cycle -- --nocapture
cargo test --test entity_state_benchmark benchmark_proximity_queries -- --nocapture
```

### Integrated Performance Benchmarks
```bash
# Run all integrated benchmarks
cargo test --test integrated_performance -- --nocapture

# Run specific load scenarios
cargo test --test integrated_performance benchmark_idle_world -- --nocapture
cargo test --test integrated_performance benchmark_low_entity_load -- --nocapture
cargo test --test integrated_performance benchmark_medium_entity_load -- --nocapture
cargo test --test integrated_performance benchmark_high_entity_load -- --nocapture
cargo test --test integrated_performance benchmark_stress_test -- --nocapture

# Run workload-specific tests
cargo test --test integrated_performance benchmark_heavy_grazing -- --nocapture
cargo test --test integrated_performance benchmark_predator_chase -- --nocapture
cargo test --test integrated_performance benchmark_reproduction_peak -- --nocapture
```

### Vegetation Benchmarks
```bash
# Run existing vegetation benchmarks
cargo test --package life-simulator --lib vegetation::benchmark -- --nocapture
```

---

## 📊 Current Performance Baselines

### Vegetation System (ResourceGrid)
- **Target**: <2000μs per tick
- **Current**: ~145μs per tick ✅
- **Active Cells**: ~156 cells
- **Event Processing**: Event-driven (no per-tick loops)
- **Budget Compliance**: 95%+

### Entity System (150 entities)
- **Target**: <3000μs per tick
- **Current**: ~2000μs estimated
- **Query Overhead**: Multi-component iteration
- **Position Lookups**: HashMap-based O(1)
- **Proximity Queries**: Linear O(N) search

---

## 🎯 Performance Targets by Load

| Load Scenario | Entities | Target Time | Budget |
|---------------|----------|-------------|--------|
| Idle | 0 | <500μs | Vegetation baseline |
| Low | 50 | <2000μs | Light load |
| Medium | 150 | <3000μs | Current production |
| High | 300 | <5000μs | Optimized target |
| Stress | 500+ | <10000μs | Maximum capacity |

---

## 🔧 Quick Profiling Commands

### CPU Profiling (Flamegraph)
```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin life-simulator
# Opens flamegraph.svg in browser

# With specific duration
cargo flamegraph --bin life-simulator -- --duration 30
```

### Sampling Profiler (samply)
```bash
# Install samply
cargo install samply

# Record profile
samply record cargo run --release --bin life-simulator

# Opens profiler UI in browser
```

### Memory Profiling (heaptrack)
```bash
# macOS: Install via brew
brew install heaptrack

# Profile memory usage
heaptrack cargo run --release --bin life-simulator
heaptrack_gui heaptrack.life-simulator.*
```

### Cargo Bench (for microbenchmarks)
```bash
# Run criterion benchmarks if implemented
cargo bench --package life-simulator
```

---

## 🐛 Common Performance Issues

### Issue 1: High Query Overhead
**Symptoms**: Tick time scales linearly with entity count
**Cause**: Multiple Bevy queries per entity
**Solution**: Implement CachedEntityState (Phase 1)

### Issue 2: Slow Proximity Queries
**Symptoms**: Fear system/mate finding takes >1ms
**Cause**: O(N) linear search through all entities
**Solution**: Implement SpatialEntityIndex (Phase 2)

### Issue 3: Cache Misses
**Symptoms**: Erratic tick times, high stddev
**Cause**: Random access patterns in entity processing
**Solution**: Batch entity processing by chunk (Phase 2)

### Issue 4: Memory Growth
**Symptoms**: Increasing memory usage over time
**Cause**: HashMap resizing, vector growth
**Solution**: Pre-allocate with capacity hints

---

## 📈 Optimization Priority Matrix

### Phase 1: Quick Wins (1 week)
1. ✅ **Event Batch Processing** - Group vegetation events by chunk
   - Complexity: Low
   - Impact: 20-30%
   - Files: `src/vegetation/resource_grid.rs`

2. ✅ **Cached Entity State** - Pre-compute urgencies and flags
   - Complexity: Medium
   - Impact: 40-60%
   - Files: `src/entities/cached_state.rs` (new)

### Phase 2: Spatial Optimizations (2 weeks)
3. ✅ **Spatial Entity Index** - Grid-based proximity queries
   - Complexity: Medium-High
   - Impact: 90% (proximity queries)
   - Files: `src/entities/spatial_index.rs` (new)

4. ✅ **Vegetation Spatial Grid** - Chunked vegetation storage
   - Complexity: Medium
   - Impact: 50% (radius queries)
   - Files: `src/vegetation/spatial_grid.rs` (new)

### Phase 3: Advanced (1 week)
5. ✅ **LOD-Based Updates** - Distance-based update frequency
   - Complexity: Medium
   - Impact: 60-80% (large worlds)
   - Files: `src/vegetation/lod.rs`, integrate with `chunk_lod.rs`

---

## 🎨 Interpreting Benchmark Results

### Entity Query Benchmark
```
Avg Query Time: 850μs
Per-Entity Cost: 5.67μs
Efficiency: Good
```
**Interpretation**: Each entity costs ~6μs to query. For 150 entities = 850μs total.
**Target**: <3μs per entity (achieved via caching)

### Position Lookup Benchmark
```
Avg Lookup Time: 45ns
HashMap Overhead: 25ns
Overhead %: 55.6%
```
**Interpretation**: HashMap adds 55% overhead vs direct access.
**Target**: Eliminate HashMap via component access

### Planning Cycle Benchmark
```
Avg Cycle Time: 2150μs
Entities/Second: 697.7
Per-Entity Cost: 14.33μs
```
**Interpretation**: Full planning cycle costs ~14μs per entity.
**Target**: <10μs per entity (combined optimizations)

### Proximity Query Benchmark
```
Avg Query Time: 125μs (linear)
Theoretical Speedup: 10x (with spatial grid)
Estimated Grid Time: 12.5μs
```
**Interpretation**: Spatial index would reduce proximity queries by 90%.
**Target**: <20μs per proximity query

---

## 📦 Optimization Implementation Checklist

### Before Starting
- [ ] Run baseline benchmarks and save results
- [ ] Create feature branch: `perf/optimization-name`
- [ ] Document current behavior (tests)

### During Implementation
- [ ] Write tests first (TDD)
- [ ] Implement optimization incrementally
- [ ] Run benchmarks after each change
- [ ] Profile with flamegraph/samply
- [ ] Check for regressions

### After Completion
- [ ] Run full benchmark suite
- [ ] Compare before/after results
- [ ] Update documentation
- [ ] Visual testing for behavior changes
- [ ] Code review

---

## 🔍 Performance Monitoring Commands

### Check Current Tick Time
```bash
# Run simulator with logging
RUST_LOG=info cargo run --bin life-simulator

# Look for TickProfiler logs
# [INFO] Tick 1000: 2.45ms (vegetation: 0.15ms, ai: 1.20ms, ...)
```

### Monitor Resource Usage
```bash
# macOS Activity Monitor
open -a "Activity Monitor"
# Search for "life-simulator"

# Command line monitoring
while true; do ps aux | grep life-simulator | grep -v grep; sleep 1; done
```

### Test Specific Scenario
```bash
# Run with specific spawn config
cargo run --bin life-simulator -- --config config/spawn_config_high_load.ron
```

---

## 📚 Related Documentation

- **Full Plan**: `docs/PERFORMANCE_OPTIMIZATION_PLAN.md`
- **Vegetation System**: `docs/PLANT_SYSTEM_PARAMS.md`
- **AI System**: `docs/EVENT_DRIVEN_PLANNER_IMPLEMENTATION.md`
- **Entity System**: `docs/SPECIES_REFERENCE.md`
- **API Reference**: `docs/API_REFERENCE.md`

---

## 🆘 Troubleshooting

### Benchmark Won't Run
```bash
# Make sure test dependencies are available
cargo test --test entity_state_benchmark --no-run

# Check for compilation errors
cargo check --tests
```

### Inconsistent Results
```bash
# Run in release mode for consistent timing
cargo test --release --test integrated_performance -- --nocapture

# Close other applications
# Run multiple times and average results
```

### Out of Memory
```bash
# Reduce entity count in benchmarks
# Edit tests/integrated_performance.rs
# Change ENTITY_COUNTS or LoadScenario values
```

---

**Last Updated**: 2025-12-25
**Version**: 1.0
