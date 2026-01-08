//! Wolf-specific behaviour and planner wiring.

use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::herbivore_toolkit::{FollowConfig, MateActionParams};
use crate::ai::planner::plan_species_actions;
use crate::ai::queue::ActionQueue;
use crate::ai::system_params::PlanningResources;
use crate::entities::entity_types::{Deer, Wolf};
use crate::entities::reproduction::{
    birth_common, mate_matching_system_with_relationships, Age,
    Mother, Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};
use crate::entities::ActiveMate;
use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::TilePosition;
use crate::entities::{Carcass, FearState};
use crate::simulation::SimulationTick;
use crate::vegetation::resource_grid::ResourceGrid;
use crate::vegetation::VegetationSpatialGrid;
use crate::world_loader::WorldLoader;

/// Wolf behaviour preset (pack predator baseline).
pub struct WolfBehavior;

impl WolfBehavior {
    /// Fast reproduction parameters for wolves (for testing)
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 160,             // ~16 seconds (fast for testing)
            gestation_ticks: 80,             // ~8 seconds
            mating_cooldown_ticks: 60,       // ~6 seconds
            postpartum_cooldown_ticks: 100,  // ~10 seconds
            litter_size_range: (2, 3),       // Pups
            mating_search_radius: 60,        // Reduced from 100: Prevents pathfinding failures (150 tile limit)
            well_fed_hunger_norm: 0.65,
            well_fed_thirst_norm: 0.60,
            well_fed_required_ticks: 30,  // ~3 seconds
            matching_interval_ticks: 15,  // check every 1.5s
            mating_duration_ticks: 25,
            min_energy_norm: 0.40,
            min_health_norm: 0.45,
        }
    }

    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new_with_foraging(
            0.15, // thirst_threshold: Drink when >= 15% thirsty
            0.15, // hunger_threshold: Hunt when >= 15% hungry (lower for testing)
            0.25,
            (8, 22),
            180,
            220,
            10,  // wander_radius: Reduced from 200 to prevent pathfinding failures
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
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.08)), // Moderate hunger
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.06)), // Moderate thirst
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.06)),
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.015)),
            cached_state: CachedEntityState::default(),
        }
    }

    /// Evaluate wolf actions via predator toolkit.
    ///
    /// PERFORMANCE: Uses RegionMap for O(1) reachability filtering
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
        spatial_grid: &VegetationSpatialGrid,
        water_grid: &crate::resources::WaterSpatialGrid,
        region_map: &crate::pathfinding::RegionMap,
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
            spatial_grid,
            water_grid,
            region_map,
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
            Option<&ActiveMate>,
            Option<&MatingTarget>,
            Option<&ReproductionConfig>,
            Option<&FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
            Option<&crate::ai::failure_memory::ActionFailureMemory>,
        ),
        With<Wolf>,
    >,
    wolf_positions: Query<(Entity, &TilePosition), With<Wolf>>,
    resources: PlanningResources,
    carcasses: Query<(Entity, &TilePosition, &Carcass)>,
    deer_query: Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
    leader_query: Query<&crate::entities::PackLeader>,
    member_query: Query<&crate::entities::PackMember>,
) {
    let loader = resources.world_loader.as_ref();
    let vegetation = resources.vegetation_grid.as_ref();
    let spatial_grid = resources.vegetation_spatial_grid.as_ref();
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_wolf_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &wolves,
        &wolf_positions,
        |entity, position, thirst, hunger, energy, behavior, fear_state| {
            let mut actions = WolfBehavior::evaluate_actions(
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
                spatial_grid,
                &resources.water_spatial_grid,
                &resources.region_map,
            );

            // PACK TACTICS: Apply generic group-aware coordination bonuses
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
            max_distance: 30,
        }),
        "🐺",
        "Wolf",
        resources.current_tick(),
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
            Option<&ActiveMate>,
            &ReproductionConfig,
        ),
        With<Wolf>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system_with_relationships::<Wolf, '🐺'>(
        &mut commands,
        &animals,
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
        crate::entities::entity_types::spawn_wolf,
        "🐺🍼",
        "Pup",
    );
}
