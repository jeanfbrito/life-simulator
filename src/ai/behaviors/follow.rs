use crate::ai::actions::ActionType;
use crate::ai::planner::UtilityScore;
use crate::entities::TilePosition;
/// Follow Behavior - make one entity follow another at a comfortable distance
///
/// This behavior chooses the nearest candidate entity and asks the follower to move toward it
/// until within a stop distance. Intended for simple "follow the leader" examples.
/// Can be used by any species to follow mothers, herd leaders, or other entities.
use bevy::prelude::*;

/// Evaluate the utility of following the nearest candidate entity
/// - entity: the follower entity id (for logging, not used for scoring)
/// - position: follower's current tile position
/// - candidates: list of (target entity, tile position) pairs that can be followed
/// - stop_distance: chebyshev distance at which the follower considers itself "close enough"
/// - max_follow_distance: cap for distance normalization in utility calculation
/// - is_juvenile: if true, uses higher priority (juveniles MUST stay near mothers)
pub fn evaluate_follow_behavior(
    _entity: Entity,
    position: &TilePosition,
    candidates: &[(Entity, IVec2)],
    stop_distance: i32,
    max_follow_distance: i32,
) -> Option<UtilityScore> {
    evaluate_follow_behavior_with_priority(_entity, position, candidates, stop_distance, max_follow_distance, false)
}

/// Evaluate follow behavior with optional juvenile priority boost
pub fn evaluate_follow_behavior_with_priority(
    _entity: Entity,
    position: &TilePosition,
    candidates: &[(Entity, IVec2)],
    stop_distance: i32,
    max_follow_distance: i32,
    is_juvenile: bool,
) -> Option<UtilityScore> {
    // Find nearest candidate by Chebyshev distance (diagonal allowed)
    let mut best: Option<(Entity, IVec2, i32)> = None; // (target, pos, chebyshev dist)

    for (target, pos) in candidates.iter().copied() {
        let d = chebyshev_distance(position.tile, pos);
        match best {
            Some((_, _, best_d)) if d >= best_d => {}
            _ => {
                best = Some((target, pos, d));
            }
        }
    }

    let Some((target, _target_pos, dist)) = best else {
        return None;
    };

    // If already within stop distance, very low utility to move
    if dist <= stop_distance {
        return None;
    }

    // Utility scales with distance beyond stop_distance up to max_follow_distance
    let effective = (dist - stop_distance) as f32;
    let max_eff = (max_follow_distance.max(1) - stop_distance.max(0)) as f32;

    // Juveniles get higher utility to make following more important
    let base_utility = (effective / max_eff).clamp(0.0, 1.0).max(0.15);
    let utility = if is_juvenile {
        // Juveniles: higher utility (0.4-0.85) to compete with survival needs
        0.4 + (base_utility * 0.45)
    } else {
        base_utility
    };

    // Priority: juveniles get priority 120 (above basic survival ~100), adults get 20
    let priority = if is_juvenile { 120 } else { 20 };

    Some(UtilityScore {
        action_type: ActionType::Follow {
            target,
            stop_distance,
        },
        utility,
        priority,
    })
}

fn chebyshev_distance(a: IVec2, b: IVec2) -> i32 {
    let d = (a - b).abs();
    d.x.max(d.y)
}
