/// Tick-based movement system for entities
/// Movement happens discretely on simulation ticks, not smoothly over time
use bevy::prelude::*;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::component::HookContext;

use crate::pathfinding::{GridPathRequest, Path};

// ============================================================================
// COMPONENTS
// ============================================================================

/// Entity's current tile position (discrete, not interpolated)
///
/// Phase 7: Component Hooks for Spatial Index
/// This component has on_add and on_insert hooks that automatically synchronize
/// the entity's position with the spatial hierarchy (Parent/Child relationships
/// with SpatialCell entities). This eliminates the need for manual update systems.
#[derive(Component, Debug, Clone, Copy, Default)]
#[component(on_add = on_tile_position_add)]
#[component(on_insert = on_tile_position_insert)]
pub struct TilePosition {
    pub tile: IVec2,
}

impl TilePosition {
    #[inline(always)]
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            tile: IVec2::new(x, y),
        }
    }

    #[inline(always)]
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
///
/// Phase 4: Required Components
/// MovementSpeed automatically requires Creature - compile-time guarantee.
#[derive(Component, Debug, Clone, Copy)]
#[require(crate::entities::Creature)]
pub struct MovementSpeed {
    /// How many ticks to wait before moving to next tile
    /// speed=1 means move every tick, speed=2 means move every 2 ticks, etc.
    pub ticks_per_move: u32,
}

impl MovementSpeed {
    /// Fast movement (1 tile per tick)
    #[inline(always)]
    pub fn fast() -> Self {
        Self { ticks_per_move: 1 }
    }

    /// Normal movement (1 tile per 2 ticks)
    #[inline(always)]
    pub fn normal() -> Self {
        Self { ticks_per_move: 2 }
    }

    /// Slow movement (1 tile per 4 ticks)
    #[inline(always)]
    pub fn slow() -> Self {
        Self { ticks_per_move: 4 }
    }

