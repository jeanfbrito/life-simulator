# UltraThink Phase 2 Quick Reference
## Automatic Scheduling Integration

**Status**: ✅ COMPLETE
**Date**: 2025-12-26

---

## What Changed

### ThinkQueue Scheduling Now Automatic

All trigger systems now schedule think requests to ThinkQueue:

| Trigger | Priority | Reason | Condition |
|---------|----------|--------|-----------|
| **Fear spike** | URGENT | FearTriggered | Fear > 0.3 + predators nearby |
| **Critical hunger** | URGENT | HungerCritical | Hunger >= 80% |
| **Critical thirst** | URGENT | ThirstCritical | Thirst >= 80% |
| **Critical energy** | URGENT | HungerCritical | Energy <= 20% |
| **Moderate hunger** | NORMAL | HungerModerate | Hunger 50-79% |
| **Moderate thirst** | NORMAL | ThirstModerate | Thirst 50-79% |
| **Low energy** | NORMAL | HungerModerate | Energy 20-30% |
| **Action completed** | NORMAL | ActionCompleted | Action finished |
| **Long idle** | LOW | Idle | Idle >= 50 ticks |
| **Idle fallback** | LOW | Idle | Idle >= 30 ticks |

---

## Code Example

### Before (Phase 1)
```rust
// Manual scheduling only
think_queue.schedule_urgent(entity, ThinkReason::FearTriggered, tick);
```

### After (Phase 2)
```rust
// Automatic scheduling from game events
// No manual scheduling needed!

// Fear detection automatically schedules:
if fear_state.fear_level > 0.3 && fear_state.nearby_predators > 0 {
    think_queue.schedule_urgent(entity, ThinkReason::FearTriggered, tick.0);
}

// Hunger detection automatically schedules:
if current_hunger >= 80.0 {
    think_queue.schedule_urgent(entity, ThinkReason::HungerCritical, tick.0);
} else if current_hunger >= 50.0 {
    think_queue.schedule_normal(entity, ThinkReason::HungerModerate, tick.0);
}
```

---

## Debug Logging

Enable with `RUST_LOG=debug`:

### Scheduling Events
```
🧠 ThinkQueue: Scheduling URGENT for fear: 0.52 fear, 2 predators
🧠 ThinkQueue: Scheduling URGENT for critical hunger: 85.0%
🧠 ThinkQueue: Scheduling NORMAL for moderate thirst: 65.0%
🧠 ThinkQueue: Scheduling NORMAL for action completion
🧠 ThinkQueue: Scheduling LOW for long idle: 75 ticks
```

### Queue Metrics (every 50 ticks)
```
🧠 ThinkQueue depth: 15 urgent, 45 normal, 180 low | Processed 50/50 | Total: 2500
```

---

## Performance Optimizations

### 1. Idle Check Frequency Reduced
```rust
// Before: Ran EVERY tick
// After: Runs every 20 ticks
if tick.0 % 20 != 0 {
    return; // Skip this tick
}
```

### 2. Duplicate Prevention
Entities can only be queued once (HashSet tracking):
```rust
if self.queued_entities.insert(entity) {
    // Schedule only if not already queued
}
```

---

## Expected Queue Activity

### Typical Distribution (500 entities)
- **Urgent**: 5-20 requests (1-4%)
- **Normal**: 30-80 requests (6-16%)
- **Low**: 100-300 requests (20-60%)
- **Total**: 135-400 requests queued
- **Processed**: 50 per tick

### Health Indicators
- ✅ **Good**: Queue stable or decreasing
- ✅ **Good**: Urgent queue < 50
- ⚠️ **Warning**: Urgent queue > 100
- ❌ **Bad**: Queue growing every tick

---

## Integration with Existing Systems

### Dual-Queue Mode (Phase 2-4)
Both queues run simultaneously:
- **ReplanQueue**: Original system (baseline)
- **ThinkQueue**: New system (being tested)

### Migration Path (Phase 5)
1. Verify ThinkQueue working correctly
2. Disable ReplanQueue
3. Remove ReplanQueue code
4. ThinkQueue becomes primary system

---

## Files Modified

### Core Integration
- `src/ai/trigger_emitters.rs` - All trigger systems updated

### New Tests
- `tests/trigger_thinkqueue_integration_test.rs` - Integration tests

### Documentation
- `ULTRATHINK_PHASE2_DELIVERY.md` - Full delivery report
- `ULTRATHINK_PHASE2_QUICK_REF.md` - This file

---

## Quick Verification

### 1. Check Code Compiles
```bash
cargo check
# Should show 0 errors
```

### 2. Run Simulator with Logging
```bash
RUST_LOG=debug cargo run --bin life-simulator
```

### 3. Watch for ThinkQueue Logs
```bash
# Look for these patterns:
# - "🧠 ThinkQueue: Scheduling..."
# - "🧠 ThinkQueue depth:..."
```

---

## Common Issues & Solutions

### Issue: No ThinkQueue logs appearing
**Solution**: Ensure RUST_LOG=debug and UltraThinkPlugin is registered

### Issue: Queue always empty
**Solution**: Entities need BehaviorConfig, Hunger, Thirst, FearState components

### Issue: Urgent queue backlog growing
**Solution**: Increase thinks_per_tick budget (default 50)

### Issue: Tests failing
**Solution**: Ensure all required resources initialized (ThinkQueue, ReplanQueue, TickProfiler)

---

## Next Phase

**Phase 3**: LOD System (Optional)
- Distance-based priority downgrade
- Importance scoring
- Further performance optimization

**Phase 5**: Migration (Critical)
- Remove ReplanQueue
- ThinkQueue becomes primary
- Clean up dual-queue code

---

## Summary

Phase 2 delivers **automatic ThinkQueue population** from game events:
- ✅ 5 trigger systems integrated
- ✅ 3 priority levels working
- ✅ Smart severity-based scheduling
- ✅ Performance optimizations applied
- ✅ Debug logging comprehensive

**The queue now fills itself - no manual scheduling required!**
