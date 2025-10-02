/// Action system for TQUAI
/// 
/// Actions are discrete behaviors that can be queued and executed on ticks.
/// They can be instant (complete in one tick) or multi-tick (span multiple ticks).

use bevy::prelude::*;
use crate::entities::stats::{Thirst, Hunger, Energy};
use crate::entities::{TilePosition, MoveOrder};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use crate::pathfinding::PathfindingFailed;

/// Result of executing an action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionResult {
    /// Action completed successfully
    Success,
    /// Action failed (preconditions not met, resource unavailable, etc.)
    Failed,
    /// Action is still in progress (will continue next tick)
    InProgress,
}

/// Request to queue an action
#[derive(Debug, Clone)]
pub struct ActionRequest {
    pub entity: Entity,
    pub action_type: ActionType,
    pub utility: f32,
    pub priority: i32,
}

/// Types of actions available
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    DrinkWater { target_tile: IVec2 },
    Graze { target_tile: IVec2 },   // Move to grass tile (eating happens via auto-eat system)
    Rest { duration_ticks: u32 },
    // Future actions:
    // Hunt { target: Entity },
    // Flee { from: Entity },
}

/// Core Action trait
/// All actions must implement this to be executable in the TQUAI system
pub trait Action: Send + Sync {
    /// Check if action can be executed (preconditions)
    fn can_execute(&self, world: &World, entity: Entity, tick: u64) -> bool;
    
    /// Execute the action for this tick
    /// Returns Success/Failed/InProgress
    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult;
    
    /// Get action name for debugging
    fn name(&self) -> &'static str;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Find a walkable tile adjacent to a water tile
fn find_adjacent_walkable_tile(
    water_pos: IVec2,
    world_loader: &WorldLoader,
) -> Option<IVec2> {
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
                // Must be walkable but NOT water
                if terrain.is_walkable() && !matches!(terrain, TerrainType::ShallowWater | TerrainType::DeepWater | TerrainType::Water) {
                    // CRITICAL: Also check that tile doesn't have resources blocking it
                    let has_blocking_resource = world_loader.get_resource_at(check_pos.x, check_pos.y)
                        .map(|r| !r.is_empty())
                        .unwrap_or(false);
                    
                    if !has_blocking_resource {
                        return Some(check_pos);
                    }
                }
            }
        }
    }
    
    None
}

// =============================================================================
// DRINK WATER ACTION
// =============================================================================

/// Action: Drink water from a shallow water tile
/// 
/// Behavior:
/// 1. If not at water tile, path to it (multi-tick)
/// 2. Once at water, drink (instant)
/// 3. Reduces thirst significantly
#[derive(Debug, Clone)]
pub struct DrinkWaterAction {
    pub target_tile: IVec2,
    pub started: bool,
}

impl DrinkWaterAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            started: false,
        }
    }
}

impl Action for DrinkWaterAction {
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
        // Check entity has thirst component
        if world.get::<Thirst>(entity).is_none() {
            return false;
        }
        
        // Check target tile is actually water
        if let Some(world_loader) = world.get_resource::<WorldLoader>() {
            if let Some(terrain_str) = world_loader.get_terrain_at(self.target_tile.x, self.target_tile.y) {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    matches!(terrain, TerrainType::ShallowWater)
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
    
    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            warn!("Entity {:?} has no position, cannot drink", entity);
            return ActionResult::Failed;
        };
        
        let current_pos = position.tile;
        
        // Check if pathfinding failed for this entity
        if world.get::<PathfindingFailed>(entity).is_some() {
            warn!(
                "Entity {:?} pathfinding failed to reach water at {:?}, aborting DrinkWater action",
                entity,
                self.target_tile
            );
            // Remove the PathfindingFailed component
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.remove::<PathfindingFailed>();
            }
            return ActionResult::Failed;
        }
        
        // Check if we're adjacent to the water tile (or standing in it)
        let distance = (current_pos - self.target_tile).abs();
        let is_adjacent = distance.x <= 1 && distance.y <= 1 && (distance.x + distance.y) > 0;
        let is_on_water = current_pos == self.target_tile;
        
        if is_adjacent || is_on_water {
            // We're close enough to drink!
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                if let Some(mut thirst) = entity_mut.get_mut::<Thirst>() {
                    // Reduce thirst
                    thirst.0.change(-30.0);
                    
                    info!(
                        "🐇 Entity {:?} drank water from {:?} while at {:?} on tick {}! Thirst: {:.1}%",
                        entity,
                        self.target_tile,
                        current_pos,
                        tick,
                        thirst.0.percentage()
                    );
                    
                    return ActionResult::Success;
                }
            }
            
