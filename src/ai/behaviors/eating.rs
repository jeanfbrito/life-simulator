use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::{stats::Hunger, TilePosition};
use crate::tilemap::TerrainType;
use crate::vegetation::resource_grid::*;
use crate::world_loader::WorldLoader;
use crate::resources::ResourceType;
/// Eating Behavior - for herbivores that consume vegetation
///
/// This behavior makes entities find and eat suitable vegetation when hungry.
/// Herbivores can now distinguish between grass, shrubs, and other edible plants.
/// Species-specific satisfaction thresholds determine pickiness.
/// Suitable for: Rabbits, Deer, Sheep, Horses, etc.
use bevy::prelude::*;

/// Minimum biomass that is worth eating at all
const MIN_EDIBLE_BIOMASS: f32 = 5.0;

/// Minimum pickiness level (prevents complete desperation)
const MIN_PICKINESS: f32 = 0.1;

/// Calculate effective satisfaction threshold based on energy level
///
/// Animals with high energy can be picky about food quality.
/// Animals with low energy become desperate and eat whatever is available.
///
/// # Examples
/// - Deer (satisfaction=40) at 100% energy: 40.0 (picky)
/// - Deer (satisfaction=40) at 50% energy: 22.5 (moderate)
/// - Deer (satisfaction=40) at 20% energy: 12.0 (desperate)
/// - Rabbit (satisfaction=15) at 100% energy: 15.0 (easy to please)
/// - Rabbit (satisfaction=15) at 20% energy: 7.0 (will eat anything)
pub fn effective_satisfaction(base_satisfaction: f32, energy_normalized: f32) -> f32 {
    // Pickiness scales with energy, but never goes below MIN_PICKINESS
    let pickiness = energy_normalized.max(MIN_PICKINESS);

    // Threshold scales from MIN_EDIBLE_BIOMASS to base_satisfaction
    MIN_EDIBLE_BIOMASS + (base_satisfaction - MIN_EDIBLE_BIOMASS) * pickiness
}

/// Herbivore diet preferences for different vegetation types
#[derive(Debug, Clone)]
pub struct HerbivoreDiet {
    /// Preference weight for grass/ground vegetation (0.0-1.0)
    pub grass_preference: f32,
    /// Preference weight for shrubs/bushes (0.0-1.0)
    pub shrub_preference: f32,
    /// Minimum biomass threshold for considering a cell (absolute units)
    pub min_biomass_threshold: f32,
}

impl HerbivoreDiet {
    /// Create a new herbivore diet with preferences
    pub fn new(grass_preference: f32, shrub_preference: f32, min_biomass_threshold: f32) -> Self {
        Self {
            grass_preference: grass_preference.clamp(0.0, 1.0),
            shrub_preference: shrub_preference.clamp(0.0, 1.0),
            min_biomass_threshold: min_biomass_threshold.max(0.0),
        }
    }

    /// Get preference for a specific resource type
    pub fn get_preference_for_resource(&self, resource_type: &Option<ResourceType>) -> f32 {
        match resource_type {
            Some(ResourceType::BerryBush) | Some(ResourceType::HazelShrub) | Some(ResourceType::Bush) => {
                self.shrub_preference
            }
            // Grass and other ground vegetation
            Some(_) | None => self.grass_preference,
        }
    }

    /// Check if a resource type is edible for this herbivore
    pub fn is_edible(&self, resource_type: &Option<ResourceType>) -> bool {
        match resource_type {
            Some(rt) => {
                // Check if resource is herbivore-edible based on consumption kind
                rt.is_herbivore_edible()
            }
            None => {
                // Unspecified resources (grass/terrain) are generally edible
                true
            }
        }
    }
}

impl Default for HerbivoreDiet {
    fn default() -> Self {
        // Default diet: prefers grass over shrubs
        Self::new(0.8, 0.4, 10.0)
    }
}

/// Predefined diet types for common herbivores
impl HerbivoreDiet {
    /// Rabbit diet: strongly prefers grass, some shrubs
    pub fn rabbit() -> Self {
        Self::new(0.9, 0.3, 8.0) // Lower threshold for small animals
    }

    /// Deer diet: balanced preference for grass and shrubs
    pub fn deer() -> Self {
        Self::new(0.6, 0.7, 15.0) // Higher threshold for larger animals
    }

    /// Generalist grazer diet: prefers grass
    pub fn grazer() -> Self {
        Self::new(0.8, 0.5, 12.0)
    }
}

