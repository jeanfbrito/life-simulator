# Stability Test - Execution Summary

## Test Overview

A long-running stability test is currently executing to validate:
- Memory leak detection over extended runtime
- Cleanup system effectiveness
- Entity lifecycle management
- Relationship cleanup correctness

## Test Parameters

- **Target Duration**: 100,000 ticks (~2.8 hours at 10 TPS)
- **Sample Interval**: Every 5 minutes (300 seconds)
- **Metrics Tracked**:
  - Memory usage (RSS in MB)
  - Tick progress
  - TPS (ticks per second)
  - Runtime and ETA

## Test Architecture

### Components

1. **Life Simulator** (`./target/release/life-simulator`)
   - Runs in headless mode
   - Logs output to `sim_output.log`
   - Executes all cleanup systems:
     - `cleanup_stale_hunting_relationships` (Cleanup set)
     - `cleanup_stale_pack_relationships` (Cleanup set)
     - `cleanup_stale_mating_relationships` (Cleanup set)
     - `cleanup_dead_entities` (every 100 ticks)
     - `cleanup_stale_entities` (periodic)

2. **Stability Monitor** (`stability_monitor_simple.py`)
   - Python script monitoring the simulator process
   - Samples memory usage via `ps` command
   - Parses tick progress from log file
   - Generates samples every 5 minutes
   - Creates final report on completion

### Files Generated

- `stability_test_TIMESTAMP.log` - Monitor log with all samples
- `sim_output.log` - Simulator console output
- `STABILITY_TEST_REPORT_TIMESTAMP.md` - Final analysis report
- `monitor_console.log` - Monitor script console output

## Cleanup Systems Validated

### 1. Hunting Relationships
**System**: `cleanup_stale_hunting_relationships`
**Schedule**: Cleanup set (every tick)
**Purpose**: Remove hunting relationships when prey or predator dies

**Validation**:
- Iterates all ActiveHunter components
- Checks if prey entity still exists
- Removes stale ActiveHunter components
- Cleans up HuntingTarget components

### 2. Pack Relationships
**System**: `cleanup_stale_pack_relationships`
**Schedule**: Cleanup set (every tick)
**Purpose**: Remove pack members when they die/despawn

**Validation**:
- Iterates all PackLeader components
- Validates each member entity exists
- Removes dead members from pack
- Updates pack member count

### 3. Mating Relationships
**System**: `cleanup_stale_mating_relationships`
**Schedule**: Cleanup set (every tick)
**Purpose**: Clean up mating relationships on partner death

**Validation**:
- Checks ActiveMate components
- Validates partner entity exists
- Removes stale mating relationships

### 4. Action Queue Cleanup
**System**: `cleanup_dead_entities`
**Schedule**: Every 100 ticks
**Purpose**: Remove dead entities from action queues

**Validation**:
- Cleans recently_completed HashMap
- Cleans pending VecDeque
- Cleans pending_cancellations HashSet
- Prevents memory buildup in queues

### 5. Replan Queue Cleanup
**System**: `cleanup_stale_entities`
**Schedule**: Periodic (via trigger emitters)
**Purpose**: Remove despawned entities from replan queue

**Validation**:
- Filters high_priority queue
- Filters normal_priority queue
- Removes from dedupe_set

## Expected Memory Pattern

### Healthy Pattern
- **Initial**: ~75-100 MB (baseline + entities)
- **Growth**: Minor linear growth as entities spawn
- **Stabilization**: Should plateau as spawn/death balance
- **Rate**: < 1 MB/min growth acceptable

### Memory Leak Indicators
- **Unbounded growth**: Continuous linear increase
- **High rate**: > 5 MB/min sustained growth
- **No plateau**: Memory never stabilizes
- **Acceleration**: Growth rate increases over time

## Success Criteria

### ✅ Pass Conditions
1. **Completion**: Reaches 100,000 ticks without crash
2. **Memory stable**: Growth rate < 1 MB/min
3. **No accumulation**: Entity count stabilizes
4. **Systems running**: All cleanup systems execute

### ⚠️ Warning Conditions
1. **Moderate growth**: 1-5 MB/min growth rate
2. **High entities**: > 1000 entities sustained
3. **Slow progress**: TPS < 5.0 sustained

