/// Pathfinding module - A* pathfinding with queued processing
///
/// Phase 1: Core Queue Infrastructure (Complete)
/// - PathfindingQueue resource with priority-based processing
/// - Budget-controlled pathfinding (40-50 paths/tick)
/// - Result caching and deduplication
/// - Bevy plugin integration
///
/// Future: Multithreading support via Rayon (Phase 6)

mod grid;
mod path_request;
mod pathfinding_queue;
mod path_components;
mod path_to_movement_bridge;

// Re-export existing pathfinding types from grid.rs
pub use grid::{
    build_pathfinding_grid_from_world, find_path, find_path_with_cache,
    pathfinding_cache_cleanup_system, process_pathfinding_requests,
    terrain_to_pathfinding_cost, update_pathfinding_grid_for_tile, GridPathRequest, Path,
    PathCache, PathNode, PathfindingFailed, PathfindingGrid,
};

// Re-export queue types (new Phase 1 additions)
pub use path_request::{PathFailureReason, PathPriority, PathReason, PathRequest, PathRequestId, PathResult};
pub use pathfinding_queue::PathfindingQueue;

// Re-export path components (new Phase 2 additions)
pub use path_components::{PathRequested, PathReady, PathFailed};

// Re-export PathReady → MovementComponent bridge
pub use path_to_movement_bridge::bridge_path_ready_to_movement;

// Bevy plugin and system
use bevy::prelude::*;
use crate::simulation::{profiler::ScopedTimer, SimulationTick, TickProfiler};

/// PathfindingQueue Bevy plugin
/// Integrates priority-based pathfinding queue into the simulation
pub struct PathfindingQueuePlugin;

impl Plugin for PathfindingQueuePlugin {
    fn build(&self, app: &mut App) {
        println!("✅ PathfindingQueuePlugin initialized");
        // Insert PathfindingQueue resource with default budget (40 paths/tick)
        app.insert_resource(PathfindingQueue::default());

        // CRITICAL FIX: Move to Update schedule to sync with AI systems
        // AI systems (bridge_actions_to_pathfinding) run in Update,
        // so pathfinding must also run in Update to ensure same-frame processing
        app.add_systems(
            Update,
            (
                process_pathfinding_queue,
                path_to_movement_bridge::bridge_path_ready_to_movement.after(process_pathfinding_queue),
            )
                .in_set(crate::simulation::SimulationSet::Movement)
                .run_if(should_tick),
        );
    }
}

/// System: Process pathfinding requests from the queue
/// Following UltraThink pattern: budget-controlled, priority-based processing
/// Phase 2: Inserts PathReady/PathFailed components instead of HashMap storage
pub fn process_pathfinding_queue(
    mut queue: ResMut<PathfindingQueue>,
    grid: Res<PathfindingGrid>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
    mut commands: Commands,
) {
    let _timer = ScopedTimer::new(&mut profiler, "pathfinding_queue");

    let budget = queue.paths_per_tick();
    let requests = queue.drain(budget);

    if requests.is_empty() {
        return;
    }

    // Process each path request
    for request in &requests {
        // Compute path using existing A* algorithm
        let path_opt = find_path(
            request.from,
            request.to,
            &grid,
            false, // No diagonal movement
            Some(5000), // Max steps for long paths (wander_radius up to 50 tiles)
        );

        // Insert appropriate component based on result
        match &path_opt {
            Some(path) => {
                // Success: insert PathReady component
                let waypoints = path.all_waypoints().to_vec();
                let cost = waypoints.len() as f32;

                commands.entity(request.entity).insert(PathReady {
                    path: std::sync::Arc::new(waypoints),
                    computed_tick: tick.0,
                    cost,
                });

                // Remove PathRequested component (path is ready)
                commands.entity(request.entity).remove::<PathRequested>();
            }
            None => {
                // Failure: insert PathFailed component
                commands.entity(request.entity).insert(PathFailed {
                    reason: PathFailureReason::Unreachable,
                    retry_count: 0,
                });

                // Remove PathRequested component (path failed)
                commands.entity(request.entity).remove::<PathRequested>();
            }
        }
    }

    // Log metrics every 50 ticks
    if tick.0 % 50 == 0 && queue.total_queued() > 0 {
        let (urgent, normal, lazy) = queue.queue_sizes();
        info!(
            "🗺️ PathfindingQueue: {} urgent, {} normal, {} lazy | Processed {}/{} | Total: {}",
            urgent,
            normal,
            lazy,
            requests.len(),
            budget,
            queue.total_processed()
        );
    }
}

/// Condition: Only run on tick updates
fn should_tick(tick: Res<SimulationTick>) -> bool {
    tick.is_changed()
}
