use bevy::prelude::*;

use crate::entities::stats::Thirst;
use crate::entities::TilePosition;
use crate::simulation::tick::SimulationTick;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;

use super::{Action, ActionResult};

// =============================================================================
// DRINK WATER ACTION
// =============================================================================

/// Action: Drink water from a shallow water tile
///
/// Behavior:
/// 1. If not at water tile, path to it (multi-tick)
/// 2. Once at water, drink (instant)
/// 3. Reduces thirst significantly
///
/// Phase 3: Uses PathfindingQueue for async pathfinding
#[derive(Debug, Clone)]
pub struct DrinkWaterAction {
    pub target_tile: IVec2,
    state: DrinkWaterState,
    retry_count: u32,
    max_retries: u32,
    pub move_target: Option<IVec2>,
    total_ticks: u32,
    max_total_ticks: u32,
}

/// State machine for async drinking with PathfindingQueue
#[derive(Debug, Clone)]
enum DrinkWaterState {
    /// Need to request path to target
    NeedPath,
    /// Waiting for pathfinding result
    WaitingForPath {
        request_id: crate::pathfinding::PathRequestId,
    },
    /// Moving to target (MovementComponent handles actual movement)
    Moving,
    /// At water, drinking
    Drinking,
}

impl DrinkWaterAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            state: DrinkWaterState::NeedPath,
            retry_count: 0,
            max_retries: 3,
            move_target: None,
            total_ticks: 0,
            max_total_ticks: 100,
        }
    }

    /// Transition from NeedPath to WaitingForPath state
    /// Called by pathfinding bridge system after queuing pathfinding request
    pub fn transition_to_waiting(&mut self, request_id: crate::pathfinding::PathRequestId) {
        self.state = DrinkWaterState::WaitingForPath { request_id };
    }
}

