# Queued Pathfinding Architecture Plan
## Multithreading-Ready Design

**Date**: 2025-12-26
**Goal**: Queue-based pathfinding with budget control, ready for future multithreading
**Inspiration**: UltraThink success (87x speedup) + Dwarf Fortress job system

---

## Vision

**Current Problem**:
```
Entity plans → Needs to move → Calls pathfinding.find_path() SYNCHRONOUSLY
Result: Pathfinding spikes when many entities plan
        High failure rate (timeouts, unreachable tiles)
        Blocks planning system execution
```

**Queue-Based Solution**:
```
Entity plans → Queues PathfindingRequest with priority → Continues planning
Pathfinding System → Processes N requests per tick (budget: 30-50)
Entity → Checks if path ready → Executes movement OR retries later
Result: Smooth pathfinding cost across ticks
        Priority-based (fleeing > needs > wandering)
        Non-blocking, multithreading-ready architecture
```

---

## Architecture Overview

### Core Components

#### 1. PathfindingQueue (Budget-Controlled Processing)
```rust
pub struct PathfindingQueue {
    // Priority queues (like UltraThink)
    urgent_queue: VecDeque<PathRequest>,    // Fleeing from predators
    normal_queue: VecDeque<PathRequest>,    // Moving to food/water
    lazy_queue: VecDeque<PathRequest>,      // Wandering exploration

    // Budget control
    paths_per_tick: usize,                  // Default: 40-50

    // Result storage
    completed_paths: HashMap<PathRequestId, PathResult>,

    // Deduplication
    pending_requests: HashSet<(Entity, IVec2, IVec2)>,
}
```

#### 2. PathRequest (Priority-Based Requests)
```rust
pub struct PathRequest {
    id: PathRequestId,           // Unique ID for tracking
    entity: Entity,              // Requesting entity
    from: IVec2,                 // Start position
    to: IVec2,                   // Goal position
    priority: PathPriority,      // Urgent/Normal/Lazy
    reason: PathReason,          // Why path needed
    requested_tick: u64,         // When requested
    timeout_ticks: u64,          // Max wait time
}

pub enum PathPriority {
    Urgent,   // Fleeing, critical - process within 1-2 ticks
    Normal,   // Food, water, resources - process within 3-5 ticks
    Lazy,     // Wandering, exploration - process within 10-20 ticks
}

pub enum PathReason {
    FleeingPredator,
    MovingToFood,
    MovingToWater,
    MovingToMate,
    Wandering,
    Hunting,
}
```

#### 3. PathResult (Cached Results)
```rust
pub enum PathResult {
    Success {
        path: Vec<IVec2>,
        computed_tick: u64,
        cost: f32,
    },
    Failed {
        reason: PathFailureReason,
        retry_count: u32,
    },
    Pending,  // Still computing
}

pub enum PathFailureReason {
    Unreachable,
    Timeout,
    InvalidStart,
    InvalidGoal,
}
```

---

## Processing Flow

### Request Phase (Entity Planning)
```rust
// When entity needs to move (in planning system)
fn plan_movement_to_resource(entity: Entity, from: IVec2, to: IVec2) {
    // Instead of: let path = pathfinding_grid.find_path(from, to);

    // Queue pathfinding request
    let request_id = pathfinding_queue.request_path(
        entity,
        from,
        to,
        PathPriority::Normal,
        PathReason::MovingToFood,
    );

    // Store request ID in action state
    queue_action(entity, Action::WaitForPath { request_id });
}
```

### Processing Phase (Pathfinding System)
```rust
fn process_pathfinding_queue(
    mut queue: ResMut<PathfindingQueue>,
    grid: Res<PathfindingGrid>,
    tick: Res<SimulationTick>,
    profiler: ResMut<TickProfiler>,
) {
    let budget = queue.paths_per_tick;
    let requests = queue.drain(budget);  // Get up to N requests

    for request in requests {
        // Compute path (currently single-threaded)
        let result = match grid.find_path(request.from, request.to) {
            Some(path) => PathResult::Success {
                path,
                computed_tick: tick.0,
                cost: path.len() as f32,
            },
            None => PathResult::Failed {
                reason: PathFailureReason::Unreachable,
                retry_count: 0,
            },
        };

        // Store result for entity to retrieve
        queue.completed_paths.insert(request.id, result);
    }
}
```

### Consumption Phase (Action Execution)
```rust
fn execute_wait_for_path_action(
    entity: Entity,
    request_id: PathRequestId,
    pathfinding_queue: &PathfindingQueue,
) -> ActionResult {
    match pathfinding_queue.get_result(request_id) {
        Some(PathResult::Success { path, .. }) => {
            // Path ready! Start moving
            ActionResult::Complete(Some(Action::MovePath {
                path: path.clone(),
                current_index: 0,
            }))
        }
        Some(PathResult::Failed { reason, .. }) => {
            // Pathfinding failed
            warn!("Path failed for {:?}: {:?}", entity, reason);
            ActionResult::Failed
        }
        None => {
            // Still pending, wait another tick
            ActionResult::InProgress
        }
    }
}
```

