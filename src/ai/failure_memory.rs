/// Action Failure Memory System (Dwarf Fortress Style)
///
/// Tracks recent action failures per entity and applies cooldowns to prevent
/// infinite retry loops. When an action fails, its utility is penalized for
/// a cooldown period, encouraging the AI to try alternative actions.
///
/// This complements the "Failed = Replan" invariant by making replans smarter.
use bevy::prelude::*;
use std::collections::HashMap;

use super::actions::ActionType;

/// How long (in ticks) before a failed action can be retried at full utility
const DEFAULT_FAILURE_COOLDOWN: u64 = 50; // ~5 seconds at 10 TPS

/// Utility multiplier applied to actions on cooldown (0.0 = disabled, 1.0 = no penalty)
const COOLDOWN_UTILITY_MULTIPLIER: f32 = 0.1; // 90% penalty while on cooldown

/// Maximum failures to remember per entity (prevents unbounded memory growth)
const MAX_FAILURES_PER_ENTITY: usize = 10;

/// Tracks action failures for a single entity
#[derive(Component, Debug, Clone, Default)]
pub struct ActionFailureMemory {
    /// Map of action type name -> tick when it last failed
    pub failures: HashMap<String, u64>,
}

impl ActionFailureMemory {
    /// Record a failure for an action type
    pub fn record_failure(&mut self, action_type: &str, tick: u64) {
        // Limit size by removing oldest if at capacity
        if self.failures.len() >= MAX_FAILURES_PER_ENTITY {
            // Find and remove the oldest failure
            if let Some(oldest_key) = self
                .failures
                .iter()
                .min_by_key(|(_, &tick)| tick)
                .map(|(k, _)| k.clone())
            {
                self.failures.remove(&oldest_key);
            }
        }

        self.failures.insert(action_type.to_string(), tick);
    }

    /// Check if an action is on cooldown
    pub fn is_on_cooldown(&self, action_type: &str, current_tick: u64) -> bool {
        if let Some(&failure_tick) = self.failures.get(action_type) {
            current_tick < failure_tick + DEFAULT_FAILURE_COOLDOWN
        } else {
            false
        }
    }

    /// Get utility multiplier for an action (1.0 if not on cooldown, lower if on cooldown)
    pub fn get_utility_multiplier(&self, action_type: &str, current_tick: u64) -> f32 {
        if self.is_on_cooldown(action_type, current_tick) {
            COOLDOWN_UTILITY_MULTIPLIER
        } else {
            1.0
        }
    }

