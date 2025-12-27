# Real-Time Performance Logging - Quick Reference

## What It Does

Logs TPS and performance metrics **every 5 seconds of wall-clock time**, regardless of tick count. Perfect for load testing and monitoring low-TPS scenarios.

## Quick Start

```bash
# Run with logging enabled
RUST_LOG=info cargo run --bin life-simulator

# You'll see output like this every 5 seconds:
# ╔════════════════════════════════════════════════╗
# ║    REAL-TIME PERFORMANCE (5.0s elapsed)      ║
# ╠════════════════════════════════════════════════╣
# ║ Current Tick:              234                 ║
# ║ Entity Count:               42                 ║
# ║ Actual TPS:                9.8                 ║
# ║ Frame Rate:               60.0 FPS            ║
# ║ Speed Multiplier:          1.0x               ║
# ║ Status:              RUNNING                 ║
# ║ Avg Tick Duration:      102.34ms              ║
# ╚════════════════════════════════════════════════╝
```

## Customization

```rust
use life_simulator::simulation::RealtimePerformanceTimer;

// Log every 10 seconds instead of 5
app.insert_resource(RealtimePerformanceTimer::new(10.0));
```

## Key Benefits

1. **Works with 0 TPS** - Shows performance even when simulation is paused or startup is slow
2. **Wall-clock timing** - Independent of tick rate, perfect for load testing
3. **Entity tracking** - See entity count from frame 1
4. **Continuous monitoring** - No gaps in logging during performance issues

## Metrics Explained

- **Current Tick**: Simulation tick counter (increments at target TPS)
- **Entity Count**: Total entities in the world (including internal Bevy entities)
- **Actual TPS**: Measured ticks per second based on timing data
- **Frame Rate**: Rendering/update loop FPS
- **Speed Multiplier**: Simulation speed setting (1.0x = normal, 2.0x = 2x speed)
- **Status**: RUNNING or PAUSED
- **Avg Tick Duration**: Average time per tick over last 60 ticks

## Comparison with Tick-Based Logging

| Feature | Real-Time Logging | Tick-Based Logging |
|---------|-------------------|-------------------|
| Frequency | Every 5 seconds | Every 50 ticks |
| Works at 0 TPS | ✅ Yes | ❌ No |
| Entity count | ✅ Yes | ❌ No |
| Frame rate | ✅ Yes | ❌ No |
| Detailed tick stats | ❌ No | ✅ Yes (min/max) |

**Best Practice**: Use both systems for complete monitoring coverage.

## Testing

```bash
# Unit tests
cargo test --lib simulation::tick::tests::test_realtime_performance_timer

# Integration tests
cargo test --test realtime_performance_logging_test
```

## Files

- **Implementation**: `src/simulation/tick.rs` (lines 353-435)
- **Registration**: `src/simulation/mod.rs`
- **Tests**: `tests/realtime_performance_logging_test.rs`
- **Full docs**: `docs/REALTIME_PERFORMANCE_LOGGING.md`
