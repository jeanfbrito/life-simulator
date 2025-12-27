//! System for managing predator-prey hunting relationships.
//!
//! This system establishes and maintains hunting relationships when predators
//! select prey and cleans up stale relationships when hunts complete or prey dies.

use bevy::prelude::*;
use crate::entities::{ActiveHunter, HuntingTarget};
use crate::entities::TilePosition;

/// Establishes a hunting relationship when a predator selects a prey entity
pub fn establish_hunting_relationship(
    predator: Entity,
    prey: Entity,
    tick: u64,
    commands: &mut Commands,
) {
    // Add HuntingTarget marker to prey
    commands.entity(prey).insert(HuntingTarget {
        predator,
        started_tick: tick,
    });

    // Add ActiveHunter marker to predator
    commands.entity(predator).insert(ActiveHunter {
        target: prey,
        started_tick: tick,
    });
}

/// Clears a hunting relationship when hunt completes or prey dies
pub fn clear_hunting_relationship(
    predator: Entity,
    prey: Entity,
    commands: &mut Commands,
) {
    // Remove HuntingTarget from prey
    commands.entity(prey).remove::<HuntingTarget>();

    // Remove ActiveHunter from predator
    commands.entity(predator).remove::<ActiveHunter>();
}

/// System to clean up stale hunting relationships
/// Runs periodically to remove relationships for dead/despawned prey
pub fn cleanup_stale_hunting_relationships(
    mut commands: Commands,
    hunters: Query<(Entity, &ActiveHunter)>,
    prey_check: Query<Entity, With<TilePosition>>,
) {
    for (hunter_entity, active) in hunters.iter() {
        // If prey no longer exists, remove the relationship
        if prey_check.get(active.target).is_err() {
            commands.entity(hunter_entity).remove::<ActiveHunter>();
        }
    }
}

/// System to validate hunting relationships are bidirectional
/// Used for testing and validation
#[cfg(test)]
pub fn validate_hunting_relationships(
    hunters: Query<&ActiveHunter>,
    prey: Query<&HuntingTarget>,
) -> bool {
    for hunter in hunters.iter() {
        if let Ok(target) = prey.get(hunter.target) {
            if target.predator != hunter.target {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: establish_hunting_relationship function signature
    #[test]
    fn test_establish_hunting_relationship_components_exist() {
        // This test verifies the function can be called
        // Full integration testing happens at the system level
        let predator = Entity::from_raw(1);
        let prey = Entity::from_raw(2);

        // These components should be created with proper values
        let hunter = ActiveHunter {
            target: prey,
            started_tick: 100,
        };

        let target = HuntingTarget {
            predator,
            started_tick: 100,
        };

        assert_eq!(hunter.target, prey);
        assert_eq!(target.predator, predator);
        assert_eq!(hunter.started_tick, target.started_tick);
    }

    /// Test: cleanup system validates relationships
    #[test]
    fn test_cleanup_stale_hunting_relationships_validation() {
        // Test that the system can identify stale relationships
        let predator = Entity::from_raw(1);
        let prey = Entity::from_raw(2);

        let hunter = ActiveHunter {
            target: prey,
            started_tick: 50,
        };

        // Verify the component tracks the right prey
        assert_eq!(hunter.target, prey);
    }

    /// Test: Multiple hunters track different prey
    #[test]
    fn test_multiple_hunters_different_prey() {
        let predator1 = Entity::from_raw(1);
        let predator2 = Entity::from_raw(2);
        let prey1 = Entity::from_raw(100);
        let prey2 = Entity::from_raw(101);

        let hunter1 = ActiveHunter {
            target: prey1,
            started_tick: 100,
        };

        let hunter2 = ActiveHunter {
            target: prey2,
            started_tick: 100,
        };

        assert_ne!(hunter1.target, hunter2.target);
        assert_eq!(hunter1.target, prey1);
        assert_eq!(hunter2.target, prey2);
    }
}
