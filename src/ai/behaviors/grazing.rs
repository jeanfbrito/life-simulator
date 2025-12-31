use crate::ai::actions::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::TilePosition;
use crate::pathfinding::RegionMap;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
/// Grazing Behavior - for herbivores that eat grass
///
/// This behavior makes entities move to nearby grass tiles to graze/forage.
/// Suitable for: Rabbits, Deer, Sheep, Horses, etc.
use bevy::prelude::*;

/// Evaluate the utility of grazing on grass
///
/// Returns a grazing action (Wander to grass tile) if grass is found nearby.
/// This represents natural idle behavior for herbivores.
///
/// PERFORMANCE: Uses RegionMap for O(1) reachability filtering (eliminates pathfinding failures)
///
/// # Parameters
/// - `position`: Current position of the entity
/// - `world_loader`: Access to terrain data
/// - `region_map`: RegionMap for reachability filtering
/// - `graze_distance`: Range in tiles to search for grass (3-8 recommended for rabbits, 5-15 for deer)
///
/// # Returns
/// - `Some(UtilityScore)` if grass is found within range
/// - `None` if no grass is available
pub fn evaluate_grazing_behavior(
    position: &TilePosition,
    world_loader: &WorldLoader,
    region_map: &RegionMap,
    graze_distance: (i32, i32), // (min, max) range
) -> Option<UtilityScore> {
    use rand::Rng;

    // Get entity's region for reachability filtering
    let entity_region = region_map.get_region(position.tile);

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

    // O(1) reachability check - skip if unreachable
    if let Some(my_region) = entity_region {
        if let Some(target_region) = region_map.get_region(target) {
            if target_region != my_region {
                // Target is in a different region - unreachable
                // Fall through to try finding nearby grass
            } else {
                // Verify it's grass (herbivores only graze on grass!)
                if let Some(terrain_str) = world_loader.get_terrain_at(target.x, target.y) {
                    if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                        if matches!(terrain, TerrainType::Grass) {
                            // Perfect! Found reachable grass to graze on
                            return Some(UtilityScore {
                                action_type: ActionType::Graze {
                                    target_tile: target,
                                },
                                utility: 0.15, // Moderate - natural idle behavior like grazing
                                priority: 10,  // Low priority - urgent needs override this
                            });
                        }
                    }
                }
            }
        }
    }

    // Target not reachable or not grass - try to find nearby reachable grass
    if let Some(grass_tile) = find_nearest_grass(target, 5, world_loader, region_map, entity_region) {
        return Some(UtilityScore {
            action_type: ActionType::Graze {
                target_tile: grass_tile,
            },
            utility: 0.15, // Moderate - natural idle behavior
            priority: 10,  // Low priority but beats doing nothing
        });
    }

    // Couldn't find reachable grass to graze on - no grazing action
    None
}

/// Find the nearest grass tile from a position (for grazing)
/// Only returns grass tiles that are in the same region as the entity (reachable)
fn find_nearest_grass(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
    region_map: &RegionMap,
    entity_region: Option<u32>,
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

                // O(1) reachability check - skip unreachable tiles
                if let Some(my_region) = entity_region {
                    if let Some(cell_region) = region_map.get_region(check_pos) {
                        if cell_region != my_region {
                            continue; // Unreachable - different region
                        }
                    } else {
                        continue; // Cell not in any walkable region
                    }
                }

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
    

    // Tests would go here - checking that grazing only happens on grass tiles
}
