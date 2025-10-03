/// Tick-based movement system for entities
/// Movement happens discretely on simulation ticks, not smoothly over time
use bevy::prelude::*;

use crate::pathfinding::{Path, PathRequest};

// ============================================================================
// COMPONENTS
// ============================================================================

/// Entity's current tile position (discrete, not interpolated)
#[derive(Component, Debug, Clone, Copy)]
pub struct TilePosition {
    pub tile: IVec2,
}

impl TilePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            tile: IVec2::new(x, y),
        }
    }

    pub fn from_tile(tile: IVec2) -> Self {
        Self { tile }
    }
}

/// Entity wants to move to a destination
#[derive(Component, Debug)]
pub struct MoveOrder {
    pub destination: IVec2,
    pub allow_diagonal: bool,
}

/// Movement speed in tiles per tick
#[derive(Component, Debug, Clone, Copy)]
pub struct MovementSpeed {
    /// How many ticks to wait before moving to next tile
    /// speed=1 means move every tick, speed=2 means move every 2 ticks, etc.
    pub ticks_per_move: u32,
}

impl MovementSpeed {
    /// Fast movement (1 tile per tick)
    pub fn fast() -> Self {
        Self { ticks_per_move: 1 }
    }

    /// Normal movement (1 tile per 2 ticks)
    pub fn normal() -> Self {
        Self { ticks_per_move: 2 }
    }

    /// Slow movement (1 tile per 4 ticks)
    pub fn slow() -> Self {
        Self { ticks_per_move: 4 }
    }

    /// Custom speed
    pub fn custom(ticks_per_move: u32) -> Self {
        Self { ticks_per_move }
    }
}

/// Internal state tracking for movement
#[derive(Component, Debug, Default)]
pub struct MovementState {
    /// Ticks since last movement
    ticks_since_move: u32,
}

// ============================================================================
// SYSTEMS (Non-Tick - runs every frame)
// ============================================================================

/// System: Convert MoveOrder into PathRequest
/// This runs every frame (not tick-synced) to queue pathfinding ASAP
pub fn initiate_pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePosition, &MoveOrder), Without<PathRequest>>,
) {
    for (entity, position, order) in query.iter() {
        // Remove MoveOrder and add PathRequest
        commands
            .entity(entity)
            .remove::<MoveOrder>()
            .insert(PathRequest {
                origin: position.tile,
                destination: order.destination,
                allow_diagonal: order.allow_diagonal,
                max_steps: Some(5000), // Prevent infinite search (needs to be high for fragmented world terrain)
            });

        info!(
            "Entity {:?} requesting path from {:?} to {:?}",
            entity, position.tile, order.destination
        );
    }
}

// ============================================================================
// SYSTEMS (TICK-SYNCED - runs on simulation tick)
// ============================================================================

/// System: Execute movement along path (TICK-SYNCED)
/// This should ONLY run during simulation ticks
pub fn tick_movement_system(
    mut query: Query<(
        Entity,
        &mut TilePosition,
        &MovementSpeed,
        &mut MovementState,
        &mut Path,
    )>,
    mut commands: Commands,
) {
    for (entity, mut position, speed, mut state, mut path) in query.iter_mut() {
        // Increment tick counter
        state.ticks_since_move += 1;

        // Check if enough ticks have passed to move
        if state.ticks_since_move < speed.ticks_per_move {
            continue; // Not time to move yet
        }

        // Reset tick counter
        state.ticks_since_move = 0;

        // Get next target tile
        if let Some(target) = path.current_target() {
            // Move to target
            let old_pos = position.tile;
            position.tile = target;
            path.advance();

            debug!(
                "Entity {:?} moved from {:?} to {:?}, {} waypoints remaining",
                entity.index(),
                old_pos,
                target,
                path.remaining().len()
            );

            // Check if path is complete
            if path.is_complete() {
                info!("Entity {:?} reached destination at {:?}", entity, target);
                commands
                    .entity(entity)
                    .remove::<Path>()
                    .remove::<MovementState>();
            }
        } else {
            // Path is empty but not marked complete (shouldn't happen)
            warn!("Entity {:?} has empty path, removing", entity);
            commands
                .entity(entity)
                .remove::<Path>()
                .remove::<MovementState>();
        }
    }
}

/// System: Initialize movement state when path is assigned
pub fn initialize_movement_state(
    mut commands: Commands,
    query: Query<Entity, (With<Path>, Without<MovementState>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(MovementState::default());
        debug!("Initialized movement state for entity {:?}", entity);
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Issue a move order to an entity
pub fn issue_move_order(commands: &mut Commands, entity: Entity, destination: IVec2) {
    commands.entity(entity).insert(MoveOrder {
        destination,
        allow_diagonal: false,
    });
}

/// Stop entity movement (clears path and orders)
pub fn stop_movement(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<Path>()
        .remove::<MoveOrder>()
        .remove::<PathRequest>()
        .remove::<MovementState>();
}

/// Check if entity is currently moving
pub fn is_moving(entity: Entity, query: &Query<(), (With<Path>, With<MovementState>)>) -> bool {
    query.get(entity).is_ok()
}

/// Get entity's current position
pub fn get_position(entity: Entity, query: &Query<&TilePosition>) -> Option<IVec2> {
    query.get(entity).ok().map(|pos| pos.tile)
}
