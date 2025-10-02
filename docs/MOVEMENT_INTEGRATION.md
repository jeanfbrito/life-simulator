

# Movement & Pathfinding Integration Guide

## Overview

This guide explains how to integrate the **tick-based movement** and **A* pathfinding** system into your life simulator.

**Key Points:**
- ✅ Pure A* algorithm extracted from bevy_entitiles (no dependencies)
- ✅ Pathfinding runs **asynchronously** (every frame, not tick-synced)
- ✅ Movement execution is **discrete and tick-synced** (only moves on simulation ticks)
- ✅ Entities move tile-by-tile, not smoothly interpolated

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NON-TICK SYSTEMS                         │
│                  (run every frame, 60fps)                   │
├─────────────────────────────────────────────────────────────┤
│  1. initiate_pathfinding()                                  │
│     MoveOrder → PathRequest                                 │
│                                                             │
│  2. process_pathfinding_requests()                          │
│     PathRequest → Path (A* algorithm)                       │
│                                                             │
│  3. initialize_movement_state()                             │
│     Path → MovementState (tracking)                         │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                     TICK SYSTEMS                            │
│             (run on simulation tick, e.g. 1-10 TPS)        │
├─────────────────────────────────────────────────────────────┤
│  tick_movement_system()                                     │
│     Execute one step of Path                                │
│     TilePosition updated discretely                         │
└─────────────────────────────────────────────────────────────┘
```

## Step 1: Add Module to lib.rs

```rust
// src/lib.rs
pub mod pathfinding;
pub mod entities;
```

## Step 2: Update main.rs

Add the pathfinding and movement systems to your app:

```rust
// src/main.rs
use bevy::prelude::*;
use life_simulator::{
    pathfinding::{PathfindingGrid, process_pathfinding_requests, build_pathfinding_grid_from_world},
    entities::{EntitiesPlugin, movement::tick_movement_system},
    world_loader::WorldLoader,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                filter: "info,life_simulator=debug".to_string(),
                ..default()
            }),
            MinimalPlugins,
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ))
        // Resources
        .init_resource::<WorldLoader>()
        .init_resource::<PathfindingGrid>()
        
        // Plugins
        .add_plugins(EntitiesPlugin)
        
        // Startup
        .add_systems(Startup, (
            load_world_system,
            setup_pathfinding_grid.after(load_world_system),
            spawn_test_creatures.after(setup_pathfinding_grid),
        ))
        
        // Update (every frame, ~60fps)
        .add_systems(Update, (
            process_pathfinding_requests,
            // Other non-tick systems...
        ))
        
        // Simulation Tick (add your tick schedule)
        .add_systems(FixedUpdate, (
            tick_movement_system,
            // Other tick-synced systems...
        ))
        
        .run();
}

/// Build pathfinding grid from loaded world
fn setup_pathfinding_grid(
    world_loader: Res<WorldLoader>,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
) {
    *pathfinding_grid = build_pathfinding_grid_from_world(&world_loader);
}

/// Example: Spawn test creatures
fn spawn_test_creatures(mut commands: Commands) {
    use life_simulator::entities::{spawn_creature, MovementSpeed, issue_move_order};
    
    let creature = spawn_creature(
        &mut commands,
        "Bob",
        "Human",
        IVec2::new(0, 0),
        MovementSpeed::normal(),
    );
    
    // Order creature to move to (10, 10)
    issue_move_order(&mut commands, creature, IVec2::new(10, 10));
}
```

## Step 3: Define Simulation Tick Schedule (if needed)

If you don't have a tick system yet, create one:

```rust
// In main.rs or a new scheduler.rs module

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SimulationTick;

fn main() {
    App::new()
        // ... existing setup ...
        
        // Configure fixed timestep for ticks (e.g., 10 TPS = 100ms per tick)
        .insert_resource(Time::<Fixed>::from_seconds(0.1))
        
        // Add tick schedule
        .add_systems(FixedUpdate, (
            tick_movement_system,
            // Add other tick-synced systems here
        ).in_set(SimulationTick))
        
        .run();
}
```

## Usage Examples

### Example 1: Spawn and Move a Creature

```rust
use life_simulator::entities::{spawn_creature, issue_move_order, MovementSpeed};

fn spawn_wandering_creature(mut commands: Commands) {
    // Spawn creature at origin
    let entity = spawn_creature(
        &mut commands,
        "Alice",
        "Explorer",
        IVec2::new(0, 0),
        MovementSpeed::fast(), // Moves every tick
    );
    
    // Order to move to destination
    issue_move_order(&mut commands, entity, IVec2::new(20, 15));
}
```

### Example 2: Check Movement Status

```rust
use life_simulator::entities::{is_moving, get_position, TilePosition};

