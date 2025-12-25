# HealthChecker Cleanup Fix - Implementation Reference

## Quick Reference

### Problem Statement
```
OLD BEHAVIOR: cleanup_old_states() → entity_states.clear()
Clears ALL entries every 50 ticks, forcing rebuild
Result: Complete state loss, memory churn, performance hit

NEW BEHAVIOR: cleanup_old_states(is_alive) → entity_states.retain(|id, _| is_alive(id))
Removes ONLY dead entities, preserves alive entity state
Result: Selective cleanup, state continuity, minimal overhead
```

## Implementation

### Method Signature
```rust
// File: src/debug/health_checks.rs, Lines 303-305

pub fn cleanup_old_states(&mut self, is_alive: impl Fn(u32) -> bool) {
    self.entity_states.retain(|entity_id, _| is_alive(*entity_id));
}
```

### System Integration
```rust
// File: src/debug/health_checks.rs, Lines 374-399

fn health_check_system(
    mut health_checker: ResMut<HealthChecker>,
    tick: Res<crate::simulation::SimulationTick>,
    metrics: Res<crate::simulation::TickMetrics>,
    entity_query: Query<Entity>,
) {
    let current_tick = tick.get();
    let tps = metrics.actual_tps();

    health_checker.check_tps(tps, current_tick);
    health_checker.check_stuck_entities(current_tick);
    health_checker.check_population_crash(current_tick);
    health_checker.check_ai_loops(current_tick);

    health_checker.reset_action_counters();

    // Collect alive entity IDs
    let alive_entities: std::collections::HashSet<u32> = entity_query
        .iter()
        .map(|entity| entity.index())
        .collect();

    // Clean only dead entities
    health_checker.cleanup_old_states(|id| alive_entities.contains(&id));

    // Rest of function...
}
```

## Data Structures

### EntityHealthState
```rust
pub struct EntityHealthState {
    pub last_position: (i32, i32),
    pub last_position_update_tick: u64,
    pub current_action: String,
    pub action_repeat_count: u32,
}
```

**What's Preserved**: All fields remain intact for alive entities

### HealthChecker Resource
```rust
pub struct HealthChecker {
    alerts: VecDeque<AlertRecord>,
    entity_states: std::collections::HashMap<u32, EntityHealthState>,
    population_history: VecDeque<(u64, u32)>,
}
```

**Key Field**: `entity_states` - HashMap of entity ID to health state

## Test Coverage

### Test 1: Selective Removal
```rust
test_cleanup_removes_dead_entities_only()
├─ Setup: 10 entities
├─ Alive: 1-7
├─ Dead: 8-10
├─ Cleanup: With alive predicate
└─ Assert: 7 remain, 3 removed
```

### Test 2: All Alive
```rust
test_cleanup_preserves_all_entities_if_all_alive()
├─ Setup: 5 entities
├─ Alive: All 5
├─ Cleanup: With alive predicate
└─ Assert: All 5 remain unchanged
```

### Test 3: All Dead
```rust
test_cleanup_removes_all_dead_entities()
├─ Setup: 5 entities
├─ Alive: None
├─ Cleanup: With empty alive set
└─ Assert: HashMap empty (len == 0)
```

### Test 4: Action State Preserved
```rust
test_cleanup_preserves_action_state_for_alive_entities()
├─ Setup: Entity 1 (15 "Attack" repeats), Entity 2 ("Move")
├─ Alive: [1]
├─ Cleanup: With alive predicate
├─ Assert: Entity 1 action_repeat_count == 15
├─ Assert: Entity 1 current_action == "Attack"
└─ Assert: Entity 2 removed
```

### Test 5: Position State Preserved
```rust
test_cleanup_preserves_position_state_for_alive_entities()
├─ Setup: 3 entities at (10,20)
├─ Store: Original positions
├─ Alive: [1, 2]
├─ Cleanup: With alive predicate
├─ Assert: Positions unchanged
└─ Assert: Entity 3 removed
```

## Call Flow

### Normal Operation (Every 50 Ticks)
```
Bevy Update Loop (Tick 50, 100, 150, ...)
    ↓
every_50_ticks condition → true
    ↓
health_check_system invoked
    ├─ check_tps()
    ├─ check_stuck_entities()
    ├─ check_population_crash()
    ├─ check_ai_loops()
    ├─ reset_action_counters()
    ├─ Query alive entities
    ├─ Build HashSet<u32> of alive IDs
    └─ cleanup_old_states(|id| alive_entities.contains(&id))
        └─ retain() filters out dead entities
```

