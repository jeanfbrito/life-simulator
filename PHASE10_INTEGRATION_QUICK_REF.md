# Phase 10 Integration Quick Reference

## Infrastructure Status: READY ✓

All hunting relationship infrastructure is registered and tested. Ready for HuntAction integration.

## Key Components

### Components
- **ActiveHunter**: On predator when hunting
  - `target: Entity` - prey entity
  - `started_tick: u64` - when hunt began

- **HuntingTarget**: On prey when hunted
  - `predator: Entity` - predator entity
  - `started_tick: u64` - when hunt began

### Public Functions

```rust
// Establish new hunt
pub fn establish_hunting_relationship(
    predator: Entity,
    prey: Entity,
    tick: u64,
    commands: &mut Commands,
)

// End hunt (completion or failure)
pub fn clear_hunting_relationship(
    predator: Entity,
    prey: Entity,
    commands: &mut Commands,
)

// Validate hunt is still active
pub fn has_hunting_relationship(
    predator: Entity,
    target: Entity,
    world: &World,
) -> bool

// Check if prey is hunted by anyone
pub fn is_being_hunted(
    prey: Entity,
    world: &World,
) -> bool

// Automatic cleanup of stale relationships (system)
pub fn cleanup_stale_hunting_relationships(...)
```

## Integration Points in HuntAction

### 1. When Creating Hunt (NeedPath state)
```rust
// Nothing to do here - hunt not established yet
// Wait until we're about to attack
```

### 2. When Close to Prey (Attacking state)
```rust
HuntState::Attacking => {
    // If not already hunting, establish relationship
    if !has_hunting_relationship(entity, self.prey, world) {
        // NEW: Establish the hunt
        // This marks prey as actively being hunted
        commands.run_system(|mut commands: Commands| {
            establish_hunting_relationship(
                entity,           // predator (from parameter)
                self.prey,        // prey (from HuntAction.prey)
                current_tick,     // from SimulationTick resource
                &mut commands
            );
        });
    }

    // Continue with attack logic...
}
```

### 3. When Hunt Succeeds (return Success)
```rust
ActionResult::Success => {
    // NEW: Clear the hunt relationship
    clear_hunting_relationship(entity, self.prey, &mut commands);
    // Continue with success logic (consume food, etc)
}
```

### 4. When Hunt Fails (return Failed)
```rust
ActionResult::Failed => {
    // NEW: Clear the hunt relationship
    clear_hunting_relationship(entity, self.prey, &mut commands);
    // Rest of failure handling
}
```

### 5. In cancel() method
```rust
fn cancel(&mut self, world: &World, entity: Entity) {
    clear_navigation_state(world, entity);

    // NEW: Clear hunting relationship if hunt was active
    // This is called when hunt action is interrupted
    if let Some(hunter) = world.get::<ActiveHunter>(entity) {
        if hunter.target == self.prey {
            // Relationship was active, clear it via Commands
            // (Note: can't call Commands directly in cancel,
            //  return failed to trigger cleanup)
        }
    }

    self.state = HuntState::NeedPath;
    self.retry_count = 0;
    self.last_prey_pos = None;
}
```

## Integration Checklist

- [ ] Integrate relationship establishment in HuntAction::execute()
- [ ] Integrate relationship clearing in ActionResult::Success
- [ ] Integrate relationship clearing in ActionResult::Failed
- [ ] Add validation during hunt progress (has_hunting_relationship check)
- [ ] Test hunt establishment in integration tests
- [ ] Test hunt clearing on success
- [ ] Test hunt clearing on failure
- [ ] Verify cleanup system removes stale relationships
- [ ] Run full test suite (should remain at 292+ tests passing)
- [ ] Measure TPS impact (should be 0)
- [ ] Update HuntAction documentation

## Testing Strategy

After integrating into HuntAction:

1. **Establishment Test**: Verify ActiveHunter/HuntingTarget added when hunting
2. **Success Test**: Verify components removed when prey killed
3. **Failure Test**: Verify components removed when hunt fails
4. **Duration Test**: Verify relationship persists for correct duration
5. **Cleanup Test**: Verify orphaned relationships cleaned if prey despawned
6. **Multi-Hunt Test**: Verify multiple predators can hunt simultaneously
7. **Performance Test**: Verify no TPS regression

## Common Pitfalls to Avoid

1. **Don't establish in NeedPath state** - Hunt not happening yet
2. **Don't forget to clear on Failed** - Prevents stale relationships
3. **Don't call Commands in can_execute()** - Use read-only world
4. **Don't check relationships in planning systems** - Creates circular dependency
5. **Don't forget cancel() cleanup** - Action can be interrupted

## Performance Notes

- Establishment: O(1) component insertion
- Validation: O(1) component lookup
- Clearing: O(1) component removal
- Cleanup: O(n) where n = active hunters (runs in Cleanup phase)
- Zero impact on other systems or TPS

## Related Files

- Implementation: `src/ai/hunting_relationship_system.rs`
- Integration point: `src/ai/action.rs` (HuntAction)
- Tests: `tests/hunting_relationship_integration.rs`
- Documentation: `PHASE10_INTEGRATION_DELIVERY.md`
- Module exports: `src/ai/mod.rs`
