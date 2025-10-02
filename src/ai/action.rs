/// Action system for TQUAI
/// 
/// Actions are discrete behaviors that can be queued and executed on ticks.
/// They can be instant (complete in one tick) or multi-tick (span multiple ticks).

use bevy::prelude::*;
use crate::entities::stats::Thirst;
use crate::entities::{TilePosition, MoveOrder};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;

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
    Wander { target_tile: IVec2 },
    // Future actions:
    // EatFood { target: Entity },
    // Flee { from: Entity },
    // Rest { duration_ticks: u32 },
}

/// Core Action trait
/// All actions must implement this to be executable in the TQUAI system
pub trait Action: Send + Sync {
    /// Check if action can be executed (preconditions)
    fn can_execute(&self, world: &World, entity: Entity, tick: u64) -> bool;
    
    /// Execute the action for this tick
    /// Returns Success/Failed/InProgress
    fn execute(&mut self, commands: &mut Commands, world: &World, entity: Entity, tick: u64) -> ActionResult;
    
    /// Get action name for debugging
    fn name(&self) -> &'static str;
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
    
    fn execute(&mut self, commands: &mut Commands, world: &World, entity: Entity, tick: u64) -> ActionResult {
        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity) else {
            warn!("Entity {:?} has no position, cannot drink", entity);
            return ActionResult::Failed;
        };
        
        let current_pos = position.tile;
        
        // Check if we're already at the water tile
        if current_pos == self.target_tile {
            // We're at the water! Drink!
            if let Some(mut thirst) = world.get::<Thirst>(entity).cloned() {
                // Reduce thirst
                thirst.0.change(-30.0);
                
                // Update the component
                commands.entity(entity).insert(thirst.clone());
                
                info!(
                    "🐇 Entity {:?} drank water at {:?} on tick {}! Thirst: {:.1}%",
                    entity,
                    self.target_tile,
                    tick,
                    thirst.0.percentage()
                );
                
                return ActionResult::Success;
            }
            
            return ActionResult::Failed;
        }
        
        // We need to move to the water
        if !self.started {
            // Issue move order on first execution
            info!(
                "🐇 Entity {:?} starting journey to water at {:?}",
                entity,
                self.target_tile
            );
            
            commands.entity(entity).insert(MoveOrder {
                destination: self.target_tile,
                allow_diagonal: false,
            });
            
            self.started = true;
        }
        
        // Check if we've arrived
        if current_pos == self.target_tile {
            // Arrived! Will drink next tick
            ActionResult::InProgress
        } else {
            // Still traveling
            ActionResult::InProgress
        }
    }
    
    fn name(&self) -> &'static str {
        "DrinkWater"
    }
}

// =============================================================================
// WANDER ACTION
// =============================================================================

/// Action: Wander to a random nearby tile
/// 
/// Behavior:
/// - Picks a random walkable tile nearby
/// - Paths to it
/// - Low priority idle behavior
#[derive(Debug, Clone)]
pub struct WanderAction {
    pub target_tile: IVec2,
    pub started: bool,
}

impl WanderAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            started: false,
        }
    }
}

impl Action for WanderAction {
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
    
    fn execute(&mut self, commands: &mut Commands, world: &World, entity: Entity, tick: u64) -> ActionResult {
        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity) else {
            warn!("Entity {:?} has no position, cannot wander", entity);
            return ActionResult::Failed;
        };
        
        let current_pos = position.tile;
        
        // Check if we've arrived at target
        if current_pos == self.target_tile {
            debug!(
                "🐇 Entity {:?} completed wander to {:?} on tick {}",
                entity,
                self.target_tile,
                tick
            );
            return ActionResult::Success;
        }
        
        // Start moving if not started yet
        if !self.started {
            debug!(
                "🐇 Entity {:?} starting wander to {:?}",
                entity,
                self.target_tile
            );
            
            commands.entity(entity).insert(MoveOrder {
                destination: self.target_tile,
                allow_diagonal: false,
            });
            
            self.started = true;
        }
        
        // Still traveling
        ActionResult::InProgress
    }
    
    fn name(&self) -> &'static str {
        "Wander"
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
        ActionType::Wander { target_tile } => {
            Box::new(WanderAction::new(target_tile))
        }
    }
}
