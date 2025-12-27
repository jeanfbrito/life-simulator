# Action Pathfinding Quick Reference
**Updated**: 2025-12-26 (Post Phase 3)

## Overview
All movement-based actions use PathfindingQueue for async, priority-based pathfinding.

## Priority Levels

| Priority | Processing Time | Use Cases |
|----------|----------------|-----------|
| **Urgent** | 1-2 ticks | Fleeing from predators (future) |
| **Normal** | 3-5 ticks | Food, water, hunting, mating |
| **Lazy** | 10-20 ticks | Wandering, exploration |

## Action Priority Assignment

```rust
// Current implementation (Phase 3 complete)
DrinkWaterAction  → PathPriority::Normal  (PathReason::MovingToWater)
GrazeAction       → PathPriority::Normal  (PathReason::MovingToFood)
HuntAction        → PathPriority::Normal  (PathReason::Hunting)
WanderAction      → PathPriority::Lazy    (PathReason::Wandering)

// Future (Phase 4)
FleeAction        → PathPriority::Urgent  (PathReason::FleeingPredator)
```

## Usage Pattern

### 1. Add State Enum to Action

```rust
#[derive(Debug, Clone)]
enum MyActionState {
    NeedPath,
    WaitingForPath { request_id: PathRequestId },
    Moving { path: Vec<IVec2>, current_index: usize },
    Executing,  // Optional: for actions that do something at target
}
```

### 2. Add Retry Tracking

```rust
pub struct MyAction {
    target: IVec2,
    state: MyActionState,
    retry_count: u32,
    max_retries: u32,  // 3 for Normal, 5 for Urgent
}
```

### 3. Implement State Machine in execute()

```rust
fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
    use crate::pathfinding::{PathPriority, PathReason, PathResult};

    let current_pos = world.get::<TilePosition>(entity)?.tile;

    match &self.state {
        MyActionState::NeedPath => {
            let mut pf_queue = world.get_resource_mut::<PathfindingQueue>()?;

            let request_id = pf_queue.request_path(
                entity,
                current_pos,
                self.target,
                PathPriority::Normal,  // Or Urgent/Lazy
                PathReason::MovingToFood,
                tick,
            );

            self.state = MyActionState::WaitingForPath { request_id };
            ActionResult::InProgress
        }

        MyActionState::WaitingForPath { request_id } => {
            let pf_queue = world.get_resource::<PathfindingQueue>()?;

            match pf_queue.get_result(*request_id).cloned() {
                Some(PathResult::Success { path, .. }) => {
                    // Insert MoveOrder to start movement
                    world.get_entity_mut(entity)?.insert(MoveOrder {
                        destination: self.target,
                        allow_diagonal: true,
                    });

                    self.state = MyActionState::Moving {
                        path: path.clone(),
                        current_index: 0,
                    };
                    ActionResult::InProgress
                }
                Some(PathResult::Failed { reason, .. }) => {
                    // Retry logic
                    if self.retry_count < self.max_retries {
                        self.retry_count += 1;
                        self.state = MyActionState::NeedPath;
                        ActionResult::InProgress
                    } else {
                        ActionResult::Failed
                    }
                }
                None => ActionResult::InProgress  // Still waiting
            }
        }

        MyActionState::Moving { .. } => {
            // Check for pathfinding failure from movement system
            if world.get::<PathfindingFailed>(entity).is_some() {
                if self.retry_count < self.max_retries {
                    self.retry_count += 1;
                    self.state = MyActionState::NeedPath;
                    world.get_entity_mut(entity)?.remove::<PathfindingFailed>();
                    return ActionResult::InProgress;
                } else {
                    return ActionResult::Failed;
                }
            }

            // Continue moving (movement system handles actual movement)
            ActionResult::InProgress
        }

        MyActionState::Executing => {
            // Perform the action (drink, eat, etc.)
            // ...
            ActionResult::Success
        }
    }
}
```

### 4. Implement cancel()

