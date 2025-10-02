/// Drinking Behavior - for entities that need water
/// 
/// This behavior makes entities seek out and move to water sources when thirsty.
/// Suitable for: All animals (Rabbits, Deer, Wolves, etc.)

use bevy::prelude::*;
use crate::entities::{TilePosition, stats::Thirst};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;

/// Evaluate the utility of drinking water
/// 
/// Returns a drinking action if thirst is above threshold and water is found nearby.
/// 
/// # Parameters
/// - `position`: Current position of the entity
/// - `thirst`: Current thirst level
/// - `world_loader`: Access to terrain data
/// - `thirst_threshold`: Minimum thirst level to seek water (0.0-1.0, recommended 0.1)
/// - `search_radius`: Maximum tiles to search for water sources
/// 
/// # Returns
/// - `Some(UtilityScore)` if thirsty enough and water is found
/// - `None` if not thirsty or no water available
pub fn evaluate_drinking_behavior(
    position: &TilePosition,
    thirst: &Thirst,
    world_loader: &WorldLoader,
    thirst_threshold: f32,
    search_radius: i32,
) -> Option<UtilityScore> {
    // Only seek water when thirst is above threshold (prevents spam drinking)
    let thirst_level = thirst.0.normalized();
    if thirst_level < thirst_threshold {
        return None; // Not thirsty enough
    }
    
    // Find nearest water tile
    let water_tile = find_nearest_water(position.tile, search_radius, world_loader)?;
    
    // Calculate utility based on thirst and distance
    let distance = position.tile.as_vec2().distance(water_tile.as_vec2());
    let distance_score = (1.0 - (distance / search_radius as f32)).max(0.0);
    
    // Use weighted combination
    // Thirst weighted at 80%, distance at 20%
    let utility = (thirst_level * 0.8 + distance_score * 0.2);
    
    // Calculate priority based on urgency
    // Higher thirst = higher priority
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
    
    // Return the water tile itself (not the adjacent tile)
    // The action handler will stop at an adjacent tile
    nearest.map(|(water_pos, _, _)| water_pos)
}

/// Find a walkable tile adjacent to water
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
    
    // Tests would go here - checking water-finding logic
}
