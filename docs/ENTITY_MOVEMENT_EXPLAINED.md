# Entity Movement System - How Entities Walk Using Pathfinding

## Overview

Entities in the Life Simulator move **tile-by-tile** along pathfound routes at a configurable speed. They don't teleport - they follow each step of their calculated path over multiple simulation ticks.

## How It Works

### 1. **Wanderer AI Picks a Destination**
Every 5 ticks (0.5 seconds at 10 TPS), idle wanderers pick a random walkable destination within their wander radius.

```rust
// From: src/entities/wandering.rs
pub fn wanderer_ai_system(
    mut commands: Commands,
    mut query: Query<(Entity, &TilePosition, &mut Wanderer), Without<MoveOrder>>,
    // ...
) {
    // Wanderer picks random destination
    let destination = pick_random_walkable_tile(...);
    commands.entity(entity).insert(MoveOrder {
        destination,
        allow_diagonal: false,
    });
}
```

### 2. **MoveOrder → PathRequest**
The movement system converts the move order into a pathfinding request:

```rust
// From: src/entities/movement.rs  
pub fn initiate_pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePosition, &MoveOrder), Without<PathRequest>>,
) {
    // Remove MoveOrder and add PathRequest
    commands.entity(entity)
        .remove::<MoveOrder>()
        .insert(PathRequest {
            origin: position.tile,
            destination: order.destination,
            allow_diagonal: order.allow_diagonal,
            max_steps: Some(1000),
        });
}
```

### 3. **A* Pathfinding Calculates Route**
The pathfinding system uses A* to find the optimal path:

```rust
// From: src/pathfinding.rs
pub fn process_pathfinding_requests(
    mut commands: Commands,
    requests: Query<(Entity, &PathRequest)>,
    grid: Res<PathfindingGrid>,
) {
    if let Some(path) = find_path(
        request.origin,
        request.destination,
        &grid,
        request.allow_diagonal,
        request.max_steps,
    ) {
        // Path found - attach to entity
        commands.entity(entity).insert(path);
    }
}
```

### 4. **Entity Moves Tile-by-Tile Along Path**
The tick-synced movement system executes the path:

```rust
// From: src/entities/movement.rs (RUNS ON FIXED UPDATE)
pub fn tick_movement_system(
    mut query: Query<(Entity, &mut TilePosition, &MovementSpeed, &mut MovementState, &mut Path)>,
    // ...
) {
    for (entity, mut position, speed, mut state, mut path) in query.iter_mut() {
        // Increment tick counter
        state.ticks_since_move += 1;
        
        // Check if enough ticks have passed to move
        if state.ticks_since_move < speed.ticks_per_move {
            continue; // Not time to move yet
        }
        
        // Reset tick counter
        state.ticks_since_move = 0;
        
        // Get next target tile and move there
        if let Some(target) = path.current_target() {
            position.tile = target;  // Move to next tile
            path.advance();           // Advance to next waypoint
        }
    }
}
```

## Movement Speed

Entities have configurable movement speeds:

```rust
#[derive(Component)]
pub struct MovementSpeed {
    pub ticks_per_move: u32,  // How many ticks to wait before moving
}

impl MovementSpeed {
    pub fn fast() -> Self    { Self { ticks_per_move: 1 } }  // 1 tile/tick  = 10 tiles/sec
    pub fn normal() -> Self  { Self { ticks_per_move: 2 } }  // 1 tile/2ticks = 5 tiles/sec
    pub fn slow() -> Self    { Self { ticks_per_move: 4 } }  // 1 tile/4ticks = 2.5 tiles/sec
    pub fn custom(t: u32) -> Self { Self { ticks_per_move: t } }
}
```

### Current Settings

Wandering entities use **`MovementSpeed::custom(5)`**:
- **1 tile per 5 ticks**
- At 10 TPS = **2 tiles per second**
- Chosen for good visualization in the web viewer (1 second polling rate)

## System Architecture

```
┌─────────────────────┐
│ Wanderer AI System  │  (Every 5 ticks)
│  Picks Destination  │
└──────────┬──────────┘
           │ Creates MoveOrder
           ▼
┌─────────────────────┐
│ initiate_pathfinding│  (Every Frame)
│  MoveOrder → Path   │
│      Request        │
└──────────┬──────────┘
           │ Creates PathRequest
           ▼
┌─────────────────────┐
│process_pathfinding  │  (Every Frame, Async)
│    requests         │
│  A* Pathfinding     │
└──────────┬──────────┘
           │ Creates Path
           ▼
┌─────────────────────┐
│tick_movement_system │  (FIXED UPDATE - 10 TPS)
│  Walks Path Step    │
│     by Step         │
└─────────────────────┘
```

## Why No Teleporting?

The movement system ensures smooth tile-by-tile movement:

