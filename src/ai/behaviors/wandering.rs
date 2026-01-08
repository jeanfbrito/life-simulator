use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

use crate::ai::UtilityScore;
use crate::ai::action::ActionType;
use crate::entities::TilePosition;
use crate::entities::types::HabitatPreference;
use crate::tilemap::TerrainType;
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;

/// Number of candidate tiles to sample when evaluating wander targets
const WANDER_SAMPLE_SIZE: usize = 12;

/// Minimum terrain score to consider a tile (filters out unsuitable terrain)
const MIN_TERRAIN_SCORE: f32 = 0.1;

/// Evaluate wandering behavior with habitat preferences
///
/// Instead of random wandering, this evaluates candidate tiles based on:
/// 1. Terrain preference (species-specific)
/// 2. Food proximity (optional bonus)
/// 3. Water proximity (optional bonus)
///
/// This keeps animals in their preferred habitats and near resources.
pub fn evaluate_wandering_behavior(
    position: &TilePosition,
    world_loader: &WorldLoader,
    wander_radius: i32,
    habitat: &HabitatPreference,
    resource_grid: Option<&ResourceGrid>,
) -> Option<UtilityScore> {
    let mut rng = rand::thread_rng();
    let mut best_target: Option<(IVec2, f32)> = None;

    // Sample multiple candidate positions and score them
    for _ in 0..WANDER_SAMPLE_SIZE {
        let distance = rng.gen_range(1..=wander_radius);
        let angle = rng.gen::<f32>() * TAU;

        let dx = (angle.cos() * distance as f32).round() as i32;
        let dy = (angle.sin() * distance as f32).round() as i32;
        let target = position.tile + IVec2::new(dx, dy);

        // Score this tile based on habitat preferences
        if let Some(score) = score_wander_tile(
            target,
            position.tile,
            world_loader,
            habitat,
            resource_grid,
        ) {
            // Keep the best scoring tile
            if best_target.is_none() || score > best_target.unwrap().1 {
                best_target = Some((target, score));
            }
        }
    }

    // Return best tile found, or None if all tiles were unsuitable
    best_target.map(|(target, _score)| UtilityScore {
        action_type: ActionType::Wander { target_tile: target },
        utility: 0.06,  // Just above threshold, always available
        priority: 1,    // Lowest priority (everything overrides)
    })
}

/// Score a potential wander tile based on habitat preferences
///
/// Returns None if tile is impassable or unsuitable.
/// Higher scores indicate more desirable tiles.
fn score_wander_tile(
    tile: IVec2,
    from: IVec2,
    world_loader: &WorldLoader,
    habitat: &HabitatPreference,
    resource_grid: Option<&ResourceGrid>,
) -> Option<f32> {
    // Get terrain at target
    let terrain_str = world_loader.get_terrain_at(tile.x, tile.y)?;
    let terrain = TerrainType::from_str(&terrain_str)?;

    // Check if terrain is walkable
    if !terrain.is_walkable() {
        return None;
    }

    // Get terrain preference score
    let terrain_score = habitat.get_terrain_weight(&terrain);
    if terrain_score < MIN_TERRAIN_SCORE {
        return None; // Species avoids this terrain
    }

    let mut total_score = terrain_score;

    // Bonus for tiles with food nearby (if food_proximity_weight > 0)
    if habitat.food_proximity_weight > 0.0 {
        if let Some(grid) = resource_grid {
            let food_score = score_food_proximity(tile, grid);
            total_score += food_score * habitat.food_proximity_weight;
        }
    }

    // Bonus for tiles near water (if water_proximity_weight > 0)
    if habitat.water_proximity_weight > 0.0 {
        let water_score = score_water_proximity(tile, world_loader);
        total_score += water_score * habitat.water_proximity_weight;
    }

    // Slight preference for closer tiles (less travel time)
    let distance = from.as_vec2().distance(tile.as_vec2());
    let distance_penalty = (distance / 50.0).min(0.2); // Max 0.2 penalty
    total_score -= distance_penalty;

    Some(total_score.max(0.0))
}

/// Score how close a tile is to food sources
/// Returns 0.0-1.0 based on nearby biomass
fn score_food_proximity(tile: IVec2, resource_grid: &ResourceGrid) -> f32 {
    // Check if this tile has biomass directly
    if let Some(cell) = resource_grid.get_cell(tile) {
        if cell.total_biomass > 10.0 {
            return 1.0; // Direct food source
        }
    }

    // Check adjacent tiles for food
    let mut adjacent_food = 0;
    for dx in -2..=2 {
        for dy in -2..=2 {
            if dx == 0 && dy == 0 { continue; }
            let check = tile + IVec2::new(dx, dy);
            if let Some(cell) = resource_grid.get_cell(check) {
                if cell.total_biomass > 10.0 {
                    adjacent_food += 1;
                }
            }
        }
    }

    // More adjacent food = higher score (max ~0.8)
    (adjacent_food as f32 / 10.0).min(0.8)
}

/// Score how close a tile is to water sources
/// Returns 0.0-1.0 based on water proximity
fn score_water_proximity(tile: IVec2, world_loader: &WorldLoader) -> f32 {
    // Check adjacent tiles for water
    for radius in 1i32..=5 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() < radius && dy.abs() < radius { continue; }
                let check = tile + IVec2::new(dx, dy);
                if let Some(terrain_str) = world_loader.get_terrain_at(check.x, check.y) {
                    if terrain_str == "Water" || terrain_str == "ShallowWater" {
                        // Found water! Score decreases with distance
                        return 1.0 - (radius as f32 / 6.0);
                    }
                }
            }
        }
    }

    0.0 // No water nearby
}

/// Legacy function for backward compatibility
/// Uses default habitat preferences
pub fn evaluate_wandering_behavior_simple(
    position: &TilePosition,
    world_loader: &WorldLoader,
    wander_radius: i32,
) -> Option<UtilityScore> {
    evaluate_wandering_behavior(
        position,
        world_loader,
        wander_radius,
        &HabitatPreference::default(),
        None,
    )
}
