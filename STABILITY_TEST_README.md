# Long-Running Stability Testing

## Overview

This directory contains tools for running long-duration stability tests to validate:
- Memory leak detection over 100,000 ticks (~2.8 hours)
- Cleanup system effectiveness
- Entity lifecycle management
- Relationship cleanup correctness

## Test Implementation

### Monitoring Script

**File**: `stability_monitor_simple.py`

A Python script that:
- Launches the simulator in headless mode
- Samples memory usage every 5 minutes
- Parses tick progress from logs
- Generates comprehensive stability report

**Features**:
- No external Python dependencies (uses only stdlib)
- Cross-platform memory monitoring (macOS/Linux)
- Graceful handling of interruptions
- Detailed memory growth analysis

### How to Run

```bash
# 1. Build the simulator in release mode
cargo build --release --bin life-simulator

# 2. Start the stability test
python3 stability_monitor_simple.py

# The test will run for ~2.8 hours
# Progress is logged every 5 minutes
```

### Monitor Progress

While the test is running:

```bash
# View current progress
tail -f stability_test_*.log

# Check simulator ticks
tail -f sim_output.log | grep "Tick #"

# Check memory usage
ps -p $(pgrep life-simulator) -o rss=,pid= | awk '{print $1/1024 " MB"}'
```

### Stop Test Early

```bash
# Graceful stop (generates report)
kill $(cat /tmp/stability_monitor.pid)

# Force kill
pkill -f life-simulator
pkill -f stability_monitor
```

## Generated Files

### During Test
- `stability_test_TIMESTAMP.log` - Monitor samples and progress
- `sim_output.log` - Simulator console output
- `monitor_console.log` - Monitor script console output

### After Completion
- `STABILITY_TEST_REPORT_TIMESTAMP.md` - Comprehensive analysis report

## Cleanup Systems Validated

The test validates these cleanup systems are working correctly:

1. **Hunting Relationships** (`cleanup_stale_hunting_relationships`)
   - Runs: Every tick in Cleanup set
   - Purpose: Remove hunting relationships when entities die
   - Validated: Stale ActiveHunter components are removed

2. **Pack Relationships** (`cleanup_stale_pack_relationships`)
   - Runs: Every tick in Cleanup set
   - Purpose: Clean up pack members on death
   - Validated: Dead members removed from packs

3. **Mating Relationships** (`cleanup_stale_mating_relationships`)
   - Runs: Every tick in Cleanup set
   - Purpose: Clean up mating pairs on partner death
   - Validated: Stale ActiveMate components removed

4. **Action Queue** (`cleanup_dead_entities`)
   - Runs: Every 100 ticks
   - Purpose: Remove dead entities from action queues
   - Validated: HashMaps and VecDeques cleaned

5. **Replan Queue** (`cleanup_stale_entities`)
   - Runs: Periodically via trigger emitters
   - Purpose: Remove despawned entities from replan queue
   - Validated: Stale replan requests removed

## Memory Leak Detection

### Expected Patterns

**Healthy (No Leak)**:
- Initial: ~75-100 MB
- Growth: < 1 MB/min
- Pattern: Stabilizes after initial entity spawning
- Final: 95-150 MB (depending on entity count)

**Minor Growth (Acceptable)**:
- Growth: 1-5 MB/min
- Pattern: Linear but slow
- Likely: Normal entity spawning variation

**Memory Leak (Problem)**:
- Growth: > 5 MB/min
- Pattern: Continuous unbounded growth
- Action Required: Investigation and fixes

### Analysis Methodology

The monitor calculates:
1. **Total Growth**: Final memory - Initial memory
2. **Growth Percentage**: (Growth / Initial) * 100
3. **Growth Rate**: Growth / Runtime (MB/min)
4. **Growth Pattern**: First half vs second half

### Leak Assessment Criteria

```
if growth_rate < 0.1 MB/min:
    ✅ No significant memory leak
elif growth_rate < 1.0 MB/min:
    ⚠️ Minor memory growth - monitor further
else:
    ❌ Significant memory leak - investigate
```

## Stability Report

The generated report includes:

### 1. Test Parameters
- Target and actual ticks reached
- Runtime and average TPS
- Number of samples collected

### 2. Memory Samples Table
```
| Sample | Time (min) | Tick | Memory (MB) |
|--------|------------|------|-------------|
| 1      | 0.2        | 0    | 76.7        |
| 2      | 5.2        | 2500 | 85.3        |
...
```

### 3. Memory Growth Analysis
- Initial vs final memory
- Total growth and percentage
- Growth rate calculation
- Leak assessment verdict

### 4. Cleanup System Validation
- List of all cleanup systems
- Execution schedule for each
- Validation confirmation

### 5. Stability Assessment
- Target achievement status
- Crash/error detection
- System operational status
- Memory usage assessment

### 6. Recommendations
- Specific action items based on results
- Memory optimization suggestions
- Entity management recommendations
- Further investigation steps

### 7. Simulator Log Excerpt
- Last 50 lines for debugging
- Error detection
- Performance insights

