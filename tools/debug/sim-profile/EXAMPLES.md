# sim-profile Examples and Use Cases

## Quick Start

### Analyze Last Run

```bash
# Run the simulator with logging
RUST_LOG=info cargo run --bin life-simulator > perf.log 2>&1

# Identify bottlenecks
target/release/sim-profile top --n 10 perf.log
```

### Create Baseline for Your Branch

```bash
# After running simulator
cargo run --bin life-simulator > baseline.log 2>&1

# Export as baseline
target/release/sim-profile export baseline.log baseline.json

# Keep for regression testing
git add baseline.json
```

## Common Workflows

### Workflow 1: Performance Investigation

You notice the game is running slowly. Let's find where time is being spent:

```bash
# Capture current performance
RUST_LOG=info cargo run --bin life-simulator > current.log 2>&1

# See what's slow
sim-profile top --n 5 current.log

# Check if vegetation is the culprit
sim-profile trend --system vegetation --chart current.log

# See how it changed over time in the run
sim-profile trend --system vegetation current.log
```

Sample output:
```
=== Top 5 Performance Bottlenecks ===

System                 Avg (ms)   Min (ms)   Max (ms) Median (ms)   Stddev
──────────────────────────────────────────────────────────────────────────────────────────
vegetation                 8.50       7.20       9.80       8.60     0.65
ai_planner                 5.20       4.80       5.90       5.10     0.35
movement                   3.10       2.80       3.50       3.10     0.18
```

Action: Vegetation system is taking 8.5ms on average, over 60% of frame budget. Investigate vegetation grid updates.

### Workflow 2: Optimization Verification

You made changes to the vegetation system. Did it help or hurt?

```bash
# Create baseline from before changes
git stash
RUST_LOG=info cargo run --bin life-simulator > before.log 2>&1
sim-profile export before.log before.json

# Apply your changes
git stash pop

# Build and test
RUST_LOG=info cargo run --release --bin life-simulator > after.log 2>&1

# Compare with strict threshold (2%)
sim-profile regression before.json after.log --threshold 2
```

Expected output (if optimization worked):
```
No regressions detected (threshold: 2%)
```

Or if it got slower:
```
=== Performance Regressions (threshold: 2%) ===
System               Baseline (ms) Current (ms)   Change (%)
────────────────────────────────────────────────────────────
vegetation                   5.00         5.15         3.0%
```

### Workflow 3: Track Optimization Progress

Make multiple optimization passes and track improvement:

```bash
#!/bin/bash

# Baseline - original code
RUST_LOG=info cargo run --release --bin life-simulator > baseline.log 2>&1
sim-profile export baseline.log baseline.json

for i in {1..5}; do
    # Make optimization
    echo "Optimization pass $i..."
    RUST_LOG=info cargo run --release --bin life-simulator > run_$i.log 2>&1

    # Show improvement
    echo "Bottlenecks after pass $i:"
    sim-profile top --n 3 run_$i.log

    # Check regression (should be negative = improvement)
    sim-profile regression baseline.json run_$i.log --threshold -50
done
```

### Workflow 4: CI/CD Integration

Add to your GitHub Actions or CI pipeline:

```yaml
name: Performance Check

on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build simulator
        run: cargo build --release --bin life-simulator

      - name: Generate baseline (stable)
        run: |
          git checkout origin/main
          cargo build --release --bin life-simulator
          RUST_LOG=info timeout 30s ./target/release/life-simulator > stable.log 2>&1 || true
          cargo run --release -p sim-profile -- export stable.log baseline.json

      - name: Test current branch
        run: |
          git checkout -
          cargo build --release --bin life-simulator
          RUST_LOG=info timeout 30s ./target/release/life-simulator > current.log 2>&1 || true

      - name: Check for regressions
        run: |
          if cargo run --release -p sim-profile -- regression baseline.json current.log --threshold 10; then
            echo "Performance within acceptable range"
          else
            echo "Performance regressed significantly"
            exit 1
          fi

      - name: Upload logs
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: performance-logs
          path: |
            baseline.log
            current.log
            baseline.json
```

