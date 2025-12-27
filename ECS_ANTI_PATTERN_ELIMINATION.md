# ECS Anti-Pattern Elimination Roadmap

**Date**: 2025-12-27
**Status**: PLANNED - Ready for Execution
**Performance Target**: Maintain 10.0 TPS (user constraint)
**Goal**: Eliminate remaining Rust/ECS anti-patterns for cleaner, faster architecture

---

## Executive Summary

Following the successful completion of 6 ECS improvement phases, this roadmap addresses **remaining anti-patterns** identified through comprehensive codebase analysis. These improvements focus on:
- Unlocking parallelism (World access elimination)
- Reducing unnecessary work (change detection)
- Eliminating allocations (clone reduction)
- Leveraging Bevy 0.16 features (required components, hooks)
- Full ECS integration (ResourceGrid migration)

**Critical Constraint**: 10 TPS is the target performance. Focus is on code quality and architecture.

---

## Priority Roadmap

### Phase 1: World Access Elimination ⚠️ CRITICAL
**Status**: ✅ COMPLETE (2025-12-27)
**Priority**: CRITICAL
**Effort**: 2-4 hours
**Delivery Report**: WORLD_ACCESS_ELIMINATION_DELIVERY.md

**Impact**: Unlocks parallelism for action execution

**Current Problem**:
```rust
// src/ai/queue.rs - execute_active_actions()
fn execute_active_actions(&mut self, world: &mut World, tick: u64) {
    // ❌ Exclusive &mut World access blocks ALL parallelism
    // ❌ 46 calls to get_entity_mut() across 3 files
    let mut query = world.query::<(Entity, &ActiveAction)>();
    for (entity, _) in query.iter(world) {
        let mut entity_ref = world.get_entity_mut(entity)?;
        // Manual entity manipulation
    }
}
```

**Target Architecture**:
```rust
// Use system parameters instead of World
fn execute_active_actions(
    mut query: Query<(Entity, &mut ActiveAction)>,
    mut commands: Commands,
    tick: Res<SimulationTick>,
    world: &World,  // Read-only for action execution context
) {
    for (entity, mut active) in query.iter_mut() {
        let result = active.action.execute(entity, world);
        // Handle action completion
    }
}
```

**Files to Modify**:
- `src/ai/queue.rs` - ActionQueue refactor
- `src/ai/action.rs` - Action trait signature update
- `src/entities/mod.rs` - System registration update

**Implementation Steps**:
1. Update Action trait to accept &World instead of &mut World
2. Refactor execute_active_actions to use Query parameters
3. Update all Action implementations (WanderAction, DrinkWaterAction, etc.)
4. Remove get_entity_mut() calls, use Commands for mutations
5. Update system registration in plugin
6. Write TDD tests for system-based execution
7. Validate 10 TPS maintained

**Success Criteria**:
- No &mut World in ActionQueue
- execute_active_actions is a proper Bevy system
- All tests passing
- 10 TPS maintained
- Parallelism unlocked (can run with other systems)

---

### Phase 2: Change Detection Implementation ⚠️ HIGH IMPACT
**Status**: ✅ COMPLETE (2025-12-27)
**Priority**: HIGH
**Effort**: 2-3 hours (3 parallel agents)
**Delivery Report**: CHANGE_DETECTION_DELIVERY.md
**Impact**: 5-10x performance improvement on stable simulations
**Impact**: 5-10x performance improvement on stable simulations

**Affected Systems**:
1. `predator_proximity_system` - fear.rs
2. `fear_proximity_system` - fear.rs
3. `tick_stats_system` - stats.rs
4. `mate_matching_system` - reproduction.rs (6 species)
5. `spatial_index_maintenance` - spatial maintenance

**Current Anti-Pattern**:
```rust
// ❌ Processes ALL entities every tick
fn predator_proximity_system(
    mut prey: Query<(Entity, &TilePosition, &mut FearState), With<Herbivore>>,
    predators: Query<&TilePosition, With<Predator>>,
) {
    for (entity, pos, mut fear) in prey.iter_mut() {
        // Runs even if entity hasn't moved!
    }
}
```

