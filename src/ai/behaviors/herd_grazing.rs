//! Herd grazing behavior for coordinated herbivore groups.
//!
//! This module provides species-specific behavior bonuses for deer and other herbivores
//! in herds. When herbivores are grouped together, they gain safety benefits that make
//! grazing and resting more attractive (safety in numbers).

use bevy::prelude::*;
use crate::ai::{UtilityScore};
use crate::ai::action::ActionType;

/// Bonus utility for herd safety (makes grazing/resting safer in groups)
const HERD_SAFETY_BONUS: f32 = 0.10;

/// Apply herd safety bonuses to grazing and resting actions.
///
/// When herbivores are in a herd, this function boosts the utility of grazing and
/// resting actions to reflect the increased safety that comes from being in a group.
/// The "safety in numbers" effect makes animals more willing to engage in lower-alert
/// behaviors when surrounded by herd members.
///
/// # Arguments
/// * `entity` - The herbivore evaluating actions
/// * `actions` - Mutable vector of available actions with utilities
/// * `leader` - The herd leader entity
/// * `members` - List of all herd member entities
pub fn apply_herd_safety_bonus(
    _entity: Entity,
    actions: &mut Vec<UtilityScore>,
    _leader: Entity,
    _members: Vec<Entity>,
) {
    // Boost graze/rest actions for herbivores in herds
    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Graze { .. } | ActionType::Rest { .. }) {
            action.utility = (action.utility + HERD_SAFETY_BONUS).min(1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_herd_safety_bonus_constant() {
        assert!(HERD_SAFETY_BONUS > 0.0);
        assert!(HERD_SAFETY_BONUS < 0.3); // Reasonable bonus range
    }
}
