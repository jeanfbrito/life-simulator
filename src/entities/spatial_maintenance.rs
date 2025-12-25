/// Spatial index maintenance system for keeping SpatialEntityIndex in sync with ECS
///
/// This module provides systems to maintain the SpatialEntityIndex by tracking:
/// - New entities being spawned
/// - Entities moving between locations
/// - Entities being despawned
///
/// The spatial index must stay synchronized for:
/// - O(k) proximity queries used by fear system, mate finding, etc.
/// - Efficient neighbor searches for predator-prey dynamics
/// - Vegetation query optimization

use bevy::prelude::*;
use crate::entities::{
    spatial_index::{EntityType, SpatialEntityIndex},
    TilePosition,
    entity_types::{Bear, Fox, Wolf, Raccoon},
};
use std::collections::HashMap;

/// Tracks previous positions of entities for detecting moves and removals
#[derive(Resource, Default, Debug)]
pub struct EntityPositionCache {
    /// Map of entity to its previous position and type
    previous_positions: HashMap<Entity, (IVec2, EntityType)>,
}

impl EntityPositionCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            previous_positions: HashMap::with_capacity(1024),
        }
    }

    /// Get the cached position of an entity
    pub fn get_position(&self, entity: Entity) -> Option<(IVec2, EntityType)> {
        self.previous_positions.get(&entity).copied()
    }

    /// Update the cached position of an entity
    pub fn update_position(&mut self, entity: Entity, pos: IVec2, entity_type: EntityType) {
        self.previous_positions.insert(entity, (pos, entity_type));
    }

    /// Remove an entity from the cache
    pub fn remove_entity(&mut self, entity: Entity) {
        self.previous_positions.remove(&entity);
    }

    /// Get all tracked entities
    pub fn all_entities(&self) -> Vec<Entity> {
        self.previous_positions.keys().copied().collect()
    }
}

/// Helper function to classify entity type from Bevy components
///
/// Queries the world to determine if an entity is a Herbivore, Predator, or Omnivore.
/// This is used when inserting or updating entities in the spatial index.
pub fn classify_entity_type(entity: Entity, world: &World) -> EntityType {
    // Check marker components to determine entity type
    // Predators: Fox, Wolf
    if world.get::<Fox>(entity).is_some() || world.get::<Wolf>(entity).is_some() {
        EntityType::Predator
    }
    // Omnivore: Bear, Raccoon
    else if world.get::<Bear>(entity).is_some() || world.get::<Raccoon>(entity).is_some() {
        EntityType::Omnivore
    }
    // Herbivores and others default to Herbivore
    else {
        EntityType::Herbivore
    }
}

/// System to add newly spawned entities to the spatial index
///
/// This system runs in the Update schedule and:
/// 1. Finds entities with TilePosition that are newly added
/// 2. Inserts them into the SpatialEntityIndex
/// 3. Caches their positions for future move/despawn detection
pub fn maintain_spatial_entity_index_insertions(
    mut spatial_index: ResMut<SpatialEntityIndex>,
    mut position_cache: ResMut<EntityPositionCache>,
    new_entities: Query<(Entity, &TilePosition), Added<TilePosition>>,
    world: &World,
) {
    for (entity, tile_pos) in new_entities.iter() {
        let pos = tile_pos.tile;
        let entity_type = classify_entity_type(entity, world);

        // Insert into spatial index
        spatial_index.insert(entity, pos, entity_type);

        // Cache position for future tracking
        position_cache.update_position(entity, pos, entity_type);
    }
}

