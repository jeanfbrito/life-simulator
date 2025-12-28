# Bevy System Parameters - Quick Reference Guide

## ⚠️ Common Pitfall: &World + Commands Conflict

### The Problem

```rust
// ❌ RUNTIME PANIC!
fn broken_system(
    mut commands: Commands,
    world: &World,
) {
    // Error: "&World conflicts with a previous mutable system parameter"
}
```

### Why It Fails

| Parameter | What It Needs | Scope |
|-----------|---------------|-------|
| `&World` | Immutable borrow | **ENTIRE** ECS world |
| `Commands` | Deferred mutable access | Any component/entity |
| `ResMut<T>` | Mutable borrow | Resource `T` |
| `Query<&mut T>` | Mutable access | Component `T` |

**Rust's Rule:** Can't have `&everything` while also having `&mut anything`

## ✅ Solution Matrix

### When to Use Each Approach

| Scenario | Solution | Performance | Code Complexity |
|----------|----------|-------------|-----------------|
| Need specific components | **Query parameters** ⭐ | Best (parallel) | Low |
| Need multiple conflicting queries | **ParamSet** | Good (sequential) | Medium |
| Need bulk entity operations | **Exclusive system** | Poor (blocking) | Low |
| Need occasional direct access | **SystemState** | Good | Medium |

### Solution 1: Query Parameters (PREFERRED)

```rust
✅ fn good_system(
    mut commands: Commands,
    // Instead of &World, query specific components
    leaders: Query<&PackLeader>,
    members: Query<&PackMember>,
    positions: Query<&TilePosition>,
) {
    // Access only what you need
    if let Ok(leader) = leaders.get(entity) {
        // ...
    }
}
```

**When:** You need specific component data
**Pros:** Parallel execution, type-safe, idiomatic Bevy
**Cons:** More verbose parameter list

### Solution 2: ParamSet (Unavoidable Conflicts)

```rust
✅ fn paramset_system(
    mut params: ParamSet<(
        &World,              // p0
        Commands,            // p1
        Query<&mut Health>,  // p2
    )>
) {
    // Access one at a time
    let world = params.p0();
    let entity_count = world.entities().len();

    // Must finish with world before using commands
    let mut commands = params.p1();
    commands.spawn(/* ... */);
}
```

**When:** Truly need `&World` AND other mutable params
**Pros:** Handles unavoidable conflicts
**Cons:** Sequential access only, max 8 params

### Solution 3: Exclusive System (Performance Cost)

```rust
✅ fn exclusive_system(world: &mut World) {
    // Full unrestricted access
    world.spawn(/* ... */);

    // Can use SystemState for scoped queries
    let mut query = world.query::<&Health>();
    for health in query.iter(world) {
        // ...
    }
}
```

**When:** Bulk spawning/despawning many entities
**Pros:** Full world access, immediate effects
**Cons:** **BLOCKS ALL OTHER SYSTEMS** - major performance hit

### Solution 4: SystemState (Hybrid Approach)

```rust
fn system_with_state(world: &mut World, state: &mut SystemState<(
    Commands,
    Query<&Health>,
    Res<GameState>,
)>) {
    let (mut commands, health_query, game_state) = state.get_mut(world);

    // Use params like normal system
    for health in health_query.iter() {
        // ...
    }

    // Apply deferred commands
    state.apply(world);
}
```

**When:** Need occasional direct world access in resource methods
**Pros:** Best of both worlds
**Cons:** Manual setup required

## 🔍 Detection & Prevention

### Pre-Commit Linter

```bash
# Check for conflicts before committing
./scripts/check_bevy_conflicts.sh

# Add to git hooks
echo './scripts/check_bevy_conflicts.sh' >> .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### CI Integration

GitHub Actions automatically runs the linter on every PR. See `.github/workflows/bevy-lint.yml`

### Manual Detection

Search your codebase:
```bash
# Find potential conflicts
grep -r "world.*:.*&World" src/ | grep "Commands"
```

## 📚 Common Patterns

### Pattern: Group Behavior Coordination

❌ **Before (Conflict):**
```rust
fn plan_actions(
    mut commands: Commands,
    world: &World,  // ❌ Conflict!
) {
    apply_group_bonuses(entity, &mut actions, world);
}
```

✅ **After (Queries):**
```rust
fn plan_actions(
    mut commands: Commands,
    leaders: Query<&PackLeader>,
    members: Query<&PackMember>,
) {
    apply_group_bonuses(entity, &mut actions, &leaders, &members);
}
```

### Pattern: Action Execution

❌ **Before (Conflict):**
```rust
fn execute_actions(
    mut commands: Commands,
    query: Query<&ActiveAction>,
    world: &World,  // ❌ Conflict!
) {
    for action in query.iter() {
        action.execute(world, entity);
    }
}
```

✅ **After (Two Systems):**
```rust
// System 1: Read-only execution
fn execute_actions_readonly(world: &mut World) {
    // Exclusive system for &World access
    // Store results in component
}

// System 2: Handle results
fn handle_action_results(
    mut commands: Commands,
    results: Query<&ActionResult>,
) {
    // Process results with Commands
}

// Chain them
app.add_systems(Update, (
    execute_actions_readonly,
    apply_deferred,  // ← CRITICAL!
    handle_action_results,
).chain());
```

## 🎓 Key Takeaways

1. **`&World` is greedy** - it borrows the entire ECS world immutably
2. **Use specific Queries** - only access what you need
3. **ParamSet for exceptions** - when you truly need conflicting access
4. **Exclusive systems are expensive** - blocks all parallelism
5. **Linter catches issues** - run before committing

## 📖 Further Reading

- [Bevy Cheat Book - Systems](https://bevy-cheatbook.github.io/programming/systems.html)
- [Bevy Cheat Book - ParamSet](https://bevy-cheatbook.github.io/programming/paramset.html)
- [Bevy Cheat Book - Exclusive Systems](https://bevy-cheatbook.github.io/programming/exclusive.html)
- [Official ParamSet Docs](https://docs.rs/bevy/latest/bevy/ecs/system/struct.ParamSet.html)
- [Bevy ECS Guide](https://bevyengine.org/learn/book/getting-started/ecs/)

---

**Last Updated:** 2025-12-28
**Bevy Version:** 0.16.x
