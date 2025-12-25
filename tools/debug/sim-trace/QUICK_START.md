# sim-trace Quick Start Guide

## Installation

```bash
# Build the tool
cargo build -p sim-trace --release

# Binary location
./target/release/sim-trace
```

## Basic Usage

```bash
# Show help
sim-trace --help
sim-trace <command> --help
```

## Common Commands

### Find stuck entities (no movement for 50+ ticks)
```bash
sim-trace simulation.log stuck
```

### Find very stuck entities (300+ ticks)
```bash
sim-trace simulation.log stuck --threshold 300
```

### Check a specific entity
```bash
sim-trace simulation.log entity --entity-id 42
```

### See last 20 ticks of entity
```bash
sim-trace simulation.log entity --entity-id 42 --history 20
```

### Find action loops (same action 30+ times)
```bash
sim-trace simulation.log action-loops --threshold 30
```

### Find only "Idle" loops
```bash
sim-trace simulation.log action-loops --action Idle
```

### Export entity timeline
```bash
sim-trace simulation.log timeline --entity-id 42 --export entity_42.json
```

### Get overview statistics
```bash
sim-trace simulation.log summary
```

### Export statistics as CSV
```bash
sim-trace simulation.log summary --csv > entities.csv
```

## Output Formats

All commands support `--json` for machine-readable output:

```bash
sim-trace log.txt stuck --json
sim-trace log.txt entity --entity-id 42 --json
sim-trace log.txt action-loops --json
sim-trace log.txt summary --json
```

## Most Useful Combinations

### Debug stuck entity behavior
```bash
# Find stuck entities
sim-trace log.txt stuck --threshold 50

# Check what the entity was doing
sim-trace log.txt entity --entity-id 42 --history 50
```

### Find AI bugs
```bash
# Find repeated actions (usually a sign of bugs)
sim-trace log.txt action-loops --threshold 20

# Check detailed history of problematic entity
sim-trace log.txt entity --entity-id 5 --history 100
```

### Generate reports
```bash
# CSV for spreadsheet analysis
sim-trace log.txt summary --csv > report.csv

# JSON for programmatic analysis
sim-trace log.txt summary --json > report.json

# Export specific entity timeline
sim-trace log.txt timeline --entity-id 42 --export analysis.json
```

## Expected Output Examples

### Stuck Entities
```
Entity ID | Stuck Ticks
-----------|-----------
42        | 150
57        | 200
```

### Entity History
```
Entity #42
Species: Deer
Lifespan: 500 ticks (spawned at tick 0)
Total snapshots: 125

Tick    | Position              | Action
--------|----------------------|--------------------
400     | (  150.0,   200.0) | Graze
410     | (  150.0,   200.0) | Idle
420     | (  150.0,   200.0) | Idle
```

### Action Loops
```
Entity | Action   | Repetitions | Duration (ticks)
-------|----------|-------------|------------------
42     | Idle     | 25          | 100
57     | Wander   | 20          | 80
```

### Summary
```
Entity Summary: 1523 total entities

Entity | Species    | Lifespan | Snapshots | Current Pos
-------|------------|----------|-----------|---------------------
1      | Rabbit     | 500      | 125       | (  450.0,   350.0)
2      | Deer       | 480      | 120       | (  500.0,   400.0)
3      | Wolf       | 420      | 105       | (  550.0,   400.0)
```

## Tips & Tricks

1. **Large files?** Run with `--tail N` to see only last results:
   ```bash
   sim-trace huge.log stuck --threshold 50 --tail 10
   ```

2. **Want just the numbers?** Use JSON output and pipe to jq:
   ```bash
   sim-trace log.txt stuck --json | jq '.[] | .entity_id'
   ```

3. **Generate reports?** Use CSV format:
   ```bash
   sim-trace log.txt summary --csv | sort -t',' -k3 -nr > lifespan_ranking.csv
   ```

4. **Track specific entity?** Export timeline and analyze:
   ```bash
   sim-trace log.txt timeline --entity-id 42 --export entity.json
   # Then parse entity.json in your analysis tool
   ```

5. **Find multiple types of issues?**
   ```bash
   sim-trace log.txt stuck --threshold 50 > stuck.txt
   sim-trace log.txt action-loops --threshold 20 > loops.txt
   sim-trace log.txt summary --csv > stats.csv
   ```

## Troubleshooting

**"Entity not found"**
- Entity ID doesn't exist in the log
- Check valid IDs with: `sim-trace log.txt summary`

**"No results"**
- Threshold might be too high
- Try lower values: `--threshold 10` instead of `--threshold 50`

**Large output?**
- Use `--tail N` to limit results
- Export to file: `sim-trace log.txt summary --csv > output.csv`

**Need specific data?**
- Use `--json` and pipe to `jq` for filtering
- Use `--csv` and import to spreadsheet

## Performance Notes

- Small logs (1K lines): <100ms
- Medium logs (100K lines): 1-2 seconds
- Large logs (1M+ lines): <30 seconds

All commands use streaming parsing, so memory usage is minimal regardless of log size.
