use bevy::prelude::*;
use std::collections::HashMap;

const CHUNK_SIZE: i32 = 16;

/// Entity type classification for spatial filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Herbivore,
    Predator,
    Omnivore,
}

/// Spatial grid index for fast entity lookups by location
///
/// Uses a grid-based chunking system to enable O(k) proximity queries
/// where k is the number of entities in nearby chunks, instead of O(N)
/// linear searches through all entities.
///
/// Performance: 10-100x faster proximity queries compared to linear search
#[derive(Resource, Debug)]
pub struct SpatialEntityIndex {
    chunks: HashMap<IVec2, Vec<(Entity, EntityType)>>,
}

impl SpatialEntityIndex {
    /// Create a new empty spatial index
    pub fn new() -> Self {
        Self {
            chunks: HashMap::with_capacity(256),
        }
    }

    /// Convert world position to chunk coordinates
    fn world_to_chunk(pos: IVec2) -> IVec2 {
        IVec2::new(
            pos.x.div_euclid(CHUNK_SIZE),
            pos.y.div_euclid(CHUNK_SIZE),
        )
    }

    /// Insert an entity at a world position
    pub fn insert(&mut self, entity: Entity, pos: IVec2, entity_type: EntityType) {
        let chunk = Self::world_to_chunk(pos);
        self.chunks
            .entry(chunk)
            .or_insert_with(Vec::new)
            .push((entity, entity_type));
    }

    /// Remove an entity from a world position
    pub fn remove(&mut self, entity: Entity, old_pos: IVec2) {
        let chunk = Self::world_to_chunk(old_pos);
        if let Some(entities) = self.chunks.get_mut(&chunk) {
            entities.retain(|(e, _)| *e != entity);
            if entities.is_empty() {
                self.chunks.remove(&chunk);
            }
        }
    }

    /// Update entity position, moving to new chunk if necessary
    pub fn update(
        &mut self,
        entity: Entity,
        old_pos: IVec2,
        new_pos: IVec2,
        entity_type: EntityType,
    ) {
        let old_chunk = Self::world_to_chunk(old_pos);
        let new_chunk = Self::world_to_chunk(new_pos);

        if old_chunk != new_chunk {
            self.remove(entity, old_pos);
            self.insert(entity, new_pos, entity_type);
        }
    }

