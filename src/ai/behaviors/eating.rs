use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::{stats::Hunger, TilePosition};
use crate::tilemap::TerrainType;
use crate::vegetation::resource_grid::*;
use crate::world_loader::WorldLoader;
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
    resource_grid: &ResourceGrid,
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

    // Find nearest suitable vegetation cell with sufficient biomass using new ResourceGrid
    let forage_tile = find_best_forage_cell_with_strategy(
        position.tile,
        search_radius,
        world_loader,
        resource_grid,
        foraging_strategy.into(),
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

/// Find the best forage cell using new ResourceGrid system
fn find_best_forage_cell_with_strategy(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    strategy: ForageSearchStrategy,
) -> Option<IVec2> {
    // Use ResourceGrid's built-in optimization first
    if let ForageSearchStrategy::Exhaustive = strategy {
        // Try the ResourceGrid's optimized search first
        if let Some((pos, biomass)) = resource_grid.find_best_cell(from, max_radius) {
            // Verify the cell is accessible (no blocking resources)
            if is_cell_accessible(pos, world_loader) && supports_vegetation_at(pos, world_loader) {
                return Some(pos);
            }
        }
    }

    // Fallback to our own search with sampling if needed
    match strategy {
        ForageSearchStrategy::Exhaustive => {
            // Exhaustive search with ResourceGrid lookups
            for radius in 1..=max_radius {
                for dx in -radius..=radius {
                    for dy in -radius..=radius {
                        // Only check perimeter (not interior)
                        if dx.abs() < radius && dy.abs() < radius {
                            continue;
                        }

                        let check_pos = from + IVec2::new(dx, dy);
                        if is_cell_suitable_for_foraging(
                            check_pos,
                            from,
                            world_loader,
                            resource_grid,
                        ) {
                            return Some(check_pos);
                        }
                    }
                }
            }
        }
        ForageSearchStrategy::Sampled { sample_size } => {
            // Random sampling within radius
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
                if is_cell_suitable_for_foraging(check_pos, from, world_loader, resource_grid) {
                    return Some(check_pos);
                }
            }
        }
    }

    None
}

/// Search strategies for forage cell selection
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
            crate::entities::types::ForagingStrategy::Sampled { sample_size } => {
                Self::Sampled { sample_size }
            }
        }
    }
}

/// Check if a cell is suitable for foraging using ResourceGrid
fn is_cell_suitable_for_foraging(
    cell: IVec2,
    from: IVec2,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
) -> bool {
    // Check if this cell supports vegetation
    if !supports_vegetation_at(cell, world_loader) {
        return false;
    }

    // Check if cell is accessible (no blocking resources)
    if !is_cell_accessible(cell, world_loader) {
        return false;
    }

    // Check if cell has sufficient biomass using ResourceGrid
    if let Some(cell_data) = resource_grid.get_cell(cell) {
        // Minimum biomass threshold (FORAGE_MIN_BIOMASS from old system)
        const FORAGE_MIN_BIOMASS: f32 = 10.0;
        cell_data.total_biomass >= FORAGE_MIN_BIOMASS && !cell_data.is_depleted()
    } else {
        false
    }
}

/// Check if terrain at position supports vegetation growth
fn supports_vegetation_at(pos: IVec2, world_loader: &WorldLoader) -> bool {
    if let Some(terrain_str) = world_loader.get_terrain_at(pos.x, pos.y) {
        if let Some(terrain) = TerrainType::from_str(&terrain_str) {
            return supports_vegetation(&terrain);
        }
    }
    false
}

/// Check if a cell is accessible (no blocking resources)
fn is_cell_accessible(pos: IVec2, world_loader: &WorldLoader) -> bool {
    !world_loader
        .get_resource_at(pos.x, pos.y)
        .map(|r| !r.is_empty())
        .unwrap_or(false)
}

/// Check if a terrain type supports vegetation growth
fn supports_vegetation(terrain: &TerrainType) -> bool {
    use TerrainType::*;
    matches!(
        terrain,
        Grass | Forest | Dirt | Swamp | Desert | Stone | Snow
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here - checking grass-finding logic
}
