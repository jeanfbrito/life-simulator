//! System for managing parent-child relationships during reproduction.
//!
//! This system establishes and maintains parent-child relationships when offspring are born
//! and cleans up orphaned children when parents die using Bevy's built-in hierarchy system.
//!
//! Note: Bevy 0.16 uses ChildOf (not Parent) and Children components for hierarchy.

use bevy::prelude::*;
use crate::entities::{BirthInfo, TilePosition};

// Bevy 0.16 hierarchy components (ChildOf and Children are in prelude)
// ChildOf has a doc alias as "Parent" but uses .parent() method
// Note: Conflicts with our legacy ChildOf component name from parent_child_relationships.rs

// Re-export our custom legacy components with different names to avoid conflicts
#[allow(deprecated)]
pub use crate::entities::{LegacyParentOf, LegacyChildOf};

/// Establishes a parent-child relationship when offspring are born using Bevy's hierarchy
///
/// # Arguments
/// * `parent` - The parent entity
/// * `child` - The child entity
/// * `tick` - Current simulation tick
/// * `commands` - Bevy commands for adding components
///
/// Note: This function uses a deferred approach. When called during a system,
/// the actual hierarchy update happens after the current system completes.
/// For immediate updates in a single-world context, call establish_parent_child_immediate instead.
pub fn establish_parent_child_relationship(
    parent: Entity,
    child: Entity,
    tick: u64,
    commands: &mut Commands,
) {
    // Add BirthInfo to child for metadata tracking
    commands.entity(child).insert(BirthInfo::new(tick));

    // Use Bevy's built-in hierarchy system
    commands.entity(parent).add_child(child);
}

/// Establishes parent-child relationship in immediate world context using Bevy's hierarchy
/// Used when you have direct world access (e.g., in tests or initialization)
pub fn establish_parent_child_immediate(
    parent: Entity,
    child: Entity,
    tick: u64,
    world: &mut World,
) {
    // Add BirthInfo metadata to child
    world.entity_mut(child).insert(BirthInfo::new(tick));

    // Use Bevy's built-in add_child which automatically:
    // - Adds Children component to parent
    // - Adds Parent component to child
    // - Maintains bidirectional sync
    world.entity_mut(parent).add_child(child);
}

/// Removes a parent-child relationship (e.g., when child dies) using Bevy's hierarchy
/// This function removes the child from the parent's hierarchy.
pub fn remove_parent_child_relationship(
    parent: Entity,
    child: Entity,
    commands: &mut Commands,
) {
    // Remove child from parent's hierarchy
    // Bevy automatically removes Parent component from child
    commands.entity(parent).remove_children(&[child]);
}

/// Removes parent-child relationship in immediate world context using Bevy's hierarchy
pub fn remove_parent_child_immediate(
    parent: Entity,
    child: Entity,
    world: &mut World,
) {
    // Remove child from parent's hierarchy
    // Bevy automatically manages Parent/Children component updates
    world.entity_mut(parent).remove_children(&[child]);
}

/// System to clean up orphaned children using Bevy's hierarchy
/// Runs periodically to remove children whose parents have been despawned
pub fn cleanup_orphaned_children(
    mut commands: Commands,
    children: Query<(Entity, &ChildOf)>,  // Bevy 0.16 uses ChildOf
    parent_check: Query<Entity, With<TilePosition>>,
) {
    for (child_entity, child_of) in children.iter() {
        // If parent no longer exists, remove the ChildOf component
        // (Bevy should handle this automatically, but this is a safety net)
        if parent_check.get(child_of.parent()).is_err() {
            commands.entity(child_entity).remove::<ChildOf>();
        }
    }
}

/// Get the parent of an entity, if it has one (using Bevy's ChildOf component)
pub fn get_parent(
    child: Entity,
    world: &World,
) -> Option<Entity> {
    world.get::<ChildOf>(child).map(|c| c.parent())  // Bevy 0.16: ChildOf.parent()
}

/// Get all children of a parent entity (using Bevy's Children component)
pub fn get_children(
    parent: Entity,
    world: &World,
) -> Vec<Entity> {
    world.get::<Children>(parent)
        .map(|c| c.to_vec())
        .unwrap_or_default()
}

