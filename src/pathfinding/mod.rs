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
mod jps;
mod path_request;
mod pathfinding_queue;
mod path_components;
mod path_to_movement_bridge;
mod region_map;

// Re-export existing pathfinding types from grid.rs
pub use grid::{
    find_path,
    pathfinding_cache_cleanup_system, process_pathfinding_requests, GridPathRequest, Path,
    PathCache, PathfindingFailed, PathfindingGrid,
};

// Re-export queue types (new Phase 1 additions)
pub use path_request::{PathFailureReason, PathReason, PathRequestId};
pub use pathfinding_queue::PathfindingQueue;

// Re-export JPS pathfinding (Phase 7: Performance optimization)
pub use jps::jps_find_path;

// Re-export path components (new Phase 2 additions)
pub use path_components::{PathRequested, PathReady, PathFailed};

// Re-export RegionMap for O(1) connectivity checks
pub use region_map::{RegionMap, build_region_map};

// Re-export PathReady → MovementComponent bridge

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
/// Includes negative path cache (Factorio pattern) to skip known-unreachable paths
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

    let current_tick = tick.0;
    let mut cache_hits = 0u32;
    let mut paths_computed = 0u32;

    // Process each path request
    for request in &requests {
        // Check negative cache first (Factorio pattern)
        if queue.is_known_unreachable(request.from, request.to, current_tick) {
            // Cache hit: immediately fail without running A*
            cache_hits += 1;
            queue.record_negative_cache_hit();

            commands.entity(request.entity).insert(PathFailed {
                reason: PathFailureReason::Unreachable,
                retry_count: 0,
            });
            commands.entity(request.entity).remove::<PathRequested>();

            debug!(
                "🗺️ Negative cache hit: {:?} → {:?} (entity {:?})",
                request.from, request.to, request.entity
            );
            continue;
        }

        // Compute path using existing A* algorithm
        paths_computed += 1;
        let path_opt = find_path(
            request.from,
            request.to,
            &grid,
            true, // Enable diagonal movement with corner-cutting prevention (15-20x speedup via JPS)
            Some(1500), // Increased from 800: Fragmented terrain needs longer paths even for short wander distances
        );

        // Insert appropriate component based on result
        match &path_opt {
            Some(path) => {
                // Success: insert PathReady component
                let waypoints = path.all_waypoints().to_vec();
                let cost = waypoints.len() as f32;

                commands.entity(request.entity).insert(PathReady {
                    path: std::sync::Arc::new(waypoints),
                    computed_tick: current_tick,
                    cost,
                });

                // Remove PathRequested component (path is ready)
                commands.entity(request.entity).remove::<PathRequested>();
            }
            None => {
                // Failure: add to negative cache and insert PathFailed component
                queue.add_unreachable(request.from, request.to, current_tick);

                commands.entity(request.entity).insert(PathFailed {
                    reason: PathFailureReason::Unreachable,
                    retry_count: 0,
                });

                // Remove PathRequested component (path failed)
                commands.entity(request.entity).remove::<PathRequested>();
            }
        }
    }

    // Periodic cache maintenance: evict expired entries every 100 ticks
    if current_tick % 100 == 0 {
        queue.evict_expired_entries(current_tick);
    }

    // Log metrics every 50 ticks
    if current_tick % 50 == 0 && (queue.total_queued() > 0 || cache_hits > 0) {
        let (urgent, normal, lazy) = queue.queue_sizes();
        let (cache_size, total_hits) = queue.negative_cache_stats();
        info!(
            "🗺️ PathfindingQueue: {} urgent, {} normal, {} lazy | Computed {}, CacheHits {} | NegCache: {} entries, {} total hits",
            urgent,
            normal,
            lazy,
            paths_computed,
            cache_hits,
            cache_size,
            total_hits
        );
    }
}

/// Condition: Only run on tick updates
fn should_tick(tick: Res<SimulationTick>) -> bool {
    tick.is_changed()
}
