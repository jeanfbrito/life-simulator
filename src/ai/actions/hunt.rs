use bevy::prelude::*;

use crate::entities::stats::{Hunger, Thirst};
use crate::entities::{SpeciesNeeds, TilePosition};

use super::super::{Action, ActionResult};

/// Action: Pursue prey and attempt a kill.
///
/// Phase 3: Uses PathfindingQueue for async pathfinding
#[derive(Debug, Clone)]
pub struct HuntAction {
    pub prey: Entity,
    state: HuntState,
    retry_count: u32,
    max_retries: u32,
    last_prey_pos: Option<IVec2>,
}

/// State machine for async hunting with PathfindingQueue
#[derive(Debug, Clone)]
enum HuntState {
    /// Need to request path to prey
    NeedPath,
    /// Waiting for pathfinding result
    WaitingForPath {
        request_id: crate::pathfinding::PathRequestId,
        target_pos: IVec2,
    },
    /// Moving to target (MovementComponent handles actual movement)
    Moving {
        target_pos: IVec2,
    },
    /// Close enough to attack
    Attacking,
}

impl HuntAction {
    pub fn new(prey: Entity) -> Self {
        Self {
            prey,
            state: HuntState::NeedPath,
            retry_count: 0,
            max_retries: 3,
            last_prey_pos: None,
        }
    }

    /// Transition from NeedPath to WaitingForPath state
    /// Called by pathfinding bridge system after queuing pathfinding request
    pub fn transition_to_waiting(&mut self, request_id: crate::pathfinding::PathRequestId) {
        // Hunt needs to track target position in WaitingForPath state
        // Use last known prey position
        let target_pos = self.last_prey_pos.unwrap_or(IVec2::ZERO);
        self.state = HuntState::WaitingForPath { request_id, target_pos };
    }
}

impl Action for HuntAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        world.get::<Hunger>(entity).is_some() && world.get::<TilePosition>(self.prey).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // CRITICAL NEEDS CHECK - survival override
        // If hunger or thirst are critical (>=80%), abort hunt for survival
        if let Ok(entity_ref) = world.get_entity(entity) {
            if let (Some(hunger), Some(thirst)) = (
                entity_ref.get::<Hunger>(),
                entity_ref.get::<Thirst>()
            ) {
                if hunger.0.normalized() >= 0.80 || thirst.0.normalized() >= 0.80 {
                    info!(
                        "🚨 Hunt aborted for {:?} - critical survival needs (H:{:.1}% T:{:.1}%)",
                        entity,
                        hunger.0.percentage(),
                        thirst.0.percentage()
                    );
                    return ActionResult::Failed; // Trigger replanning for survival
                }
            }
        }

        let Some(predator_pos) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };

        let Some(prey_pos) = world.get::<TilePosition>(self.prey).copied() else {
            debug!("🎯 Prey {:?} lost before hunt completed", self.prey);
            return ActionResult::Failed;
        };

        let diff = predator_pos.tile - prey_pos.tile;
        let distance = diff.x.abs().max(diff.y.abs()) as f32;

        // Check if close enough to attack
        if distance <= 1.5 {
            self.state = HuntState::Attacking;
        }

        // If prey moved significantly, request new path
        if let Some(last_pos) = self.last_prey_pos {
            if (last_pos - prey_pos.tile).abs().max_element() > 3 {
                // Prey moved significantly, need new path
                self.state = HuntState::NeedPath;
            }
        }
        self.last_prey_pos = Some(prey_pos.tile);

        // State machine for async pathfinding
        match &self.state {
            HuntState::NeedPath => {
                // Signal system layer to queue pathfinding
                ActionResult::NeedsPathfinding { target: prey_pos.tile }
            }

            HuntState::WaitingForPath { request_id: _, target_pos } => {
                // Check for PathReady component (Phase 2: Component-based pathfinding)
                let entity_ref = world.get_entity(entity).ok();

                // Check if path is ready
                if let Some(entity_ref) = entity_ref {
                    if entity_ref.contains::<crate::pathfinding::PathReady>() {
                        // Path ready! System layer will insert MovementComponent
                        self.state = HuntState::Moving {
                            target_pos: *target_pos,
                        };
                        return ActionResult::InProgress;
                    }

                    // Check if path failed
                    if entity_ref.contains::<crate::pathfinding::PathFailed>() {
                        // Pathfinding failed, retry if under max retries
                        if self.retry_count < self.max_retries {
                            self.retry_count += 1;
                            self.state = HuntState::NeedPath;
                            debug!(
                                "Hunt path failed for entity {:?}, retry {}/{}",
                                entity, self.retry_count, self.max_retries
                            );
                            return ActionResult::InProgress;
                        } else {
                            debug!(
                                "Hunt gave up for entity {:?} after {} retries",
                                entity, self.max_retries
                            );
                            return ActionResult::Failed;
                        }
                    }
                }

                // Still waiting for path (no PathReady or PathFailed component yet)
                ActionResult::InProgress
            }

            HuntState::Moving { target_pos: _ } => {
                // Movement is handled by execute_movement_component system
                // Just continue progress - attack transition is handled by distance check above
                ActionResult::InProgress
            }

            HuntState::Attacking => {
                // We're close enough to attack!
                clear_navigation_state(world, entity);

                let bite_size = world
                    .get::<SpeciesNeeds>(entity)
                    .map(|n| n.eat_amount)
                    .unwrap_or(60.0);
                let available_meat = world
                    .get::<SpeciesNeeds>(self.prey)
                    .map(|n| n.eat_amount * 3.0)
                    .unwrap_or(80.0);

                // Allow predators to fully consume small prey (e.g., rabbits) while
                // still leaving carcasses for large kills.
                let consumed = if available_meat <= bite_size * 2.0 {
                    available_meat
                } else {
                    bite_size
                };

                // NOTE: Hunger changes, prey despawning, and carcass spawning
                // will be handled by system layer via Commands
                info!(
                    "🐺 Entity {:?} hunted prey {:?}, will consume {:.1} nutrition",
                    entity, self.prey, consumed
                );

                ActionResult::Success
            }
        }
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.state = HuntState::NeedPath;
        self.retry_count = 0;
        self.last_prey_pos = None;
    }

    fn name(&self) -> &'static str {
        "Hunt"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Helper function to clear navigation state
/// This is now a no-op since actions can't mutate World.
/// Navigation state clearing will be handled by the system layer via Commands.
/// Keeping function signature for compatibility during refactor.
fn clear_navigation_state(world: &World, entity: Entity) {
    let _ = (world, entity); // Suppress unused warnings
}
