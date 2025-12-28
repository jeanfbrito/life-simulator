# Entity Count Stress Test Report

## Executive Summary

This report documents the entity count stress testing infrastructure for the Life Simulator. The system has been configured to test performance with 100, 300, 500, and 700 entities to identify scaling bottlenecks and performance limits.

**Key Findings:**
- **Target TPS**: 10 ticks per second (100ms per tick)
- **Primary Metric**: Tick time and consistency under load
- **Test Configuration**: Realistic species distribution (70% herbivores, 30% predators)

---

## Test Infrastructure

### Stress Test Binary
Location: `/Users/jean/Github/life-simulator/src/bin/stress_test.rs`

**Features:**
- Minimal simulation loop for baseline performance measurement
- Configurable duration and entity counts
- Real-time tick time tracking
- Percentile analysis (P50, P95, P99)
- Statistical analysis (average, stddev, range)

**Build:**
```bash
cargo build --bin stress_test --release
```

**Run:**
```bash
# Basic (60 second test)
cargo run --release --bin stress_test

# Custom duration
STRESS_TEST_DURATION=120 cargo run --release --bin stress_test

# Custom config
STRESS_TEST_CONFIG=config/spawn_config_stress_100.ron \
  cargo run --release --bin stress_test
```

### Test Configurations

#### spawn_config_stress_100.ron
**Entity Distribution:**
- Rabbits: 70 (70%)
- Deer: 20 (20%)
- Wolves: 8 (8%)
- Foxes: 2 (2%)
- **Total: 100 entities**

**Spawn Areas:**
- Herbivores centered at (0, 0) with 40-60 tile radius
- Predators distributed in separate quadrants
- Search radius proportional to entity count

#### spawn_config_stress_300.ron
**Entity Distribution:**
- Rabbits: 210 (70%)
- Deer: 60 (20%)
- Wolves: 24 (8%)
- Foxes: 6 (2%)
- **Total: 300 entities**

#### spawn_config_stress_test.ron (500 entities)
**Entity Distribution:**
- Rabbits: 300 (60%)
- Deer: 100 (20%)
- Wolves: 80 (16%)
- Foxes: 20 (4%)
- **Total: 500 entities**

**Spawn Areas:**
- Primary herbivores centered at origin (0, 0)
- Secondary herbivores at (20, 20)
- Predators in (-40, -40) and (40, -40) for spatial separation
- Search radii: 80-50 tiles per group

#### spawn_config_stress_700.ron
**Entity Distribution:**
- Rabbits: 490 (70%)
- Deer: 140 (20%)
- Wolves: 56 (8%)
- Foxes: 14 (2%)
- **Total: 700 entities**

**Spawn Configuration:**
- Larger search radii (100 tiles max)
- More distributed spawn centers
- Maximum spawn attempts: 10 per entity

---

## Performance Metrics

### Measurement Framework

**Tick Time Metrics:**
```
Average Tick Time = Total elapsed time / Number of ticks
Median (P50) = 50th percentile of tick times
P95 = 95th percentile (worst case acceptable)
P99 = 99th percentile (outliers)
Stddev = Standard deviation (consistency)
```

**Throughput Metrics:**
```
TPS = Ticks per second = Tick count / Elapsed seconds
Target: 10.0 TPS
Budget: 100,000 µs (100ms) per tick
```

### Performance Targets

| Load | Entities | Target TPS | Target Tick Time | Max Stddev |
|------|----------|-----------|-----------------|-----------|
| Low | 100 | 10.0 | 50 ms | 10 ms |
| Medium | 300 | 10.0 | 75 ms | 15 ms |
| High | 500 | 10.0 | 100 ms (budget limit) | 20 ms |
| Ultra | 700 | 8.0+ | 125 ms | 30 ms |

### Analysis Thresholds

**TPS Assessment:**
- ✅ **PASS**: TPS ≥ 9.5 (meets target)
- ⚠️ **MARGINAL**: 8.0-9.5 TPS (below target)
- ❌ **FAIL**: < 8.0 TPS (significant bottleneck)

