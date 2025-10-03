use super::action::ActionType;
use super::queue::ActionQueue;
use crate::ai::herbivore_toolkit::{
    maybe_add_follow_mother, maybe_add_mate_action, FollowConfig, MateActionParams,
};
use crate::entities::{
    reproduction::{Age, MatingIntent, Mother, ReproductionConfig},
    stats::{Energy, Hunger, Thirst},
    BehaviorConfig, Deer, Rabbit, TilePosition,
};
use crate::world_loader::WorldLoader;
/// Utility Planner for TQUAI
///
/// Evaluates entity needs and available actions asynchronously (every frame),
/// queues high-utility actions for execution on ticks.
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
const UTILITY_THRESHOLD: f32 = 0.05; // Only queue actions above this utility (lowered to allow early water seeking)

/// System that plans actions for entities every frame
/// This runs async (not tick-synced) for responsiveness
pub fn plan_entity_actions(
    mut commands: Commands,
    mut queue: ResMut<ActionQueue>,
    rabbit_query: Query<
        (
            Entity,
            &TilePosition,
            &Thirst,
            &Hunger,
            &Energy,
            &BehaviorConfig,
            Option<&crate::entities::reproduction::Age>,
            Option<&crate::entities::reproduction::Mother>,
            Option<&crate::entities::reproduction::MatingIntent>,
            Option<&ReproductionConfig>,
        ),
        With<Rabbit>,
    >,
    deer_query: Query<
        (
            Entity,
            &TilePosition,
            &Thirst,
            &Hunger,
            &Energy,
            &BehaviorConfig,
            Option<&crate::entities::reproduction::Age>,
            Option<&crate::entities::reproduction::Mother>,
            Option<&MatingIntent>,
            Option<&ReproductionConfig>,
        ),
        With<Deer>,
    >,
    deer_positions: Query<(Entity, &TilePosition), With<Deer>>,
    rabbit_positions: Query<(Entity, &TilePosition), With<Rabbit>>,
    world_loader: Res<WorldLoader>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    // Plan for each rabbit
    // Build a quick lookup for rabbit positions (used for mother lookups)
    let mut rabbit_pos_map: HashMap<u32, IVec2> = HashMap::new();
    for (e, pos) in rabbit_positions.iter() {
        rabbit_pos_map.insert(e.index(), pos.tile);
    }

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
    ) in rabbit_query.iter()
    {
        if queue.has_action(entity) {
            continue;
        }

        // Delegate rabbit behavior evaluation to the rabbit module
        let mut actions = crate::entities::types::rabbit::RabbitBehavior::evaluate_actions(
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            &world_loader,
        );

        let mate_added = maybe_add_mate_action(
            &mut actions,
            mating_intent,
            repro_cfg,
            thirst,
            hunger,
            energy,
            MateActionParams {
                utility: 0.45,
                priority: 350,
                threshold_margin: 0.05,
                energy_margin: 0.05,
            },
        );
        if mating_intent.is_some() && repro_cfg.is_some() && !mate_added {
            debug!(
                "⏸️ Entity {:?} delaying mating due to needs (thirst {:.2}, hunger {:.2}, energy {:.2})",
                entity,
                thirst.0.normalized(),
                hunger.0.normalized(),
                energy.0.normalized()
            );
        }

        let mother_position = mother.and_then(|m| rabbit_pos_map.get(&m.0.index()).copied());
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
            FollowConfig {
                stop_distance: 2,
                max_distance: 20,
            },
        );

        if mother.is_some() && !followed && mother_position.is_none() {
            commands
                .entity(entity)
                .remove::<crate::entities::reproduction::Mother>();
        }

        if !actions.is_empty() {
            info!(
                "🧠 Entity {:?} at {:?} - Thirst: {:.1}% - Evaluated {} actions",
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

        // Queue the best action if it's above threshold
        if let Some(best_action) = actions
            .into_iter()
            .filter(|a| a.utility >= UTILITY_THRESHOLD)
            .max_by(|a, b| a.utility.partial_cmp(&b.utility).unwrap())
        {
            info!(
                "✅ Entity {:?} queuing action {:?} with utility {:.2}",
                entity, best_action.action_type, best_action.utility
            );

            queue.queue_action(
                entity,
                best_action.action_type,
                best_action.utility,
                best_action.priority,
                tick.0,
            );
        } else if has_actions {
            warn!(
                "❌ Entity {:?} - No actions above threshold {:.2}",
                entity, UTILITY_THRESHOLD
            );
        }
    }

    // Plan for each deer
    let rabbit_list: Vec<(Entity, IVec2)> = rabbit_positions
        .iter()
        .map(|(e, pos)| (e, pos.tile))
        .collect();

    let mut deer_pos_map: HashMap<u32, IVec2> = HashMap::new();
    for (e, pos) in deer_positions.iter() {
        deer_pos_map.insert(e.index(), pos.tile);
    }

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
    ) in deer_query.iter()
    {
        if queue.has_action(entity) {
            continue;
        }

        // Delegate deer behavior evaluation to the deer module
        let mut actions = crate::entities::types::deer::DeerBehavior::evaluate_actions(
            entity,
            position,
            thirst,
            hunger,
            energy,
            behavior_config,
            &world_loader,
            &rabbit_list,
        );

        let mate_added = maybe_add_mate_action(
            &mut actions,
            mating_intent,
            repro_cfg,
            thirst,
            hunger,
            energy,
            MateActionParams {
                utility: 0.45,
                priority: 350,
                threshold_margin: 0.05,
                energy_margin: 0.05,
            },
        );
        if mating_intent.is_some() && repro_cfg.is_some() && !mate_added {
            debug!(
                "🦌⏸️ Deer {:?} delaying mating (thirst {:.2}, hunger {:.2}, energy {:.2})",
                entity,
                thirst.0.normalized(),
                hunger.0.normalized(),
                energy.0.normalized()
            );
        }

        let mother_position = mother.and_then(|m| deer_pos_map.get(&m.0.index()).copied());
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
            FollowConfig {
                stop_distance: 2,
                max_distance: 25,
            },
        );
        if mother.is_some() && !followed && mother_position.is_none() {
            commands
                .entity(entity)
                .remove::<crate::entities::reproduction::Mother>();
        }

        if !actions.is_empty() {
            info!(
                "🧠 Deer {:?} at {:?} - Thirst: {:.1}% - Evaluated {} actions",
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
                "✅ Deer {:?} queuing action {:?} with utility {:.2}",
                entity, best_action.action_type, best_action.utility
            );

            queue.queue_action(
                entity,
                best_action.action_type,
                best_action.utility,
                best_action.priority,
                tick.0,
            );
        } else if has_actions {
            warn!(
                "❌ Deer {:?} - No actions above threshold {:.2}",
                entity, UTILITY_THRESHOLD
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utility_threshold() {
        // Utility threshold should filter out low-value actions
        assert!(UTILITY_THRESHOLD > 0.0 && UTILITY_THRESHOLD < 1.0);
    }
}
