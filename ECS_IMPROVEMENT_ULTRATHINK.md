# ECS Architecture Improvement - UltraThink Analysis

**Date**: 2025-12-26
**Goal**: Identify opportunities to better leverage Bevy's ECS for performance and code quality
**Context**: Current implementation uses ECS well in some areas, but has resource-heavy patterns that could be more ECS-native

---

## Current ECS Usage Analysis

### ✅ What We're Doing Well

1. **Marker Components** (UltraThink pattern)
   ```rust
   #[derive(Component)]
   pub struct NeedsReplanning { reason: String }

   // Usage: Mark entities that need planning
   commands.entity(entity).insert(NeedsReplanning { reason });

   // Query: Only process marked entities
   for (entity, needs) in query.iter() { ... }
   ```
   **Rating**: ✅ Excellent - This is pure ECS

2. **Entity Stats as Components**
   ```rust
   #[derive(Component)]
   pub struct Hunger(pub Stat);

   #[derive(Component)]
   pub struct Thirst(pub Stat);
   ```
   **Rating**: ✅ Good - Components for entity properties

3. **Resource-Based Queues** (UltraThink, ReplanQueue, PathfindingQueue)
   ```rust
   #[derive(Resource)]
   pub struct ThinkQueue { ... }
   ```
   **Rating**: ⚠️ Mixed - Good for global state, but could complement with components

### ❌ What Could Be More ECS-Native

1. **ActionQueue Stores Entity State in HashMap**
   ```rust
   // Current (Anti-pattern)
   #[derive(Resource)]
   pub struct ActionQueue {
       active_actions: HashMap<Entity, ActiveAction>,  // ❌ HashMap lookup
       pending_actions: VecDeque<QueuedAction>,
   }
   ```
   **Problem**:
   - HashMap lookups instead of component queries
   - No parallelization (single Resource)
   - Manual entity lifetime management
   - Can't use Bevy's change detection

   **Better ECS Pattern**:
   ```rust
   // Store action state as component on entity
   #[derive(Component)]
   pub struct ActiveAction {
       action: Box<dyn Action>,
       priority: i32,
       started_tick: u64,
   }

   // Query instead of HashMap lookup
   fn execute_actions(
       mut query: Query<(Entity, &mut ActiveAction, &TilePosition, ...)>,
   ) {
       query.par_iter_mut().for_each(|(entity, mut action, pos, ...)| {
           // Parallel execution! Each entity independent
           action.action.execute(...);
       });
   }
   ```

2. **PathfindingQueue Results in HashMap**
   ```rust
   // Current
   pub struct PathfindingQueue {
       completed_paths: HashMap<PathRequestId, PathResult>,  // ❌ HashMap
   }
   ```
   **Better ECS Pattern**:
   ```rust
   #[derive(Component)]
   pub struct PathingTo {
       request_id: PathRequestId,
       target: IVec2,
       priority: PathPriority,
       state: PathingState,
   }

   #[derive(Component)]
   pub struct PathResult {
       path: Vec<IVec2>,
       computed_tick: u64,
   }

   // Actions query for path result
   fn execute_move_action(
       query: Query<(Entity, &ActiveAction, Option<&PathResult>)>,
   ) {
       for (entity, action, path_result) in query.iter() {
           if let Some(result) = path_result {
               // Path ready! Use it
           }
       }
   }
   ```

3. **Spatial Index in HashMap**
   ```rust
   // Current
   pub struct SpatialEntityIndex {
       chunks: HashMap<IVec2, HashSet<Entity>>,  // ❌ Manual spatial tracking
   }
   ```
   **Better ECS Pattern**:
   ```rust
   // Use Bevy's hierarchy + spatial components
   #[derive(Component)]
   pub struct SpatialChunk(pub IVec2);

   #[derive(Component)]
   pub struct ChunkMembership(pub IVec2);

   // Hierarchical organization (Bevy handles Parent/Child)
   commands.spawn((
       SpatialChunk(chunk_pos),
       // Bevy automatically tracks children
   ));

   commands.entity(entity).insert(ChunkMembership(chunk_pos));

   // Query entities in chunk using Bevy's hierarchy
   fn query_chunk_entities(
       chunks: Query<(&SpatialChunk, &Children)>,
   ) { ... }
   ```

