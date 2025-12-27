# Bevy 0.16 Action Systems Research - World Access Patterns

**Date**: 2025-12-27
**Research Agent**: research-agent (Context7 + Web Research)
**Purpose**: Phase 1 - World Access Elimination Implementation Guidance

---

## Executive Summary

**Key Findings:**

1. **Exclusive World Access Blocks Parallelism**: `&mut World` usage prevents Bevy from running systems in parallel, significantly impacting performance.

2. **Commands Are Deferred**: `Commands` queue mutations for later application at `ApplyDeferred` points, enabling parallelism but not executing immediately.

3. **Use Exclusive Systems Sparingly**: Only justified for bulk operations (mass spawning/despawning) or when requirements are truly unpredictable.

4. **SystemParam Pattern is Superior**: Using proper system parameters (`Query`, `Commands`, `Res`, `ResMut`) allows parallel execution.

5. **ActionQueue Should Refactor**: Convert from `&mut World` to parameterized systems to unlock multi-core parallelism.

---

## Current Anti-Pattern (Our Code)

```rust
pub trait Action: Send + Sync {
    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult;
}

// ❌ This blocks ALL parallelism!
impl ActionQueue {
    pub fn execute_active_actions(&mut self, world: &mut World, tick: u64) {
        // Exclusive World access - no other systems can run
        let mut query = world.query::<(Entity, &ActiveAction)>();
        for (entity, _) in query.iter(world) {
            let mut entity_ref = world.get_entity_mut(entity)?;
            // Manual entity manipulation
        }
    }
}
```

**Problems**:
- Blocks ALL parallel execution
- Manual entity iteration
- 46 calls to `get_entity_mut()` across 3 files
- Prevents Bevy from scheduling work efficiently

---

## Recommended Pattern

```rust
// ✅ System-based execution (parallelizable)
fn execute_active_actions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ActiveAction)>,
    tick: Res<SimulationTick>,
    mut hunger: Query<&mut Hunger>,
    mut thirst: Query<&mut Thirst>,
    mut energy: Query<&mut Energy>,
    positions: Query<&TilePosition>,
    world_loader: Res<WorldLoader>,
) {
    // Can run in parallel with non-conflicting systems!
    for (entity, mut active) in &mut query {
        // Execute action using system parameters
    }
}
```

**Benefits**:
- Bevy can schedule this in parallel with other systems
- Multi-core CPU utilization
- Type-safe component access
- Automatic change detection

---

## Performance Impact

**From Bevy Documentation**:
> "Exclusive systems halt the entire schedule, eliminating parallelism."

**Expected Improvements**:
- **Multi-core systems**: 2-4x throughput improvement
- **Large entity counts** (1000+ entities): Most significant gains
- **CPU-bound scenarios**: Major improvement with parallel execution

**Our Scenario**:
- 500 entities with actions
- Multiple systems could run in parallel
- Currently all blocked by `&mut World` in ActionQueue

---

## Implementation Roadmap

### Phase 1: Interim Solution (SystemState)

Use `SystemState` to manage parameters while keeping trait signature:

```rust
use bevy::ecs::system::SystemState;

pub struct ActionQueue {
    execution_state: Option<SystemState<(
        Commands<'static, 'static>,
        Query<'static, 'static, &'static mut Hunger>,
        Query<'static, 'static, &'static mut TilePosition>,
        Res<'static, WorldLoader>,
    )>>,
}

impl ActionQueue {
    pub fn execute_active_actions(&mut self, world: &mut World, tick: u64) {
        // Initialize SystemState on first use
        if self.execution_state.is_none() {
            self.execution_state = Some(SystemState::new(world));
        }

        // Get mutable access to system parameters
        let state = self.execution_state.as_mut().unwrap();
        let (mut commands, mut hunger, mut positions, loader) = state.get_mut(world);

        // Use parameters instead of raw World access
        for entity in entities_to_execute {
            if let Ok(mut h) = hunger.get_mut(entity) {
                h.current -= 10.0;
            }
        }

        // CRITICAL: Apply deferred commands
        state.apply(world);
    }
}
```

