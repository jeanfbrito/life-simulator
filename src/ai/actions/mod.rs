use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use crate::types::newtypes::Utility;
use bevy::prelude::*;

// =============================================================================
// SUBMODULES
// =============================================================================

pub mod drink;
pub mod graze;
pub mod rest;
pub mod scavenge;
pub mod hunt;
pub mod follow;
pub mod mate;
pub mod harvest;
pub mod wander;

// Re-export action structs
pub use drink::DrinkWaterAction;
pub use graze::GrazeAction;
pub use rest::RestAction;
pub use scavenge::ScavengeAction;
pub use hunt::HuntAction;
pub use follow::FollowAction;
pub use mate::MateAction;
pub use harvest::HarvestAction;
pub use wander::WanderAction;

// =============================================================================
// CONSTANTS
// =============================================================================

const DEFAULT_CARCASS_DECAY: u32 = 6_000;
const MIN_CARCASS_NUTRITION: f32 = 5.0;

// =============================================================================
// COMMON TYPES
// =============================================================================

/// Result of executing an action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionResult {
    /// Action completed successfully
    Success,
    /// Action failed (preconditions not met, resource unavailable, etc.)
    Failed,
    /// Action is still in progress (will continue next tick)
    InProgress,
    /// Action completed but should trigger a follow-up action
    /// Used for giving-up behavior when patch quality is too low
    TriggerFollowUp,
    /// Action needs pathfinding to target position
    /// System layer will queue pathfinding request and transition action to WaitingForPath
    NeedsPathfinding { target: IVec2 },
}

/// Request to queue an action
#[derive(Debug, Clone)]
pub struct ActionRequest {
    pub entity: Entity,
    pub action_type: ActionType,
    /// How desirable this action is (0.0-1.0) - now typed for clarity
    pub utility: Utility,
    pub priority: i32,
}

/// Types of actions available
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    DrinkWater {
        target_tile: IVec2,
    },
    Graze {
        target_tile: IVec2,
    }, // Move to grass tile (eating happens via auto-eat system)
    Rest {
        duration_ticks: u32,
    },
    Scavenge {
        carcass: Entity,
    },
    Hunt {
        prey: Entity,
    },
    Follow {
        target: Entity,
        stop_distance: i32,
    },
    Mate {
        partner: Entity,
        meeting_tile: IVec2,
        duration_ticks: u32,
    },
    Harvest {
        target_tile: IVec2,
        resource_type: crate::resources::ResourceType,
    },
    Wander {
        target_tile: IVec2,
    },
    // Future actions:
    // Flee { from: Entity },
}

/// Core Action trait
/// All actions must implement this to be executable in the TQUAI system
pub trait Action: Send + Sync {
    /// Check if action can be executed (preconditions)
    fn can_execute(&self, world: &World, entity: Entity) -> bool;

    /// Execute the action for this tick
    /// Returns Success/Failed/InProgress
    /// Uses read-only World access - mutations handled by systems via Commands
    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult;

    /// Cancel the action (called when a higher priority action needs to interrupt)
    /// Default implementation does nothing - override for actions that need cleanup
    fn cancel(&mut self, world: &World, entity: Entity) {
        // Default: no cleanup needed
    }

    /// Get action name for debugging
    fn name(&self) -> &'static str;

    /// Downcast to concrete type for state transitions
    /// Required for pathfinding bridge system to transition action states
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Find a walkable tile adjacent to a water tile
pub(crate) fn find_adjacent_walkable_tile(water_pos: IVec2, world_loader: &WorldLoader) -> Option<IVec2> {
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
                if terrain.is_walkable()
                    && !matches!(
                        terrain,
                        TerrainType::ShallowWater | TerrainType::DeepWater | TerrainType::Water
                    )
                {
                    // CRITICAL: Only Trees and Rocks block movement (not bushes, flowers, shrubs)
                    let has_blocking_resource = world_loader
                        .get_resource_at(check_pos.x, check_pos.y)
                        .map(|r| crate::resources::is_blocking_resource(&r))
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

/// Remove movement-related components so a cancelled action stops any in-flight navigation
/// NOTE: This function is deprecated in favor of using Commands in the system layer.
/// Actions should not mutate directly - mutations handled by execute_active_actions system.
#[deprecated(note = "Use Commands in system layer instead")]
pub(crate) fn clear_navigation_state(world: &World, entity: Entity) {
    // This function is now a no-op since actions can't mutate World.
    // Navigation state clearing will be handled by the system layer via Commands.
    // Keeping function signature for compatibility during refactor.
    let _ = (world, entity); // Suppress unused warnings
}

// =============================================================================
// ACTION FACTORY
// =============================================================================

/// Create an action from an ActionType
pub fn create_action(action_type: ActionType) -> Box<dyn Action> {
    match action_type {
        ActionType::DrinkWater { target_tile } => Box::new(DrinkWaterAction::new(target_tile)),
        ActionType::Graze { target_tile } => Box::new(GrazeAction::new(target_tile)),
        ActionType::Rest { duration_ticks } => Box::new(RestAction::new(duration_ticks)),
        ActionType::Scavenge { carcass } => Box::new(ScavengeAction::new(carcass)),
        ActionType::Hunt { prey } => Box::new(HuntAction::new(prey)),
        ActionType::Follow {
            target,
            stop_distance,
        } => Box::new(FollowAction::new(target, stop_distance)),
        ActionType::Mate {
            partner,
            meeting_tile,
            duration_ticks,
        } => Box::new(MateAction::new(partner, meeting_tile, duration_ticks)),
        ActionType::Harvest {
            target_tile,
            resource_type,
        } => Box::new(HarvestAction::new(target_tile, resource_type)),
        ActionType::Wander { target_tile } => Box::new(WanderAction::new(target_tile)),
    }
}