4. **State Machines Buried in Resources**
   ```rust
   // Current: WanderState inside ActionQueue HashMap
   enum WanderState {
       NeedPath,
       WaitingForPath { request_id },
       Moving { path, index },
   }

   // ❌ Hidden in ActionQueue resource
   ```
   **Better ECS Pattern**:
   ```rust
   // State as component (queryable, visible)
   #[derive(Component)]
   pub enum MovementState {
       Idle,
       RequestingPath { target: IVec2, request_id: PathRequestId },
       FollowingPath { path: Vec<IVec2>, index: usize },
   }

   // Query entities by state
   fn handle_idle_entities(
       query: Query<Entity, With<MovementState::Idle>>,
   ) { ... }
   ```

---

## Improvement Roadmap

### Phase 1: Actions as Components (High Impact)

**Current Problem**:
```rust
// ActionQueue resource with HashMap
active_actions: HashMap<Entity, ActiveAction>

// Usage requires resource access
fn some_system(mut queue: ResMut<ActionQueue>) {
    if let Some(action) = queue.active_actions.get(&entity) { ... }
}
```

**ECS-Native Solution**:
```rust
#[derive(Component)]
pub struct CurrentAction {
    pub action_type: ActionType,
    pub state: ActionState,
    pub priority: i32,
    pub started_tick: u64,
}

// Parallel query execution
fn execute_actions(
    mut query: Query<(Entity, &mut CurrentAction, &TilePosition, &mut Hunger, ...)>,
    world: &World,
) {
    // Bevy can parallelize this across entities!
    query.par_iter_mut().for_each_mut(|(entity, mut action, pos, hunger, ...)| {
        match action.state {
            ActionState::Executing => {
                // Execute action logic
            }
            ActionState::Completed => {
                // Will be removed by cleanup system
            }
        }
    });
}

// Cleanup system (automatic)
fn cleanup_completed_actions(
    mut commands: Commands,
    query: Query<(Entity, &CurrentAction), With<ActionState::Completed>>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).remove::<CurrentAction>();
    }
}
```

**Benefits**:
- ✅ Parallel execution (Bevy handles thread safety)
- ✅ No HashMap lookups (direct component access)
- ✅ Change detection (only process changed entities)
- ✅ Automatic cleanup when entity despawns
- ✅ Queries can filter by action state

**Effort**: 4-6 hours (moderate refactor)
**Impact**: High (enables parallelism, cleaner code)

---

### Phase 2: PathResult as Component (Medium Impact)

**Current Problem**:
```rust
// PathfindingQueue stores results
completed_paths: HashMap<PathRequestId, PathResult>

// Actions must query resource
fn execute_action(pf_queue: Res<PathfindingQueue>) {
    if let Some(result) = pf_queue.get_result(&request_id) { ... }
}
```

**ECS-Native Solution**:
```rust
#[derive(Component)]
pub struct PathRequested {
    pub request_id: PathRequestId,
    pub target: IVec2,
    pub priority: PathPriority,
    pub requested_tick: u64,
}

#[derive(Component)]
pub struct PathReady {
    pub path: Vec<IVec2>,
    pub computed_tick: u64,
}

// Pathfinding system adds PathReady when done
fn process_pathfinding(
    mut commands: Commands,
    requests: Query<(Entity, &PathRequested), Without<PathReady>>,
    grid: Res<PathfindingGrid>,
) {
    for (entity, request) in requests.iter().take(50) {  // Budget
        if let Some(path) = grid.find_path(...) {
            commands.entity(entity).insert(PathReady { path, ... });
        }
    }
}

// Actions query for ready paths
fn execute_move_action(
    mut query: Query<(Entity, &mut MovementState, Option<&PathReady>)>,
) {
    for (entity, mut state, path_ready) in query.iter_mut() {
        match state {
            MovementState::RequestingPath { .. } => {
                if let Some(ready) = path_ready {
                    *state = MovementState::FollowingPath {
                        path: ready.path.clone(),
                        index: 0,
                    };
                }
            }
        }
    }
}
```

**Benefits**:
- ✅ Component-based queries (no HashMap)
- ✅ Automatic cleanup (components removed when entity despawns)
- ✅ Change detection (react when PathReady added)
- ✅ Can query "all entities waiting for paths"

**Effort**: 3-4 hours
**Impact**: Medium (cleaner code, better ECS patterns)

---

### Phase 3: Movement State as Component (Medium Impact)