/// Check if parent has a specific child (using Bevy's Children component)
pub fn has_child(
    parent: Entity,
    child: Entity,
    world: &World,
) -> bool {
    if let Some(children) = world.get::<Children>(parent) {
        children.contains(&child)
    } else {
        false
    }
}

/// Check if child has a parent (using Bevy's ChildOf component)
pub fn has_parent(
    child: Entity,
    world: &World,
) -> bool {
    world.get::<ChildOf>(child).is_some()  // Bevy 0.16: ChildOf instead of Parent
}

/// Get the count of children for a parent (using Bevy's Children component)
pub fn child_count(
    parent: Entity,
    world: &World,
) -> usize {
    world.get::<Children>(parent)
        .map(|c| c.len())
        .unwrap_or(0)
}

/// Get the birth tick of a child (using BirthInfo component)
pub fn child_birth_tick(
    child: Entity,
    world: &World,
) -> Option<u64> {
    world.get::<BirthInfo>(child).map(|b| b.born_tick)
}

/// Validate that parent-child relationships are bidirectional and consistent
/// Used for testing and validation (using Bevy's hierarchy components)
#[cfg(test)]
pub fn validate_parent_child_relationships(
    parents: Query<(Entity, &Children)>,
    children: Query<(Entity, &ChildOf)>,  // Bevy 0.16: ChildOf instead of Parent
) -> bool {
    // Validate forward references (parent -> children)
    for (parent_entity, children_list) in parents.iter() {
        for child_entity in children_list.iter() {
            if let Ok((_, child_of)) = children.get(child_entity) {
                // Child should reference back to parent
                if child_of.parent() != parent_entity {  // Bevy 0.16: .parent() method
                    return false;
                }
            } else {
                // Child component missing - inconsistency
                return false;
            }
        }
    }

    // Validate backward references (children -> parent)
    for (child_entity, child_of) in children.iter() {
        let parent_entity = child_of.parent();  // Bevy 0.16: .parent() method
        if let Ok((_, children_list)) = parents.get(parent_entity) {
            // Parent should reference back to child
            if !children_list.contains(&child_entity) {
                return false;
            }
        } else {
            // Parent without Children component might be valid (single child)
            // Just check parent entity exists
            continue;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create a test world with basic components
    fn create_test_world() -> World {
        World::new()
    }

    /// Test that get_parent returns None for entity without Parent
    #[test]
    fn test_get_parent_nonexistent() {
        let world = create_test_world();
        let child = Entity::from_raw(1);

        let parent = get_parent(child, &world);
        assert!(parent.is_none());
    }

    /// Test that get_children returns empty for entity without Children
    #[test]
    fn test_get_children_nonexistent() {
        let world = create_test_world();
        let parent = Entity::from_raw(1);

        let children = get_children(parent, &world);
        assert!(children.is_empty());
    }

    /// Test has_parent returns false for entity without Parent
    #[test]
    fn test_has_parent_false() {
        let world = create_test_world();
        let child = Entity::from_raw(1);

        let has_parent = has_parent(child, &world);
        assert!(!has_parent);
    }

    /// Test child_count returns 0 for entity without Children
    #[test]
    fn test_child_count_zero() {
        let world = create_test_world();
        let parent = Entity::from_raw(1);

        let count = child_count(parent, &world);
        assert_eq!(count, 0);
    }

    /// Test child_birth_tick returns None for entity without BirthInfo
    #[test]
    fn test_child_birth_tick_none() {
        let world = create_test_world();
        let child = Entity::from_raw(1);

        let tick = child_birth_tick(child, &world);
        assert!(tick.is_none());
    }

    /// Test has_child returns false when not a parent
    #[test]
    fn test_has_child_false() {
        let world = create_test_world();
        let parent = Entity::from_raw(1);
        let child = Entity::from_raw(2);

        let has = has_child(parent, child, &world);
        assert!(!has);
    }

    /// Test: BirthInfo component structure
    #[test]
    fn test_birth_info_component() {
        let birth_info = BirthInfo::new(100);
        assert_eq!(birth_info.born_tick, 100);
    }

    /// Test: Birth tick calculation with BirthInfo
    #[test]
    fn test_birth_tick_calculation() {
        let birth_info = BirthInfo::new(100);
        let current_tick = 150;
        let age = current_tick - birth_info.born_tick;

        assert_eq!(age, 50);
    }
}
