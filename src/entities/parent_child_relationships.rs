//! Parent-child relationships using Bevy's built-in hierarchy system.
//!
//! This module implements type-safe parent-child relationships for reproduction
//! using Bevy's native Parent/Children components with additional birth metadata.
//!
//! # Pattern
//! ```text
//! Parent Entity
//!   ├─ Children (Bevy built-in, tracks all children)
//!
//! Child Entity
//!   ├─ Parent (Bevy built-in, tracks parent reference)
//!   └─ BirthInfo (custom metadata for birth tick)
//! ```

// Allow deprecated items - this module contains legacy components for backward compatibility
#![allow(deprecated)]

use bevy::prelude::*;

/// Component tracking birth metadata for offspring entities.
/// Applied to child when born, stores birth timing information.
#[derive(Component, Debug, Clone, Copy)]
pub struct BirthInfo {
    /// Simulation tick when this entity was born
    pub born_tick: u64,
}

impl BirthInfo {
    /// Create a new BirthInfo tracker
    pub fn new(born_tick: u64) -> Self {
        Self { born_tick }
    }
}

// ============================================================================
// LEGACY COMPONENTS - DEPRECATED
// ============================================================================
// These components are kept temporarily for backward compatibility during migration.
// They will be removed once all systems are converted to use Bevy's hierarchy.

/// DEPRECATED: Use Bevy's Children component instead.
/// Component tracking all offspring of a parent entity.
///
/// Renamed from `ParentOf` to `LegacyParentOf` to avoid collision with Bevy 0.16's
/// built-in hierarchy components.
#[deprecated(
    since = "0.1.0",
    note = "Use Bevy's Children component with BirthInfo instead"
)]
#[derive(Component, Debug, Clone)]
pub struct LegacyParentOf {
    /// All offspring entities
    pub children: Vec<Entity>,
    /// Simulation tick of first birth
    pub first_birth_tick: u64,
}

#[allow(deprecated)]
impl LegacyParentOf {
    /// Create a new LegacyParentOf tracker
    pub fn new(first_birth_tick: u64) -> Self {
        Self {
            children: Vec::new(),
            first_birth_tick,
        }
    }

    /// Add a child to this parent's offspring list
    pub fn add_child(&mut self, child: Entity) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Remove a child from this parent's offspring list
    pub fn remove_child(&mut self, child: Entity) {
        self.children.retain(|e| *e != child);
    }

    /// Get the number of offspring
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Check if this parent has a specific offspring
    pub fn has_child(&self, child: Entity) -> bool {
        self.children.contains(&child)
    }
}

/// DEPRECATED: Use Bevy's Parent component with BirthInfo instead.
/// Component indicating this entity's parent (birth mother).
///
/// Renamed from `ChildOf` to `LegacyChildOf` to avoid collision with Bevy 0.16's
/// built-in ChildOf component for the hierarchy system.
#[deprecated(
    since = "0.1.0",
    note = "Use Bevy's Parent component with BirthInfo instead"
)]
#[derive(Component, Debug, Clone, Copy)]
pub struct LegacyChildOf {
    /// Birth parent entity
    pub parent: Entity,
    /// Simulation tick when born
    pub born_tick: u64,
}

#[allow(deprecated)]
impl LegacyChildOf {
    /// Create a new LegacyChildOf tracker
    pub fn new(parent: Entity, born_tick: u64) -> Self {
        Self { parent, born_tick }
    }
}

// Backward compatibility type aliases
#[deprecated(since = "0.1.0", note = "Use LegacyParentOf instead")]
pub type ParentOf = LegacyParentOf;