/// Evaluate the utility of eating vegetation
///
/// Returns an eating action if hunger is above threshold and suitable vegetation with sufficient biomass is found nearby.
/// Herbivores now consider both grass and shrubs based on their diet preferences.
/// Uses satisfaction model: animals stop searching when they find grass above their satisfaction threshold.
///
/// # Parameters
/// - `position`: Current position of the entity
/// - `hunger`: Current hunger level
/// - `energy_normalized`: Current energy level (0.0-1.0), affects pickiness
/// - `satisfaction_biomass`: Species-specific biomass threshold for satisfaction
/// - `world_loader`: Access to terrain data
/// - `resource_grid`: Access to vegetation biomass data
/// - `hunger_threshold`: Minimum hunger level to seek food (0.0-1.0)
/// - `search_radius`: Maximum tiles to search for vegetation
/// - `foraging_strategy`: Search strategy for finding food
/// - `diet`: Herbivore diet preferences
///
/// # Returns
/// - `Some(UtilityScore)` if hungry enough and suitable vegetation is found
/// - `None` if not hungry or no suitable vegetation available
pub fn evaluate_eating_behavior(
    position: &TilePosition,
    hunger: &Hunger,
    energy_normalized: f32,
    satisfaction_biomass: f32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    hunger_threshold: f32,
    search_radius: i32,
    _foraging_strategy: crate::entities::types::ForagingStrategy,
    diet: &HerbivoreDiet,
) -> Option<UtilityScore> {
    // Only seek food when hunger is above threshold
    let hunger_level = hunger.0.normalized();
    if hunger_level < hunger_threshold {
        return None; // Not hungry enough
    }

    // Calculate effective satisfaction threshold based on energy
    let effective_threshold = effective_satisfaction(satisfaction_biomass, energy_normalized);

    // Find satisfying vegetation cell (stops early when good enough food is found)
    let forage_tile = find_satisfying_forage_cell(
        position.tile,
        search_radius,
        effective_threshold,
        world_loader,
        resource_grid,
        diet,
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

/// Find a satisfying forage cell using the satisfaction model
///
/// Searches in expanding rings (nearby first) and stops as soon as a cell
/// with biomass >= satisfaction_threshold is found. If no satisfying cell
/// is found, returns the best available cell as a fallback.
///
/// This prevents animals from traveling far when good food is nearby.
fn find_satisfying_forage_cell(
    from: IVec2,
    max_radius: i32,
    satisfaction_threshold: f32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    diet: &HerbivoreDiet,
) -> Option<IVec2> {
    let mut best_fallback: Option<(IVec2, f32)> = None;

    // Search in expanding rings (nearby first)
    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Only check perimeter (not interior already checked)
                if dx.abs() < radius && dy.abs() < radius {
                    continue;
                }

                let check_pos = from + IVec2::new(dx, dy);

                // Check if cell is suitable for foraging
                if let Some(biomass) = get_cell_biomass(check_pos, world_loader, resource_grid, diet) {
                    if biomass >= MIN_EDIBLE_BIOMASS {
                        // Track best fallback in case nothing satisfies
                        if best_fallback.is_none() || biomass > best_fallback.unwrap().1 {
                            best_fallback = Some((check_pos, biomass));
                        }

                        // Found satisfying grass? Stop searching immediately!
                        if biomass >= satisfaction_threshold {
                            return Some(check_pos);
                        }
                    }
                }
            }
        }
    }

    // Nothing satisfying found - return best available (desperate eating)
    best_fallback.map(|(pos, _)| pos)
}

/// Get the biomass of a cell if it's suitable for foraging
fn get_cell_biomass(
    cell: IVec2,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    diet: &HerbivoreDiet,
) -> Option<f32> {
    // Check if this cell supports vegetation
    if !supports_vegetation_at(cell, world_loader) {
        return None;
    }

    // Check if cell is accessible (no blocking resources)
    if !is_cell_accessible(cell, world_loader) {
        return None;
    }

    // Check if cell has biomass using ResourceGrid
    if let Some(cell_data) = resource_grid.get_cell(cell) {
        // Check if resource is edible for this herbivore
        if !diet.is_edible(&cell_data.resource_type) {
            return None;
        }

        // Check if not depleted
        if cell_data.is_depleted() {
            return None;
        }

        Some(cell_data.total_biomass)
    } else {
        // No ResourceGrid data - assume basic grass with low biomass
        if diet.grass_preference > 0.0 {
            Some(MIN_EDIBLE_BIOMASS) // Minimum viable
        } else {
            None
        }
    }
}