**Current Problem**:
```rust
// WanderState hidden inside ActionQueue HashMap
enum WanderState { NeedPath, WaitingForPath, Moving }

// Can't query "all wandering entities"
// Can't see state without ActionQueue access
```

**ECS-Native Solution**:
```rust
#[derive(Component)]
pub enum MovementState {
    Idle,
    Wandering {
        target: IVec2,
        path: Option<Vec<IVec2>>,
        current_index: usize,
    },
    MovingToResource {
        resource_pos: IVec2,
        path: Option<Vec<IVec2>>,
        current_index: usize,
    },
    Fleeing {
        from: IVec2,
        path: Option<Vec<IVec2>>,
        current_index: usize,
    },
}

// Query by movement type
fn update_wandering(
    mut query: Query<(Entity, &mut MovementState, &TilePosition), With<MovementState::Wandering>>,
) {
    for (entity, mut state, pos) in query.iter_mut() {
        if let MovementState::Wandering { path, current_index, .. } = state.as_mut() {
            // Update wandering
        }
    }
}

// Debugging: "Show me all fleeing entities"
fn debug_fleeing(
    query: Query<(Entity, &MovementState), With<MovementState::Fleeing>>,
) {
    println!("Fleeing entities: {}", query.iter().count());
}
```

**Benefits**:
- ✅ Queryable by state (filter idle, wandering, fleeing)
- ✅ Visible in ECS inspector/debugger
- ✅ Can react to state changes with change detection
- ✅ Separate systems for different movement types

**Effort**: 3-4 hours
**Impact**: Medium (better visibility, debuggability)

---

### Phase 4: Spatial Hierarchy with Bevy's Parent/Child (High Impact)

**Current Problem**:
```rust
// Manual HashMap tracking
pub struct SpatialEntityIndex {
    chunks: HashMap<IVec2, HashSet<Entity>>,
}

// Manual insert/remove
spatial_index.insert(entity, chunk_pos);
spatial_index.remove(entity);
```

**ECS-Native Solution**:
```rust
// Spatial chunk as entity with children
#[derive(Component)]
pub struct SpatialChunk {
    pub position: IVec2,
}

// Entities are children of their chunk
commands.spawn((
    SpatialChunk { position: IVec2::new(0, 0) },
    SpatialHierarchyRoot,
));

// Add entity to chunk using Bevy's hierarchy
commands.entity(chunk_entity).add_child(entity);

// Query entities in chunk using Bevy's Children
fn query_chunk(
    chunks: Query<(&SpatialChunk, &Children)>,
    entities: Query<&TilePosition>,
) {
    for (chunk, children) in chunks.iter() {
        for &child in children.iter() {
            if let Ok(pos) = entities.get(child) {
                // Entity in this chunk
            }
        }
    }
}

// Bevy automatically handles:
// - Parent/Child relationships
// - Cleanup when parent despawns
// - Hierarchy iteration
```

**Benefits**:
- ✅ Bevy manages relationships (no manual HashMap)
- ✅ Automatic cleanup (children removed when parent despawns)
- ✅ Built-in hierarchy queries
- ✅ Transform propagation (if using Bevy transforms)

**Effort**: 6-8 hours (significant refactor)
**Impact**: High (removes manual spatial tracking, better ECS)

---

### Phase 5: Event-Driven Communication (Medium Impact)

**Current Problem**:
```rust
// Queues as resources
ReplanQueue, ThinkQueue, PathfindingQueue

// Systems manually push/drain
queue.push(entity, reason);
let requests = queue.drain(budget);
```

**ECS-Native Solution**:
```rust
// Use Bevy Events for communication
#[derive(Event)]
pub struct ReplanRequested {
    pub entity: Entity,
    pub reason: String,
    pub priority: ReplanPriority,
}

#[derive(Event)]
pub struct PathRequested {
    pub entity: Entity,
    pub from: IVec2,
    pub to: IVec2,
    pub priority: PathPriority,
}

// Systems send events
fn trigger_replan(
    mut events: EventWriter<ReplanRequested>,
) {
    events.send(ReplanRequested { entity, reason, priority });
}

// Systems receive events
fn handle_replan_requests(
    mut commands: Commands,
    mut events: EventReader<ReplanRequested>,
) {
    for event in events.read().take(50) {  // Budget
        commands.entity(event.entity).insert(NeedsReplanning { reason: event.reason });
    }
}
```

