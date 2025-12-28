//! Fox-specific configuration and planner wiring.

use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::planner::plan_species_actions;
use crate::ai::queue::ActionQueue;
use crate::ai::system_params::PlanningResources;
use crate::entities::entity_types::{Fox, Rabbit};
use crate::entities::reproduction::{
    birth_common, mate_matching_system_with_relationships, Age,
    Mother, Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};
use crate::entities::ActiveMate;
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
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 10_500, // ~17.5 minutes
            gestation_ticks: 4_500, // 7.5 minutes
            mating_cooldown_ticks: 4_000,
            postpartum_cooldown_ticks: 6_000,
            litter_size_range: (3, 5),
            mating_search_radius: 120,
            well_fed_hunger_norm: 0.5,
            well_fed_thirst_norm: 0.5,
            well_fed_required_ticks: 600,
            matching_interval_ticks: 120, // Check every 12s (optimized)
            mating_duration_ticks: 50,
            min_energy_norm: 0.5,
            min_health_norm: 0.50,
        }
    }

    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new_with_foraging(
            0.5, // prefer to hydrate more often than bears
            0.5, // aggressive hunters when half hungry
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
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.06)),
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.04)),
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
            Option<&ReproductionConfig>,
            Option<&FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
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
        None,
        None,
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
        (With<Fox>, Or<(Changed<TilePosition>, Changed<ReproductionCooldown>, Changed<Pregnancy>, Changed<WellFedStreak>)>),
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