### Data Flow During Cleanup
```
Before Cleanup:
entity_states HashMap
├─ Entity 1: (pos, tick, action, count)
├─ Entity 2: (pos, tick, action, count)
├─ Entity 3: (pos, tick, action, count)  ← Dead
└─ Entity 4: (pos, tick, action, count)  ← Dead

After Cleanup (is_alive checks [1, 2]):
entity_states HashMap
├─ Entity 1: (pos, tick, action, count)  [PRESERVED]
└─ Entity 2: (pos, tick, action, count)  [PRESERVED]

Removed:
Entity 3, Entity 4
```

## Performance Characteristics

### Time Complexity
```
HashMap::retain(predicate)
- Iterates all entries: O(n) where n = total entities tracked
- Predicate check: O(1) for HashSet lookup
- Overall: O(n) but with dead entities removed (m << n after cleanup)

vs Previous clear():
- HashMap::clear(): O(n) to deallocate all
- HashMap rebuild: O(n) to repopulate from scratch
```

### Space Complexity
```
Before Fix:
- entity_states: O(n) all entries
- Next cycle: O(0) entities lost, must rebuild all

After Fix:
- entity_states: O(m) alive entities only (m << n)
- Next cycle: O(m) preserved, no rebuild needed
- HashSet temporary: O(n) but freed after cleanup
```

## Integration Points

### Upstream: Entity Spawn/Death
```
Entity spawned
    ├─ AI system updates entity_position()
    ├─ AI system updates entity_action()
    └─ Health checker tracks in entity_states

Entity dies
    ├─ Entity component removed from ECS
    ├─ Health checker still has stale entry
    └─ cleanup_old_states() removes it at next 50-tick boundary
```

### Downstream: Alert Generation
```
Health checker runs checks (using entity_states)
    ├─ check_stuck_entities() - uses last_position_update_tick
    ├─ check_ai_loops() - uses action_repeat_count
    └─ These continue to work with preserved state
```

## Configuration

### Cleanup Frequency
```rust
// Run condition (Line 369)
fn every_50_ticks(tick: Res<crate::simulation::SimulationTick>) -> bool {
    tick.get() % 50 == 0
}
```
Cleanup runs every 50 simulation ticks (configurable via this function)

### State Capacity
```rust
// Default allocation in HealthChecker::default()
entity_states: std::collections::HashMap::new()
```
HashMap grows dynamically as needed

## Debugging

### Inspect Health States
```rust
let summary = health_checker.get_health_summary();
println!("entity_states_count: {}", summary["entity_states_count"]);
// Shows current number of tracked entities
```

### Monitor Cleanup Effect
Before fix: Every 50 ticks, entity_states_count drops to 0 then rebuilds
After fix: entity_states_count gradually increases only for new/dead entities

### Test Individually
```bash
cargo test --lib debug::health_checks::tests::test_cleanup_removes_dead_entities_only
cargo test --lib debug::health_checks::tests::test_cleanup_preserves_all_entities_if_all_alive
cargo test --lib debug::health_checks::tests::test_cleanup_removes_all_dead_entities
cargo test --lib debug::health_checks::tests::test_cleanup_preserves_action_state_for_alive_entities
cargo test --lib debug::health_checks::tests::test_cleanup_preserves_position_state_for_alive_entities
```

## Related Code

### Adjacent Methods
```rust
// Updates tracked state
pub fn update_entity_position(&mut self, entity_id: u32, position: (i32, i32), tick: u64)
pub fn update_entity_action(&mut self, entity_id: u32, action: String)

// Checks against tracked state
pub fn check_stuck_entities(&mut self, current_tick: u64) -> bool
pub fn check_ai_loops(&mut self, current_tick: u64) -> bool

// Resets state counters
pub fn reset_action_counters(&mut self)

// Cleanup (this fix)
pub fn cleanup_old_states(&mut self, is_alive: impl Fn(u32) -> bool)
```

### Related Structs
```rust
pub enum HealthAlert {
    TpsBelow10,
    EntitiesStuck,
    PopulationCrash,
    AiLoops,
}

#[derive(Resource, Debug)]
pub struct HealthChecker {
    alerts: VecDeque<AlertRecord>,
    entity_states: std::collections::HashMap<u32, EntityHealthState>,
    population_history: VecDeque<(u64, u32)>,
}
```

## Key Takeaways

1. **HashMap::retain()** is the right tool for this job
2. **Predicate function** provides flexibility and testability
3. **HashSet lookup** is O(1) and efficient for liveness check
4. **In-place filtering** avoids intermediate allocations
5. **TDD approach** ensures all scenarios are covered

## File Locations

- Implementation: `/Users/jean/Github/life-simulator/src/debug/health_checks.rs` (lines 303-305, 378, 393-399)
- Tests: `/Users/jean/Github/life-simulator/src/debug/health_checks.rs` (lines 671-792)
- Docs: `/Users/jean/Github/life-simulator/HEALTHCHECKER_CLEANUP_FIX.md`
- Summary: `/Users/jean/Github/life-simulator/HEALTHCHECKER_FIX_COMPLETION.md`
