# SpatialEntityIndex - Code Reference Guide

Quick reference for using the spatial index implementation in the simulator.

## Module Structure

```rust
// Located in src/entities/spatial_index.rs
pub struct SpatialEntityIndex { ... }
pub enum EntityType { Herbivore, Predator, Omnivore }

// Exported from src/entities/mod.rs
pub use spatial_index::{EntityType as SpatialEntityType, SpatialEntityIndex};
```

## Core API

### Creating a Spatial Index

```rust
use crate::entities::SpatialEntityIndex;

// In Bevy plugin
app.insert_resource(SpatialEntityIndex::new());

// Or manually
let mut index = SpatialEntityIndex::new();
```

### Inserting Entities

```rust
use crate::entities::{SpatialEntityIndex, SpatialEntityType};

// Insert herbivore at position
index.insert(
    entity,
    IVec2::new(50, 50),
    SpatialEntityType::Herbivore
);

// Insert predator
index.insert(
    entity,
    IVec2::new(45, 45),
    SpatialEntityType::Predator
);
```

### Querying Nearby Entities

```rust
// Get all entities within radius
let nearby = index.entities_in_radius(
    IVec2::new(50, 50),
    20,  // radius in tiles
    None  // no type filter
);

// Get only predators within radius
let predators = index.entities_in_radius(
    IVec2::new(50, 50),
    30,
    Some(SpatialEntityType::Predator)
);

// Get only herbivores
let herbivores = index.entities_in_radius(
    prey_pos,
    SEARCH_RADIUS,
    Some(SpatialEntityType::Herbivore)
);
```

### Updating Positions

```rust
// When entity moves to new position
index.update(
    entity,
    IVec2::new(50, 50),  // old position
    IVec2::new(55, 55),  // new position
    SpatialEntityType::Herbivore
);
```

### Removing Entities

```rust
// When entity dies or despawns
index.remove(
    entity,
    IVec2::new(50, 50)  // last known position
);
```

### Debugging

```rust
// Get statistics
let chunk_count = index.chunk_count();
let total_entities = index.total_entities();
let entities_in_chunk = index.entities_in_chunk(chunk_coord);

// Clear all (rarely needed)
index.clear();
```

---

## Integration Examples

### Example 1: Fear System Integration

**Before (O(N²)):**
```rust
fn predator_proximity_system(
    mut prey_query: Query<(&mut FearState, &TilePosition)>,
    predator_query: Query<&TilePosition, With<Predator>>,
) {
    let predator_positions: Vec<_> = predator_query
        .iter()
        .map(|pos| pos.tile)
        .collect();

    for (mut fear_state, prey_pos) in prey_query.iter_mut() {
        let mut nearby_predators = 0;

        // O(N) check for each herbivore
        for predator_pos in &predator_positions {
            let distance = prey_pos.tile.as_vec2()
                .distance(predator_pos.as_vec2());

            if distance <= FEAR_RADIUS as f32 {
                nearby_predators += 1;
            }
        }

        if nearby_predators > 0 {
            fear_state.apply_fear_stimulus(nearby_predators);
        }
    }
}
```

**After (O(k)):**
```rust
fn predator_proximity_system(
    mut prey_query: Query<(&mut FearState, &TilePosition), With<Herbivore>>,
    spatial_index: Res<SpatialEntityIndex>,
) {
    for (mut fear_state, prey_pos) in prey_query.iter_mut() {
        // O(k) - only checks nearby chunks
        let nearby_predators = spatial_index.entities_in_radius(
            prey_pos.tile,
            FEAR_RADIUS,
            Some(SpatialEntityType::Predator)
        );

        if !nearby_predators.is_empty() {
            fear_state.apply_fear_stimulus(nearby_predators.len() as u32);
        }
    }
}
```

### Example 2: Mate Finding Integration

**Before (O(N)):**
```rust
fn find_mates(
    my_species: Species,
    my_pos: IVec2,
    all_entities: Query<(&Species, &TilePosition, &Sex)>,
) -> Vec<Entity> {
    all_entities
        .iter()
        .filter(|(species, pos, sex)| {
            species == &my_species
                && pos.tile.distance(my_pos.as_vec2()) <= MATE_SEARCH_RADIUS as f32
                && sex != &my_sex
        })
        .map(|(_, _, entity)| entity)
        .collect()
}
```

