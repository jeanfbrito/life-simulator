# Testing the Tick System

## Overview

This document explains how to test and verify the tick-based simulation system.

## Manual Testing (Recommended)

The most reliable way to test the tick system is by running the simulation and observing entity movement over time.

### Quick Test

```bash
# Start the simulation
cargo run --bin life-simulator

# In another terminal, monitor entity positions
watch -n 2 'curl -s http://127.0.0.1:54321/api/entities | jq ".entities[] | select(.name | startswith(\"Human\")) | {name, position}"'
```

### Detailed Position Tracking Test

```bash
#!/bin/bash
# Save as: scripts/test_movement.sh

# Kill any existing simulation
pkill -f "target/debug/life-simulator"

# Start fresh
cargo build --quiet
cargo run --bin life-simulator > /tmp/life-simulator.log 2>&1 &

# Wait for startup
sleep 3

# Track first human for 30 seconds
echo "=== Movement Test (30 seconds) ==="
echo "Time | Human_0 Position"
echo "-----|------------------"

for i in {1..15}; do
    timestamp=$(date +"%M:%S")
    position=$(curl -s http://127.0.0.1:54321/api/entities | \
               jq -r '.entities[] | select(.name == "Human_0") | "\(.position.x),\(.position.y)"')
    printf "[%s] %2ds: %s\n" "$timestamp" "$((i*2))" "$position"
    sleep 2
done

echo ""
echo "✅ Test complete - check if positions changed over time"
```

## Expected Behavior

With the current configuration:
- **Base TPS**: 10 ticks per second
- **Human Movement Speed**: 30 ticks per tile  
- **Expected Movement Time**: 3 seconds per tile

### What You Should Observe

1. ✅ **Ticks incrementing**: Check logs for tick counter
2. ✅ **Entities spawn**: Humans appear at startup
3. ✅ **Periodic movement**: Entities move every ~3 seconds
4. ✅ **Position changes**: X/Y coordinates update in API responses
5. ✅ **AI decisions**: Entities change direction occasionally

### Sample Output

```
Time | Position
-----|----------
00:01| (7,20)    ← Starting position
00:03| (7,20)    ← Accumulating movement ticks
00:05| (6,20)    ← Moved 1 tile!
00:07| (5,20)    ← Moved again
00:09| (5,20)    
00:11| (4,20)    ← Moved
00:13| (5,20)    ← Changed direction (AI decision)
```

## Automated Testing

### Integration Tests (Work in Progress)

Due to API visibility issues, comprehensive integration tests are currently being developed. The test files are available but require exposing internal APIs:

- `tests/test_utils.rs` - Test helper functions
- `tests/tick_system_tests.rs` - Tick system integration tests

These tests will be enabled once the following items are made public:
- `accumulate_ticks` function
- `EntityName` or equivalent component
- `tick_movement_system` function export

### Unit Tests

Individual tick components can be tested in isolation:

```bash
# Test tick accumulator logic
cargo test --lib tick

# Test movement components  
cargo test --lib movement

# Test wandering AI
cargo test --lib wandering
```

## Performance Testing

### Tick Rate Verification

```bash
# Watch simulation logs for tick rate info
tail -f /tmp/life-simulator.log | grep "Tick #"
```

Look for output like:
```
🎯 Tick #100 | TPS: 10.0 | Avg duration: 2.5ms
🎯 Tick #200 | TPS: 10.1 | Avg duration: 2.3ms
```

### Multi-Entity Stress Test

Modify `src/main.rs` to spawn more entities:

```rust
// In setup_world system, increase spawn count
entity_types::spawn_humans(&mut commands, &world_loader, 100); // Was 10
entity_types::spawn_rabbits(&mut commands, &world_loader, 200); // Was 20
```

Then run and monitor performance:
```bash
cargo run --release --bin life-simulator
```

Check that:
- TPS stays near 10.0 (not dropping significantly)
- Average tick duration stays under 10ms
- Memory usage remains stable

##Verifying Fixes

The fixes documented in `TICK_SYSTEM_FIXES.md` can be verified by:

### 1. Tick Accumulation Works

```bash
# Check that ticks increment at correct rate
curl -s http://127.0.0.1:54321/api/simulation | jq '.current_tick'
sleep 5
curl -s http://127.0.0.1:54321/api/simulation | jq '.current_tick'
# Difference should be ~50 (5 seconds * 10 TPS)
```

### 2. should_tick Flag Functions

```bash
# Monitor logs for tick systems running
tail -f /tmp/life-simulator.log | grep -E "(wanderer|movement|stats)"
```

You should see evidence of these systems executing periodically.

### 3. Movement Speed Correct

```bash
# Time how long it takes for movement
# Expected: ~3 seconds per tile at 30 ticks per tile
scripts/test_movement.sh
```

### 4. Speed Multiplier Works

While simulation is running, press keyboard keys:
- `1` = 0.5x speed (slower)
- `2` = 1.0x speed (normal)
- `3` = 2.0x speed (faster)
- `4` = 3.0x speed (ultra fast)

Observe entity movement speed changes in the monitoring script.

### 5. Frame Rate Independence

The tick system should maintain consistent TPS regardless of frame rate. This is verified by:
- Running in headless mode (no rendering overhead)
- Checking TPS metrics in logs remain stable
- Verifying movement timing is consistent

## Troubleshooting

### Entities Not Moving

1. Check if simulation is paused (press Space to unpause)
2. Verify ticks are incrementing in logs
3. Check that wandering AI is spawning paths
4. Ensure movement speed isn't too slow

### Inconsistent Tick Rate

1. Check CPU usage - might be system overload
2. Verify no infinite loops in tick systems
3. Check tick duration metrics for spikes
4. Reduce entity count if needed

### API Not Responding

1. Ensure web server started successfully
2. Check port 54321 isn't already in use
3. Verify firewall isn't blocking local connections
4. Try `curl -v` for verbose debugging

## CI/CD Integration (Future)

When integration tests are enabled, add to CI pipeline:

```yaml
# .github/workflows/test.yml
- name: Run tick system tests
  run: cargo test --test tick_system_tests --release

- name: Run movement verification test
  run: ./scripts/test_movement.sh
  timeout-minutes: 2
```

## References

- **Architecture**: `docs/TICK_SYSTEM_FIXES.md`
- **Implementation**: `src/simulation/tick.rs`
- **Movement System**: `src/entities/movement.rs`
- **Wandering AI**: `src/entities/wandering.rs`

## Questions?

If the tick system isn't behaving as expected:
1. Review `docs/TICK_SYSTEM_FIXES.md` for known issues
2. Check logs in `/tmp/life-simulator.log`
3. Run manual position tracking test
4. Verify TPS metrics in simulation logs
