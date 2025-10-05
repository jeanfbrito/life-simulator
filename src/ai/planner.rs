use super::action::ActionType;
use super::queue::ActionQueue;
use crate::ai::herbivore_toolkit::{
    maybe_add_follow_mother, maybe_add_mate_action, FollowConfig, MateActionParams,
};
use crate::entities::reproduction::{Age, MatingIntent, Mother, ReproductionConfig};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{BehaviorConfig, TilePosition};
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

/// Shared herbivore planning helper.
///
/// Species modules call this to evaluate actions, add mating/follow intents, and queue
/// the best result above the global utility threshold.
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
            Option<&MatingIntent>,
            Option<&ReproductionConfig>,
            Option<&crate::entities::FearState>,
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

    for (
        entity,
        position,
        thirst,
        hunger,
        energy,
        behavior_config,
        age,
        mother,
        mating_intent,
        repro_cfg,
        fear_state,
    ) in query.iter()
    {
        if queue.has_action(entity) {
            continue;
        }

        let mut actions =
            evaluate_actions(entity, position, thirst, hunger, energy, behavior_config, fear_state);

        if let Some(params) = mate_params {
            let mate_added = maybe_add_mate_action(
                &mut actions,
                mating_intent,
                repro_cfg,
                thirst,
                hunger,
                energy,
                params,
            );
            if mating_intent.is_some() && repro_cfg.is_some() && !mate_added {
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
                commands.entity(entity).remove::<Mother>();
            }
        }

        if !actions.is_empty() {
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

        if let Some(best_action) = actions
            .into_iter()
            .filter(|a| a.utility >= UTILITY_THRESHOLD)
            .max_by(|a, b| a.utility.partial_cmp(&b.utility).unwrap())
        {
            info!(
                "✅{} {} {:?} queuing action {:?} with utility {:.2}",
                emoji, label, entity, best_action.action_type, best_action.utility
            );

            queue.queue_action(
                entity,
                best_action.action_type,
                best_action.utility,
                best_action.priority,
                tick,
            );
        } else if has_actions {
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
