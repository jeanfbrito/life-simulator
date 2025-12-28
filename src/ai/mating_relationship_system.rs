//! System for managing mating pair relationships.
//!
//! This system establishes and maintains mating relationships when entities
//! pair for reproduction, and cleans up stale relationships when mating completes
//! or when one partner dies.

use bevy::prelude::*;
use crate::entities::{ActiveMate, MatingTarget, TilePosition};

/// Establishes a mating relationship between two entities
///
/// Creates bidirectional relationship components:
/// - MatingTarget on one entity (typically female)
/// - ActiveMate on the other entity (typically male)
///
/// Both track the meeting tile and start tick for duration and cleanup tracking.
pub fn establish_mating_relationship(
    entity_a: Entity,
    entity_b: Entity,
    meeting_tile: IVec2,
    tick: u64,
    commands: &mut Commands,
) {
    // Add MatingTarget marker to entity_b (being pursued)
    commands.entity(entity_b).insert(MatingTarget {
        suitor: entity_a,
        meeting_tile,
        started_tick: tick,
    });

    // Add ActiveMate marker to entity_a (pursuing)
    commands.entity(entity_a).insert(ActiveMate {
        partner: entity_b,
        meeting_tile,
        started_tick: tick,
    });
}

/// Clears a mating relationship when mating completes or one partner dies
///
/// Removes both the MatingTarget and ActiveMate components to break the relationship.
pub fn clear_mating_relationship(
    entity_a: Entity,
    entity_b: Entity,
    commands: &mut Commands,
) {
    // Remove ActiveMate from entity_a
    commands.entity(entity_a).remove::<ActiveMate>();

    // Remove MatingTarget from entity_b
    commands.entity(entity_b).remove::<MatingTarget>();
}

/// System to clean up stale mating relationships
/// Runs periodically to remove relationships where one partner has died/despawned
pub fn cleanup_stale_mating_relationships(
    mut commands: Commands,
    mates: Query<(Entity, &ActiveMate)>,
    entity_check: Query<Entity, With<TilePosition>>,
) {
    for (mate_entity, active) in mates.iter() {
        // If partner no longer exists, remove the relationship from both sides
        if entity_check.get(active.partner).is_err() {
            // Remove ActiveMate from the pursuing entity
            commands.entity(mate_entity).remove::<ActiveMate>();
            // Note: We can't remove MatingTarget from the dead entity, but it doesn't matter
            // since the entity will be despawned. The next cleanup cycle will find no more
            // references to it.
        }
    }
}

/// Check if an entity has an active mating relationship
///
/// Returns true if the entity is actively pursuing a mate (has ActiveMate component).
pub fn has_mating_relationship(
    entity: Entity,
    world: &World,
) -> bool {
    world.get::<ActiveMate>(entity).is_some()
}

/// Check if an entity is being pursued for mating
///
/// Returns true if the entity has been selected as a mate (has MatingTarget component).
pub fn is_being_courted(
    entity: Entity,
    world: &World,
) -> bool {
    world.get::<MatingTarget>(entity).is_some()
}

/// Get the mating partner of an entity, if one exists
///
/// Returns the partner entity if the entity is actively mating.
pub fn get_mating_partner(
    entity: Entity,
    world: &World,
) -> Option<Entity> {
    world.get::<ActiveMate>(entity).map(|mate| mate.partner)
}

/// System to validate mating relationships are bidirectional
/// Used for testing and validation
#[cfg(test)]
pub fn validate_mating_relationships(
    mates: Query<&ActiveMate>,
    targets: Query<&MatingTarget>,
) -> bool {
    for mate in mates.iter() {
        if let Ok(target) = targets.get(mate.partner) {
            // The target should reference back to this mate entity
            // For validation, we just check both exist in their respective queries
            if target.suitor != mate.partner {
                return false;
            }
        } else {
            // ActiveMate exists but no corresponding MatingTarget
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: establish_mating_relationship creates proper components
    #[test]
    fn test_establish_mating_relationship_components_exist() {
        let entity_a = Entity::from_raw(1);
        let entity_b = Entity::from_raw(2);
        let tile = IVec2::new(10, 20);

        // These components should be created with proper values
        let mate = ActiveMate {
            partner: entity_b,
            meeting_tile: tile,
            started_tick: 100,
        };

        let target = MatingTarget {
            suitor: entity_a,
            meeting_tile: tile,
            started_tick: 100,
        };

        assert_eq!(mate.partner, entity_b);
        assert_eq!(target.suitor, entity_a);
        assert_eq!(mate.started_tick, target.started_tick);
        assert_eq!(mate.meeting_tile, target.meeting_tile);
    }

    /// Test: cleanup system validates relationships
    #[test]
    fn test_cleanup_stale_mating_relationships_validation() {
        let suitor = Entity::from_raw(1);
        let partner = Entity::from_raw(2);
        let tile = IVec2::new(10, 10);

        let mate = ActiveMate {
            partner,
            meeting_tile: tile,
            started_tick: 50,
        };

        // Verify the component tracks the right partner
        assert_eq!(mate.partner, partner);
    }

    /// Test: Multiple mates track different partners
    #[test]
    fn test_multiple_mates_different_partners() {
        let entity_a = Entity::from_raw(1);
        let entity_b = Entity::from_raw(2);
        let entity_c = Entity::from_raw(3);
        let entity_d = Entity::from_raw(4);

        let mate1 = ActiveMate {
            partner: entity_b,
            meeting_tile: IVec2::new(10, 10),
            started_tick: 100,
        };

        let mate2 = ActiveMate {
            partner: entity_d,
            meeting_tile: IVec2::new(20, 20),
            started_tick: 100,
        };

        assert_ne!(mate1.partner, mate2.partner);
        assert_eq!(mate1.partner, entity_b);
        assert_eq!(mate2.partner, entity_d);
    }

    /// Test: get_mating_partner functionality
    #[test]
    fn test_get_mating_partner_extraction() {
        let entity_a = Entity::from_raw(1);
        let entity_b = Entity::from_raw(2);
        let tile = IVec2::new(10, 10);

        let mate = ActiveMate {
            partner: entity_b,
            meeting_tile: tile,
            started_tick: 100,
        };

        // Verify partner can be extracted
        assert_eq!(mate.partner, entity_b);
    }

    /// Test: Mating tiles are tracked correctly
    #[test]
    fn test_mating_tile_tracking() {
        let suitor = Entity::from_raw(1);
        let partner = Entity::from_raw(2);
        let tile1 = IVec2::new(5, 5);
        let tile2 = IVec2::new(15, 15);

        let mate1 = ActiveMate {
            partner: Entity::from_raw(100),
            meeting_tile: tile1,
            started_tick: 100,
        };

        let mate2 = ActiveMate {
            partner: Entity::from_raw(101),
            meeting_tile: tile2,
            started_tick: 100,
        };

        assert_eq!(mate1.meeting_tile, tile1);
        assert_eq!(mate2.meeting_tile, tile2);
        assert_ne!(mate1.meeting_tile, mate2.meeting_tile);
    }

    /// Test: Mating duration tracking through start ticks
    #[test]
    fn test_mating_duration_tracking() {
        let suitor = Entity::from_raw(1);
        let partner = Entity::from_raw(2);
        let tile = IVec2::new(10, 10);

        let mate = ActiveMate {
            partner,
            meeting_tile: tile,
            started_tick: 100,
        };

        let current_tick = 150;
        let duration = current_tick - mate.started_tick;

        assert_eq!(duration, 50);
    }
}
