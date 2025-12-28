use bevy::prelude::*;

use crate::ai::action::ActionType;
use crate::ai::behaviors::{
    evaluate_drinking_behavior, evaluate_eating_behavior, evaluate_fleeing_behavior,
    evaluate_follow_behavior, evaluate_grazing_behavior, evaluate_resting_behavior,
    evaluate_wandering_behavior, eating::HerbivoreDiet,
};
use crate::ai::planner::UtilityScore;
use crate::entities::reproduction::{Age, Mother, ReproductionConfig};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{ActiveMate, BehaviorConfig, FearState, TilePosition};
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;

/// Evaluate the baseline herbivore actions (drink, eat, rest, graze).
///
/// Species-specific modules can call this helper and then push any additional
/// actions (social, unique behaviours) afterwards.
pub fn evaluate_core_actions(
    position: &TilePosition,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    fear_state: Option<&FearState>,
    diet: &HerbivoreDiet,
) -> Vec<UtilityScore> {
    let mut actions = Vec::new();

    // Get fear utility modifier if fear state is available
    let fear_modifier = fear_state.map_or(1.0, |f| f.get_utility_modifier());

    if let Some(drink) = evaluate_drinking_behavior(
        position,
        thirst,
        world_loader,
        behavior_config.thirst_threshold,
        behavior_config.water_search_radius,
    ) {
        actions.push(drink);
    }

    if let Some(eat) = evaluate_eating_behavior(
        position,
        hunger,
        world_loader,
        resource_grid,
        behavior_config.hunger_threshold,
        behavior_config.food_search_radius,
        behavior_config.foraging_strategy,
        diet,
    ) {
        actions.push(eat);
    }

    if let Some(rest) =
        evaluate_resting_behavior(position, energy, behavior_config.energy_threshold)
    {
        actions.push(rest);
    }

    if let Some(graze) =
        evaluate_grazing_behavior(position, world_loader, behavior_config.graze_range)
    {
        actions.push(graze);
    }

    // Wandering - lowest priority idle behavior (always available)
    if let Some(wander_score) = evaluate_wandering_behavior(
        position,
        world_loader,
        behavior_config.wander_radius,
    ) {
        actions.push(wander_score);
    }

    // Apply fear modifiers to action utilities
    if fear_modifier < 1.0 {
        for action in &mut actions {
            // Reduce feeding-related utilities under fear
            match action.action_type {
                ActionType::Graze { .. } => {
                    action.utility *= fear_modifier;
                }
                // Drinking and resting utilities might be less affected
                ActionType::DrinkWater { .. } | ActionType::Rest { .. } => {
                    action.utility *= (1.0 + fear_modifier) / 2.0; // Moderate reduction
                }
                // Other actions (wandering, following, mating, etc.) are not affected
                _ => {}
            }

            // Log fear modification for debugging
            if fear_modifier < 0.9 {
                debug!(
                    "🦊 Fear modified action utility: {:?} {:.2} → {:.2} (modifier: {:.2})",
                    action.action_type,
                    action.utility / fear_modifier,
                    action.utility,
                    fear_modifier
                );
            }
        }
    }

    actions
}

/// Configuration for follow-the-mother behaviour.
#[derive(Debug, Clone, Copy)]
pub struct FollowConfig {
    pub stop_distance: i32,
    pub max_distance: i32,
}

/// Attempt to add a follow-mother action for juvenile herbivores.
///
/// Returns true if an action was added.
pub fn maybe_add_follow_mother(
    actions: &mut Vec<UtilityScore>,
    entity: Entity,
    position: &TilePosition,
    hunger: &Hunger,
    thirst: &Thirst,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    age: Option<&Age>,
    mother: Option<&Mother>,
    mother_position: Option<IVec2>,
    follow_cfg: FollowConfig,
) -> bool {
    let Some(age) = age else {
        return false;
    };
    if age.is_adult() {
        return false;
    }

    let Some(mother) = mother else {
        return false;
    };
    let Some(mother_pos) = mother_position else {
        return false;
    };

    let hunger_ok = hunger.0.normalized() < behavior_config.hunger_threshold;
    let thirst_ok = thirst.0.normalized() < behavior_config.thirst_threshold;
    let energy_ok = energy.0.normalized() > behavior_config.energy_threshold;
    if !(hunger_ok && thirst_ok && energy_ok) {
        return false;
    }

    let slice = [(mother.0, mother_pos)];
    if let Some(follow) = evaluate_follow_behavior(
        entity,
        position,
        &slice,
        follow_cfg.stop_distance,
        follow_cfg.max_distance,
    ) {
        actions.push(follow);
        true
    } else {
        false
    }
}

