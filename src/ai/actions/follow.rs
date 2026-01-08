use super::*;
use crate::entities::stats::{Hunger, Thirst};
use crate::entities::TilePosition;
use crate::pathfinding::{Path, PathfindingFailed};
use bevy::prelude::*;

// =============================================================================
// FOLLOW ACTION
// =============================================================================

/// Action: Follow a target entity until within a certain distance
#[derive(Debug, Clone)]
pub struct FollowAction {
    pub target: Entity,
    pub stop_distance: i32,
    pub started: bool,
}

impl FollowAction {
    pub fn new(target: Entity, stop_distance: i32) -> Self {
        Self {
            target,
            stop_distance,
            started: false,
        }
    }
}

impl Action for FollowAction {
    fn can_execute(&self, world: &World, _entity: Entity) -> bool {
        // Target must still exist and have a position
        world.get_entity(self.target).is_ok() && world.get::<TilePosition>(self.target).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // CRITICAL NEEDS CHECK - survival override
        // If hunger or thirst are critical (>=80%), abort follow for survival
        if let Ok(entity_ref) = world.get_entity(entity) {
            if let (Some(hunger), Some(thirst)) = (
                entity_ref.get::<Hunger>(),
                entity_ref.get::<Thirst>()
            ) {
                if hunger.0.normalized() >= 0.80 || thirst.0.normalized() >= 0.80 {
                    info!(
                        "🚨 Follow aborted for {:?} - critical survival needs (H:{:.1}% T:{:.1}%)",
                        entity,
                        hunger.0.percentage(),
                        thirst.0.percentage()
                    );
                    return ActionResult::Failed; // Trigger replanning for survival
                }
            }
        }

        // Abort on pathfinding failure for this entity
        if world.get::<PathfindingFailed>(entity).is_some() {
            warn!(
                "Entity {:?} pathfinding failed while following, aborting Follow action",
                entity
            );
            // NOTE: PathfindingFailed removal handled by system layer
            return ActionResult::Failed;
        }

        let Some(follower_pos) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };
        let Some(target_pos) = world.get::<TilePosition>(self.target).copied() else {
            return ActionResult::Failed;
        };

        // Check distance
        let d = {
            let diff = (follower_pos.tile - target_pos.tile).abs();
            diff.x.max(diff.y)
        };

        if d <= self.stop_distance {
            return ActionResult::Success;
        }

        // If not currently moving (no Path), system layer will issue move order
        let is_moving = world.get::<Path>(entity).is_some();
        if !is_moving {
            self.started = true;
            // NOTE: MoveOrder insertion handled by system layer
        }

        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.started = false;
        debug!(
            "🚫 Follow action cancelled for entity {:?}, stopping movement",
            entity
        );
    }

    fn name(&self) -> &'static str {
        "Follow"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Remove movement-related components so a cancelled action stops any in-flight navigation
/// NOTE: This function is deprecated in favor of using Commands in the system layer.
/// Actions should not mutate directly - mutations handled by execute_active_actions system.
#[deprecated(note = "Use Commands in system layer instead")]
fn clear_navigation_state(world: &World, entity: Entity) {
    // This function is now a no-op since actions can't mutate World.
    // Navigation state clearing will be handled by the system layer via Commands.
    // Keeping function signature for compatibility during refactor.
    let _ = (world, entity); // Suppress unused warnings
}