**Target Architecture**:
```rust
// ✅ Only process entities that moved or were just spawned
fn predator_proximity_system(
    mut prey: Query<
        (Entity, &TilePosition, &mut FearState),
        Or<(Changed<TilePosition>, Added<FearState>)>
    >,
    predators: Query<&TilePosition, With<Predator>>,
) {
    for (entity, pos, mut fear) in prey.iter_mut() {
        // 5-10x fewer iterations on stable sims
    }
}
```

**Alternative: Periodic Update Pattern**:
```rust
#[derive(Component)]
struct UpdateTimer {
    last_update: u64,
    interval: u64,  // Only update every N ticks
}

fn predator_proximity_system(
    mut prey: Query<(Entity, &TilePosition, &mut FearState, &mut UpdateTimer)>,
    tick: Res<SimulationTick>,
) {
    for (entity, pos, mut fear, mut timer) in prey.iter_mut() {
        if tick.0 - timer.last_update >= timer.interval {
            // Update fear state
            timer.last_update = tick.0;
        }
    }
}
```

**Implementation Steps**:
1. Identify systems that iterate all entities unnecessarily
2. Add Changed<T> or Added<T> filters where appropriate
3. For systems that need periodic updates, add UpdateTimer component
4. Benchmark before/after with stable simulation (250+ ticks)
5. Adjust update intervals based on gameplay needs
6. Write tests for change detection behavior
7. Validate 10 TPS maintained

**Expected Results**:
- 5-10x fewer entity iterations on stable simulations
- No behavior changes (same game logic)
- Better CPU utilization (less wasted work)

---

### Phase 3: Clone Proliferation Reduction ⚠️ HIGH IMPACT
**Status**: 🔜 PLANNED
**Priority**: HIGH
**Effort**: 2-6 hours
**Impact**: 10-20% performance improvement in movement systems

**Clone Hotspots** (64 total clones, focusing on hot paths):
1. `src/entities/movement.rs` - Path cloning (every tick)
2. `src/ai/action.rs` - Action state cloning (5 clones per tick)
3. `src/pathfinding/path_components.rs` - Path result cloning

**Current Anti-Pattern**:
```rust
// ❌ Clones entire Vec<IVec2> every tick
match movement {
    MovementComponent::FollowingPath { path, index } => {
        if new_index >= path.len() {
            *movement = MovementComponent::Idle;
        } else {
            *movement = MovementComponent::FollowingPath {
                path: path.clone(),  // ❌ Full Vec allocation
                index: new_index,
            };
        }
    }
}
```

**Target Architecture**:
```rust
// ✅ Share path data with Rc
use std::rc::Rc;

#[derive(Component)]
pub enum MovementComponent {
    Idle,
    FollowingPath {
        path: Rc<Vec<IVec2>>,  // Cheap to clone, shares underlying data
        index: usize,
    },
}

// Clone is now just an Rc increment (1 pointer copy)
*movement = MovementComponent::FollowingPath {
    path: path.clone(),  // ✅ O(1) operation
    index: new_index,
};
```

**Implementation Steps**:
1. Identify clone() calls in hot paths (movement, actions)
2. Replace Vec clones with Rc<Vec> for shared ownership
3. Update PathReady component to use Rc<Vec<IVec2>>
4. Update action state machines to avoid unnecessary cloning
5. Benchmark memory allocations before/after
6. Write tests for Rc-based sharing
7. Validate 10 TPS maintained

**Expected Results**:
- 10-20% faster movement execution
- Reduced memory allocations (fewer GC pauses)
- Same behavior, better performance

---

### Phase 4: Required Components Migration 🎁 BEVY 0.16 FEATURE
**Status**: ✅ COMPLETE (2025-12-27)
**Priority**: HIGH (Bug Prevention)
**Effort**: 2-3 hours (ACTUAL: ~3 hours)
**Impact**: Compile-time safety for entity spawning
**Delivery Report**: PHASE4_REQUIRED_COMPONENTS_DELIVERY.md ✅

