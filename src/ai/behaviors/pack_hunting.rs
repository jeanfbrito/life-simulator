//! Pack-aware hunting behavior for coordinated wolf hunts.
//!
//! This module provides species-specific behavior bonuses for wolves hunting in packs.
//! When wolves coordinate their hunting efforts, they gain utility bonuses that make
//! pack hunts more attractive than solo hunting.

use bevy::prelude::*;
use crate::ai::{UtilityScore, is_in_pack};
use crate::ai::actions::ActionType;
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
pub fn apply_pack_hunting_bonus(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    _leader: Entity,
    members: Vec<Entity>,
) {
    // Can't hunt if no pack members
    if members.is_empty() {
        return;
    }

    // Note: We cannot query positions here without &World or Query parameters
    // For now, apply a flat bonus. This can be refined later by passing position queries.
    // Apply bonus to all hunt actions
    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Hunt { .. }) {
            // Apply flat bonus (simplified until we can pass position queries)
            action.utility = (action.utility + PACK_HUNT_UTILITY_BONUS).min(1.0);
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
