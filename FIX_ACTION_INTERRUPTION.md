# Critical Bug Fix: Action Interruption by Force Periodic Replanning

## Problem Statement

The `force_periodic_replanning` system was interrupting active actions before they could complete:

1. Rabbit starts **Graze action** (20-tick duration)
2. **Force replanning runs every 10 ticks** (at tick 10, 20, 30, etc.)
3. Force replanning **queues a NEW Graze action**, canceling the current one
4. **Graze action NEVER completes**, hunger never decreases
5. Rabbit gets stuck in a loop of interrupted actions

### Root Cause

The `force_periodic_replanning` function was forcing **ALL entities** to replan every 10 ticks, regardless of whether they had an active action in progress. This is an emergency workaround that was designed to handle a broken trigger system, but it had an unintended side effect: it would interrupt multi-tick actions.

### Why This Matters

- Entities couldn't complete actions like Graze (20 ticks), DrinkWater (15 ticks), Rest (30 ticks)
- Hunger/Thirst/Energy stats never changed
- Simulation appeared functional (entities had plans) but nothing was actually happening
- This created the false impression that actions were executing when they weren't

## Solution

Modified `src/ai/force_planning.rs` to **only force replanning for IDLE entities** by adding `Without<ActiveAction>` to all queries:

### Key Changes

**Before (Lines 45-50):**
```rust
rabbits: Query<Entity, With<Rabbit>>,
deer: Query<Entity, With<Deer>>,
raccoons: Query<Entity, With<Raccoon>>,
bears: Query<Entity, With<Bear>>,
foxes: Query<Entity, With<Fox>>,
wolves: Query<Entity, With<Wolf>>,
```

**After (Lines 59-64):**
```rust
rabbits: Query<Entity, (With<Rabbit>, Without<ActiveAction>)>,
deer: Query<Entity, (With<Deer>, Without<ActiveAction>)>,
raccoons: Query<Entity, (With<Raccoon>, Without<ActiveAction>)>,
bears: Query<Entity, (With<Bear>, Without<ActiveAction>)>,
foxes: Query<Entity, (With<Fox>, Without<ActiveAction>)>,
wolves: Query<Entity, (With<Wolf>, Without<ActiveAction>)>,
```

### Why This Works

- `Without<ActiveAction>` filters to only entities that DON'T have an active action
- Entities actively executing actions (Graze, DrinkWater, etc.) are skipped
- Actions can now complete their full duration without interruption
- Idle entities still get force-replanned every 10 ticks to stay functional

### Execution Flow

**With the Fix:**

```
Tick 0: Rabbit idle, force replanning triggered
  -> Insert NeedsReplanning
  -> Event-driven planner selects Graze action (20 ticks)
  -> Insert ActiveAction component

Tick 10: Force replanning checks
  -> Rabbit has ActiveAction? YES
  -> SKIP (don't force replanning)
  -> Graze continues (10 ticks of 20 remaining)

Tick 15: Event-driven planner runs
  -> Hunger decreased due to Graze
  -> No replanning needed (action still active)
  -> Graze continues (5 ticks of 20 remaining)

Tick 20: Graze action completes
  -> ActiveAction removed
  -> Hunger has decreased
  -> Rabbit is now idle

Tick 20: Force replanning runs (10-tick boundary)
  -> Rabbit is idle (no ActiveAction)
  -> INCLUDE in replanning
  -> Re-evaluate utilities
  -> Select next action (Drink, Rest, Wander, etc.)
```

## Testing

Created `src/bin/test_action_completion.rs` to verify:

1. Entity spawns with a Rabbit marker
2. NeedsReplanning is inserted to trigger planning
3. Test monitors ActiveAction and NeedsReplanning components
4. At tick 15 (5 ticks into a 20-tick action):
   - Verifies ActiveAction still exists (action not removed)
   - Verifies NeedsReplanning NOT inserted (action not interrupted)
5. Confirms action can complete naturally

### Test Build
```bash
cargo build --bin test_action_completion
```

### Running the Test
```bash
./target/debug/test_action_completion
```

## Impact on Other Systems

**Action Execution (src/ai/queue.rs):**
- No changes needed
- When action completes, `handle_action_results` removes `ActiveAction`
- Hunger/Thirst/Energy are updated when action completes

**Event-Driven Planner (src/ai/event_driven_planner.rs):**
- No changes needed
- Only processes entities with `NeedsReplanning`
- Already respects active actions in its logic

**Emergency Workaround Status:**
- This remains an emergency fix for the broken trigger system
- But now it's surgical: only affects truly idle entities
- Once the real trigger system is fixed, this can be removed

## Files Modified

1. **src/ai/force_planning.rs**
   - Added `ActiveAction` import
   - Modified all 6 entity queries to include `Without<ActiveAction>`
   - Updated documentation to explain the fix
   - Clarified logging message

2. **src/bin/test_action_completion.rs** (new)
   - Test binary to verify action completion without interruption
   - Monitors ActiveAction and NeedsReplanning components
   - Validates the fix is working correctly

## Verification

```bash
# 1. Verify code compiles
cargo check
cargo build

# 2. Run the test
./target/debug/test_action_completion

# 3. Check main simulation
cargo run --bin life-simulator
```

## Performance Impact

**Positive:**
- Fewer forced replans (only idle entities)
- Reduced CPU work in force_periodic_replanning
- Entities complete actions faster

**Neutral:**
- No change to overall simulation performance
- Same 10 TPS tick rate
- Same number of ticks between replans for idle entities

## Future Improvements

1. **Remove this workaround:** Once the real trigger system is fixed, delete `src/ai/force_planning.rs` entirely
2. **Proper trigger system:** The trigger system should detect when entities need replanning (idle, needs changed, threat detected, etc.)
3. **Event-driven planning:** Entities should plan reactively (when needs exceed thresholds) not periodically
4. **Monitor emergency mode:** Track how many entities are force-replanned each tick (should be small number of idle entities)

## Regression Testing

To ensure this fix doesn't break anything:

1. Verify hunger/thirst decrease over time when entities Graze/Drink
2. Verify predators complete Hunting actions
3. Verify mating actions complete naturally
4. Verify Rest actions complete (energy recovers)
5. Monitor forced replan counts in logs (should be small, mostly idle entities)

## Conclusion

This fix is surgical and low-risk:
- Adds one ECS filter condition (`Without<ActiveAction>`)
- Doesn't remove existing functionality
- Maintains the emergency workaround for broken triggers
- Allows normal action completion flow
- Includes test to verify correctness