**Benefits**:
- ✅ Bevy manages event ordering
- ✅ Automatic cleanup (events cleared each frame)
- ✅ Multiple systems can listen
- ✅ Type-safe communication

**Tradeoff**: Events are single-frame, queues persist
**Solution**: Hybrid - Events for triggers, Components for persistent state

**Effort**: 2-3 hours
**Impact**: Medium (cleaner inter-system communication)

---

### Phase 6: System Sets and Parallelism (High Impact)

**Current Problem**:
```rust
// Systems run sequentially
app.add_systems(FixedUpdate, (
    event_driven_planner_system,
    plan_rabbit_actions,
    plan_deer_actions,
    // ... all sequential
).chain());
```

**ECS-Native Solution**:
```rust
// Organize systems into sets
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    // Planning phase (can parallelize by species)
    Planning,
    // Action execution (can parallelize by entity)
    ActionExecution,
    // Movement (can parallelize by entity)
    Movement,
    // Cleanup
    Cleanup,
}

app.configure_sets(FixedUpdate, (
    SimulationSet::Planning,
    SimulationSet::ActionExecution,
    SimulationSet::Movement,
    SimulationSet::Cleanup,
).chain());

// Systems in same set can run in parallel!
app.add_systems(FixedUpdate, (
    plan_rabbit_actions,
    plan_deer_actions,
    plan_fox_actions,
    plan_wolf_actions,
    plan_bear_actions,
    plan_raccoon_actions,
).in_set(SimulationSet::Planning));  // All run in parallel!

app.add_systems(FixedUpdate, (
    execute_wander_actions,
    execute_graze_actions,
    execute_drink_actions,
    execute_hunt_actions,
).in_set(SimulationSet::ActionExecution));  // All parallel!
```

**Benefits**:
- ✅ Maximum CPU utilization (Bevy parallelizes automatically)
- ✅ Clear execution order (sets define phases)
- ✅ Better organization (systems grouped by purpose)
- ✅ Easier to add new systems

**Effort**: 4-5 hours
**Impact**: High (better CPU usage, clearer architecture)

---

## Performance Comparison

### Current Architecture (Resource-Heavy)
```rust
// Sequential execution
fn execute_all_actions(
    mut queue: ResMut<ActionQueue>,  // Single resource = no parallelism
) {
    for (entity, action) in queue.active_actions.iter_mut() {
        // Process one at a time
        action.execute(...);
    }
}

// CPU usage: 1 core (sequential)
// Execution time: N × action_time
```

### ECS-Native Architecture (Component-Based)
```rust
// Parallel execution
fn execute_all_actions(
    mut query: Query<(Entity, &mut CurrentAction, ...)>,
) {
    query.par_iter_mut().for_each_mut(|(entity, mut action, ...)| {
        // Bevy runs these in parallel across multiple cores!
        action.execute(...);
    });
}

// CPU usage: All cores (parallel)
// Execution time: (N × action_time) / num_cores
// Expected speedup: 4-8x on 8-core CPU
```

---

## Recommended Implementation Order

### Priority 1 (High Impact, Moderate Effort)
1. **Actions as Components** (Phase 1)
   - Enables parallelism
   - Removes HashMap overhead
   - Foundation for other improvements
   - **Effort**: 4-6 hours
   - **Payoff**: 2-4x speedup potential

2. **System Sets for Parallelism** (Phase 6)
   - Organize existing systems
   - Enable parallel execution
   - **Effort**: 4-5 hours
   - **Payoff**: 2-4x speedup potential

### Priority 2 (Medium Impact, Low-Medium Effort)
3. **PathResult as Component** (Phase 2)
   - Natural extension of Phase 1
   - Cleaner code
   - **Effort**: 3-4 hours
   - **Payoff**: Better ECS patterns, no performance gain

4. **Movement State as Component** (Phase 3)
   - Better debugging visibility
   - State-based queries
   - **Effort**: 3-4 hours
   - **Payoff**: Better debuggability

### Priority 3 (Future Work)
5. **Spatial Hierarchy** (Phase 4)
   - Significant refactor
   - Defer until other improvements done
   - **Effort**: 6-8 hours
   - **Payoff**: Cleaner spatial queries

6. **Event-Driven Communication** (Phase 5)
   - Nice-to-have enhancement
   - Hybrid approach (Events + Components)
   - **Effort**: 2-3 hours
   - **Payoff**: Cleaner communication

---

## Expected Performance Gains

