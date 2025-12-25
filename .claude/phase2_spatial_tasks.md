# Phase 2 Spatial Optimization Task Specifications

## Task 2.1: Spatial Entity Index

**Agent**: feature-implementation-agent
**Dependencies**: Phase 1 complete
**Priority**: High (90% improvement for proximity queries)

### Files to Create
- `src/entities/spatial_index.rs`

### Implementation
```rust
use bevy::prelude::*;
use std::collections::HashMap;

const CHUNK_SIZE: i32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Herbivore,
    Predator,
    Omnivore,
}

#[derive(Resource)]
pub struct SpatialEntityIndex {
    chunks: HashMap<IVec2, Vec<(Entity, EntityType)>>,
}

impl SpatialEntityIndex {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::with_capacity(256),
        }
    }

    fn world_to_chunk(pos: IVec2) -> IVec2 {
        IVec2::new(pos.x.div_euclid(CHUNK_SIZE), pos.y.div_euclid(CHUNK_SIZE))
    }

    pub fn insert(&mut self, entity: Entity, pos: IVec2, entity_type: EntityType) {
        let chunk = Self::world_to_chunk(pos);
        self.chunks
            .entry(chunk)
            .or_insert_with(Vec::new)
            .push((entity, entity_type));
    }

    pub fn remove(&mut self, entity: Entity, old_pos: IVec2) {
        let chunk = Self::world_to_chunk(old_pos);
        if let Some(entities) = self.chunks.get_mut(&chunk) {
            entities.retain(|(e, _)| *e != entity);
        }
    }

    pub fn update(&mut self, entity: Entity, old_pos: IVec2, new_pos: IVec2, entity_type: EntityType) {
        let old_chunk = Self::world_to_chunk(old_pos);
        let new_chunk = Self::world_to_chunk(new_pos);

        if old_chunk != new_chunk {
            self.remove(entity, old_pos);
            self.insert(entity, new_pos, entity_type);
        }
    }

    pub fn entities_in_radius(
        &self,
        center: IVec2,
        radius: i32,
        filter: Option<EntityType>,
    ) -> Vec<Entity> {
        let chunk_radius = (radius + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let center_chunk = Self::world_to_chunk(center);

        let mut results = Vec::new();

        for dx in -chunk_radius..=chunk_radius {
            for dy in -chunk_radius..=chunk_radius {
                let chunk = center_chunk + IVec2::new(dx, dy);
                if let Some(entities) = self.chunks.get(&chunk) {
                    for (entity, entity_type) in entities {
                        if filter.is_none() || Some(*entity_type) == filter {
                            results.push(*entity);
                        }
                    }
                }
            }
        }

        results
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }
}

// Systems to maintain the index
pub fn spatial_index_insert_system(
    mut spatial_index: ResMut<SpatialEntityIndex>,
    added_query: Query<(Entity, &TilePosition, &EntityTypeMarker), Added<TilePosition>>,
) {
    for (entity, pos, entity_type) in added_query.iter() {
        spatial_index.insert(entity, pos.tile, entity_type.0);
    }
}

pub fn spatial_index_update_system(
    mut spatial_index: ResMut<SpatialEntityIndex>,
    changed_query: Query<(Entity, &TilePosition, &EntityTypeMarker), Changed<TilePosition>>,
    mut previous_positions: Local<HashMap<Entity, IVec2>>,
) {
    for (entity, pos, entity_type) in changed_query.iter() {
        if let Some(&old_pos) = previous_positions.get(&entity) {
            spatial_index.update(entity, old_pos, pos.tile, entity_type.0);
        } else {
            spatial_index.insert(entity, pos.tile, entity_type.0);
        }
        previous_positions.insert(entity, pos.tile);
    }
}

pub fn spatial_index_remove_system(
    mut spatial_index: ResMut<SpatialEntityIndex>,
    removed: RemovedComponents<TilePosition>,
    positions: Query<&TilePosition>,
) {
    for entity in removed.read() {
        if let Ok(pos) = positions.get(entity) {
            spatial_index.remove(entity, pos.tile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_index_insert_and_query() {
        let mut index = SpatialEntityIndex::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        index.insert(entity1, IVec2::new(5, 5), EntityType::Herbivore);
        index.insert(entity2, IVec2::new(10, 10), EntityType::Predator);

        let nearby = index.entities_in_radius(IVec2::new(5, 5), 10, None);
        assert!(nearby.contains(&entity1));
        assert!(nearby.contains(&entity2));
    }

    #[test]
    fn test_spatial_index_filter() {
        let mut index = SpatialEntityIndex::new();
        let herbivore = Entity::from_raw(1);
        let predator = Entity::from_raw(2);

        index.insert(herbivore, IVec2::new(5, 5), EntityType::Herbivore);
        index.insert(predator, IVec2::new(6, 6), EntityType::Predator);

        let only_predators = index.entities_in_radius(
            IVec2::new(5, 5),
            10,
            Some(EntityType::Predator),
        );

        assert_eq!(only_predators.len(), 1);
        assert_eq!(only_predators[0], predator);
    }

    #[test]
    fn test_spatial_index_update() {
        let mut index = SpatialEntityIndex::new();
        let entity = Entity::from_raw(1);

        index.insert(entity, IVec2::new(0, 0), EntityType::Herbivore);
        index.update(entity, IVec2::new(0, 0), IVec2::new(50, 50), EntityType::Herbivore);

        let near_origin = index.entities_in_radius(IVec2::new(0, 0), 10, None);
        assert!(!near_origin.contains(&entity));

        let near_new_pos = index.entities_in_radius(IVec2::new(50, 50), 10, None);
        assert!(near_new_pos.contains(&entity));
    }
}
```