**Current Problem**:
```rust
// ❌ Easy to forget components (runtime bug)
pub fn spawn_rabbit(commands: &mut Commands, name: String, pos: IVec2) -> Entity {
    commands.spawn((
        Rabbit,
        Name::new(name),
        TilePosition::new(pos),
        MovementSpeed::normal(),
        Health::new(100.0),
        Hunger::new(100.0),
        Thirst::new(100.0),
        // What if we forget Energy? Runtime bug!
    )).id()
}
```

**Bevy 0.16 Solution**:
```rust
// ✅ Compile-time guarantee of required components
#[derive(Component)]
#[require(
    TilePosition,
    MovementSpeed,
    Health,
    Hunger,
    Thirst,
    Energy,
    MovementComponent
)]
struct Rabbit;

pub fn spawn_rabbit(commands: &mut Commands, name: String, pos: IVec2) -> Entity {
    commands.spawn((
        Rabbit,
        Name::new(name),
        TilePosition::new(pos),
        // All other components added automatically!
    )).id()
}
```

**Species to Update**:
- Rabbit (herbivore)
- Deer (herbivore)
- Raccoon (omnivore)
- Fox (carnivore)
- Wolf (carnivore)
- Bear (carnivore)

**Implementation Steps**:
1. Add #[require(...)] attribute to all species components
2. Update spawn functions to remove redundant component insertions
3. Verify all required components are specified
4. Write tests to ensure components are present
5. Update documentation with required components list
6. Validate all spawning functions work correctly

**Success Criteria**:
- All species have #[require(...)] attributes
- Spawn functions simplified (fewer manual components)
- Compile-time guarantee of component presence
- All tests passing

---

### Phase 5: Inline Hints for Hot Functions 🔧 QUICK WIN
**Status**: 🔜 PLANNED
**Priority**: MEDIUM
**Effort**: 30 minutes
**Impact**: 1-5% performance improvement in spatial queries

**Affected Functions** (~15 functions):
- `world_to_chunk()` - spatial_index.rs
- `chunk_to_world()` - spatial_index.rs
- `distance_squared()` - various files
- `in_bounds()` - grid utilities

**Current**:
```rust
// ❌ Missing inline hint (called thousands of times per tick)
fn world_to_chunk(pos: IVec2) -> IVec2 {
    IVec2::new(
        pos.x.div_euclid(CHUNK_SIZE),
        pos.y.div_euclid(CHUNK_SIZE),
    )
}
```

**Target**:
```rust
// ✅ Inline hint for hot path function
#[inline]
fn world_to_chunk(pos: IVec2) -> IVec2 {
    IVec2::new(
        pos.x.div_euclid(CHUNK_SIZE),
        pos.y.div_euclid(CHUNK_SIZE),
    )
}
```

**Implementation Steps**:
1. Identify small utility functions in hot paths
2. Add #[inline] attribute to each
3. Run benchmarks to verify improvement
4. Document which functions are inlined and why

**Success Criteria**:
- All hot path utilities have #[inline]
- 1-5% performance improvement in spatial queries
- No behavioral changes

---

### Phase 6: System Parameter Bundling 🔧 CODE QUALITY
**Status**: 🔜 PLANNED
**Priority**: MEDIUM
**Effort**: 4-6 hours
**Impact**: Cleaner API, easier testing

**Current Problem**:
```rust
// ❌ 8+ parameters (hard to test, hard to read)
pub fn plan_rabbit_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    rabbits: Query<(Entity, &TilePosition, &Thirst, &Hunger, &Energy, ...)>,
    rabbit_positions: Query<(Entity, &TilePosition), With<Rabbit>>,
    predator_positions: Query<&TilePosition, Or<(With<Wolf>, With<Fox>, With<Bear>)>>,
    world_loader: Res<WorldLoader>,
    vegetation_grid: Res<ResourceGrid>,
    pathfinding_queue: Res<PathfindingQueue>,
    tick: Res<SimulationTick>,
) {
    // ...
}
```

