use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::{stats::Hunger, TilePosition};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use crate::vegetation::{VegetationGrid, consumption::FORAGE_MIN_BIOMASS};
/// Eating Behavior - for herbivores that consume grass
///
/// This behavior makes entities find and eat grass when hungry.
/// Suitable for: Rabbits, Deer, Sheep, Horses, etc.
use bevy::prelude::*;

/// Evaluate the utility of eating grass
///
/// Returns an eating action if hunger is above threshold and vegetation with sufficient biomass is found nearby.
///
/// # Parameters
/// - `position`: Current position of the entity
/// - `hunger`: Current hunger level
/// - `world_loader`: Access to terrain data
/// - `vegetation_grid`: Access to vegetation biomass data
/// - `hunger_threshold`: Minimum hunger level to seek food (0.0-1.0)
/// - `search_radius`: Maximum tiles to search for grass
///
/// # Returns
/// - `Some(UtilityScore)` if hungry enough and suitable vegetation is found
/// - `None` if not hungry or no suitable vegetation available
pub fn evaluate_eating_behavior(
    position: &TilePosition,
    hunger: &Hunger,
    world_loader: &WorldLoader,
    vegetation_grid: &VegetationGrid,
    hunger_threshold: f32,
    search_radius: i32,
    foraging_strategy: crate::entities::types::ForagingStrategy,
) -> Option<UtilityScore> {
    // Only seek food when hunger is above threshold
    let hunger_level = hunger.0.normalized();
    if hunger_level < hunger_threshold {
        return None; // Not hungry enough
    }

    // Note: We don't check if already on suitable vegetation because eating is handled by action execution

    // Find nearest suitable vegetation tile with sufficient biomass
    let forage_tile = find_best_forage_tile_with_strategy(
        position.tile,
        search_radius,
        world_loader,
        vegetation_grid,
        foraging_strategy.into()
    )?;

    // Calculate utility based on hunger and distance
    let distance = position.tile.as_vec2().distance(forage_tile.as_vec2());
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

    // Return Graze action to move to vegetation tile (action execution will handle eating)
    Some(UtilityScore {
        action_type: ActionType::Graze {
            target_tile: forage_tile,
        },
        utility,
        priority,
    })
}

/// Find the best forage tile using configurable search strategy
fn find_best_forage_tile(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
    vegetation_grid: &VegetationGrid,
) -> Option<IVec2> {
    find_best_forage_tile_with_strategy(
        from,
        max_radius,
        world_loader,
        vegetation_grid,
        ForageSearchStrategy::Exhaustive,
    )
}

/// Search strategies for forage tile selection
#[derive(Debug, Clone)]
enum ForageSearchStrategy {
    /// Search all tiles within radius (thorough, more expensive)
    Exhaustive,
    /// Sample K random tiles within radius (faster, good approximation)
    Sampled { sample_size: usize },
}

impl From<crate::entities::types::ForagingStrategy> for ForageSearchStrategy {
    fn from(strategy: crate::entities::types::ForagingStrategy) -> Self {
        match strategy {
            crate::entities::types::ForagingStrategy::Exhaustive => Self::Exhaustive,
            crate::entities::types::ForagingStrategy::Sampled { sample_size } => Self::Sampled { sample_size },
        }
    }
}

/// Find the best forage tile using specified search strategy
fn find_best_forage_tile_with_strategy(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
    vegetation_grid: &VegetationGrid,
    strategy: ForageSearchStrategy,
) -> Option<IVec2> {
    let mut best_tile: Option<(IVec2, f32)> = None; // (position, utility_score)

    match strategy {
        ForageSearchStrategy::Exhaustive => {
            // Search all tiles in expanding square pattern
            for radius in 1..=max_radius {
                for dx in -radius..=radius {
                    for dy in -radius..=radius {
                        // Only check perimeter (not interior)
                        if dx.abs() < radius && dy.abs() < radius {
                            continue;
                        }

                        let check_pos = from + IVec2::new(dx, dy);
                        if let Some(score) = evaluate_forage_tile(
                            check_pos, from, world_loader, vegetation_grid
                        ) {
                            if let Some((_, best_utility)) = best_tile {
                                if score > best_utility {
                                    best_tile = Some((check_pos, score));
                                }
                            } else {
                                best_tile = Some((check_pos, score));
                            }
                        }
                    }
                }

                // Early exit if we found suitable vegetation at this radius
                if best_tile.is_some() {
                    break;
                }
            }
        }
        ForageSearchStrategy::Sampled { sample_size } => {
            // Sample K random tiles within radius
            let mut candidates = Vec::new();

            for radius in 1..=max_radius {
                for dx in -radius..=radius {
                    for dy in -radius..=radius {
                        let check_pos = from + IVec2::new(dx, dy);
                        let distance = from.as_vec2().distance(check_pos.as_vec2());
                        if distance <= max_radius as f32 {
                            candidates.push(check_pos);
                        }
                    }
                }
            }

            // Shuffle and sample
            use rand::seq::SliceRandom;
            use rand::thread_rng;
            let mut rng = thread_rng();
            candidates.shuffle(&mut rng);

            // Evaluate first K candidates
            for check_pos in candidates.into_iter().take(sample_size) {
                if let Some(score) = evaluate_forage_tile(
                    check_pos, from, world_loader, vegetation_grid
                ) {
                    if let Some((_, best_utility)) = best_tile {
                        if score > best_utility {
                            best_tile = Some((check_pos, score));
                        }
                    } else {
                        best_tile = Some((check_pos, score));
                    }
                }
            }
        }
    }

    best_tile.map(|(pos, _)| pos)
}

/// Evaluate a tile for foraging suitability and return utility score
fn evaluate_forage_tile(
    tile: IVec2,
    from: IVec2,
    world_loader: &WorldLoader,
    vegetation_grid: &VegetationGrid,
) -> Option<f32> {
    // Check if this tile supports vegetation (grass, forest, etc.)
    if let Some(terrain_str) = world_loader.get_terrain_at(tile.x, tile.y) {
        if let Some(terrain) = TerrainType::from_str(&terrain_str) {
            if supports_vegetation(&terrain) {
                // Check if tile has sufficient biomass
                if let Some(vegetation) = vegetation_grid.get(tile) {
                    if vegetation.biomass >= FORAGE_MIN_BIOMASS {
                        // Check if tile is accessible (no blocking resources)
                        let has_resource = world_loader
                            .get_resource_at(tile.x, tile.y)
                            .map(|r| !r.is_empty())
                            .unwrap_or(false);

                        if !has_resource {
                            // Calculate utility: biomass / (1 + distance_penalty)
                            let distance = from.as_vec2().distance(tile.as_vec2());
                            let distance_penalty = distance * 0.1;
                            let utility_score = vegetation.biomass / (1.0 + distance_penalty);
                            return Some(utility_score);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Check if a terrain type supports vegetation growth
fn supports_vegetation(terrain: &TerrainType) -> bool {
    use TerrainType::*;
    matches!(terrain, Grass | Forest | Dirt | Swamp | Desert | Stone | Snow)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here - checking grass-finding logic
}
