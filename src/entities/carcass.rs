//! Carcass component and management systems supporting scavengers.

use bevy::prelude::*;

use crate::simulation::SimulationTick;

/// Represents a carcass left behind after an animal dies.
#[derive(Component, Debug, Clone)]
pub struct Carcass {
    /// Total nutrition remaining (reduces as predators feed).
    pub nutrition: f32,
    /// Remaining ticks before the carcass decays completely.
    pub decay_ticks: u32,
    /// Species name for logging/visualisation.
    pub species: String,
}

impl Carcass {
    /// Create a new carcass with supplied nutrition and optional decay duration.
    pub fn new(species: impl Into<String>, nutrition: f32, decay_ticks: u32) -> Self {
        Self {
            nutrition,
            decay_ticks,
            species: species.into(),
        }
    }

    /// Amount actually consumed (clamped to available nutrition).
    pub fn consume(&mut self, amount: f32) -> f32 {
        let consumed = amount.min(self.nutrition);
        self.nutrition -= consumed;
        consumed
    }

    /// Whether the carcass has fully decayed or been eaten.
    pub fn is_spent(&self) -> bool {
        self.nutrition <= 0.1 || self.decay_ticks == 0
    }
}

/// Tick down carcasses each simulation tick and despawn when spent.
pub fn tick_carcasses(
    mut commands: Commands,
    mut carcasses: Query<(Entity, &mut Carcass)>,
    tick: Res<SimulationTick>,
) {
    if tick.0 % 2 != 0 {
        // Decay every other tick to reduce churn.
        return;
    }

    for (entity, mut carcass) in carcasses.iter_mut() {
        if carcass.decay_ticks > 0 {
            carcass.decay_ticks -= 1;
        }

        if carcass.is_spent() {
            debug!(
                "🦴 Carcass of {} removed (remaining nutrition {:.1})",
                carcass.species, carcass.nutrition
            );
            commands.entity(entity).despawn();
        }
    }
}
