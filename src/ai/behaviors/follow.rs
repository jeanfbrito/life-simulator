/// Follow Behavior - make one entity follow another at a comfortable distance
///
/// This behavior chooses the nearest rabbit and asks the follower to move toward it
/// until within a stop distance. Intended for simple "follow the leader" examples.

use bevy::prelude::*;
use crate::entities::TilePosition;
use crate::ai::action::ActionType;
use crate::ai::planner::UtilityScore;

/// Evaluate the utility of following the nearest rabbit
/// - entity: the follower entity id (for logging, not used for scoring)
/// - position: follower's current tile position
/// - rabbits: list of (rabbit entity, tile position) pairs that can be followed
/// - stop_distance: chebyshev distance at which the follower considers itself "close enough"
/// - max_follow_distance: cap for distance normalization in utility calculation
pub fn evaluate_follow_behavior(
    _entity: Entity,
    position: &TilePosition,
    rabbits: &[(Entity, IVec2)],
    stop_distance: i32,
    max_follow_distance: i32,
) -> Option<UtilityScore> {
    // Find nearest rabbit by Chebyshev distance (diagonal allowed)
    let mut best: Option<(Entity, IVec2, i32)> = None; // (target, pos, chebyshev dist)

    for (target, pos) in rabbits.iter().copied() {
        let d = chebyshev_distance(position.tile, pos);
        match best {
            Some((_, _, best_d)) if d >= best_d => {}
            _ => {
                best = Some((target, pos, d));
            }
        }
    }

    let Some((target, target_pos, dist)) = best else { return None; };

    // If already within stop distance, very low utility to move
    if dist <= stop_distance { return None; }

    // Utility scales with distance beyond stop_distance up to max_follow_distance
    let effective = (dist - stop_distance) as f32;
    let max_eff = (max_follow_distance.max(1) - stop_distance.max(0)) as f32;
    let utility = (effective / max_eff).clamp(0.0, 1.0).max(0.15); // ensure small pull

    // Priority: below basic needs, above idle grazing
    let priority = 20;

    Some(UtilityScore {
        action_type: ActionType::Follow { target, stop_distance },
        utility,
        priority,
    })
}

fn chebyshev_distance(a: IVec2, b: IVec2) -> i32 {
    let d = (a - b).abs();
    d.x.max(d.y)
}