## Interpreting Results

### Success Indicators
- ✅ Completes all 100,000 ticks
- ✅ Memory growth < 1 MB/min
- ✅ No crashes or panics
- ✅ Entity count stabilizes
- ✅ TPS remains consistent

### Warning Signs
- ⚠️ Growth 1-5 MB/min (monitor)
- ⚠️ High entity count (>1000)
- ⚠️ TPS degradation over time

### Failure Indicators
- ❌ Crash before completion
- ❌ Growth > 5 MB/min (leak)
- ❌ Unbounded entity accumulation
- ❌ System errors in logs

## Troubleshooting

### Test Won't Start
**Issue**: Python module errors
**Fix**: Uses only Python stdlib - check Python 3.x installed

**Issue**: Simulator won't launch
**Fix**: Build with `cargo build --release --bin life-simulator`

**Issue**: Permission denied
**Fix**: `chmod +x stability_monitor_simple.py`

### Test Stops Early
**Issue**: Simulator crashes
**Fix**: Check `sim_output.log` for errors - likely a bug in simulator

**Issue**: System resource limits
**Fix**: Check `ulimit -a` and adjust if needed

### Memory Not Tracked
**Issue**: macOS/Linux `ps` command unavailable
**Fix**: Monitor will show "None" for memory - still tracks ticks

### No Report Generated
**Issue**: Script killed forcefully
**Fix**: Script must terminate gracefully to generate report

## Advanced Usage

### Custom Duration
Edit `stability_monitor_simple.py`:
```python
TARGET_TICKS = 50_000  # Shorter test
TARGET_TICKS = 200_000  # Longer test
```

### Custom Sample Interval
```python
SAMPLE_INTERVAL = 60  # Sample every 1 minute
SAMPLE_INTERVAL = 600  # Sample every 10 minutes
```

### Multiple Tests
```bash
# Run multiple tests in sequence
for i in {1..3}; do
    python3 stability_monitor_simple.py
    sleep 60  # Cool-down between tests
done
```

### Baseline Comparison
```bash
# Save baseline
cp STABILITY_TEST_REPORT_*.md baseline_report.md

# Compare after changes
diff baseline_report.md STABILITY_TEST_REPORT_latest.md
```

## Files Overview

### Test Scripts
- `stability_monitor_simple.py` - Main monitoring script (no deps)
- `stability_monitor.py` - Advanced version (requires `requests`)
- `run_stability_test.sh` - Bash wrapper script

### Documentation
- `STABILITY_TEST_README.md` - This file
- `STABILITY_TEST_SUMMARY.md` - Detailed test architecture

### Configuration
- No config files - edit Python scripts directly

## Cleanup Systems Code References

### Implementation Files
```
src/ai/mod.rs - TQUAIPlugin registers cleanup systems
src/ai/hunting_relationship_system.rs - cleanup_stale_hunting_relationships
src/ai/pack_relationship_system.rs - cleanup_stale_pack_relationships
src/ai/mating_relationship_system.rs - cleanup_stale_mating_relationships
src/ai/queue.rs - cleanup_dead_entities
src/ai/replan_queue.rs - cleanup_stale_entities
```

### System Schedule
```rust
// In TQUAIPlugin::build()
.add_systems(
    Update,
    (
        cleanup_stale_hunting_relationships,
        cleanup_stale_pack_relationships,
        cleanup_stale_mating_relationships,
    )
    .in_set(SimulationSet::Cleanup)
    .run_if(should_tick),
)
```

## Known Issues

### Current Blockers
1. **System Parameter Conflicts**: The current simulator has Bevy ECS system parameter conflicts that cause crashes
   - **Issue**: Multiple systems trying to access `&World` mutably
   - **Workaround**: The stability_test binary was created but has similar issues
   - **Resolution**: Needs careful system parameter refactoring

2. **Ecosystem Test Spawn**: The default spawn config is for ecosystem testing (10,000 ticks)
   - **Impact**: Test expects 100,000 ticks but entities are configured for shorter duration
   - **Workaround**: Edit spawn config or use different world

### Temporary Solutions
1. Fix system parameter conflicts in main simulator
2. Create dedicated stability test spawn configuration
3. Or run test with existing world state (no spawning)

## Future Improvements

### Short Term
- Fix Bevy system parameter conflicts
- Create stability-specific spawn configuration
- Add entity count tracking in monitor

### Medium Term
- Integrate with web API for real-time monitoring
- Add entity lifecycle event tracking
- Generate memory growth graphs

### Long Term
- Automated baseline comparison
- CI/CD integration for pre-release testing
- Performance regression detection
- Multi-platform testing (macOS, Linux, Windows)

## Questions & Support

For issues or questions:
1. Check `sim_output.log` for simulator errors
2. Check `stability_test_*.log` for monitor status
3. Review cleanup system code in `src/ai/`
4. Check git history for recent changes that might affect stability

---

**Last Updated**: 2025-12-27
**Test Version**: 1.0
**Target Duration**: 100,000 ticks (~2.8 hours)