**Pros**:
- Minimal API changes
- Can refactor incrementally
- Still uses SystemState benefits

**Cons**:
- Still blocks parallelism (function uses `&mut World`)
- Interim solution only

### Phase 2: Full System Conversion (Recommended)

Convert ActionQueue to proper Bevy systems:

```rust
// Action trait uses read-only World
pub trait Action: Send + Sync {
    fn execute(&mut self, entity: Entity, world: &World) -> ActionResult;
}

// Proper Bevy system
fn execute_active_actions_system(
    mut commands: Commands,
    mut action_query: Query<(Entity, &mut ActiveAction)>,
    mut hunger: Query<&mut Hunger>,
    mut thirst: Query<&mut Thirst>,
    tick: Res<SimulationTick>,
    world_loader: Res<WorldLoader>,
) {
    for (entity, mut active_action) in &mut action_query {
        // Access read-only world for validation
        let result = active_action.action.execute(entity, world.as_ref());

        // Use system parameters for mutations
        match result {
            ActionResult::Complete => {
                commands.entity(entity).remove::<ActiveAction>();
            }
            ActionResult::InProgress => {
                // Continue next tick
            }
            ActionResult::Failed => {
                commands.entity(entity).remove::<ActiveAction>();
            }
        }
    }
}

// Register in plugin
app.add_systems(
    FixedUpdate,
    execute_active_actions_system
        .in_set(SimulationSet::ActionExecution)
        .after(SimulationSet::Planning)
);
```

**Pros**:
- Full parallelism benefits
- Proper Bevy integration
- Type-safe component access
- Automatic scheduling

**Cons**:
- Larger refactor
- Need to update all Action implementations
- Need to pass more context to actions

---

## Commands vs Query Mutations

### Use Commands For:
- Spawning/despawning entities
- Adding/removing components
- Structural changes

```rust
commands.entity(entity).despawn();
commands.entity(entity).insert(NewComponent);
commands.entity(entity).remove::<OldComponent>();
```

### Use Query Mutations For:
- Modifying component values
- Updates that need immediate visibility
- Parallelizable work with `par_iter_mut()`

```rust
for mut health in &mut health_query {
    health.current -= damage;
}
```

### Critical Gotcha: Commands Are Deferred!

```rust
fn broken_system(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }

    // ❌ This will STILL iterate - commands not applied yet!
    for entity in &query {
        println!("Still exists: {:?}", entity);
    }
}
```

Commands apply at `ApplyDeferred` points in the schedule, not immediately!

---

## Parallel Execution Examples

### Can Run in Parallel (No Data Conflicts):

```rust
// System 1: Modifies Transform
fn move_system(mut query: Query<(&mut Transform, &Velocity)>) {
    query.par_iter_mut().for_each(|(mut transform, velocity)| {
        transform.translation += velocity.extend(0.0);
    });
}

// System 2: Modifies Health
fn damage_system(mut query: Query<(&mut Health, &Damage)>) {
    for (mut health, damage) in &mut query {
        health.current -= damage.amount;
    }
}

// Bevy can run these simultaneously on different cores!
```

### Cannot Run in Parallel (Conflicting Data):

```rust
// System 1: Mutably accesses Health
fn healing_system(mut query: Query<&mut Health>) {
    for mut health in &mut query {
        health.current += 10.0;
    }
}

// System 2: ALSO mutably accesses Health
fn poison_system(mut query: Query<&mut Health>) {
    for mut health in &mut query {
        health.current -= 5.0;
    }
}

// ❌ Bevy must run these sequentially (data conflict)
```

---

## Action Validation Pattern

Separate read-only validation from execution:

