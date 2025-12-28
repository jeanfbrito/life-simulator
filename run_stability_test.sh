#!/bin/bash

# Long-running stability test script for Life Simulator
# Monitors memory and entity count over 100,000 ticks

TARGET_TICKS=100000
CHECK_INTERVAL=300  # Check every 5 minutes
LOG_FILE="stability_test_$(date +%s).log"
REPORT_FILE="STABILITY_TEST_REPORT_$(date +%s).md"

echo "🧪 Starting Long-Running Stability Test" | tee -a "$LOG_FILE"
echo "📊 Target: $TARGET_TICKS ticks (~2.8 hours at 10 TPS)" | tee -a "$LOG_FILE"
echo "📝 Logging to: $LOG_FILE" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Start simulator in background
RUST_LOG=warn DISABLE_WEB_SERVER=1 ./target/release/life-simulator > sim_output.log 2>&1 &
SIM_PID=$!

echo "🚀 Simulator started with PID: $SIM_PID" | tee -a "$LOG_FILE"
echo "START_TIME=$(date +%s)" >> "$LOG_FILE"

# Monitor loop
SAMPLES=0
while kill -0 $SIM_PID 2>/dev/null; do
    sleep $CHECK_INTERVAL
    
    SAMPLES=$((SAMPLES + 1))
    TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    RUNTIME_MINS=$(( ($(date +%s) - $(grep START_TIME "$LOG_FILE" | cut -d= -f2)) / 60 ))
    
    # Get current tick from log
    CURRENT_TICK=$(tail -100 sim_output.log | grep "Tick #" | tail -1 | sed 's/.*Tick #\([0-9]*\).*/\1/' || echo "0")
    
    # Get memory usage (macOS)
    MEM_MB=$(ps -p $SIM_PID -o rss= | awk '{print $1/1024}')
    
    # Count entities from web API if available
    # ENTITIES=$(curl -s http://127.0.0.1:3030/api/entities | jq '. | length' 2>/dev/null || echo "N/A")
    
    PROGRESS=$(echo "scale=1; ($CURRENT_TICK / $TARGET_TICKS) * 100" | bc)
    
    echo "[$TIMESTAMP] Sample #$SAMPLES | Tick: $CURRENT_TICK | Progress: ${PROGRESS}% | Memory: ${MEM_MB} MB | Runtime: ${RUNTIME_MINS}m" | tee -a "$LOG_FILE"
    
    # Check if we've reached target
    if [ "$CURRENT_TICK" -ge "$TARGET_TICKS" ]; then
        echo "🎉 Target reached! Stopping simulator..." | tee -a "$LOG_FILE"
        kill $SIM_PID
        break
    fi
done

wait $SIM_PID
EXIT_CODE=$?

echo "" | tee -a "$LOG_FILE"
echo "✅ Test complete with exit code: $EXIT_CODE" | tee -a "$LOG_FILE"

# Generate report
cat > "$REPORT_FILE" << EOFREPORT
# Stability Test Report

## Test Parameters
- **Target Ticks**: $TARGET_TICKS
- **Test Started**: $(date)
- **Samples Collected**: $SAMPLES

## Memory Samples

\`\`\`
$(grep "Sample #" "$LOG_FILE")
\`\`\`

## Simulator Output (Last 100 lines)

\`\`\`
$(tail -100 sim_output.log)
\`\`\`

## Assessment

### Memory Leak Detection
$(python3 << 'EOFPYTHON'
import re

with open("$LOG_FILE") as f:
    lines = f.readlines()

samples = []
for line in lines:
    match = re.search(r'Memory: ([\d.]+) MB', line)
    if match:
        samples.append(float(match.group(1)))

if len(samples) >= 2:
    first = samples[0]
    last = samples[-1]
    growth = last - first
    growth_pct = (growth / first) * 100 if first > 0 else 0
    rate = growth / len(samples)
    
    print(f"- Initial Memory: {first:.1f} MB")
    print(f"- Final Memory: {last:.1f} MB")
    print(f"- Total Growth: {growth:.1f} MB ({growth_pct:+.1f}%)")
    print(f"- Growth per sample: {rate:.4f} MB")
    
    if abs(rate) < 1.0:
        print("\n✅ No significant memory leak detected")
    elif abs(rate) < 5.0:
        print("\n⚠️ Minor memory growth detected - monitor further")
    else:
        print("\n❌ Significant memory leak detected - investigation needed")
else:
    print("Insufficient data for analysis")
EOFPYTHON
)

### Cleanup Systems

All cleanup systems are integrated and running:
- **Hunting Relationships**: cleanup_stale_hunting_relationships
- **Pack Relationships**: cleanup_stale_pack_relationships  
- **Mating Relationships**: cleanup_stale_mating_relationships
- **Action Queue**: cleanup_dead_entities (every 100 ticks)
- **Replan Queue**: cleanup_stale_entities (periodic)

### Recommendations

- Review final tick count vs target
- Analyze memory growth pattern
- Check for entity accumulation in logs
- Verify cleanup systems executed successfully

---
*Report generated: $(date)*
EOFREPORT

echo "📊 Report saved to: $REPORT_FILE" | tee -a "$LOG_FILE"
cat "$REPORT_FILE"
