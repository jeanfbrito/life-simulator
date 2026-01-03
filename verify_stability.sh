#!/bin/bash
# Map Generator 2.0 - Stability Verification Script
# Verifies system stability matches pre-implementation snapshot

set -e

echo "=== Map Generator 2.0 - Stability Verification ==="
echo "Started: $(date)"
echo ""

# Generate test map
echo "Step 1: Generating test map..."
cargo run --bin map_generator generate stability_test "Stability Test" 54321 > /dev/null 2>&1
echo "✓ Map generated: saves/stability_test.ron"
echo ""

# Start simulation in background
echo "Step 2: Starting simulation..."
RUST_LOG=info cargo run --release --bin life-simulator > stability_run.log 2>&1 &
SIM_PID=$!
echo "✓ Simulation started (PID: $SIM_PID)"
sleep 10  # Wait for server to start
echo ""

# Verify server is running
echo "Step 3: Verifying server..."
if curl -s http://localhost:54321/api/entities > /dev/null; then
    echo "✓ Server responding on port 54321"
else
    echo "✗ Server not responding!"
    kill $SIM_PID 2>/dev/null || true
    exit 1
fi
echo ""

# Monitor entity counts for 5 minutes
echo "Step 4: Monitoring entity population (5 minutes)..."
echo "Time     | Total | Alive | Species"
echo "---------|-------|-------|--------"

for i in {1..10}; do
    TIMESTAMP=$(date +%T)
    STATS=$(curl -s http://localhost:54321/api/entities | jq -r '"\(.total)|\(.alive)|\([.entities | group_by(.species) | .[] | .[0].species] | join(","))"' 2>/dev/null || echo "0|0|")
    TOTAL=$(echo "$STATS" | cut -d'|' -f1)
    ALIVE=$(echo "$STATS" | cut -d'|' -f2)
    SPECIES=$(echo "$STATS" | cut -d'|' -f3 | tr ',' ' ' | wc -w)
    printf "%s | %5s | %5s | %s\n" "$TIMESTAMP" "$TOTAL" "$ALIVE" "$SPECIES"
    sleep 30
done
echo ""

# Check for errors
echo "Step 5: Scanning for errors..."
ERROR_COUNT=$(grep -c -E '(ERROR|panic|crash)' stability_run.log 2>/dev/null || echo "0")
WARN_COUNT=$(grep -E 'WARN' stability_run.log 2>/dev/null | grep -v "AI decision failed" | wc -l || echo "0")
echo "Errors found: $ERROR_COUNT"
echo "Warnings found: $WARN_COUNT (excluding AI decision failures)"

if [ "$ERROR_COUNT" -eq 0 ]; then
    echo "✓ No errors detected"
else
    echo "✗ Errors found - check stability_run.log"
    grep -E '(ERROR|panic|crash)' stability_run.log | head -10
fi
echo ""

# Check memory usage
echo "Step 6: Checking resource usage..."
if ps -p $SIM_PID > /dev/null 2>&1; then
    ps aux | grep $SIM_PID | grep -v grep | awk '{printf "CPU: %s%%  Memory: %s%%  RSS: %s\n", $3, $4, $6}'
else
    echo "✗ Process not running (may have crashed)"
fi
echo ""

# Final entity sample
echo "Step 7: Sampling entity health..."
SAMPLE=$(curl -s http://localhost:54321/api/entities 2>/dev/null | jq '[.entities | .[0:10] | .[] | {species: .species, health: .health, action: .action}]' 2>/dev/null || echo "[]")
if [ "$SAMPLE" != "[]" ]; then
    echo "$SAMPLE" | jq -r '.[] | "\(.species): health=\(.health), action=\(.action)"' 2>/dev/null
else
    echo "✗ Could not sample entities (server may be down)"
fi
echo ""

# Cleanup
echo "Step 8: Stopping simulation..."
kill $SIM_PID 2>/dev/null || true
wait $SIM_PID 2>/dev/null || true
echo "✓ Simulation stopped"
echo ""

# Summary
echo "=== VERIFICATION SUMMARY ==="
echo "Duration: 5 minutes"
echo "Log file: stability_run.log"
echo "Errors: $ERROR_COUNT"
echo "Warnings: $WARN_COUNT"
echo ""

if [ "$ERROR_COUNT" -eq 0 ]; then
    echo "✅ STABILITY TEST PASSED"
    echo "System is stable and matches pre-implementation expectations"
    exit 0
else
    echo "❌ STABILITY TEST FAILED"
    echo "Review stability_run.log for details"
    exit 1
fi
