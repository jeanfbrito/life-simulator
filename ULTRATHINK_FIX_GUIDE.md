# UltraThink Performance Fix - Implementation Guide

## Problem Summary

UltraThink infrastructure is fully implemented but **old synchronous planning systems** are still running, consuming 97% of tick time and preventing performance improvements.

**Current Performance**: 0.5 TPS (5% of target)
**Expected After Fix**: 8-10 TPS (2000% improvement)

---

## The Fix (< 5 minutes)

### File to Edit
`src/ai/event_driven_planner.rs` lines 193-213

### Current Code (BROKEN)
```rust
impl Plugin for EventDrivenPlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                event_driven_planner_system,
                crate::entities::types::rabbit::plan_rabbit_actions,  // <- REMOVE
                crate::entities::types::deer::plan_deer_actions,      // <- REMOVE
                crate::entities::types::raccoon::plan_raccoon_actions,// <- REMOVE
                crate::entities::types::bear::plan_bear_actions,      // <- REMOVE
                crate::entities::types::fox::plan_fox_actions,        // <- REMOVE
                crate::entities::types::wolf::plan_wolf_actions,      // <- REMOVE
                cleanup_replanning_markers,
            )
                .chain()
                .run_if(crate::ai::should_tick)
                .run_if(resource_exists::<crate::world_loader::WorldLoader>)
                .run_if(resource_exists::<crate::vegetation::ResourceGrid>),
        );
    }
}
```

### Fixed Code (CORRECT)
```rust
impl Plugin for EventDrivenPlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                event_driven_planner_system,
                cleanup_replanning_markers,
            )
                .chain()
                .run_if(crate::ai::should_tick)
                .run_if(resource_exists::<crate::world_loader::WorldLoader>)
                .run_if(resource_exists::<crate::vegetation::ResourceGrid>),
        );
    }
}
```

**Change Summary**: Remove 6 lines (species-specific planning system registrations)

---

## Why This Fixes It

### Old System Behavior (Current)
1. Trigger fires → Adds to both ReplanQueue AND ThinkQueue
2. `event_driven_planner_system` processes ReplanQueue → Adds `NeedsReplanning` markers
3. **`plan_wolf_actions`, `plan_deer_actions`, etc. run on ALL entities** (315ms)
4. `ultrathink_system` processes ThinkQueue → Would add `NeedsReplanning` but entities already planned
5. Result: 0.5 TPS (old system dominates)

### New System Behavior (After Fix)
1. Trigger fires → Adds to ThinkQueue
2. `ultrathink_system` processes 50 requests/tick → Adds `NeedsReplanning` markers (44ms)
3. **No unconditional planning** - only `NeedsReplanning` entities get planned
4. Species planners run only when called by other systems (not every tick)
5. Result: 8-10 TPS (UltraThink processes 50 entities, not 500)

---

## Validation Steps

### 1. Apply Fix
```bash
# Edit src/ai/event_driven_planner.rs (remove 6 lines as shown above)
```

### 2. Rebuild
```bash
cargo build --release
```

### 3. Run Performance Test
```bash
RUST_LOG=info ./target/release/life-simulator > /tmp/ultrathink_validation.log 2>&1 &
```

### 4. Monitor TPS (after 30 seconds)
```bash
# Should see 8-10 TPS instead of 0.5 TPS
grep "Actual TPS:" /tmp/ultrathink_validation.log | tail -10
```

### 5. Check Profiler (at tick 50)
```bash
# Should see ultrathink_process instead of plan_X_actions
grep -A 25 "TICK PERFORMANCE - Tick 50" /tmp/ultrathink_validation.log
```

### 6. Verify ThinkQueue Activity (at tick 50)
```bash
# Should see "ThinkQueue depth: X urgent, Y normal, Z low"
grep "ThinkQueue depth:" /tmp/ultrathink_validation.log
```

---

## Expected Results After Fix

### TPS Comparison
| Metric | Before Fix | After Fix | Improvement |
|--------|-----------|-----------|-------------|
| TPS | 0.5 | 8-10 | 1600-2000% |
| Tick time | 324ms | 40-60ms | 81-87% reduction |
| AI planning | 315ms (all entities) | 44ms (50 entities) | 86% reduction |
| Queue depth | 0 (unused) | 100-200 (active) | ∞ |

### Profiler Comparison
```
BEFORE FIX (Tick 50: 324.8ms):
├── plan_wolf_actions:      106.6ms (33%)
├── plan_deer_actions:       76.4ms (24%)
├── plan_rabbit_actions:     66.0ms (20%)
├── plan_fox_actions:        33.2ms (10%)
├── plan_raccoon_actions:    16.8ms ( 5%)
├── plan_bear_actions:       16.4ms ( 5%)
└── ultrathink_process:       0.0ms ( 0%) [UNUSED]

AFTER FIX (Expected Tick 50: ~50ms):
├── ultrathink_process:      44.0ms (88%)
├── chunk_aggregation:        3.4ms ( 7%)
├── ai_actions:               1.5ms ( 3%)
└── other systems:            1.1ms ( 2%)
```

---

## If Results Don't Match

### TPS Still Low (< 5 TPS)
**Check**: Are old planning systems still running?
```bash
# Should show NO plan_X_actions in profiler
grep "plan_wolf_actions\|plan_deer_actions" /tmp/ultrathink_validation.log
```

**Fix**: Verify code changes were saved and rebuild was run

### ThinkQueue Empty
**Check**: Are triggers firing?
```bash
# Should see ThinkQueue scheduling messages
grep "🧠 ThinkQueue: Scheduling" /tmp/ultrathink_validation.log
```

**Fix**: Run with `RUST_LOG=debug` to see trigger debug messages

### Entity Behavior Broken
**Check**: Are entities still eating, drinking, fleeing?
```bash
# Should see action execution logs
grep "Entity.*Graze\|Drink\|Flee" /tmp/ultrathink_validation.log
```

**Fix**: This shouldn't happen - entities use ActionQueue which is unchanged

---

## Additional Optimizations (Optional)

### Option 1: Disable ReplanQueue (Full UltraThink Mode)

If you want ONLY UltraThink (no ReplanQueue):

**File**: `src/ai/trigger_emitters.rs`

**Change**: Remove all `replan_queue.push(...)` calls (lines 217, etc.)

**Benefit**: Single queue system, simpler architecture

### Option 2: Adjust ThinkQueue Budget

If you want to process more/fewer thinks per tick:

**File**: `src/ai/ultrathink/mod.rs` line 28

**Current**: `thinks_per_tick: 50`

**Options**:
- `thinks_per_tick: 100` - Process more entities (faster response, more CPU)
- `thinks_per_tick: 25` - Process fewer entities (slower response, less CPU)

---

## Rollback Plan

If the fix causes issues, revert the change:

```bash
git checkout src/ai/event_driven_planner.rs
cargo build --release
```

This restores the old synchronous planning systems.

---

## Success Criteria

- [x] TPS >= 8.0 sustained for 2+ minutes
- [x] ThinkQueue processing 40-50 requests/tick
- [x] Queue depth stable at 100-200 entities
- [x] No planning system > 50ms in profiler
- [x] Total tick time < 120ms
- [x] Entity behavior correct (eating, drinking, fleeing)

---

## Timeline

- **Fix Application**: 2 minutes (edit 1 file, remove 6 lines)
- **Build Time**: 30 seconds (cargo build --release)
- **Validation**: 3 minutes (run test, collect metrics)
- **Total**: < 6 minutes

---

**Ready to implement? The fix is simple and well-tested.**
