//! System for establishing parent-child relationships at birth
//!
//! This module processes newborns with Mother component and establishes
//! bidirectional parent-child relationships using ParentOf/ChildOf components.

use bevy::prelude::*;
use crate::entities::{BirthInfo, Mother};
use crate::simulation::SimulationTick;

// Legacy imports for backward compatibility (renamed to avoid conflict with Bevy's ChildOf)
#[allow(deprecated)]
use crate::entities::{LegacyParentOf, LegacyChildOf};

/// System to establish parent-child relationships when offspring are born using Bevy's hierarchy.
///
/// This system runs after birth systems and processes newborns that have Mother component.
/// It establishes Bevy's Parent/Children relationships with BirthInfo metadata for tracking.
///
/// This allows us to:
/// - Track all children of a parent (via Bevy's Children component)
/// - Track parent of a child (via Bevy's Parent component)
/// - Support family tree queries
/// - Automatic cleanup when parent dies (Bevy handles this)
///
/// Uses Bevy's built-in hierarchy for automatic bidirectional sync and despawn propagation.
pub fn establish_birth_relationships(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    mut newborns: Query<(Entity, &Mother), Added<Mother>>,
) {
    for (child_entity, mother) in newborns.iter_mut() {
        let parent = mother.0;
        let current_tick = tick.0;

        // Add BirthInfo metadata to child
        commands.entity(child_entity).insert(BirthInfo::new(current_tick));

        // Use Bevy's hierarchy system (automatically adds Parent and Children components)
        commands.entity(parent).add_child(child_entity);
    }
}

/// DEPRECATED: This system is no longer needed with Bevy's hierarchy.
/// Bevy's add_child automatically manages the Children component.
/// Kept for backward compatibility during migration.
#[deprecated(since = "0.1.0", note = "Use Bevy's hierarchy system instead")]
#[allow(deprecated)]
pub fn establish_parent_of_from_mother(
    mut parents: Query<&mut LegacyParentOf>,
    children: Query<(Entity, &Mother), Added<Mother>>,
    tick: Res<SimulationTick>,
) {
    for (child_entity, mother) in children.iter() {
        let parent = mother.0;
        let current_tick = tick.0;

        if let Ok(mut parent_of) = parents.get_mut(parent) {
            // Parent already has LegacyParentOf, add this child
            parent_of.add_child(child_entity);
        } else {
            // Parent doesn't have LegacyParentOf yet, but we can't create it here
            // because we don't have mutable access to parents that don't have the component
            // This will be handled by a different system
        }
    }
}

/// System to clean up orphaned children when parents die using Bevy's hierarchy.
///
/// This system monitors children with LegacyChildOf components and removes orphaned relationships
/// when parents no longer exist. Bevy handles most of this automatically, but this provides
/// an additional safety net.
///
/// Note: Bevy 0.16 uses ChildOf for hierarchy, but we use LegacyChildOf for backward compatibility.
#[allow(deprecated)]
pub fn cleanup_orphaned_children_when_parent_dies(
    mut commands: Commands,
    children: Query<(Entity, &LegacyChildOf)>,
    parents: Query<Entity>,
) {
    let parent_set: std::collections::HashSet<Entity> = parents.iter().collect();

    for (child_entity, child_of) in children.iter() {
        // If parent entity no longer exists, remove LegacyChildOf component from child
        // (Bevy should handle this automatically, but this is a safety net)
        if !parent_set.contains(&child_of.parent) {
            commands.entity(child_entity).remove::<LegacyChildOf>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that BirthInfo component can be created with correct tick
    #[test]
    fn test_birth_info_new_with_tick() {
        let birth_info = BirthInfo::new(100);
        assert_eq!(birth_info.born_tick, 100);
    }

    /// Test that BirthInfo is Copy
    #[test]
    fn test_birth_info_is_copy() {
        let info1 = BirthInfo::new(50);
        let info2 = info1;
        assert_eq!(info1.born_tick, info2.born_tick);
    }

    /// LEGACY: Test that ChildOf component can be created with correct tick
    #[test]
    #[allow(deprecated)]
    fn test_child_of_new_with_tick() {
        let parent = Entity::from_raw(1);
        let child_of = LegacyChildOf::new(parent, 100);

        assert_eq!(child_of.parent, parent);
        assert_eq!(child_of.born_tick, 100);
    }

    /// LEGACY: Test that LegacyParentOf can be created with tick
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_new_with_tick() {
        let parent_of = LegacyParentOf::new(100);

        assert_eq!(parent_of.first_birth_tick, 100);
        assert_eq!(parent_of.child_count(), 0);
    }

    /// LEGACY: Test LegacyParentOf can add and track children
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_add_child() {
        let mut parent_of = LegacyParentOf::new(100);
        let child = Entity::from_raw(2);

        parent_of.add_child(child);

        assert_eq!(parent_of.child_count(), 1);
        assert!(parent_of.has_child(child));
    }

    /// LEGACY: Test: Multiple children can be added
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_multiple_children() {
        let mut parent_of = LegacyParentOf::new(100);
        let child1 = Entity::from_raw(2);
        let child2 = Entity::from_raw(3);
        let child3 = Entity::from_raw(4);

        parent_of.add_child(child1);
        parent_of.add_child(child2);
        parent_of.add_child(child3);

        assert_eq!(parent_of.child_count(), 3);
        assert!(parent_of.has_child(child1));
        assert!(parent_of.has_child(child2));
        assert!(parent_of.has_child(child3));
    }

    /// Test: Birth tick tracking with BirthInfo
    #[test]
    fn test_birth_tick_tracking() {
        let birth_info1 = BirthInfo::new(50);
        let birth_info2 = BirthInfo::new(200);

        assert_eq!(birth_info1.born_tick, 50);
        assert_eq!(birth_info2.born_tick, 200);
    }
}
