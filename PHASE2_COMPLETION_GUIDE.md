# Phase 2 Completion Guide - Quick Reference

**Time Required**: 1-2 hours
**Complexity**: Low (pattern replication)
**Risk**: Low (backward compatible, incremental changes)

---

## Remaining Tasks

### Task 1: Migrate Remaining Actions (45 minutes)

**Files to modify**: `src/ai/action.rs`

**Actions needing update**:
1. Graze - line ~565
2. Hunt - line ~1060
3. Wander - line ~1771

**Pattern to apply** (copy from DrinkWater at line ~287):

```rust
// BEFORE (Old HashMap pattern)
ActionState::WaitingForPath { request_id } => {
    let path_result = {
        let Some(pf_queue) = world.get_resource::<crate::pathfinding::PathfindingQueue>() else {
            return ActionResult::Failed;
        };
        pf_queue.get_result(*request_id).cloned()
    };

    match path_result {
        Some(PathResult::Success { path, .. }) => { /* transition to Moving */ }
        Some(PathResult::Failed { reason, .. }) => { /* retry or fail */ }
        None => ActionResult::InProgress,
    }
}

// AFTER (New Component pattern)
ActionState::WaitingForPath { request_id } => {
    let entity_ref = world.get_entity(entity).ok();

    if let Some(entity_ref) = entity_ref {
        // Check for PathReady component
        if let Some(path_ready) = entity_ref.get::<crate::pathfinding::PathReady>() {
            let path = path_ready.path.clone();

            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.insert(MoveOrder { destination: target, allow_diagonal: true });
                entity_mut.remove::<crate::pathfinding::PathReady>();
            }

            self.state = ActionState::Moving { path, current_index: 0 };
            return ActionResult::InProgress;
        }

        // Check for PathFailed component
        if let Some(path_failed) = entity_ref.get::<crate::pathfinding::PathFailed>() {
            let reason = path_failed.reason;

            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.remove::<crate::pathfinding::PathFailed>();
            }

            if self.retry_count < self.max_retries {
                self.retry_count += 1;
                self.state = ActionState::NeedPath;
                return ActionResult::InProgress;
            } else {
                return ActionResult::Failed;
            }
        }
    }

    ActionResult::InProgress // Still waiting
}
```

**Verification**:
```bash
cargo test --lib action  # Should pass
cargo build --release    # Should succeed
```

---

### Task 2: Remove HashMap Storage (30 minutes)

**File**: `src/pathfinding/pathfinding_queue.rs`

**Step 1**: Remove completed_paths field
```rust
// DELETE THIS
#[derive(Resource)]
pub struct PathfindingQueue {
    urgent_queue: VecDeque<PathRequest>,
    normal_queue: VecDeque<PathRequest>,
    lazy_queue: VecDeque<PathRequest>,
    paths_per_tick: usize,
    completed_paths: HashMap<PathRequestId, PathResult>, // ❌ DELETE
    pending_requests: HashSet<(Entity, IVec2, IVec2)>,
    next_id: u64,
    total_paths_processed: u64,
}
```

**Step 2**: Update new() constructor
```rust
// DELETE completed_paths initialization
impl PathfindingQueue {
    pub fn new(paths_per_tick: usize) -> Self {
        Self {
            urgent_queue: VecDeque::new(),
            normal_queue: VecDeque::new(),
            lazy_queue: VecDeque::new(),
            paths_per_tick,
            // completed_paths: HashMap::new(), // ❌ DELETE
            pending_requests: HashSet::new(),
            next_id: 1,
            total_paths_processed: 0,
        }
    }
}
```

**Step 3**: Remove get_result() and store_result() methods
```rust
// DELETE THESE METHODS (lines ~151-159)
pub fn store_result(&mut self, id: PathRequestId, result: PathResult) {
    self.completed_paths.insert(id, result);
}

pub fn get_result(&self, id: PathRequestId) -> Option<&PathResult> {
    self.completed_paths.get(&id)
}
```

**Step 4**: Clean up process_pathfinding_queue
File: `src/pathfinding/mod.rs`

```rust
// DELETE legacy HashMap storage (lines ~110-121)
// Remove this entire block:
let result = match path_opt {
    Some(path) => PathResult::Success {
        path: path.all_waypoints().to_vec(),
        computed_tick: tick.0,
    },
    None => PathResult::Failed {
        reason: PathFailureReason::Unreachable,
        retry_count: 0,
    },
};
queue.store_result(request.id, result); // ❌ DELETE
```

