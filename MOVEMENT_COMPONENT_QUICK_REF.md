# MovementComponent Quick Reference

**Phase 3 ECS Improvement** - Movement State as Component

---

## Component Definition

```rust
#[derive(Component, Debug, Clone)]
pub enum MovementComponent {
    Idle,
    PathRequested { request_id: PathRequestId },
    FollowingPath { path: Vec<IVec2>, index: usize },
    Stuck { attempts: u32 },
}
```

---

## Import

```rust
use life_simulator::entities::MovementComponent;
```

---

## Constructor Methods

| Method | Usage | Description |
|--------|-------|-------------|
| `idle()` | `MovementComponent::idle()` | Create idle state |
| `path_requested(id)` | `MovementComponent::path_requested(PathRequestId::new(42))` | Create path requested state |
| `following_path(path)` | `MovementComponent::following_path(vec![pos1, pos2])` | Create following path state |
| `stuck(attempts)` | `MovementComponent::stuck(3)` | Create stuck state |

---

## State Check Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `is_idle()` | `bool` | True if idle |
| `is_path_requested()` | `bool` | True if waiting for path |
| `is_following_path()` | `bool` | True if following path |
| `is_stuck()` | `bool` | True if stuck |

---

## Data Access Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `get_path()` | `Option<&Vec<IVec2>>` | Get current path if following |
| `get_path_index()` | `Option<usize>` | Get path index if following |
| `get_request_id()` | `Option<PathRequestId>` | Get request ID if waiting |
| `get_stuck_attempts()` | `Option<u32>` | Get stuck attempts if stuck |

---

## Common Patterns

### Spawn Entity with Movement

```rust
let entity = commands.spawn((
    TilePosition::new(0, 0),
    MovementComponent::Idle,
)).id();
```

### Transition States

```rust
// Idle → Path Requested
commands.entity(entity).insert(MovementComponent::PathRequested {
    request_id: PathRequestId::new(42),
});

// Path Requested → Following Path
commands.entity(entity).insert(MovementComponent::FollowingPath {
    path: vec![IVec2::new(1, 1), IVec2::new(2, 2)],
    index: 0,
});

// Following Path → Idle
commands.entity(entity).insert(MovementComponent::Idle);
```

### Query Movement State

```rust
fn my_system(query: Query<(Entity, &MovementComponent)>) {
    for (entity, movement) in query.iter() {
        match movement {
            MovementComponent::Idle => { /* handle idle */ },
            MovementComponent::PathRequested { request_id } => { /* handle waiting */ },
            MovementComponent::FollowingPath { path, index } => { /* handle moving */ },
            MovementComponent::Stuck { attempts } => { /* handle stuck */ },
        }
    }
}
```

### Filter by State

```rust
fn process_moving_entities(
    query: Query<(Entity, &MovementComponent)>,
) {
    for (entity, movement) in query.iter() {
        if movement.is_following_path() {
            // Process only moving entities
        }
    }
}
```

### Execute Movement (Example System)

```rust
fn execute_movement(
    mut query: Query<(Entity, &mut MovementComponent, &mut TilePosition)>,
) {
    for (entity, mut movement, mut pos) in query.iter_mut() {
        match *movement {
            MovementComponent::FollowingPath { ref path, ref mut index } => {
                if *index < path.len() {
                    *pos = TilePosition::from_tile(path[*index]);
                    *index += 1;

                    if *index >= path.len() {
                        *movement = MovementComponent::Idle;
                    }
                }
            }
            _ => {}
        }
    }
}
```

---

## State Transition Flow

```
Idle
  ↓ (request path)
PathRequested { request_id }
  ↓ (path computed)
FollowingPath { path, index }
  ↓ (path complete)
Idle

OR

FollowingPath { path, index }
  ↓ (blocked/failed)
Stuck { attempts }
  ↓ (retry or give up)
Idle OR PathRequested
```

---

## Benefits

1. **Separation of Concerns**: Movement logic separate from action state machines
2. **Reusability**: Any system can query/modify movement state
3. **Visibility**: Movement state visible in component inspector
4. **Single Source of Truth**: All actions use same movement representation

---

## File Location

- **Component**: `src/entities/movement_component.rs`
- **Export**: `src/entities/mod.rs`
- **Tests**: `tests/movement_state_test.rs`
