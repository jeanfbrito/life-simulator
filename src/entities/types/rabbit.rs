/// Rabbit-specific behavior configuration
/// 
/// Defines behavior parameters optimized for rabbit entities.

use super::BehaviorConfig;
use bevy::prelude::Resource;

/// Rabbit behavior preset
pub struct RabbitBehavior;

/// Rabbit reproduction tuning parameters
#[derive(Resource, Debug, Clone, Copy)]
pub struct RabbitReproductionConfig {
    pub maturity_ticks: u32,
    pub gestation_ticks: u32,
    pub mating_cooldown_ticks: u32,
    pub postpartum_cooldown_ticks: u32,
    pub litter_size_range: (u8, u8),
    pub mating_search_radius: i32,
    pub well_fed_hunger_norm: f32,
    pub well_fed_thirst_norm: f32,
    pub well_fed_required_ticks: u32,
    pub matching_interval_ticks: u32,
    pub min_energy_norm: f32,
    pub min_health_norm: f32,
}

impl RabbitBehavior {
    /// Default reproduction parameters for rabbits
    pub fn reproduction_config() -> RabbitReproductionConfig {
        RabbitReproductionConfig {
            maturity_ticks: 3600,                 // ~6 minutes at 10 TPS
            gestation_ticks: 1200,               // ~2 minutes
            mating_cooldown_ticks: 600,          // ~1 minute (male)
            postpartum_cooldown_ticks: 1800,     // ~3 minutes (female)
            litter_size_range: (2, 6),
            mating_search_radius: 50,
            well_fed_hunger_norm: 0.35,
            well_fed_thirst_norm: 0.35,
            well_fed_required_ticks: 300,        // ~30s sustained
            matching_interval_ticks: 50,         // run matcher every 5s
            min_energy_norm: 0.5,
            min_health_norm: 0.6,
        }
    }
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
            0.75,       // thirst_threshold: Drink when >= 75% thirsty
            0.5,        // hunger_threshold: Eat when >= 50% hungry
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
        let needs = Self::needs();
        // Rabbits: higher metabolism — eat/drink more often, tire a bit faster
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.08)),  // moderate hunger gain
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.03)),  // slower thirst gain to avoid spam-drinking
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.07)), // slightly faster energy drain
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.01)), // same regen for now
        }
    }

    /// Species-level needs and consumption profile for rabbits
    pub fn needs() -> super::SpeciesNeeds {
        super::SpeciesNeeds {
            // Scaled from DF size (rabbit max 500 cm^3) to manageable sim numbers
            hunger_max: 70.0,
            thirst_max: 90.0,
            // Smaller consumption per event than deer
            eat_amount: 14.0,
            drink_amount: 45.0,
        }
    }

    /// Evaluate Rabbit actions
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
        // Note: thresholds are defined as 0.75 above
        assert_eq!(config.thirst_threshold, 0.75);
        assert_eq!(config.graze_range, (3, 8));
        assert_eq!(config.water_search_radius, 100);
    }
}