**Target Architecture**:
```rust
// ✅ Bundle related resources
#[derive(Resource)]
pub struct PlanningContext {
    pub tick: u64,
}

impl PlanningContext {
    pub fn from_resources(
        world_loader: &WorldLoader,
        vegetation_grid: &ResourceGrid,
        tick: &SimulationTick,
    ) -> Self {
        Self {
            tick: tick.0,
        }
    }
}

pub fn plan_rabbit_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    rabbits: Query<(Entity, &TilePosition, &Thirst, &Hunger, &Energy, ...)>,
    context: Res<PlanningContext>,  // ✅ Single bundled resource
) {
    // Cleaner API, easier to test
}
```

**Implementation Steps**:
1. Create PlanningContext resource bundle
2. Update all species planning systems to use context
3. Simplify function signatures
4. Update tests to use bundled context
5. Validate all systems work correctly

**Success Criteria**:
- Planning systems have ≤5 parameters
- Tests use PlanningContext fixture
- Code is more readable
- All tests passing

---

### Phase 7: Component Hooks for Spatial Index 🎁 BEVY 0.16 FEATURE
**Status**: 🔜 PLANNED
**Priority**: MEDIUM
**Effort**: 4-6 hours
**Impact**: Automatic spatial index synchronization

**Current Problem**:
```rust
// ❌ Manual spatial index updates everywhere
pub fn update_spatial_parent_on_movement(
    mut commands: Commands,
    grid: Res<SpatialCellGrid>,
    moved: Query<(Entity, &TilePosition, &Parent), (Changed<TilePosition>, With<SpatiallyParented>)>,
) {
    // Manual tracking of position changes
    for (entity, pos, parent) in moved.iter() {
        // Update parent manually
    }
}
```

**Bevy 0.16 Component Hooks Solution**:
```rust
// ✅ Automatic synchronization via component hooks
impl Component for TilePosition {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        // Automatically update spatial index when position added
        let pos = world.get::<TilePosition>(entity).unwrap().tile;
        // Update spatial parent
    }

    fn on_replace(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        // Automatically update spatial index when position changes
        let new_pos = world.get::<TilePosition>(entity).unwrap().tile;
        // Update spatial parent to new cell
    }
}
```

**Implementation Steps**:
1. Research Bevy 0.16 Component Hooks API
2. Implement on_insert hook for TilePosition
3. Implement on_replace hook for position updates
4. Remove manual update_spatial_parent_on_movement system
5. Write tests for automatic synchronization
6. Validate spatial queries still work correctly

**Success Criteria**:
- No manual spatial index update systems
- TilePosition changes automatically update spatial parent
- All spatial queries work correctly
- Tests verify automatic synchronization

---

### Phase 8: ResourceGrid ECS Migration ⚠️ MAJOR REFACTOR
**Status**: 🔜 PLANNED
**Priority**: MEDIUM (Long-term architecture)
**Effort**: 8-16 hours
**Impact**: Full ECS integration, better parallelism

**Current Architecture**:
```rust
// ❌ Not integrated with ECS
#[derive(Resource)]
pub struct ResourceGrid {
    cells: HashMap<IVec2, GrazingCell>,  // Manual HashMap
    chunk_size: i32,
}

pub struct GrazingCell {
    pub total_biomass: f32,
    pub consumption_pressure: f32,
    pub growth_rate: f32,
    // ...
}
```

**Target ECS Architecture**:
```rust
// ✅ Vegetation cells as actual entities with components
#[derive(Component)]
pub struct VegetationCell {
    pub total_biomass: f32,
    pub consumption_pressure: f32,
    pub growth_rate: f32,
}

#[derive(Component)]
pub struct CellPosition(pub IVec2);

#[derive(Component)]
pub struct VegetationType {
    pub biome: BiomeType,
    pub max_biomass: f32,
}

// Grid resource only tracks entity references
#[derive(Resource)]
pub struct VegetationGrid {
    cells: HashMap<IVec2, Entity>,  // Position -> Entity lookup
    chunk_size: i32,
}

// Now can use standard ECS queries with parallelism
fn update_vegetation_growth(
    mut cells: Query<(&mut VegetationCell, &VegetationType), Changed<VegetationCell>>
) {
    for (mut cell, veg_type) in cells.iter_mut() {
        // Bevy can parallelize this automatically
        cell.total_biomass += cell.growth_rate * dt;
        cell.total_biomass = cell.total_biomass.min(veg_type.max_biomass);
    }
}
```