```rust
#[derive(SystemParam)]
struct ActionValidator<'w, 's> {
    positions: Query<'w, 's, &'static TilePosition>,
    hunger: Query<'w, 's, &'static Hunger>,
    world_loader: Res<'w, WorldLoader>,
}

impl ActionValidator<'_, '_> {
    fn can_execute(&self, entity: Entity, action_type: &ActionType) -> bool {
        // Read-only validation - fully parallelizable!
        match action_type {
            ActionType::Eat => {
                self.hunger.get(entity).map(|h| h.current < 50.0).unwrap_or(false)
            }
            ActionType::Wander => true,
        }
    }
}

fn validate_pending_actions(
    validator: ActionValidator,
    mut query: Query<(Entity, &mut PendingAction)>,
) {
    // Fully parallelizable validation!
    query.par_iter_mut().for_each(|(entity, mut pending)| {
        pending.valid = validator.can_execute(entity, &pending.action_type);
    });
}
```

---

## Read-Only World Access

When Action trait needs world context but shouldn't mutate:

```rust
pub trait Action: Send + Sync {
    // ✅ Read-only World access
    fn execute(&mut self, entity: Entity, world: &World) -> ActionResult;
}

impl Action for WanderAction {
    fn execute(&mut self, entity: Entity, world: &World) -> ActionResult {
        // Can query read-only data
        let positions = world.query::<&TilePosition>();
        let my_pos = positions.get(world, entity).ok()?;

        // Can access resources read-only
        let world_loader = world.resource::<WorldLoader>();

        // ❌ Cannot mutate - must return instructions instead
        ActionResult::Complete
    }
}
```

---

## Research Sources

**Context7 (Official Bevy Docs)**:
- Bevy ECS Module (`/websites/rs_bevy`) - 189,332 code snippets
- System parameters documentation
- Commands API reference
- Parallel execution patterns
- Custom QueryData and SystemParam examples

**Web Research**:
- [Bevy Cheat Book - Exclusive Systems](https://bevy-cheatbook.github.io/programming/exclusive.html)
- [Bevy Cheat Book - Direct World Access](https://bevy-cheatbook.github.io/programming/world.html)
- [Tainted Coders - Exclusive Systems](https://taintedcoders.com/bevy/patterns/exclusive-systems)
- [GitHub Discussion - Turn-Based Patterns](https://github.com/bevyengine/bevy/discussions/3370)

---

## Recommended Implementation Steps

### Step 1: Update Action Trait
```rust
// Change signature from &mut World to &World
pub trait Action: Send + Sync {
    fn execute(&mut self, entity: Entity, world: &World) -> ActionResult;
}
```

### Step 2: Update All Action Implementations
- WanderAction
- DrinkWaterAction
- GrazeAction
- HuntAction
- FleeAction

Remove mutations, return instructions instead.

### Step 3: Convert execute_active_actions to System
```rust
fn execute_active_actions_system(
    mut commands: Commands,
    query: Query<(Entity, &mut ActiveAction)>,
    // Add all needed system parameters
) {
    // Use system parameters for mutations
}
```

### Step 4: Register System in Plugin
```rust
app.add_systems(
    FixedUpdate,
    execute_active_actions_system
        .in_set(SimulationSet::ActionExecution)
);
```

### Step 5: Remove &mut World from ActionQueue
- No longer needs execute_active_actions method
- System handles execution directly

### Step 6: Validate and Benchmark
- All tests passing
- Measure parallel execution improvement
- Verify 10 TPS maintained

---

## Expected Outcomes

**Before** (Current):
- ActionQueue blocks all parallelism
- Single-threaded action execution
- Manual entity manipulation
- 46 calls to `get_entity_mut()`

**After** (Refactored):
- Actions can run in parallel with other systems
- Multi-core CPU utilization
- Type-safe component access
- Proper Bevy scheduling

**Performance Gain**: 2-4x on multi-core systems (user has 10 TPS constraint, but better CPU utilization)

---

**Research Complete**: 2025-12-27
**Next Step**: Deploy infrastructure-implementation-agent for Action trait refactor