### Current Performance
- TPS: 10.0 (with UltraThink + Pathfinding Queue)
- CPU utilization: ~1-2 cores (mostly sequential)
- Action execution: Sequential per entity

### After ECS Improvements (Phases 1+6)
- TPS: **15-20** (estimated with parallelism)
- CPU utilization: **6-8 cores** (parallel systems)
- Action execution: **Parallel across entities**

**Calculation**:
- Phase 1 (Actions as Components): 2-4x speedup from parallel action execution
- Phase 6 (System Sets): 2-3x speedup from parallel planning
- Combined: **4-8x total speedup potential**

**Realistic Expectation**:
- Conservative: 15 TPS (50% improvement)
- Optimistic: 20-25 TPS (100-150% improvement)

---

## Code Examples: Before vs After

### Example 1: Action Execution

**Before (Current)**:
```rust
// Sequential, resource-heavy
fn execute_actions(
    mut queue: ResMut<ActionQueue>,
    world: &mut World,
) {
    let entities: Vec<Entity> = queue.active_actions.keys().copied().collect();

    for entity in entities {
        if let Some(action) = queue.active_actions.get_mut(&entity) {
            // Sequential execution
            action.action.execute(world, entity);
        }
    }
}

// Problems:
// ❌ Sequential (1 core)
// ❌ HashMap lookup
// ❌ No change detection
```

**After (ECS-Native)**:
```rust
// Parallel, component-based
fn execute_actions(
    mut query: Query<(Entity, &mut CurrentAction, &TilePosition, &mut Hunger, ...)>,
    world: &World,
) {
    query.par_iter_mut().for_each_mut(|(entity, mut action, pos, hunger, ...)| {
        // Parallel execution across all cores!
        match &mut action.state {
            ActionState::Grazing { duration, elapsed } => {
                // Execute graze logic
            }
            ActionState::Moving { path, index } => {
                // Execute movement
            }
        }
    });
}

// Benefits:
// ✅ Parallel (8 cores)
// ✅ Component query (no HashMap)
// ✅ Change detection available
```

### Example 2: Pathfinding Results

**Before (Current)**:
```rust
// HashMap in resource
fn check_path_ready(
    pf_queue: Res<PathfindingQueue>,
    entity: Entity,
    request_id: PathRequestId,
) -> Option<Vec<IVec2>> {
    pf_queue.get_result(&request_id)
        .and_then(|result| match result {
            PathResult::Success { path, .. } => Some(path.clone()),
            _ => None,
        })
}

// Problems:
// ❌ HashMap lookup
// ❌ Manual lifetime management
// ❌ Can't query "all entities with ready paths"
```

**After (ECS-Native)**:
```rust
// Component on entity
fn check_path_ready(
    query: Query<&PathReady>,
    entity: Entity,
) -> Option<Vec<IVec2>> {
    query.get(entity).ok()
        .map(|ready| ready.path.clone())
}

// Or better: react to path becoming ready
fn on_path_ready(
    query: Query<(Entity, &PathReady, &mut MovementState), Changed<PathReady>>,
) {
    // Only processes entities where PathReady was just added!
    for (entity, path, mut state) in query.iter_mut() {
        *state = MovementState::FollowingPath {
            path: path.path.clone(),
            index: 0,
        };
    }
}

// Benefits:
// ✅ Component query
// ✅ Automatic cleanup
// ✅ Change detection (reactive)
// ✅ Can query all ready paths
```

---

## Conclusion

**Recommended Path Forward**:

1. **Start with Phase 1 (Actions as Components)** - Highest impact, enables parallelism
2. **Follow with Phase 6 (System Sets)** - Organize systems, enable parallel planning
3. **Then Phase 2 (PathResult as Component)** - Natural extension of Phase 1
4. **Optional: Phases 3-5** - Quality of life improvements

**Expected Timeline**:
- Phase 1: 4-6 hours (high impact)
- Phase 6: 4-5 hours (high impact)
- Total: **8-11 hours for 4-8x speedup potential**

**Performance Target**:
- Current: 10.0 TPS
- After Phases 1+6: **15-25 TPS** (50-150% improvement)

The key insight: **Move from resource-heavy HashMap patterns to component-based queries** to unlock Bevy's parallel execution capabilities. The current architecture works well (10.0 TPS achieved), but has significant untapped performance potential through better ECS utilization.

**Status**: Ready to implement when you want to pursue these improvements! 🚀
