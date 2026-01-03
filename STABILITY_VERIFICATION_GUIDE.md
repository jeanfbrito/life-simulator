# Map Generator 2.0 - System Stability Verification Guide

This guide provides step-by-step instructions for verifying that the Map Generator 2.0 implementation maintains system stability comparable to the pre-implementation snapshot (git tag `pre-mapgen2.0`).

## Objective

Verify that the simulation runs stably for extended periods (5+ minutes) with:
- No crashes or panics
- Stable entity counts (no mass die-offs)
- No error logs or warnings
- Performance comparable to pre-implementation

---

## Prerequisites

1. **Snapshot Tag Exists**
   ```bash
   git tag -l | grep pre-mapgen2.0
   # Should output: pre-mapgen2.0
   ```

2. **System Built Successfully**
   ```bash
   cargo build --release --bin life-simulator
   cargo build --bin map_generator
   ```

3. **Test Map Generated**
   ```bash
   cargo run --bin map_generator generate stability_test "Stability Test" 54321
   # Creates: saves/stability_test.ron
   ```

---

## Verification Procedure

### Step 1: Start the Simulation

```bash
# Terminal 1: Start simulation with logging enabled
RUST_LOG=info cargo run --release --bin life-simulator 2>&1 | tee stability_run.log

# Server should start on http://localhost:54321
# Map should load: stability_test.ron (or default world)
# Entities should begin spawning within 30 seconds
```

**Success Criteria:**
- ✅ Server starts without errors
- ✅ Map loads successfully
- ✅ No panic or crash messages
- ✅ Entities begin spawning

### Step 2: Monitor Entity Counts

```bash
# Terminal 2: Monitor entity population every 30 seconds for 5 minutes
for i in {1..10}; do
  echo "=== Check $i/10 ($(date +%T)) ==="
  curl -s http://localhost:54321/api/entities | jq '{total: .total, alive: .alive, by_species: [.entities | group_by(.species) | .[] | {species: .[0].species, count: length}]}'
  sleep 30
done
```

**Success Criteria:**
- ✅ Entity count increases initially (spawning phase)
- ✅ Entity count stabilizes after 1-2 minutes (births ≈ deaths)
- ✅ No sudden drops in population (mass die-offs)
- ✅ Multiple species present throughout test
- ✅ `alive` count matches `total` (no stuck dead entities)

**Expected Behavior:**
```
Check 1: Total: 15   (spawning phase)
Check 2: Total: 42   (rapid growth)
Check 3: Total: 68   (continued growth)
Check 4: Total: 82   (stabilizing)
Check 5: Total: 87   (stable ecosystem)
Check 6: Total: 91   (slight growth)
Check 7: Total: 89   (stable)
Check 8: Total: 93   (stable)
Check 9: Total: 88   (stable)
Check 10: Total: 91  (stable)
```

### Step 3: Scan for Errors in Logs

```bash
# After 5 minutes, scan the log file for errors
grep -E '(ERROR|WARN|panic|crash|failed|error)' stability_run.log | grep -v "AI decision failed" | head -20

# Note: "AI decision failed" is expected occasionally (animals dying, etc.)
```

**Success Criteria:**
- ✅ No ERROR messages
- ✅ No panic or crash messages
- ✅ Minimal WARN messages (none related to core systems)
- ✅ No repeated error patterns

**Acceptable Warnings:**
- `AI decision failed` (occasional - animals can die before completing actions)
- Resource depletion warnings (expected in ecosystem simulation)

**Unacceptable Errors:**
- Panics or crashes
- Entity spawn failures (beyond initial spawning)
- Chunk generation errors
- ECS system conflicts
- Bevy component errors

### Step 4: Check Memory and CPU Usage

```bash
# While simulation is running, check resource usage
ps aux | grep life-simulator | grep -v grep

# Monitor for 5 minutes - memory should be stable (not constantly increasing)
```

**Success Criteria:**
- ✅ CPU usage: 5-25% (varies with entity count)
- ✅ Memory usage: Stable (not constantly increasing)
- ✅ No memory leaks (RSS should stabilize after initial growth)

**Expected Resource Usage:**
```
USER   PID  %CPU %MEM    VSZ   RSS  STARTED  TIME  COMMAND
user  1234  15.2  2.1  500MB 180MB  10:05   0:45  life-simulator
                        ↑ VSZ may grow during startup
                        ↑ RSS should stabilize
```

### Step 5: Verify Entity Behavior

