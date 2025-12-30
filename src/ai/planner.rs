use super::actions::ActionType;
use super::queue::ActionQueue;
use crate::ai::herbivore_toolkit::{
    maybe_add_follow_mother, maybe_add_mate_action, FollowConfig, MateActionParams,
};
use crate::entities::reproduction::{Age, Mother, ReproductionConfig};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{ActiveMate, MatingTarget, BehaviorConfig, TilePosition};
use bevy::prelude::*;
use std::collections::HashMap;

/// Utility score with associated action
#[derive(Debug, Clone)]
pub struct UtilityScore {
    pub action_type: ActionType,
    pub utility: f32,
    pub priority: i32,
}

/// Planner configuration
pub const UTILITY_THRESHOLD: f32 = 0.05; // Queue only meaningful actions

/// Emergency thresholds for critical survival needs
/// These are more conservative (trigger earlier) than the emergency mating block (90%)
/// to provide time to boost survival actions before health damage begins.
const EMERGENCY_HUNGER_THRESHOLD: f32 = 0.85; // 85%+ hunger = emergency
const EMERGENCY_THIRST_THRESHOLD: f32 = 0.85; // 85%+ thirst = emergency
const EMERGENCY_ENERGY_THRESHOLD: f32 = 0.15; // 15% or less energy = emergency

/// Priority value assigned to survival actions during emergencies
const EMERGENCY_SURVIVAL_PRIORITY: i32 = 500;

/// Apply emergency priority overrides to survival actions when entity is critically deprived.
///
/// This function modifies action utilities and priorities in-place when emergency conditions
/// are detected. Emergency conditions are:
/// - Hunger >= 85% (entities take health damage at 90%+)
/// - Thirst >= 85%
/// - Energy <= 15%
///
/// During emergencies:
/// - Survival actions (Graze, DrinkWater, Rest) get 2x utility and priority 500
/// - Mating actions are suppressed to priority 0 and 0.1x utility
///
/// This ensures entities prioritize survival over reproduction when critically deprived.
fn apply_emergency_priority_override(
    actions: &mut [UtilityScore],
    hunger: &Hunger,
    thirst: &Thirst,
    energy: &Energy,
) {
    let hunger_critical = hunger.0.normalized() >= EMERGENCY_HUNGER_THRESHOLD;
    let thirst_critical = thirst.0.normalized() >= EMERGENCY_THIRST_THRESHOLD;
    let energy_critical = energy.0.normalized() <= EMERGENCY_ENERGY_THRESHOLD;

    if hunger_critical || thirst_critical || energy_critical {
        debug!(
            "🚨 EMERGENCY MODE: hunger {:.1}%, thirst {:.1}%, energy {:.1}%",
            hunger.0.normalized() * 100.0,
            thirst.0.normalized() * 100.0,
            energy.0.normalized() * 100.0
        );

        for action in actions.iter_mut() {
            match action.action_type {
                ActionType::Graze { .. } if hunger_critical => {
                    action.utility *= 2.0;
                    action.priority = EMERGENCY_SURVIVAL_PRIORITY;
                }
                ActionType::DrinkWater { .. } if thirst_critical => {
                    action.utility *= 2.0;
                    action.priority = EMERGENCY_SURVIVAL_PRIORITY;
                }
                ActionType::Rest { .. } if energy_critical => {
                    action.utility *= 1.5;
                    action.priority = EMERGENCY_SURVIVAL_PRIORITY;
                }
                ActionType::Mate { .. } => {
                    // Suppress mating during ANY emergency
                    action.utility *= 0.1;
                    action.priority = 0;
                }
                _ => {}
            }
        }
    }
}

