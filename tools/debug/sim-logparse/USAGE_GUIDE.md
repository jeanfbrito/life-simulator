# sim-logparse Usage Guide

## Quick Start

### 1. Build the Tool

```bash
cd /Users/jean/Github/life-simulator
cargo build -p sim-logparse --release
```

### 2. Run with Your Logs

```bash
# Using cargo directly
cargo run -p sim-logparse -- /path/to/your.log <command> [options]

# Or use the wrapper script
./tools/debug/sim-logparse/run.sh /path/to/your.log <command> [options]
```

## Common Use Cases

### Analyze Simulation Performance Over Time

```bash
# View all TPS metrics
cargo run -p sim-logparse -- simulation.log metrics --metric tps

# Get statistics
cargo run -p sim-logparse -- simulation.log summary
```

**Output:**
```
2025-12-24 10:30:01: 59.8 TPS (entities: 1234, chunks: 42)
2025-12-24 10:30:02: 60.2 TPS (entities: 1240, chunks: 42)
2025-12-24 10:30:03: 59.5 TPS (entities: 1245, chunks: 42)
```

### Find Performance Issues

```bash
# Detect TPS drops below 10 FPS
cargo run -p sim-logparse -- simulation.log anomaly --anomaly-type tps-drop --threshold 10

# Adjust threshold for your target performance
cargo run -p sim-logparse -- simulation.log anomaly --anomaly-type tps-drop --threshold 30
```

**Output:**
```
Found 2 TPS drop anomalies:
  [4] 8.3 TPS at 2025-12-24 10:30:06
  [5] 9.2 TPS at 2025-12-24 10:30:08
```

### Export Data for Analysis

```bash
# Export as JSON for processing
cargo run -p sim-logparse -- simulation.log summary --export-json > metrics.json

# Export as CSV for spreadsheet analysis
cargo run -p sim-logparse -- simulation.log summary --export-csv > metrics.csv
```

### Filter Logs by Level

```bash
# Show only warnings and errors
cargo run -p sim-logparse -- simulation.log --log-level warn metrics

# Show only errors
cargo run -p sim-logparse -- simulation.log --log-level error metrics

# All levels (default)
cargo run -p sim-logparse -- simulation.log --log-level all metrics
```

### Tail Recent Logs

```bash
# Show last 100 TPS metrics
cargo run -p sim-logparse -- simulation.log metrics --tail 100

# Show last 50 metrics with details
cargo run -p sim-logparse -- simulation.log metrics --tail 50 --metric tps
```

## Real-World Examples

### Monitor Live Simulation

To monitor a running simulation in real-time:

```bash
# In one terminal, start the simulation with logging
RUST_LOG=info cargo run --bin life-simulator > simulation.log 2>&1

# In another terminal, periodically check for anomalies
watch -n 5 "cargo run -p sim-logparse -- simulation.log anomaly --anomaly-type tps-drop --threshold 10"
```

### Generate Performance Report

```bash
# Create a comprehensive performance analysis
cargo run -p sim-logparse -- simulation.log summary --export-json > report.json
cargo run -p sim-logparse -- simulation.log summary --export-csv > report.csv
```

### Find When Performance Degrades

```bash
# Check full metrics with timestamps
cargo run -p sim-logparse -- simulation.log metrics --metric tps | head -20
cargo run -p sim-logparse -- simulation.log metrics --metric tps | tail -20
```

### Entity Population Analysis

```bash
# Track entity spawning
cargo run -p sim-logparse -- simulation.log metrics --metric entities
```

## Log Format Requirements

For the tool to work with your logs, ensure they match these patterns:

### Performance Metrics (Required for TPS analysis)
```
[YYYY-MM-DD HH:MM:SS] TICK PERFORMANCE: TPS=XX.X dt=XX.XXms entities=XXXX chunks=XX
```

Example:
```
[2025-12-24 10:30:01] TICK PERFORMANCE: TPS=59.8 dt=16.67ms entities=1234 chunks=42
```

### Entity Spawning (Optional for entity tracking)
```
[YYYY-MM-DD HH:MM:SS] Entity spawned: type=TYPENAME count=XX population=XXXX
```

Example:
```
[2025-12-24 10:30:04] Entity spawned: type=Deer count=15 population=1245
```

## Interpreting Results

### TPS (Ticks Per Second)

- **60 TPS** - Normal simulation speed (60 FPS equivalent)
- **30-59 TPS** - Slight slowdown, acceptable performance
- **10-29 TPS** - Noticeable lag, performance issue
- **<10 TPS** - Severe lag, critical performance problem

### Entity Count

- Tracks the number of active entities in the simulation
- High entity counts may correlate with TPS drops
- Useful for identifying when performance breaks occur

### Chunk Count

- Number of terrain chunks being managed
- High chunk counts with low TPS indicates rendering/memory issues
- Stable chunk count with TPS drops suggests simulation logic issues

## Troubleshooting

### Tool doesn't find any metrics

Check that your log format matches the expected patterns:
```bash
# Search for TICK PERFORMANCE lines
grep "TICK PERFORMANCE" simulation.log | head -5
```

### Missing entity data

Entity spawning messages are optional. If not logged:
```bash
# Verify logging is enabled
RUST_LOG=info cargo run --bin life-simulator
```

### Performance analysis looks wrong

Verify the log file contains actual performance data:
```bash
# Check file size and recent entries
tail -100 simulation.log
wc -l simulation.log
```

## Optimizing for Large Files

The parser is stream-based and memory-efficient, but for very large files:

```bash
# Use tail to get recent logs
tail -100000 simulation.log > recent.log
cargo run -p sim-logparse -- recent.log summary
```

## Integrating with Other Tools

### Export to Matplotlib
```bash
cargo run -p sim-logparse -- simulation.log summary --export-csv > metrics.csv
# Then use Python/matplotlib to visualize
python3 plot_metrics.py metrics.csv
```

### Export to Excel
```bash
cargo run -p sim-logparse -- simulation.log summary --export-csv > metrics.csv
# Open in Excel, Google Sheets, or similar
```

### Process with jq
```bash
cargo run -p sim-logparse -- simulation.log summary --export-json | \
  jq '.[] | select(.tps < 10)'
```

## Performance Notes

- Processing speed: 1M+ lines per second
- Memory usage: <10MB for 100MB+ files
- Regex compilation: Compiled once at startup (not per line)

## Help and Options

See all available options:
```bash
cargo run -p sim-logparse -- --help
```

See command-specific options:
```bash
cargo run -p sim-logparse -- example.log metrics --help
cargo run -p sim-logparse -- example.log anomaly --help
cargo run -p sim-logparse -- example.log summary --help
```