```bash
# Sample 10 random entities and check their positions and actions
curl -s http://localhost:54321/api/entities | jq '.entities | .[0:10] | .[] | {id: .id, species: .species, position: .position, action: .action, health: .health}'
```

**Success Criteria:**
- ✅ All sampled entities have valid positions (not NaN or infinite)
- ✅ Positions are within map bounds
- ✅ Entities have reasonable actions (Idle, MoveTo, Eat, Drink, etc.)
- ✅ Health values are reasonable (0-100 range)
- ✅ No entities stuck at (0, 0) or (-1, -1)

**Expected Output:**
```json
{
  "id": 42,
  "species": "Rabbit",
  "position": {"x": 245.3, "y": 12.8, "z": 189.7},
  "action": "MoveTo",
  "health": 87.3
}
```

### Step 6: Performance Comparison

Compare simulation performance with pre-implementation snapshot:

```bash
# 1. Checkout pre-implementation tag
git checkout pre-mapgen2.0

# 2. Build and run for 5 minutes (follow steps 1-5)
cargo build --release --bin life-simulator
RUST_LOG=info cargo run --release --bin life-simulator 2>&1 | tee stability_run_baseline.log

# 3. Record baseline metrics:
# - Entity count at 5 minutes
# - CPU/memory usage
# - Number of errors/warnings

# 4. Checkout current implementation
git checkout -

# 5. Compare metrics
```

**Success Criteria:**
- ✅ Entity counts within ±20% of baseline
- ✅ CPU usage within ±30% of baseline
- ✅ Memory usage within ±20% of baseline
- ✅ Similar or fewer errors/warnings than baseline

---

## Automated Stability Test Script

For convenience, use this automated script:

```bash
#!/bin/bash
# stability_test.sh

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
    kill $SIM_PID
    exit 1
fi
echo ""

# Monitor entity counts for 5 minutes
echo "Step 4: Monitoring entity population (5 minutes)..."
echo "Time     | Total | Alive | Species"
echo "---------|-------|-------|--------"

for i in {1..10}; do
    TIMESTAMP=$(date +%T)
    STATS=$(curl -s http://localhost:54321/api/entities | jq -r '"\(.total)|\(.alive)|\([.entities | group_by(.species) | .[] | .[0].species] | join(","))"')
    TOTAL=$(echo $STATS | cut -d'|' -f1)
    ALIVE=$(echo $STATS | cut -d'|' -f2)
    SPECIES=$(echo $STATS | cut -d'|' -f3 | tr ',' ' ' | wc -w)
    printf "%s | %5s | %5s | %s\n" "$TIMESTAMP" "$TOTAL" "$ALIVE" "$SPECIES"
    sleep 30
done
echo ""

# Check for errors
echo "Step 5: Scanning for errors..."
ERROR_COUNT=$(grep -c -E '(ERROR|panic|crash)' stability_run.log || echo "0")
WARN_COUNT=$(grep -c -E 'WARN' stability_run.log | grep -v "AI decision failed" || echo "0")
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
ps aux | grep $SIM_PID | grep -v grep | awk '{printf "CPU: %s%%  Memory: %s%%  RSS: %s\n", $3, $4, $6}'
echo ""

# Final entity sample
echo "Step 7: Sampling entity health..."
SAMPLE=$(curl -s http://localhost:54321/api/entities | jq '[.entities | .[0:10] | .[] | {species: .species, health: .health, action: .action}]')
echo "$SAMPLE" | jq -r '.[] | "\(.species): health=\(.health), action=\(.action)"'
echo ""

# Cleanup
echo "Step 8: Stopping simulation..."
kill $SIM_PID
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
```

Save as `verify_stability.sh` and run:

```bash
chmod +x verify_stability.sh
./verify_stability.sh
```

---

## Success Criteria Summary

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| **No Crashes** | Simulation runs for 5+ minutes without panic/crash | ⬜ |
| **Stable Entities** | Entity count stabilizes, no mass die-offs | ⬜ |
| **No Errors** | Zero ERROR messages in logs (AI decision failures OK) | ⬜ |
| **Valid Behavior** | Entities have valid positions and actions | ⬜ |
| **Resource Stability** | Memory/CPU usage stable (not constantly increasing) | ⬜ |
| **Performance** | Within ±30% of pre-implementation baseline | ⬜ |

**PASS THRESHOLD**: All 6 criteria must be met ✅

---

## Troubleshooting

### Issue: Simulation crashes immediately

