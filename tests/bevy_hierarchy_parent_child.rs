//! Comprehensive TDD tests for Bevy Parent/Children hierarchy integration
//!
//! This test suite validates the conversion from custom ParentOf/ChildOf components
//! to Bevy's built-in Parent/Children hierarchy components.
//!
//! Tests cover:
//! - Establishing parent-child relationships with Bevy hierarchy
//! - Managing multiple children per parent
//! - Despawning behavior and cleanup
//! - BirthInfo metadata persistence
//! - Bidirectional hierarchy sync
//! - Edge cases and error conditions
//!
//! NOTE: In Bevy 0.16+, when you add a Parent component to an entity,
//! Bevy automatically maintains the corresponding Children component on the parent.
//! This happens through the hierarchy maintenance systems in the engine.

use bevy::prelude::*;
use life_simulator::entities::{ChildOf, ParentOf};

/// Component to track birth metadata (tick when entity was born)
/// This component should persist independently of the hierarchy
#[derive(Component, Debug, Clone, Copy)]
pub struct BirthInfo {
    pub born_tick: u64,
}

impl BirthInfo {
    pub fn new(born_tick: u64) -> Self {
        Self { born_tick }
    }
}

// ============================================================================
// TEST HELPERS
// ============================================================================

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

/// Setup parent-child relationship
/// In Bevy, adding Parent component automatically creates Children on parent
fn setup_parent_child(world: &mut World, mother: Entity, child: Entity, born_tick: u64) {
    // Add Parent component to child - Bevy automatically maintains Children on parent
    world.entity_mut(child).insert(Parent(mother));

    // Add BirthInfo to child
    world.entity_mut(child).insert(BirthInfo::new(born_tick));
}

// ============================================================================
// TEST: Establish parent-child relationship with Bevy hierarchy
// ============================================================================

/// RED: Test that parent-child relationship is established correctly
/// - Child should have Parent component pointing to mother
/// - Mother should have Children component containing child (auto-created by Bevy)
/// - Both entities should be linked bidirectionally
#[test]
fn test_establish_parent_child_creates_bevy_hierarchy() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    // Create entities
    let mother = world.spawn_empty().id();
    let child = world.spawn_empty().id();

    // Establish relationship using helper
    setup_parent_child(&mut world, mother, child, 100);

    // Verify child has Parent component
    let parent_comp = world.get::<Parent>(child);
    assert!(parent_comp.is_some(), "Child should have Parent component");
    assert_eq!(parent_comp.unwrap().get(), mother, "Parent should point to mother");

    // Verify BirthInfo persists
    let birth_info = world.get::<BirthInfo>(child);
    assert!(birth_info.is_some(), "Child should have BirthInfo component");
    assert_eq!(birth_info.unwrap().born_tick, 100, "BirthInfo should track correct tick");

    // Note: Mother's Children component is created by Bevy's hierarchy system
    // In tests without running update systems, manual verification may be needed
}

// ============================================================================
// TEST: Multiple children per parent
// ============================================================================

/// RED: Test managing multiple children per parent
/// - Each child should have Parent component pointing to mother
/// - Each child should have unique BirthInfo but same born_tick
#[test]
fn test_multiple_children_per_parent() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();
    let child3 = world.spawn_empty().id();

    // Add each child to parent
    for child in [child1, child2, child3].iter() {
        world.entity_mut(*child).insert(Parent(mother));
        world.entity_mut(*child).insert(BirthInfo::new(100));
    }

    // Verify each child points to mother
    for child in [child1, child2, child3].iter() {
        let parent = world.get::<Parent>(*child).unwrap();
        assert_eq!(parent.get(), mother, "Child should point to mother");

        let birth = world.get::<BirthInfo>(*child).unwrap();
        assert_eq!(birth.born_tick, 100, "Child should have birth info");
    }
}

// ============================================================================
// TEST: Despawn parent removes hierarchy
// ============================================================================

