use super::action::ActionType;
use super::behaviors::{
    evaluate_drinking_behavior, evaluate_eating_behavior, evaluate_follow_behavior,
    evaluate_grazing_behavior, evaluate_resting_behavior,
};
use super::consideration::{
    CombinationMethod, ConsiderationSet, DistanceConsideration, ThirstConsideration,
};
use super::queue::ActionQueue;
use crate::entities::{
    reproduction::{MatingIntent, ReproductionConfig},
    stats::{Energy, Hunger, Thirst},
    BehaviorConfig, Deer, Rabbit, TilePosition,
};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
/// Utility Planner for TQUAI
///
/// Evaluates entity needs and available actions asynchronously (every frame),
/// queues high-utility actions for execution on ticks.
use bevy::prelude::*;

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
    use std::collections::HashMap;
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

        // If there is a mating intent, add a Mate action to rendezvous
        if let (Some(intent), Some(cfg)) = (mating_intent, repro_cfg) {
            let thirst_level = thirst.0.normalized();
            let hunger_level = hunger.0.normalized();
            let energy_level = energy.0.normalized();

            let thirst_safe = thirst_level <= cfg.well_fed_thirst_norm + 0.05;
            let hunger_safe = hunger_level <= cfg.well_fed_hunger_norm + 0.05;
            let energy_safe = energy_level >= (cfg.min_energy_norm + 0.05).min(1.0);

            if thirst_safe && hunger_safe && energy_safe {
                actions.push(UtilityScore {
                    action_type: ActionType::Mate {
                        partner: intent.partner,
                        meeting_tile: intent.meeting_tile,
                        duration_ticks: intent.duration_ticks,
                    },
                    utility: 0.45, // Below survival needs
                    priority: 350, // Lower than critical hunger/thirst, above idle
                });
            } else {
                debug!(
                    "⏸️ Entity {:?} delaying mating due to needs (thirst {:.2}, hunger {:.2}, energy {:.2})",
                    entity,
                    thirst_level,
                    hunger_level,
                    energy_level
                );
            }
        }

        // Juvenile follow-mother behavior (when not doing necessities)
        if let (Some(age), Some(mother)) = (age, mother) {
            if !age.is_adult() {
                // Only follow when needs are not urgent
                let hunger_ok = hunger.0.normalized() < behavior_config.hunger_threshold;
                let thirst_ok = thirst.0.normalized() < behavior_config.thirst_threshold;
                let energy_ok = energy.0.normalized() > behavior_config.energy_threshold;
                if hunger_ok && thirst_ok && energy_ok {
                    if let Some(&mpos) = rabbit_pos_map.get(&mother.0.index()) {
                        let rabbits_slice = [(mother.0, mpos)];
                        if let Some(follow) = crate::ai::behaviors::evaluate_follow_behavior(
                            entity,
                            position,
                            &rabbits_slice,
                            2,  // stop_distance: stay within 2 tiles of mother
                            20, // max_follow_distance for utility scaling
                        ) {
                            actions.push(follow);
                        }
                    } else {
                        // Mother missing (likely dead or despawned). Remove Mother component to avoid stale refs.
                        commands
                            .entity(entity)
                            .remove::<crate::entities::reproduction::Mother>();
                    }
                }
            }
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

        if let (Some(intent), Some(cfg)) = (mating_intent, repro_cfg) {
            let thirst_level = thirst.0.normalized();
            let hunger_level = hunger.0.normalized();
            let energy_level = energy.0.normalized();

            let thirst_safe = thirst_level <= cfg.well_fed_thirst_norm + 0.05;
            let hunger_safe = hunger_level <= cfg.well_fed_hunger_norm + 0.05;
            let energy_safe = energy_level >= (cfg.min_energy_norm + 0.05).min(1.0);

            if thirst_safe && hunger_safe && energy_safe {
                actions.push(UtilityScore {
                    action_type: ActionType::Mate {
                        partner: intent.partner,
                        meeting_tile: intent.meeting_tile,
                        duration_ticks: intent.duration_ticks,
                    },
                    utility: 0.45,
                    priority: 350,
                });
            } else {
                debug!(
                    "🦌⏸️ Deer {:?} delaying mating (thirst {:.2}, hunger {:.2}, energy {:.2})",
                    entity, thirst_level, hunger_level, energy_level
                );
            }
        }

        if let (Some(age), Some(mother)) = (age, mother) {
            if !age.is_adult() {
                let hunger_ok = hunger.0.normalized() < behavior_config.hunger_threshold;
                let thirst_ok = thirst.0.normalized() < behavior_config.thirst_threshold;
                let energy_ok = energy.0.normalized() > behavior_config.energy_threshold;

                if hunger_ok && thirst_ok && energy_ok {
                    if let Some(&mpos) = deer_pos_map.get(&mother.0.index()) {
                        let deer_slice = [(mother.0, mpos)];
                        if let Some(follow) =
                            evaluate_follow_behavior(entity, position, &deer_slice, 2, 25)
                        {
                            actions.push(follow);
                        }
                    } else {
                        commands
                            .entity(entity)
                            .remove::<crate::entities::reproduction::Mother>();
                    }
                }
            }
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

/// Evaluate all possible actions for an entity using its behavior configuration
fn evaluate_entity_actions(
    _entity: Entity,
    position: &TilePosition,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    world_loader: &WorldLoader,
) -> Vec<UtilityScore> {
    let mut actions = Vec::new();

    // Behavior: Drinking (when thirsty)
    // Use entity's configured thirst threshold and search radius
    if let Some(drink_utility) = evaluate_drinking_behavior(
        position,
        thirst,
        world_loader,
        behavior_config.thirst_threshold,
        behavior_config.water_search_radius,
    ) {
        actions.push(drink_utility);
    }

    // Behavior: Eating (when hungry)
    // Use entity's configured hunger threshold and search radius
    if let Some(eat_utility) = evaluate_eating_behavior(
        position,
        hunger,
        world_loader,
        behavior_config.hunger_threshold,
        behavior_config.food_search_radius,
    ) {
        actions.push(eat_utility);
    }

    // Behavior: Resting (when tired)
    // Use entity's configured energy threshold
    if let Some(rest_utility) =
        evaluate_resting_behavior(position, energy, behavior_config.energy_threshold)
    {
        actions.push(rest_utility);
    }

    // Behavior: Grazing (idle herbivore behavior)
    // Use entity's configured graze range
    if let Some(graze_utility) =
        evaluate_grazing_behavior(position, world_loader, behavior_config.graze_range)
    {
        actions.push(graze_utility);
    }

    // Future behaviors:
    // - Flee from predators (when wolf nearby)
    // - Socialize with other rabbits

    actions
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
