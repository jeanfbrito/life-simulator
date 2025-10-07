use crate::entities::entity_types::{Herbivore, Wolf};
use crate::entities::{Creature, TilePosition};
use crate::vegetation::constants::predator_effects::*;
/// Predator fear system for herbivore behavior modification
///
/// This module implements predator proximity detection and fear-based behavior
/// modification as outlined in Phase 3 of the plant system plan.
use bevy::prelude::*;

/// Component representing fear state in herbivores
#[derive(Component, Debug, Clone)]
pub struct FearState {
    /// Current fear level (0.0 = no fear, 1.0 = maximum fear)
    pub fear_level: f32,
    /// Number of nearby predators detected
    pub nearby_predators: u32,
    /// Ticks since last predator detection (for fear decay)
    pub ticks_since_danger: u32,
    /// Maximum fear level reached recently (for persistent effects)
    pub peak_fear: f32,
}

impl Default for FearState {
    fn default() -> Self {
        Self {
            fear_level: 0.0,
            nearby_predators: 0,
            ticks_since_danger: 0,
            peak_fear: 0.0,
        }
    }
}

impl FearState {
    /// Create a new fear state
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply fear stimulus from predator detection
    pub fn apply_fear_stimulus(&mut self, predator_count: u32) {
        self.nearby_predators = predator_count;
        self.ticks_since_danger = 0;

        // Calculate fear level based on predator count
        self.fear_level = (predator_count as f32 * 0.4).min(1.0);
        self.peak_fear = self.peak_fear.max(self.fear_level);
    }

    /// Decay fear over time when no predators present
    pub fn decay_fear(&mut self) {
        self.ticks_since_danger += 1;

        // Only decay if no predators detected recently
        if self.nearby_predators == 0 {
            // Exponential decay with half-life of ~30 ticks (3 seconds)
            let decay_rate = 0.95;
            self.fear_level *= decay_rate;

            // Reset peak fear after complete decay
            if self.fear_level < 0.01 {
                self.peak_fear = 0.0;
            }
        }

        // Reset predator count after safety period
        if self.ticks_since_danger > 10 {
            self.nearby_predators = 0;
        }
    }

    /// Check if entity is currently fearful
    pub fn is_fearful(&self) -> bool {
        self.fear_level > 0.1
    }

    /// Get fear multiplier for utility modification
    pub fn get_utility_modifier(&self) -> f32 {
        // Higher fear reduces feeding utility but increases escape utility
        if self.is_fearful() {
            1.0 - (self.fear_level * 0.5) // Up to 50% reduction in feeding utility
        } else {
            1.0
        }
    }

    /// Get movement speed modifier under fear
    pub fn get_speed_modifier(&self) -> f32 {
        if self.is_fearful() {
            // Move faster when fearful (escape response)
            1.0 + (self.fear_level * (FEAR_SPEED_BOOST - 1.0))
        } else {
            1.0
        }
    }

    /// Get feeding duration reduction under fear
    pub fn get_feeding_reduction(&self) -> f32 {
        if self.is_fearful() {
            // Feed less when fearful (vigilance trade-off)
            self.fear_level * FEAR_FEEDING_REDUCTION
        } else {
            0.0
        }
    }

    /// Get biomass tolerance increase under fear
    pub fn get_biomass_tolerance(&self) -> f32 {
        if self.is_fearful() {
            // Accept lower quality food when fearful
            self.fear_level * FEAR_BIOMASS_TOLERANCE
        } else {
            0.0
        }
    }
}

