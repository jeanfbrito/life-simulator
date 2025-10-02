# Movement System - Quick Start

## What We Built

✅ **Pure A* pathfinding** - No external dependencies, extracted from bevy_entitiles  
✅ **Tick-based discrete movement** - Entities move tile-by-tile on simulation ticks  
✅ **Async pathfinding** - Path calculation runs every frame, movement runs on ticks  
✅ **Terrain-aware costs** - Different terrain types have different movement costs  

## Files Created

```
src/
  pathfinding.rs          # A* algorithm, PathfindingGrid
  entities/
    mod.rs                # Entity types, plugin, spawn helpers
    movement.rs           # Tick-based movement execution
docs/
  MOVEMENT_INTEGRATION.md # Complete integration guide
  MOVEMENT_QUICKSTART.md  # This file
```

## Key Concepts

### 1. Two-Phase System

**Phase 1: Pathfinding (Non-Tick, Every Frame)**
- `MoveOrder` → `PathRequest` → `Path`
- Runs at ~60fps for responsive path calculation
- Uses A* algorithm with terrain costs

**Phase 2: Movement (Tick-Synced)**
- Executes one waypoint per N ticks (based on speed)
- Discrete tile-by-tile jumps (no interpolation)
- Synced with simulation logic

### 2. Component Flow

```rust
// Spawn entity
TilePosition + MovementSpeed

// Give order
+ MoveOrder

// Pathfinding starts
- MoveOrder
+ PathRequest

// Path computed
- PathRequest
+ Path

// Movement prepared
+ MovementState

// Moving... (each tick)
TilePosition.tile updated

// Arrived
- Path
- MovementState
```

### 3. Speed Control

```rust
MovementSpeed::fast()     // 1 tile per tick
MovementSpeed::normal()   // 1 tile per 2 ticks
MovementSpeed::slow()     // 1 tile per 4 ticks
MovementSpeed::custom(n)  // 1 tile per n ticks
```

At 10 TPS:
- Fast = 10 tiles/sec
- Normal = 5 tiles/sec  
- Slow = 2.5 tiles/sec

## Minimal Integration

### 1. Add to `src/lib.rs`

```rust
pub mod pathfinding;
pub mod entities;
```

### 2. Update `src/main.rs`

```rust
use life_simulator::{
    pathfinding::{PathfindingGrid, process_pathfinding_requests, build_pathfinding_grid_from_world},
    entities::{EntitiesPlugin, movement::tick_movement_system},
};

fn main() {
    App::new()
        .init_resource::<PathfindingGrid>()
        .add_plugins(EntitiesPlugin)
        .add_systems(Startup, setup_pathfinding_grid)
        .add_systems(Update, process_pathfinding_requests)
        .add_systems(FixedUpdate, tick_movement_system)  // Or your tick schedule
        .run();
}

fn setup_pathfinding_grid(
    world_loader: Res<WorldLoader>,
    mut grid: ResMut<PathfindingGrid>,
) {
    *grid = build_pathfinding_grid_from_world(&world_loader);
}
```

### 3. Spawn a Creature

```rust
use life_simulator::entities::{spawn_creature, issue_move_order, MovementSpeed};

fn test_system(mut commands: Commands) {
    let entity = spawn_creature(
        &mut commands,
        "Bob",
        "Human",
        IVec2::new(0, 0),
        MovementSpeed::normal(),
    );
    
    issue_move_order(&mut commands, entity, IVec2::new(10, 10));
}
```

## Testing

```bash
# Run pathfinding tests
cargo test --lib pathfinding

# Run with debug logs
RUST_LOG=life_simulator=debug cargo run --bin life-simulator

# Expected output:
# [INFO] PathfindingGrid built with 10000 tiles
# [INFO] Entity 0v1 requesting path from (0, 0) to (10, 10)
# [DEBUG] Entity 0v1 moved to (1, 0), remaining waypoints: 19
# [INFO] Entity 0v1 reached destination at (10, 10)
```

## API Reference

### Spawn & Control

```rust
// Spawn
let entity = spawn_creature(commands, name, species, pos, speed);

// Give move order
issue_move_order(commands, entity, destination);

// Stop movement
stop_movement(commands, entity);

// Check status
let moving = is_moving(entity, &query);
let pos = get_position(entity, &query);
```

### Pathfinding Grid

```rust
// Build from world
let grid = build_pathfinding_grid_from_world(&world_loader);

// Update single tile
update_pathfinding_grid_for_tile(&mut grid, pos, terrain);

// Manual control
grid.set_cost(pos, cost);
grid.get_cost(pos);
grid.is_walkable(pos);
```

## What's Different from bevy_entitiles

| Feature | bevy_entitiles | Our Implementation |
|---------|----------------|-------------------|
| Dependencies | Required crate | Zero dependencies |
| Execution | Multi-threaded tasks | Simple frame-based |
| Movement | Smooth interpolation | Discrete tick-based |
| Tilemap | Separate entity | Direct terrain integration |
| Complexity | Full ECS integration | Minimal, focused |

## Next Steps

1. ✅ Basic movement working
2. ⬜ Add to web viewer visualization
3. ⬜ Implement creature AI (wandering, goals)
4. ⬜ Add needs system (hunger → find food)
5. ⬜ Dynamic obstacles (entities blocking tiles)

## Troubleshooting

**Entity not moving?**
- Check `MovementSpeed` component exists
- Verify `tick_movement_system` in tick schedule
- Enable debug logs: `RUST_LOG=life_simulator=debug`

**Pathfinding fails?**
- Destination might be impassable (DeepWater)
- Check PathfindingGrid initialized
- Try increasing `max_steps`

**Movement too fast/slow?**
- Adjust `MovementSpeed.ticks_per_move`
- Change tick rate: `Time::<Fixed>::from_seconds(0.1)`

## Documentation

- **Full Guide**: `docs/MOVEMENT_INTEGRATION.md`
- **CLAUDE.md**: Section "Pathfinding and Movement System"
- **Source Code**: `src/pathfinding.rs`, `src/entities/movement.rs`
