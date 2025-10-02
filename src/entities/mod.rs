/// Entities module - manages creatures and their behaviors
pub mod movement;

use bevy::prelude::*;
use crate::pathfinding::PathfindingGrid;

pub use movement::{
    TilePosition, MoveOrder, MovementSpeed, MovementState,
    issue_move_order, stop_movement, is_moving, get_position,
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
            // Non-tick systems (run every frame)
            .add_systems(Update, (
                movement::initiate_pathfinding,
                movement::initialize_movement_state,
            ));
        
        // NOTE: tick_movement_system should be added to your TICK schedule
        // Example in main.rs:
        // .add_systems(SimulationTick, movement::tick_movement_system)
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
