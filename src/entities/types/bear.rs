//! Bear-specific stats, behaviour configuration, and planner wiring.

use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::herbivore_toolkit::{FollowConfig, MateActionParams};
use crate::ai::planner::plan_species_actions;
use crate::ai::queue::ActionQueue;
use crate::ai::system_params::PlanningResources;
use crate::entities::entity_types::{Bear, Deer};
use crate::entities::reproduction::{
    birth_common, mate_matching_system_with_relationships, Age,
    Mother, Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};
use crate::entities::{ActiveMate, MatingTarget};
use crate::entities::{SpatialCell, SpatialCellGrid};
use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::TilePosition;
use crate::entities::{Carcass, FearState};
use crate::simulation::SimulationTick;
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;

/// Bear behaviour preset with omnivore+scavenger leaning parameters.
pub struct BearBehavior;

impl BearBehavior {
    /// Fast reproduction parameters for bears (for testing)
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 180,             // ~18 seconds (fast for testing)
            gestation_ticks: 100,            // ~10 seconds
            mating_cooldown_ticks: 70,       // ~7 seconds
            postpartum_cooldown_ticks: 120,  // ~12 seconds
            litter_size_range: (1, 2),       // Cubs
            mating_search_radius: 80,
            well_fed_hunger_norm: 0.60,
            well_fed_thirst_norm: 0.60,
            well_fed_required_ticks: 30,  // ~3 seconds
            matching_interval_ticks: 15,  // check every 1.5s
            mating_duration_ticks: 25,
            min_energy_norm: 0.45,
            min_health_norm: 0.45,
        }
    }

    /// Core behavioural thresholds for bears.
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new_with_foraging(
            0.15,    // thirst_threshold: Drink when >= 15% thirsty
            0.15,    // hunger_threshold: Forage/hunt when >= 15% hungry (lower for testing)
            0.3,     // rest when energy below 30%
            (6, 18), // forage radius when sampling plants
            150,     // water search radius
            150,     // food search radius (shared with scavenging grid)
            80,      // roaming radius (large territory)
            super::ForagingStrategy::Sampled { sample_size: 6 },
        )
    }

    /// Species-level needs and consumption profile.
    pub fn needs() -> super::SpeciesNeeds {
        super::SpeciesNeeds {
            hunger_max: 350.0,
            thirst_max: 250.0,
            eat_amount: 120.0,
            drink_amount: 150.0,
        }
    }

    /// Preconfigured stats bundle tuned for the bear metabolism.
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        use crate::entities::CachedEntityState;
        let needs = Self::needs();

        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.08)), // Moderate hunger
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.06)), // Moderate thirst
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.05)),
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.015)),
            cached_state: CachedEntityState::default(),
        }
    }

    /// Evaluate bear actions via the predator toolkit (filled in later steps).
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_actions(
        entity: Entity,
        position: &TilePosition,
        thirst: &Thirst,
        hunger: &Hunger,
        energy: &Energy,
        behavior_config: &BehaviorConfig,
        world_loader: &WorldLoader,
        fear_state: Option<&FearState>,
        carcasses: &Query<(Entity, &TilePosition, &Carcass)>,
        deer: &Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
        vegetation: &ResourceGrid,
    ) -> Vec<crate::ai::UtilityScore> {
        crate::ai::predator_toolkit::evaluate_bear_actions(
            entity,
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            world_loader,
            fear_state,
            carcasses,
            deer,
            vegetation,
        )
    }
}

/// Planner entry point for bear entities.
pub fn plan_bear_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    bears: Query<
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
            Option<&MatingTarget>,
            Option<&ReproductionConfig>,
            Option<&FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
            Option<&crate::ai::failure_memory::ActionFailureMemory>,
        ),
        With<Bear>,
    >,
    bear_positions: Query<(Entity, &TilePosition), With<Bear>>,
    resources: PlanningResources,
    carcasses: Query<(Entity, &TilePosition, &Carcass)>,
    deer_query: Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    let loader = resources.world_loader.as_ref();
    let vegetation = resources.vegetation_grid.as_ref();
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_bear_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &bears,
        &bear_positions,
        |entity, position, thirst, hunger, energy, behavior, fear_state| {
            BearBehavior::evaluate_actions(
                entity,
                position,
                thirst,
                hunger,
                energy,
                behavior,
                loader,
                fear_state,
                &carcasses,
                &deer_query,
                vegetation,
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
            max_distance: 35,
        }),
        "🐻",
        "Bear",
        resources.current_tick(),
    );
}

/// Bear mate-matching uses the generic reproduction helper.
/// Runs at matching_interval_ticks frequency (configured in ReproductionConfig).
pub fn bear_mate_matching_system(
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
        With<Bear>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system_with_relationships::<Bear, '🐻'>(
        &mut commands,
        &animals,
        tick.0,
    );
}

/// Bear birth system wrapper.
pub fn bear_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Bear>>,
) {
    birth_common::<Bear>(
        &mut commands,
        &mut mothers,
        |cmds, name, pos| crate::entities::entity_types::spawn_bear(cmds, name, pos),
        "🐻🍼",
        "Cub",
    );
}