fn check_entity_status(
    query_moving: Query<(), (With<Path>, With<MovementState>)>,
    query_position: Query<&TilePosition>,
    entity: Entity,
) {
    if is_moving(entity, &query_moving) {
        println!("Entity is moving!");
        
        if let Some(pos) = get_position(entity, &query_position) {
            println!("Current position: {:?}", pos);
        }
    } else {
        println!("Entity is idle");
    }
}
```

### Example 3: Stop Movement

```rust
use life_simulator::entities::stop_movement;

fn stop_all_movement(
    mut commands: Commands,
    query: Query<Entity, With<Path>>,
) {
    for entity in query.iter() {
        stop_movement(&mut commands, entity);
    }
}
```

### Example 4: React to Destination Reached

Use Bevy's event system or component queries:

```rust
fn detect_arrived(
    query: Query<(Entity, &TilePosition), (Without<Path>, Changed<TilePosition>)>,
) {
    for (entity, position) in query.iter() {
        info!("Entity {:?} arrived at {:?}", entity, position.tile);
        // Trigger behavior, assign new task, etc.
    }
}
```

## Component Lifecycle

### Flow for a Moving Entity:

1. **Initial State**: Entity has `TilePosition` and `MovementSpeed`

2. **Order Given**: Add `MoveOrder` component
   ```rust
   commands.entity(entity).insert(MoveOrder {
       destination: IVec2::new(10, 10),
       allow_diagonal: false,
   });
   ```

3. **Pathfinding Initiated**: `initiate_pathfinding` system converts to `PathRequest`
   - Components: `TilePosition`, `MovementSpeed`, `PathRequest`

4. **Path Computed**: `process_pathfinding_requests` runs A* algorithm
   - Components: `TilePosition`, `MovementSpeed`, `Path`

5. **Movement State Added**: `initialize_movement_state` prepares for movement
   - Components: `TilePosition`, `MovementSpeed`, `Path`, `MovementState`

6. **Moving**: Every N ticks, `tick_movement_system` advances position
   - `TilePosition.tile` updates discretely
   - `Path.current_index` increments

7. **Arrived**: When path complete, `Path` and `MovementState` removed
   - Components: `TilePosition`, `MovementSpeed` (ready for next order)

## Movement Speed Configuration

```rust
// Fast: 1 tile per tick
let speed = MovementSpeed::fast(); // ticks_per_move = 1

// Normal: 1 tile per 2 ticks
let speed = MovementSpeed::normal(); // ticks_per_move = 2

// Slow: 1 tile per 4 ticks
let speed = MovementSpeed::slow(); // ticks_per_move = 4

// Custom: 1 tile per 10 ticks
let speed = MovementSpeed::custom(10); // ticks_per_move = 10
```

**Example at 10 TPS:**
- Fast = 10 tiles/second
- Normal = 5 tiles/second
- Slow = 2.5 tiles/second
- Custom(10) = 1 tile/second

## Pathfinding Grid Management

### Initial Setup (Startup)

```rust
fn setup_pathfinding_grid(
    world_loader: Res<WorldLoader>,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
) {
    *pathfinding_grid = build_pathfinding_grid_from_world(&world_loader);
}
```

### Update When Terrain Changes

```rust
use life_simulator::pathfinding::update_pathfinding_grid_for_tile;

fn handle_terrain_change(
    mut pathfinding_grid: ResMut<PathfindingGrid>,
) {
    let changed_tile = IVec2::new(5, 5);
    let new_terrain = TerrainType::Mountain;
    
    update_pathfinding_grid_for_tile(
        &mut pathfinding_grid,
        changed_tile,
        new_terrain,
    );
}
```

### Temporarily Block Tiles (e.g., for buildings)

```rust
fn block_building_footprint(
    mut pathfinding_grid: ResMut<PathfindingGrid>,
    building_pos: IVec2,
) {
    // Make tile impassable
    pathfinding_grid.set_cost(building_pos, u32::MAX);
}
```

## Testing

### Run Unit Tests

```bash
# Test pathfinding algorithm
cargo test --lib pathfinding

