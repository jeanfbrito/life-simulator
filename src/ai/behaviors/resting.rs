use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::{stats::Energy, TilePosition};
/// Resting Behavior - for entities that need sleep/energy recovery
///
/// This behavior makes entities find safe spots and rest when tired.
/// Suitable for: All animals (Rabbits, Deer, Wolves, etc.)
use bevy::prelude::*;

/// Evaluate the utility of resting
///
/// Returns a resting action if energy is below threshold.
/// Entities rest in place - no need to find a special location (for now).
///
/// # Parameters
/// - `position`: Current position of the entity
/// - `energy`: Current energy level
/// - `energy_threshold`: Minimum energy level to trigger rest (0.0-1.0)
///   Note: Lower energy = more tired, so threshold should be inverted
///   Example: 0.3 threshold means rest when energy drops below 30%
///
/// # Returns
/// - `Some(UtilityScore)` if tired enough
/// - `None` if not tired
pub fn evaluate_resting_behavior(
    _position: &TilePosition,
    energy: &Energy,
    energy_threshold: f32,
) -> Option<UtilityScore> {
    // Energy is normalized 0.0-1.0 where 1.0 = full energy, 0.0 = exhausted
    // So we rest when energy is LOW (below threshold)
    let energy_level = energy.0.normalized();

    // If energy is above threshold, we're not tired enough to rest
    if energy_level >= energy_threshold {
        return None; // Too much energy, keep moving
    }

    // Calculate tiredness (inverted energy)
    let tiredness = 1.0 - energy_level;

    // Utility is purely based on how tired we are
    // More tired = higher utility to rest
    let utility = tiredness;

    // Calculate priority based on urgency
    let priority = if energy_level < 0.15 {
        800 // Critical - about to collapse
    } else if energy_level < 0.3 {
        300 // Important - quite tired
    } else {
        50 // Low priority - slightly tired
    };

    Some(UtilityScore {
        action_type: ActionType::Rest {
            duration_ticks: calculate_rest_duration(energy_level),
        },
        utility,
        priority,
    })
}

/// Calculate how many ticks to rest based on energy level
/// More tired = rest longer
fn calculate_rest_duration(energy_level: f32) -> u32 {
    if energy_level < 0.15 {
        600 // Very tired - sleep ~60 seconds at 10 TPS
    } else if energy_level < 0.3 {
        300 // Moderately tired - sleep ~30 seconds
    } else {
        150 // Slightly tired - nap ~15 seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::stats::Stat;

    #[test]
    fn test_rest_when_low_energy() {
        let position = TilePosition::from_tile(IVec2::new(0, 0));
        let mut energy = Energy(Stat::new(20.0, 0.0, 100.0, -0.05)); // 20% energy

        let result = evaluate_resting_behavior(&position, &energy, 0.3);

        assert!(
            result.is_some(),
            "Should want to rest at 20% energy with 30% threshold"
        );

        let score = result.unwrap();
        assert!(matches!(score.action_type, ActionType::Rest { .. }));
        assert!(
            score.utility > 0.7,
            "Utility should be high when very tired"
        );
    }

    #[test]
    fn test_no_rest_when_high_energy() {
        let position = TilePosition::from_tile(IVec2::new(0, 0));
        let energy = Energy(Stat::new(80.0, 0.0, 100.0, -0.05)); // 80% energy

        let result = evaluate_resting_behavior(&position, &energy, 0.3);

        assert!(
            result.is_none(),
            "Should NOT rest at 80% energy with 30% threshold"
        );
    }

    #[test]
    fn test_rest_duration_scales_with_tiredness() {
        // Very low energy = longer rest
        let duration_very_low = calculate_rest_duration(0.10);
        let duration_moderate = calculate_rest_duration(0.25);
        let duration_slight = calculate_rest_duration(0.40);

        assert!(duration_very_low > duration_moderate);
        assert!(duration_moderate > duration_slight);
    }
}
