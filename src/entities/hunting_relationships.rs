//! Predator-prey hunting relationships using Bevy's relations system.
//!
//! This module implements type-safe hunting relationships using Bevy 0.16's
//! relation components (Parent/Child hierarchy). A hunting relationship is established
//! when a predator begins pursuing prey.
//!
//! # Pattern
//! ```text
//! Predator Entity
//!   └─ HuntingTarget (child) → Prey Entity
//!      └─ marked with HuntingTarget component
//! ```

use bevy::prelude::*;

/// Marker component indicating that this entity is actively being hunted.
/// Applied to the prey entity when a predator targets it for hunting.
#[derive(Component, Debug, Clone, Copy)]
pub struct HuntingTarget {
    /// Which predator is hunting this entity
    pub predator: Entity,
    /// Simulation tick when hunt began
    pub started_tick: u64,
}

/// Marker component indicating that this entity is actively hunting.
/// Applied to the predator entity when it selects prey.
#[derive(Component, Debug, Clone, Copy)]
pub struct ActiveHunter {
    /// What this predator is hunting
    pub target: Entity,
    /// Simulation tick when hunt began
    pub started_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RED: Test that HuntingTarget can be created with predator and tick info
    #[test]
    fn test_hunting_target_creation() {
        let predator = Entity::PLACEHOLDER;
        let target = HuntingTarget {
            predator,
            started_tick: 100,
        };

        assert_eq!(target.predator, predator);
        assert_eq!(target.started_tick, 100);
    }

    /// RED: Test that ActiveHunter can be created with target and tick info
    #[test]
    fn test_active_hunter_creation() {
        let prey = Entity::PLACEHOLDER;
        let hunter = ActiveHunter {
            target: prey,
            started_tick: 100,
        };

        assert_eq!(hunter.target, prey);
        assert_eq!(hunter.started_tick, 100);
    }

    /// RED: Test that HuntingTarget is Copy and Debug
    #[test]
    fn test_hunting_target_is_copy() {
        let predator = Entity::PLACEHOLDER;
        let target1 = HuntingTarget {
            predator,
            started_tick: 100,
        };
        let target2 = target1; // Should copy without issue

        assert_eq!(target1.predator, target2.predator);
        assert_eq!(target1.started_tick, target2.started_tick);
    }

    /// RED: Test that ActiveHunter is Copy and Debug
    #[test]
    fn test_active_hunter_is_copy() {
        let prey = Entity::PLACEHOLDER;
        let hunter1 = ActiveHunter {
            target: prey,
            started_tick: 100,
        };
        let hunter2 = hunter1; // Should copy without issue

        assert_eq!(hunter1.target, hunter2.target);
        assert_eq!(hunter1.started_tick, hunter2.started_tick);
    }

    /// Test: Predator and prey relationship timing
    #[test]
    fn test_hunting_relationship_timing() {
        let predator = Entity::PLACEHOLDER;
        let prey = Entity::PLACEHOLDER;

        let target = HuntingTarget {
            predator,
            started_tick: 50,
        };

        let hunter = ActiveHunter {
            target: prey,
            started_tick: 50,
        };

        // Both should have same start tick
        assert_eq!(target.started_tick, hunter.started_tick);
    }

    /// Test: Multiple predators can't all hunt same prey (validation at higher level)
    /// This test just verifies each component works independently
    #[test]
    fn test_different_predators_different_prey() {
        let predator1 = Entity::from_raw(1);
        let predator2 = Entity::from_raw(2);
        let prey1 = Entity::from_raw(100);
        let prey2 = Entity::from_raw(101);

        let hunter1 = ActiveHunter {
            target: prey1,
            started_tick: 50,
        };

        let hunter2 = ActiveHunter {
            target: prey2,
            started_tick: 50,
        };

        // Each hunter can have their own prey
        assert_ne!(hunter1.target, hunter2.target);
        assert_eq!(hunter1.target, prey1);
        assert_eq!(hunter2.target, prey2);
    }

    /// Test: Hunt duration calculation
    #[test]
    fn test_hunt_duration_calculation() {
        let predator = Entity::PLACEHOLDER;
        let prey = Entity::PLACEHOLDER;

        let target = HuntingTarget {
            predator,
            started_tick: 100,
        };

        let current_tick = 150;
        let hunt_duration = current_tick - target.started_tick;

        assert_eq!(hunt_duration, 50);
    }
}