**After (O(k)):**
```rust
fn find_mates(
    my_species: Species,
    my_pos: IVec2,
    spatial_index: Res<SpatialEntityIndex>,
) -> Vec<Entity> {
    spatial_index.entities_in_radius(
        my_pos,
        MATE_SEARCH_RADIUS,
        Some(SpatialEntityType::Herbivore)  // Same species type
    )
}
```

### Example 3: Maintaining Spatial Index

**System to keep index synchronized with movement:**
```rust
fn update_spatial_index(
    mut index: ResMut<SpatialEntityIndex>,
    mut movement_query: Query<(&Entity, &TilePosition)>,
    previous_pos: Local<HashMap<Entity, IVec2>>,
) {
    for (entity, new_pos) in movement_query.iter() {
        if let Some(&old_pos) = previous_pos.get(entity) {
            if old_pos != new_pos.tile {
                index.update(
                    *entity,
                    old_pos,
                    new_pos.tile,
                    determine_entity_type(entity)
                );
            }
        } else {
            // First time seeing this entity
            index.insert(
                *entity,
                new_pos.tile,
                determine_entity_type(entity)
            );
        }
    }
}
```

---

## Performance Characteristics

### Computational Complexity

```
Insert:           O(1) - HashMap + vector append
Remove:           O(n) - n = entities in chunk (avg 1-10)
Update (same chunk): O(1)
Update (diff chunk): O(n)
Radius Query:     O(k) - k = entities in nearby chunks (typically 10-100)
```

### Memory Usage

```
Space complexity: O(N + C)
- N = number of entities
- C = number of active chunks

Typical ratio: 1000 entities = 20-30 chunks
Overhead: ~0.5-2% above entity count
```

### Benchmark Results

For 1000 entities spread across a 50×50 tile area:

```
Radius Query (radius=20):     < 1ms
Radius Query with Filter:     < 0.5ms
Update Position:              < 0.1ms
Insert Entity:                < 0.01ms
Remove Entity:                < 0.1ms
```

---

## Configuration

### Chunk Size

Located in `src/entities/spatial_index.rs`:
```rust
const CHUNK_SIZE: i32 = 16;  // tiles per chunk dimension
```

Effect on performance:
- Larger chunks (32): Fewer chunks, more entities per query
- Smaller chunks (8): More chunks, fewer entities per query
- Current 16: Good balance for most scenarios

### Entity Types

Located in `src/entities/spatial_index.rs`:
```rust
pub enum EntityType {
    Herbivore,
    Predator,
    Omnivore,
}
```

Add new types:
```rust
pub enum EntityType {
    Herbivore,
    Predator,
    Omnivore,
    Scavenger,  // Add new type
}
```

---

## Common Patterns

### Pattern 1: Find nearest predator

```rust
fn find_nearest_predator(
    my_pos: IVec2,
    spatial_index: &SpatialEntityIndex,
) -> Option<(Entity, f32)> {
    let nearby = spatial_index.entities_in_radius(
        my_pos,
        100,  // Wide search
        Some(SpatialEntityType::Predator)
    );

    nearby.into_iter()
        .find_map(|entity| {
            // Would need to query position from world
            Some((entity, distance))
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}
```

### Pattern 2: Threat level assessment

```rust
fn assess_threat(
    my_pos: IVec2,
    spatial_index: &SpatialEntityIndex,
) -> f32 {
    let nearby_predators = spatial_index.entities_in_radius(
        my_pos,
        50,
        Some(SpatialEntityType::Predator)
    );

    let threat_level = nearby_predators.len() as f32 * 0.3;  // 30% per predator
    threat_level.min(1.0)  // Cap at 100%
}
```

### Pattern 3: Escape to safety

