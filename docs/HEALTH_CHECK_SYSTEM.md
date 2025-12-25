# Health Check System Documentation

## Overview

The Health Check System (`src/debug/health_checks.rs`) provides real-time monitoring and alerting for simulation performance and entity health. It detects critical issues like TPS drops, stuck entities, population crashes, and AI loops.

## Features

### Health Alerts

The system detects four types of health issues:

1. **TpsBelow10** - TPS drops below 10 (minimum healthy threshold)
2. **EntitiesStuck** - Entities haven't moved for 50+ ticks
3. **PopulationCrash** - Population loses 50%+ entities in 100 ticks
4. **AiLoops** - Same AI action repeats 20+ times on an entity

### Alert Storage

- **Ring Buffer**: Stores up to 100 most recent alerts with timestamps
- **Automatic Cleanup**: Old alerts are automatically removed when buffer is full
- **Tick Tracking**: Each alert records the exact simulation tick it occurred

### Performance Monitoring

The system includes helper methods for web server integration:

- `get_health_summary()` - Returns health status as JSON
- `get_alert_counts()` - Returns count of each alert type
- `is_healthy()` - Boolean check for overall system health

## Configuration

All thresholds are configurable as constants at the top of `health_checks.rs`:

```rust
const MIN_HEALTHY_TPS: f64 = 10.0;                      // TPS alert threshold
const STUCK_ENTITY_THRESHOLD_TICKS: u64 = 50;          // Stuck entity detection
const POPULATION_CRASH_THRESHOLD: f32 = 50.0;          // Population loss %
const AI_LOOP_REPEAT_THRESHOLD: u32 = 20;              // AI action repeats
const POPULATION_WINDOW_TICKS: u64 = 100;              // Population monitoring window
```

## API Reference

### HealthChecker Resource

Main resource for health checking. Automatically inserted when `HealthCheckPlugin` is added.

#### Methods

```rust
// Alert management
pub fn add_alert(&mut self, alert: HealthAlert, tick: u64)
pub fn get_alerts(&self) -> Vec<AlertRecord>
pub fn get_latest_alert(&self, alert_type: HealthAlert) -> Option<AlertRecord>
pub fn count_alerts_in_window(&self, alert_type: HealthAlert, window_ticks: u64, current_tick: u64) -> usize
pub fn clear_alerts(&mut self)

// TPS checking
pub fn check_tps(&mut self, tps: f64, tick: u64) -> bool

// Stuck entity detection
pub fn update_entity_position(&mut self, entity_id: u32, position: (i32, i32), tick: u64)
pub fn check_stuck_entities(&mut self, current_tick: u64) -> bool

// Population crash detection
pub fn update_population(&mut self, entity_count: u32, tick: u64)
pub fn check_population_crash(&mut self, current_tick: u64) -> bool

// AI loop detection
pub fn update_entity_action(&mut self, entity_id: u32, action: String)
pub fn check_ai_loops(&mut self, current_tick: u64) -> bool
pub fn reset_action_counters(&mut self)

// Cleanup
pub fn cleanup_old_states(&mut self)

// Web server integration
pub fn get_health_summary(&self) -> serde_json::Value
pub fn get_alert_counts(&self) -> HashMap<&'static str, usize>
pub fn is_healthy(&self) -> bool
```

### HealthAlert Enum

```rust
pub enum HealthAlert {
    TpsBelow10,        // TPS dropped below 10
    EntitiesStuck,     // Entities haven't moved in 50+ ticks
    PopulationCrash,   // Population lost 50%+ in 100 ticks
    AiLoops,           // Same action repeated 20+ times
}
```

### AlertRecord

```rust
pub struct AlertRecord {
    pub alert_type: HealthAlert,      // Type of alert
    pub tick: u64,                     // Tick when alert occurred
    pub timestamp_ms: u64,             // System timestamp in milliseconds
}
```

### EntityHealthState

Internal tracking for individual entities:

```rust
pub struct EntityHealthState {
    pub last_position: (i32, i32),           // Last known position
    pub last_position_update_tick: u64,      // When position was last updated
    pub current_action: String,              // Current AI action
    pub action_repeat_count: u32,            // How many times action repeated
}
```

## Usage

### Basic Setup

The plugin is already registered in `src/main.rs`:

```rust
.add_plugins((
    SimulationPlugin,
    EntitiesPlugin,
    TQUAIPlugin,
    VegetationPlugin,
    HealthCheckPlugin,  // Health monitoring
))
```

### Accessing Health Status

In any system, you can access the health checker:

