/// Deer-specific behavior configuration
///
/// Defines behavior parameters and action evaluation for deer entities.
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
use crate::entities::FearState;
use crate::entities::Mother;
use crate::entities::{Deer, TilePosition};
use crate::simulation::SimulationTick;
use crate::world_loader::WorldLoader;

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
    /// Note: Follow-mother behavior is handled by the planner using maybe_add_follow_mother
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
        crate::ai::herbivore_toolkit::evaluate_core_actions(
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            world_loader,
            vegetation_grid,
            fear_state,
        )
    }
}

pub fn plan_deer_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    deer: Query<
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
            Option<&FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
        ),
        With<Deer>,
    >,
    deer_positions: Query<(Entity, &TilePosition), With<Deer>>,
    world_loader: Res<WorldLoader>,
    vegetation_grid: Res<crate::vegetation::resource_grid::ResourceGrid>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    let loader = world_loader.as_ref();

    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_deer_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &deer,
        &deer_positions,
        |_, position, thirst, hunger, energy, behavior, fear_state| {
            DeerBehavior::evaluate_actions(
                position,
                thirst,
                hunger,
                energy,
                behavior,
                loader,
                &vegetation_grid,
                fear_state,
            )
        },
        Some(MateActionParams {
            utility: 0.45,
            priority: 350,
            threshold_margin: 0.05,
            energy_margin: 0.05,
        }),
        Some(FollowConfig {
            stop_distance: 2,
            max_distance: 25,
        }),
        "🦌",
        "Deer",
        tick.0,
    );
}

pub fn deer_mate_matching_system(
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
        With<Deer>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system::<Deer, '🦌'>(&mut commands, &animals, tick.0);
}

pub fn deer_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Deer>>,
) {
    birth_common::<Deer>(
        &mut commands,
        &mut mothers,
        |cmds, name, pos| entity_types::spawn_deer(cmds, name, pos),
        "🦌🍼",
        "Fawn",
    );
}
