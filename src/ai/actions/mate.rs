use super::*;
use crate::entities::stats::{Hunger, Thirst};
use crate::entities::TilePosition;
use crate::pathfinding::{Path, PathfindingFailed, PathRequestId};
use bevy::prelude::*;

// =============================================================================
// MATE ACTION
// =============================================================================

/// State machine for MateAction pathfinding
#[derive(Debug, Clone)]
pub enum MateState {
    /// Initial state - needs to request pathfinding
    NeedPath,
    /// Waiting for pathfinding system to provide a path
    WaitingForPath { request_id: PathRequestId },
    /// At meeting tile, waiting for partner
    AtMeetingTile,
}

/// Action: Rendezvous with partner and mate; pregnancy applied on female only
#[derive(Debug, Clone)]
pub struct MateAction {
    pub partner: Entity,
    pub meeting_tile: IVec2,
    pub duration_ticks: u32,
    pub started: bool,
    pub waited: u32,
    pub total_wait: u32,
    pub max_wait_ticks: u32,
    pub state: MateState,
}

impl MateAction {
    pub fn new(partner: Entity, meeting_tile: IVec2, duration_ticks: u32) -> Self {
        Self {
            partner,
            meeting_tile,
            duration_ticks,
            started: false,
            waited: 0,
            total_wait: 0,
            max_wait_ticks: duration_ticks.saturating_mul(5).max(duration_ticks + 25),
            state: MateState::NeedPath,
        }
    }

    /// Transition from NeedPath to WaitingForPath state
    /// Called by the action_pathfinding_bridge when path request is submitted
    pub fn transition_to_waiting(&mut self, request_id: PathRequestId) {
        self.state = MateState::WaitingForPath { request_id };
    }
}

impl Action for MateAction {
    fn can_execute(&self, world: &World, _entity: Entity) -> bool {
        world.get_entity(self.partner).is_ok() && world.get::<TilePosition>(self.partner).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        use crate::entities::reproduction::Sex;

        // CRITICAL NEEDS CHECK - survival override
        // If hunger or thirst are critical (>=80%), abort mating for survival
        if let Ok(entity_ref) = world.get_entity(entity) {
            if let (Some(hunger), Some(thirst)) = (
                entity_ref.get::<Hunger>(),
                entity_ref.get::<Thirst>()
            ) {
                if hunger.0.normalized() >= 0.80 || thirst.0.normalized() >= 0.80 {
                    info!(
                        "🚨 Mate aborted for {:?} - critical survival needs (H:{:.1}% T:{:.1}%)",
                        entity,
                        hunger.0.percentage(),
                        thirst.0.percentage()
                    );
                    return ActionResult::Failed; // Trigger replanning for survival
                }
            }
        }

        // NOTE: This action has been simplified to read-only World access.
        // All component mutations (removing ActiveMate/MatingTarget, PathfindingFailed, inserting Pregnancy, etc.)
        // will be handled by the system layer based on the returned ActionResult.

        // Abort if either entity failed to find a path
        if world.get::<PathfindingFailed>(entity).is_some()
            || world.get::<PathfindingFailed>(self.partner).is_some()
        {
            warn!(
                "⚠️ MateAction: pathfinding failed for {:?} or {:?}, aborting",
                entity, self.partner
            );
            // NOTE: Component removal handled by system layer
            return ActionResult::Failed;
        }

        // Abort if partner missing
        if world.get::<TilePosition>(self.partner).is_none() {
            warn!("⚠️ MateAction: partner {:?} missing", self.partner);
            // NOTE: Component removal handled by system layer
            return ActionResult::Failed;
        }

        let Some(me_pos) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };
        let Some(partner_pos) = world.get::<TilePosition>(self.partner).copied() else {
            return ActionResult::Failed;
        };

        // State machine for pathfinding
        match &self.state {
            MateState::NeedPath => {
                if me_pos.tile != self.meeting_tile {
                    // Request pathfinding to meeting tile
                    return ActionResult::NeedsPathfinding { target: self.meeting_tile };
                }
                // Already at meeting tile, transition directly
                self.state = MateState::AtMeetingTile;
            }
            MateState::WaitingForPath { request_id: _ } => {
                // Check if path was delivered
                if let Some(path) = world.get::<Path>(entity) {
                    if !path.is_complete() {
                        // Path received, entity will be moved by navigation system
                        return ActionResult::InProgress;
                    }
                }
                // Check if at meeting tile (path complete or already there)
                if me_pos.tile == self.meeting_tile {
                    self.state = MateState::AtMeetingTile;
                } else {
                    // Still waiting for path or entity to arrive
                    return ActionResult::InProgress;
                }
            }
            MateState::AtMeetingTile => {
                // Continue to mating logic below
            }
        }

        // We are on the meeting tile
        clear_navigation_state(world, entity);
        self.started = true;

        // Track total waiting time once we've reached the rendezvous point
        self.total_wait = self.total_wait.saturating_add(1);
        if self.total_wait > self.max_wait_ticks {
            debug!(
                "⚠️ MateAction: entity {:?} waited {} ticks for partner {:?} without success",
                entity, self.total_wait, self.partner
            );
            return ActionResult::Failed;
        }

        // At meeting tile: ensure partner arrives on same or adjacent tile
        let diff = (self.meeting_tile - partner_pos.tile).abs();
        let partner_adjacent = diff.x.max(diff.y) <= 1;

        if !partner_adjacent {
            // Partner is still approaching
            // NOTE: MoveOrder insertion for partner handled by system layer
            self.waited = self.waited.saturating_sub(1);

            debug!(
                "💕 MateAction: Entity {:?} waiting for partner {:?} - not adjacent. Me: {:?}, Partner: {:?}, Meeting: {:?}",
                entity, self.partner, me_pos.tile, partner_pos.tile, self.meeting_tile
            );

            return ActionResult::InProgress;
        }

        // Partner is adjacent—stop them from wandering off while waiting
        clear_navigation_state(world, self.partner);

        // Both are within touching distance: perform mating over duration
        self.waited = self.waited.saturating_add(1);

        // Debug logging for mating progress
        if self.waited <= 1 || self.waited % 10 == 0 || self.waited >= self.duration_ticks {
            info!(
                "💕 MateAction: Entity {:?} mating progress: {}/{} ticks, partner {:?} at meeting tile {:?}",
                entity, self.waited, self.duration_ticks, self.partner, self.meeting_tile
            );
        }

        if self.waited < self.duration_ticks {
            return ActionResult::InProgress;
        }

        // Duration complete: mating successful!
        // NOTE: System layer will handle:
        // - Pregnancy/ReproductionCooldown insertion based on Sex
        // - ActiveMate/MatingTarget relationship cleanup
        // - Navigation state clearing
        let me_female = world
            .get::<Sex>(entity)
            .is_some_and(|s| matches!(s, Sex::Female));
        let partner_female = world
            .get::<Sex>(self.partner)
            .is_some_and(|s| matches!(s, Sex::Female));

        info!(
            "❤️ Mating complete for entity {:?} with partner {:?} (me_female: {}, partner_female: {})",
            entity, self.partner, me_female, partner_female
        );

        // Clear navigation state (read-only operation)
        clear_navigation_state(world, entity);
        clear_navigation_state(world, self.partner);

        ActionResult::Success
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        // NOTE: System layer will handle ActiveMate/MatingTarget cleanup
        debug!(
            "🚫 Entity {:?} mating interrupted, system will clean up mating relationship",
            entity
        );

        clear_navigation_state(world, entity);
        clear_navigation_state(world, self.partner);
    }

    fn name(&self) -> &'static str {
        "Mate"
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