    /// Custom speed
    #[inline(always)]
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
// COMPONENT HOOKS - Phase 7: Automatic Spatial Index Synchronization
// ============================================================================

/// Hook fired when TilePosition is first added to an entity
///
/// Automatically reparents the entity to its correct spatial cell
/// in the spatial hierarchy. This eliminates the need for a separate
/// reparent_entities_to_cells system.
fn on_tile_position_add(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    // Get the position value
    let position = match world.get::<TilePosition>(entity) {
        Some(pos) => pos.tile,
        None => return, // Entity already despawned?
    };

    // Get spatial grid resource
    let grid = match world.get_resource::<super::spatial_cell::SpatialCellGrid>() {
        Some(grid) => grid,
        None => return, // Grid not initialized yet
    };

    // Calculate chunk coordinate
    let chunk_coord = grid.chunk_coord_for_position(position);

    // Get cell entity from grid
    let cell_entity = match grid.get_cell(chunk_coord) {
        Some(cell) => cell,
        None => return, // Cell doesn't exist (out of bounds)
    };

    drop(grid); // Release resource borrow

    // Queue deferred commands to reparent entity to cell
    let mut commands = world.commands();
    commands.entity(cell_entity).add_child(entity);
    commands.entity(entity).insert(super::spatial_cell::SpatiallyParented);
}

/// Hook fired when TilePosition is inserted (including updates)
///
/// Detects if entity moved to a different chunk and reparents if needed.
/// Optimizes away reparenting when entity stays in same chunk.
fn on_tile_position_insert(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    // Get the new position value
    let new_position = match world.get::<TilePosition>(entity) {
        Some(pos) => pos.tile,
        None => return,
    };

    // Get spatial grid resource
    let grid = match world.get_resource::<super::spatial_cell::SpatialCellGrid>() {
        Some(grid) => grid,
        None => return,
    };

    // Calculate new chunk coordinate
    let new_chunk = grid.chunk_coord_for_position(new_position);

    // Get the entity's current parent (if any)
    let current_parent = match world.get::<ChildOf>(entity) {
        Some(child_of) => child_of.parent(),
        None => return, // Not yet in hierarchy (shouldn't happen with on_add hook)
    };

    // Get current cell's chunk coordinate
    let current_chunk = match world.get::<super::spatial_cell::SpatialCell>(current_parent) {
        Some(cell) => cell.chunk_coord,
        None => return, // Parent isn't a spatial cell?
    };

    // If still in same chunk, no reparenting needed (optimization)
    if current_chunk == new_chunk {
        return;
    }

    // Get new cell entity
    let new_cell_entity = match grid.get_cell(new_chunk) {
        Some(cell) => cell,
        None => return, // Cell doesn't exist (out of bounds)
    };

    drop(grid); // Release resource borrow

    // Queue deferred command to reparent to new cell
    let mut commands = world.commands();
    commands.entity(new_cell_entity).add_child(entity);
}

// ============================================================================
// SYSTEMS (Non-Tick - runs every frame)
// ============================================================================

/// System: Convert MoveOrder into GridPathRequest
/// This runs every frame (not tick-synced) to queue pathfinding ASAP
pub fn initiate_pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePosition, &MoveOrder), Without<GridPathRequest>>,
) {
    for (entity, position, order) in query.iter() {
        // Remove MoveOrder and add GridPathRequest
        commands
            .entity(entity)
            .remove::<MoveOrder>()
            .insert(GridPathRequest {
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
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    use crate::simulation::profiler::end_timing_resource;
    use crate::simulation::profiler::start_timing_resource;

    start_timing_resource(&mut profiler, "movement");

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

    end_timing_resource(&mut profiler, "movement");
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
#[inline]
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
        .remove::<GridPathRequest>()
        .remove::<MovementState>();
}

/// Check if entity is currently moving
#[inline]
pub fn is_moving(entity: Entity, query: &Query<(), (With<Path>, With<MovementState>)>) -> bool {
    query.get(entity).is_ok()
}

/// Get entity's current position
#[inline]
pub fn get_position(entity: Entity, query: &Query<&TilePosition>) -> Option<IVec2> {
    query.get(entity).ok().map(|pos| pos.tile)
}

// ============================================================================
// MOVEMENT COMPONENT SYSTEM (Phase 3 - ECS Architecture Improvement)
// ============================================================================

/// Tracks movement timing for MovementComponent-based movement
#[derive(Component, Debug, Default)]
pub struct MovementTick {
    /// Ticks since last movement
    pub ticks_since_move: u32,
}

/// System: Execute movement using MovementComponent
/// This system processes entities with MovementComponent and moves them along their path
/// Respects MovementSpeed.ticks_per_move for proper speed control
pub fn execute_movement_component(
    mut query: Query<(
        Entity,
        &mut TilePosition,
        &mut super::MovementComponent,
        &MovementSpeed,
        Option<&mut MovementTick>,
    )>,
    mut commands: Commands,
) {
    for (entity, mut position, mut movement, speed, movement_tick) in query.iter_mut() {
        if let super::MovementComponent::FollowingPath { path, index } = &*movement {
            // Get or initialize movement tick tracker
            let _ticks_since_move = if let Some(mut tick) = movement_tick {
                tick.ticks_since_move += 1;
                if tick.ticks_since_move < speed.ticks_per_move {
                    continue; // Not time to move yet
                }
                tick.ticks_since_move = 0;
                0
            } else {
                // First tick - add MovementTick component and move
                commands.entity(entity).insert(MovementTick::default());
                0
            };

            // Check if we have more waypoints
            if *index < path.len() {
                // Move to next waypoint
                let next_pos = path[*index];
                position.tile = next_pos;

                // Advance index
                let new_index = *index + 1;

                if new_index >= path.len() {
                    // Path complete, transition to Idle
                    *movement = super::MovementComponent::Idle;
                    commands.entity(entity).remove::<MovementTick>();
                    debug!(
                        "Entity {:?} completed path, now at {:?}",
                        entity, next_pos
                    );
                } else {
                    // Update index to continue following path
                    // Arc::clone is cheap - only increments atomic reference count, doesn't copy Vec
                    *movement = super::MovementComponent::FollowingPath {
                        path: std::sync::Arc::clone(path),
                        index: new_index,
                    };
                }
            } else {
                // Path is empty or index out of bounds, transition to Idle
                *movement = super::MovementComponent::Idle;
                commands.entity(entity).remove::<MovementTick>();
                debug!("Entity {:?} path empty, transitioning to Idle", entity);
            }
        }
    }
}