```rust
fn my_system(health_checker: Res<HealthChecker>) {
    let alerts = health_checker.get_alerts();
    let summary = health_checker.get_health_summary();
    let is_healthy = health_checker.is_healthy();
}
```

### Updating Entity Tracking

To track entity positions and actions (should be done in entity movement/AI systems):

```rust
fn update_health_tracking(
    mut health_checker: ResMut<HealthChecker>,
    tick: Res<SimulationTick>,
    query: Query<(Entity, &TilePosition, &CurrentAction)>,
) {
    for (entity, position, action) in query.iter() {
        let entity_id = entity.index();
        health_checker.update_entity_position(entity_id, (position.x, position.y), tick.get());
        health_checker.update_entity_action(entity_id, format!("{:?}", action));
    }
}
```

### Web Server Integration

The health check system is designed for web server integration. The JSON summary can be exposed via API:

```rust
let summary = health_checker.get_health_summary();
// Returns:
{
    "total_alerts": 5,
    "recent_alerts": [
        {
            "type": "TPS below 10",
            "tick": 250,
            "timestamp_ms": 1234567890
        },
        // ... more alerts
    ],
    "entity_states_count": 128,
    "population_history_len": 50
}
```

## Monitoring Schedule

The health check system runs every 50 ticks (5 seconds at 10 TPS):

```rust
health_check_system.run_if(every_50_ticks)  // Runs when tick % 50 == 0
```

This can be adjusted by modifying the `every_50_ticks` condition or the system scheduling.

## Integration Points

The system is designed to be extended with these integration points:

1. **Entity Tracking** - Needs to be connected to movement and AI systems
2. **Population Tracking** - Needs entity count updates from entity manager
3. **Web Server API** - Health summary can be exposed via `/health` endpoint
4. **Logging** - Alerts are automatically logged with `info!()` macro

## Test Coverage

The system includes 16 comprehensive unit tests covering:

- Alert creation and storage
- Ring buffer management
- TPS threshold detection
- Stuck entity detection
- Population crash detection
- AI loop detection
- Alert windowing and counting
- Health summary generation
- Mixed alert scenarios

Run tests with:

```bash
cargo test --lib debug::health_checks
```

## Performance Considerations

- **Memory**: Ring buffer limited to 100 alerts (minimal overhead)
- **Entity States**: Cleared every 50 ticks to prevent memory leaks
- **Population History**: Window limited to 100 ticks
- **CPU**: Checks run once every 50 ticks, not every frame

## Future Enhancements

Potential improvements for the health check system:

1. **Custom Alert Thresholds** - Make thresholds configurable via resources
2. **Alert Severity Levels** - Add warning/critical levels
3. **Historical Trending** - Track alert frequency over longer periods
4. **Performance Metrics** - Add CPU/memory usage monitoring
5. **Recovery Suggestions** - Provide actionable recommendations for alerts
6. **Web Dashboard** - Interactive health monitoring UI

## Files

- **Implementation**: `/Users/jean/Github/life-simulator/src/debug/health_checks.rs`
- **Module Export**: `/Users/jean/Github/life-simulator/src/debug/mod.rs`
- **Library Export**: `/Users/jean/Github/life-simulator/src/lib.rs` (line 3)
- **Main Plugin**: `/Users/jean/Github/life-simulator/src/main.rs` (lines 7, 21, 50)

## Examples

### Example 1: Check for Critical Alerts

```rust
fn alert_check_system(health_checker: Res<HealthChecker>, tick: Res<SimulationTick>) {
    let current_tick = tick.get();

    // Check if we had TPS issues in last 100 ticks
    let tps_alerts = health_checker.count_alerts_in_window(
        HealthAlert::TpsBelow10,
        100,
        current_tick,
    );

    if tps_alerts > 3 {
        warn!("Multiple TPS drops detected!");
    }
}
```

### Example 2: Get Alert Summary

```rust
fn report_health(health_checker: Res<HealthChecker>) {
    let counts = health_checker.get_alert_counts();
    let summary = health_checker.get_health_summary();

    println!("Health Report:");
    println!("  Total alerts: {}", summary["total_alerts"]);
    println!("  TPS alerts: {}", counts.get("tps_below_10").unwrap_or(&0));
    println!("  Stuck entities: {}", counts.get("entities_stuck").unwrap_or(&0));
}
```

### Example 3: Reset on Recovery

```rust
fn recovery_system(mut health_checker: ResMut<HealthChecker>) {
    if health_checker.is_healthy() {
        health_checker.clear_alerts();
    }
}
```
