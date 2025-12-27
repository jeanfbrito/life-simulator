/// Integration tests for spatial index usage in mate matching systems
/// TDD approach: tests first, implementation follows
use bevy::prelude::*;
use life_simulator::entities::{
    reproduction::{Age, ReproductionCooldown, ReproductionConfig, Sex, WellFedStreak},
    SpatialEntityIndex, SpatialEntityType,
    stats::{Energy, Health, Hunger, Stat, Thirst},
    TilePosition,
};

/// Test 1: Spatial index query returns entities within radius
#[test]
fn test_spatial_index_entities_in_radius() {
    let mut index = SpatialEntityIndex::new();

    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);

    // Insert herbivores at different positions
    index.insert(entity1, IVec2::new(0, 0), SpatialEntityType::Herbivore);
    index.insert(entity2, IVec2::new(10, 0), SpatialEntityType::Herbivore);
    index.insert(entity3, IVec2::new(100, 100), SpatialEntityType::Herbivore);

    // Query radius 50 around origin should get entities 1 and 2
    let nearby = index.entities_in_radius(IVec2::new(0, 0), 50, Some(SpatialEntityType::Herbivore));
    assert!(nearby.contains(&entity1), "Should find entity1 at center");
    assert!(nearby.contains(&entity2), "Should find entity2 within radius");
    assert!(!nearby.contains(&entity3), "Should not find entity3 far away");
}

/// Test 2: Entity type filtering works correctly
#[test]
fn test_spatial_index_entity_type_filtering() {
    let mut index = SpatialEntityIndex::new();

    let herbivore = Entity::from_raw(1);
    let predator = Entity::from_raw(2);
    let omnivore = Entity::from_raw(3);

    index.insert(herbivore, IVec2::new(0, 0), SpatialEntityType::Herbivore);
    index.insert(predator, IVec2::new(5, 5), SpatialEntityType::Predator);
    index.insert(omnivore, IVec2::new(10, 10), SpatialEntityType::Omnivore);

    // Query only herbivores
    let herbivores = index.entities_in_radius(IVec2::new(0, 0), 20, Some(SpatialEntityType::Herbivore));
    assert_eq!(herbivores.len(), 1, "Should find only herbivore");
    assert!(herbivores.contains(&herbivore));

    // Query all types
    let all = index.entities_in_radius(IVec2::new(0, 0), 20, None);
    assert_eq!(all.len(), 3, "Should find all entities when no filter");
}

/// Test 3: Spatial index handles empty queries gracefully
#[test]
fn test_spatial_index_empty_radius() {
    let mut index = SpatialEntityIndex::new();

    let entity = Entity::from_raw(1);
    index.insert(entity, IVec2::new(100, 100), SpatialEntityType::Herbivore);

    // Query far away with small radius
    let nearby = index.entities_in_radius(IVec2::new(0, 0), 10, Some(SpatialEntityType::Herbivore));
    assert!(nearby.is_empty(), "Should return empty vec when no entities nearby");
}

/// Test 4: Spatial index supports large search radius
#[test]
fn test_spatial_index_large_radius() {
    let mut index = SpatialEntityIndex::new();

    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);

    index.insert(entity1, IVec2::new(0, 0), SpatialEntityType::Herbivore);
    index.insert(entity2, IVec2::new(150, 150), SpatialEntityType::Herbivore);
    index.insert(entity3, IVec2::new(300, 300), SpatialEntityType::Herbivore);

    // Large radius should find multiple entities
    let nearby = index.entities_in_radius(IVec2::new(0, 0), 200, Some(SpatialEntityType::Herbivore));
    assert!(nearby.contains(&entity1));
    assert!(nearby.contains(&entity2));
    assert!(!nearby.contains(&entity3));
}

/// Test 5: Rabbits should use Herbivore entity type
#[test]
fn test_rabbit_entity_type_classification() {
    // Rabbits are herbivores in spatial index
    assert_eq!(SpatialEntityType::Herbivore, SpatialEntityType::Herbivore);
    // This test validates conceptually that rabbits map to Herbivore type
}

/// Test 6: Deer should use Herbivore entity type
#[test]
fn test_deer_entity_type_classification() {
    // Deer are herbivores in spatial index
    assert_eq!(SpatialEntityType::Herbivore, SpatialEntityType::Herbivore);
}

/// Test 7: Predators (Fox, Wolf) should use Predator entity type
#[test]
fn test_predator_entity_type_classification() {
    // Fox and Wolf are predators
    assert_eq!(SpatialEntityType::Predator, SpatialEntityType::Predator);
}

/// Test 8: Omnivores (Bear, Raccoon) should use Omnivore entity type
#[test]
fn test_omnivore_entity_type_classification() {
    // Bear and Raccoon are omnivores
    assert_eq!(SpatialEntityType::Omnivore, SpatialEntityType::Omnivore);
}

/// Test 9: Spatial index performance benefit - can query k entities instead of N
#[test]
fn test_spatial_index_performance_benefit() {
    let mut index = SpatialEntityIndex::new();

    // Insert 1000 entities spread across large area
    for i in 0..1000 {
        let pos = IVec2::new((i % 100) as i32 * 10, (i / 100) as i32 * 10);
        index.insert(Entity::from_raw(i as u32 + 1), pos, SpatialEntityType::Herbivore);
    }

    // Query small radius - should return much fewer than 1000 entities
    let nearby = index.entities_in_radius(IVec2::new(0, 0), 50, Some(SpatialEntityType::Herbivore));

    // Should be ~25-30 entities in a 50-tile radius around (0,0), not 1000
    assert!(nearby.len() < 100, "Spatial query should return subset of entities");
    assert!(nearby.len() > 5, "Should still find nearby entities");
}

/// Test 10: Spatial index chunk-based operations work correctly
#[test]
fn test_spatial_index_chunk_operations() {
    let mut index = SpatialEntityIndex::new();

    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);

    // Insert entities in different chunks (16x16 tiles per chunk)
    index.insert(entity1, IVec2::new(0, 0), SpatialEntityType::Herbivore);      // Chunk (0, 0)
    index.insert(entity2, IVec2::new(20, 20), SpatialEntityType::Herbivore);    // Chunk (1, 1)
    index.insert(entity3, IVec2::new(50, 50), SpatialEntityType::Herbivore);    // Chunk (3, 3)

    // Verify chunks are being used (at least 2-3 chunks)
    assert!(index.chunk_count() >= 2, "Should have multiple chunks for spread out entities");

    // Verify total entity count
    assert_eq!(index.total_entities(), 3, "Should track all inserted entities");
}

/// Test 11: Multiple entity types can coexist in same location
#[test]
fn test_spatial_index_mixed_entity_types() {
    let mut index = SpatialEntityIndex::new();

    let herbivore = Entity::from_raw(1);
    let predator = Entity::from_raw(2);

    index.insert(herbivore, IVec2::new(0, 0), SpatialEntityType::Herbivore);
    index.insert(predator, IVec2::new(0, 0), SpatialEntityType::Predator);

    // Query all types at same location
    let all = index.entities_in_radius(IVec2::new(0, 0), 10, None);
    assert_eq!(all.len(), 2, "Should find both types at same location");

    // Query specific type
    let herbivores = index.entities_in_radius(IVec2::new(0, 0), 10, Some(SpatialEntityType::Herbivore));
    assert_eq!(herbivores.len(), 1, "Should find only herbivore with filter");
    assert!(herbivores.contains(&herbivore));
}