### Integration Points
1. Add to `src/entities/mod.rs`
2. Register resource in simulation setup
3. Add update systems to simulation
4. Update fear system to use spatial queries
5. Update mate finding to use spatial queries

---

## Task 2.2: Vegetation Spatial Grid

**Agent**: feature-implementation-agent
**Dependencies**: Phase 1 complete
**Priority**: High (50% improvement for radius queries)

### Files to Create
- `src/vegetation/spatial_grid.rs`

### Implementation
Chunked storage for vegetation cells with spatial queries.

```rust
use bevy::prelude::*;
use std::collections::HashMap;
use super::resource_grid::{GrazingCell, ResourceGrid};

const CHUNK_SIZE: i32 = 16;

#[derive(Debug, Clone)]
pub struct VegetationChunk {
    cells: HashMap<IVec2, GrazingCell>,
}

impl VegetationChunk {
    pub fn new() -> Self {
        Self {
            cells: HashMap::with_capacity(256), // 16x16 max
        }
    }

    pub fn insert(&mut self, local_pos: IVec2, cell: GrazingCell) {
        self.cells.insert(local_pos, cell);
    }

    pub fn get(&self, local_pos: IVec2) -> Option<&GrazingCell> {
        self.cells.get(&local_pos)
    }

    pub fn get_mut(&mut self, local_pos: IVec2) -> Option<&mut GrazingCell> {
        self.cells.get_mut(&local_pos)
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

pub struct SpatialVegetationGrid {
    chunks: HashMap<IVec2, VegetationChunk>,
}

impl SpatialVegetationGrid {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::with_capacity(64),
        }
    }

    fn world_to_chunk(pos: IVec2) -> IVec2 {
        IVec2::new(pos.x.div_euclid(CHUNK_SIZE), pos.y.div_euclid(CHUNK_SIZE))
    }

    fn world_to_local(pos: IVec2) -> IVec2 {
        IVec2::new(pos.x.rem_euclid(CHUNK_SIZE), pos.y.rem_euclid(CHUNK_SIZE))
    }

    pub fn get_cell(&self, pos: IVec2) -> Option<&GrazingCell> {
        let chunk_pos = Self::world_to_chunk(pos);
        let local_pos = Self::world_to_local(pos);
        self.chunks.get(&chunk_pos)?.get(local_pos)
    }

    pub fn get_cell_mut(&mut self, pos: IVec2) -> Option<&mut GrazingCell> {
        let chunk_pos = Self::world_to_chunk(pos);
        let local_pos = Self::world_to_local(pos);
        self.chunks.get_mut(&chunk_pos)?.get_mut(local_pos)
    }

    pub fn insert_cell(&mut self, pos: IVec2, cell: GrazingCell) {
        let chunk_pos = Self::world_to_chunk(pos);
        let local_pos = Self::world_to_local(pos);

        let chunk = self.chunks.entry(chunk_pos).or_insert_with(VegetationChunk::new);
        chunk.insert(local_pos, cell);
    }

    pub fn find_cells_in_radius(&self, center: IVec2, radius: i32) -> Vec<(IVec2, &GrazingCell)> {
        let chunk_radius = (radius + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let center_chunk = Self::world_to_chunk(center);

        let mut results = Vec::new();

        for dx in -chunk_radius..=chunk_radius {
            for dy in -chunk_radius..=chunk_radius {
                let chunk_pos = center_chunk + IVec2::new(dx, dy);

                if let Some(chunk) = self.chunks.get(&chunk_pos) {
                    for (&local_pos, cell) in &chunk.cells {
                        let world_pos = chunk_pos * CHUNK_SIZE + local_pos;
                        let distance = center.as_vec2().distance(world_pos.as_vec2());

                        if distance <= radius as f32 {
                            results.push((world_pos, cell));
                        }
                    }
                }
            }
        }

        results
    }

    pub fn total_cells(&self) -> usize {
        self.chunks.values().map(|c| c.cell_count()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_grid_insert_and_get() {
        let mut grid = SpatialVegetationGrid::new();
        let cell = GrazingCell::default();

        grid.insert_cell(IVec2::new(5, 5), cell.clone());

        let retrieved = grid.get_cell(IVec2::new(5, 5));
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_radius_query() {
        let mut grid = SpatialVegetationGrid::new();

        // Insert cells in a pattern
        for x in 0..10 {
            for y in 0..10 {
                grid.insert_cell(IVec2::new(x, y), GrazingCell::default());
            }
        }

        let nearby = grid.find_cells_in_radius(IVec2::new(5, 5), 3);
        assert!(nearby.len() > 0);
        assert!(nearby.len() < 100); // Should be much less than all cells
    }
}
```

---

## Task 2.3: Integration & Migration

**Agent**: component-implementation-agent
**Dependencies**: Tasks 2.1 and 2.2 complete
**Priority**: Critical (tie everything together)

### Integration Steps

1. **Add EntityTypeMarker Component**
   - Mark entities as Herbivore/Predator/Omnivore
   - Required for spatial index filtering

2. **Update Fear System**
   - Replace linear search with spatial index query
   - File: `src/entities/fear.rs`

3. **Update Mate Finding**
   - Use spatial index for potential partner search
   - File: `src/entities/reproduction.rs`

4. **Migrate ResourceGrid to SpatialGrid** (Optional - can be Phase 3)
   - Wrapper around SpatialVegetationGrid
   - Maintain backwards compatibility

5. **System Registration**
   - Add spatial index systems to simulation
   - Ensure proper ordering

### Testing
- Run all integration tests
- Verify fear system still works
- Verify mating system still works
- Run performance benchmarks

---

**Next Steps After Phase 2:**
- Phase 3: LOD system integration
- Phase 4: Final profiling and optimization
- Phase 5: Comprehensive benchmarking and documentation
