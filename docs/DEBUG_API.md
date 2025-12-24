# Debug API Endpoints

This document describes the debug API endpoints that expose health check system data through HTTP.

## Overview

The debug API provides three main endpoints for monitoring simulation health and performance:

1. **Health Status** - Overall health assessment and alert counts
2. **Recent Alerts** - Detailed list of recent health alerts
3. **TPS Monitoring** - Current and average ticks-per-second performance

All endpoints return JSON-formatted responses and are accessed through the web server running on the configured port (default: 54321).

## Architecture

### Data Flow

```
Bevy World (HealthChecker)
    ↓
HealthCheckApiPlugin updates every 50 ticks
    ↓
HealthCheckApi snapshots current state
    ↓
Global instance accessible from web server thread
    ↓
Web server serves JSON via HTTP endpoints
```

### Key Components

- **HealthChecker** (`src/debug/health_checks.rs`): Tracks simulation metrics
- **HealthCheckApi** (`src/debug/api.rs`): Thread-safe wrapper for web access
- **HealthCheckSnapshot**: Immutable snapshot of health data at a point in time
- **Web Server** (`src/web_server_simple.rs`): Serves HTTP endpoints

## Endpoints

### 1. GET /api/debug/health

Returns the overall health status with alert counts.

**Response Format:**
```json
{
  "status": "ok|degraded|critical",
  "alerts": {
    "tps_below_10": 0,
    "entities_stuck": 0,
    "population_crash": 0,
    "ai_loops": 0
  },
  "current_tps": 60.0,
  "total_alerts": 0,
  "is_healthy": true
}
```

**Status Meanings:**
- `ok`: System is healthy (no critical alerts, TPS >= 10)
- `degraded`: System has issues (TPS between 5-10 or has non-critical alerts)
- `critical`: System is in trouble (TPS < 5)

**Alert Types:**
- `tps_below_10`: Number of times TPS dropped below 10
- `entities_stuck`: Number of times entities were detected as stuck
- `population_crash`: Number of times population crashed 50%+ in 100 ticks
- `ai_loops`: Number of times AI infinite loops were detected

**Example:**
```bash
curl http://127.0.0.1:54321/api/debug/health
```

### 2. GET /api/debug/alerts

Returns a list of recent health alerts (last 100 max).

**Response Format:**
```json
{
  "alerts": [
    {
      "tick": 1000,
      "type": "TPS below 10",
      "timestamp_ms": 1640000000000,
      "message": "TPS below 10 at tick 1000"
    },
    {
      "tick": 1050,
      "type": "Entities stuck",
      "timestamp_ms": 1640000000050,
      "message": "Entities stuck at tick 1050"
    }
  ],
  "total": 2
}
```

**Alert Types:**
- `TPS below 10`: Ticks-per-second fell below 10 (performance issue)
- `Entities stuck`: One or more entities haven't moved in 50+ ticks
- `Population crash`: Population lost 50%+ entities in 100 ticks (ecosystem collapse)
- `AI loops detected`: Entity repeated same action 20+ times (AI bug)

**Limits:**
- Returns maximum 100 most recent alerts
- Alerts are sorted newest first
- Include both tick number and Unix timestamp in milliseconds

**Example:**
```bash
curl http://127.0.0.1:54321/api/debug/alerts
```

### 3. GET /api/debug/tps

Returns current and average TPS with performance status.

**Response Format:**
```json
{
  "current_tps": 59.5,
  "average_tps": 59.5,
  "status": "excellent"
}
```

**Status Meanings:**
- `excellent`: TPS >= 59 (optimal performance)
- `good`: TPS 30-59 (good performance)
- `ok`: TPS 10-30 (acceptable performance)
- `degraded`: TPS < 10 (poor performance)

**Notes:**
- `current_tps`: TPS as of the most recent health check (updated every 50 ticks)
- `average_tps`: Average TPS over recent period
- Status is based on `current_tps` threshold

**Example:**
```bash
curl http://127.0.0.1:54321/api/debug/tps
```

## Testing

### Unit Tests

Unit tests are in `src/debug/api.rs` and verify:

- JSON serialization format
- Snapshot creation and updates
- API wrapper functionality
- Status classification logic
- Alert formatting

Run with:
```bash
cargo test --lib debug::api
```

### Integration Tests

Integration tests are in `tests/web_server_debug_endpoints.rs` and verify:

- Complete JSON response structures
- All required fields and their types
- Status value ranges and meanings
- Alert type formats
- Timestamp handling
- Edge cases (empty lists, multiple alerts, etc.)

Run with:
```bash
cargo test --test web_server_debug_endpoints
```

## Data Update Frequency

- Health checks run **every 50 ticks** in simulation time
- Health snapshots are updated after each check
- Web API serves the most recent snapshot
- Timestamps are captured when alerts are created (per health check system)

## Implementation Details

### Thread Safety

The implementation uses:
- `Arc<RwLock<T>>` for thread-safe data sharing
- `OnceLock` for lazy initialization of global instance
- Snapshot pattern to prevent data races

### Web Server Integration

- Endpoints are registered in `handle_connection()` function
- Simple pattern matching on request path
- JSON serialization via `serde_json`
- No external dependencies beyond Bevy/serde

### Performance

- Minimal overhead from snapshot creation
- Updates only every 50 ticks (not every frame)
- Lightweight JSON serialization
- Read-only access from web server thread

## Usage Examples

### Check if system is healthy

```bash
curl http://127.0.0.1:54321/api/debug/health | jq '.is_healthy'
```

### Get all alert types with counts

```bash
curl http://127.0.0.1:54321/api/debug/health | jq '.alerts'
```

### Monitor recent performance issues

```bash
curl http://127.0.0.1:54321/api/debug/alerts | jq '.alerts[] | select(.type | contains("TPS"))'
```

### Check current performance status

```bash
curl http://127.0.0.1:54321/api/debug/tps | jq '.status'
```

### Watch TPS in real-time

```bash
watch -n 1 'curl -s http://127.0.0.1:54321/api/debug/tps | jq ".current_tps"'
```

## Integration with Existing Endpoints

The debug endpoints are fully independent and do not interfere with:

- `/api/entities` - Entity position data
- `/api/species` - Species metadata
- `/api/vegetation/*` - Vegetation system metrics
- `/api/chunks` - World terrain data
- All other existing endpoints

All endpoints coexist in the same HTTP routing table.

## Future Enhancements

Potential improvements for future versions:

1. **Historical Data**: Store longer alert history and performance trends
2. **Real-time WebSocket**: Stream health updates without polling
3. **Configurable Thresholds**: Allow dynamic adjustment of alert triggers
4. **Performance Graphs**: Time-series visualization of TPS trends
5. **Alert Filtering**: Query alerts by type or time range
6. **Health Report**: Comprehensive system analysis and recommendations
