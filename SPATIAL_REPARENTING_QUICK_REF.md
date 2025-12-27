# Spatial Reparenting System - Quick Reference

## Overview
Budget-controlled system that migrates entities from HashMap-based spatial tracking to Bevy's Parent/Child hierarchy for improved performance and query capabilities.

## Components

### SpatiallyParented (Marker)
```rust
#[derive(Component)]
pub struct SpatiallyParented;
```
**Purpose**: Marks entities that have been reparented to spatial cells
**Usage**: Automatically added by reparenting systems
**Location**: `src/entities/spatial_cell.rs`

## Systems

### 1. reparent_entities_to_cells
**Type**: Tick-based (Cleanup phase)
**Budget**: 50 entities per tick
**Purpose**: Incrementally migrate entities to spatial hierarchy

```rust
// Query: Entities WITHOUT SpatiallyParented marker
Query<(Entity, &TilePosition), Without<SpatiallyParented>>

// Operation:
// 1. Get chunk coordinate from TilePosition
// 2. Find SpatialCell entity for that chunk
// 3. Add entity as child: commands.entity(cell).add_child(entity)
// 4. Mark entity: commands.entity(entity).insert(SpatiallyParented)
```

**When it runs**: Every tick, in Cleanup phase (after all movement/spawning)
**Migration time**: ~2-10 ticks for typical entity counts (100-500 entities)

### 2. update_spatial_parent_on_movement
**Type**: Frame-based (Update schedule)
**Purpose**: Keep parent relationships current as entities move

```rust
// Query: Entities WITH SpatiallyParented that moved
Query<(Entity, &TilePosition, &ChildOf),
      (Changed<TilePosition>, With<SpatiallyParented>)>

// Operation:
// 1. Check if entity moved to different chunk
// 2. If yes, reparent to new cell: commands.entity(new_cell).add_child(entity)
```

**When it runs**: Every frame, immediately after position changes
**Optimization**: Only processes entities that actually moved (Changed<TilePosition>)

## Querying the Hierarchy

### Get all entities in a chunk
```rust
fn query_chunk_entities(
    cells: Query<(&SpatialCell, &Children)>,
    grid: Res<SpatialCellGrid>,
) {
    let chunk_coord = IVec2::new(5, 10);
    if let Some(cell_entity) = grid.get_cell(chunk_coord) {
        if let Ok((cell, children)) = cells.get(cell_entity) {
            for child in children.iter() {
                // Process entity in this chunk
            }
        }
    }
}
```

### Get entity's parent cell
```rust
fn get_entity_cell(
    entity: Entity,
    child_of_query: Query<&ChildOf>,
    cells: Query<&SpatialCell>,
) -> Option<IVec2> {
    if let Ok(child_of) = child_of_query.get(entity) {
        let parent = child_of.parent();
        if let Ok(cell) = cells.get(parent) {
            return Some(cell.chunk_coord);
        }
    }
    None
}
```

## Performance Notes

### Budget Calculation
- **50 entities/tick** chosen for 10 TPS target
- Migration overhead: ~5-10% of tick budget
- Adjust BUDGET constant in `reparent_entities_to_cells` if needed

### Memory Usage
- **SpatiallyParented**: 0 bytes (ZST marker)
- **ChildOf**: 8 bytes per entity
- **Total overhead**: +8 bytes per entity (minimal)

### Query Performance
- **Chunk lookup**: O(1) via HashMap
- **Add child**: O(1) Bevy operation
- **Movement tracking**: Only Changed<TilePosition> entities

## Debugging

### Check reparenting progress
```bash
RUST_LOG=debug cargo run --bin life-simulator
```
Look for:
```
Reparenting progress: processed 50 entities this tick, 450 remaining
```

### Check movement reparenting
```bash
RUST_LOG=trace cargo run --bin life-simulator
```
Look for:
```
Updated spatial parents for 12 moved entities
```

### Verify hierarchy in tests
```rust
// Check entity has parent
assert!(app.world().entity(entity).get::<ChildOf>().is_some());

// Check parent is correct cell
let child_of = app.world().entity(entity).get::<ChildOf>().unwrap();
let parent_entity = child_of.parent();
let cell = app.world().entity(parent_entity).get::<SpatialCell>().unwrap();
assert_eq!(cell.chunk_coord, expected_chunk);
```

## Migration Timeline

### Typical Scenario (200 entities)
- **Tick 1**: 50 entities reparented (25% complete)
- **Tick 2**: 100 entities reparented (50% complete)
- **Tick 3**: 150 entities reparented (75% complete)
- **Tick 4**: 200 entities reparented (100% complete)

After migration, all entities maintain correct parent via movement tracking system.

## Common Issues

### Entity not reparenting
**Check**: Does entity have TilePosition component?
**Fix**: Ensure all spatial entities have TilePosition

### Wrong parent after movement
**Check**: Is update_spatial_parent_on_movement running?
**Fix**: System should run every frame in Update schedule

### Performance degradation
**Check**: Is budget too high?
**Fix**: Reduce BUDGET constant (try 25-30 entities/tick)

## Related Systems

- **SpatialCellGrid**: O(1) chunk → entity lookup
- **SpatialEntityIndex**: Legacy HashMap-based tracking (will be deprecated)
- **SpatialMaintenancePlugin**: Maintains HashMap index during transition

## Code Locations

- **Components**: `src/entities/spatial_cell.rs` lines 41-45
- **Systems**: `src/entities/spatial_cell.rs` lines 162-240
- **Plugin Registration**: `src/entities/mod.rs` lines 193, 270
- **Tests**: `tests/spatial_hierarchy_test.rs` tests 9-14
- **Exports**: `src/entities/mod.rs` lines 33-36
