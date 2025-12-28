//! Mating pair relationships using type-safe Bevy components.
//!
//! This module implements type-safe mating relationships following the same
//! pattern as hunting relationships. A mating relationship consists of two
//! components that form a bidirectional pair:
//!
//! - MatingTarget: Applied to one entity (typically the female) marking that
//!   they are being pursued for mating
//! - ActiveMate: Applied to the other entity (typically the male) marking that
//!   they are pursuing a mate
//!
//! This replaces the manual MatingIntent component with a safer, more maintainable
//! relationship system that ensures bidirectional consistency and automatic cleanup.

use bevy::prelude::*;

/// Marker component indicating that this entity is being pursued for mating.
/// Applied to one entity when another entity initiates a mating relationship.
///
/// This component tracks which entity initiated the mating and when it started,
/// enabling cleanup of stale relationships and duration tracking.
#[derive(Component, Debug, Clone, Copy)]
pub struct MatingTarget {
    /// Which entity initiated the mating relationship (typically male)
    pub suitor: Entity,
    /// The tile where mating meeting occurs
    pub meeting_tile: IVec2,
    /// Simulation tick when mating started
    pub started_tick: u64,
}

/// Marker component indicating that this entity is actively pursuing a mate.
/// Applied to one entity when it selects a mate for reproduction.
///
/// This component tracks which entity is being pursued and when the pursuit started,
/// enabling cleanup of stale relationships and duration tracking.
#[derive(Component, Debug, Clone, Copy)]
pub struct ActiveMate {
    /// Which entity this entity is pursuing (typically female)
    pub partner: Entity,
    /// The tile where mating meeting occurs
    pub meeting_tile: IVec2,
    /// Simulation tick when mating started
    pub started_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RED: Test that MatingTarget can be created with suitor and tick info
    #[test]
    fn test_mating_target_creation() {
        let suitor = Entity::PLACEHOLDER;
        let tile = IVec2::new(10, 20);
        let target = MatingTarget {
            suitor,
            meeting_tile: tile,
            started_tick: 100,
        };

        assert_eq!(target.suitor, suitor);
        assert_eq!(target.meeting_tile, tile);
        assert_eq!(target.started_tick, 100);
    }

    /// RED: Test that ActiveMate can be created with partner and tick info
    #[test]
    fn test_active_mate_creation() {
        let partner = Entity::PLACEHOLDER;
        let tile = IVec2::new(15, 25);
        let mate = ActiveMate {
            partner,
            meeting_tile: tile,
            started_tick: 100,
        };

        assert_eq!(mate.partner, partner);
        assert_eq!(mate.meeting_tile, tile);
        assert_eq!(mate.started_tick, 100);
    }

    /// RED: Test that MatingTarget is Copy and Clone
    #[test]
    fn test_mating_target_is_copy() {
        let suitor = Entity::PLACEHOLDER;
        let tile = IVec2::new(10, 20);
        let target1 = MatingTarget {
            suitor,
            meeting_tile: tile,
            started_tick: 100,
        };
        let target2 = target1; // Should copy without issue

        assert_eq!(target1.suitor, target2.suitor);
        assert_eq!(target1.meeting_tile, target2.meeting_tile);
        assert_eq!(target1.started_tick, target2.started_tick);
    }

    /// RED: Test that ActiveMate is Copy and Clone
    #[test]
    fn test_active_mate_is_copy() {
        let partner = Entity::PLACEHOLDER;
        let tile = IVec2::new(15, 25);
        let mate1 = ActiveMate {
            partner,
            meeting_tile: tile,
            started_tick: 100,
        };
        let mate2 = mate1; // Should copy without issue

        assert_eq!(mate1.partner, mate2.partner);
        assert_eq!(mate1.meeting_tile, mate2.meeting_tile);
        assert_eq!(mate1.started_tick, mate2.started_tick);
    }

    /// Test: Mating relationship matching
    #[test]
    fn test_mating_relationship_matching() {
        let entity_a = Entity::from_raw(1);
        let entity_b = Entity::from_raw(2);
        let tile = IVec2::new(20, 30);

        let target = MatingTarget {
            suitor: entity_a,
            meeting_tile: tile,
            started_tick: 50,
        };

        let mate = ActiveMate {
            partner: entity_b,
            meeting_tile: tile,
            started_tick: 50,
        };

        // Verify bidirectional consistency
        assert_eq!(target.suitor, entity_a);
        assert_eq!(mate.partner, entity_b);
        assert_eq!(target.started_tick, mate.started_tick);
        assert_eq!(target.meeting_tile, mate.meeting_tile);
    }

    /// Test: Multiple entities can have different mates
    #[test]
    fn test_multiple_mating_pairs() {
        let male_a = Entity::from_raw(1);
        let female_a = Entity::from_raw(2);
        let male_b = Entity::from_raw(3);
        let female_b = Entity::from_raw(4);

        let mate_a = ActiveMate {
            partner: female_a,
            meeting_tile: IVec2::new(10, 10),
            started_tick: 50,
        };

        let mate_b = ActiveMate {
            partner: female_b,
            meeting_tile: IVec2::new(20, 20),
            started_tick: 50,
        };

        // Each male can have their own mate
        assert_ne!(mate_a.partner, mate_b.partner);
        assert_eq!(mate_a.partner, female_a);
        assert_eq!(mate_b.partner, female_b);
    }

    /// Test: Mating duration calculation
    #[test]
    fn test_mating_duration_calculation() {
        let suitor = Entity::PLACEHOLDER;
        let tile = IVec2::new(10, 10);

        let target = MatingTarget {
            suitor,
            meeting_tile: tile,
            started_tick: 100,
        };

        let current_tick = 150;
        let mating_duration = current_tick - target.started_tick;

        assert_eq!(mating_duration, 50);
    }

    /// Test: Different meeting tiles for different pairs
    #[test]
    fn test_different_meeting_tiles() {
        let suitor1 = Entity::from_raw(1);
        let suitor2 = Entity::from_raw(2);
        let tile1 = IVec2::new(10, 10);
        let tile2 = IVec2::new(50, 50);

        let target1 = MatingTarget {
            suitor: suitor1,
            meeting_tile: tile1,
            started_tick: 100,
        };

        let target2 = MatingTarget {
            suitor: suitor2,
            meeting_tile: tile2,
            started_tick: 100,
        };

        assert_ne!(target1.meeting_tile, target2.meeting_tile);
        assert_eq!(target1.meeting_tile, tile1);
        assert_eq!(target2.meeting_tile, tile2);
    }
}