            return ActionResult::Failed;
        }
        
        // We need to move closer to the water
        if !self.started {
            // Issue move order on first execution
            // Find a walkable tile adjacent to the water
            if let Some(world_loader) = world.get_resource::<WorldLoader>() {
                if let Some(adjacent_pos) = find_adjacent_walkable_tile(self.target_tile, world_loader) {
                    info!(
                        "🐇 Entity {:?} starting journey to water at {:?} (will stop at adjacent tile {:?})",
                        entity,
                        self.target_tile,
                        adjacent_pos
                    );
                    
                    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                        entity_mut.insert(MoveOrder {
                            destination: adjacent_pos,
                            allow_diagonal: true,  // Enable diagonal pathfinding
                        });
                    }
                    
                    self.started = true;
                } else {
                    warn!("No adjacent walkable tile found for water at {:?}", self.target_tile);
                    return ActionResult::Failed;
                }
            } else {
                return ActionResult::Failed;
            }
        }
        
        // Still traveling
        ActionResult::InProgress
    }
    
    fn name(&self) -> &'static str {
        "DrinkWater"
    }
}

// =============================================================================
// GRAZE ACTION
// =============================================================================

/// Action: Move to a grass tile (for grazing/eating)
/// 
/// Behavior:
/// - Moves to target grass tile
/// - Once there, auto-eat system will trigger eating
/// - Used when hungry
#[derive(Debug, Clone)]
pub struct GrazeAction {
    pub target_tile: IVec2,
    pub started: bool,
}

impl GrazeAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            started: false,
        }
    }
}

impl Action for GrazeAction {
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
        // Check entity has position
        if world.get::<TilePosition>(entity).is_none() {
            return false;
        }
        
        // Check target tile is walkable
        if let Some(world_loader) = world.get_resource::<WorldLoader>() {
            if let Some(terrain_str) = world_loader.get_terrain_at(self.target_tile.x, self.target_tile.y) {
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
    
    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            warn!("Entity {:?} has no position, cannot wander", entity);
            return ActionResult::Failed;
        };
        
        let current_pos = position.tile;
        
        // Check if pathfinding failed for this entity
        if world.get::<PathfindingFailed>(entity).is_some() {
            debug!(
                "Entity {:?} pathfinding failed to reach wander target {:?}, aborting Wander action",
                entity,
                self.target_tile
            );
            // Remove the PathfindingFailed component
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.remove::<PathfindingFailed>();
            }
            return ActionResult::Failed;
        }
        
        // Check if we've arrived at target
        if current_pos == self.target_tile {
            debug!(
                "🐇 Entity {:?} arrived at grass {:?} on tick {}",
                entity,
                self.target_tile,
                tick
            );
            return ActionResult::Success;
        }
        
        // Start moving if not started yet
        if !self.started {
            debug!(
                "🐇 Entity {:?} moving to grass at {:?}",
                entity,
                self.target_tile
            );
            
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.insert(MoveOrder {
                    destination: self.target_tile,
                    allow_diagonal: true,  // Enable diagonal pathfinding
                });
            }
            
            self.started = true;
        }
        
        // Still traveling
        ActionResult::InProgress
    }
    
    fn name(&self) -> &'static str {
        "Graze"
    }
}

// =============================================================================
// REST ACTION
// =============================================================================

/// Action: Rest in place to regenerate energy
#[derive(Debug, Clone)]
pub struct RestAction {
    pub duration_ticks: u32,
    pub ticks_remaining: u32,
    pub started: bool,
}

impl RestAction {
    pub fn new(duration_ticks: u32) -> Self {
        Self {
            duration_ticks,
            ticks_remaining: duration_ticks,
            started: false,
        }
    }
}

impl Action for RestAction {
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
        world.get::<Energy>(entity).is_some()
    }
    
    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
        if !self.started {
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                if let Some(mut energy) = entity_mut.get_mut::<Energy>() {
                    energy.set_resting();
                    info!("😴 Entity {:?} started resting for {} ticks (energy: {:.1}%)", entity, self.duration_ticks, energy.0.percentage());
                }
            }
            self.started = true;
        }
        
        self.ticks_remaining = self.ticks_remaining.saturating_sub(1);
        
        if self.ticks_remaining == 0 {
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                if let Some(mut energy) = entity_mut.get_mut::<Energy>() {
                    energy.set_active();
                    info!("😊 Entity {:?} finished resting on tick {}! Energy: {:.1}%", entity, tick, energy.0.percentage());
                }
            }
            return ActionResult::Success;
        }
        
        ActionResult::InProgress
    }
    
    fn name(&self) -> &'static str {
        "Rest"
    }
}

// =============================================================================
// ACTION FACTORY
// =============================================================================

/// Create an action from an ActionType
pub fn create_action(action_type: ActionType) -> Box<dyn Action> {
    match action_type {
        ActionType::DrinkWater { target_tile } => {
            Box::new(DrinkWaterAction::new(target_tile))
        }
        ActionType::Graze { target_tile } => {
            Box::new(GrazeAction::new(target_tile))
        }
        ActionType::Rest { duration_ticks } => {
            Box::new(RestAction::new(duration_ticks))
        }
    }
}