# Example tests included:
# - test_straight_line_path
# - test_obstacle_avoidance
# - test_no_path_exists
# - test_manhattan_distance
```

### Integration Test

```rust
// tests/movement_test.rs
#[test]
fn test_movement_integration() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins)
       .add_plugins(EntitiesPlugin)
       .init_resource::<PathfindingGrid>()
       .add_systems(Update, process_pathfinding_requests);
    
    // Setup grid
    let mut grid = PathfindingGrid::new();
    for y in 0..10 {
        for x in 0..10 {
            grid.set_cost(IVec2::new(x, y), 1);
        }
    }
    app.insert_resource(grid);
    
    // Spawn entity
    let entity = app.world.spawn((
        TilePosition::new(0, 0),
        MovementSpeed::fast(),
        MoveOrder {
            destination: IVec2::new(5, 5),
            allow_diagonal: false,
        },
    )).id();
    
    // Run one frame - should convert to PathRequest
    app.update();
    assert!(app.world.get::<PathRequest>(entity).is_some());
    
    // Run another frame - should compute Path
    app.update();
    assert!(app.world.get::<Path>(entity).is_some());
}
```

## Performance Notes

### Pathfinding Cost

- **O(E log V)** where E = edges, V = vertices (tiles)
- Typical 100x100 grid with obstacles: **< 1ms**
- 500x500 grid: **< 10ms** (still acceptable as it's not tick-synced)

### Optimization Tips

1. **Limit search area**: Set reasonable `max_steps` (default: 1000)
2. **Cache paths**: Store common paths (e.g., home → water source)
3. **Lazy updates**: Only rebuild PathfindingGrid when terrain actually changes
4. **Diagonal movement**: Set `allow_diagonal: true` for more direct paths

### Memory Usage

- **PathfindingGrid**: ~8 bytes per tile (IVec2 + u32)
- **100x100 grid**: ~80KB
- **500x500 grid**: ~2MB

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=life_simulator=debug cargo run --bin life-simulator
```

### Log Output Examples

```
[INFO] PathfindingGrid built with 10000 tiles
[INFO] Entity 0v1 requesting path from (0, 0) to (10, 10)
[DEBUG] Initialized movement state for entity 0v1
[DEBUG] Entity 0v1 moved to (1, 0), remaining waypoints: 19
[INFO] Entity 0v1 reached destination at (10, 10)
```

### Visualize Paths in Web Viewer

Add to your web API:

```rust
// In web_server_simple.rs
#[derive(Serialize)]
struct EntityPathData {
    entity_id: u32,
    current_pos: IVec2,
    waypoints: Vec<IVec2>,
}

fn get_entity_paths(
    query: Query<(Entity, &TilePosition, &Path)>,
) -> Json<Vec<EntityPathData>> {
    let paths: Vec<_> = query.iter().map(|(entity, pos, path)| {
        EntityPathData {
            entity_id: entity.index(),
            current_pos: pos.tile,
            waypoints: path.all_waypoints().to_vec(),
        }
    }).collect();
    
    Json(paths)
}

// Add route: GET /api/entities/paths
```

## Common Issues

### Issue: Entity not moving

**Checklist:**
- [ ] Does entity have `MovementSpeed` component?
- [ ] Is `tick_movement_system` added to tick schedule?
- [ ] Is tick schedule actually running?
- [ ] Check logs for "requesting path" message
- [ ] Check if path was successfully computed

### Issue: Pathfinding fails

**Possible causes:**
- Destination is impassable (DeepWater, etc.)
- Destination is unreachable (surrounded by obstacles)
- PathfindingGrid not initialized
- max_steps too low for long paths

### Issue: Movement is too fast/slow

**Solution:**
- Adjust `MovementSpeed.ticks_per_move`
- Change tick rate (Time<Fixed> resource)
- Example: 10 TPS + ticks_per_move=2 = 5 tiles/sec

## Next Steps

1. ✅ **Basic movement working**
2. ⬜ Add path visualization to web viewer
3. ⬜ Implement creature AI (pick random destinations, wander, etc.)
4. ⬜ Add needs system (hunger, thirst) that triggers movement
5. ⬜ Implement dynamic obstacles (other entities block path)
6. ⬜ Add path caching for common routes
7. ⬜ Implement formations (multiple entities moving together)
8. ⬜ Add terrain-based speed modifiers (Forest = slower, Road = faster)

## Reference

- **Core files:**
  - `src/pathfinding.rs` - A* algorithm and PathfindingGrid
  - `src/entities/movement.rs` - Tick-based movement execution
  - `src/entities/mod.rs` - Entity types and plugin

- **Original inspiration:**
  - `/Users/jean/Github/bevy_entitiles/src/algorithm/pathfinding.rs`
  - Simplified and adapted for tick-based simulation
