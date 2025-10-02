/// Grazing Behavior - for herbivores that eat grass
/// 
/// This behavior makes entities move to nearby grass tiles to graze/forage.
/// Suitable for: Rabbits, Deer, Sheep, Horses, etc.

use bevy::prelude::*;
use crate::entities::TilePosition;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;

/// Evaluate the utility of grazing on grass
/// 
/// Returns a grazing action (Wander to grass tile) if grass is found nearby.
/// This represents natural idle behavior for herbivores.
/// 
/// # Parameters
/// - `position`: Current position of the entity
/// - `world_loader`: Access to terrain data
/// - `graze_distance`: Range in tiles to search for grass (3-8 recommended for rabbits, 5-15 for deer)
/// 
/// # Returns
/// - `Some(UtilityScore)` if grass is found within range
/// - `None` if no grass is available
pub fn evaluate_grazing_behavior(
    position: &TilePosition,
    world_loader: &WorldLoader,
    graze_distance: (i32, i32), // (min, max) range
) -> Option<UtilityScore> {
    use rand::Rng;
    
    // Pick a random nearby tile for grazing/foraging
    // Herbivores don't wander far - they graze in a small area
    let mut rng = rand::thread_rng();
    let distance = rng.gen_range(graze_distance.0..=graze_distance.1);
    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    
    let offset = IVec2::new(
        (angle.cos() * distance as f32) as i32,
        (angle.sin() * distance as f32) as i32,
    );
    
    let target = position.tile + offset;
    
    // Verify it's grass (herbivores only graze on grass!)
    if let Some(terrain_str) = world_loader.get_terrain_at(target.x, target.y) {
        if let Some(terrain) = TerrainType::from_str(&terrain_str) {
            if matches!(terrain, TerrainType::Grass) {
                // Perfect! Found grass to graze on
                return Some(UtilityScore {
                    action_type: ActionType::Graze {
                        target_tile: target,
                    },
                    utility: 0.15, // Moderate - natural idle behavior like grazing
                    priority: 10,  // Low priority - urgent needs override this
                });
            } else if terrain.is_walkable() {
                // Not grass, but walkable - try to find nearby grass
                if let Some(grass_tile) = find_nearest_grass(target, 5, world_loader) {
                    return Some(UtilityScore {
                        action_type: ActionType::Graze {
                            target_tile: grass_tile,
                        },
                        utility: 0.15, // Moderate - natural idle behavior
                        priority: 10,  // Low priority but beats doing nothing
                    });
                }
            }
        }
    }
    
    // Couldn't find grass to graze on - no grazing action
    None
}

/// Find the nearest grass tile from a position (for grazing)
fn find_nearest_grass(
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
                        if matches!(terrain, TerrainType::Grass) {
                            return Some(check_pos);
                        }
                    }
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests would go here - checking that grazing only happens on grass tiles
}