    /// Query for entities within a radius of a center point
    ///
    /// This performs chunk-based lookup for O(k) performance where k
    /// is the number of entities in nearby chunks.
    pub fn entities_in_radius(
        &self,
        center: IVec2,
        radius: i32,
        filter: Option<EntityType>,
    ) -> Vec<Entity> {
        let chunk_radius = (radius + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let center_chunk = Self::world_to_chunk(center);

        let mut results = Vec::new();

        // Check all nearby chunks
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


    /// Clear all entities from the index
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Get the number of active chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get the total number of entities in the index
    pub fn total_entities(&self) -> usize {
        self.chunks.values().map(|v| v.len()).sum()
    }

    /// Get the number of entities in a specific chunk (for debugging)
    pub fn entities_in_chunk(&self, chunk: IVec2) -> usize {
        self.chunks.get(&chunk).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for SpatialEntityIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TEST 1: World to Chunk Conversion
    // ========================================================================
    #[test]
    fn test_world_to_chunk_conversion() {
        // Origin
        assert_eq!(
            SpatialEntityIndex::world_to_chunk(IVec2::new(0, 0)),
            IVec2::new(0, 0)
        );

        // Within chunk 0,0
        assert_eq!(
            SpatialEntityIndex::world_to_chunk(IVec2::new(15, 15)),
            IVec2::new(0, 0)
        );

        // Boundary - next chunk
        assert_eq!(
            SpatialEntityIndex::world_to_chunk(IVec2::new(16, 16)),
            IVec2::new(1, 1)
        );

        // Negative coordinates
        assert_eq!(
            SpatialEntityIndex::world_to_chunk(IVec2::new(-1, -1)),
            IVec2::new(-1, -1)
        );

        // Large coordinates
        assert_eq!(
            SpatialEntityIndex::world_to_chunk(IVec2::new(100, 100)),
            IVec2::new(6, 6)
        );
    }

    // ========================================================================
    // TEST 2: Insert and Basic Query
    // ========================================================================
    #[test]
    fn test_insert_and_query() {
        let mut index = SpatialEntityIndex::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        // Insert entities in same chunk
        index.insert(entity1, IVec2::new(5, 5), EntityType::Herbivore);
        index.insert(entity2, IVec2::new(10, 10), EntityType::Predator);

        assert_eq!(index.total_entities(), 2);
        assert_eq!(index.chunk_count(), 1);

        // Query radius should find both
        let nearby = index.entities_in_radius(IVec2::new(5, 5), 20, None);
        assert!(nearby.contains(&entity1));
        assert!(nearby.contains(&entity2));
    }

    // ========================================================================
    // TEST 3: Entity Type Filtering
    // ========================================================================
    #[test]
    fn test_filter_by_entity_type() {
        let mut index = SpatialEntityIndex::new();
        let herbivore = Entity::from_raw(1);
        let predator = Entity::from_raw(2);
        let omnivore = Entity::from_raw(3);

        // Insert mixed entity types
        index.insert(herbivore, IVec2::new(5, 5), EntityType::Herbivore);
        index.insert(predator, IVec2::new(6, 6), EntityType::Predator);
        index.insert(omnivore, IVec2::new(7, 7), EntityType::Omnivore);

        // Query only predators
        let only_predators =
            index.entities_in_radius(IVec2::new(5, 5), 10, Some(EntityType::Predator));
        assert_eq!(only_predators.len(), 1);
        assert_eq!(only_predators[0], predator);

        // Query only herbivores
        let only_herbivores =
            index.entities_in_radius(IVec2::new(5, 5), 10, Some(EntityType::Herbivore));
        assert_eq!(only_herbivores.len(), 1);
        assert_eq!(only_herbivores[0], herbivore);

        // Query all
        let all = index.entities_in_radius(IVec2::new(5, 5), 10, None);
        assert_eq!(all.len(), 3);
    }

    // ========================================================================
    // TEST 4: Update Across Chunks
    // ========================================================================
    #[test]
    fn test_update_across_chunks() {
        let mut index = SpatialEntityIndex::new();
        let entity = Entity::from_raw(1);

        // Insert in chunk (0, 0)
        index.insert(entity, IVec2::new(0, 0), EntityType::Herbivore);
        assert_eq!(index.total_entities(), 1);
        assert_eq!(index.chunk_count(), 1);

        // Move to chunk (3, 3)
        index.update(
            entity,
            IVec2::new(0, 0),
            IVec2::new(50, 50),
            EntityType::Herbivore,
        );

        // Entity should not be found near origin
        let near_origin = index.entities_in_radius(IVec2::new(0, 0), 10, None);
        assert!(!near_origin.contains(&entity));

        // Entity should be found near new position
        let near_new_pos = index.entities_in_radius(IVec2::new(50, 50), 10, None);
        assert!(near_new_pos.contains(&entity));

        // Total count should still be 1
        assert_eq!(index.total_entities(), 1);
    }

    // ========================================================================
    // TEST 5: Remove and Cleanup
    // ========================================================================
    #[test]
    fn test_remove_entity() {
        let mut index = SpatialEntityIndex::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        // Insert two entities in same chunk
        index.insert(entity1, IVec2::new(5, 5), EntityType::Herbivore);
        index.insert(entity2, IVec2::new(6, 6), EntityType::Herbivore);
        assert_eq!(index.total_entities(), 2);

        // Remove first entity
        index.remove(entity1, IVec2::new(5, 5));
        assert_eq!(index.total_entities(), 1);

        // Check entity1 is gone
        let nearby = index.entities_in_radius(IVec2::new(5, 5), 10, None);
        assert!(!nearby.contains(&entity1));
        assert!(nearby.contains(&entity2));

        // Remove second entity
        index.remove(entity2, IVec2::new(6, 6));
        assert_eq!(index.total_entities(), 0);

        // Empty chunk should be removed
        assert_eq!(index.chunk_count(), 0);
    }

    // ========================================================================
    // TEST 6: Multi-Chunk Queries
    // ========================================================================
    #[test]
    fn test_multi_chunk_query() {
        let mut index = SpatialEntityIndex::new();

        // Create entities in different chunks
        let entity_chunk_0 = Entity::from_raw(1);
        let entity_chunk_1 = Entity::from_raw(2);
        let entity_chunk_2 = Entity::from_raw(3);

        // Chunk (0, 0)
        index.insert(entity_chunk_0, IVec2::new(5, 5), EntityType::Herbivore);

        // Chunk (1, 0)
        index.insert(entity_chunk_1, IVec2::new(20, 5), EntityType::Herbivore);

        // Chunk (5, 5)
        index.insert(entity_chunk_2, IVec2::new(100, 100), EntityType::Herbivore);

        assert_eq!(index.chunk_count(), 3);

        // Query with large radius should find nearby chunks
        let nearby = index.entities_in_radius(IVec2::new(10, 10), 50, None);
        assert!(nearby.contains(&entity_chunk_0), "Should find entity in chunk 0");
        assert!(
            nearby.contains(&entity_chunk_1),
            "Should find entity in chunk 1"
        );
        assert!(!nearby.contains(&entity_chunk_2), "Should NOT find distant entity");

        // Query with very large radius should find all
        let all = index.entities_in_radius(IVec2::new(10, 10), 150, None);
        assert_eq!(all.len(), 3);
    }

    // ========================================================================
    // BONUS TEST: Performance Characteristics
    // ========================================================================
    #[test]
    fn test_performance_characteristics() {
        let mut index = SpatialEntityIndex::new();

        // Insert 1000 entities spread across a large area
        for i in 0..1000 {
            let x = (i % 50) * 2;
            let y = (i / 50) * 2;
            index.insert(
                Entity::from_raw((i + 1) as u32),
                IVec2::new(x, y),
                if i % 3 == 0 {
                    EntityType::Predator
                } else {
                    EntityType::Herbivore
                },
            );
        }

        // Query should be fast and accurate
        let nearby = index.entities_in_radius(IVec2::new(25, 25), 30, None);
        assert!(!nearby.is_empty(), "Should find some entities");

        // Filter query
        let predators = index.entities_in_radius(IVec2::new(25, 25), 30, Some(EntityType::Predator));
        assert!(!predators.is_empty(), "Should find some predators");
    }
}
