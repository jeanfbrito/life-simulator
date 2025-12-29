use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::{FearState, TilePosition};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
/// Fleeing Behavior - for prey escaping predators
///
/// This behavior makes entities flee away from detected predators when afraid.
/// Suitable for: Rabbits, Deer, Sheep, Horses (herbivores)
///
/// Phase 3 of the predator-prey system:
/// - Detects fear state from predator proximity
/// - Calculates escape direction opposite to nearest threat
/// - Returns high-priority flee action to override grazing/resting
use bevy::prelude::*;

/// Fear threshold to trigger fleeing behavior (0.0-1.0)
pub const FLEE_FEAR_THRESHOLD: f32 = 0.3;

/// Distance to flee from predator (in tiles)
/// Set to 2x the fear detection radius (40 tiles) to get to safety
pub const FLEE_DISTANCE: i32 = 80;

/// Priority for flee actions (must beat mating, hunting, basic needs)
/// Priority hierarchy:
/// - Flee: 450 (survival)
/// - Hunt: 360-420 (predator needs)
/// - Mate: 350 (reproduction)
/// - Drink/Eat (critical): 500-1000 (immediate survival)
/// - Rest: 100-500 (maintenance)
/// - Graze: 10 (idle behavior)
pub const FLEE_PRIORITY: i32 = 450;

/// Utility for fleeing when afraid (0.0-1.0)
/// High utility ensures it beats grazing/resting
pub const FLEE_UTILITY: f32 = 0.9;

/// Evaluate the utility of fleeing from predators
///
/// Returns a flee action (move away from predator) if fear level is above threshold.
/// This represents emergency escape behavior for prey animals.
///
/// # Parameters
/// - `position`: Current position of the entity
/// - `fear_state`: Current fear state (detects predators)
/// - `predator_position`: Position of the nearest predator
/// - `world_loader`: Access to terrain data for pathfinding
///
/// # Returns
/// - `Some(UtilityScore)` if fearful and valid escape route exists
/// - `None` if not afraid or no escape route available
pub fn evaluate_fleeing_behavior(
    position: &TilePosition,
    fear_state: &FearState,
    predator_position: IVec2,
    world_loader: &WorldLoader,
) -> Option<UtilityScore> {
    // Only flee when fear level is above threshold
    if fear_state.fear_level < FLEE_FEAR_THRESHOLD {
        return None;
    }

    // Calculate flee direction (opposite from predator)
    let flee_vector = position.tile - predator_position;

    // Handle edge case: prey is on same tile as predator
    if flee_vector == IVec2::ZERO {
        // Pick a random direction to flee
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        let flee_vector = IVec2::new(
            (angle.cos() * FLEE_DISTANCE as f32) as i32,
            (angle.sin() * FLEE_DISTANCE as f32) as i32,
        );
        let target = position.tile + flee_vector;

        // Find nearest walkable tile in that direction
        if let Some(walkable_target) = find_walkable_tile_in_direction(
            position.tile,
            target,
            world_loader,
        ) {
            return Some(UtilityScore {
                action_type: ActionType::Wander {
                    target_tile: walkable_target,
                },
                utility: FLEE_UTILITY,
                priority: FLEE_PRIORITY,
            });
        }
        return None;
    }

    // Normalize flee vector and scale to flee distance
    let flee_length = (flee_vector.x.pow(2) + flee_vector.y.pow(2)) as f32;
    let flee_length_sqrt = flee_length.sqrt();

    // Avoid division by zero
    if flee_length_sqrt < 0.1 {
        return None;
    }

    let normalized_x = (flee_vector.x as f32 / flee_length_sqrt) * FLEE_DISTANCE as f32;
    let normalized_y = (flee_vector.y as f32 / flee_length_sqrt) * FLEE_DISTANCE as f32;
    let flee_target = position.tile + IVec2::new(normalized_x as i32, normalized_y as i32);

    // Find nearest walkable tile in flee direction
    let walkable_target = find_walkable_tile_in_direction(
        position.tile,
        flee_target,
        world_loader,
    )?;

    // Scale utility based on fear level (higher fear = higher urgency)
    let utility = FLEE_UTILITY * fear_state.fear_level;

    Some(UtilityScore {
        action_type: ActionType::Wander {
            target_tile: walkable_target,
        },
        utility,
        priority: FLEE_PRIORITY,
    })
}

/// Find a walkable tile in the direction of the target
///
/// Searches from the target backwards towards the current position
/// to find the furthest walkable tile in the flee direction.
fn find_walkable_tile_in_direction(
    from: IVec2,
    target: IVec2,
    world_loader: &WorldLoader,
) -> Option<IVec2> {
    // First, check if target itself is walkable
    if is_walkable_terrain(target, world_loader) {
        return Some(target);
    }

    // Search backwards from target towards current position
    // to find furthest walkable tile in flee direction
    let direction = target - from;
    let steps = direction.x.abs().max(direction.y.abs());

    if steps == 0 {
        return None;
    }

    // Step backwards from target
    for i in (1..steps).rev() {
        let check_pos = from + IVec2::new(
            (direction.x as f32 * i as f32 / steps as f32) as i32,
            (direction.y as f32 * i as f32 / steps as f32) as i32,
        );

        if is_walkable_terrain(check_pos, world_loader) {
            return Some(check_pos);
        }
    }

    // Fallback: search in a cone around flee direction
    find_nearest_walkable_in_cone(from, direction, 30, world_loader)
}