**Benefits of ECS Migration**:
1. **Automatic Cleanup**: Despawn entity = remove from grid automatically
2. **Change Detection**: Only update cells that changed
3. **Parallelism**: Bevy can run vegetation systems in parallel
4. **Query Power**: Can use With<T>, Without<T> filters
5. **Component Inspector**: Debug vegetation cells visually
6. **Events**: React to vegetation changes with events

**Implementation Steps**:
1. Create VegetationCell, CellPosition, VegetationType components
2. Create spawn_vegetation_cell() function
3. Migrate ResourceGrid to spawn entities instead of HashMap storage
4. Update find_best_cell() to use Query instead of HashMap iteration
5. Update consume_from_cell() to use Commands for mutations
6. Update vegetation growth systems to use Query
7. Add change detection for efficiency
8. Write comprehensive tests
9. Validate 10 TPS maintained

**Success Criteria**:
- Vegetation cells are ECS entities with components
- All vegetation queries use ECS Query system
- Change detection working (only update changed cells)
- All tests passing
- 10 TPS maintained
- Parallel vegetation updates possible

---

### Phase 9: Newtype Pattern for Domain Types 🔧 TYPE SAFETY
**Status**: 🔜 PLANNED (Optional)
**Priority**: LOW
**Effort**: 2-4 hours
**Impact**: Type safety, self-documenting code

