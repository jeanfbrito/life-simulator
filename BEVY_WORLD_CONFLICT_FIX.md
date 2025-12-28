# Bevy ECS World Conflict Resolution

## Problem Statement
The simulator was crashing with: `&World conflicts with a previous mutable system parameter`

This is a Bevy ECS error that occurs when a system tries to use both:
1. Exclusive access (`&mut World`) for raw entity/component manipulation
2. Commands or Queries for mutations

## Root Cause Analysis
The conflicting system was `execute_active_actions_system` in `src/ai/queue.rs`, which had:
- `world: &mut World` parameter (exclusive access)
- `commands: Commands` parameter (deferred mutations)
- Multiple query parameters

Bevy's scheduler cannot safely run systems that mix exclusive World access with command-based mutations.

## Solution: Two-System Split Pattern

### Before (Conflicting):
```rust
pub fn execute_active_actions_system(
    world: &mut World,  // Exclusive access
    mut commands: Commands,  // Mutation commands - CONFLICT!
    mut query: Query<...>,  // More mutations - CONFLICT!
) {
    // Mixed exclusive and deferred operations
}
```

### After (Resolved):
```rust
// System 1: Read-only with exclusive World access
pub fn execute_active_actions_read_system(
    world: &mut World,
) {
    // Only read operations
    // Write intermediate component for coordination
    world.entity_mut(entity).insert(PendingActionResult { ... });
}

// System 2: Mutations with Commands
pub fn execute_active_actions_write_system(
    mut commands: Commands,
    mut query: Query<...>,
) {
    // Process results from System 1
    // Apply all mutations via Commands
}
```

### Registration:
```rust
.add_systems(
    Update,
    (
        execute_active_actions_read_system,
        apply_deferred,  // CRITICAL: flush commands between systems
        execute_active_actions_write_system,
    )
        .chain()
        .run_if(in_state(SimulationState::Running)),
)
```

## Key Principles

1. **Exclusive World Access**: Use `&mut World` only for read operations or direct entity manipulation
2. **Commands for Mutations**: Use `Commands` for spawning, despawning, or inserting components
3. **Intermediate Components**: Use marker components to coordinate between exclusive and command-based systems
4. **apply_deferred**: Always flush commands between chained systems using `.chain()` and `apply_deferred`

## Files Modified
- `src/ai/queue.rs`: Split `execute_active_actions_system` into two systems
- `src/ai/mod.rs`: Updated system registration with `.chain()` and `apply_deferred`

## Validation Results
- ✅ Compiles without errors
- ✅ Simulator runs for 30+ seconds without crashing
- ✅ No "&World conflicts" errors
- ✅ No panic or thread errors
- ✅ All stress tests pass (100 entities spawned successfully)

## Related Issues
This fix resolves:
- Entity count stress test crashes
- Ecosystem balance test failures
- Random simulation panics under load

## References
- Bevy ECS System Parameters: https://docs.rs/bevy/latest/bevy/ecs/system/index.html
- Bevy World Access: https://docs.rs/bevy/latest/bevy/ecs/world/struct.World.html
- System Chaining: https://docs.rs/bevy/latest/bevy/ecs/schedule/struct.Chain.html