## Data Export and Analysis

### Export for Spreadsheet Analysis

```bash
# Export to JSON
sim-profile export performance.log data.json

# View in Python/Pandas
python3 << 'EOF'
import json
import pandas as pd

with open('data.json') as f:
    data = json.load(f)

# Flatten to dataframe
rows = []
for tick in data:
    for system, metrics in tick['systems'].items():
        rows.append({
            'tick': tick['tick'],
            'system': system,
            'ms': metrics['ms'],
            'percentage': metrics['percentage']
        })

df = pd.DataFrame(rows)

# Summary statistics
print(df.groupby('system')['ms'].describe())

# Export to CSV
df.to_csv('performance.csv', index=False)
EOF
```

### Generate Charts with Python

```python
import json
import matplotlib.pyplot as plt

with open('data.json') as f:
    data = json.load(f)

systems = {}
for tick in data:
    for system, metrics in tick['systems'].items():
        if system not in systems:
            systems[system] = {'ticks': [], 'ms': []}
        systems[system]['ticks'].append(tick['tick'])
        systems[system]['ms'].append(metrics['ms'])

# Plot trends
fig, axes = plt.subplots(len(systems), 1, figsize=(12, 4*len(systems)))
for i, (system, data) in enumerate(systems.items()):
    axes[i].plot(data['ticks'], data['ms'], marker='o')
    axes[i].set_title(f'{system} Performance Over Time')
    axes[i].set_xlabel('Tick')
    axes[i].set_ylabel('Time (ms)')
    axes[i].grid(True)

plt.tight_layout()
plt.savefig('performance_trends.png')
```

## Troubleshooting

### "No tick performance data found"

The log file doesn't contain TickProfiler output. Ensure:

```bash
# Run simulator with proper logging
RUST_LOG=info cargo run --bin life-simulator > output.log 2>&1

# Or pipe stderr to see debug logs
cargo run --bin life-simulator 2>&1 | tee output.log
```

### Parser Not Finding Systems

If `top` returns 0 systems, the regex patterns may not match your format. Check:

```bash
# View raw log lines
grep "TICK PERFORMANCE" output.log | head -5
grep "├──" output.log | head -5

# Patterns should match:
# 🔧 TICK PERFORMANCE - Tick 50 | Total: 10.5ms
# ├── system_name    : 5.2ms ( 49%)
```

### Comparing Different Log Formats

If logs are from different versions of the profiler, export both first:

```bash
sim-profile export old_format.log old_data.json
sim-profile export new_format.log new_data.json

# Compare JSONs manually or with jq
jq '.[] | select(.tick == 50)' old_data.json
```

## Advanced Usage

### Batch Analysis

Analyze multiple runs:

```bash
#!/bin/bash

for i in {1..10}; do
    echo "Run $i..."
    RUST_LOG=info cargo run --release --bin life-simulator > run_$i.log 2>&1
    sim-profile top --n 1 run_$i.log
done
```

### Generate HTML Report

Combine with templating to create HTML reports:

```bash
#!/bin/bash

cat > report.html << 'EOF'
<!DOCTYPE html>
<html>
<head><title>Performance Report</title></head>
<body>
<h1>Performance Analysis</h1>
<pre>
EOF

# Append analysis results
echo "Generated at: $(date)" >> report.html
sim-profile top --n 5 performance.log >> report.html

cat >> report.html << 'EOF'
</pre>
</body>
</html>
EOF

# Open in browser
open report.html  # macOS
# xdg-open report.html  # Linux
```

## Performance Tips

- For large logs (10,000+ ticks), use `--release` build:
  ```bash
  cargo build --release -p sim-profile
  ./target/release/sim-profile top --n 10 huge.log
  ```

- Export once, analyze many times:
  ```bash
  sim-profile export run.log data.json
  # Now query JSON with jq, Python, etc
  ```

- Create incremental baselines for long-running optimizations:
  ```bash
  sim-profile export run_v1.log v1.json
  sim-profile export run_v2.log v2.json
  # Compare v1 vs v2 with regression analysis
  ```
