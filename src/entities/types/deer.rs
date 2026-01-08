/// Deer-specific behavior configuration
///
/// Defines behavior parameters and action evaluation for deer entities.
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
use crate::entities::{Deer, TilePosition};
use crate::simulation::SimulationTick;

/// Deer behavior preset
pub struct DeerBehavior;

impl DeerBehavior {
    /// Fast reproduction parameters for deer (for testing)
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 150,             // ~15 seconds (fast for testing)
            gestation_ticks: 80,             // ~8 seconds
            mating_cooldown_ticks: 50,       // ~5 seconds
            postpartum_cooldown_ticks: 100,  // ~10 seconds
            litter_size_range: (1, 2),       // Fawns
            mating_search_radius: 40,        // Reduced from 60: Prevents pathfinding failures (150 tile limit)
            well_fed_hunger_norm: 0.65,
            well_fed_thirst_norm: 0.65,
            well_fed_required_ticks: 25,  // ~2.5 seconds
            matching_interval_ticks: 15,  // check every 1.5s
            mating_duration_ticks: 20,    // ~2s together
            min_energy_norm: 0.35,
            min_health_norm: 0.35,
        }
    }

    /// Get the default behavior configuration for deer
    /// Deer prefer forest edges with access to meadows
    pub fn config() -> BehaviorConfig {
        use super::HabitatPreference;
        BehaviorConfig::new(
            0.20,    // thirst_threshold: Drink when >= 20% thirsty
            0.45,    // hunger_threshold (eat less frequently)
            0.30,    // energy_threshold
            (5, 15), // graze_range
            150,     // water_search_radius
            150,     // food_search_radius
            10,      // wander_radius: Reduced from 20 to prevent pathfinding failures on fragmented terrain
        )
        .with_satisfaction(40.0) // Deer are selective - search for quality patches
        .with_habitat(HabitatPreference::deer()) // Prefer forest edges, meadows
    }

    /// Species-specific stats preset for deer (initial values and rates)
    /// Keeps stat components generic, only the preset lives here.
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        use crate::entities::CachedEntityState;
        let needs = Self::needs();
        // Deer: lower metabolism — eat/drink less often, tire a bit slower
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.05)), // slower hunger gain
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.02)), // much slower thirst gain
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.04)),         // slower energy drain
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.015)),          // Phase 3: 50% faster regen
            cached_state: CachedEntityState::default(),
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
    ///
    /// PERFORMANCE: Uses spatial grids for O(k) lookups instead of O(radius²)
    /// - RegionMap for O(1) reachability filtering (eliminates pathfinding failures)
    pub fn evaluate_actions(
        position: &crate::entities::TilePosition,
        thirst: &crate::entities::stats::Thirst,
        hunger: &crate::entities::stats::Hunger,
        energy: &crate::entities::stats::Energy,
        behavior_config: &BehaviorConfig,
        world_loader: &crate::world_loader::WorldLoader,
        vegetation_grid: &crate::vegetation::resource_grid::ResourceGrid,
        spatial_grid: &crate::vegetation::VegetationSpatialGrid,
        water_grid: &crate::resources::WaterSpatialGrid,
        region_map: &crate::pathfinding::RegionMap,
        fear_state: Option<&crate::entities::FearState>,
    ) -> Vec<crate::ai::UtilityScore> {
        // Use deer-specific diet preferences
        let diet = HerbivoreDiet::deer();

        crate::ai::herbivore_toolkit::evaluate_core_actions(
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            world_loader,
            vegetation_grid,
            spatial_grid,
            water_grid,
            region_map,
            fear_state,
            &diet,
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
            Option<&ActiveMate>,
            Option<&MatingTarget>,
            Option<&ReproductionConfig>,
            Option<&FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
            Option<&crate::ai::failure_memory::ActionFailureMemory>,
        ),
        With<Deer>,
    >,
    deer_positions: Query<(Entity, &TilePosition), With<Deer>>,
    predator_positions: Query<&TilePosition, Or<(With<Wolf>, With<Fox>, With<Bear>)>>,
    resources: PlanningResources,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
    leader_query: Query<&crate::entities::PackLeader>,
    member_query: Query<&crate::entities::PackMember>,
) {
    let loader = resources.world_loader.as_ref();

    // Collect predator positions once for all deer
    let predator_pos_list: Vec<IVec2> = predator_positions.iter().map(|pos| pos.tile).collect();

    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_deer_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &deer,
        &deer_positions,
        |entity, position, thirst, hunger, energy, behavior, fear_state| {
            let mut actions = DeerBehavior::evaluate_actions(
                position,
                thirst,
                hunger,
                energy,
                behavior,
                loader,
                &resources.vegetation_grid,
                &resources.vegetation_spatial_grid,
                &resources.water_spatial_grid,
                &resources.region_map,
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

            // HERD GRAZING: Apply generic group-aware coordination bonuses
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
            max_distance: 25,
        }),
        "🦌",
        "Deer",
        resources.current_tick(),
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
            Option<&ActiveMate>,
            &ReproductionConfig,
        ),
        With<Deer>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system_with_relationships::<Deer, '🦌'>(
        &mut commands,
        &animals,
        tick.0,
    );
}

pub fn deer_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Deer>>,
) {
    birth_common::<Deer>(
        &mut commands,
        &mut mothers,
        entity_types::spawn_deer,
        "🦌🍼",
        "Fawn",
    );
}
