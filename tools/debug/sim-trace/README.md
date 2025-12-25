# sim-trace - Entity Behavior Analyzer CLI

A command-line tool for analyzing entity behavior in Life Simulator logs. Extracts spawn events, movement history, action sequences, and detects anomalies like stuck entities and action loops.

## Features

- **Entity Spawn Parsing**: Extract entity creation events from logs
- **Movement History**: Track position changes and detect stuck entities
- **Action Sequence Analysis**: Identify repeated action patterns and loops
- **Entity Timelines**: Export complete entity histories to JSON
- **Summary Statistics**: Generate overview of all entities in simulation
- **Multiple Output Formats**: Text (human-readable), JSON, and CSV outputs
- **Memory Efficient**: Stream-based parsing for large log files

## Building

```bash
cargo build -p sim-trace
```

## Commands

### 1. Find Stuck Entities

Find entities that haven't moved for N+ consecutive ticks:

```bash
sim-trace --stuck 50 <logfile>
sim-trace <logfile> stuck --threshold 50
sim-trace <logfile> stuck --threshold 50 --json
sim-trace <logfile> stuck --threshold 50 --tail 10
```

**Options:**
- `--threshold <N>`: Minimum consecutive ticks without movement (default: 50)
- `--tail <N>`: Show only last N results
- `--json`: Output as JSON

**Example Output:**
```
Entity ID | Stuck Ticks
-----------|-----------
1         | 50
5         | 75
```

### 2. Analyze Specific Entity

Show movement and action history for a specific entity:

```bash
sim-trace <logfile> entity --entity-id 42
sim-trace <logfile> entity --entity-id 42 --history 100
sim-trace <logfile> entity --entity-id 42 --json
sim-trace <logfile> entity -e 42
```

**Options:**
- `--entity-id, -e <ID>`: Entity ID to analyze (required)
- `--history <N>`: Show only last N ticks
- `--show-actions`: Include action names (default: true)
- `--json`: Output as JSON

**Example Output:**
```
Entity #42
Species: Deer
Lifespan: 150 ticks (spawned at tick 0)
Total snapshots: 45

Tick    | Position              | Action
--------|----------------------|--------------------
100     | (  150.0,   200.0) | Graze
110     | (  155.0,   205.0) | Walk
120     | (  160.0,   210.0) | Drink
```

### 3. Detect Action Loops

Identify actions repeated excessively (potential AI bugs):

```bash
sim-trace <logfile> action-loops --threshold 20
sim-trace <logfile> action-loops --threshold 20 --json
sim-trace <logfile> action-loops --action Idle
sim-trace <logfile> action-loops --threshold 20 --tail 5
```

**Options:**
- `--threshold <N>`: Minimum repetitions to flag as loop (default: 20)
- `--action <NAME>`: Filter by specific action type
- `--tail <N>`: Show only last N results
- `--json`: Output as JSON

**Example Output:**
```
Entity | Action   | Repetitions | Duration (ticks)
-------|----------|-------------|------------------
1      | Graze    | 50          | 100
5      | Idle     | 30          | 60
```

### 4. Generate Entity Timeline

Export complete entity history to JSON file:

```bash
sim-trace <logfile> timeline --entity-id 5 --export timeline.json
sim-trace <logfile> timeline -e 5 -o timeline.json
sim-trace <logfile> timeline --entity-id 5 --json
```

**Options:**
- `--entity-id, -e <ID>`: Entity ID to export (required)
- `--export, -o <PATH>`: Output file path (JSON format)
- `--positions`: Include position history (default: true)
- `--actions`: Include action history (default: true)
- `--json`: Display as JSON instead of exporting

**Output Format:**
```json
{
  "entity_id": 5,
  "species": "Deer",
  "spawn_tick": 0,
  "death_tick": null,
  "lifespan_ticks": 150,
  "total_snapshots": 45,
  "snapshots": [
    {
      "tick": 0,
      "position": {"x": 100.0, "y": 200.0},
      "action": "Spawn"
    },
    {
      "tick": 10,
      "position": {"x": 105.0, "y": 205.0},
      "action": "Graze"
    }
  ]
}
```

