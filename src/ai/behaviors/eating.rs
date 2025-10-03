use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::{stats::Hunger, TilePosition};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
/// Eating Behavior - for herbivores that consume grass
///
/// This behavior makes entities find and eat grass when hungry.
/// Suitable for: Rabbits, Deer, Sheep, Horses, etc.
use bevy::prelude::*;

/// Evaluate the utility of eating grass
///
/// Returns an eating action if hunger is above threshold and grass is found nearby.
///
/// # Parameters
/// - `position`: Current position of the entity
/// - `hunger`: Current hunger level
/// - `world_loader`: Access to terrain data
/// - `hunger_threshold`: Minimum hunger level to seek food (0.0-1.0)
/// - `search_radius`: Maximum tiles to search for grass
///
/// # Returns
/// - `Some(UtilityScore)` if hungry enough and grass is found
/// - `None` if not hungry or no grass available
pub fn evaluate_eating_behavior(
    position: &TilePosition,
    hunger: &Hunger,
    world_loader: &WorldLoader,
    hunger_threshold: f32,
    search_radius: i32,
) -> Option<UtilityScore> {
    // Only seek food when hunger is above threshold
    let hunger_level = hunger.0.normalized();
    if hunger_level < hunger_threshold {
        return None; // Not hungry enough
    }

    // Note: We don't check if already on grass because eating is handled by auto-eat system

    // Find nearest grass tile
    let grass_tile = find_nearest_grass(position.tile, search_radius, world_loader)?;

    // Calculate utility based on hunger and distance
    let distance = position.tile.as_vec2().distance(grass_tile.as_vec2());
    let distance_score = (1.0 - (distance / search_radius as f32)).max(0.0);

    // Weighted combination: hunger 80%, distance 20%
    let utility = hunger_level * 0.8 + distance_score * 0.2;

    // Calculate priority based on urgency
    let priority = if hunger_level > 0.7 {
        900 // Critical - very hungry
    } else if hunger_level > 0.4 {
        400 // Important - moderately hungry
    } else {
        100 // Low priority - slightly hungry
    };

    // Return Graze action to move to grass (auto-eat system will handle eating)
    Some(UtilityScore {
        action_type: ActionType::Graze {
            target_tile: grass_tile,
        },
        utility,
        priority,
    })
}

/// Find the nearest grass tile from a position
fn find_nearest_grass(from: IVec2, max_radius: i32, world_loader: &WorldLoader) -> Option<IVec2> {
    let mut nearest: Option<(IVec2, f32)> = None; // (position, distance)

    // Search in expanding square pattern
    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Only check perimeter (not interior)
                if dx.abs() < radius && dy.abs() < radius {
                    continue;
                }

                let check_pos = from + IVec2::new(dx, dy);

                // Check if this tile is grass
                if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
                    if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                        if matches!(terrain, TerrainType::Grass) {
                            // Check if tile is accessible (no blocking resources)
                            let has_resource = world_loader
                                .get_resource_at(check_pos.x, check_pos.y)
                                .map(|r| !r.is_empty())
                                .unwrap_or(false);

                            if !has_resource {
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
        }

        // Early exit if we found grass at this radius
        if nearest.is_some() {
            break;
        }
    }

    nearest.map(|(pos, _)| pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here - checking grass-finding logic
}
