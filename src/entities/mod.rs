/// Entities module - manages creatures and their behaviors
pub mod movement;
pub mod entity_tracker;
pub mod stats;
pub mod entity_types;
pub mod types;
pub mod auto_eat;
pub mod current_action;
pub mod reproduction;

use bevy::prelude::*;

pub use movement::{
    TilePosition, MoveOrder, MovementSpeed, MovementState,
    issue_move_order, stop_movement, is_moving, get_position,
};

// Wandering component REMOVED - use utility AI Wander action instead!

pub use entity_tracker::{
    init_entity_tracker, sync_entities_to_tracker, get_entities_json,
};

pub use stats::{
    Stat, Hunger, Thirst, Energy, Health,
    EntityStatsBundle,
    tick_stats_system, death_system,
    utility_eat, utility_drink, utility_rest, utility_heal,
    get_most_urgent_need,
};

pub use entity_types::{
    Human, Rabbit, Deer, Wolf,
    EntityTemplate,
    spawn_human, spawn_rabbit, spawn_deer,
    spawn_humans, spawn_rabbits,
    count_entities_by_type,
};

pub use reproduction::{
    Sex, Age, ReproductionCooldown, Pregnancy, WellFedStreak, Mother, MatingIntent,
    update_age_and_wellfed_system,
    tick_reproduction_timers_system,
    rabbit_mate_matching_system,
    rabbit_birth_system,
};

pub use types::{
    BehaviorConfig, SpeciesNeeds,
};

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
            .add_systems(Update, (
                movement::initiate_pathfinding,
                movement::initialize_movement_state,
                entity_tracker::sync_entities_to_tracker,  // Sync for web API
            ))
            
            // Tick systems (run when should_tick is true)
.add_systems(Update, (
                stats::tick_stats_system,       // Update entity stats
                movement::tick_movement_system, // Movement execution
                auto_eat::auto_eat_system,      // Auto-eat when on grass
                update_age_and_wellfed_system,  // Age and WellFed
                tick_reproduction_timers_system, // Timers for repro
                rabbit_mate_matching_system,    // Pairing (MVP)
                rabbit_birth_system,            // Handle births
                stats::death_system,            // Handle death
            ).run_if(should_run_tick_systems));
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
    commands.spawn((
        Creature {
            name: name.into(),
            species: species.into(),
        },
        TilePosition::from_tile(tile_pos),
        speed,
    )).id()
}
