/// Rabbit-specific behavior configuration
///
/// Defines behavior parameters optimized for rabbit entities.
use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::herbivore_toolkit::{FollowConfig, MateActionParams, maybe_add_flee_action};
use crate::ai::behaviors::eating::HerbivoreDiet;
use crate::ai::planner::plan_species_actions;
use crate::ai::queue::ActionQueue;
use crate::ai::system_params::PlanningResources;
use crate::entities::entity_types;
use crate::entities::entity_types::{Bear, Fox, Wolf};
use crate::entities::reproduction::{
    birth_common, mate_matching_system_with_relationships, Age,
    Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};
use crate::entities::ActiveMate;
use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::FearState;
use crate::entities::Mother;
use crate::entities::Rabbit;
use crate::entities::TilePosition;
use crate::simulation::SimulationTick;

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
            matching_interval_ticks: 20,  // run matcher every 2s (optimized for breeding)
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
    /// - Moderate territory: 25 tile wander radius (Phase 3: aligned with mating search)
    /// - Habitat: Prefer open grassland, avoid dense forest
    pub fn config() -> BehaviorConfig {
        use super::HabitatPreference;
        BehaviorConfig::new(
            0.75,   // thirst_threshold: Drink when >= 75% thirsty
            0.40,   // hunger_threshold: Eat when >= 40% hungry (enables 300-tick well-fed streak)
            0.3,    // energy_threshold: Rest when energy drops below 30%
            (3, 8), // graze_range: Short-range grazing (3-8 tiles)
            100,    // water_search_radius: Wide water search
            100,    // food_search_radius: Wide food search
            25,     // wander_radius: Moderate territory (Phase 3: aligned with mating search)
        )
        .with_satisfaction(15.0) // Rabbits are not picky - eat nearby grass quickly
        .with_habitat(HabitatPreference::rabbit()) // Prefer grassland, avoid forest
    }

    /// Species-specific stats preset for rabbits (initial values and rates)
    /// Keeps stat components generic, only the preset lives here.
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        use crate::entities::CachedEntityState;
        let needs = Self::needs();
        // Rabbits: higher metabolism — eat/drink more often, tire a bit faster
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.08)), // moderate hunger gain
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.03)), // slower thirst gain to avoid spam-drinking
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.05)), // reduced energy drain to improve movement viability
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.015)),  // Phase 3: 50% faster regen
            cached_state: CachedEntityState::default(),
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
        vegetation_grid: &crate::vegetation::resource_grid::ResourceGrid,
        fear_state: Option<&crate::entities::FearState>,
    ) -> Vec<crate::ai::UtilityScore> {
        // Use rabbit-specific diet preferences
        let diet = HerbivoreDiet::rabbit();

        crate::ai::herbivore_toolkit::evaluate_core_actions(
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            world_loader,
            vegetation_grid,
            fear_state,
            &diet,
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
            Option<&ActiveMate>,
            Option<&ReproductionConfig>,
            Option<&FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
            Option<&crate::ai::failure_memory::ActionFailureMemory>,
        ),
        With<Rabbit>,
    >,
    rabbit_positions: Query<(Entity, &TilePosition), With<Rabbit>>,
    predator_positions: Query<&TilePosition, Or<(With<Wolf>, With<Fox>, With<Bear>)>>,
    resources: PlanningResources,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
    leader_query: Query<&crate::entities::PackLeader>,
    member_query: Query<&crate::entities::PackMember>,
) {
    let loader = resources.world_loader.as_ref();

    // Collect predator positions once for all rabbits
    let predator_pos_list: Vec<IVec2> = predator_positions.iter().map(|pos| pos.tile).collect();

    let _timer =
        crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_rabbit_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &rabbits,
        &rabbit_positions,
        |entity, position, thirst, hunger, energy, behavior, fear_state| {
            let mut actions = RabbitBehavior::evaluate_actions(
                position,
                thirst,
                hunger,
                energy,
                behavior,
                loader,
                &resources.vegetation_grid,
                fear_state,
            );

            // Add flee action if afraid of predators (Phase 3: Explicit Flee Behavior)
            maybe_add_flee_action(
                &mut actions,
                position,
                fear_state,
                &predator_pos_list,
                loader,
            );

            // WARREN DEFENSE: Apply generic group-aware coordination bonuses
            use crate::ai::apply_group_behavior_bonuses;
            apply_group_behavior_bonuses(entity, &mut actions, &leader_query, &member_query);

            actions
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
        resources.current_tick(),
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
            Option<&ActiveMate>,
            &ReproductionConfig,
        ),
        (With<Rabbit>, Or<(Changed<TilePosition>, Changed<ReproductionCooldown>, Changed<Pregnancy>, Changed<WellFedStreak>)>),
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system_with_relationships::<Rabbit, '🐇'>(
        &mut commands,
        &animals,
        tick.0,
    );
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

    #[test]
    fn test_rabbit_stats_bundle_includes_cached_state() {
        let bundle = RabbitBehavior::stats_bundle();

        // Verify all stats are present
        assert_eq!(bundle.hunger.0.max, 70.0);
        assert_eq!(bundle.thirst.0.max, 90.0);
        assert_eq!(bundle.energy.0.current, 100.0);
        assert_eq!(bundle.health.0.current, 100.0);

        // Verify cached state is present
        assert_eq!(bundle.cached_state.tile, bevy::math::IVec2::ZERO);
        // Default cached state starts dirty and needs update on first use
        assert!(bundle.cached_state.dirty);
    }
}