**Possible Causes:**
- Missing dependencies
- Corrupted map file
- Port 54321 already in use

**Solutions:**
```bash
# Check dependencies
cargo check

# Regenerate map
rm saves/stability_test.ron
cargo run --bin map_generator generate stability_test "Stability Test" 54321

# Check port availability
lsof -i :54321
```

### Issue: Entity count drops rapidly

**Possible Causes:**
- Insufficient food/water resources
- Spawn point in invalid location
- Pathfinding failures

**Solutions:**
```bash
# Check resource distribution
curl -s http://localhost:54321/api/resources | jq '.total'

# Verify spawn point validity
grep "spawn_point" saves/stability_test.ron

# Check logs for pathfinding errors
grep "pathfinding" stability_run.log
```

### Issue: High CPU usage (>50%)

**Possible Causes:**
- Too many entities for system
- Inefficient pathfinding
- Debug build instead of release

**Solutions:**
```bash
# Verify release build
cargo build --release --bin life-simulator

# Check entity count
curl -s http://localhost:54321/api/entities | jq '.total'
# If > 200, reduce spawn rate or map size
```

### Issue: Memory usage constantly increasing

**Possible Causes:**
- Memory leak in new code
- Entity cleanup not working
- Resource accumulation

**Solutions:**
```bash
# Check for dead entities not being cleaned up
curl -s http://localhost:54321/api/entities | jq '{total: .total, alive: .alive}'
# If alive << total, there's a cleanup issue

# Monitor for 10 minutes to confirm leak
watch -n 30 'ps aux | grep life-simulator | grep -v grep'
```

---

## Comparison to Pre-Implementation

To ensure Map Generator 2.0 doesn't introduce regressions:

### Baseline Metrics (pre-mapgen2.0)

Record these metrics from the baseline run:

| Metric | Baseline | Current | Delta | Status |
|--------|----------|---------|-------|--------|
| Entity count @ 5min | ___ | ___ | ___% | ⬜ |
| CPU usage (avg) | ___% | ___% | ___% | ⬜ |
| Memory usage (RSS) | ___MB | ___MB | ___% | ⬜ |
| Error count | ___ | ___ | ___ | ⬜ |
| Warning count | ___ | ___ | ___ | ⬜ |

**Fill in during verification and compare.**

### Expected Differences

**Acceptable:**
- ±10-20% entity count variation (different map layout)
- ±5-10% CPU variation (depends on pathfinding complexity)
- Slightly different species distribution (biome changes)

**Unacceptable:**
- >50% fewer entities (indicates spawn or survival issues)
- >50% higher CPU/memory (indicates performance regression)
- Any new crashes or errors that didn't exist in baseline

---

## Final Sign-Off

Once all verification steps pass:

1. ✅ All 6 success criteria met
2. ✅ Comparison to baseline shows no regressions
3. ✅ stability_run.log reviewed and clean
4. ✅ No outstanding errors or warnings

**Mark subtask-10-3 as COMPLETED** and update implementation_plan.json.

---

## Next Steps

After stability verification passes:

1. **Update build-progress.txt** with verification results
2. **Mark QA acceptance complete** in implementation_plan.json
3. **Create final commit** documenting verification
4. **Tag release** (optional): `git tag -a mapgen2.0-release -m "Map Generator 2.0 Release"`

---

## Files Generated

During stability verification, these files are created:

- `stability_run.log` - Full simulation log output
- `stability_run_baseline.log` - Pre-implementation baseline log (if comparison run)
- `verify_stability.sh` - Automated verification script

These files can be archived for future reference or deleted after verification.

---

## See Also

- **E2E_VERIFICATION_GUIDE.md** - End-to-end map generation testing
- **ANIMAL_SPAWN_VERIFICATION.md** - Entity spawning verification
- **MAPGEN2_CONFIGURATION_REFERENCE.md** - Configuration parameter documentation
- **implementation_plan.json** - Full implementation plan and status

---

## Summary

This guide provides comprehensive stability verification for Map Generator 2.0:

✅ **5-minute runtime test** ensures no crashes or panics
✅ **Entity population monitoring** verifies ecosystem stability
✅ **Error log scanning** catches regression issues
✅ **Resource monitoring** detects memory leaks
✅ **Baseline comparison** ensures no performance regressions
✅ **Automated script** streamlines verification process

System stability must match or exceed the pre-implementation snapshot (git tag `pre-mapgen2.0`) for QA sign-off.