**Verification**:
```bash
cargo test --lib pathfinding  # Should pass
cargo build --release         # Should succeed
```

---

### Task 3: Integration Testing (15 minutes)

**Run simulation**:
```bash
cargo run --release --bin life-simulator
```

**Monitor**:
- Console output for TPS: Should show ~10.0 TPS
- Watch for pathfinding errors (should be none)
- Let run for ~30 seconds to verify stability

**Expected output**:
```
🎮 Tick 1000 | TPS: 10.0 | Entities: 150
🗺️ PathfindingQueue: 2 urgent, 15 normal, 8 lazy | Processed 40/40 | Total: 25000
✅ No errors
```

**If errors occur**:
1. Check DrinkWater/Graze/Hunt/Wander WaitingForPath handlers
2. Verify PathReady/PathFailed components are removed after use
3. Ensure request_path_with_component() is used

---

### Task 4: Update Tests (15 minutes)

**File**: `tests/path_component_storage_test.rs`

**Update test_pathfinding_queue_no_completed_paths_hashmap**:
```rust
#[test]
fn test_pathfinding_queue_no_completed_paths_hashmap() {
    // Verify that PathfindingQueue no longer has completed_paths HashMap
    let queue = PathfindingQueue::new(10);

    // Compilation check: these methods should NOT exist
    // Uncommenting these should cause compile errors:
    // queue.get_result(PathRequestId::new(1));  // Should fail to compile
    // queue.store_result(...);                  // Should fail to compile

    assert_eq!(queue.paths_per_tick(), 10);
    assert_eq!(queue.total_queued(), 0);
}
```

**Run all tests**:
```bash
cargo test path_component_storage_test
cargo test --lib
```

---

## Validation Checklist

Before marking Phase 2 complete:

- [ ] All 4 actions (DrinkWater, Graze, Hunt, Wander) use component queries
- [ ] `completed_paths` HashMap removed from PathfindingQueue
- [ ] `get_result()` and `store_result()` methods deleted
- [ ] Legacy HashMap code removed from process_pathfinding_queue
- [ ] All unit tests passing (8/8 path_component_storage_test)
- [ ] All lib tests passing (274+ tests)
- [ ] Integration test run successful (cargo run --release)
- [ ] 10 TPS maintained (verified in console output)
- [ ] No pathfinding errors in logs
- [ ] Release build successful

---

## Emergency Rollback Plan

If issues arise:

**Step 1**: Revert action changes
```bash
git checkout src/ai/action.rs
```

**Step 2**: Restore HashMap (if already removed)
```rust
// Add back to PathfindingQueue
completed_paths: HashMap<PathRequestId, PathResult>,

// Add back methods
pub fn store_result(&mut self, id: PathRequestId, result: PathResult) {
    self.completed_paths.insert(id, result);
}

pub fn get_result(&self, id: PathRequestId) -> Option<&PathResult> {
    self.completed_paths.get(&id)
}
```

**Step 3**: Test rollback
```bash
cargo test --lib
cargo run --release
```

---

## Expected Outcome

**After completion**:
- ✅ Full Phase 2 implementation complete
- ✅ No HashMap lookups in pathfinding
- ✅ All actions use reactive component queries
- ✅ 10 TPS maintained
- ✅ Ready for Phase 3 (Movement State as Component)

**Performance**:
- No TPS regression (10.0 maintained)
- Slight memory reduction (HashMap removed)
- No behavioral changes to simulation

**Code Quality**:
- More ECS-idiomatic
- Better testability
- Easier debugging (component inspector)
- Preparation for future parallelism

---

## Time Breakdown

| Task | Est. Time | Complexity |
|------|-----------|------------|
| Migrate Graze | 15 min | Low |
| Migrate Hunt | 15 min | Low |
| Migrate Wander | 15 min | Low |
| Remove HashMap | 30 min | Low |
| Integration Test | 15 min | Low |
| Update Tests | 15 min | Low |
| **Total** | **1h 45min** | **Low** |

---

## Notes

- **Backward Compatible**: Old get_result() calls will fail to compile (intentional)
- **Incremental**: Can be done one action at a time
- **Low Risk**: Components already working, just pattern replication
- **No Rush**: Can complete in multiple sessions if needed

---

**Last Updated**: 2025-12-26
**Status**: Ready for execution
**Next Agent**: Can be completed by infrastructure-implementation-agent or manually