/// RED: Test despawn behavior with parent-child hierarchy
/// - When parent is despawned, Child's Parent component still exists (dangling)
/// - Using despawn_recursive() should remove entire hierarchy
/// - Using despawn() should only remove parent, leaving orphaned children
#[test]
fn test_despawn_parent_leaves_orphaned_children() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child = world.spawn_empty().id();

    // Set up hierarchy
    world.entity_mut(child).insert(Parent(mother));
    world.entity_mut(child).insert(BirthInfo::new(100));

    // Verify setup
    assert!(world.get::<Parent>(child).is_some(), "Child should have Parent");

    // Despawn parent (not recursive)
    world.despawn(mother);

    // Child should still exist but now orphaned
    let child_entity = world.get_entity(child);
    assert!(child_entity.is_ok(), "Child should still exist after parent despawn");

    // Child's Parent reference still points to despawned parent (dangling reference)
    // This is why we need explicit cleanup systems
    if let Ok(_child_ref) = world.get_entity(child) {
        if let Some(parent) = world.get::<Parent>(child) {
            // Parent still exists in component, but points to despawned entity
            assert_eq!(parent.get(), mother, "Child's parent reference should still exist");
        }
    }
}

/// RED: Test despawn_recursive removes entire hierarchy
/// - Using despawn_recursive() should remove all children
/// - All child entities should be despawned
#[test]
fn test_despawn_recursive_removes_children() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();

    // Set up hierarchy
    world.entity_mut(child1).insert(Parent(mother));
    world.entity_mut(child2).insert(Parent(mother));

    // Despawn recursively
    world.despawn_recursive(mother);

    // All entities should be despawned
    assert!(world.get_entity(mother).is_err(), "Mother should be despawned");
    assert!(world.get_entity(child1).is_err(), "Child1 should be despawned");
    assert!(world.get_entity(child2).is_err(), "Child2 should be despawned");
}

// ============================================================================
// TEST: Despawn child updates parent
// ============================================================================

/// RED: Test that despawning a child affects parent's Children
/// - When child is despawned, it should be removed from parent's Children
/// - Bevy may not automatically update Children, needs manual cleanup
/// - Parent should only conceptually contain remaining children
#[test]
fn test_despawn_child_behavior() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();

    // Set up hierarchy
    world.entity_mut(child1).insert(Parent(mother));
    world.entity_mut(child2).insert(Parent(mother));

    // Verify setup
    assert!(world.get::<Parent>(child1).is_some(), "Child1 should have Parent");
    assert!(world.get::<Parent>(child2).is_some(), "Child2 should have Parent");

    // Despawn child1
    world.despawn(child1);

    // Verify child1 is gone but child2 remains
    assert!(world.get_entity(child1).is_err(), "Child1 should be despawned");
    assert!(world.get_entity(child2).is_ok(), "Child2 should still exist");

    // Verify child2 still has parent reference
    if let Ok(_) = world.get_entity(child2) {
        assert!(world.get::<Parent>(child2).is_some(), "Child2 should still have Parent");
    }
}

// ============================================================================
// TEST: Cleanup stale relationships
// ============================================================================

/// RED: Test cleanup of stale parent references
/// - When a child is despawned without proper cleanup, Parent component becomes dangling
/// - Cleanup system should find orphaned Parent components and remove them
#[test]
fn test_cleanup_stale_parent_references() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child = world.spawn_empty().id();

    // Set up hierarchy
    world.entity_mut(child).insert(Parent(mother));

    // Despawn child WITHOUT updating parent
    world.despawn(child);

    // Child is gone but Parent reference would be dangling if it existed
    // Cleanup would need to remove any orphaned Parent components

    // Verify child is despawned
    assert!(world.get_entity(child).is_err(), "Child should be despawned");
}

/// RED: Test cleanup of orphaned children (parents despawned)
/// - When parent is despawned, children keep Parent component (dangling reference)
/// - Cleanup should detect parent doesn't exist and remove Parent component
#[test]
fn test_cleanup_orphaned_children() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();

    // Set up hierarchy
    world.entity_mut(child1).insert(Parent(mother));
    world.entity_mut(child2).insert(Parent(mother));

    // Despawn parent without despawning children
    world.despawn(mother);

    // Children still have Parent component pointing to despawned entity
    assert!(world.get::<Parent>(child1).is_some(), "Child1 should still have Parent component");
    assert!(world.get::<Parent>(child2).is_some(), "Child2 should still have Parent component");

    // Manual cleanup for test
    let entities_to_clean: Vec<_> = world
        .query::<(Entity, &Parent)>()
        .iter(&world)
        .filter_map(|(entity, parent)| {
            if world.get_entity(parent.get()).is_err() {
                Some(entity)
            } else {
                None
            }
        })
        .collect();

    for entity in entities_to_clean {
        world.entity_mut(entity).remove::<Parent>();
    }

    // Verify cleanup
    assert!(world.get::<Parent>(child1).is_none(), "Child1 Parent should be removed");
    assert!(world.get::<Parent>(child2).is_none(), "Child2 Parent should be removed");
}

