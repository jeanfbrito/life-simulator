# ECS Improvement Roadmap - Complete Implementation Plan

**Date**: 2025-12-26
**Status**: Phase 1 Complete ✅ | Phases 2-6 Planned
**Performance Target**: Maintain 10.0 TPS (user constraint: "we will keep it at 10TPS for definition")
**Goal**: Better ECS architecture through component-based design, NOT performance gains

---

## Executive Summary

Converting resource-heavy HashMap patterns to component-based ECS architecture for:
- Better code quality and maintainability
- Automatic cleanup on entity despawn
- Query-based access instead of manual HashMap lookups
- Preparation for future parallel execution (optional)

**Critical Constraint**: 10 TPS is the target performance. Do not exceed it. Focus is architectural improvement.

---

## Phase 1: Actions as Components ✅ COMPLETE

**Status**: ✅ SHIPPED & VALIDATED (2025-12-26)

### Implementation Summary
- **Component Created**: `ActiveAction` (src/entities/active_action.rs)
- **Refactored**: ActionQueue HashMap → Component storage
- **Removed**: HashMap<Entity, ActiveAction> from ActionQueue resource
- **Updated**: execute_active_actions() uses Query instead of HashMap iteration

### Code Pattern
```rust
// BEFORE (Resource-Heavy)
#[derive(Resource)]
pub struct ActionQueue {
    active_actions: HashMap<Entity, ActiveAction>,  // ❌ Removed
}

fn execute_actions(mut queue: ResMut<ActionQueue>) {
    for (entity, action) in queue.active_actions.iter_mut() {
        action.execute(); // Sequential
    }
}

// AFTER (Component-Based)
#[derive(Component)]
pub struct ActiveAction {
    pub action: Box<dyn Action>,
    pub started_at_tick: u64,
}

fn execute_actions(
    query: Query<(Entity, &ActiveAction)>,
    world: &mut World,
) {
    for (entity, active) in query.iter(world) {
        // Execute with component-based access
    }
}
```

### Performance Validation
- **TPS**: 10.0 sustained ✅ (no regression)
- **Tick Time**: 5.2ms average (baseline: 4.8-5.3ms) ✅
- **Tests**: 274/274 unit tests passing, 5/5 integration tests passing ✅
- **Builds**: Release build successful ✅

### Architectural Benefits
1. **Automatic Cleanup**: Components removed when entities despawn (no manual tracking)
2. **Query-Based Access**: No HashMap lookups needed
3. **ECS-Native**: Leverages Bevy's component storage
4. **Future-Ready**: Enables parallel iteration with par_iter_mut()

### Files Modified
- Created: `src/entities/active_action.rs`
- Created: `tests/action_component_storage_test.rs`
- Modified: `src/ai/queue.rs`
- Modified: `src/entities/mod.rs`

### Deliverables
- `PHASE1_PERFORMANCE_VALIDATION.md` - Performance validation report
- `PHASE1_VALIDATION_SUMMARY.txt` - Quick reference

---

## Phase 2: PathResult as Component 🎯 NEXT

**Status**: 🔜 PLANNED
**Effort**: 3-4 hours
**Risk**: Low (similar to Phase 1)
**Priority**: High (natural continuation of Phase 1)

### Current Architecture
```rust
// PathfindingQueue resource stores results in HashMap
#[derive(Resource)]
pub struct PathfindingQueue {
    completed_paths: HashMap<PathRequestId, PathResult>,  // ❌ To be replaced
    urgent_queue: VecDeque<PathRequest>,
    normal_queue: VecDeque<PathRequest>,
    lazy_queue: VecDeque<PathRequest>,
}
```

### Target Architecture
```rust
// PathRequested component when entity requests path
#[derive(Component)]
pub struct PathRequested {
    pub request_id: PathRequestId,
    pub target: IVec2,
    pub priority: PathPriority,
    pub requested_tick: u64,
}

// PathReady component when path is computed
#[derive(Component)]
pub struct PathReady {
    pub path: Vec<IVec2>,
    pub computed_tick: u64,
    pub cost: f32,
}

// PathFailed component when path computation fails
#[derive(Component)]
pub struct PathFailed {
    pub reason: PathFailureReason,
    pub retry_count: u32,
}
```

### Reactive System with Change Detection
```rust
// System reacts when PathReady component is added
fn on_path_ready(
    query: Query<(Entity, &PathReady, &mut MovementState), Changed<PathReady>>,
) {
    for (entity, path_ready, mut movement) in query.iter_mut() {
        // Automatically transition to moving when path ready
        *movement = MovementState::FollowingPath {
            path: path_ready.path.clone(),
            index: 0,
        };
    }
}
```

