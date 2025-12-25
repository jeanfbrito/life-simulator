use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

use crate::ai::UtilityScore;
use crate::ai::action::ActionType;
use crate::entities::TilePosition;
use crate::world_loader::WorldLoader;

/// Evaluate wandering behavior - idle exploration within territory
pub fn evaluate_wandering_behavior(
    position: &TilePosition,
    world_loader: &WorldLoader,
    wander_radius: i32,
) -> Option<UtilityScore> {
    let mut rng = rand::thread_rng();

    // Try 5 random polar coordinates
    for _ in 0..5 {
        let distance = rng.gen_range(1..=wander_radius);
        let angle = rng.gen::<f32>() * TAU;

        let dx = (angle.cos() * distance as f32).round() as i32;
        let dy = (angle.sin() * distance as f32).round() as i32;
        let target = position.tile + IVec2::new(dx, dy);

        // Check if walkable
        if let Some(terrain) = world_loader.get_terrain_at(target.x, target.y) {
            if terrain != "Water" && terrain != "DeepWater" && terrain != "Mountain" {
                return Some(UtilityScore {
                    action_type: ActionType::Wander { target_tile: target },
                    utility: 0.06,  // Just above threshold, always available
                    priority: 1,    // Lowest priority (everything overrides)
                });
            }
        }
    }

    // Fallback: no wander action if all attempts fail
    None
}