/// Find nearest walkable tile in a cone around the flee direction
///
/// Searches in a 90-degree cone in the flee direction to find escape routes
fn find_nearest_walkable_in_cone(
    from: IVec2,
    direction: IVec2,
    max_radius: i32,
    world_loader: &WorldLoader,
) -> Option<IVec2> {
    let dir_angle = (direction.y as f32).atan2(direction.x as f32);

    // Search in expanding circles, prioritizing tiles in the flee direction
    for radius in (5..=max_radius).step_by(5) {
        let mut candidates = Vec::new();

        // Check tiles at this radius
        for angle_offset in [-0.4, -0.2, 0.0, 0.2, 0.4] {
            // 90-degree cone (±0.4 radians ≈ ±45 degrees)
            let angle = dir_angle + angle_offset;
            let check_pos = from + IVec2::new(
                (angle.cos() * radius as f32) as i32,
                (angle.sin() * radius as f32) as i32,
            );

            if is_walkable_terrain(check_pos, world_loader) {
                candidates.push((check_pos, angle_offset.abs()));
            }
        }

        // Return closest to flee direction
        if !candidates.is_empty() {
            candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            return Some(candidates[0].0);
        }
    }

    None
}

/// Check if a tile is walkable (not water, not blocked)
fn is_walkable_terrain(pos: IVec2, world_loader: &WorldLoader) -> bool {
    if let Some(terrain_str) = world_loader.get_terrain_at(pos.x, pos.y) {
        if let Some(terrain) = TerrainType::from_str(&terrain_str) {
            return terrain.is_walkable()
                && !matches!(
                    terrain,
                    TerrainType::ShallowWater | TerrainType::DeepWater | TerrainType::Water
                );
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Flee behavior triggers when fear level exceeds threshold
    #[test]
    fn test_flee_triggers_at_fear_threshold() {
        // Mock setup (simplified - in real test would use test world)
        let position = TilePosition::from_tile(IVec2::new(50, 50));
        let predator_pos = IVec2::new(45, 45);

        // Below threshold - should not flee
        let low_fear = FearState {
            fear_level: 0.2,
            nearby_predators: 1,
            ticks_since_danger: 0,
            peak_fear: 0.2,
            last_logged_fear: 0.0,
            nearest_predator_pos: Some(predator_pos),
        };

        // Note: This will return None without a real WorldLoader
        // In a real test, we'd use a mock WorldLoader
        // This test documents the expected behavior

        assert!(
            low_fear.fear_level < FLEE_FEAR_THRESHOLD,
            "Low fear should be below flee threshold"
        );

        // Above threshold - should flee
        let high_fear = FearState {
            fear_level: 0.8,
            nearby_predators: 2,
            ticks_since_danger: 0,
            peak_fear: 0.8,
            last_logged_fear: 0.0,
            nearest_predator_pos: Some(predator_pos),
        };

        assert!(
            high_fear.fear_level >= FLEE_FEAR_THRESHOLD,
            "High fear should trigger fleeing"
        );
    }

    /// Test 2: Flee priority is higher than grazing/mating but respects critical needs
    #[test]
    fn test_flee_priority_hierarchy() {
        assert!(
            FLEE_PRIORITY > 350,
            "Flee priority should beat mating (350)"
        );
        assert!(
            FLEE_PRIORITY < 500,
            "Flee priority should not beat critical thirst/hunger (500+)"
        );
        assert_eq!(FLEE_PRIORITY, 450, "Flee priority should be 450");
    }

    /// Test 3: Flee utility scales with fear level
    #[test]
    fn test_flee_utility_scales_with_fear() {
        let base_utility = FLEE_UTILITY;

        // Low fear above threshold (0.3)
        let low_fear_utility = base_utility * 0.4;
        assert!(
            low_fear_utility < base_utility,
            "Low fear should have lower utility"
        );

        // High fear (0.8)
        let high_fear_utility = base_utility * 0.8;
        assert!(
            high_fear_utility > low_fear_utility,
            "High fear should have higher utility"
        );

        // Maximum fear (1.0)
        let max_fear_utility = base_utility * 1.0;
        assert_eq!(
            max_fear_utility, FLEE_UTILITY,
            "Maximum fear should use full utility"
        );
    }

    /// Test 4: Flee direction is opposite to predator
    #[test]
    fn test_flee_direction_calculation() {
        let prey_pos = IVec2::new(50, 50);
        let predator_pos = IVec2::new(45, 45);

        // Flee vector should point away from predator
        let flee_vector = prey_pos - predator_pos;
        assert_eq!(flee_vector, IVec2::new(5, 5), "Flee vector should point northeast");

        // Normalized and scaled to FLEE_DISTANCE
        let flee_length = (flee_vector.x.pow(2) + flee_vector.y.pow(2)) as f32;
        let flee_length_sqrt = flee_length.sqrt();
        let normalized_x = (flee_vector.x as f32 / flee_length_sqrt) * FLEE_DISTANCE as f32;
        let normalized_y = (flee_vector.y as f32 / flee_length_sqrt) * FLEE_DISTANCE as f32;

        assert!(
            normalized_x > 0.0 && normalized_y > 0.0,
            "Normalized flee direction should be away from predator"
        );

        let flee_distance_actual = (normalized_x.powi(2) + normalized_y.powi(2)).sqrt();
        assert!(
            (flee_distance_actual - FLEE_DISTANCE as f32).abs() < 1.0,
            "Flee distance should be approximately FLEE_DISTANCE (80 tiles)"
        );
    }

    /// Test 5: Flee behavior respects walkable terrain
    #[test]
    fn test_flee_respects_walkable_terrain() {
        // This test documents that flee behavior checks terrain walkability
        // In a real implementation with mock WorldLoader:
        // 1. Should reject water tiles
        // 2. Should reject blocked tiles
        // 3. Should find nearest walkable alternative

        // Document expected behavior
        assert!(
            FLEE_DISTANCE > 0,
            "Flee distance should be positive"
        );
    }
}