### Benefits
1. **Change Detection**: Systems react when PathReady added (no polling)
2. **Automatic Cleanup**: Path components removed when entity despawns
3. **Query-Based**: No HashMap lookups in PathfindingQueue
4. **State Visibility**: Path state visible in component inspector

### Implementation Steps
1. Create PathRequested, PathReady, PathFailed components
2. Refactor PathfindingQueue to remove completed_paths HashMap
3. Update path request system to insert PathRequested component
4. Update path processing to insert PathReady/PathFailed components
5. Update action systems to use Changed<PathReady> queries
6. Remove manual result retrieval code
7. Write TDD tests for component-based pathfinding
8. Validate 10 TPS maintained

### Success Criteria
- PathfindingQueue.completed_paths HashMap removed
- All path requests use component storage
- Change detection working (on_path_ready triggers)
- All tests passing
- 10 TPS maintained (no regression)

---

## Phase 3: Movement State as Component 🎯 PLANNED

**Status**: 🔜 PLANNED
**Effort**: 3-4 hours
**Risk**: Low (similar to Phase 1-2)
**Priority**: Medium (completes the action/path/movement trilogy)

### Current Architecture
```rust
// Movement state stored in action state machines
enum WanderState {
    NeedPath,
    WaitingForPath { request_id: PathRequestId },
    Moving { path: Vec<IVec2>, current_index: usize },
}
```

### Target Architecture
```rust
#[derive(Component)]
pub enum MovementState {
    Idle,
    PathRequested { request_id: PathRequestId },
    FollowingPath { path: Vec<IVec2>, index: usize },
    Stuck { attempts: u32 },
}

// Separate movement execution system
fn execute_movement(
    mut query: Query<(Entity, &mut MovementState, &mut TilePosition)>,
    tick: Res<SimulationTick>,
) {
    for (entity, mut state, mut pos) in query.iter_mut() {
        match *state {
            MovementState::FollowingPath { ref path, ref mut index } => {
                if *index < path.len() {
                    *pos = TilePosition(path[*index]);
                    *index += 1;
                } else {
                    *state = MovementState::Idle;
                }
            }
            _ => {}
        }
    }
}
```

### Benefits
1. **Separation of Concerns**: Movement logic separate from action logic
2. **Visibility**: Movement state visible in component inspector
3. **Reusability**: Any system can query/modify movement state
4. **Debugging**: Easier to debug movement issues

### Implementation Steps
1. Create MovementState component enum
2. Extract movement logic from action state machines
3. Create execute_movement system
4. Update actions to use MovementState component
5. Remove movement state from action structs
6. Write TDD tests
7. Validate 10 TPS maintained

---

## Phase 4: Spatial Hierarchy with Parent/Child ⚠️ HIGH RISK

**Status**: 🔜 PLANNED (DEFER UNTIL SPATIAL WORK COMPLETES)
**Effort**: 6-8 hours
**Risk**: HIGH - Conflicts with active spatial index work
**Priority**: Low (can defer until spatial/vegetation features stabilize)

### Current Architecture
```rust
// Manual HashMap-based spatial tracking
#[derive(Resource)]
pub struct SpatialEntityIndex {
    cell_to_entities: HashMap<IVec2, Vec<Entity>>,  // ❌ To be replaced
}
```

### Target Architecture
```rust
// Use Bevy's built-in parent/child hierarchy
#[derive(Component)]
pub struct SpatialCell;  // Marker component for spatial cells

// Entities are children of spatial cells
commands.entity(cell_entity).add_child(animal_entity);

// Query hierarchy
fn query_nearby_entities(
    cells: Query<(&Children, &TilePosition), With<SpatialCell>>,
    entities: Query<&Species>,
) {
    for (children, cell_pos) in cells.iter() {
        for &child in children.iter() {
            if let Ok(species) = entities.get(child) {
                // Process entity in this cell
            }
        }
    }
}
```

### ⚠️ CONFLICT WARNING
**Active Development Detected**:
- `SPATIAL_GRID_INTEGRATION.md`
- `SPATIAL_INDEX_FEAR_INTEGRATION.md`
- `SPATIAL_MATE_MATCHING_DELIVERY.md`
- `tests/fear_spatial_index_integration.rs`
- `tests/spatial_mate_integration_test.rs`

**Recommendation**: Defer Phase 4 until spatial index work stabilizes to avoid merge conflicts.