/// System to detect predator proximity and update fear states
pub fn predator_proximity_system(
    mut prey_query: Query<
        (Entity, &Creature, &TilePosition, &mut FearState),
        (With<Herbivore>, Without<Wolf>),
    >,
    predator_query: Query<&TilePosition, With<Wolf>>,
) {
    // Collect predator positions
    let predator_positions: Vec<IVec2> = predator_query.iter().map(|pos| pos.tile).collect();

    // Update fear states for all prey
    for (entity, creature, prey_pos, mut fear_state) in prey_query.iter_mut() {
        let mut nearby_predators = 0;

        // Check each predator
        for predator_pos in &predator_positions {
            let distance = prey_pos.tile.as_vec2().distance(predator_pos.as_vec2());

            if distance <= FEAR_RADIUS as f32 {
                nearby_predators += 1;

                // Log fear detection
                debug!(
                    "👀 Fear sensor: entity {:?} detects predator at distance {:.1} (radius {})",
                    entity, distance, FEAR_RADIUS
                );
            }
        }

        // Apply fear stimulus if predators detected
        if nearby_predators > 0 {
            fear_state.apply_fear_stimulus(nearby_predators);

            info!(
                "😨 {} {:?} fear level: {:.2} ({} predators within {} tiles)",
                creature.species, entity, fear_state.fear_level, nearby_predators, FEAR_RADIUS
            );
        } else {
            let was_fearful = fear_state.is_fearful();
            fear_state.decay_fear();

            if was_fearful && !fear_state.is_fearful() {
                debug!(
                    "🙂 {} {:?} fear dissipated after {} ticks without predators",
                    creature.species, entity, fear_state.ticks_since_danger
                );
            }
        }
    }
}

/// System to apply fear-based movement speed modifications
pub fn fear_speed_system(
    mut prey_query: Query<
        (
            &mut FearState,
            &mut crate::entities::MovementSpeed,
            &Creature,
        ),
        (With<Herbivore>, Without<Wolf>),
    >,
) {
    for (fear_state, mut movement_speed, creature) in prey_query.iter_mut() {
        if fear_state.is_fearful() {
            let speed_modifier = fear_state.get_speed_modifier();
            let base_speed = movement_speed.ticks_per_move;

            // Apply speed boost (reduce ticks per tile)
            let boosted_speed = (base_speed as f32 / speed_modifier) as u32;
            movement_speed.ticks_per_move = boosted_speed.max(1); // Minimum 1 tick per move

            debug!(
                "🏃 Fear speed boost: {:.2}x ({} → {} ticks/tile)",
                speed_modifier, base_speed, movement_speed.ticks_per_move
            );

            debug!(
                "🐾 {} speed boost: {:.2}x ({} → {} ticks/tile)",
                creature.species, speed_modifier, base_speed, movement_speed.ticks_per_move
            );
        }
    }
}

/// Plugin to register predator fear systems
pub struct FearPlugin;

impl Plugin for FearPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (predator_proximity_system, fear_speed_system).chain(),
        );

        // Initialize fear states for all existing herbivores
        app.add_systems(Startup, initialize_fear_states);
    }
}

/// System to initialize fear states for existing herbivores
fn initialize_fear_states(
    mut commands: Commands,
    herbivore_query: Query<Entity, (With<Herbivore>, Without<FearState>)>,
) {
    for entity in herbivore_query.iter() {
        commands.entity(entity).insert(FearState::new());
        debug!("🐰 Initialized fear state for entity {:?}", entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fear_state_decay() {
        let mut fear_state = FearState {
            fear_level: 0.8,
            nearby_predators: 0,
            ticks_since_danger: 0,
            peak_fear: 0.8,
        };

        // Decay over time
        for _ in 0..50 {
            fear_state.decay_fear();
        }

        assert!(
            fear_state.fear_level < 0.1,
            "Fear should decay significantly"
        );
    }

    #[test]
    fn test_fear_utility_modifier() {
        let mut fear_state = FearState::new();

        // No fear
        assert_eq!(fear_state.get_utility_modifier(), 1.0);

        // High fear
        fear_state.fear_level = 0.8;
        assert!(fear_state.get_utility_modifier() < 1.0);
        assert!(fear_state.get_utility_modifier() > 0.5);
    }

    #[test]
    fn test_fear_speed_modifier() {
        let mut fear_state = FearState::new();

        // No fear
        assert_eq!(fear_state.get_speed_modifier(), 1.0);

        // High fear
        fear_state.fear_level = 1.0;
        assert_eq!(fear_state.get_speed_modifier(), FEAR_SPEED_BOOST);
    }
}
