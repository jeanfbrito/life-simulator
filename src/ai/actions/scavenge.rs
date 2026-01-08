use super::*;
use crate::entities::stats::Hunger;
use crate::entities::{Carcass, SpeciesNeeds, TilePosition};
use bevy::prelude::*;

// =============================================================================
// SCAVENGE ACTION
// =============================================================================

/// Action: Move to a carcass and consume available nutrition.
#[derive(Debug, Clone)]
pub struct ScavengeAction {
    pub carcass: Entity,
    pub started: bool,
}

impl ScavengeAction {
    pub fn new(carcass: Entity) -> Self {
        Self {
            carcass,
            started: false,
        }
    }
}

impl Action for ScavengeAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        world.get::<Hunger>(entity).is_some() && world.get::<Carcass>(self.carcass).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };

        let Some(carcass_pos) = world.get::<TilePosition>(self.carcass).copied() else {
            debug!("🦴 Scavenge target vanished before arrival");
            return ActionResult::Failed;
        };

        if position.tile != carcass_pos.tile {
            // NOTE: MoveOrder insertion will be handled by system layer
            self.started = true;
            return ActionResult::InProgress;
        }

        clear_navigation_state(world, entity);

        let bite_size = world
            .get::<SpeciesNeeds>(entity)
            .map(|n| n.eat_amount)
            .unwrap_or(50.0);

        // NOTE: Carcass consumption, hunger changes, and despawning
        // will be handled by system layer via Commands
        info!(
            "🦴 Entity {:?} scavenging from carcass {:?} (bite size: {:.1})",
            entity, self.carcass, bite_size
        );

        self.started = false;
        ActionResult::Success
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.started = false;
    }

    fn name(&self) -> &'static str {
        "Scavenge"
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