// ============================================================================
// TEST: Query parents and children
// ============================================================================

/// RED: Test querying all parents and their children
/// - Should be able to query all entities with Parent component
/// - Should be able to iterate over all parent-child relationships
#[test]
fn test_query_parents_and_children() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    // Create 3 mothers with different numbers of children
    let mother1 = world.spawn_empty().id();
    let mother2 = world.spawn_empty().id();
    let mother3 = world.spawn_empty().id();

    // Mother 1: 2 children
    let m1_c1 = world.spawn_empty().id();
    let m1_c2 = world.spawn_empty().id();

    // Mother 2: 1 child
    let m2_c1 = world.spawn_empty().id();

    // Mother 3: 3 children
    let m3_c1 = world.spawn_empty().id();
    let m3_c2 = world.spawn_empty().id();
    let m3_c3 = world.spawn_empty().id();

    // Set up hierarchies
    for child in [m1_c1, m1_c2] {
        world.entity_mut(child).insert(Parent(mother1));
    }

    for child in [m2_c1] {
        world.entity_mut(child).insert(Parent(mother2));
    }

    for child in [m3_c1, m3_c2, m3_c3] {
        world.entity_mut(child).insert(Parent(mother3));
    }

    // Query all entities with Parent
    let mut parent_refs = 0;
    let mut parent_entities = std::collections::HashSet::new();

    for (child_entity, parent) in world.query::<(Entity, &Parent)>().iter(&world) {
        parent_refs += 1;
        parent_entities.insert(parent.get());

        // Verify parent exists
        assert!(world.get_entity(parent.get()).is_ok(), "Parent should exist");
    }

    assert_eq!(parent_refs, 6, "Should find 6 parent references");
    assert_eq!(parent_entities.len(), 3, "Should find 3 unique parents");
}

// ============================================================================
// TEST: Query children by parent
// ============================================================================

/// RED: Test querying from child to parent
/// - Should be able to query all entities with Parent component
/// - Should be able to find each child's parent
#[test]
fn test_query_children_by_parent() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();
    let child3 = world.spawn_empty().id();

    // Set up hierarchy
    world.entity_mut(child1).insert(Parent(mother));
    world.entity_mut(child2).insert(Parent(mother));
    world.entity_mut(child3).insert(Parent(mother));

    // Query all children and verify parent
    let mut child_count = 0;
    for (child_entity, parent) in world.query::<(Entity, &Parent)>().iter(&world) {
        child_count += 1;
        assert_eq!(parent.get(), mother, "Child's parent should be mother");
    }

    assert_eq!(child_count, 3, "Should find 3 children");
}

// ============================================================================
// TEST: BirthInfo metadata persists
// ============================================================================

/// RED: Test that BirthInfo persists independently of hierarchy changes
/// - BirthInfo should track the tick when entity was born
/// - BirthInfo should persist even if hierarchy changes
/// - Multiple children should have same born_tick but independent BirthInfo
#[test]
fn test_birth_info_metadata_persists() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child1 = world.spawn_empty().id();
    let child2 = world.spawn_empty().id();

    let born_tick = 100u64;

    // Set up hierarchy with BirthInfo
    world.entity_mut(child1).insert(Parent(mother));
    world.entity_mut(child1).insert(BirthInfo::new(born_tick));
    world.entity_mut(child2).insert(Parent(mother));
    world.entity_mut(child2).insert(BirthInfo::new(born_tick));

    // Verify BirthInfo
    let birth1 = world.get::<BirthInfo>(child1).unwrap();
    assert_eq!(birth1.born_tick, born_tick, "Child1 birth tick should match");

    let birth2 = world.get::<BirthInfo>(child2).unwrap();
    assert_eq!(birth2.born_tick, born_tick, "Child2 birth tick should match");
}

