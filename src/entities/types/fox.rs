//! Fox-specific configuration and planner wiring.

use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::herbivore_toolkit::{FollowConfig, MateActionParams};
use crate::ai::planner::plan_species_actions;
use crate::ai::queue::ActionQueue;
use crate::ai::system_params::PlanningResources;
use crate::entities::entity_types::{Fox, Rabbit};
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

/// Fox behaviour preset (applies to red fox or coyote analogue).
pub struct FoxBehavior;

impl FoxBehavior {
    /// Fast reproduction parameters for foxes (for testing)
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 130,             // ~13 seconds (fast for testing)
            gestation_ticks: 70,             // ~7 seconds
            mating_cooldown_ticks: 50,       // ~5 seconds
            postpartum_cooldown_ticks: 90,   // ~9 seconds
            litter_size_range: (2, 4),       // Kits
            mating_search_radius: 80,
            well_fed_hunger_norm: 0.60,
            well_fed_thirst_norm: 0.60,
            well_fed_required_ticks: 25,  // ~2.5 seconds
            matching_interval_ticks: 12,  // check every 1.2s
            mating_duration_ticks: 20,
            min_energy_norm: 0.40,
            min_health_norm: 0.40,
        }
    }

    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new_with_foraging(
            0.15, // thirst_threshold: Drink when >= 15% thirsty
            0.15, // hunger_threshold: Hunt when >= 15% hungry (lower for testing)
            0.3,
            (5, 14),
            150,
            160,
            40,
            super::ForagingStrategy::Exhaustive,
        )
    }

    pub fn needs() -> super::SpeciesNeeds {
        super::SpeciesNeeds {
            hunger_max: 180.0,
            thirst_max: 150.0,
            eat_amount: 60.0,
            drink_amount: 90.0,
        }
    }

    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        use crate::entities::CachedEntityState;
        let needs = Self::needs();

        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.08)), // Moderate hunger
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.06)), // Moderate thirst
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.06)),
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.015)),
            cached_state: CachedEntityState::default(),
        }
    }

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
        rabbits: &Query<(Entity, &TilePosition, Option<&Age>), With<Rabbit>>,
        vegetation: &ResourceGrid,
    ) -> Vec<crate::ai::UtilityScore> {
        crate::ai::predator_toolkit::evaluate_fox_actions(
            entity,
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            world_loader,
            fear_state,
            carcasses,
            rabbits,
            vegetation,
        )
    }
}

pub fn plan_fox_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    foxes: Query<
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
        With<Fox>,
    >,
    fox_positions: Query<(Entity, &TilePosition), With<Fox>>,
    resources: PlanningResources,
    carcasses: Query<(Entity, &TilePosition, &Carcass)>,
    rabbits: Query<(Entity, &TilePosition, Option<&Age>), With<Rabbit>>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    let loader = resources.world_loader.as_ref();
    let vegetation = resources.vegetation_grid.as_ref();
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_fox_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &foxes,
        &fox_positions,
        |entity, position, thirst, hunger, energy, behavior, fear_state| {
            FoxBehavior::evaluate_actions(
                entity, position, thirst, hunger, energy, behavior, loader, fear_state, &carcasses,
                &rabbits, vegetation,
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
        "🦊",
        "Fox",
        resources.current_tick(),
    );
}

pub fn fox_mate_matching_system(
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
        With<Fox>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system_with_relationships::<Fox, '🦊'>(
        &mut commands,
        &animals,
        tick.0,
    );
}

pub fn fox_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Fox>>,
) {
    birth_common::<Fox>(
        &mut commands,
        &mut mothers,
        |cmds, name, pos| crate::entities::entity_types::spawn_fox(cmds, name, pos),
        "🦊🍼",
        "Kit",
    );
}