### ❌ Fail Conditions
1. **Crash**: Simulator crashes before completion
2. **Memory leak**: > 5 MB/min sustained growth
3. **Entity leak**: Unbounded entity accumulation
4. **System failure**: Cleanup systems not executing

## Monitoring While Running

### Check Progress
```bash
# View current status
tail -10 stability_test_*.log

# Check simulator tick progress
tail -20 sim_output.log | grep "Tick #"

# Monitor process
ps aux | grep life-simulator
```

### Memory Usage
```bash
# Current memory
ps -p $(pgrep life-simulator) -o rss=,pid= | awk '{print $1/1024 " MB (PID " $2 ")"}'
```

### Sample Output
```
[2025-12-27 22:33:51] Sample #1: Tick 0 (0.0%) | Memory: 76.7 MB | Runtime: 0.2m | TPS: 0.0 | ETA: 0.0m
[2025-12-27 22:38:51] Sample #2: Tick 2500 (2.5%) | Memory: 85.3 MB | Runtime: 5.2m | TPS: 8.0 | ETA: 203.1m
[2025-12-27 22:43:51] Sample #3: Tick 5000 (5.0%) | Memory: 88.1 MB | Runtime: 10.2m | TPS: 8.2 | ETA: 193.2m
...
```

## Report Generation

Upon completion (or interruption), the monitor generates:

### STABILITY_TEST_REPORT_TIMESTAMP.md

Contains:
- **Test Parameters**: Duration, ticks reached, TPS
- **Memory Samples Table**: All samples with tick/memory data
- **Memory Growth Analysis**:
  - Initial vs final memory
  - Total growth and percentage
  - Growth rate per minute
  - Leak assessment
- **Cleanup System Validation**: List of all systems
- **Stability Assessment**: Pass/fail evaluation
- **Recommendations**: Action items based on results
- **Simulator Log**: Last 50 lines for debugging

## Interpreting Results

### Memory Growth Analysis

**Example: Healthy**
```
- Initial Memory: 76.7 MB (t=0.2m)
- Final Memory: 95.3 MB (t=170.5m)
- Total Growth: 18.6 MB (+24.3%)
- Growth Rate: 0.109 MB/min

✅ No significant memory leak detected
   Memory usage is stable over time.
```

**Example: Leak Detected**
```
- Initial Memory: 76.7 MB (t=0.2m)
- Final Memory: 245.8 MB (t=170.5m)
- Total Growth: 169.1 MB (+220.5%)
- Growth Rate: 0.993 MB/min

❌ Significant memory leak detected
   Investigation required!
```

### Recommendations

The report will include specific recommendations based on findings:
- Memory leak investigation steps
- Entity count management suggestions
- Performance optimization opportunities
- Cleanup system verification steps

## Post-Test Actions

### If Test Passes
1. Review final memory usage
2. Verify all cleanup systems ran
3. Check entity lifecycle logs
4. Archive report for baseline

### If Test Fails
1. Analyze memory growth pattern
2. Review cleanup system execution
3. Profile memory usage with tools
4. Investigate entity accumulation
5. Check for HashMap/Vec bloat

### Memory Profiling
If leak detected, use:
```bash
# Run with memory profiling
cargo build --release
cargo flamegraph --bin life-simulator

# Or use Valgrind (Linux)
valgrind --tool=massif ./target/release/life-simulator
```

## Current Test Status

- **Started**: 2025-12-27 22:33:41
- **PID**: Check `cat /tmp/stability_monitor.pid`
- **Progress**: Check `tail stability_test_*.log`
- **ETA**: ~2.8 hours from start (if 10 TPS maintained)

## Cleanup After Test

```bash
# Stop test manually if needed
kill $(cat /tmp/stability_monitor.pid)

# Clean up log files
mv stability_test_*.log results/
mv STABILITY_TEST_REPORT_*.md results/
mv sim_output.log results/sim_output_stability.log
```

---

*Test configuration: 100,000 ticks, 5-minute sampling, macOS memory monitoring*
*For questions or issues, check the monitor log and simulator output*