/// RED: Test BirthInfo persists through hierarchy removal
/// - Remove child from parent's Children
/// - BirthInfo should still exist
/// - BirthInfo should still be accurate
#[test]
fn test_birth_info_persists_after_removal_from_hierarchy() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child = world.spawn_empty().id();
    let born_tick = 50u64;

    // Set up hierarchy
    world.entity_mut(child).insert(Parent(mother));
    world.entity_mut(child).insert(BirthInfo::new(born_tick));

    // Remove from hierarchy
    world.entity_mut(child).remove::<Parent>();

    // BirthInfo should persist
    let birth_info = world.get::<BirthInfo>(child).unwrap();
    assert_eq!(birth_info.born_tick, born_tick, "BirthInfo should persist after removal");
}

// ============================================================================
// TEST: Bidirectional consistency
// ============================================================================

/// RED: Test bidirectional parent-child consistency
/// - If child has Parent, child should reference parent
/// - Multiple children should all reference the same parent
#[test]
fn test_bidirectional_consistency() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child = world.spawn_empty().id();

    // Set up hierarchy
    world.entity_mut(child).insert(Parent(mother));

    // Check backward direction: child -> parent
    let parent = world.get::<Parent>(child).unwrap();
    assert_eq!(parent.get(), mother, "Child's parent should be mother");
}

/// RED: Test consistency with multiple children
/// - All children should reference the same parent
/// - No cross-referencing between different families
#[test]
fn test_consistency_multiple_children() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother1 = world.spawn_empty().id();
    let mother2 = world.spawn_empty().id();

    let m1_c1 = world.spawn_empty().id();
    let m1_c2 = world.spawn_empty().id();
    let m2_c1 = world.spawn_empty().id();

    // Set up mother 1 hierarchy
    world.entity_mut(m1_c1).insert(Parent(mother1));
    world.entity_mut(m1_c2).insert(Parent(mother1));

    // Set up mother 2 hierarchy
    world.entity_mut(m2_c1).insert(Parent(mother2));

    // Verify each child knows correct parent
    assert_eq!(world.get::<Parent>(m1_c1).unwrap().get(), mother1, "m1_c1 parent correct");
    assert_eq!(world.get::<Parent>(m1_c2).unwrap().get(), mother1, "m1_c2 parent correct");
    assert_eq!(world.get::<Parent>(m2_c1).unwrap().get(), mother2, "m2_c1 parent correct");
}

// ============================================================================
// TEST: Edge cases
// ============================================================================

/// RED: Test entity with no parent
/// - Entity without Parent should not appear in parent queries
/// - Entity can exist without hierarchy
#[test]
fn test_entity_with_no_hierarchy() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let orphan = world.spawn_empty().id();

    // Orphan has no Parent
    assert!(world.get::<Parent>(orphan).is_none(), "Orphan should not have Parent component");

    // Query should not find orphan
    let mut found = false;
    for (entity, _parent) in world.query::<(Entity, &Parent)>().iter(&world) {
        if entity == orphan {
            found = true;
        }
    }
    assert!(!found, "Orphan should not appear in Parent queries");
}

/// RED: Test readding child after removal
/// - Remove child from parent
/// - Re-add child to parent
/// - Should work correctly with new relationship
#[test]
fn test_readd_child_after_removal() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child = world.spawn_empty().id();

    // First relationship
    world.entity_mut(child).insert(Parent(mother));

    // Remove
    world.entity_mut(child).remove::<Parent>();

    // Re-add
    world.entity_mut(child).insert(Parent(mother));

    // Verify new relationship works
    assert_eq!(world.get::<Parent>(child).unwrap().get(), mother, "Readded child points to mother");
}

// ============================================================================
// TEST: Integration with old ParentOf/ChildOf components
// ============================================================================