/// Shared herbivore planning helper.
///
/// Species modules call this to evaluate actions, add mating/follow intents, and queue
/// the best result above the global utility threshold.
///
/// Includes failure memory to prevent infinite retry loops (Dwarf Fortress style).
pub fn plan_species_actions<M: Component>(
    commands: &mut Commands,
    queue: &mut ActionQueue,
    query: &Query<
        (
            Entity,
            &TilePosition,
            &Thirst,
            &Hunger,
            &Energy,
            &BehaviorConfig,
            Option<&Age>,
            Option<&Mother>,
            Option<&ActiveMate>,
            Option<&MatingTarget>,
            Option<&ReproductionConfig>,
            Option<&crate::entities::FearState>,
            Option<&crate::ai::event_driven_planner::NeedsReplanning>,
            Option<&crate::ai::failure_memory::ActionFailureMemory>,
        ),
        With<M>,
    >,
    positions: &Query<(Entity, &TilePosition), With<M>>,
    mut evaluate_actions: impl FnMut(
        Entity,
        &TilePosition,
        &Thirst,
        &Hunger,
        &Energy,
        &BehaviorConfig,
        Option<&crate::entities::FearState>,
    ) -> Vec<UtilityScore>,
    mate_params: Option<MateActionParams>,
    follow_cfg: Option<FollowConfig>,
    emoji: &str,
    label: &str,
    tick: u64,
) {
    let position_lookup: HashMap<Entity, IVec2> = positions
        .iter()
        .map(|(entity, pos)| (entity, pos.tile))
        .collect();

    // Emit planner diagnostics at most once every 10 ticks to avoid frame-time logging cost
    let should_log = tick % 10 == 0;

    for (
        entity,
        position,
        thirst,
        hunger,
        energy,
        behavior_config,
        age,
        mother,
        active_mate,
        mating_target,
        repro_cfg,
        fear_state,
        needs_replan,
        failure_memory,
    ) in query.iter()
    {
        let needs_replanning = needs_replan.is_some();

        // CRITICAL: Only plan if entity has been marked for replanning by UltraThink or triggers
        // This ensures UltraThink budget controls planning frequency
        if !needs_replanning {
            // Expected behavior: entities without NeedsReplanning are intentionally skipped
            // This is NOT a failure - it's budget-controlled planning
            continue;
        }

        let mut actions = evaluate_actions(
            entity,
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            fear_state,
        );

        if let Some(params) = mate_params {
            let mate_added = maybe_add_mate_action(
                &mut actions,
                active_mate,
                mating_target,
                repro_cfg,
                thirst,
                hunger,
                energy,
                params,
                tick,
            );
            let has_mating_relationship = active_mate.is_some() || mating_target.is_some();
            if has_mating_relationship && repro_cfg.is_some() && !mate_added {
                debug!(
                    "{}⏸️ {} {:?} delaying mating (thirst {:.2}, hunger {:.2}, energy {:.2})",
                    emoji,
                    label,
                    entity,
                    thirst.0.normalized(),
                    hunger.0.normalized(),
                    energy.0.normalized()
                );
            }
        }

        if let Some(cfg) = follow_cfg {
            let mother_position = mother.and_then(|m| position_lookup.get(&m.0).copied());
            let followed = maybe_add_follow_mother(
                &mut actions,
                entity,
                position,
                hunger,
                thirst,
                energy,
                behavior_config,
                age,
                mother,
                mother_position,
                cfg,
            );

            if mother.is_some() && !followed && mother_position.is_none() {
                warn!(
                    "{} {} {:?} - Mother entity no longer exists in position lookup, removing Mother component",
                    emoji, label, entity
                );
                commands.entity(entity).remove::<Mother>();
            }
        }

        if should_log && !actions.is_empty() {
            info!(
                "🧠{} {} {:?} at {:?} - Thirst: {:.1}% - Evaluated {} actions",
                emoji,
                label,
                entity,
                position.tile,
                thirst.0.percentage(),
                actions.len()
            );
            for action in &actions {
                info!(
                    "   - {:?} utility: {:.3}",
                    action.action_type, action.utility
                );
            }
        }

        let has_actions = !actions.is_empty();

        // Apply emergency priority overrides BEFORE selecting best action
        apply_emergency_priority_override(&mut actions, hunger, thirst, energy);

        // Apply failure cooldown penalties (Dwarf Fortress style: penalize recently failed actions)
        if let Some(memory) = failure_memory {
            crate::ai::failure_memory::apply_failure_penalties(&mut actions, memory, tick);
        }

        if let Some(best_action) = actions
            .into_iter()
            .filter(|a| a.utility >= UTILITY_THRESHOLD)
            .max_by(|a, b| {
                // Sort by priority first, THEN utility
                a.priority
                    .cmp(&b.priority)
                    .then(a.utility.partial_cmp(&b.utility).unwrap())
            })
        {
            if should_log {
                info!(
                    "✅{} {} {:?} queuing action {:?} with utility {:.2}",
                    emoji, label, entity, best_action.action_type, best_action.utility
                );
            }

            queue.queue_action(
                entity,
                best_action.action_type,
                best_action.utility,
                best_action.priority,
                tick,
            );
        } else if should_log && has_actions {
            warn!(
                "❌{} {} {:?} - No actions above threshold {:.2}",
                emoji, label, entity, UTILITY_THRESHOLD
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utility_threshold() {
        assert!(UTILITY_THRESHOLD > 0.0 && UTILITY_THRESHOLD < 1.0);
    }
}

#[cfg(test)]
mod emergency_priority_tests {
    use super::*;
    use crate::entities::stats::{Energy, Hunger, Stat, Thirst};

    #[test]
    fn test_survival_action_priority_boost_when_hungry() {
        // Setup: Create actions list with Graze (low priority) and Mate (high priority)
        let mut actions = vec![
            UtilityScore {
                action_type: ActionType::Graze {
                    target_tile: IVec2::ZERO,
                },
                utility: 0.6,
                priority: 100, // Normal grazing priority
            },
            UtilityScore {
                action_type: ActionType::Mate {
                    partner: Entity::PLACEHOLDER,
                    meeting_tile: IVec2::ZERO,
                    duration_ticks: 30,
                },
                utility: 0.7,
                priority: 350, // Mating priority
            },
        ];

        // Create critical hunger state (85%+)
        let hunger = Hunger(Stat::new(85.0, 0.0, 100.0, 0.0)); // 85% normalized
        let thirst = Thirst(Stat::new(0.0, 0.0, 100.0, 0.0)); // 0% normalized
        let energy = Energy(Stat::new(50.0, 0.0, 100.0, 0.0)); // 50% normalized

        // Apply emergency override function
        apply_emergency_priority_override(&mut actions, &hunger, &thirst, &energy);

        // Sort by priority first, then utility (like plan_species_actions does)
        actions.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(b.utility.partial_cmp(&a.utility).unwrap())
        });

        // Graze should now be first (priority 500 > 0)
        assert!(matches!(actions[0].action_type, ActionType::Graze { .. }));
        assert_eq!(actions[0].priority, 500);
        assert_eq!(actions[0].utility, 1.2); // 0.6 * 2.0

        // Mate should be suppressed
        assert!(matches!(actions[1].action_type, ActionType::Mate { .. }));
        assert_eq!(actions[1].priority, 0);
    }

    #[test]
    fn test_drink_water_priority_boost_when_thirsty() {
        let mut actions = vec![
            UtilityScore {
                action_type: ActionType::DrinkWater {
                    target_tile: IVec2::ZERO,
                },
                utility: 0.5,
                priority: 100,
            },
            UtilityScore {
                action_type: ActionType::Mate {
                    partner: Entity::PLACEHOLDER,
                    meeting_tile: IVec2::ZERO,
                    duration_ticks: 30,
                },
                utility: 0.8,
                priority: 350,
            },
        ];

        // Create critical thirst state (85%+)
        let hunger = Hunger(Stat::new(0.0, 0.0, 100.0, 0.0));
        let thirst = Thirst(Stat::new(85.0, 0.0, 100.0, 0.0)); // 85% normalized
        let energy = Energy(Stat::new(50.0, 0.0, 100.0, 0.0));

        apply_emergency_priority_override(&mut actions, &hunger, &thirst, &energy);

        actions.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(b.utility.partial_cmp(&a.utility).unwrap())
        });

        // DrinkWater should be first
        assert!(matches!(
            actions[0].action_type,
            ActionType::DrinkWater { .. }
        ));
        assert_eq!(actions[0].priority, 500);
        assert_eq!(actions[0].utility, 1.0); // 0.5 * 2.0
    }

    #[test]
    fn test_rest_priority_boost_when_energy_critical() {
        let mut actions = vec![
            UtilityScore {
                action_type: ActionType::Rest { duration_ticks: 20 },
                utility: 0.4,
                priority: 100,
            },
            UtilityScore {
                action_type: ActionType::Graze {
                    target_tile: IVec2::ZERO,
                },
                utility: 0.6,
                priority: 100,
            },
        ];

        // Create critical energy state (15% or less)
        let hunger = Hunger(Stat::new(0.0, 0.0, 100.0, 0.0));
        let thirst = Thirst(Stat::new(0.0, 0.0, 100.0, 0.0));
        let energy = Energy(Stat::new(15.0, 0.0, 100.0, 0.0)); // 15% normalized

        apply_emergency_priority_override(&mut actions, &hunger, &thirst, &energy);

        actions.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(b.utility.partial_cmp(&a.utility).unwrap())
        });

        // Rest should be first with boosted priority
        assert!(matches!(actions[0].action_type, ActionType::Rest { .. }));
        assert_eq!(actions[0].priority, 500);
        assert_eq!(actions[0].utility, 0.6); // 0.4 * 1.5
    }

    #[test]
    fn test_mating_suppressed_during_emergency() {
        let mut actions = vec![UtilityScore {
            action_type: ActionType::Mate {
                partner: Entity::PLACEHOLDER,
                meeting_tile: IVec2::ZERO,
                duration_ticks: 30,
            },
            utility: 0.9,
            priority: 350,
        }];

        // Create emergency hunger state
        let hunger = Hunger(Stat::new(85.0, 0.0, 100.0, 0.0)); // Critical
        let thirst = Thirst(Stat::new(0.0, 0.0, 100.0, 0.0));
        let energy = Energy(Stat::new(50.0, 0.0, 100.0, 0.0));

        apply_emergency_priority_override(&mut actions, &hunger, &thirst, &energy);

        // Mate action should be suppressed
        assert!((actions[0].utility - 0.09).abs() < 0.001); // 0.9 * 0.1 ≈ 0.09
        assert_eq!(actions[0].priority, 0);
    }

    #[test]
    fn test_no_boost_when_not_critical() {
        let mut actions = vec![
            UtilityScore {
                action_type: ActionType::Graze {
                    target_tile: IVec2::ZERO,
                },
                utility: 0.6,
                priority: 100,
            },
            UtilityScore {
                action_type: ActionType::Mate {
                    partner: Entity::PLACEHOLDER,
                    meeting_tile: IVec2::ZERO,
                    duration_ticks: 30,
                },
                utility: 0.7,
                priority: 350,
            },
        ];

        // Create non-critical states (below emergency thresholds)
        let hunger = Hunger(Stat::new(50.0, 0.0, 100.0, 0.0)); // 50% - not critical
        let thirst = Thirst(Stat::new(50.0, 0.0, 100.0, 0.0)); // 50% - not critical
        let energy = Energy(Stat::new(50.0, 0.0, 100.0, 0.0)); // 50% - not critical

        apply_emergency_priority_override(&mut actions, &hunger, &thirst, &energy);

        // Priorities should remain unchanged
        assert_eq!(actions[0].priority, 100);
        assert_eq!(actions[0].utility, 0.6);
        assert_eq!(actions[1].priority, 350);
        assert_eq!(actions[1].utility, 0.7);
    }
}
