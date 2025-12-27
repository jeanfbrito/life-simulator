# Children Component Query - Quick Reference

## Bevy 0.16 Hierarchy Patterns

### Basic Children Query
```rust
use bevy::prelude::*;

// Query for children of spatial cells
fn my_system(cells: Query<&Children, With<SpatialCell>>) {
    for children in cells.iter() {
        // Children.iter() yields Entity directly (not &Entity)
        for child_entity in children.iter() {
            // Process child entity
        }
    }
}
```

### Spatial Radius Query Helper
```rust
use crate::entities::{entities_in_radius_via_children, SpatialCellGrid, SpatialCell};

fn my_spatial_system(
    grid: Res<SpatialCellGrid>,
    cells: Query<&Children, With<SpatialCell>>,
) {
    let center = IVec2::new(0, 0);
    let radius = 32.0;

    // Get all entities within radius
    let nearby = entities_in_radius_via_children(
        &grid,
        &cells,
        center,
        radius,
    );

    // Process nearby entities
    for entity in nearby {
        // ...
    }
}
```

### Fear System Pattern
```rust
pub fn predator_proximity_system(
    mut prey_query: Query<(Entity, &TilePosition, &mut FearState), With<Herbivore>>,
    predator_query: Query<Entity, Or<(With<Wolf>, With<Fox>, With<Bear>)>>,
    grid: Res<SpatialCellGrid>,
    cells: Query<&Children, With<SpatialCell>>,
) {
    for (entity, pos, mut fear_state) in prey_query.iter_mut() {
        // Get nearby entities
        let nearby = entities_in_radius_via_children(
            &grid,
            &cells,
            pos.tile,
            FEAR_RADIUS as f32,
        );

        // Filter for predators
        let predator_count = nearby
            .iter()
            .filter(|&&e| predator_query.contains(e))
            .count() as u32;

        // Update fear state
        if predator_count > 0 {
            fear_state.apply_fear_stimulus(predator_count);
        }
    }
}
```

### Mate Matching Pattern
```rust
pub fn mate_matching_system<M: Component, const EMOJI: char>(
    commands: &mut Commands,
    animals: &Query<(Entity, &TilePosition, ...), With<M>>,
    grid: &SpatialCellGrid,
    cells: &Query<&Children, With<SpatialCell>>,
    current_tick: u64,
) {
    // Collect eligible females
    let females: Vec<(Entity, IVec2, ...)> = animals
        .iter()
        .filter(|(_, _, _, _, sex_opt, ..)| sex_opt == Some(&Sex::Female))
        .collect();

    for (female_e, fpos, fcfg) in females {
        // Find nearby males using spatial query
        let nearby = entities_in_radius_via_children(
            grid,
            cells,
            fpos,
            fcfg.mating_search_radius as f32,
        );

        // Check nearby entities for compatible males
        for nearby_entity in nearby {
            if let Ok((male_e, male_pos, male_sex, ..)) = animals.get(nearby_entity) {
                if male_sex == Sex::Male {
                    // Form mating pair
                    commands.entity(female_e).insert(MatingIntent {
                        partner: male_e,
                        meeting_tile: fpos,
                        duration_ticks: fcfg.mating_duration_ticks,
                    });
                    break;
                }
            }
        }
    }
}
```

## Performance Characteristics

### Time Complexity
- **O(k)** where k = entities in nearby chunks
- Typical values: k = 2-5 for fear radius, 10-50 for mate search

### Chunk Calculation
```rust
let radius_chunks = (radius / grid.chunk_size() as f32).ceil() as i32;
let center_chunk = grid.chunk_coord_for_position(center);

// Query chunks from (center - radius_chunks) to (center + radius_chunks)
for dx in -radius_chunks..=radius_chunks {
    for dy in -radius_chunks..=radius_chunks {
        let chunk = center_chunk + IVec2::new(dx, dy);
        // Process chunk...
    }
}
```

### Memory Usage
- Vec allocation for results
- Typical size: 2-50 entities depending on density
- No persistent allocations

## Migration Checklist

### From HashMap SpatialEntityIndex
1. Update imports:
   ```rust
   // Remove
   use crate::entities::{SpatialEntityIndex, SpatialEntityType};

   // Add
   use crate::entities::{entities_in_radius_via_children, SpatialCell, SpatialCellGrid};
   ```

2. Update system parameters:
   ```rust
   // Remove
   spatial_index: Res<SpatialEntityIndex>,

   // Add
   grid: Res<SpatialCellGrid>,
   cells: Query<&Children, With<SpatialCell>>,
   ```

3. Update query calls:
   ```rust
   // Old
   let nearby = spatial_index.entities_in_radius(
       pos.tile,
       radius,
       Some(SpatialEntityType::Predator),
   );

   // New
   let nearby = entities_in_radius_via_children(
       &grid,
       &cells,
       pos.tile,
       radius as f32,
   );
   ```

4. Add filtering if needed:
   ```rust
   // HashMap version had type filtering built-in
   // Children version returns all entities, filter manually
   let predators: Vec<_> = nearby
       .iter()
       .filter(|&&e| predator_query.contains(e))
       .collect();
   ```

## Common Patterns

### Get All Children in Chunk
```rust
if let Some(cell_entity) = grid.get_cell(chunk_coord) {
    if let Ok(children) = cells.get(cell_entity) {
        for child in children.iter() {
            // Process child
        }
    }
}
```

### Count Entities in Radius
```rust
let count = entities_in_radius_via_children(&grid, &cells, center, radius).len();
```

### Filter by Component
```rust
let nearby = entities_in_radius_via_children(&grid, &cells, center, radius);
let filtered: Vec<_> = nearby
    .iter()
    .filter(|&&e| query.contains(e))
    .collect();
```

## Debugging Tips

### Verify Chunk Coverage
```rust
let radius_chunks = (radius / 16.0).ceil() as i32;
println!("Querying {}x{} chunks for radius {}",
    radius_chunks * 2 + 1,
    radius_chunks * 2 + 1,
    radius
);
```

### Check Entity Parenting
```rust
// Verify entity is parented to spatial cell
fn check_parenting(
    entity: Entity,
    parent_query: Query<&Parent>,
    cell_query: Query<&SpatialCell>,
) {
    if let Ok(parent) = parent_query.get(entity) {
        if let Ok(cell) = cell_query.get(parent.get()) {
            println!("Entity {:?} is in chunk {:?}", entity, cell.chunk_coord);
        }
    }
}
```

### Measure Query Performance
```rust
use std::time::Instant;

let start = Instant::now();
let nearby = entities_in_radius_via_children(&grid, &cells, center, radius);
let elapsed = start.elapsed();
println!("Query found {} entities in {:?}", nearby.len(), elapsed);
```

## References

- Bevy Hierarchy Docs: https://docs.rs/bevy/0.16/bevy/hierarchy/
- Children Component: `bevy::prelude::Children`
- Parent Component: `bevy::prelude::Parent`
- Implementation: `src/entities/spatial_cell.rs`
- Tests: `tests/spatial_children_query_test.rs`
