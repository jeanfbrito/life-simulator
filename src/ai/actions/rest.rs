use super::*;
use crate::entities::stats::Energy;
use crate::simulation::tick::SimulationTick;
use bevy::prelude::*;

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
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        world.get::<Energy>(entity).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        let tick = world.get_resource::<SimulationTick>()
            .map(|t| t.0)
            .unwrap_or(0);

        if !self.started {
            if let Some(entity_ref) = world.get_entity(entity).ok() {
                if let Some(energy) = entity_ref.get::<Energy>() {
                    info!(
                        "😴 Entity {:?} started resting for {} ticks (energy: {:.1}%)",
                        entity,
                        self.duration_ticks,
                        energy.0.percentage()
                    );
                }
            }
            self.started = true;
            // NOTE: Energy state changes (set_resting/set_active) will be handled by system layer
        }

        self.ticks_remaining = self.ticks_remaining.saturating_sub(1);

        let energy_full = if let Some(entity_ref) = world.get_entity(entity).ok() {
            if let Some(energy) = entity_ref.get::<Energy>() {
                energy.0.is_full()
            } else {
                false
            }
        } else {
            false
        };

        if self.ticks_remaining == 0 || energy_full {
            if let Some(entity_ref) = world.get_entity(entity).ok() {
                if let Some(energy) = entity_ref.get::<Energy>() {
                    info!(
                        "😊 Entity {:?} finished resting on tick {}! Energy: {:.1}%",
                        entity,
                        tick,
                        energy.0.percentage()
                    );
                }
            }
            // NOTE: Energy state changes will be handled by system layer
            return ActionResult::Success;
        }

        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        // NOTE: Energy state changes will be handled by system layer
        debug!(
            "🚫 Entity {:?} resting interrupted, system will reset energy to active",
            entity
        );
    }

    fn name(&self) -> &'static str {
        "Rest"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