#[deprecated(since = "0.1.0", note = "Use LegacyChildOf instead")]
pub type ChildOf = LegacyChildOf;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that BirthInfo can be created
    #[test]
    fn test_birth_info_creation() {
        let birth_info = BirthInfo::new(100);
        assert_eq!(birth_info.born_tick, 100);
    }

    /// Test that BirthInfo is Copy
    #[test]
    fn test_birth_info_is_copy() {
        let info1 = BirthInfo::new(50);
        let info2 = info1; // Should copy without issue
        assert_eq!(info1.born_tick, info2.born_tick);
    }

    // ========================================================================
    // LEGACY TESTS - Keep for backward compatibility during migration
    // ========================================================================

    /// RED: Test that LegacyParentOf can be created
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_creation() {
        let parent = LegacyParentOf::new(100);

        assert_eq!(parent.first_birth_tick, 100);
        assert_eq!(parent.child_count(), 0);
        assert!(parent.children.is_empty());
    }

    /// RED: Test that LegacyChildOf can be created
    #[test]
    #[allow(deprecated)]
    fn test_child_of_creation() {
        let parent_entity = Entity::PLACEHOLDER;
        let child = LegacyChildOf::new(parent_entity, 50);

        assert_eq!(child.parent, parent_entity);
        assert_eq!(child.born_tick, 50);
    }

    /// RED: Test that LegacyParentOf can track children
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_add_child() {
        let mut parent = LegacyParentOf::new(100);
        let child1 = Entity::from_raw(1);
        let child2 = Entity::from_raw(2);

        parent.add_child(child1);
        assert_eq!(parent.child_count(), 1);
        assert!(parent.has_child(child1));

        parent.add_child(child2);
        assert_eq!(parent.child_count(), 2);
        assert!(parent.has_child(child2));
    }

    /// RED: Test that duplicate children are not added twice
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_no_duplicate_children() {
        let mut parent = LegacyParentOf::new(100);
        let child = Entity::from_raw(1);

        parent.add_child(child);
        parent.add_child(child); // Add same child again

        assert_eq!(parent.child_count(), 1, "Child should only be added once");
    }

    /// RED: Test that LegacyParentOf can remove children
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_remove_child() {
        let mut parent = LegacyParentOf::new(100);
        let child1 = Entity::from_raw(1);
        let child2 = Entity::from_raw(2);

        parent.add_child(child1);
        parent.add_child(child2);
        assert_eq!(parent.child_count(), 2);

        parent.remove_child(child1);
        assert_eq!(parent.child_count(), 1);
        assert!(!parent.has_child(child1));
        assert!(parent.has_child(child2));
    }

    /// RED: Test that removing non-existent child is safe
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_remove_nonexistent_child() {
        let mut parent = LegacyParentOf::new(100);
        let child1 = Entity::from_raw(1);
        let child2 = Entity::from_raw(2);

        parent.add_child(child1);
        parent.remove_child(child2); // Remove child that was never added

        assert_eq!(parent.child_count(), 1);
        assert!(parent.has_child(child1));
    }

    /// RED: Test that LegacyParentOf tracks first_birth_tick
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_first_birth_tick() {
        let parent1 = LegacyParentOf::new(50);
        let parent2 = LegacyParentOf::new(200);

        assert_eq!(parent1.first_birth_tick, 50);
        assert_eq!(parent2.first_birth_tick, 200);
    }

    /// RED: Test that LegacyChildOf is Copy and Debug
    #[test]
    #[allow(deprecated)]
    fn test_child_of_is_copy() {
        let parent = Entity::PLACEHOLDER;
        let child1 = LegacyChildOf::new(parent, 100);
        let child2 = child1; // Should copy without issue

        assert_eq!(child1.parent, child2.parent);
        assert_eq!(child1.born_tick, child2.born_tick);
    }

    /// RED: Test parent-child relationship consistency
    #[test]
    #[allow(deprecated)]
    fn test_parent_child_consistency() {
        let parent = LegacyParentOf::new(100);
        let parent_entity = Entity::from_raw(1);
        let child_entity = Entity::from_raw(2);

        let child = LegacyChildOf::new(parent_entity, 100);

        // Both should reference each other
        assert_eq!(child.parent, parent_entity);
        assert_eq!(parent.first_birth_tick, child.born_tick);
    }

    /// RED: Test multiple children tracking
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_multiple_children() {
        let mut parent = LegacyParentOf::new(100);
        let children: Vec<Entity> = (0..5)
            .map(|i| Entity::from_raw(i + 10))
            .collect();

        for child in &children {
            parent.add_child(*child);
        }

        assert_eq!(parent.child_count(), 5);
        for child in &children {
            assert!(parent.has_child(*child));
        }
    }

    /// RED: Test that we can clear all children
    #[test]
    #[allow(deprecated)]
    fn test_parent_of_clear_children() {
        let mut parent = LegacyParentOf::new(100);
        let children: Vec<Entity> = (0..3)
            .map(|i| Entity::from_raw(i + 10))
            .collect();

        for child in &children {
            parent.add_child(*child);
        }

        // Clear all children by removing each
        for child in &children {
            parent.remove_child(*child);
        }

        assert_eq!(parent.child_count(), 0);
    }
}