impl Action for DrinkWaterAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        // Check entity has thirst component
        if world.get::<Thirst>(entity).is_none() {
            warn!("DrinkWater can_execute: entity {:?} has no Thirst", entity);
            return false;
        }

        // Check target tile is adjacent to water (target_tile is now a walkable tile next to water)
        if let Some(world_loader) = world.get_resource::<WorldLoader>() {
            // Check all adjacent tiles for water (including the target tile itself)
            let offsets = [
                IVec2::new(0, 0),  // Check target tile itself too
                IVec2::new(0, 1),
                IVec2::new(1, 0),
                IVec2::new(0, -1),
                IVec2::new(-1, 0),
                IVec2::new(1, 1),
                IVec2::new(1, -1),
                IVec2::new(-1, 1),
                IVec2::new(-1, -1),
            ];

            let mut found_terrains = Vec::new();
            for offset in offsets {
                let check_pos = self.target_tile + offset;
                if let Some(terrain_str) =
                    world_loader.get_terrain_at(check_pos.x, check_pos.y)
                {
                    found_terrains.push(format!("({},{}): {}", check_pos.x, check_pos.y, terrain_str));
                    if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                        if matches!(terrain, TerrainType::ShallowWater | TerrainType::Water) {
                            return true;
                        }
                    }
                }
            }
            warn!(
                "DrinkWater can_execute failed: target {:?}, terrains: {:?}",
                self.target_tile, found_terrains
            );
            false
        } else {
            warn!("DrinkWater can_execute: no WorldLoader resource");
            false
        }
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // Track total elapsed ticks to prevent infinite retry loops
        self.total_ticks = self.total_ticks.saturating_add(1);
        if self.total_ticks > self.max_total_ticks {
            debug!(
                "DrinkWater gave up for entity {:?} after {} total ticks",
                entity, self.total_ticks
            );
            return ActionResult::Failed;
        }

        // Get current tick from SimulationTick resource
        let tick = world.get_resource::<SimulationTick>()
            .map(|t| t.0)
            .unwrap_or(0);

        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            warn!("Entity {:?} has no position, cannot drink", entity);
            return ActionResult::Failed;
        };

        let current_pos = position.tile;

        // Compute move target once if not set
        if self.move_target.is_none() {
            if let Some(world_loader) = world.get_resource::<WorldLoader>() {
                self.move_target = find_adjacent_walkable_tile(self.target_tile, world_loader)
                    .or_else(|| Some(self.target_tile));
            } else {
                return ActionResult::Failed;
            }
        }

        let move_target = self.move_target.unwrap_or(self.target_tile);

        // Check if we're at the target tile (which is adjacent to water)
        // or close enough to drink (within 1 tile of target)
        let distance = (current_pos - self.target_tile).abs();
        let is_at_target = current_pos == self.target_tile;
        let is_close_enough = distance.x <= 1 && distance.y <= 1;

        if is_at_target || is_close_enough {
            // Transition to drinking state - we're at or near the water-adjacent tile
            self.state = DrinkWaterState::Drinking;
        }

        // State machine for async pathfinding
        match &self.state {
            DrinkWaterState::NeedPath => {
                // Signal system layer to queue pathfinding
                ActionResult::NeedsPathfinding { target: move_target }
            }

            DrinkWaterState::WaitingForPath { request_id: _ } => {
                // Check for PathReady component (Phase 2: Component-based pathfinding)
                let entity_ref = world.get_entity(entity).ok();

                // Check if path is ready
                if let Some(entity_ref) = entity_ref {
                    if entity_ref.contains::<crate::pathfinding::PathReady>() {
                        // Path ready! System layer will insert MovementComponent
                        self.state = DrinkWaterState::Moving;
                        return ActionResult::InProgress;
                    }

                    // Check if path failed
                    if entity_ref.contains::<crate::pathfinding::PathFailed>() {
                        // Pathfinding failed, retry if under max retries
                        if self.retry_count < self.max_retries {
                            self.retry_count += 1;
                            self.state = DrinkWaterState::NeedPath;
                            debug!(
                                "DrinkWater path failed for entity {:?}, retry {}/{}",
                                entity, self.retry_count, self.max_retries
                            );
                            return ActionResult::InProgress;
                        } else {
                            debug!(
                                "DrinkWater gave up for entity {:?} after {} retries",
                                entity, self.max_retries
                            );
                            return ActionResult::Failed;
                        }
                    }
                }

                // Still waiting for path (no PathReady or PathFailed component yet)
                ActionResult::InProgress
            }

            DrinkWaterState::Moving => {
                // Check if movement is complete via MovementComponent
                if let Ok(entity_ref) = world.get_entity(entity) {
                    if let Some(movement) = entity_ref.get::<crate::entities::MovementComponent>() {
                        if movement.is_idle() {
                            // Movement complete, transition to drinking
                            self.state = DrinkWaterState::Drinking;
                        }
                    }
                }

                // Continue moving (execute_movement_component system handles actual movement)
                ActionResult::InProgress
            }

            DrinkWaterState::Drinking => {
                // We're close enough to drink!
                // NOTE: Cannot mutate Thirst with read-only World
                // System layer will handle the actual drinking mutation
                // Return Success to signal action complete
                let entity_ref = world.get_entity(entity).ok();
                if let Some(entity_ref) = entity_ref {
                    let amount = entity_ref
                        .get::<crate::entities::types::SpeciesNeeds>()
                        .map(|needs| needs.drink_amount)
                        .unwrap_or(50.0);

                    if entity_ref.contains::<Thirst>() {
                        info!(
                            "💧 Entity {:?} drinking water from {:?} at {:?} on tick {} (amount: {:.1})",
                            entity,
                            self.target_tile,
                            current_pos,
                            tick,
                            amount
                        );

                        // Return Success - system layer will apply thirst reduction
                        return ActionResult::Success;
                    }
                }

                ActionResult::Failed
            }
        }
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.state = DrinkWaterState::NeedPath;
        self.retry_count = 0;
        self.move_target = None;
        self.total_ticks = 0;
        debug!(
            "🚫 DrinkWater action cancelled for entity {:?}, clearing navigation state",
            entity
        );
    }

    fn name(&self) -> &'static str {
        "DrinkWater"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Find a walkable tile adjacent to a water tile
fn find_adjacent_walkable_tile(water_pos: IVec2, world_loader: &WorldLoader) -> Option<IVec2> {
    // Check all 8 adjacent tiles (including diagonals)
    let adjacent_offsets = [
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, 1),
        IVec2::new(-1, -1),
    ];

    for offset in adjacent_offsets {
        let check_pos = water_pos + offset;

        if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
            if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                if terrain.is_walkable() {
                    return Some(check_pos);
                }
            }
        }
    }

    None
}

/// Clear navigation state for an entity (deprecated - handled by system layer)
///
/// Actions should not mutate directly - mutations handled by execute_active_actions system.
#[deprecated(note = "Use Commands in system layer instead")]
fn clear_navigation_state(world: &World, entity: Entity) {
    // This function is now a no-op since actions can't mutate World.
    // Navigation state clearing will be handled by the system layer via Commands.
    // Keeping function signature for compatibility during refactor.
    let _ = (world, entity); // Suppress unused warnings
}