/// Parameters controlling mate-action queuing for a species.
#[derive(Debug, Clone, Copy)]
pub struct MateActionParams {
    pub utility: f32,
    pub priority: i32,
    /// Extra slack applied to hunger/thirst thresholds compared to the
    /// reproduction config requirements.
    pub threshold_margin: f32,
    /// Extra slack applied to minimum energy requirement.
    pub energy_margin: f32,
}

/// Add a mate action if the entity currently has a mating intent and meets
/// the reproduction requirements. Returns true if an action was added.
pub fn maybe_add_mate_action(
    actions: &mut Vec<UtilityScore>,
    mating_intent: Option<&ActiveMate>,
    repro_cfg: Option<&ReproductionConfig>,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    params: MateActionParams,
    current_tick: u64,
) -> bool {
    let Some(intent) = mating_intent else {
        return false;
    };
    let Some(cfg) = repro_cfg else {
        return false;
    };

    let thirst_level = thirst.0.normalized();
    let hunger_level = hunger.0.normalized();
    let energy_level = energy.0.normalized();

    // EMERGENCY OVERRIDE: Block mating if critically deprived
    //
    // This emergency check ensures survival always takes precedence over reproduction,
    // even if reproduction config thresholds would otherwise allow mating.
    //
    // Without this, entities can starve/dehydrate while mating because:
    // - Mate actions have high priority (350) that overrides survival actions (~100)
    // - Species configs may have lenient thresholds with large margins
    // - Emergency survival should ALWAYS block reproduction attempts
    //
    // At 90%+ hunger/thirst, entities begin taking health damage and should focus
    // exclusively on survival (eating/drinking) rather than reproduction.
    const EMERGENCY_HUNGER_THRESHOLD: f32 = 0.90;  // 90%+ hunger = emergency
    const EMERGENCY_THIRST_THRESHOLD: f32 = 0.90;  // 90%+ thirst = emergency

    if hunger_level >= EMERGENCY_HUNGER_THRESHOLD
        || thirst_level >= EMERGENCY_THIRST_THRESHOLD
    {
        debug!(
            "🚨 Mating blocked by emergency needs: hunger {:.1}%, thirst {:.1}%",
            hunger_level * 100.0,
            thirst_level * 100.0
        );
        return false;
    }

    // Existing safety checks (more lenient than emergency thresholds)
    let thirst_safe = thirst_level <= cfg.well_fed_thirst_norm + params.threshold_margin;
    let hunger_safe = hunger_level <= cfg.well_fed_hunger_norm + params.threshold_margin;
    let energy_safe = energy_level >= (cfg.min_energy_norm + params.energy_margin).min(1.0);

    if thirst_safe && hunger_safe && energy_safe {
        // Calculate duration_ticks as the time spent so far (current_tick - started_tick)
        let elapsed = current_tick.saturating_sub(intent.started_tick) as u32;
        actions.push(UtilityScore {
            action_type: ActionType::Mate {
                partner: intent.partner,
                meeting_tile: intent.meeting_tile,
                duration_ticks: elapsed,
            },
            utility: params.utility,
            priority: params.priority,
        });
        true
    } else {
        false
    }
}