**Consistency Assessment:**
- ✅ **STABLE**: Stddev < 50% of average
- ⚠️ **VARIABLE**: Stddev 50-100% of average
- ❌ **ERRATIC**: Stddev > 100% of average

**Outlier Assessment:**
- ✅ **GOOD**: P99 < 1.5x average
- ⚠️ **CONCERNING**: P99 1.5-2.0x average
- ❌ **PROBLEMATIC**: P99 > 2.0x average

---

## Test Execution Guide

### Running Individual Tests

**Test 100 Entities:**
```bash
cargo run --release --bin stress_test -- \
  --config config/spawn_config_stress_100.ron \
  --duration 60
```

**Test 300 Entities:**
```bash
cargo run --release --bin stress_test -- \
  --config config/spawn_config_stress_300.ron \
  --duration 60
```

**Test 500 Entities:**
```bash
STRESS_TEST_CONFIG=config/spawn_config_stress_test.ron \
  cargo run --release --bin stress_test
```

**Test 700 Entities:**
```bash
STRESS_TEST_CONFIG=config/spawn_config_stress_700.ron \
  STRESS_TEST_DURATION=120 \
  cargo run --release --bin stress_test
```

### Running Full Test Suite

```bash
# Make script executable
chmod +x scripts/run_stress_test.sh

# Run full suite (4 tests × 60 seconds)
./scripts/run_stress_test.sh

# Quick suite (4 tests × 10 seconds)
./scripts/run_stress_test.sh --quick

# Extended duration
./scripts/run_stress_test.sh --duration 180
```

### Profiling with Flamegraph

```bash
# Install flamegraph if needed
cargo install flamegraph

# Run stress test with profiling
# Note: Requires Linux/WSL or macOS with appropriate tools
cargo flamegraph --bin stress_test --release -- \
  --config config/spawn_config_stress_test.ron \
  --duration 30

# Output: flamegraph.svg
```

---

## Bottleneck Identification Strategy

### Phase 1: Baseline Measurements

1. **100 Entity Test**
   - Establishes baseline performance
   - Should easily exceed target TPS
   - Expected: 50-80 ms per tick

2. **300 Entity Test**
   - Identifies regression points
   - Expected: 70-100 ms per tick

3. **500 Entity Test**
   - High load scenario
   - At performance budget limit
   - Expected: 100-120 ms per tick

4. **700 Entity Test**
   - Breaking point testing
   - May exceed target TPS
   - Identifies critical bottlenecks

### Phase 2: System Isolation

Create modified test configurations that disable specific systems:

```rust
// Tests with systems disabled (in test harness):
1. Disable AI system → measure base movement overhead
2. Disable pathfinding → measure navigation costs
3. Disable vegetation → measure resource system cost
4. Disable movement → measure update loop cost
```

### Phase 3: Profiling Analysis

**Flamegraph Interpretation:**
- **Wide boxes**: Functions taking lots of CPU time
- **Tall stacks**: Deep call chains
- **Long flame patterns**: Loop iterations

**Common Bottlenecks to Look For:**
1. **Spatial Queries** (O(n²) behavior with entity count)
   - Broad phase collision checks
   - Quadtree/grid lookups
   - Radius-based entity searches

2. **Pathfinding Operations** (A* complexity)
   - Frequent recalculations
   - Large search spaces
   - Cache misses

3. **AI Decision Making**
   - Utility calculations per entity
   - Behavior tree evaluations
   - State machine updates

4. **Vegetation System**
   - ResourceGrid updates
   - Consumption calculations
   - Grid-based lookups

5. **Bevy Infrastructure**
   - Change detection overhead
   - Component queries
   - System scheduling

### Phase 4: Root Cause Analysis

Once bottlenecks are identified:

1. **Quantify Impact**
   - What % of total time does it consume?
   - How does it scale with entity count?

2. **Identify Cause**
   - Algorithmic complexity (O(n), O(n²), etc.)
   - Memory bandwidth limitations
   - Insufficient parallelization