/// System to update positions of moving entities in the spatial index
///
/// This system runs in the Update schedule and:
/// 1. Finds entities whose TilePosition has changed
/// 2. Updates them in the SpatialEntityIndex using the cached old position
/// 3. Updates the position cache with the new position
pub fn maintain_spatial_entity_index_updates(
    mut spatial_index: ResMut<SpatialEntityIndex>,
    mut position_cache: ResMut<EntityPositionCache>,
    moved_entities: Query<(Entity, &TilePosition), Changed<TilePosition>>,
    world: &World,
) {
    for (entity, tile_pos) in moved_entities.iter() {
        let new_pos = tile_pos.tile;
        let entity_type = classify_entity_type(entity, world);

        // Get previous position from cache
        if let Some((old_pos, _)) = position_cache.get_position(entity) {
            // Update position in spatial index
            spatial_index.update(entity, old_pos, new_pos, entity_type);

            // Update cache with new position
            position_cache.update_position(entity, new_pos, entity_type);
        } else {
            // If not in cache, this is a newly positioned entity - insert it
            spatial_index.insert(entity, new_pos, entity_type);
            position_cache.update_position(entity, new_pos, entity_type);
        }
    }
}

/// System to remove despawned entities from the spatial index
///
/// This system runs at the end of each frame and:
/// 1. Finds entities that were in the position cache but are no longer in the world
/// 2. Removes them from the SpatialEntityIndex
/// 3. Removes them from the position cache
pub fn maintain_spatial_entity_index_removals(
    mut spatial_index: ResMut<SpatialEntityIndex>,
    mut position_cache: ResMut<EntityPositionCache>,
    query: Query<Entity>, // Query all entities that still exist
) {
    // Get set of existing entities
    let existing_entities: std::collections::HashSet<Entity> = query.iter().collect();

    // Find entities in cache that no longer exist
    let to_remove: Vec<Entity> = position_cache
        .all_entities()
        .into_iter()
        .filter(|e| !existing_entities.contains(e))
        .collect();

    // Remove them from both spatial index and cache
    for entity in to_remove {
        if let Some((pos, _)) = position_cache.get_position(entity) {
            spatial_index.remove(entity, pos);
        }
        position_cache.remove_entity(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests validate the maintenance system logic in isolation.
    // Integration tests with Bevy's full ECS are in the main test suite.

    #[test]
    fn test_entity_position_cache_creation() {
        let cache = EntityPositionCache::new();
        assert_eq!(cache.all_entities().len(), 0);
    }

    #[test]
    fn test_entity_position_cache_update_and_get() {
        let mut cache = EntityPositionCache::new();
        let entity = Entity::from_raw(1);
        let pos = IVec2::new(5, 10);
        let etype = EntityType::Herbivore;

        cache.update_position(entity, pos, etype);

        let retrieved = cache.get_position(entity);
        assert_eq!(retrieved, Some((pos, etype)));
    }

    #[test]
    fn test_entity_position_cache_remove() {
        let mut cache = EntityPositionCache::new();
        let entity = Entity::from_raw(1);
        let pos = IVec2::new(5, 10);

        cache.update_position(entity, pos, EntityType::Herbivore);
        assert_eq!(cache.all_entities().len(), 1);

        cache.remove_entity(entity);
        assert_eq!(cache.all_entities().len(), 0);
        assert_eq!(cache.get_position(entity), None);
    }

    #[test]
    fn test_entity_position_cache_multiple_entities() {
        let mut cache = EntityPositionCache::new();

        for i in 0..10 {
            let entity = Entity::from_raw(i + 1);
            let pos = IVec2::new(i as i32, i as i32 * 2);
            cache.update_position(entity, pos, EntityType::Herbivore);
        }

        assert_eq!(cache.all_entities().len(), 10);
    }

    #[test]
    fn test_entity_position_cache_update_overwrites() {
        let mut cache = EntityPositionCache::new();
        let entity = Entity::from_raw(1);

        cache.update_position(entity, IVec2::new(5, 5), EntityType::Herbivore);
        cache.update_position(entity, IVec2::new(10, 10), EntityType::Predator);

        let retrieved = cache.get_position(entity);
        assert_eq!(retrieved, Some((IVec2::new(10, 10), EntityType::Predator)));
    }
}