---

## Multithreading Readiness

### Phase 1: Single-Threaded Queue (Current Implementation)
```rust
// Process paths sequentially (like current UltraThink)
for request in requests {
    let result = grid.find_path(request.from, request.to);
    queue.completed_paths.insert(request.id, result);
}
```

### Phase 2: Parallel Processing (Future Enhancement)
```rust
use rayon::prelude::*;

// Process paths in parallel using all CPU cores
let results: Vec<(PathRequestId, PathResult)> = requests
    .par_iter()  // Rayon parallel iterator
    .map(|request| {
        let result = grid.find_path(request.from, request.to);
        (request.id, result)
    })
    .collect();

// Collect results back to main thread
for (id, result) in results {
    queue.completed_paths.insert(id, result);
}
```

**Why This Architecture Supports Multithreading**:
1. **PathfindingGrid is already Arc-wrapped**: Thread-safe reads
2. **Queue-based**: Clean separation of request/process/consume
3. **No shared mutable state during computation**: Each path computation is independent
4. **Result collection is single-threaded**: Safe HashMap insertion in main thread

### Phase 3: Thread Pool (Advanced - Future)
```rust
use std::sync::mpsc;

// Dedicated pathfinding thread pool
struct PathfindingThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<PathRequest>,
}

// Workers process paths continuously
// Main thread queues requests, collects results
// Zero blocking on main simulation thread
```

---

## Integration Points

### 1. Wander Action (HIGH PRIORITY)
**Current** (synchronous):
```rust
// In Wander action execute()
let target = pick_random_wander_target();
let path = grid.find_path(current_pos, target)?;  // BLOCKS HERE
```

**Queued** (asynchronous):
```rust
// When Wander action starts
let target = pick_random_wander_target();
let request_id = pathfinding_queue.request_path(
    entity, current_pos, target,
    PathPriority::Lazy,  // Low priority for wandering
    PathReason::Wandering,
);
self.state = WanderState::WaitingForPath { request_id };

// In next tick's execute()
match self.state {
    WanderState::WaitingForPath { request_id } => {
        if let Some(PathResult::Success { path, .. }) = queue.get_result(request_id) {
            self.state = WanderState::Moving { path, index: 0 };
        }
    }
    // ...
}
```

### 2. MoveTo Action (Food, Water, Resources)
**Priority**: Normal (need-based, not urgent)
```rust
pathfinding_queue.request_path(
    entity, current_pos, resource_pos,
    PathPriority::Normal,
    PathReason::MovingToFood,
);
```

### 3. Flee Action (From Predators)
**Priority**: Urgent (life-threatening)
```rust
pathfinding_queue.request_path(
    entity, current_pos, safe_pos,
    PathPriority::Urgent,
    PathReason::FleeingPredator,
);
```

### 4. Hunt Action (Predators)
**Priority**: Normal (hunting behavior)
```rust
pathfinding_queue.request_path(
    entity, predator_pos, prey_pos,
    PathPriority::Normal,
    PathReason::Hunting,
);
```

---

## Performance Expectations

### Current Pathfinding Cost (Estimated)
```
Baseline (500 entities, many planning):
- 200+ entities attempt pathfinding per tick
- Each path: ~0.5-2.0ms (varies by distance)
- Spike: 100-400ms when burst occurs
- Result: TPS drops, action failures
```

### Queued Pathfinding Cost (Expected)
```
With Queue (budget: 40 paths/tick):
- Process exactly 40 paths per tick
- Cost: 40 × 1.0ms avg = 40ms/tick
- Smooth, predictable cost
- Result: Stable TPS, fewer failures
```

### With Multithreading (Future)
```
With 8 CPU cores:
- Process 40 paths in parallel
- Cost: 40 / 8 = 5ms effective time
- Can increase budget to 100+ paths/tick
- Result: Even higher TPS possible
```

---

## Implementation Phases

### Phase 1: Core Queue Infrastructure (4-5 hours)
**Files to create**:
- `src/pathfinding/pathfinding_queue.rs` - PathfindingQueue implementation
- `src/pathfinding/path_request.rs` - PathRequest, PathResult types
- `tests/pathfinding_queue_test.rs` - Queue behavior tests

**Deliverables**:
1. PathfindingQueue resource with priority queues
2. request_path(), drain(), get_result() methods
3. Budget processing (40-50 paths/tick)
4. Result caching and deduplication
5. Unit tests (5-7 tests)