### Benefits (When Implemented)
1. **Built-in Hierarchy**: Use Bevy's Parent/Child system
2. **Automatic Updates**: Children automatically follow parent
3. **Query Optimization**: Bevy optimizes hierarchy queries
4. **Spatial Queries**: Natural "entities in cell" queries

### Implementation Steps (When Ready)
1. Create SpatialCell marker component
2. Spawn spatial cell entities as grid
3. Reparent entities to spatial cells
4. Update spatial queries to use Children component
5. Remove SpatialEntityIndex HashMap
6. Write extensive tests (hierarchy is complex)
7. Validate 10 TPS maintained

---

## Phase 5: Event-Driven Communication 🎯 PLANNED

**Status**: 🔜 PLANNED
**Effort**: 2-3 hours
**Risk**: Low-Medium
**Priority**: Medium (improves reactivity)

### Current Architecture
```rust
// Polling-based: systems check every tick
fn check_health_system(query: Query<&Health>) {
    for health in query.iter() {
        if health.current <= 0 {
            // React to death
        }
    }
}
```

### Target Architecture
```rust
// Event-driven: systems react to events
#[derive(Event)]
pub struct EntityDied {
    pub entity: Entity,
    pub cause: DeathCause,
}

fn detect_death(
    query: Query<(Entity, &Health), Changed<Health>>,
    mut events: EventWriter<EntityDied>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0 {
            events.send(EntityDied {
                entity,
                cause: DeathCause::Starvation,
            });
        }
    }
}

fn handle_death(
    mut events: EventReader<EntityDied>,
    mut commands: Commands,
) {
    for event in events.read() {
        commands.entity(event.entity).despawn();
    }
}
```

### Benefits
1. **Reactive**: Systems only run when events occur
2. **Decoupled**: Event producers don't know consumers
3. **Efficient**: No polling every tick
4. **Debuggable**: Event stream visible

### Event Types to Implement
- EntityDied
- EntityBorn
- ActionCompleted
- PathCompleted
- StatCritical (hunger, thirst, etc.)
- PredatorDetected
- MateFound

### Implementation Steps
1. Define event types
2. Create event writer systems (producers)
3. Create event reader systems (consumers)
4. Replace polling with event-driven logic
5. Write tests for event flows
6. Validate 10 TPS maintained

---

## Phase 6: System Sets and Parallelism 🎯 PLANNED

**Status**: 🔜 PLANNED
**Effort**: 4-5 hours
**Risk**: Medium (requires careful ordering)
**Priority**: Low (optimization, not needed for 10 TPS target)

### Current Architecture
```rust
// Systems run sequentially
app.add_systems(FixedUpdate, (
    plan_rabbit_actions,
    plan_deer_actions,
    execute_actions,
    execute_movement,
));
```

### Target Architecture
```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    Planning,
    ActionExecution,
    Movement,
    Cleanup,
}

// Systems in same set run in parallel
app.add_systems(FixedUpdate, (
    // All planning systems run in parallel (different species)
    plan_rabbit_actions,
    plan_deer_actions,
    plan_fox_actions,
    plan_wolf_actions,
).in_set(SimulationSet::Planning))
.add_systems(FixedUpdate, (
    execute_actions,
).in_set(SimulationSet::ActionExecution).after(SimulationSet::Planning))
.add_systems(FixedUpdate, (
    execute_movement,
).in_set(SimulationSet::Movement).after(SimulationSet::ActionExecution))
.add_systems(FixedUpdate, (
    cleanup_dead_entities,
).in_set(SimulationSet::Cleanup).after(SimulationSet::Movement));
```

### Benefits
1. **Parallelism**: Systems in same set run on multiple cores
2. **Ordering**: Clear execution order between sets
3. **Optimization**: Bevy handles thread safety
4. **Scalability**: Better multi-core utilization

### ⚠️ USER CONSTRAINT
"we will keep it at 10TPS for definition, there is no why to increase it"

**Implication**: Phase 6 provides better architecture but won't increase TPS beyond 10.0. Benefits:
- Better CPU utilization (lower % usage for same TPS)
- Smoother execution (less variance)
- Preparation for future scaling

### Implementation Steps
1. Define SystemSet enum
2. Categorize existing systems
3. Add system ordering with .in_set() and .after()
4. Test execution order correctness
5. Enable parallel queries where safe
6. Validate 10 TPS maintained (not exceeded)

---

## Implementation Timeline

