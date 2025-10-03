/// Rabbit-specific behavior configuration
///
/// Defines behavior parameters optimized for rabbit entities.
use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::herbivore_toolkit::{FollowConfig, MateActionParams};
use crate::ai::planner::plan_species_actions;
use crate::ai::queue::ActionQueue;
use crate::entities::entity_types;
use crate::entities::reproduction::{
    birth_common, mate_matching_system, Age, MatingIntent, Pregnancy, ReproductionConfig,
    ReproductionCooldown, Sex, WellFedStreak,
};
use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::Mother;
use crate::entities::Rabbit;
use crate::entities::TilePosition;
use crate::simulation::SimulationTick;
use crate::world_loader::WorldLoader;

/// Rabbit behavior preset
pub struct RabbitBehavior;

impl RabbitBehavior {
    /// Default reproduction parameters for rabbits
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 3600,            // ~6 minutes at 10 TPS
            gestation_ticks: 1200,           // ~2 minutes
            mating_cooldown_ticks: 600,      // ~1 minute (male)
            postpartum_cooldown_ticks: 1800, // ~3 minutes (female)
            litter_size_range: (2, 6),
            mating_search_radius: 50,
            well_fed_hunger_norm: 0.35,
            well_fed_thirst_norm: 0.35,
            well_fed_required_ticks: 300, // ~30s sustained
            matching_interval_ticks: 50,  // run matcher every 5s
            mating_duration_ticks: 30,    // ~3s mating interaction
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
            0.75,   // thirst_threshold: Drink when >= 75% thirsty
            0.5,    // hunger_threshold: Eat when >= 50% hungry
            0.3,    // energy_threshold: Rest when energy drops below 30%
            (3, 8), // graze_range: Short-range grazing (3-8 tiles)
            100,    // water_search_radius: Wide water search
            100,    // food_search_radius: Wide food search
            15,     // wander_radius: Small territory
        )
    }

    /// Species-specific stats preset for rabbits (initial values and rates)
    /// Keeps stat components generic, only the preset lives here.
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        let needs = Self::needs();
        // Rabbits: higher metabolism — eat/drink more often, tire a bit faster
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.08)), // moderate hunger gain
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.03)), // slower thirst gain to avoid spam-drinking
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.07)), // slightly faster energy drain
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.01)),  // same regen for now
        }
    }

    /// Species-level needs and consumption profile for rabbits
    pub fn needs() -> super::SpeciesNeeds {
        super::SpeciesNeeds {
            // Scaled from DF size (rabbit max 500 cm^3) to manageable sim numbers
            hunger_max: 70.0,
            thirst_max: 90.0,
            // Increase per-event consumption so rabbits eat/drink more completely
            eat_amount: 28.0,
            drink_amount: 70.0,
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
        crate::ai::herbivore_toolkit::evaluate_core_actions(
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            world_loader,
        )
    }
}

pub fn plan_rabbit_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    rabbits: Query<
        (
            Entity,
            &TilePosition,
            &Thirst,
            &Hunger,
            &Energy,
            &BehaviorConfig,
            Option<&Age>,
            Option<&Mother>,
            Option<&MatingIntent>,
            Option<&ReproductionConfig>,
        ),
        With<Rabbit>,
    >,
    rabbit_positions: Query<(Entity, &TilePosition), With<Rabbit>>,
    world_loader: Res<WorldLoader>,
    tick: Res<SimulationTick>,
) {
    let loader = world_loader.as_ref();

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &rabbits,
        &rabbit_positions,
        |_, position, thirst, hunger, energy, behavior| {
            RabbitBehavior::evaluate_actions(position, thirst, hunger, energy, behavior, loader)
        },
        Some(MateActionParams {
            utility: 0.45,
            priority: 350,
            threshold_margin: 0.05,
            energy_margin: 0.05,
        }),
        Some(FollowConfig {
            stop_distance: 2,
            max_distance: 20,
        }),
        "🐇",
        "Rabbit",
        tick.0,
    );
}

pub fn rabbit_mate_matching_system(
    mut commands: Commands,
    animals: Query<
        (
            Entity,
            &TilePosition,
            &Age,
            &ReproductionCooldown,
            &Energy,
            &Health,
            &WellFedStreak,
            Option<&Pregnancy>,
            Option<&Sex>,
            Option<&MatingIntent>,
            &ReproductionConfig,
        ),
        With<Rabbit>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system::<Rabbit, '🐇'>(&mut commands, &animals, tick.0);
}

pub fn rabbit_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Rabbit>>,
) {
    birth_common::<Rabbit>(
        &mut commands,
        &mut mothers,
        |cmds, name, pos| entity_types::spawn_rabbit(cmds, name, pos),
        "🐇🍼",
        "Kit",
    );
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