/// Add a flee action if the entity is fearful and predators are nearby.
///
/// This function integrates the flee behavior from Phase 3 of the predator-prey system.
/// Returns true if a flee action was added.
///
/// # Parameters
/// - `actions`: Action list to potentially add flee action to
/// - `position`: Current position of the prey entity
/// - `fear_state`: Fear state (contains fear level and predator detection)
/// - `predator_positions`: List of nearby predator positions
/// - `world_loader`: World terrain data for pathfinding
///
/// # Returns
/// - `true` if flee action was added
/// - `false` if not fearful or no valid escape route
pub fn maybe_add_flee_action(
    actions: &mut Vec<UtilityScore>,
    position: &TilePosition,
    fear_state: Option<&FearState>,
    predator_positions: &[IVec2],
    world_loader: &WorldLoader,
) -> bool {
    // Only flee if fear state exists and indicates fear
    let Some(fear) = fear_state else {
        return false;
    };

    // Check if fear level is high enough to trigger fleeing
    if !fear.is_fearful() {
        return false;
    }

    // Find nearest predator to flee from
    let nearest_predator = find_nearest_predator(position.tile, predator_positions);
    let Some(predator_pos) = nearest_predator else {
        return false;
    };

    // Evaluate flee behavior
    if let Some(flee_action) =
        evaluate_fleeing_behavior(position, fear, predator_pos, world_loader)
    {
        debug!(
            "😱 Flee action added: utility {:.2}, priority {}, fear level {:.2}",
            flee_action.utility, flee_action.priority, fear.fear_level
        );
        actions.push(flee_action);
        true
    } else {
        false
    }
}

