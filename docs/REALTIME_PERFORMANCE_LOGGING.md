# Real-Time Performance Logging

## Overview

The simulation now includes a real-time performance monitoring system that logs TPS and other metrics every 5-10 seconds of wall-clock time, independent of tick count. This is particularly useful for load testing and monitoring performance during periods of low or zero TPS.

## Features

- **Wall-clock based logging**: Logs every 5 seconds (configurable) regardless of simulation state
- **Works with zero TPS**: Unlike tick-based logging, this works even when no ticks are executing
- **Comprehensive metrics**:
  - Current tick number
  - Entity count
  - Actual TPS (ticks per second)
  - Frame rate (FPS)
  - Speed multiplier
  - Simulation status (RUNNING/PAUSED)
  - Average tick duration in milliseconds

## Implementation

### Core Components

1. **RealtimePerformanceTimer** - Resource that tracks wall-clock time between logs
2. **log_realtime_performance** - System that runs every frame and checks if logging should occur

### Configuration

The default log interval is 5 seconds. To customize:

```rust
use life_simulator::simulation::RealtimePerformanceTimer;

// In your app setup:
app.insert_resource(RealtimePerformanceTimer::new(10.0)); // Log every 10 seconds
```

### Example Output

```
╔════════════════════════════════════════════════╗
║    REAL-TIME PERFORMANCE (5.2s elapsed)      ║
╠════════════════════════════════════════════════╣
║ Current Tick:              234                 ║
║ Entity Count:               42                 ║
║ Actual TPS:                9.8                 ║
║ Frame Rate:               60.0 FPS            ║
║ Speed Multiplier:          1.0x               ║
║ Status:              RUNNING                 ║
║ Avg Tick Duration:      102.34ms              ║
╚════════════════════════════════════════════════╝
```

## Use Cases

### Load Testing

Monitor performance continuously during stress tests without waiting for tick milestones:

```bash
RUST_LOG=info cargo run --bin life-simulator --release
```

The system will log every 5 seconds, showing exactly how the simulation performs under load.

### Debugging Startup Issues

The old tick-based logging showed "0.0 TPS" during early startup because not enough ticks had accumulated. The new system provides immediate feedback:

- Shows actual frame rate even when TPS is 0
- Displays entity count from frame 1
- Reports simulation status (paused vs running)

### Performance Regression Detection

When combined with the tick-based logging (every 50 ticks), you get:
- **Short-term view**: Real-time logging every 5 seconds
- **Long-term view**: Tick-based logging at regular tick intervals

This dual system helps identify:
- Sudden performance drops (real-time)
- Long-term performance trends (tick-based)

## TDD Implementation

This feature was implemented using Test-Driven Development:

1. **RED**: Created failing test `test_realtime_performance_timer`
2. **GREEN**: Implemented `RealtimePerformanceTimer` and `log_realtime_performance` system
3. **REFACTOR**: Added entity count tracking and integration tests

### Tests

```bash
# Unit tests
cargo test --lib simulation::tick::tests::test_realtime_performance_timer

# Integration tests
cargo test --test realtime_performance_logging_test
```

## Files Modified

- `src/simulation/tick.rs` - Added timer resource and logging system
- `src/simulation/mod.rs` - Registered new resource and system
- `tests/realtime_performance_logging_test.rs` - Integration tests

## Future Enhancements

Potential improvements:
- Configurable metrics (choose which stats to log)
- Export to CSV for analysis
- Adaptive logging interval based on performance
- Integration with telemetry systems (OpenTelemetry, Prometheus)
