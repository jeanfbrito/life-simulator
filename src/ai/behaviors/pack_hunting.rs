//! Pack-aware hunting behavior for coordinated wolf hunts.
//!
//! This module provides species-specific behavior bonuses for wolves hunting in packs.
//! When wolves coordinate their hunting efforts, they gain utility bonuses that make
//! pack hunts more attractive than solo hunting.

use bevy::prelude::*;
use crate::ai::{UtilityScore, is_in_pack};
use crate::ai::action::ActionType;
use crate::entities::TilePosition;

/// Bonus utility for pack hunting (makes pack hunts more attractive than solo)
const PACK_HUNT_UTILITY_BONUS: f32 = 0.15;

/// Distance threshold for considering pack members "coordinated" (in tiles)
const PACK_COORDINATION_RADIUS: f32 = 80.0;

fn distance(a: IVec2, b: IVec2) -> f32 {
    let diff = a - b;
    ((diff.x * diff.x + diff.y * diff.y) as f32).sqrt()
}

/// Apply pack hunting bonuses to hunt actions for coordinated wolf packs.
///
/// When a wolf is in a pack and hunting, this function boosts the utility of hunt
/// actions to reflect the advantage of coordinated pack hunting. The bonus scales
/// with the number of pack members nearby (within PACK_COORDINATION_RADIUS).
///
/// # Arguments
/// * `entity` - The wolf evaluating actions
/// * `actions` - Mutable vector of available actions with utilities
/// * `leader` - The pack leader entity
/// * `members` - List of all pack member entities
/// * `world` - The Bevy World for accessing components
pub fn apply_pack_hunting_bonus(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    leader: Entity,
    members: Vec<Entity>,
    world: &World,
) {
    // Verify this wolf is actually in the pack
    if !is_in_pack(entity, world) {
        return;
    }

    // Can't hunt if no pack members
    if members.is_empty() {
        return;
    }

    // Get this wolf's position for coordination checks
    let Some(my_pos) = world.get::<TilePosition>(entity) else {
        return;
    };

    // Count how many pack members are nearby
    let mut nearby_packmates = 0;
    for &member in members.iter() {
        if let Some(member_pos) = world.get::<TilePosition>(member) {
            let dist = distance(my_pos.tile, member_pos.tile);
            if dist <= PACK_COORDINATION_RADIUS {
                nearby_packmates += 1;
            }
        }
    }

    // Apply bonus to all hunt actions
    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Hunt { .. }) {
            // Scale bonus by number of coordinating pack members
            let coordination_factor =
                (nearby_packmates as f32 / members.len().max(1) as f32).min(1.0);
            let bonus = PACK_HUNT_UTILITY_BONUS * coordination_factor;
            action.utility = (action.utility + bonus).min(1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        assert_eq!(distance(IVec2::new(0, 0), IVec2::new(3, 4)), 5.0);
    }

    #[test]
    fn test_pack_hunt_bonus_constant() {
        assert!(PACK_HUNT_UTILITY_BONUS > 0.0);
        assert!(PACK_HUNT_UTILITY_BONUS < 0.5); // Reasonable bonus range
    }

    #[test]
    fn test_coordination_radius() {
        assert!(PACK_COORDINATION_RADIUS > 0.0);
    }
}