    /// Get remaining cooldown ticks (0 if not on cooldown)
    pub fn remaining_cooldown(&self, action_type: &str, current_tick: u64) -> u64 {
        if let Some(&failure_tick) = self.failures.get(action_type) {
            let cooldown_end = failure_tick + DEFAULT_FAILURE_COOLDOWN;
            if current_tick < cooldown_end {
                cooldown_end - current_tick
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Clear expired failures (cleanup)
    pub fn cleanup_expired(&mut self, current_tick: u64) {
        self.failures
            .retain(|_, &mut failure_tick| current_tick < failure_tick + DEFAULT_FAILURE_COOLDOWN);
    }

    /// Clear a specific failure (e.g., when action succeeds)
    pub fn clear_failure(&mut self, action_type: &str) {
        self.failures.remove(action_type);
    }

    /// Clear all failures (e.g., when entity state changes significantly)
    pub fn clear_all(&mut self) {
        self.failures.clear();
    }
}

/// Apply failure memory penalties to utility scores
///
/// Call this during planning to penalize recently failed actions.
pub fn apply_failure_penalties(
    scores: &mut [super::planner::UtilityScore],
    memory: &ActionFailureMemory,
    current_tick: u64,
) {
    for score in scores.iter_mut() {
        let action_name = action_type_to_string(&score.action_type);
        let multiplier = memory.get_utility_multiplier(&action_name, current_tick);

        if multiplier < 1.0 {
            let old_utility = score.utility;
            score.utility *= multiplier;
            let base_name = action_type_to_base_name(&score.action_type);
            debug!(
                "⏳ Action '{}' on cooldown ({}): utility {:.2} → {:.2} ({}% penalty)",
                base_name,
                action_name,
                old_utility,
                score.utility,
                ((1.0 - multiplier) * 100.0) as i32
            );
        }
    }
}

/// Convert ActionType to string for failure tracking
///
/// IMPORTANT: For actions with targets, includes the target in the key.
/// This prevents "Graze to A fails" from penalizing "Graze to B".
pub fn action_type_to_string(action_type: &ActionType) -> String {
    match action_type {
        // Target-specific actions - include destination in key
        ActionType::Graze { target_tile } => format!("Graze:({},{})", target_tile.x, target_tile.y),
        ActionType::DrinkWater { target_tile } => format!("DrinkWater:({},{})", target_tile.x, target_tile.y),
        ActionType::Wander { target_tile } => format!("Wander:({},{})", target_tile.x, target_tile.y),
        ActionType::Harvest { target_tile, .. } => format!("Harvest:({},{})", target_tile.x, target_tile.y),

        // Entity-targeted actions - include entity ID (prevents hunting same prey repeatedly)
        ActionType::Hunt { prey } => format!("Hunt:{}", prey.to_bits()),
        ActionType::Scavenge { carcass } => format!("Scavenge:{}", carcass.to_bits()),
        ActionType::Follow { target, .. } => format!("Follow:{}", target.to_bits()),
        ActionType::Mate { partner, .. } => format!("Mate:{}", partner.to_bits()),

        // Duration-only actions - just use type name (these rarely fail pathfinding)
        ActionType::Rest { .. } => "Rest".to_string(),
    }
}

/// Convert ActionType to base type name (without target) for display
pub fn action_type_to_base_name(action_type: &ActionType) -> &'static str {
    match action_type {
        ActionType::Graze { .. } => "Graze",
        ActionType::DrinkWater { .. } => "DrinkWater",
        ActionType::Rest { .. } => "Rest",
        ActionType::Wander { .. } => "Wander",
        ActionType::Hunt { .. } => "Hunt",
        ActionType::Scavenge { .. } => "Scavenge",
        ActionType::Follow { .. } => "Follow",
        ActionType::Mate { .. } => "Mate",
        ActionType::Harvest { .. } => "Harvest",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_check_cooldown() {
        let mut memory = ActionFailureMemory::default();

        // No failure recorded
        assert!(!memory.is_on_cooldown("Wander", 100));
        assert_eq!(memory.get_utility_multiplier("Wander", 100), 1.0);

        // Record failure at tick 100
        memory.record_failure("Wander", 100);

        // Should be on cooldown
        assert!(memory.is_on_cooldown("Wander", 100));
        assert!(memory.is_on_cooldown("Wander", 149)); // Still in cooldown
        assert_eq!(
            memory.get_utility_multiplier("Wander", 100),
            COOLDOWN_UTILITY_MULTIPLIER
        );

        // After cooldown expires
        assert!(!memory.is_on_cooldown("Wander", 150));
        assert_eq!(memory.get_utility_multiplier("Wander", 150), 1.0);
    }

    #[test]
    fn test_remaining_cooldown() {
        let mut memory = ActionFailureMemory::default();
        memory.record_failure("Graze", 100);

        assert_eq!(memory.remaining_cooldown("Graze", 100), DEFAULT_FAILURE_COOLDOWN);
        assert_eq!(memory.remaining_cooldown("Graze", 120), DEFAULT_FAILURE_COOLDOWN - 20);
        assert_eq!(memory.remaining_cooldown("Graze", 150), 0);
    }

    #[test]
    fn test_max_failures_limit() {
        let mut memory = ActionFailureMemory::default();

        // Fill up to max
        for i in 0..MAX_FAILURES_PER_ENTITY {
            memory.record_failure(&format!("Action{}", i), i as u64);
        }

        assert_eq!(memory.failures.len(), MAX_FAILURES_PER_ENTITY);

        // Adding one more should evict oldest
        memory.record_failure("NewAction", 1000);
        assert_eq!(memory.failures.len(), MAX_FAILURES_PER_ENTITY);
        assert!(memory.failures.contains_key("NewAction"));
        assert!(!memory.failures.contains_key("Action0")); // Oldest evicted
    }

    #[test]
    fn test_clear_failure() {
        let mut memory = ActionFailureMemory::default();
        memory.record_failure("Wander", 100);
        assert!(memory.is_on_cooldown("Wander", 100));

        memory.clear_failure("Wander");
        assert!(!memory.is_on_cooldown("Wander", 100));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut memory = ActionFailureMemory::default();
        memory.record_failure("OldAction", 0);
        memory.record_failure("NewAction", 100);

        memory.cleanup_expired(100);

        assert!(!memory.failures.contains_key("OldAction")); // Expired
        assert!(memory.failures.contains_key("NewAction")); // Still valid
    }

    #[test]
    fn test_target_specific_failures() {
        use bevy::math::IVec2;

        // Test that failing Graze to target A doesn't affect Graze to target B
        let graze_a = ActionType::Graze { target_tile: IVec2::new(10, 20) };
        let graze_b = ActionType::Graze { target_tile: IVec2::new(30, 40) };

        let key_a = action_type_to_string(&graze_a);
        let key_b = action_type_to_string(&graze_b);

        // Keys should be different
        assert_ne!(key_a, key_b);
        assert_eq!(key_a, "Graze:(10,20)");
        assert_eq!(key_b, "Graze:(30,40)");

        // Record failure for target A only
        let mut memory = ActionFailureMemory::default();
        memory.record_failure(&key_a, 100);

        // Target A should be on cooldown
        assert!(memory.is_on_cooldown(&key_a, 100));

        // Target B should NOT be on cooldown
        assert!(!memory.is_on_cooldown(&key_b, 100));
    }
}
