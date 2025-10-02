/// Utility Planner for TQUAI
/// 
/// Evaluates entity needs and available actions asynchronously (every frame),
/// queues high-utility actions for execution on ticks.

use bevy::prelude::*;
use crate::entities::{Rabbit, TilePosition, stats::{Thirst, Hunger, Energy}, BehaviorConfig};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use super::action::ActionType;
use super::queue::ActionQueue;
use super::consideration::{ThirstConsideration, DistanceConsideration, ConsiderationSet, CombinationMethod};
use super::behaviors::{evaluate_grazing_behavior, evaluate_drinking_behavior, evaluate_eating_behavior, evaluate_resting_behavior};

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
    mut queue: ResMut<ActionQueue>,
    rabbit_query: Query<(Entity, &TilePosition, &Thirst, &Hunger, &Energy, &BehaviorConfig), With<Rabbit>>,
    world_loader: Res<WorldLoader>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    // Plan for each rabbit
    for (entity, position, thirst, hunger, energy, behavior_config) in rabbit_query.iter() {
        // Skip if entity already has an action queued/active
        if queue.has_action(entity) {
            continue;
        }
        
        // Evaluate all possible actions
        let actions = evaluate_entity_actions(entity, position, thirst, hunger, energy, behavior_config, &world_loader);
        
        // Debug: Log all evaluated actions
        if !actions.is_empty() {
            info!(
                "🧠 Entity {:?} at {:?} - Thirst: {:.1}% - Evaluated {} actions",
                entity,
                position.tile,
                thirst.0.percentage(),
                actions.len()
            );
            for action in &actions {
                info!("   - {:?} utility: {:.3}", action.action_type, action.utility);
            }
        }
        
        let has_actions = !actions.is_empty();
        
        // Queue the best action if it's above threshold
        if let Some(best_action) = actions.into_iter()
            .filter(|a| a.utility >= UTILITY_THRESHOLD)
            .max_by(|a, b| a.utility.partial_cmp(&b.utility).unwrap())
        {
            info!(
                "✅ Entity {:?} queuing action {:?} with utility {:.2}",
                entity,
                best_action.action_type,
                best_action.utility
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
                entity,
                UTILITY_THRESHOLD
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
    if let Some(rest_utility) = evaluate_resting_behavior(
        position,
        energy,
        behavior_config.energy_threshold,
    ) {
        actions.push(rest_utility);
    }
    
    // Behavior: Grazing (idle herbivore behavior)
    // Use entity's configured graze range
    if let Some(graze_utility) = evaluate_grazing_behavior(
        position,
        world_loader,
        behavior_config.graze_range,
    ) {
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