1. **Path is a List of Waypoints**: Each tile along the route
2. **current_index Tracks Progress**: Entity knows where it is in the path
3. **Tick-Synced Movement**: Only moves when enough ticks have passed
4. **One Tile at a Time**: Advances one waypoint per movement cycle

### Example Path Execution

```
Tick 0:  Entity at (0,0), Path: [(1,0), (2,0), (3,0)]
Tick 5:  Move to (1,0), Path: [(2,0), (3,0)]
Tick 10: Move to (2,0), Path: [(3,0)]
Tick 15: Move to (3,0), Path: []
Tick 16: Arrived! Path complete.
```

## Visualization in Web Viewer

The web viewer polls the entity API every 1 second:

### Before (Teleporting)
- Entity at (0, 0) at t=0s
- Entity at (10, 0) at t=1s
- **Appears to teleport 10 tiles!**

### After (Smooth Movement)
With `MovementSpeed::custom(5)` (2 tiles/sec):
- Entity at (0, 0) at t=0s
- Entity at (2, 0) at t=1s
- Entity at (4, 0) at t=2s
- **Smooth, visible movement!**

## Key Components

### TilePosition
```rust
#[derive(Component)]
pub struct TilePosition {
    pub tile: IVec2,  // Current discrete tile position
}
```

### MoveOrder
```rust
#[derive(Component)]
pub struct MoveOrder {
    pub destination: IVec2,
    pub allow_diagonal: bool,
}
```

### Path
```rust
#[derive(Component)]
pub struct Path {
    waypoints: Vec<IVec2>,   // All tiles in path
    current_index: usize,     // Where we are in path
}
```

### MovementState
```rust
#[derive(Component)]
pub struct MovementState {
    ticks_since_move: u32,  // Tick counter for speed throttling
}
```

## Performance Considerations

### Pathfinding
- Runs **asynchronously** (not tick-synced)
- Uses **A*** for optimal paths
- Capped at **1000 steps** max to prevent infinite loops
- Reuses **PathfindingGrid** resource (built once at startup)

### Movement
- Runs on **FixedUpdate** (tick-synced)
- Only processes entities **with active paths**
- Speed throttling prevents **excessive CPU usage**
- Paths are **removed** when complete (no memory leak)

## Tuning Movement Speed

To make entities faster or slower, edit `src/entities/wandering.rs`:

```rust
pub fn spawn_wandering_person(...) -> Entity {
    commands.spawn((
        // ...
        MovementSpeed::custom(5),  // Change this number!
        // ...
    ))
}
```

Speed recommendations:
- **`custom(1)`**: 10 tiles/sec - Very fast, hard to see individual steps
- **`custom(3)`**: ~3 tiles/sec - Fast but visible
- **`custom(5)`**: 2 tiles/sec - **Current setting** (good for visualization)
- **`custom(10)`**: 1 tile/sec - Slow, very visible
- **`custom(20)`**: 0.5 tiles/sec - Very slow

## Debug Commands

To verify movement is working:

```bash
# Watch movement logs (debug mode only)
tail -f /tmp/life-simulator.log | grep -E "moved from|Path found"

# Test API response
curl http://127.0.0.1:54321/api/entities | jq '.'

# Watch entities move in real-time
watch -n 1 'curl -s http://127.0.0.1:54321/api/entities | jq ".entities[] | {name, position}"'
```

## Future Enhancements

### Smooth Interpolation
Currently entities snap to tiles. Could add smooth interpolation:
```rust
pub struct TilePosition {
    pub tile: IVec2,           // Discrete position
    pub interpolation: f32,    // 0.0 to 1.0 progress to next tile
}
```

### Dynamic Speed
Entities could slow down on difficult terrain:
```rust
let tile_cost = grid.get_cost(target);
let adjusted_speed = speed.ticks_per_move * tile_cost;
```

### Collision Avoidance
Entities could detect and avoid other entities:
```rust
if is_tile_occupied(target, other_entities) {
    wait_or_reroute();
}
```

### Path Smoothing
A* paths can be jagged. Could smooth using:
- String-pulling algorithm
- Bezier curve fitting
- Waypoint reduction

## Related Files

- `src/entities/movement.rs` - Core movement system
- `src/entities/wandering.rs` - AI that triggers movement
- `src/pathfinding.rs` - A* pathfinding algorithm
- `src/simulation/mod.rs` - Tick system configuration
- `web-viewer/js/entity-manager.js` - Web viewer entity polling

## Summary

✅ Entities move **tile-by-tile**, not teleporting  
✅ Movement uses **A* pathfinding** for optimal routes  
✅ Speed is **configurable** per entity  
✅ System is **tick-synced** (runs on FixedUpdate)  
✅ Movement is **smooth and predictable**  
✅ Performance is **optimized** with tick throttling  

The movement system provides a solid foundation for more complex behaviors like formation movement, pursuit/evasion, and tactical positioning!