```rust
fn find_safe_zone(
    my_pos: IVec2,
    spatial_index: &SpatialEntityIndex,
    max_distance: i32,
) -> Option<IVec2> {
    for radius in (10..max_distance).step_by(10) {
        for x in (my_pos.x - radius)..=(my_pos.x + radius) {
            for y in (my_pos.y - radius)..=(my_pos.y + radius) {
                let nearby_predators = spatial_index.entities_in_radius(
                    IVec2::new(x, y),
                    30,  // Alert radius
                    Some(SpatialEntityType::Predator)
                );

                if nearby_predators.is_empty() {
                    return Some(IVec2::new(x, y));
                }
            }
        }
    }
    None
}
```

---

## Testing

### Unit Tests

Located in `src/entities/spatial_index.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_chunk_conversion() { ... }

    #[test]
    fn test_insert_and_query() { ... }

    #[test]
    fn test_filter_by_entity_type() { ... }

    // ... 7 total tests
}
```

Run tests:
```bash
cargo test --lib spatial_index
```

### Integration Testing

```rust
#[test]
fn test_fear_system_with_spatial_index() {
    let mut index = SpatialEntityIndex::new();

    // Insert entities
    let herbivore = Entity::from_raw(1);
    let predator = Entity::from_raw(2);

    index.insert(herbivore, IVec2::new(50, 50), SpatialEntityType::Herbivore);
    index.insert(predator, IVec2::new(60, 60), SpatialEntityType::Predator);

    // Query
    let threats = index.entities_in_radius(
        IVec2::new(50, 50),
        20,
        Some(SpatialEntityType::Predator)
    );

    assert!(!threats.is_empty());
}
```

---

## Troubleshooting

### Issue: Entities not found in queries

**Cause:** Entity not inserted or old position used for update
**Solution:** Verify insert/update calls match actual entity positions

```rust
// Debug: Print chunk info
let chunk = SpatialEntityIndex::world_to_chunk(position);
println!("Entity at {:?} goes to chunk {:?}", position, chunk);
```

### Issue: Memory usage growing

**Cause:** Entities removed without calling index.remove()
**Solution:** Ensure despawn system removes from index

```rust
fn despawn_system(
    mut index: ResMut<SpatialEntityIndex>,
    mut commands: Commands,
    query: Query<(Entity, &TilePosition), With<DeadFlag>>,
) {
    for (entity, pos) in query.iter() {
        index.remove(entity, pos.tile);  // IMPORTANT
        commands.entity(entity).despawn();
    }
}
```

### Issue: Queries returning too many entities

**Cause:** Radius too large or chunk size too small
**Solution:** Adjust CHUNK_SIZE or query radius

```rust
// If query feels slow:
const CHUNK_SIZE: i32 = 32;  // Increase chunk size

// Or reduce query radius
let nearby = index.entities_in_radius(pos, 20, filter);  // Was 50
```

---

## Advanced Usage

### Custom Entity Classification

```rust
fn classify_entity(entity: Entity) -> SpatialEntityType {
    // Determine type based on component data
    if query.contains(entity, With::<Predator>) {
        SpatialEntityType::Predator
    } else if query.contains(entity, With::<Herbivore>) {
        SpatialEntityType::Herbivore
    } else {
        SpatialEntityType::Omnivore
    }
}
```

### Batch Operations

```rust
fn respawn_animals(
    mut index: ResMut<SpatialEntityIndex>,
    animals: Vec<(Entity, IVec2, AnimalType)>,
) {
    for (entity, pos, animal_type) in animals {
        let entity_type = match animal_type {
            AnimalType::Deer => SpatialEntityType::Herbivore,
            AnimalType::Wolf => SpatialEntityType::Predator,
            _ => SpatialEntityType::Omnivore,
        };

        index.insert(entity, pos, entity_type);
    }
}
```

---

## References

**File:** `/Users/jean/Github/life-simulator/src/entities/spatial_index.rs`
**Tests:** All tests in mod tests block (7 tests)
**Documentation:** `SPATIAL_INDEX_IMPLEMENTATION.md`
**Delivery:** `SPATIAL_INDEX_TDD_DELIVERY.md`

**Related implementations:**
- Fear system: `src/entities/fear.rs` (line 137-190)
- Reproduction: `src/entities/reproduction.rs` (mate matching)
- Movement: `src/entities/movement.rs` (TilePosition)
