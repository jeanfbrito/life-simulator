# sim-logparse - Log Parser for Life Simulator

A high-performance, stream-based log parser CLI tool for analyzing Life Simulator performance metrics and detecting anomalies.

## Features

- **Stream-based parsing** - Efficiently processes 100MB+ log files with <10MB memory overhead
- **TPS metrics extraction** - Extract "TICK PERFORMANCE" lines for performance analysis
- **Entity count tracking** - Track entity spawn messages and population changes
- **Anomaly detection** - Detect TPS drops and stuck entity behaviors
- **Multiple export formats** - JSON, CSV, and human-readable table output
- **Log level filtering** - Filter by info, warn, error, or all levels

## Installation

The tool is part of the Life Simulator workspace. Build it with:

```bash
cargo build -p sim-logparse --release
```

Or run directly:

```bash
cargo run --bin sim-logparse -- <logfile> <command> [options]
```

## CLI Usage

### Extract TPS Metrics

```bash
# Show all TPS metrics
cargo run --bin sim-logparse -- simulation.log metrics

# Show only TPS values
cargo run --bin sim-logparse -- simulation.log metrics --metric tps

# Show entity counts
cargo run --bin sim-logparse -- simulation.log metrics --metric entities

# Show last 100 lines
cargo run --bin sim-logparse -- simulation.log metrics --tail 100
```

### Detect Anomalies

```bash
# Detect TPS drops below threshold (default: 10)
cargo run --bin sim-logparse -- simulation.log anomaly --anomaly-type tps-drop --threshold 10

# Detect stuck entities (requires position tracking in logs)
cargo run --bin sim-logparse -- simulation.log anomaly --anomaly-type stuck-entity --threshold 50
```

### Generate Summary

```bash
# Show summary statistics
cargo run --bin sim-logparse -- simulation.log summary

# Export as JSON
cargo run --bin sim-logparse -- simulation.log summary --export-json

# Export as CSV
cargo run --bin sim-logparse -- simulation.log summary --export-csv
```

### Log Level Filtering

```bash
# Show only warnings and errors
cargo run --bin sim-logparse -- simulation.log --log-level warn metrics

# Show only errors
cargo run --bin sim-logparse -- simulation.log --log-level error metrics
```

## Log Format

The parser expects logs in the following formats:

### TPS Performance Metrics
```
[2025-12-24 10:30:01] TICK PERFORMANCE: TPS=59.8 dt=16.67ms entities=1234 chunks=42
```

Extracted fields:
- `timestamp` - Log timestamp in [YYYY-MM-DD HH:MM:SS] format
- `tps` - Ticks per second (float)
- `dt_ms` - Delta time in milliseconds (float)
- `entities` - Entity count (integer)
- `chunks` - Chunk count (integer)

### Entity Spawn Messages
```
[2025-12-24 10:30:04] Entity spawned: type=Deer count=15 population=1245
```

Extracted fields:
- `entity_type` - Type of entity (string)
- `count` - Number spawned (integer)

## Performance Characteristics

- **Memory Usage**: <10MB for 100MB+ log files (stream-based processing)
- **Processing Speed**: Processes 1M+ log lines per second
- **Scalability**: Linear O(n) complexity in log file size

## Architecture

### Modules

- **`parser.rs`** - Stream-based log parsing and metric extraction
  - `LogParser` - Main parsing orchestrator
  - `extract_tps_metrics()` - TPS metric regex and extraction
  - `extract_entity_count()` - Entity spawn message parsing
  - `matches_log_level()` - Log level filtering

- **`anomaly.rs`** - Anomaly detection algorithms
  - `detect_tps_drops()` - Find TPS values below threshold
  - `is_stuck_entity()` - Detect minimal entity movement
  - `is_completely_stuck()` - Detect zero movement

- **`output.rs`** - Output formatting and export
  - `metrics_to_json()` - JSON export
  - `metrics_to_csv()` - CSV export
  - `metrics_to_table()` - Human-readable table
  - `parse_duration()` - Parse time expressions (5m, 1h, etc.)

- **`cli.rs`** - Command-line interface using clap
  - `Args` - Main CLI arguments
  - `Command` - Subcommand routing (metrics, anomaly, summary)
  - Options structs for each command

## Testing

The tool includes comprehensive unit tests for all functionality:

```bash
# Run all tests
cargo test -p sim-logparse

# Run specific test
cargo test -p sim-logparse test_tps_extraction_from_log_line
```

Test coverage includes:
- TPS extraction regex patterns
- Entity count parsing
- Anomaly detection logic
- Stream processing (memory efficiency)
- Export format validation
- Log level filtering

## Example Output

### Metrics Command
```
2025-12-24 10:30:01: 59.8 TPS (entities: 1234, chunks: 42)
2025-12-24 10:30:02: 60.2 TPS (entities: 1240, chunks: 42)
2025-12-24 10:30:03: 59.5 TPS (entities: 1245, chunks: 42)
```

### Anomaly Detection
```
Found 2 TPS drop anomalies:
  [4] 8.3 TPS at 2025-12-24 10:30:06
  [5] 9.2 TPS at 2025-12-24 10:30:08
```

### Summary
```
=== Log Summary ===
Total metrics: 12
Time range: 2025-12-24 10:30:01 to 2025-12-24 10:30:15

TPS Statistics:
  Average: 50.82
  Min: 8.30
  Max: 60.30

Entity Statistics:
  Average count: 1326
  Total recorded: 2
```

## Future Enhancements

- Support for custom log formats via configuration files
- Time-based filtering (last 5m, since timestamp, etc.)
- Statistical analysis (percentiles, stddev, trends)
- Real-time log streaming support
- Interactive analysis mode
- Performance profiling integration

## License

MIT OR Apache-2.0 (same as Life Simulator)