/// Find the nearest predator position to flee from
fn find_nearest_predator(prey_pos: IVec2, predator_positions: &[IVec2]) -> Option<IVec2> {
    predator_positions
        .iter()
        .min_by_key(|&&pred_pos| {
            let diff = prey_pos - pred_pos;
            diff.x.abs() + diff.y.abs() // Manhattan distance for faster computation
        })
        .copied()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod emergency_mating_tests {
    use super::*;
    use crate::entities::reproduction::{MatingIntent, ReproductionConfig};
    use bevy::prelude::Entity;

    /// Helper to create test reproduction config
    fn test_repro_config() -> ReproductionConfig {
        ReproductionConfig {
            maturity_ticks: 3600,
            gestation_ticks: 1200,
            mating_cooldown_ticks: 600,
            postpartum_cooldown_ticks: 1200,
            litter_size_range: (1, 4),
            mating_search_radius: 20,
            well_fed_hunger_norm: 0.3,  // Must be <= 30% hungry
            well_fed_thirst_norm: 0.3,  // Must be <= 30% thirsty
            well_fed_required_ticks: 600,
            matching_interval_ticks: 100,
            mating_duration_ticks: 100,
            min_energy_norm: 0.4,        // Must have >= 40% energy
            min_health_norm: 0.5,
        }
    }

    /// Helper to create test mating intent
    fn test_mating_intent() -> crate::entities::ActiveMate {
        crate::entities::ActiveMate {
            partner: Entity::from_raw(999),
            meeting_tile: IVec2::new(10, 10),
            started_tick: 0, // Started at tick 0
        }
    }

    /// Helper to create test mate action params
    fn test_mate_params() -> MateActionParams {
        MateActionParams {
            utility: 0.8,
            priority: 350,
            threshold_margin: 0.1,
            energy_margin: 0.05,
        }
    }

    #[test]
    fn test_mating_blocked_when_critically_hungry() {
        // Setup: Entity with critical hunger (>90%) but within reproduction config thresholds
        // This tests the emergency override that should block mating regardless of config margins
        let mut hunger = Hunger::new();
        hunger.0.set(95.0);  // 95% hungry = critical emergency

        let thirst = Thirst::new();  // Normal thirst (0%)
        let energy = Energy::new();  // Normal energy (100%)

        let mating_intent = test_mating_intent();

        // Create a permissive config that would normally allow mating at 95% hunger
        // (if we had huge margins) - emergency check should override this
        let mut repro_cfg = test_repro_config();
        repro_cfg.well_fed_hunger_norm = 1.0;  // Very permissive (allow up to 100% hunger)

        let mut actions = Vec::new();
        let params = MateActionParams {
            utility: 0.8,
            priority: 350,
            threshold_margin: 0.1,  // Even with margin, 95% > 90% emergency threshold
            energy_margin: 0.05,
        };

        let result = maybe_add_mate_action(
            &mut actions,
            Some(&mating_intent),
            Some(&repro_cfg),
            &thirst,
            &hunger,
            &energy,
            params,
            100, // current_tick
        );

        assert!(!result, "Mating should be blocked when critically hungry (95%)");
        assert!(actions.is_empty(), "No mate action should be added when critically hungry");
    }

    #[test]
    fn test_mating_blocked_when_critically_thirsty() {
        // Setup: Entity with critical thirst (>90%) but within reproduction config thresholds
        let hunger = Hunger::new();  // Normal hunger (0%)

        let mut thirst = Thirst::new();
        thirst.0.set(95.0);  // 95% thirsty = critical emergency

        let energy = Energy::new();  // Normal energy (100%)

        let mating_intent = test_mating_intent();

        // Create permissive config
        let mut repro_cfg = test_repro_config();
        repro_cfg.well_fed_thirst_norm = 1.0;  // Very permissive

        let mut actions = Vec::new();
        let params = MateActionParams {
            utility: 0.8,
            priority: 350,
            threshold_margin: 0.1,
            energy_margin: 0.05,
        };

        let result = maybe_add_mate_action(
            &mut actions,
            Some(&mating_intent),
            Some(&repro_cfg),
            &thirst,
            &hunger,
            &energy,
            params,
            100, // current_tick
        );

        assert!(!result, "Mating should be blocked when critically thirsty (95%)");
        assert!(actions.is_empty(), "No mate action should be added when critically thirsty");
    }

    #[test]
    fn test_mating_blocked_at_90_percent_threshold() {
        // Setup: Entity exactly at 90% threshold
        let mut hunger = Hunger::new();
        hunger.0.set(90.0);  // Exactly 90% = should block

        let thirst = Thirst::new();
        let energy = Energy::new();

        let mating_intent = test_mating_intent();

        // Create permissive config
        let mut repro_cfg = test_repro_config();
        repro_cfg.well_fed_hunger_norm = 1.0;

        let mut actions = Vec::new();
        let params = MateActionParams {
            utility: 0.8,
            priority: 350,
            threshold_margin: 0.1,
            energy_margin: 0.05,
        };

        let result = maybe_add_mate_action(
            &mut actions,
            Some(&mating_intent),
            Some(&repro_cfg),
            &thirst,
            &hunger,
            &energy,
            params,
            100, // current_tick
        );

        assert!(!result, "Mating should be blocked at exactly 90% hunger threshold");
        assert!(actions.is_empty(), "No mate action should be added at 90% threshold");
    }

    #[test]
    fn test_mating_allowed_when_not_critical() {
        // Setup: Entity with safe stats (below emergency threshold)
        let mut hunger = Hunger::new();
        hunger.0.set(30.0);  // 30% hungry = safe (below 90%)

        let mut thirst = Thirst::new();
        thirst.0.set(20.0);  // 20% thirsty = safe (below 90%)

        let energy = Energy::new();  // 100% energy = safe

        let mating_intent = test_mating_intent();
        let repro_cfg = test_repro_config();

        let mut actions = Vec::new();
        let result = maybe_add_mate_action(
            &mut actions,
            Some(&mating_intent),
            Some(&repro_cfg),
            &thirst,
            &hunger,
            &energy,
            test_mate_params(),
            100, // current_tick
        );

        assert!(result, "Mating should be allowed when stats are safe (30% hunger, 20% thirst)");
        assert_eq!(actions.len(), 1, "Mate action should be added when not critical");

        // Verify the action was created correctly
        if let Some(action) = actions.first() {
            assert_eq!(action.priority, 350, "Mate action priority should be 350");
            match &action.action_type {
                ActionType::Mate { partner, .. } => {
                    assert_eq!(*partner, Entity::from_raw(999), "Partner should match");
                }
                _ => panic!("Expected Mate action type"),
            }
        }
    }

    #[test]
    fn test_mating_blocked_when_both_needs_critical() {
        // Setup: Both hunger and thirst critical
        let mut hunger = Hunger::new();
        hunger.0.set(95.0);  // Critical hunger

        let mut thirst = Thirst::new();
        thirst.0.set(92.0);  // Critical thirst

        let energy = Energy::new();

        let mating_intent = test_mating_intent();
        let repro_cfg = test_repro_config();

        let mut actions = Vec::new();
        let result = maybe_add_mate_action(
            &mut actions,
            Some(&mating_intent),
            Some(&repro_cfg),
            &thirst,
            &hunger,
            &energy,
            test_mate_params(),
            100, // current_tick
        );

        assert!(!result, "Mating should be blocked when both hunger and thirst are critical");
        assert!(actions.is_empty(), "No mate action when both needs are critical");
    }
}