/// Find the best forage cell using new ResourceGrid system with diet preferences
/// (Legacy function - kept for compatibility but satisfaction model is preferred)
#[allow(dead_code)]
fn find_best_forage_cell_with_strategy(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    strategy: ForageSearchStrategy,
    diet: &HerbivoreDiet,
) -> Option<IVec2> {
    // Find the best forage cell considering diet preferences
    match strategy {
        ForageSearchStrategy::Exhaustive => {
            find_best_forage_cell_exhaustive(from, max_radius, world_loader, resource_grid, diet)
        }
        ForageSearchStrategy::Sampled { sample_size } => {
            find_best_forage_cell_sampled(from, max_radius, world_loader, resource_grid, diet, sample_size)
        }
    }
}

/// Exhaustive search for the best forage cell considering diet preferences
fn find_best_forage_cell_exhaustive(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    diet: &HerbivoreDiet,
) -> Option<IVec2> {
    let mut best_cell: Option<(IVec2, f32)> = None;
    let mut best_score = 0.0;

    // Search in expanding square pattern, prioritizing closer cells
    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Only check perimeter (not interior) to prioritize closer cells
                if dx.abs() < radius && dy.abs() < radius {
                    continue;
                }

                let check_pos = from + IVec2::new(dx, dy);

                if let Some(score) = evaluate_forage_cell_score(check_pos, from, world_loader, resource_grid, diet) {
                    // Apply distance penalty (closer cells get higher scores)
                    let distance = from.as_vec2().distance(check_pos.as_vec2());
                    let distance_penalty = distance / max_radius as f32;
                    let adjusted_score = score * (1.0 - distance_penalty * 0.5); // 50% max distance penalty

                    if adjusted_score > best_score {
                        best_score = adjusted_score;
                        best_cell = Some((check_pos, adjusted_score));
                    }
                }
            }
        }
    }

    best_cell.map(|(pos, _)| pos)
}

/// Sampled search for forage cells - faster but less thorough
fn find_best_forage_cell_sampled(
    from: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    diet: &HerbivoreDiet,
    sample_size: usize,
) -> Option<IVec2> {
    // Collect all candidate positions
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

    // Shuffle and sample candidates
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    let mut rng = thread_rng();
    candidates.shuffle(&mut rng);

    let mut best_cell: Option<(IVec2, f32)> = None;
    let mut best_score = 0.0;

    // Evaluate sampled candidates
    for check_pos in candidates.into_iter().take(sample_size) {
        if let Some(score) = evaluate_forage_cell_score(check_pos, from, world_loader, resource_grid, diet) {
            if score > best_score {
                best_score = score;
                best_cell = Some((check_pos, score));
            }
        }
    }

    best_cell.map(|(pos, _)| pos)
}

/// Evaluate the score of a forage cell based on diet preferences and biomass
fn evaluate_forage_cell_score(
    cell: IVec2,
    _from: IVec2,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    diet: &HerbivoreDiet,
) -> Option<f32> {
    // Check if this cell supports vegetation
    if !supports_vegetation_at(cell, world_loader) {
        return None;
    }

    // Check if cell is accessible (no blocking resources)
    if !is_cell_accessible(cell, world_loader) {
        return None;
    }

    // Check if cell has sufficient biomass using ResourceGrid
    if let Some(cell_data) = resource_grid.get_cell(cell) {
        // Check if resource is edible for this herbivore
        if !diet.is_edible(&cell_data.resource_type) {
            return None;
        }

        // Check biomass threshold
        if cell_data.total_biomass < diet.min_biomass_threshold || cell_data.is_depleted() {
            return None;
        }

        // Calculate score based on diet preference and biomass
        let preference = diet.get_preference_for_resource(&cell_data.resource_type);
        let biomass_factor = (cell_data.total_biomass / diet.min_biomass_threshold).min(3.0); // Cap at 3x threshold

        Some(preference * biomass_factor)
    } else {
        // No ResourceGrid data - assume basic grass
        if diet.grass_preference > 0.0 {
            Some(diet.grass_preference * 0.5) // Lower score for unknown cells
        } else {
            None
        }
    }
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
    _from: IVec2,
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