### Immediate (Phase 2)
**Duration**: 3-4 hours
**Risk**: Low
**Deploy**: infrastructure-implementation-agent + testing-implementation-agent

### Short-term (Phase 2 + 3)
**Duration**: 6-8 hours total
**Risk**: Low
**Deploy**: Batch both phases together (similar patterns)

### Medium-term (Phase 5)
**Duration**: 2-3 hours
**Risk**: Low-Medium
**Deploy**: After Phase 2-3 complete

### Long-term (Phase 4 + 6)
**Duration**: 10-13 hours
**Risk**: Phase 4 HIGH (spatial conflicts), Phase 6 Medium
**Deploy**: After spatial/vegetation work stabilizes

---

## Success Metrics

### Must Have (Every Phase)
- ✅ All tests passing (unit + integration)
- ✅ 10.0 TPS maintained (not exceeded per user constraint)
- ✅ Tick times stable (~5ms average)
- ✅ No behavioral changes to simulation
- ✅ Release build successful

### Nice to Have
- 🎯 Reduced memory usage (fewer HashMap allocations)
- 🎯 Better code readability
- 🎯 Easier debugging (component inspector visibility)
- 🎯 Preparation for future parallelism (even if not used)

---

## Risk Mitigation

**Phase 2-3**: Low risk, similar to Phase 1 (proven pattern)
**Phase 4**: HIGH RISK - defer until spatial work complete
**Phase 5**: Medium risk - test event ordering carefully
**Phase 6**: Medium risk - validate system ordering doesn't break behavior

**General Strategy**:
1. TDD methodology for every phase (RED → GREEN → REFACTOR)
2. Performance validation after each phase (10 TPS maintained)
3. Incremental deployment (one phase at a time)
4. Rollback plan (git commits after each phase)

---

## Orchestration Plan

**Recommended Execution Order**:
1. ✅ Phase 1: Actions as Components (COMPLETE)
2. 🔜 Phase 2: PathResult as Component (NEXT - 3-4 hours)
3. 🔜 Phase 3: Movement State as Component (AFTER PHASE 2 - 3-4 hours)
4. 🔜 Phase 5: Event-Driven Communication (AFTER PHASE 3 - 2-3 hours)
5. ⏸️ Phase 4: Spatial Hierarchy (DEFER - conflicts with active work)
6. ⏸️ Phase 6: System Sets (DEFER - optimization phase, not urgent)

**Agent Deployment Strategy**:
- **Phase 2**: infrastructure-implementation-agent → testing-implementation-agent → tdd-validation-agent
- **Phase 3**: infrastructure-implementation-agent → testing-implementation-agent → tdd-validation-agent
- **Phase 5**: feature-implementation-agent → testing-implementation-agent → tdd-validation-agent
- **Phase 4**: (when ready) infrastructure-implementation-agent → testing-implementation-agent → enhanced-quality-gate
- **Phase 6**: infrastructure-implementation-agent → testing-implementation-agent → enhanced-quality-gate

---

## Current Status Summary

**Completed**: Phase 1 ✅
**Active**: None (awaiting orchestration)
**Planned**: Phases 2, 3, 5 (in order)
**Deferred**: Phases 4, 6 (until spatial work stabilizes)

**Performance Baseline**: 10.0 TPS sustained, 5.2ms tick times

**Next Action**: Deploy Phase 2 implementation agents

---

## References

**Documentation**:
- `ECS_IMPROVEMENT_ULTRATHINK.md` - Original analysis
- `PHASE1_PERFORMANCE_VALIDATION.md` - Phase 1 validation
- `PATHFINDING_QUEUE_SUCCESS_REPORT.md` - Pathfinding queue baseline

**Related Active Work** (avoid conflicts):
- `SPATIAL_GRID_INTEGRATION.md`
- `SPATIAL_INDEX_FEAR_INTEGRATION.md`
- `SPATIAL_MATE_MATCHING_DELIVERY.md`
- `VEGETATION_GRID_DELIVERY.md`

**Code References**:
- `src/ai/queue.rs` - ActionQueue (Phase 1 modified)
- `src/pathfinding/pathfinding_queue.rs` - PathfindingQueue (Phase 2 target)
- `src/ai/action.rs` - Action state machines (Phase 3 target)
- `src/entities/spatial_index.rs` - SpatialEntityIndex (Phase 4 target)

---

**Last Updated**: 2025-12-26
**Status**: Phase 1 Complete, Ready for Phase 2 Deployment
**Constraint**: Maintain 10 TPS (not to be exceeded)
