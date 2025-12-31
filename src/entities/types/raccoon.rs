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
use crate::entities::{ActiveMate, MatingTarget};
use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::FearState;
use crate::entities::Mother;
use crate::entities::{Raccoon, TilePosition};
use crate::simulation::SimulationTick;

/// Raccoon behavior preset
pub struct RaccoonBehavior;

impl RaccoonBehavior {
    /// Fast reproduction parameters for raccoons (for testing)
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 120,             // ~12 seconds (fast for testing)
            gestation_ticks: 60,             // ~6 seconds
            mating_cooldown_ticks: 40,       // ~4 seconds
            postpartum_cooldown_ticks: 80,   // ~8 seconds
            litter_size_range: (2, 3),       // Kits
            mating_search_radius: 35,        // Reduced from 50: Prevents pathfinding failures (150 tile limit)
            well_fed_hunger_norm: 0.60,
            well_fed_thirst_norm: 0.60,
            well_fed_required_ticks: 20,  // ~2 seconds
            matching_interval_ticks: 12,  // check every 1.2s
            mating_duration_ticks: 18,    // ~1.8s together
            min_energy_norm: 0.35,
            min_health_norm: 0.35,
        }
    }

    /// Behavior configuration
    /// Raccoons are generalists that prefer forest and wetlands
    pub fn config() -> BehaviorConfig {
        use super::HabitatPreference;
        BehaviorConfig::new(
            0.20,    // thirst_threshold: Drink when >= 20% thirsty
            0.45,    // hunger_threshold
            0.30,    // energy_threshold
            (4, 12), // graze/forage range (semi-opportunistic)
            120,     // water search radius
            120,     // food search radius (they roam)
            10,      // wander_radius: Reduced from 25 to prevent pathfinding failures
        )
        .with_satisfaction(20.0) // Raccoons are opportunistic but have some standards
        .with_habitat(HabitatPreference::raccoon()) // Prefer forest, wetlands
    }

    /// Stats preset for raccoons
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        use crate::entities::CachedEntityState;
        let needs = Self::needs();
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.06)),
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.04)),
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.05)),
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.015)),
            cached_state: CachedEntityState::default(),
        }
    }

    /// Species-level needs
    pub fn needs() -> super::SpeciesNeeds {
        super::SpeciesNeeds {
            hunger_max: 180.0,
            thirst_max: 150.0,
            eat_amount: 45.0,
            drink_amount: 90.0,
        }
    }

    /// Evaluate raccoon actions using shared herbivore logic
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
        // Use raccoon diet preferences (generalist but prefers shrubs)
        let diet = HerbivoreDiet::new(0.6, 0.8, 12.0); // Balanced preference with shrub emphasis

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

pub fn plan_raccoon_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    raccoons: Query<
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
        With<Raccoon>,
    >,
    raccoon_positions: Query<(Entity, &TilePosition), With<Raccoon>>,
    predator_positions: Query<&TilePosition, Or<(With<Wolf>, With<Fox>, With<Bear>)>>,
    resources: PlanningResources,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
    leader_query: Query<&crate::entities::PackLeader>,
    member_query: Query<&crate::entities::PackMember>,
) {
    let loader = resources.world_loader.as_ref();

    // Collect predator positions once for all raccoons
    let predator_pos_list: Vec<IVec2> = predator_positions.iter().map(|pos| pos.tile).collect();

    let _timer =
        crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_raccoon_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &raccoons,
        &raccoon_positions,
        |entity, position, thirst, hunger, energy, behavior, fear_state| {
            let mut actions = RaccoonBehavior::evaluate_actions(
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

            // Add flee action if afraid of predators
            maybe_add_flee_action(
                &mut actions,
                position,
                fear_state,
                &predator_pos_list,
                loader,
            );

            // Apply generic group-aware coordination bonuses
            use crate::ai::apply_group_behavior_bonuses;
            apply_group_behavior_bonuses(entity, &mut actions, &leader_query, &member_query);

            actions
        },
        Some(MateActionParams {
            utility: 0.42,
            priority: 320,
            threshold_margin: 0.05,
            energy_margin: 0.05,
        }),
        Some(FollowConfig {
            stop_distance: 2,
            max_distance: 18,
        }),
        "🦝",
        "Raccoon",
        resources.current_tick(),
    );
}

pub fn raccoon_mate_matching_system(
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
        With<Raccoon>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system_with_relationships::<Raccoon, '🦝'>(
        &mut commands,
        &animals,
        tick.0,
    );
}

pub fn raccoon_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Raccoon>>,
) {
    birth_common::<Raccoon>(
        &mut commands,
        &mut mothers,
        entity_types::spawn_raccoon,
        "🦝🍼",
        "Kit",
    );
}
