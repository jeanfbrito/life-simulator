/// Rabbit-specific behavior configuration
/// 
/// Defines behavior parameters optimized for rabbit entities.

use super::BehaviorConfig;

/// Rabbit behavior preset
pub struct RabbitBehavior;

impl RabbitBehavior {
    /// Get the default behavior configuration for rabbits
    /// 
    /// Rabbit characteristics:
    /// - Proactive drinkers: Seek water at 15% thirsty to avoid dehydration
    /// - Moderate eaters: Eat grass when 40% hungry
    /// - Light sleepers: Rest when energy drops below 30%
    /// - Short-range grazers: Prefer to stay close (3-8 tiles) when eating
    /// - Moderate search radius: 100 tiles for resources
    /// - Small territory: 15 tile wander radius
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new(
            0.15,       // thirst_threshold: Drink when 15% thirsty
            0.4,        // hunger_threshold: Eat when 40% hungry
            0.3,        // energy_threshold: Rest when energy drops below 30%
            (3, 8),     // graze_range: Short-range grazing (3-8 tiles)
            100,        // water_search_radius: Wide water search
            100,        // food_search_radius: Wide food search
            15,         // wander_radius: Small territory
        )
    }

    /// Species-specific stats preset for rabbits (initial values and rates)
    /// Keeps stat components generic, only the preset lives here.
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{EntityStatsBundle, Hunger, Thirst, Energy, Health, Stat};
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, 100.0, 0.10)),  // default rabbit hunger rate
            thirst: Thirst(Stat::new(0.0, 0.0, 100.0, 0.15)),  // default thirst rate
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.05)),
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.01)),
        }
    }

    /// Evaluate Rabbit actions in one place (the Rabbit's module)
    /// Delegates to generic behavior evaluators but centralizes Rabbit logic.
    pub fn evaluate_actions(
        position: &crate::entities::TilePosition,
        thirst: &crate::entities::stats::Thirst,
        hunger: &crate::entities::stats::Hunger,
        energy: &crate::entities::stats::Energy,
        behavior_config: &BehaviorConfig,
        world_loader: &crate::world_loader::WorldLoader,
    ) -> Vec<crate::ai::UtilityScore> {
        use crate::ai::behaviors::{
            evaluate_drinking_behavior,
            evaluate_eating_behavior,
            evaluate_grazing_behavior,
            evaluate_resting_behavior,
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

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rabbit_config() {
        let config = RabbitBehavior::config();
        // Note: thresholds are defined as 0.15 above
        assert_eq!(config.thirst_threshold, 0.15);
        assert_eq!(config.graze_range, (3, 8));
        assert_eq!(config.water_search_radius, 100);
    }
}
