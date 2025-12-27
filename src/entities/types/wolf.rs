//! Wolf-specific behaviour and planner wiring.

use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::planner::plan_species_actions;
use crate::ai::queue::ActionQueue;
use crate::entities::entity_types::{Deer, Wolf};
use crate::entities::reproduction::{
    birth_common, mate_matching_system, mate_matching_system_with_children, Age, MatingIntent,
    Mother, Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};
use crate::entities::{SpatialCell, SpatialCellGrid};
use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::TilePosition;
use crate::entities::{Carcass, FearState};
use crate::simulation::SimulationTick;
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;

/// Wolf behaviour preset (pack predator baseline).
pub struct WolfBehavior;

impl WolfBehavior {
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 14_000,
            gestation_ticks: 4_500,
            mating_cooldown_ticks: 7_000,
            postpartum_cooldown_ticks: 10_000,
            litter_size_range: (2, 4),
            mating_search_radius: 160,
            well_fed_hunger_norm: 0.55,
            well_fed_thirst_norm: 0.5,
            well_fed_required_ticks: 900,
            matching_interval_ticks: 150, // Check every 15s (optimized)
            mating_duration_ticks: 60,
            min_energy_norm: 0.45,
            min_health_norm: 0.6,
        }
    }

    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new_with_foraging(
            0.55,
            0.45,
            0.25,
            (8, 22),
            180,
            220,
            200,
            super::ForagingStrategy::Exhaustive,
        )
    }

    pub fn needs() -> super::SpeciesNeeds {
        super::SpeciesNeeds {
            hunger_max: 260.0,
            thirst_max: 200.0,
            eat_amount: 100.0,
            drink_amount: 120.0,
        }
    }

    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        use crate::entities::CachedEntityState;
        let needs = Self::needs();

        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.05)),
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
        deer: &Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
        vegetation: &ResourceGrid,
    ) -> Vec<crate::ai::UtilityScore> {
        crate::ai::predator_toolkit::evaluate_wolf_actions(
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

pub fn plan_wolf_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    wolves: Query<
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
        With<Wolf>,
    >,
    wolf_positions: Query<(Entity, &TilePosition), With<Wolf>>,
    world_loader: Res<WorldLoader>,
    carcasses: Query<(Entity, &TilePosition, &Carcass)>,
    deer_query: Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
    vegetation_grid: Res<ResourceGrid>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    let loader = world_loader.as_ref();
    let vegetation = vegetation_grid.as_ref();
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_wolf_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &wolves,
        &wolf_positions,
        |entity, position, thirst, hunger, energy, behavior, fear_state| {
            WolfBehavior::evaluate_actions(
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
        None,
        None,
        "🐺",
        "Wolf",
        tick.0,
    );
}

pub fn wolf_mate_matching_system(
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
        (With<Wolf>, Or<(Changed<TilePosition>, Changed<ReproductionCooldown>, Changed<Pregnancy>, Changed<WellFedStreak>)>),
    >,
    grid: Res<SpatialCellGrid>,
    cells: Query<&Children, With<SpatialCell>>,
    tick: Res<SimulationTick>,
) {
    mate_matching_system_with_children::<Wolf, '🐺'>(
        &mut commands,
        &animals,
        &grid,
        &cells,
        tick.0,
    );
}

pub fn wolf_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Wolf>>,
) {
    birth_common::<Wolf>(
        &mut commands,
        &mut mothers,
        |cmds, name, pos| crate::entities::entity_types::spawn_wolf(cmds, name, pos),
        "🐺🍼",
        "Pup",
    );
}