### 5. Summary Statistics

Generate overview statistics for all entities:

```bash
sim-trace <logfile> summary
sim-trace <logfile> summary --json
sim-trace <logfile> summary --csv
sim-trace <logfile> summary --top-entities 10
```

**Options:**
- `--json`: Output as JSON
- `--csv`: Output as CSV
- `--top-entities <N>`: Show top N entities by lifespan

**Example Output (Text):**
```
Entity Summary: 1523 total entities

Entity | Species    | Lifespan | Snapshots | Current Pos
-------|------------|----------|-----------|---------------------
1      | Rabbit     | 500      | 125       | (  450.0,   350.0)
2      | Deer       | 480      | 120       | (  500.0,   400.0)
```

**Example Output (CSV):**
```csv
entity_id,species,lifespan_ticks,snapshots,spawn_tick,x,y
1,Rabbit,500,125,0,450.0,350.0
2,Deer,480,120,0,500.0,400.0
```

## Log Format Requirements

The tool recognizes these log patterns:

### Entity Spawn
```
✅ Spawned rabbit #42: Name 🐇 at IVec2(100, 200)
Spawned Deer #5: Bambi at IVec2(150, 250)
```

### Tick Information
```
TICK=12345: Processing
[TICK 999] update
tick: 500
```

### Position Updates
```
Entity 42 at position (100.5, 200.5)
Entity 1 at (123, 456)
```

### Action Updates
```
Entity 42 status: {"current_action": "Graze"}
"current_action": "Drink"
action: Walk
```

## Data Structures

### EntitySnapshot
Represents a single state observation of an entity:
- `tick`: Simulation tick number
- `entity_id`: Unique entity identifier
- `x, y`: Position coordinates
- `action`: Current action name (optional)
- `species`: Entity species (optional)

### EntityHistory
Complete temporal record for one entity:
- `entity_id`: Unique identifier
- `snapshots`: Ordered list of EntitySnapshot
- `spawn_tick`: When entity was created
- `death_tick`: When entity died (if applicable)
- `lifespan_ticks`: Total lifetime in ticks

## Use Cases

### Debugging AI Issues
Find entities stuck in loops or unresponsive:
```bash
sim-trace simulation.log stuck --threshold 50
sim-trace simulation.log action-loops --threshold 30
```

### Performance Analysis
Identify entities causing performance issues:
```bash
sim-trace simulation.log entity --entity-id 42 --history 100
```

### Data Export for Analysis
Export complete timelines for external analysis:
```bash
sim-trace simulation.log timeline --entity-id 5 --export entity_5.json
```

### Ecosystem Monitoring
Check overall health and statistics:
```bash
sim-trace simulation.log summary --csv > stats.csv
```

## Implementation Notes

- **Stream-based Parsing**: Handles large log files without loading entire content
- **Lazy Evaluation**: Only analyzes data relevant to requested command
- **Zero-copy Patterns**: Minimizes memory allocations during parsing
- **Regex Flexibility**: Recognizes various log formats automatically

## Testing

Run comprehensive test suite:

```bash
cargo test -p sim-trace
```

Tests cover:
- Entity spawn parsing
- Position delta calculation
- Stuck entity detection
- Action loop detection
- Timeline export format
- CLI argument parsing
- Output formatting (text, JSON, CSV)
- Stream-based parsing

All tests use TDD approach with 100% pass rate.

## Files

- `src/main.rs`: CLI entry point and command handlers
- `src/cli.rs`: Argument parsing and command structures (60+ tests)
- `src/entities.rs`: Core data structures and analysis logic (20+ tests)
- `src/parser.rs`: Log file parsing and extraction (10+ tests)
- `src/output.rs`: Formatting and export functions (10+ tests)
- `src/lib.rs`: Public API for module re-export

## Performance

Typical performance on simulations:
- Small log (1K lines): <100ms
- Medium log (100K lines): ~1-2s
- Large log (1M+ lines): <30s

Memory usage remains constant regardless of log size due to stream processing.