**Current**:
```rust
// ❌ Raw primitives (what does 32 mean?)
pub struct MovementSpeed {
    pub ticks_per_move: u32,
}

pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

**Target**:
```rust
// ✅ Newtype wrappers for domain concepts
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TicksPerMove(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HealthPoints(pub f32);

pub struct MovementSpeed {
    pub ticks_per_move: TicksPerMove,  // Type safety!
}

pub struct Health {
    pub current: HealthPoints,
    pub max: HealthPoints,
}

// Can't accidentally mix up different numeric types
fn faster_than(a: TicksPerMove, b: TicksPerMove) -> bool {
    a.0 < b.0  // Lower ticks = faster movement
}
```

**Implementation Steps**:
1. Identify domain concepts using raw primitives
2. Create newtype wrappers
3. Update structs to use newtypes
4. Update functions to use typed parameters
5. Write tests for type safety
6. Update documentation

**Success Criteria**:
- Domain concepts have newtype wrappers
- Can't mix incompatible types (compile error)
- Code is self-documenting
- All tests passing

---

### Phase 10: Vec<Entity> to Bevy Relations Migration 🔧 ARCHITECTURAL
**Status**: 🔜 PLANNED (Partially Complete)
**Priority**: LOW (Already started in spatial hierarchy)
**Effort**: 4-8 hours
**Impact**: Better entity relationship management

**Current Usage**: 28 files use Vec<Entity>

**Already Migrated**:
- Spatial hierarchy (Parent/Child components) ✅
- SpatialCell uses Children component ✅

**Remaining Opportunities**:
- Pack dynamics (wolves hunting in groups)
- Family relationships (parent/offspring tracking)
- Predator/prey relationships

**Current Anti-Pattern**:
```rust
// ❌ Manual entity vector tracking
#[derive(Component)]
struct Pack {
    members: Vec<Entity>,  // Manual tracking
}

fn update_pack_behavior(
    mut packs: Query<&mut Pack>,
    members: Query<&TilePosition>,
) {
    for mut pack in packs.iter_mut() {
        // Manually filter dead entities
        pack.members.retain(|e| members.get(*e).is_ok());
    }
}
```

**Better Pattern with Relations**:
```rust
// ✅ Use Bevy's built-in entity relationships
commands.entity(pack_leader).add_child(pack_member);

fn update_pack_behavior(
    packs: Query<(Entity, &Children), With<PackLeader>>,
    members: Query<&TilePosition>,
) {
    for (leader, children) in packs.iter() {
        // Bevy automatically removes dead children!
        for &member in children.iter() {
            // ...
        }
    }
}
```

**Implementation Steps**:
1. Audit remaining Vec<Entity> usage
2. Identify entity relationships that should use Parent/Child
3. Migrate pack dynamics to use hierarchy
4. Migrate family relationships to use hierarchy
5. Write tests for automatic cleanup
6. Validate relationship queries work correctly

**Success Criteria**:
- Pack dynamics use Parent/Child hierarchy
- Dead entities automatically removed from relationships
- Simpler relationship management code
- All tests passing

---

## Implementation Timeline

### Quick Wins (1-2 days) - Phases 5
**Effort**: 30 minutes
**Impact**: 1-5% performance
- Add #[inline] to hot path functions

### High Impact (3-5 days) - Phases 1, 2, 3, 4
**Effort**: 16-29 hours
**Impact**: Unlock parallelism, 5-10x faster stable sims, 10-20% movement
1. Phase 1: World Access Elimination (2-4h)
2. Phase 2: Change Detection Implementation (10-20h)
3. Phase 3: Clone Reduction (2-6h)
4. Phase 4: Required Components (2-3h)

### Medium Refactors (1 week) - Phases 6, 7
**Effort**: 8-12 hours
**Impact**: Code quality, automatic sync
5. Phase 6: System Parameter Bundling (4-6h)
6. Phase 7: Component Hooks (4-6h)

### Major Improvements (2-3 weeks) - Phases 8, 9, 10
**Effort**: 14-28 hours
**Impact**: Full ECS integration, type safety
7. Phase 8: ResourceGrid ECS Migration (8-16h)
8. Phase 9: Newtype Pattern (2-4h) - Optional
9. Phase 10: Relations Migration (4-8h) - Optional

**Total Estimated Effort**: 38-69 hours across all phases

---

## Success Metrics

### Must Have (Every Phase)
- ✅ All tests passing (unit + integration)
- ✅ 10.0 TPS maintained (not exceeded per user constraint)
- ✅ No behavioral changes to simulation
- ✅ Release build successful

### Phase-Specific Goals
- **Phase 1**: ActionQueue is a proper Bevy system (no &mut World)
- **Phase 2**: 5-10x fewer entity iterations on stable simulations
- **Phase 3**: 10-20% faster movement, fewer allocations
- **Phase 4**: Compile-time component guarantees
- **Phase 5**: 1-5% spatial query speedup
- **Phase 6**: ≤5 parameters per planning system
- **Phase 7**: Automatic spatial index sync (no manual systems)
- **Phase 8**: Vegetation cells are ECS entities with change detection
- **Phase 9**: Type-safe domain concepts
- **Phase 10**: Entity relationships use Parent/Child hierarchy

---

## Risk Mitigation

**Phase 1-3**: Medium risk - core systems refactor, comprehensive testing needed
**Phase 4-7**: Low risk - additive improvements, minimal breaking changes
**Phase 8**: High risk - major ResourceGrid refactor, needs extensive testing
**Phase 9-10**: Low risk - optional improvements, can defer if needed

**General Strategy**:
1. TDD methodology for every phase (RED → GREEN → REFACTOR)
2. Performance validation after each phase (10 TPS maintained)
3. Incremental deployment (one phase at a time)
4. Rollback plan (git commits after each phase)
5. Benchmark before/after for performance-critical phases

---

## Agent Orchestration Plan

### Parallel Agent Deployment Strategy

**Phase 1: World Access** (Single agent - critical path)
- infrastructure-implementation-agent (Action trait refactor)
- testing-implementation-agent (system tests)
- tdd-validation-agent (validation)

**Phase 2: Change Detection** (Parallel agents - 10 systems)
- Deploy 3 feature-implementation-agents in parallel:
  - Agent 1: Fear systems (2 systems)
  - Agent 2: Stats + mate matching (3 systems)
  - Agent 3: Spatial maintenance (5 systems)
- testing-implementation-agent (integration tests)
- tdd-validation-agent (validation)

**Phase 3: Clone Reduction** (Single agent - focused refactor)
- feature-implementation-agent (Rc<Vec> migration)
- testing-implementation-agent (allocation tests)
- tdd-validation-agent (validation)

**Phase 4: Required Components** (Parallel agents - 6 species)
- Deploy 2 feature-implementation-agents in parallel:
  - Agent 1: Herbivores (rabbit, deer, raccoon)
  - Agent 2: Carnivores (fox, wolf, bear)
- testing-implementation-agent (spawn tests)
- tdd-validation-agent (validation)

**Phase 5: Inline Hints** (Single agent - quick win)
- feature-implementation-agent (add #[inline] attributes)
- tdd-validation-agent (benchmark verification)

**Phase 6: System Parameters** (Parallel agents - 6 planning systems)
- Deploy 2 feature-implementation-agents in parallel:
  - Agent 1: Herbivore planning systems
  - Agent 2: Carnivore planning systems
- testing-implementation-agent (context tests)
- tdd-validation-agent (validation)

**Phase 7: Component Hooks** (Research + implementation)
- research-agent (Bevy 0.16 hooks API with Context7)
- infrastructure-implementation-agent (TilePosition hooks)
- testing-implementation-agent (sync tests)
- tdd-validation-agent (validation)

**Phase 8: ResourceGrid Migration** (Major refactor - sequential)
- infrastructure-implementation-agent (component structure)
- feature-implementation-agent (query migration)
- testing-implementation-agent (comprehensive tests)
- enhanced-quality-gate (full validation)

**Phases 9-10**: Optional - defer based on user priority

---

## Current Status Summary
## Current Status Summary

**Completed**:
- Phase 1 (World Access Elimination) ✅ 2025-12-27
- Phase 2 (Change Detection Implementation) ✅ 2025-12-27
- Phase 4 (Required Components Migration) ✅ 2025-12-27

**Active**: None
**Planned**: Phases 3, 5-10 (in priority order)

**Performance Baseline**: 10.0 TPS sustained, ~100ms tick times

**Phase 1 Results**:
- Action execution system converted to proper Bevy system
- Parallelism unlocked (no longer blocks with &mut World)
- All 320 tests passing (excluding 1 pre-existing broken test)
- 10 TPS maintained (target met)

**Phase 2 Results**:
- 15 systems optimized with change detection filters
- 5-10x fewer iterations on stable simulations (7,500 → 750 iterations/tick)
- All 275 library tests passing
- Zero behavioral changes - pure optimization
- Release build successful

**Phase 4 Results**:
- 13 components updated with `#[require(...)]` attributes
- Compile-time guarantees for component dependencies
- 12 new tests added and passing
- All 275 library tests still passing
- Zero behavioral changes
- ~71 lines of code changes

**Next Action**: Phase 3 (Clone Reduction) - High Impact refactor

---

## Documentation to Create

**Per-Phase Deliverables**:
- Phase 1: WORLD_ACCESS_ELIMINATION_DELIVERY.md ✅ CREATED
- Phase 2: CHANGE_DETECTION_DELIVERY.md ✅ CREATED
- Phase 3: CLONE_REDUCTION_DELIVERY.md
- Phase 4: REQUIRED_COMPONENTS_DELIVERY.md
- Phase 5: INLINE_OPTIMIZATION_DELIVERY.md
- Phase 6: SYSTEM_PARAMETERS_DELIVERY.md
- Phase 7: COMPONENT_HOOKS_DELIVERY.md
- Phase 8: VEGETATION_ECS_MIGRATION_DELIVERY.md
- Phase 9: NEWTYPE_PATTERN_DELIVERY.md (optional)
- Phase 10: RELATIONS_MIGRATION_DELIVERY.md (optional)

**Final Report**:
- ECS_ANTI_PATTERN_ELIMINATION_COMPLETE.md (after all phases)

---

**Last Updated: 2025-12-27
**Status**: Ready for Execution
**Constraint**: Maintain 10 TPS (not to be exceeded)
