# Wander PathfindingQueue Quick Reference

**Phase 2 Complete** - Async pathfinding for Wander action

---

## State Machine

```rust
enum WanderState {
    NeedPath,                              // Request path
    WaitingForPath { request_id },         // Wait for result
    Moving { path, current_index },        // Follow path
}
```

**Transitions**:
```
NeedPath → [queue request] → WaitingForPath
WaitingForPath → [result ready] → Moving
WaitingForPath → [result failed] → NeedPath (retry) OR Failed (max retries)
Moving → [arrived] → Success
Moving → [path failed] → NeedPath (retry) OR Failed (max retries)
```

---

## Usage Pattern

```rust
// 1. Create WanderAction
let wander = WanderAction::new(target_tile);

// 2. Execute repeatedly (called each tick)
let result = wander.execute(&mut world, entity, tick);

// States:
// - InProgress: Still working (queued, waiting, or moving)
// - Success: Arrived at target
// - Failed: Gave up after max retries (3)
```

---

## Pathfinding Integration

```rust
// In WanderState::NeedPath
let request_id = pf_queue.request_path(
    entity,
    current_pos,
    target_tile,
    PathPriority::Lazy,      // Lowest priority
    PathReason::Wandering,   // For metrics
    tick,
);

// In WanderState::WaitingForPath
match pf_queue.get_result(request_id) {
    Some(PathResult::Success { path, .. }) => { /* start moving */ }
    Some(PathResult::Failed { reason, .. }) => { /* retry or fail */ }
    None => { /* wait another tick */ }
}
```

---

## Retry Logic

**Max Retries**: 3 (configurable)

**Retry Flow**:
1. Path fails (unreachable, timeout, etc.)
2. `retry_count++`
3. If `retry_count < 3`: Go to `NeedPath` (try again)
4. Else: Return `ActionResult::Failed` (give up)

---

## Testing

**Run Phase 2 tests**:
```bash
cargo test --test wander_queue_integration
```

**Expected output**:
```
test test_wander_queues_path_request ... ok
test test_wander_waits_for_path_result ... ok
test test_wander_moves_after_path_success ... ok
test test_wander_retries_after_path_failure ... ok
test test_wander_gives_up_after_max_retries ... ok

test result: ok. 5 passed; 0 failed
```

---

## Performance

**Before (Synchronous)**:
```
Entity wanders → Calls grid.find_path() → BLOCKS until path computed
```

**After (Asynchronous)**:
```
Tick 1: Entity wanders → Queue path request → Continue tick
Tick 2-N: Check result → Still pending → Continue tick
Tick N+1: Result ready → Start moving → Non-blocking
```

**Priority**:
- Wander uses `PathPriority::Lazy` (processed after urgent/normal)
- Can wait 10-20 ticks for path (wandering is patient)
- Allows fleeing/feeding to get paths first

---

## Files Modified

- `src/ai/action.rs` - WanderAction implementation
- `tests/wander_queue_integration.rs` - Integration tests

---

## Next Actions to Integrate

1. **GrazeAction** (PathPriority::Normal) - Moving to food
2. **DrinkWaterAction** (PathPriority::Normal) - Moving to water
3. **FleeAction** (PathPriority::Urgent) - Fleeing predators
4. **HuntAction** (PathPriority::Normal) - Hunting prey
5. **MateAction** (PathPriority::Normal) - Meeting partner

**Pattern to follow**: Same state machine as WanderAction, adjust priority
