//! Warren defense behavior for coordinated rabbit groups.
//!
//! This module provides species-specific behavior bonuses for rabbits in warrens.
//! When rabbits are grouped together, they gain alert bonuses that make fleeing
//! more attractive (group alert system).

use bevy::prelude::*;
use crate::ai::{UtilityScore};
use crate::ai::actions::ActionType;

/// Bonus utility for warren defense (group alert effect)
const WARREN_FLEE_BONUS: f32 = 0.20;

/// Apply warren defense bonuses to escape/movement actions for coordinated rabbit groups.
///
/// When rabbits are in a warren (burrow group), this function boosts the utility of
/// movement/escape actions to reflect the increased alertness that comes from being in
/// a coordinated group. The "group alert" effect means that when one rabbit detects
/// danger, the entire warren becomes more responsive to threats.
///
/// # Arguments
/// * `entity` - The rabbit evaluating actions
/// * `actions` - Mutable vector of available actions with utilities
/// * `leader` - The warren leader entity
/// * `members` - List of all warren member entities
pub fn apply_warren_defense_bonus(
    _entity: Entity,
    actions: &mut Vec<UtilityScore>,
    _leader: Entity,
    _members: Vec<Entity>,
) {
    // Boost movement actions for rabbits in warrens (group alert)
    // This primarily affects Graze actions used for escape/relocation
    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Graze { .. } | ActionType::Wander { .. }) {
            action.utility = (action.utility + WARREN_FLEE_BONUS).min(1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warren_flee_bonus_constant() {
        assert!(WARREN_FLEE_BONUS > 0.0);
        assert!(WARREN_FLEE_BONUS < 0.5); // Reasonable bonus range
    }
}
