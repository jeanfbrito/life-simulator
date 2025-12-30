/// Pathfinding bridge system - connects NeedsPathfinding results to PathfindingQueue
///
/// This system identifies ActionExecutionResult components with NeedsPathfinding result
/// and queues pathfinding requests to PathfindingQueue, then transitions action state
/// from NeedPath to WaitingForPath.
///
/// CRITICAL: Uses Commands and Query parameters (NO &World) to avoid Bevy ECS conflicts
use bevy::prelude::*;
use crate::ai::actions::ActionResult;
use crate::ai::queue::ActionExecutionResult;
use crate::entities::{ActiveAction, TilePosition};
use crate::pathfinding::{PathfindingQueue, PathPriority, PathReason, PathRequested};
use crate::simulation::SimulationTick;

/// Bridge system that handles NeedsPathfinding action results
///
/// This system:
/// 1. Scans for ActionExecutionResult with NeedsPathfinding result
/// 2. Queues pathfinding request to PathfindingQueue
/// 3. Inserts PathRequested component on entity
/// 4. Transitions action internal state from NeedPath -> WaitingForPath
///
/// TECHNICAL NOTES:
/// - Uses Commands (not &World) to avoid parameter conflicts
/// - Runs in ActionExecution set, after execute_active_actions_read_only
/// - Runs before handle_action_results to ensure pathfinding is queued before result cleanup
pub fn bridge_actions_to_pathfinding(
    mut commands: Commands,
    mut pathfinding_queue: ResMut<PathfindingQueue>,
    tick: Res<SimulationTick>,
    mut query: Query<(Entity, &TilePosition, &mut ActiveAction, &ActionExecutionResult)>,
) {
    let current_tick = tick.0;

    let total_entities = query.iter().count();
    if total_entities > 0 {
        debug!("🔍 Bridge: Processing {} entities with ActionExecutionResult", total_entities);
    }

    for (entity, tile_pos, mut active_action, result_data) in query.iter_mut() {
        debug!("🔍 Bridge: Checking entity {:?} with action '{}', result: {:?}",
            entity, result_data.action_name, result_data.result);

        // Check if action result is NeedsPathfinding
        if let ActionResult::NeedsPathfinding { target } = result_data.result {
            let current_pos = tile_pos.tile;

            // Only queue if not already at target
            if current_pos == target {
                debug!(
                    "🗺️ Entity {:?} already at target {:?}, skipping pathfinding",
                    entity, target
                );
                continue;
            }

            // Determine appropriate reason based on action type
            let reason = match result_data.action_name.as_str() {
                "DrinkWater" => PathReason::MovingToWater,
                "Graze" => PathReason::MovingToFood,
                "Hunt" => PathReason::Hunting,
                "Wander" => PathReason::Wandering,
                "Mate" => PathReason::MovingToMate,
                _ => PathReason::Wandering, // Default fallback
            };

            // Queue pathfinding request with appropriate priority
            let priority = reason.default_priority();
            let request_id = pathfinding_queue.request_path(
                entity,
                current_pos,
                target,
                priority,
                reason,
                current_tick,
            );

            debug!(
                "🗺️ Bridge: Queued pathfinding for entity {:?} (action: '{}') from {:?} -> {:?}, request_id: {}, reason: {}",
                entity,
                result_data.action_name,
                current_pos,
                target,
                request_id.as_u64(),
                reason
            );

            // Insert PathRequested component to track the request
            commands.entity(entity).insert(PathRequested {
                request_id,
                target,
                priority,
                requested_tick: current_tick,
            });

            // Transition action state from NeedPath -> WaitingForPath
            // SAFETY: We need to mutate the action's internal state
            // The action is stored as a trait object, so we use unsafe to downcast
            // and transition the state machine.
            //
            // We check the action name to determine which state enum to use:
            transition_action_to_waiting(&mut active_action.action, request_id, &result_data.action_name);
        }
    }
}

/// Transition action state from NeedPath to WaitingForPath
///
/// This function uses unsafe downcast to access the concrete action type
/// and transition its internal state machine.
fn transition_action_to_waiting(
    action: &mut Box<dyn crate::ai::actions::Action>,
    request_id: crate::pathfinding::PathRequestId,
    action_name: &str,
) {
    use crate::ai::actions::{DrinkWaterAction, GrazeAction, HuntAction, MateAction, WanderAction};

    // Match action name to determine concrete type
    match action_name {
        "DrinkWater" => {
            if let Some(drink_action) = action.as_any_mut().downcast_mut::<DrinkWaterAction>() {
                drink_action.transition_to_waiting(request_id);
            }
        }
        "Graze" => {
            if let Some(graze_action) = action.as_any_mut().downcast_mut::<GrazeAction>() {
                graze_action.transition_to_waiting(request_id);
            }
        }
        "Hunt" => {
            if let Some(hunt_action) = action.as_any_mut().downcast_mut::<HuntAction>() {
                hunt_action.transition_to_waiting(request_id);
            }
        }
        "Wander" => {
            if let Some(wander_action) = action.as_any_mut().downcast_mut::<WanderAction>() {
                wander_action.transition_to_waiting(request_id);
            }
        }
        "Mate" => {
            if let Some(mate_action) = action.as_any_mut().downcast_mut::<MateAction>() {
                mate_action.transition_to_waiting(request_id);
            }
        }
        _ => {
            warn!(
                "🗺️ Bridge: Unknown action type '{}', cannot transition state",
                action_name
            );
        }
    }
}