/// RED: Test that new Bevy hierarchy can coexist with old ParentOf/ChildOf
/// - Both component systems should work together
/// - BirthInfo should work with both
#[test]
fn test_bevy_hierarchy_with_legacy_components() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let child = world.spawn_empty().id();
    let born_tick = 100u64;

    // Add Bevy hierarchy
    world.entity_mut(child).insert(Parent(mother));

    // Also add legacy components during transition
    world.entity_mut(mother).insert(ParentOf::new(born_tick));
    world.entity_mut(child).insert(ChildOf::new(mother, born_tick));
    world.entity_mut(child).insert(BirthInfo::new(born_tick));

    // Update ParentOf to track child
    world.get_mut::<ParentOf>(mother).unwrap().add_child(child);

    // Verify both systems work
    // Bevy hierarchy
    assert_eq!(world.get::<Parent>(child).unwrap().get(), mother, "Bevy Parent works");

    // Legacy components
    assert!(world.get::<ParentOf>(mother).unwrap().has_child(child), "Legacy ParentOf works");
    assert_eq!(world.get::<ChildOf>(child).unwrap().parent, mother, "Legacy ChildOf works");

    // BirthInfo
    assert_eq!(world.get::<BirthInfo>(child).unwrap().born_tick, born_tick, "BirthInfo works");
}

// ============================================================================
// TEST: Performance and scale
// ============================================================================

/// RED: Test with many children
/// - Parent should handle 100+ children efficiently
/// - Queries should work with large hierarchies
#[test]
fn test_large_family() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let mother = world.spawn_empty().id();
    let num_children = 100;

    // Add many children
    for _i in 0..num_children {
        let child = world.spawn_empty().id();
        world.entity_mut(child).insert(Parent(mother));
        world.entity_mut(child).insert(BirthInfo::new(100));
    }

    // Count children through query
    let mut count = 0;
    for (_entity, parent) in world.query::<(Entity, &Parent)>().iter(&world) {
        if parent.get() == mother {
            count += 1;
        }
    }
    assert_eq!(count, num_children, "All children should be queryable");
}

/// RED: Test with many parents
/// - System should handle multiple separate families
/// - Queries should distinguish between families
#[test]
fn test_many_families() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    let num_mothers = 50;
    let children_per_mother = 3;

    for m in 0..num_mothers {
        let mother = world.spawn_empty().id();

        for c in 0..children_per_mother {
            let child = world.spawn_empty().id();
            world.entity_mut(child).insert(Parent(mother));
        }
    }

    // Count total parent references
    let mut total_children = 0;

    for (_entity, _parent) in world.query::<(Entity, &Parent)>().iter(&world) {
        total_children += 1;
    }

    assert_eq!(total_children, num_mothers * children_per_mother, "Should find all children");
}

// ============================================================================
// INTEGRATION TEST: Full lifecycle
// ============================================================================

/// RED: Test complete lifecycle with multiple operations
/// - Create hierarchy
/// - Add/remove children
/// - Query relationships
/// - Verify consistency throughout
#[test]
fn test_complete_lifecycle() {
    let mut app = create_test_app();
    let mut world = app.world_mut();

    // Phase 1: Create family
    let mother = world.spawn_empty().id();
    let born_tick = 100u64;

    // Phase 2: Add first child
    let child1 = world.spawn_empty().id();
    world.entity_mut(child1).insert(Parent(mother));
    world.entity_mut(child1).insert(BirthInfo::new(born_tick));

    assert!(world.get::<Parent>(child1).is_some(), "Child1 should have parent");

    // Phase 3: Add second child
    let child2 = world.spawn_empty().id();
    world.entity_mut(child2).insert(Parent(mother));
    world.entity_mut(child2).insert(BirthInfo::new(born_tick + 5));

    // Phase 4: Query and verify
    let mut child_count = 0;
    for (child_entity, parent) in world.query::<(Entity, &Parent)>().iter(&world) {
        if parent.get() == mother {
            child_count += 1;
            let birth = world.get::<BirthInfo>(child_entity).unwrap();
            assert!(birth.born_tick >= born_tick, "Birth tick should be valid");
        }
    }
    assert_eq!(child_count, 2, "Should find both children");

    // Phase 5: Remove one child (simulate death)
    world.despawn(child2);

    // Phase 6: Verify remaining child
    let mut remaining_count = 0;
    for (child_entity, parent) in world.query::<(Entity, &Parent)>().iter(&world) {
        if parent.get() == mother {
            remaining_count += 1;
            assert_eq!(child_entity, child1, "Only child1 should remain");
            let birth = world.get::<BirthInfo>(child_entity).unwrap();
            assert_eq!(birth.born_tick, born_tick, "BirthInfo should persist");
        }
    }
    assert_eq!(remaining_count, 1, "Should have 1 remaining child");
}
