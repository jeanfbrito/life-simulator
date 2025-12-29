/// AIEntityBundle - Single source of truth for AI tracker components
///
/// This bundle provides all the TRACKER components required for an entity to fully
/// participate in the AI trigger systems (trigger emitters, action queue, planning).
///
/// NOTE: This bundle does NOT include BehaviorConfig, SpeciesNeeds, or species-specific
/// components. Those are added separately by species spawn functions since they are
/// species-specific (e.g., RabbitBehavior::config(), DeerBehavior::needs()).
///
/// Components included:
/// - IdleTracker: Tracks idle time for long-idle replanning triggers
/// - StatThresholdTracker: Tracks stat threshold crossings for need-based replanning
/// - CurrentAction: Tracks current action state for visualization and debugging
use bevy::prelude::*;

use crate::ai::trigger_emitters::{IdleTracker, StatThresholdTracker};
use crate::entities::current_action::CurrentAction;

/// Bundle containing AI tracker components for entity spawning
///
/// This bundle ensures entities are immediately ready for AI trigger systems
/// without waiting for runtime initialization by initialize_new_entity_trackers.
///
/// Usage:
/// ```rust,ignore
/// commands.spawn((
///     // Species-specific components
///     RabbitBehavior::config(),
///     RabbitBehavior::needs(),
///     RabbitBehavior::stats_bundle(),
///     // AI tracker bundle
///     AIEntityBundle::new(0, 0.0, 0.0, 1.0),
///     // ... other components
/// ));
/// ```
#[derive(Bundle)]
pub struct AIEntityBundle {
    /// Tracks idle time for long-idle replanning triggers
    pub idle_tracker: IdleTracker,
    /// Tracks stat threshold crossings for need-based replanning
    pub stat_threshold_tracker: StatThresholdTracker,
    /// Current action state for visualization and debugging
    pub current_action: CurrentAction,
}

impl AIEntityBundle {
    /// Create a new AIEntityBundle with the specified initial stat values
    ///
    /// # Arguments
    /// * `initial_tick` - The current simulation tick (for idle tracking)
    /// * `initial_hunger` - Initial hunger level (0.0-1.0 normalized, 0.0 = not hungry)
    /// * `initial_thirst` - Initial thirst level (0.0-1.0 normalized, 0.0 = not thirsty)
    /// * `initial_energy` - Initial energy level (0.0-1.0 normalized, 1.0 = full energy)
    pub fn new(
        initial_tick: u64,
        initial_hunger: f32,
        initial_thirst: f32,
        initial_energy: f32,
    ) -> Self {
        Self {
            idle_tracker: IdleTracker::new(initial_tick),
            stat_threshold_tracker: StatThresholdTracker::new(
                initial_hunger,
                initial_thirst,
                initial_energy,
            ),
            current_action: CurrentAction::none(),
        }
    }

    /// Create a new AIEntityBundle with default initial stats (healthy entity at tick 0)
    ///
    /// Defaults to:
    /// - Tick: 0 (simulation start)
    /// - Hunger: 0.0 (not hungry)
    /// - Thirst: 0.0 (not thirsty)
    /// - Energy: 1.0 (full energy, normalized)
    pub fn default_healthy() -> Self {
        Self::new(0, 0.0, 0.0, 1.0)
    }

    /// Create a new AIEntityBundle at a specific tick with default healthy stats
    ///
    /// Use this when spawning entities after simulation start.
    ///
    /// # Arguments
    /// * `initial_tick` - The current simulation tick
    pub fn at_tick(initial_tick: u64) -> Self {
        Self::new(initial_tick, 0.0, 0.0, 1.0)
    }
}

impl Default for AIEntityBundle {
    fn default() -> Self {
        Self::default_healthy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_entity_bundle_new() {
        let bundle = AIEntityBundle::new(100, 0.5, 0.3, 0.8);

        // Verify idle tracker initialized with correct tick
        assert_eq!(bundle.idle_tracker.last_action_tick, 100);
        assert_eq!(bundle.idle_tracker.ticks_since_action, 0);

        // Verify stat threshold tracker initialized with correct values
        assert_eq!(bundle.stat_threshold_tracker.previous_hunger, 0.5);
        assert_eq!(bundle.stat_threshold_tracker.previous_thirst, 0.3);
        assert_eq!(bundle.stat_threshold_tracker.previous_energy, 0.8);
        assert!(!bundle.stat_threshold_tracker.hunger_triggered);
        assert!(!bundle.stat_threshold_tracker.thirst_triggered);
        assert!(!bundle.stat_threshold_tracker.energy_triggered);

        // Verify current action is idle/none
        assert!(
            bundle.current_action.action_name == "Idle"
                || bundle.current_action.action_name == "None"
        );
    }

    #[test]
    fn test_ai_entity_bundle_default_healthy() {
        let bundle = AIEntityBundle::default_healthy();

        // Verify default stat values (healthy entity)
        assert_eq!(bundle.stat_threshold_tracker.previous_hunger, 0.0);
        assert_eq!(bundle.stat_threshold_tracker.previous_thirst, 0.0);
        assert_eq!(bundle.stat_threshold_tracker.previous_energy, 1.0);

        // Verify idle tracker starts at tick 0
        assert_eq!(bundle.idle_tracker.last_action_tick, 0);
    }

    #[test]
    fn test_ai_entity_bundle_at_tick() {
        let bundle = AIEntityBundle::at_tick(500);

        // Verify idle tracker starts at specified tick
        assert_eq!(bundle.idle_tracker.last_action_tick, 500);

        // Verify default healthy stats
        assert_eq!(bundle.stat_threshold_tracker.previous_hunger, 0.0);
        assert_eq!(bundle.stat_threshold_tracker.previous_thirst, 0.0);
        assert_eq!(bundle.stat_threshold_tracker.previous_energy, 1.0);
    }

    #[test]
    fn test_ai_entity_bundle_default_trait() {
        let bundle = AIEntityBundle::default();

        // Default should be same as default_healthy
        assert_eq!(bundle.stat_threshold_tracker.previous_hunger, 0.0);
        assert_eq!(bundle.stat_threshold_tracker.previous_thirst, 0.0);
        assert_eq!(bundle.stat_threshold_tracker.previous_energy, 1.0);
        assert_eq!(bundle.idle_tracker.last_action_tick, 0);
    }
}
