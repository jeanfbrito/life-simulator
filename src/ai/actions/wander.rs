use crate::entities::TilePosition;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use bevy::prelude::*;

use super::super::{Action, ActionResult};

// =============================================================================
// WANDER ACTION
// =============================================================================

/// Wander action - idle exploration within territory
///
/// Behavior:
/// - Moves to a random walkable tile within wander_radius
/// - Lowest priority action (always available as fallback)
/// - Used when no needs are pressing
///
/// Phase 2: Uses PathfindingQueue for async pathfinding
#[derive(Debug, Clone)]
pub struct WanderAction {
    pub target_tile: IVec2,
    state: WanderState,
    retry_count: u32,
    max_retries: u32,
}

/// State machine for async wandering with PathfindingQueue
#[derive(Debug, Clone)]
enum WanderState {
    /// Need to request path to target
    NeedPath,
    /// Waiting for pathfinding result
    WaitingForPath {
        request_id: crate::pathfinding::PathRequestId,
    },
    /// Moving to target (MovementComponent handles actual movement)
    Moving,
}

impl WanderAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            state: WanderState::NeedPath,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Transition from NeedPath to WaitingForPath state
    /// Called by pathfinding bridge system after queuing pathfinding request
    pub fn transition_to_waiting(&mut self, request_id: crate::pathfinding::PathRequestId) {
        self.state = WanderState::WaitingForPath { request_id };
    }
}

impl Action for WanderAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        // Check entity has position
        if world.get::<TilePosition>(entity).is_none() {
            return false;
        }

        // Check target tile is walkable
        if let Some(world_loader) = world.get_resource::<WorldLoader>() {
            if let Some(terrain_str) =
                world_loader.get_terrain_at(self.target_tile.x, self.target_tile.y)
            {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    terrain.is_walkable()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // Get current position
        let Some(pos) = world.get::<TilePosition>(entity) else {
            return ActionResult::Failed;
        };
        let current_pos = pos.tile;

        // Check if arrived at target
        if current_pos == self.target_tile {
            return ActionResult::Success;
        }

        // State machine for async pathfinding
        match &self.state {
            WanderState::NeedPath => {
                // Signal system layer to queue pathfinding
                ActionResult::NeedsPathfinding { target: self.target_tile }
            }

            WanderState::WaitingForPath { request_id: _ } => {
                // Check for PathReady component (Phase 2: Component-based pathfinding)
                let entity_ref = world.get_entity(entity).ok();

                // Check if path is ready
                if let Some(entity_ref) = entity_ref {
                    if entity_ref.contains::<crate::pathfinding::PathReady>() {
                        // Path ready! System layer will insert MovementComponent
                        self.state = WanderState::Moving;
                        return ActionResult::InProgress;
                    }

                    // Check if path failed
                    if entity_ref.contains::<crate::pathfinding::PathFailed>() {
                        // Pathfinding failed, retry with new target if under max retries
                        if self.retry_count < self.max_retries {
                            self.retry_count += 1;
                            self.state = WanderState::NeedPath;
                            debug!(
                                "Wander path failed for entity {:?}, retry {}/{}",
                                entity, self.retry_count, self.max_retries
                            );
                            return ActionResult::InProgress;
                        } else {
                            debug!(
                                "Wander gave up for entity {:?} after {} retries",
                                entity, self.max_retries
                            );
                            return ActionResult::Failed;
                        }
                    }
                }

                // Still waiting for path (no PathReady or PathFailed component yet)
                ActionResult::InProgress
            }

            WanderState::Moving => {
                // Check if movement is complete via MovementComponent
                if let Ok(entity_ref) = world.get_entity(entity) {
                    if let Some(movement) = entity_ref.get::<crate::entities::MovementComponent>() {
                        if movement.is_idle() {
                            // Movement complete!
                            return ActionResult::Success;
                        }
                    }
                }

                // Continue moving (execute_movement_component system handles actual movement)
                ActionResult::InProgress
            }
        }
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        // NOTE: MovementComponent insertion handled by system layer
        let _ = (world, entity); // Suppress unused warnings
        // Reset state machine
        self.state = WanderState::NeedPath;
        self.retry_count = 0;
    }

    fn name(&self) -> &'static str {
        "Wander"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
