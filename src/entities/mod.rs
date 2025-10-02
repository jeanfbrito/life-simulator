/// Entities module - manages creatures and their behaviors
pub mod movement;
pub mod wandering;
pub mod entity_tracker;

use bevy::prelude::*;

pub use movement::{
    TilePosition, MoveOrder, MovementSpeed, MovementState,
    issue_move_order, stop_movement, is_moving, get_position,
};

pub use wandering::{
    Wanderer, wanderer_ai_system,
    spawn_wandering_person, spawn_wandering_people,
};

pub use entity_tracker::{
    init_entity_tracker, sync_entities_to_tracker, get_entities_json,
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
            .add_systems(Update, (
                movement::initiate_pathfinding,
                movement::initialize_movement_state,
                entity_tracker::sync_entities_to_tracker,  // Sync for web API
            ))
            
            // Tick systems (run on fixed timestep)
            .add_systems(FixedUpdate, (
                wandering::wanderer_ai_system,  // AI runs on ticks
                movement::tick_movement_system, // Movement execution
            ).chain());
        
        // NOTE: Systems added to FixedUpdate will run at tick rate (default 10 TPS)
    }
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
