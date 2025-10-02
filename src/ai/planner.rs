/// Utility Planner for TQUAI
/// 
/// Evaluates entity needs and available actions asynchronously (every frame),
/// queues high-utility actions for execution on ticks.

use bevy::prelude::*;
use crate::entities::{Rabbit, TilePosition, stats::Thirst};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use super::action::ActionType;
use super::queue::ActionQueue;
use super::consideration::{ThirstConsideration, DistanceConsideration, ConsiderationSet, CombinationMethod};

/// Utility score with associated action
#[derive(Debug, Clone)]
pub struct UtilityScore {
    pub action_type: ActionType,
    pub utility: f32,
    pub priority: i32,
}

/// Planner configuration
const UTILITY_THRESHOLD: f32 = 0.3; // Only queue actions above this utility
const MAX_SEARCH_RADIUS: i32 = 30; // Max tiles to search for resources

/// System that plans actions for entities every frame
/// This runs async (not tick-synced) for responsiveness
pub fn plan_entity_actions(
    mut queue: ResMut<ActionQueue>,
    rabbit_query: Query<(Entity, &TilePosition, &Thirst), With<Rabbit>>,
    world_loader: Res<WorldLoader>,
    tick: Res<crate::simulation::SimulationTick>,
    world: &World,
) {
    // Plan for each rabbit
    for (entity, position, thirst) in rabbit_query.iter() {
        // Skip if entity already has an action queued/active
        if queue.has_action(entity) {
            continue;
        }
        
        // Evaluate all possible actions
        let actions = evaluate_rabbit_actions(entity, position, thirst, &world_loader, world);
        
        // Queue the best action if it's above threshold
        if let Some(best_action) = actions.into_iter()
            .filter(|a| a.utility >= UTILITY_THRESHOLD)
            .max_by(|a, b| a.utility.partial_cmp(&b.utility).unwrap())
        {
            debug!(
                "🧠 Entity {:?} planning action {:?} with utility {:.2}",
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
        }
    }
}

/// Evaluate all possible actions for a rabbit
fn evaluate_rabbit_actions(
    entity: Entity,
    position: &TilePosition,
    thirst: &Thirst,
    world_loader: &WorldLoader,
    world: &World,
) -> Vec<UtilityScore> {
    let mut actions = Vec::new();
    
    // Action: Drink Water
    if let Some(drink_utility) = evaluate_drink_water_action(entity, position, thirst, world_loader, world) {
        actions.push(drink_utility);
    }
    
    // Action: Wander (idle behavior)
    if let Some(wander_utility) = evaluate_wander_action(entity, position, world_loader, world) {
        actions.push(wander_utility);
    }
    
    // Future actions:
    // - Eat food
    // - Flee from predators
    // - Rest when tired
    // - Socialize with other rabbits
    
    actions
}

/// Evaluate the utility of drinking water
fn evaluate_drink_water_action(
    entity: Entity,
    position: &TilePosition,
    thirst: &Thirst,
    world_loader: &WorldLoader,
    world: &World,
) -> Option<UtilityScore> {
    // Find nearest water tile
    let water_tile = find_nearest_water(position.tile, MAX_SEARCH_RADIUS, world_loader)?;
    
    // Build consideration set for this action
    let mut considerations = ConsiderationSet::new(CombinationMethod::Multiply);
    
    // Consideration 1: How thirsty are we?
    considerations.add(ThirstConsideration::new());
    
    // Consideration 2: How far is the water?
    considerations.add(DistanceConsideration::new(water_tile, MAX_SEARCH_RADIUS as f32));
    
    // Evaluate combined utility
    let utility = considerations.evaluate(world, entity);
    
    // Calculate priority based on urgency
    // Higher thirst = higher priority
    let thirst_level = thirst.0.normalized();
    let priority = if thirst_level > 0.7 {
        1000 // Critical - very thirsty
    } else if thirst_level > 0.4 {
        500  // Important - moderately thirsty
    } else {
        100  // Low priority - slightly thirsty
    };
    
    Some(UtilityScore {
        action_type: ActionType::DrinkWater {
            target_tile: water_tile,
        },
        utility,
        priority,
    })
}

/// Evaluate the utility of wandering to a random location
fn evaluate_wander_action(
    entity: Entity,
    position: &TilePosition,
    world_loader: &WorldLoader,
    world: &World,
) -> Option<UtilityScore> {
    use rand::Rng;
    
    // Pick a random nearby tile (within 5-15 tiles)
    let mut rng = rand::thread_rng();
    let wander_distance = rng.gen_range(5..=15);
    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    
    let offset = IVec2::new(
        (angle.cos() * wander_distance as f32) as i32,
        (angle.sin() * wander_distance as f32) as i32,
    );
    
    let target = position.tile + offset;
    
    // Verify it's walkable
    if let Some(terrain_str) = world_loader.get_terrain_at(target.x, target.y) {
        if let Some(terrain) = TerrainType::from_str(&terrain_str) {
            if !terrain.is_walkable() {
                // Try to find a nearby walkable tile
                let adjusted_target = find_nearest_walkable(target, 5, world_loader)?;
                
                // Wander has constant low utility (idle behavior)
                // Priority is very low - any other action should override it
                return Some(UtilityScore {
                    action_type: ActionType::Wander {
                        target_tile: adjusted_target,
                    },
                    utility: 0.2, // Low constant utility
                    priority: 10,  // Very low priority
                });
            }
        }
    }
    
    // Wander has constant low utility (idle behavior)
    Some(UtilityScore {
        action_type: ActionType::Wander {
            target_tile: target,
        },
        utility: 0.2, // Low constant utility
        priority: 10,  // Very low priority
    })
}

/// Find the nearest walkable tile from a position
fn find_nearest_walkable(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
) -> Option<IVec2> {
    // Search in expanding square pattern
    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Only check perimeter
                if dx.abs() < radius && dy.abs() < radius {
                    continue;
                }
                
                let check_pos = from + IVec2::new(dx, dy);
                
                if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
                    if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                        if terrain.is_walkable() {
                            return Some(check_pos);
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Find the nearest water tile (ShallowWater) from a position
fn find_nearest_water(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
) -> Option<IVec2> {
    let mut nearest: Option<(IVec2, f32)> = None;
    
    // Search in expanding square pattern
    for radius in 1..=max_radius {
        // Check tiles at this radius
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Only check perimeter (not interior)
                if dx.abs() < radius && dy.abs() < radius {
                    continue;
                }
                
                let check_pos = from + IVec2::new(dx, dy);
                
                // Check if this tile is shallow water
                if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
                    if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                        if matches!(terrain, TerrainType::ShallowWater) {
                            let distance = from.as_vec2().distance(check_pos.as_vec2());
                            
                            if let Some((_, best_dist)) = nearest {
                                if distance < best_dist {
                                    nearest = Some((check_pos, distance));
                                }
                            } else {
                                nearest = Some((check_pos, distance));
                            }
                        }
                    }
                }
            }
        }
        
        // Early exit if we found water at this radius
        if nearest.is_some() {
            break;
        }
    }
    
    nearest.map(|(pos, _)| pos)
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
