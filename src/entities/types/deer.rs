/// Deer-specific behavior configuration
///
/// Defines behavior parameters and action evaluation for deer entities.
use super::BehaviorConfig;
use crate::entities::reproduction::ReproductionConfig;

/// Deer behavior preset
pub struct DeerBehavior;

impl DeerBehavior {
    /// Fast reproduction parameters for deer (for testing)
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            // Deer take noticeably longer to mature and recover between litters than rabbits.
            maturity_ticks: 12_000,           // ~20 minutes at 10 TPS
            gestation_ticks: 6_000,           // ~10 minutes pregnant
            mating_cooldown_ticks: 2_000,     // ~3.3 minutes before males can mate again
            postpartum_cooldown_ticks: 9_000, // ~15 minutes recovery for females
            litter_size_range: (1, 2),        // Single fawn common, twins possible
            mating_search_radius: 60,         // Seek partners within a broad clearing
            well_fed_hunger_norm: 0.55,       // Require healthier condition than rabbits
            well_fed_thirst_norm: 0.55,
            well_fed_required_ticks: 600, // Must stay well fed for ~60s
            matching_interval_ticks: 300, // Check every 30s for partners
            mating_duration_ticks: 50,    // ~5s spent together
            min_energy_norm: 0.35,        // Need reasonable energy reserves
            min_health_norm: 0.4,         // Avoid mating when injured
        }
    }

    /// Get the default behavior configuration for deer
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new(
            0.65,    // thirst_threshold (wait longer before drinking)
            0.45,    // hunger_threshold (eat less frequently)
            0.30,    // energy_threshold
            (5, 15), // graze_range
            150,     // water_search_radius
            150,     // food_search_radius
            40,      // wander_radius
        )
    }

    /// Species-specific stats preset for deer (initial values and rates)
    /// Keeps stat components generic, only the preset lives here.
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        let needs = Self::needs();
        // Deer: lower metabolism — eat/drink less often, tire a bit slower
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.05)), // slower hunger gain
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.02)), // much slower thirst gain
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.04)),         // slower energy drain
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.01)),          // same regen for now
        }
    }

    /// Species-level needs and consumption profile for deer
    pub fn needs() -> super::SpeciesNeeds {
        super::SpeciesNeeds {
            // Scaled from DF size (deer max 140,000 cm^3) to manageable sim numbers (~5x rabbit)
            hunger_max: 300.0,
            thirst_max: 300.0,
            // Larger consumption per event than rabbit
            eat_amount: 60.0,
            drink_amount: 150.0,
        }
    }

    /// Evaluate Deer actions
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
            evaluate_drinking_behavior, evaluate_eating_behavior, evaluate_follow_behavior,
            evaluate_grazing_behavior, evaluate_resting_behavior,
        };

        let mut actions = Vec::new();

        if let Some(drink) = evaluate_drinking_behavior(
            position,
            thirst,
            world_loader,
            behavior_config.thirst_threshold,
            behavior_config.water_search_radius,
        ) {
            actions.push(drink);
        }

        if let Some(eat) = evaluate_eating_behavior(
            position,
            hunger,
            world_loader,
            behavior_config.hunger_threshold,
            behavior_config.food_search_radius,
        ) {
            actions.push(eat);
        }

        if let Some(rest) =
            evaluate_resting_behavior(position, energy, behavior_config.energy_threshold)
        {
            actions.push(rest);
        }

        if let Some(graze) =
            evaluate_grazing_behavior(position, world_loader, behavior_config.graze_range)
        {
            actions.push(graze);
        }

        actions
    }
}