```rust
fn cancel(&mut self, world: &mut World, entity: Entity) {
    // Clear navigation state
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        entity_mut.remove::<MoveOrder>();
        entity_mut.remove::<Path>();
        entity_mut.remove::<PathfindingFailed>();
    }

    // Reset state machine
    self.state = MyActionState::NeedPath;
    self.retry_count = 0;
}
```

## Common Mistakes to Avoid

1. **Don't call `find_path()` synchronously**
   ```rust
   // ❌ BAD - Synchronous pathfinding
   let path = find_path(from, to, grid, false, Some(1000));

   // ✅ GOOD - Async pathfinding via queue
   let request_id = pf_queue.request_path(entity, from, to, priority, reason, tick);
   ```

2. **Don't forget retry logic**
   ```rust
   // ❌ BAD - Immediate failure
   PathResult::Failed { .. } => ActionResult::Failed,

   // ✅ GOOD - Retry with limit
   PathResult::Failed { .. } => {
       if self.retry_count < self.max_retries {
           self.retry_count += 1;
           self.state = NeedPath;
           ActionResult::InProgress
       } else {
           ActionResult::Failed
       }
   }
   ```

3. **Don't forget to check PathfindingFailed component**
   ```rust
   // ✅ GOOD - Check for movement system failures
   if world.get::<PathfindingFailed>(entity).is_some() {
       // Handle failure and retry
   }
   ```

4. **Don't forget to remove PathfindingFailed after handling**
   ```rust
   // ✅ GOOD - Clean up failure marker
   world.get_entity_mut(entity)?.remove::<PathfindingFailed>();
   ```

## Testing Actions

```rust
#[test]
fn test_my_action_uses_correct_priority() {
    let mut world = World::new();
    world.insert_resource(PathfindingQueue::default());

    let entity = world.spawn(TilePosition::from_tile(IVec2::new(10, 10))).id();
    let mut action = MyAction::new(IVec2::new(20, 20));

    // Execute once to queue path
    action.execute(&mut world, entity, 0);

    // Verify priority
    let queue = world.get_resource::<PathfindingQueue>().unwrap();
    let (urgent, normal, lazy) = queue.queue_sizes();

    assert_eq!(normal, 1, "Should use Normal priority");
}
```

## Performance Characteristics

### Budget Control
- **Default budget**: 40-50 paths per tick
- **Prevents spikes**: Smooth pathfinding cost across ticks
- **Priority-based**: Urgent → Normal → Lazy processing order

### Example Queue State
```
PathfindingQueue: 2 urgent, 8 normal, 15 lazy | Processed 40/40 | Total: 1,234
                  ↑         ↑         ↑           ↑
                  Flee      Food      Wander      Per-tick budget
                            Water
                            Hunt
```

## Files to Reference

- **Core implementation**: `src/ai/action.rs`
- **Queue system**: `src/pathfinding/pathfinding_queue.rs`
- **Request types**: `src/pathfinding/path_request.rs`
- **Integration tests**: `tests/action_queue_integration.rs`
- **Phase 2 reference**: `WANDER_PATHFINDING_QUICK_REF.md`

## Quick Checklist for New Actions

- [ ] Add state enum (NeedPath, WaitingForPath, Moving, Executing)
- [ ] Add retry tracking (retry_count, max_retries)
- [ ] Implement NeedPath state (queue path request)
- [ ] Implement WaitingForPath state (check result, handle retry)
- [ ] Implement Moving state (check PathfindingFailed, continue)
- [ ] Implement Executing state (perform action)
- [ ] Implement cancel() (clear navigation, reset state)
- [ ] Choose correct priority (Urgent/Normal/Lazy)
- [ ] Choose correct reason (FleeingPredator/MovingToFood/etc.)
- [ ] Write integration test
- [ ] Verify no synchronous find_path() calls

---

**Status**: All core actions integrated with PathfindingQueue
**Next**: Integrate Flee action with Urgent priority (Phase 4)