3. **Evaluate Solutions**
   - Algorithm improvements (e.g., spatial indexing)
   - Caching strategies
   - Lazy evaluation
   - Parallelization

4. **Implement & Verify**
   - Apply optimization
   - Re-run stress tests
   - Confirm improvement

---

## Expected Results Template

### Test Run: 500 Entities

```
╔════════════════════════════════════════════════════════════════╗
║                    STRESS TEST RESULTS                         ║
╠════════════════════════════════════════════════════════════════╣
│ Entities Spawned: 500
│ Total Ticks: 601
│ Elapsed Time: 60.25 seconds

│ TIMING METRICS:
│   Average Tick Time: 99.58 µs (0.100 ms)
│   Median (P50):      98.20 µs (0.098 ms)
│   P95:               115.40 µs (0.115 ms)
│   P99:               125.30 µs (0.125 ms)
│   Std Dev:           12.45 µs

│ THROUGHPUT:
│   Actual TPS: 9.97 ticks/sec
│   Target TPS: 10.0 ticks/sec
│   Budget Used: 99.6% (10ms budget per tick)
│   Status: GOOD (within budget)

│ TICK TIME DISTRIBUTION:
│   Min:  87.50 µs
│   Max:  187.20 µs
│   Range: 99.70 µs

│ ANALYSIS:
│   ✅ PASS: Meets target throughput
│   ✅ STABLE: Consistent tick performance
╚════════════════════════════════════════════════════════════════╝
```

---

## Investigation Checklist

### Pre-Test Checklist
- [ ] Binary compiled in release mode
- [ ] Config files verified
- [ ] Output directory writable
- [ ] No other heavy processes running
- [ ] System is in stable state (no thermal throttling)

### During Test
- [ ] Monitor system resources (optional: `watch -n 1 'top -bn1'`)
- [ ] Note any anomalies or stutters
- [ ] Check for thermal throttling
- [ ] Verify consistent environment

### Post-Test Analysis
- [ ] Review TPS vs target
- [ ] Check consistency (stddev)
- [ ] Compare across entity counts
- [ ] Identify breakpoint
- [ ] Generate flamegraph if TPS degraded
- [ ] Document findings

---

## Optimization Recommendations

### For Current Implementation

1. **If TPS > 10 for all tests:**
   - System scales well, focus on feature completeness
   - Maintain current architecture

2. **If TPS 9-10 at 500 entities:**
   - Monitor carefully
   - Avoid adding O(n²) operations
   - Consider lazy evaluation

3. **If TPS 8-9 at 500 entities:**
   - Identify hot paths with flamegraph
   - Optimize top 3-5 functions
   - Consider spatial partitioning improvements

4. **If TPS < 8 at 500 entities:**
   - Critical bottleneck exists
   - Requires major optimization
   - Consider algorithm redesign

### Common Optimizations

**Spatial Indexing:**
- Implement hierarchical spatial partitioning
- Use quadtrees or grid-based acceleration
- Cache spatial queries

**Pathfinding:**
- Implement pathfinding caching
- Use simplified pathfinding for high-count scenarios
- Consider crowd simulation techniques

**AI System:**
- Batch utility calculations
- Implement LOD (Level of Detail) AI
- Reduce decision frequency for distant entities

**Memory:**
- Profile allocation patterns
- Reduce per-entity memory footprint
- Use array-of-structs for cache efficiency

---

## Files Generated by This Testing

### Configuration Files
- `config/spawn_config_stress_100.ron` - 100 entity config
- `config/spawn_config_stress_300.ron` - 300 entity config
- `config/spawn_config_stress_test.ron` - 500 entity config (default)
- `config/spawn_config_stress_700.ron` - 700 entity config

### Source Code
- `src/bin/stress_test.rs` - Stress test binary
- `tests/entity_stress_test.rs` - Test framework and validation
- `scripts/run_stress_test.sh` - Automated test runner

