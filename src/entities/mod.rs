pub mod auto_eat;
pub mod current_action;
pub mod entity_tracker;
pub mod entity_types;
/// Entities module - manages creatures and their behaviors
pub mod movement;
pub mod registry;
pub mod reproduction;
pub mod spawn_config;
pub mod stats;
pub mod systems_registry;
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
    spawn_raccoon, Deer, EntityTemplate, Human, Rabbit, Raccoon, Wolf,
};

pub use registry::{
    spawn_deer_legacy, spawn_human_legacy, spawn_humans_legacy, spawn_rabbit_legacy,
    spawn_rabbits_legacy, spawn_raccoon_legacy, spawn_species_batch, spawn_using_registry,
    SpeciesDescriptor, SpeciesRegistry, SPECIES_REGISTRY,
};

/// Generate JSON metadata for all species for viewer configuration
pub fn get_species_metadata_json() -> String {
    use crate::entities::registry::SPECIES_REGISTRY;
    use std::collections::HashMap;

    let mut species_data: HashMap<String, serde_json::Value> = HashMap::new();

    for descriptor in SPECIES_REGISTRY.get_descriptors() {
        let species_info = serde_json::json!({
            "name": descriptor.species,
            "emoji": descriptor.emoji,
            "viewer_scale": descriptor.viewer_scale,
            "viewer_order": descriptor.viewer_order,
            "viewer_color": descriptor.viewer_color,
            "name_prefix": descriptor.name_prefix,
            "juvenile_name_prefix": descriptor.juvenile_name_prefix,
            "is_juvenile": descriptor.is_juvenile,
            "movement_speed": descriptor.movement_speed,
            "wander_radius": descriptor.wander_radius
        });

        species_data.insert(descriptor.species.to_string(), species_info);
    }

    // Add juvenile scaling information that was previously hard-coded in renderer
    let juvenile_scales = serde_json::json!({
        "Rabbit": 0.7,
        "Deer": 0.8,
        "Raccoon": 0.75
    });

    let result = serde_json::json!({
        "species": species_data,
        "juvenile_scales": juvenile_scales,
        "default_entity": {
            "emoji": "❓",
            "sizeMultiplier": 1.0,
            "offsetX": 0.0,
            "offsetY": -0.2
        }
    });

    result.to_string()
}

pub use systems_registry::{
    get_birth_system_names, get_mate_matching_system_names, get_planner_system_names,
    SpeciesSystemsDescriptor, SpeciesSystemsRegistry, SPECIES_SYSTEMS_REGISTRY,
};

pub use reproduction::{
    mate_matching_system, tick_reproduction_timers_system, update_age_and_wellfed_system, Age,
    MatingIntent, Mother, Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};

pub use types::deer::{deer_birth_system, deer_mate_matching_system};
pub use types::rabbit::{rabbit_birth_system, rabbit_mate_matching_system};
pub use types::raccoon::{raccoon_birth_system, raccoon_mate_matching_system};
pub use types::{BehaviorConfig, SpeciesNeeds};

pub use current_action::CurrentAction;

pub use spawn_config::{
    spawn_entities_from_config, SpawnArea, SpawnConfig, SpawnGroup, SpawnMessages, SpawnSettings,
    SpawnSex,
};

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
                    raccoon_mate_matching_system,    // Pairing (raccoons)
                    rabbit_birth_system,             // Rabbit births
                    deer_birth_system,               // Deer births
                    raccoon_birth_system,            // Raccoon births
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