**Success Criteria**:
- All tests passing
- Queue prioritizes correctly (Urgent > Normal > Lazy)
- Budget limits enforced
- No duplicate requests

---

### Phase 2: Wander Action Integration (3-4 hours)
**Files to modify**:
- `src/ai/action.rs` - Add WaitForPath action state
- `src/ai/behaviors/wandering.rs` - Modify Wander to queue paths

**Deliverables**:
1. WaitForPath action type
2. Wander action uses queued pathfinding
3. Graceful handling of path failures (retry logic)
4. Integration test

**Success Criteria**:
- Entities successfully wander using queued paths
- Failed paths handled gracefully (retry or give up)
- No blocking pathfinding in Wander action

---

### Phase 3: Other Action Integration (2-3 hours)
**Files to modify**:
- `src/ai/behaviors/eating.rs` - MoveTo for food
- `src/ai/behaviors/drinking.rs` - MoveTo for water
- `src/ai/behaviors/fleeing.rs` - Flee with urgent priority

**Deliverables**:
1. All movement actions use PathfindingQueue
2. Correct priority assignment per action type
3. Integration tests

**Success Criteria**:
- Fleeing uses Urgent priority
- Food/water uses Normal priority
- All actions non-blocking

---

### Phase 4: Adaptive Budget (1-2 hours)
**Files to modify**:
- `src/pathfinding/pathfinding_queue.rs` - Add adaptive budget logic

**Deliverables**:
1. Monitor queue depth over time
2. Increase budget when queue growing
3. Decrease budget when queue empty
4. Metrics logging

**Success Criteria**:
- Budget adjusts dynamically
- Queue depth remains stable
- Pathfinding doesn't starve

---

### Phase 5: Performance Validation (1-2 hours)
**Testing**:
1. Run with 500 entities, measure TPS
2. Compare vs baseline (current UltraThink)
3. Verify pathfinding smoothness
4. Check action success rate

**Expected Results**:
- TPS maintains 10.0 (possibly improves)
- Pathfinding cost smooth (~40ms/tick)
- Fewer action failures
- Better entity behavior

---

## Multithreading Future Work (Phase 6 - Later)

### Step 1: Rayon Parallel Processing
```rust
// Add to Cargo.toml
rayon = "1.8"

// In pathfinding_queue.rs
let results: Vec<_> = requests
    .par_iter()
    .map(|req| compute_path(req, &grid))
    .collect();
```

**Expected Speedup**: 4-8x on 8-core CPU

### Step 2: Thread Pool (Optional)
- Dedicated pathfinding worker threads
- Async request/response
- Zero blocking on main thread

**Expected Benefit**: Can increase budget to 100-200 paths/tick without affecting TPS

---

## Risk Mitigation

**Risk 1: Queue backlog (paths requested faster than processed)**
- Mitigation: Adaptive budget increases when queue grows
- Fallback: Priority ensures urgent paths never starved

**Risk 2: Path failures cause action gridlock**
- Mitigation: Retry logic with exponential backoff
- Fallback: Give up after N retries, choose new action

**Risk 3: Multithreading complexity**
- Mitigation: Phase 1-5 are single-threaded, proven architecture
- Phase 6 is optional enhancement, not required

**Risk 4: Entity behavior changes**
- Mitigation: Async path requests maintain same logic
- Fallback: WaitForPath action bridges old/new behavior

---

## Success Metrics

### Must Have
- ✅ 10.0 TPS maintained with queued pathfinding
- ✅ Pathfinding cost smooth (<50ms/tick)
- ✅ Queue depth stable (not growing unbounded)
- ✅ All entities eventually get paths

### Nice to Have
- 🎯 Improved action success rate (fewer failures)
- 🎯 Adaptive budget working smoothly
- 🎯 Queue metrics visible in logs
- 🎯 Architecture ready for Rayon integration

---

## Estimated Total Effort

**Phase 1-5** (Single-threaded, production-ready):
- Core infrastructure: 4-5 hours
- Wander integration: 3-4 hours
- Other actions: 2-3 hours
- Adaptive budget: 1-2 hours
- Validation: 1-2 hours
**Total: 11-16 hours**

**Phase 6** (Multithreading - future):
- Rayon integration: 2-3 hours
- Testing/tuning: 2-3 hours
**Total: 4-6 hours** (optional, later)

---

## Next Steps

1. **Review this plan** - Get user approval on architecture
2. **Create pathfinding queue module** - Scaffold files
3. **Implement Phase 1** - Core queue infrastructure
4. **Test Phase 1** - Verify queue works independently
5. **Implement Phase 2** - Wander action integration
6. **Iterate through phases** - Each phase adds capability

**This architecture mirrors UltraThink's success** - queue-based, priority-driven, budget-controlled, and multithreading-ready! 🎯
