/// Deer-specific behavior configuration
///
/// Defines behavior parameters and action evaluation for deer entities.

use super::BehaviorConfig;

/// Deer behavior preset
pub struct DeerBehavior;

impl DeerBehavior {
    /// Get the default behavior configuration for deer
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new(
            0.20,      // thirst_threshold
            0.30,      // hunger_threshold
            0.30,      // energy_threshold
            (5, 15),   // graze_range
            150,       // water_search_radius
            150,       // food_search_radius
            40,        // wander_radius
        )
    }

    /// Evaluate Deer actions in one place (the Deer's module)
    /// Includes following the nearest rabbit as an example social behavior.
    pub fn evaluate_actions(
        entity: bevy::prelude::Entity,
        position: &crate::entities::TilePosition,
        thirst: &crate::entities::stats::Thirst,
        hunger: &crate::entities::stats::Hunger,
        energy: &crate::entities::stats::Energy,
        behavior_config: &BehaviorConfig,
        world_loader: &crate::world_loader::WorldLoader,
        rabbits: &[(bevy::prelude::Entity, bevy::prelude::IVec2)],
    ) -> Vec<crate::ai::UtilityScore> {
        use crate::ai::behaviors::{
            evaluate_drinking_behavior,
            evaluate_eating_behavior,
            evaluate_grazing_behavior,
            evaluate_resting_behavior,
            evaluate_follow_behavior,
        };

        let mut actions = Vec::new();

        if let Some(drink) = evaluate_drinking_behavior(
            position,
            thirst,
            world_loader,
            behavior_config.thirst_threshold,
            behavior_config.water_search_radius,
        ) { actions.push(drink); }

        if let Some(eat) = evaluate_eating_behavior(
            position,
            hunger,
            world_loader,
            behavior_config.hunger_threshold,
            behavior_config.food_search_radius,
        ) { actions.push(eat); }

        if let Some(rest) = evaluate_resting_behavior(
            position,
            energy,
            behavior_config.energy_threshold,
        ) { actions.push(rest); }

        if let Some(graze) = evaluate_grazing_behavior(
            position,
            world_loader,
            behavior_config.graze_range,
        ) { actions.push(graze); }

        if let Some(follow) = evaluate_follow_behavior(
            entity,
            position,
            rabbits,
            2,   // stop distance
            40,  // max follow distance normalization
        ) { actions.push(follow); }

        actions
    }
}
