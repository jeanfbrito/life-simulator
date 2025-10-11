use super::BehaviorConfig;
use bevy::prelude::*;

use crate::ai::herbivore_toolkit::{FollowConfig, MateActionParams};
use crate::ai::behaviors::eating::HerbivoreDiet;
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
use crate::entities::{Raccoon, TilePosition};
use crate::simulation::SimulationTick;
use crate::world_loader::WorldLoader;

/// Raccoon behavior preset
pub struct RaccoonBehavior;

impl RaccoonBehavior {
    /// Reproduction parameters for raccoons
    pub fn reproduction_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 6_000,            // ~10 minutes to maturity
            gestation_ticks: 3_600,           // ~6 minutes pregnant
            mating_cooldown_ticks: 1_800,     // ~3 minutes between matings (male)
            postpartum_cooldown_ticks: 5_400, // ~9 minutes recovery (female)
            litter_size_range: (2, 4),        // Raccoons usually have litters of 2-4
            mating_search_radius: 50,         // Comfortable search radius
            well_fed_hunger_norm: 0.5,
            well_fed_thirst_norm: 0.5,
            well_fed_required_ticks: 480, // ~48 seconds well-fed streak
            matching_interval_ticks: 240, // Evaluate partners every ~24 seconds
            mating_duration_ticks: 40,    // ~4 seconds together
            min_energy_norm: 0.4,
            min_health_norm: 0.4,
        }
    }

    /// Behavior configuration
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new(
            0.55,    // thirst_threshold: raccoons tolerate thirst a bit longer
            0.45,    // hunger_threshold
            0.30,    // energy_threshold
            (4, 12), // graze/forage range (semi-opportunistic)
            120,     // water search radius
            120,     // food search radius (they roam)
            25,      // wander radius
        )
    }

    /// Stats preset for raccoons
    pub fn stats_bundle() -> crate::entities::stats::EntityStatsBundle {
        use crate::entities::stats::{Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst};
        let needs = Self::needs();
        EntityStatsBundle {
            hunger: Hunger(Stat::new(0.0, 0.0, needs.hunger_max, 0.06)),
            thirst: Thirst(Stat::new(0.0, 0.0, needs.thirst_max, 0.04)),
            energy: Energy(Stat::new(100.0, 0.0, 100.0, -0.05)),
            health: Health(Stat::new(100.0, 0.0, 100.0, 0.01)),
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
            Option<&MatingIntent>,
            Option<&ReproductionConfig>,
            Option<&FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
        ),
        With<Raccoon>,
    >,
    raccoon_positions: Query<(Entity, &TilePosition), With<Raccoon>>,
    world_loader: Res<WorldLoader>,
    vegetation_grid: Res<crate::vegetation::resource_grid::ResourceGrid>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    let loader = world_loader.as_ref();

    let _timer =
        crate::simulation::profiler::ScopedTimer::new(&mut profiler, "plan_raccoon_actions");

    plan_species_actions(
        &mut commands,
        queue.as_mut(),
        &raccoons,
        &raccoon_positions,
        |_, position, thirst, hunger, energy, behavior, fear_state| {
            RaccoonBehavior::evaluate_actions(
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
        tick.0,
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
            Option<&MatingIntent>,
            &ReproductionConfig,
        ),
        With<Raccoon>,
    >,
    tick: Res<SimulationTick>,
) {
    mate_matching_system::<Raccoon, '🦝'>(&mut commands, &animals, tick.0);
}

pub fn raccoon_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<Raccoon>>,
) {
    birth_common::<Raccoon>(
        &mut commands,
        &mut mothers,
        |cmds, name, pos| entity_types::spawn_raccoon(cmds, name, pos),
        "🦝🍼",
        "Kit",
    );
}
