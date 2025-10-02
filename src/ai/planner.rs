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
const UTILITY_THRESHOLD: f32 = 0.05; // Only queue actions above this utility (lowered to allow early water seeking)
const MAX_SEARCH_RADIUS: i32 = 100; // Max tiles to search for resources (wider range to prevent death from thirst)

/// System that plans actions for entities every frame
/// This runs async (not tick-synced) for responsiveness
pub fn plan_entity_actions(
    mut queue: ResMut<ActionQueue>,
    rabbit_query: Query<(Entity, &TilePosition, &Thirst), With<Rabbit>>,
    world_loader: Res<WorldLoader>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    // Plan for each rabbit
    for (entity, position, thirst) in rabbit_query.iter() {
        // Skip if entity already has an action queued/active
        if queue.has_action(entity) {
            continue;
        }
        
        // Evaluate all possible actions
        let actions = evaluate_rabbit_actions(entity, position, thirst, &world_loader);
        
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

/// Evaluate all possible actions for a rabbit
fn evaluate_rabbit_actions(
    _entity: Entity,
    position: &TilePosition,
    thirst: &Thirst,
    world_loader: &WorldLoader,
) -> Vec<UtilityScore> {
    let mut actions = Vec::new();
    
    // Action: Drink Water
    if let Some(drink_utility) = evaluate_drink_water_action(position, thirst, world_loader) {
        actions.push(drink_utility);
    }
    
    // Action: Wander (idle behavior)
    if let Some(wander_utility) = evaluate_wander_action(position, world_loader) {
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
    position: &TilePosition,
    thirst: &Thirst,
    world_loader: &WorldLoader,
) -> Option<UtilityScore> {
    // Only seek water when thirst is above 10% (prevents spam drinking)
    let thirst_level = thirst.0.normalized();
    if thirst_level < 0.1 {
        return None; // Not thirsty enough
    }
    
    // Find nearest water tile
    let water_tile = find_nearest_water(position.tile, MAX_SEARCH_RADIUS, world_loader)?;
    
    // Calculate utility based on thirst and distance
    let distance = position.tile.as_vec2().distance(water_tile.as_vec2());
    let distance_score = (1.0 - (distance / MAX_SEARCH_RADIUS as f32)).max(0.0);
    
    // Use weighted combination
    // Thirst weighted at 80%, distance at 20%
    let utility = (thirst_level * 0.8 + distance_score * 0.2);
    
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
/// This represents natural idle behavior - grazing, foraging, exploring nearby
fn evaluate_wander_action(
    position: &TilePosition,
    world_loader: &WorldLoader,
) -> Option<UtilityScore> {
    use rand::Rng;
    
    // Pick a random nearby tile for grazing/foraging (short range: 3-8 tiles)
    // Rabbits don't wander far - they graze in a small area
    let mut rng = rand::thread_rng();
    let graze_distance = rng.gen_range(3..=8);
    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    
    let offset = IVec2::new(
        (angle.cos() * graze_distance as f32) as i32,
        (angle.sin() * graze_distance as f32) as i32,
    );
    
    let target = position.tile + offset;
    
    // Verify it's walkable
    if let Some(terrain_str) = world_loader.get_terrain_at(target.x, target.y) {
        if let Some(terrain) = TerrainType::from_str(&terrain_str) {
            if !terrain.is_walkable() {
                // Try to find a nearby walkable tile
                let adjusted_target = find_nearest_walkable(target, 5, world_loader)?;
                
                // Grazing/foraging has moderate utility - it's natural idle behavior
                // Priority is low but not trivial - rabbits should graze when not doing important things
                return Some(UtilityScore {
                    action_type: ActionType::Wander {
                        target_tile: adjusted_target,
                    },
                    utility: 0.15, // Moderate - natural idle behavior
                    priority: 10,  // Low priority but beats doing nothing
                });
            }
        }
    }
    
    // Grazing/foraging is natural idle behavior
    // Rabbits graze when they have nothing urgent to do
    // This creates realistic "eating grass around" behavior
    Some(UtilityScore {
        action_type: ActionType::Wander {
            target_tile: target,
        },
        utility: 0.15, // Moderate - natural idle behavior like grazing
        priority: 10,  // Low priority - urgent needs override this
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
/// Returns a walkable tile ADJACENT to water, not the water tile itself
fn find_nearest_water(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
) -> Option<IVec2> {
    let mut nearest: Option<(IVec2, IVec2, f32)> = None; // (water_pos, adjacent_pos, distance)
    
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
                            // Found water! Now find an adjacent walkable tile
                            if let Some(adjacent_tile) = find_adjacent_walkable_to_water(check_pos, world_loader) {
                                let distance = from.as_vec2().distance(adjacent_tile.as_vec2());
                                
                                if let Some((_, _, best_dist)) = nearest {
                                    if distance < best_dist {
                                        nearest = Some((check_pos, adjacent_tile, distance));
                                    }
                                } else {
                                    nearest = Some((check_pos, adjacent_tile, distance));
                                }
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
    
    // Return the water tile position (action will handle adjacency)
    nearest.map(|(water_pos, _, _)| water_pos)
}

/// Find a walkable tile adjacent to a water tile
fn find_adjacent_walkable_to_water(
    water_pos: IVec2,
    world_loader: &WorldLoader,
) -> Option<IVec2> {
    // Check all 8 adjacent tiles (including diagonals)
    let adjacent_offsets = [
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, 1),
        IVec2::new(-1, -1),
    ];
    
    for offset in adjacent_offsets {
        let check_pos = water_pos + offset;
        
        if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
            if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                // Must be walkable but NOT water
                if terrain.is_walkable() && !matches!(terrain, TerrainType::ShallowWater | TerrainType::DeepWater | TerrainType::Water) {
                    return Some(check_pos);
                }
            }
        }
    }
    
    None
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