### Test Validation
```bash
# Run stress test validation
cargo test --test entity_stress_test --release

# Expected test output includes:
# - Stress scenario configuration validation
# - Config file existence checks
# - Scaling analysis predictions
# - Performance target definitions
# - Bottleneck identification strategy documentation
```

---

## Running the Full Pipeline

### Step 1: Validate Configuration
```bash
cargo test --test entity_stress_test --release -- --nocapture
```

### Step 2: Build Stress Test Binary
```bash
cargo build --bin stress_test --release
```

### Step 3: Run Individual Tests
```bash
# Low load baseline
STRESS_TEST_DURATION=30 \
  STRESS_TEST_CONFIG=config/spawn_config_stress_100.ron \
  cargo run --release --bin stress_test

# Medium load
STRESS_TEST_DURATION=60 \
  STRESS_TEST_CONFIG=config/spawn_config_stress_300.ron \
  cargo run --release --bin stress_test

# High load
STRESS_TEST_DURATION=60 \
  STRESS_TEST_CONFIG=config/spawn_config_stress_test.ron \
  cargo run --release --bin stress_test
```

### Step 4: Run Full Suite
```bash
chmod +x scripts/run_stress_test.sh
./scripts/run_stress_test.sh
```

### Step 5: Analyze Results
1. Compare TPS across entity counts
2. Identify if scaling is linear or degraded
3. Determine if system meets performance targets
4. Document findings and recommendations

### Step 6: Profile if Needed
```bash
# If TPS below target at high entity counts
cargo flamegraph --bin stress_test --release
```

---

## Troubleshooting

### Issue: Binary Won't Compile
**Solution**: Ensure Cargo.toml has the stress_test binary entry:
```toml
[[bin]]
name = "stress_test"
path = "src/bin/stress_test.rs"
```

### Issue: Low TPS at 100 Entities
**Possible Causes:**
- System under heavy load (other applications)
- Debug build instead of release
- Thermal throttling
- Disk I/O contention

**Solution:**
- Close unnecessary applications
- Rebuild with `--release` flag
- Check system temperature
- Try again with fresh system state

### Issue: Inconsistent Results Between Runs
**Possible Causes:**
- System load variance
- Thermal effects
- CPU frequency scaling
- Background tasks

**Solution:**
- Disable CPU frequency scaling if possible
- Close all other applications
- Run multiple times and average results
- Use same conditions for comparable tests

### Issue: Binary Crashes or Hangs
**Solution:**
- Check spawn config file exists
- Verify RON file syntax
- Reduce entity count
- Add more spawn area to reduce spawn failures

---

## Next Phase: Full Simulation Testing

Once stress test results are validated, proceed to full integration tests:

1. **Enable Full Simulation**
   - Add actual AI systems
   - Enable pathfinding
   - Enable vegetation system
   - Measure real-world performance

2. **Create Scenario Tests**
   - Predator-prey equilibrium
   - Reproduction cycles
   - Seasonal variations
   - Long-running stability

3. **Memory Profiling**
   - Track memory usage over time
   - Identify memory leaks
   - Optimize data structures
   - Profile allocations

4. **Extended Duration Tests**
   - 10,000+ tick simulations
   - Check for memory leaks
   - Verify stability
   - Monitor resource usage

---

## References

### Performance Analysis Tools
- **Flamegraph**: Visualize CPU time distribution
- **Perf**: Linux profiler
- **Instruments**: macOS profiler
- **Windows Performance Analyzer**: Windows profiler

### Bevy Documentation
- Bevy Diagnostics: https://docs.rs/bevy/latest/bevy/diagnostic/
- Performance Optimization: https://bevyengine.org/learn/book/getting-started/

### Optimization Resources
- Game Engine Architecture (Jason Gregory)
- Real-Time Rendering (Akenine-Möller et al.)
- GPU Gems series

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2024-12-27 | TDD Agent | Initial stress test infrastructure |

---

*Last Updated: 2024-12-27*
*Status: Ready for testing*
