pub mod auto_eat;
pub mod current_action;
pub mod entity_tracker;
pub mod entity_types;
/// Entities module - manages creatures and their behaviors
pub mod movement;
pub mod reproduction;
pub mod stats;
pub mod types;

use bevy::prelude::*;

pub use movement::{
    get_position, is_moving, issue_move_order, stop_movement, MoveOrder, MovementSpeed,
    MovementState, TilePosition,
};

// Wandering component REMOVED - use utility AI Wander action instead!

pub use entity_tracker::{get_entities_json, init_entity_tracker, sync_entities_to_tracker};

pub use stats::{
    death_system, get_most_urgent_need, tick_stats_system, utility_drink, utility_eat,
    utility_heal, utility_rest, Energy, EntityStatsBundle, Health, Hunger, Stat, Thirst,
};

pub use entity_types::{
    count_entities_by_type, spawn_deer, spawn_human, spawn_humans, spawn_rabbit, spawn_rabbits,
    Deer, EntityTemplate, Human, Rabbit, Wolf,
};

pub use reproduction::{
    deer_birth_system, deer_mate_matching_system, rabbit_birth_system, rabbit_mate_matching_system,
    tick_reproduction_timers_system, update_age_and_wellfed_system, Age, MatingIntent, Mother,
    Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};

pub use types::{BehaviorConfig, SpeciesNeeds};

pub use current_action::CurrentAction;

// ============================================================================
// ENTITY TYPES
// ============================================================================

/// Basic creature entity
#[derive(Component, Debug)]
pub struct Creature {
    pub name: String,
    pub species: String,
}

/// Marker for different entity types
#[derive(Component, Debug)]
pub enum EntityType {
    Human,
    Animal,
    Monster,
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Startup
            .add_systems(Startup, entity_tracker::init_entity_tracker)
            // Non-tick systems (run every frame)
            .add_systems(
                Update,
                (
                    movement::initiate_pathfinding,
                    movement::initialize_movement_state,
                    entity_tracker::sync_entities_to_tracker, // Sync for web API
                ),
            )
            // Tick systems (run when should_tick is true)
            .add_systems(
                Update,
                (
                    stats::tick_stats_system,        // Update entity stats
                    movement::tick_movement_system,  // Movement execution
                    auto_eat::auto_eat_system,       // Auto-eat when on grass
                    update_age_and_wellfed_system,   // Age and WellFed
                    tick_reproduction_timers_system, // Timers for repro
                    rabbit_mate_matching_system,     // Pairing (rabbits)
                    deer_mate_matching_system,       // Pairing (deer)
                    rabbit_birth_system,             // Rabbit births
                    deer_birth_system,               // Deer births
                    stats::death_system,             // Handle death
                )
                    .run_if(should_run_tick_systems),
            );
    }
}

/// Run condition for tick-based systems
fn should_run_tick_systems(state: Res<crate::simulation::SimulationState>) -> bool {
    state.should_tick
}

// ============================================================================
// SPAWN HELPERS
// ============================================================================

/// Spawn a basic creature at a tile position
pub fn spawn_creature(
    commands: &mut Commands,
    name: impl Into<String>,
    species: impl Into<String>,
    tile_pos: IVec2,
    speed: MovementSpeed,
) -> Entity {
    commands
        .spawn((
            Creature {
                name: name.into(),
                species: species.into(),
            },
            TilePosition::from_tile(tile_pos),
            speed,
        ))
        .id()
}
