# Entity Stress Test - Quick Start Guide

## 5-Minute Quick Start

### 1. Build the Stress Test Binary
```bash
cargo build --bin stress_test --release
```

### 2. Run a Quick Test (100 entities)
```bash
STRESS_TEST_DURATION=30 \
  STRESS_TEST_CONFIG=config/spawn_config_stress_100.ron \
  cargo run --release --bin stress_test
```

### 3. Run the Main Test (500 entities)
```bash
STRESS_TEST_DURATION=60 cargo run --release --bin stress_test
```

### 4. Full Test Suite (all entity counts)
```bash
chmod +x scripts/run_stress_test.sh
./scripts/run_stress_test.sh
```

---

## Test Scenarios at a Glance

| Scenario | Entities | Config File | Typical Duration |
|----------|----------|-------------|-----------------|
| Baseline | 100 | `spawn_config_stress_100.ron` | 30 sec |
| Light Load | 300 | `spawn_config_stress_300.ron` | 60 sec |
| **Main Test** | **500** | `spawn_config_stress_test.ron` | **60 sec** |
| Ultra Load | 700 | `spawn_config_stress_700.ron` | 120 sec |

---

## Key Metrics to Watch

### Target Performance
- **TPS (Ticks Per Second)**: Target = 10.0 TPS
- **Tick Time**: Target ≤ 100ms per tick
- **Consistency**: Stddev < 50% of average

### Results Interpretation

```
✅ PASS:        TPS ≥ 9.5  (System meets target)
⚠️  MARGINAL:   TPS 8-9.5  (Below target, needs optimization)
❌ FAIL:         TPS < 8    (Critical bottleneck)
```

---

## One-Command Test Runner

Run the complete test suite with one command:

```bash
# Full suite (4 tests, ~4 minutes total)
./scripts/run_stress_test.sh

# Quick suite (4 tests, ~40 seconds)
./scripts/run_stress_test.sh --quick

# Extended suite (120 second per test)
./scripts/run_stress_test.sh --duration 120
```

---

## Environment Variables

Control test execution with environment variables:

```bash
# Set test duration (seconds)
STRESS_TEST_DURATION=120 cargo run --release --bin stress_test

# Use custom config
STRESS_TEST_CONFIG=config/spawn_config_stress_500.ron \
  cargo run --release --bin stress_test

# Target specific tick count
STRESS_TEST_TICKS=1000 cargo run --release --bin stress_test
```

---

## Typical Output Example

```
╔════════════════════════════════════════════════════════════════╗
║        LIFE SIMULATOR - ENTITY STRESS TEST                     ║
║        Testing performance with 500+ entities                  ║
╚════════════════════════════════════════════════════════════════╝

📋 Stress Test Configuration:
   Duration: 60 seconds
   Target Ticks: 1000
   Config File: config/spawn_config_stress_test.ron

🔧 Setting up stress test environment...
📊 Starting performance measurement...

╭─ TICK MEASUREMENTS ──────────────────────────────────────╮
   Tick 100 - 50000 µs
   Tick 200 - 51234 µs
   Tick 300 - 49876 µs
   ...
╰──────────────────────────────────────────────────────────╯

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
│
│ ANALYSIS:
│   ✅ PASS: Meets target throughput
│   ✅ STABLE: Consistent tick performance
╚════════════════════════════════════════════════════════════════╝
```

---

## Performance Targets

### Expected Results by Entity Count

| Count | Target TPS | Target Tick Time | Notes |
|-------|-----------|-----------------|-------|
| 100 | 10.0+ | <50ms | Baseline |
| 300 | 10.0+ | <75ms | Light load |
| 500 | 10.0 | ~100ms | At budget limit |
| 700 | 8.0+ | <150ms | Stress test |

---

## Troubleshooting

### Binary Not Found
```bash
# Check if binary exists
ls -la target/release/stress_test

# Rebuild if needed
cargo build --bin stress_test --release
```

### Low TPS at 100 Entities
**Cause**: System overloaded

**Fix**:
```bash
# Close other applications
# Run on fresh system
cargo run --release --bin stress_test
```

### Tests Complete Too Quickly
**Cause**: Timeout too short

**Fix**:
```bash
# Increase duration
STRESS_TEST_DURATION=120 cargo run --release --bin stress_test
```

### Config File Not Found
```bash
# Verify config exists
ls -la config/spawn_config_*.ron

# Check file path
pwd  # Should be at /Users/jean/Github/life-simulator
```

---

## Next Steps After Testing

1. **Review Results**
   - Compare TPS across entity counts
   - Check consistency (stddev)
   - Identify scaling behavior

2. **Profile if Needed**
   ```bash
   cargo flamegraph --bin stress_test --release
   ```

3. **Identify Bottlenecks**
   - Read ENTITY_STRESS_TEST_REPORT.md for analysis guide
   - Profile hot paths
   - Review system scaling

4. **Optimize**
   - Implement identified optimizations
   - Re-run tests to verify improvement
   - Document performance improvements

---

## Test Configurations Explained

### spawn_config_stress_100.ron
Minimal baseline test
- 70 rabbits, 20 deer, 8 wolves, 2 foxes
- Small spawn radius (40 tiles)
- Use for: Initial validation, CI/CD baseline

### spawn_config_stress_300.ron
Light production load
- 210 rabbits, 60 deer, 24 wolves, 6 foxes
- Medium spawn radius (50-80 tiles)
- Use for: Typical gameplay scenarios

### spawn_config_stress_test.ron
**Main test scenario**
- 300 rabbits, 100 deer, 80 wolves, 20 foxes
- Large spawn radius (80 tiles)
- Use for: Primary performance target

### spawn_config_stress_700.ron
Maximum load testing
- 490 rabbits, 140 deer, 56 wolves, 14 foxes
- Extra large spawn radius (100 tiles)
- Use for: Finding breaking points

---

## Files Created

| File | Purpose |
|------|---------|
| `src/bin/stress_test.rs` | Stress test binary |
| `tests/entity_stress_test.rs` | Test framework |
| `scripts/run_stress_test.sh` | Automated test suite |
| `config/spawn_config_stress_*.ron` | Test configurations |
| `ENTITY_STRESS_TEST_REPORT.md` | Complete documentation |
| `STRESS_TEST_QUICK_START.md` | This file |

---

## Useful Commands

```bash
# Check compilation
cargo check --bin stress_test

# Build for release
cargo build --bin stress_test --release

# Run tests
cargo test --test entity_stress_test --release -- --nocapture

# Run quick test
STRESS_TEST_DURATION=30 cargo run --release --bin stress_test

# Run with 300 entities
STRESS_TEST_CONFIG=config/spawn_config_stress_300.ron \
  cargo run --release --bin stress_test

# Run with 700 entities (takes longer)
STRESS_TEST_CONFIG=config/spawn_config_stress_700.ron \
  STRESS_TEST_DURATION=120 \
  cargo run --release --bin stress_test

# Full test suite
./scripts/run_stress_test.sh

# Quick suite
./scripts/run_stress_test.sh --quick
```

---

## Performance Baseline

After first run, save results as baseline:

```bash
# Run test and save results
STRESS_TEST_DURATION=60 cargo run --release --bin stress_test > baseline_500_entities.txt

# Compare future runs against this baseline
diff baseline_500_entities.txt current_run.txt
```

---

*For detailed information, see: ENTITY_STRESS_TEST_REPORT.md*
