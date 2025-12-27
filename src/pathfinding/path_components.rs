/// Path Components - Component-based storage for pathfinding results
/// Replaces HashMap storage in PathfindingQueue (Phase 2 of ECS improvement)
/// Uses Arc<Vec<IVec2>> for cheap path cloning (Phase 3: Clone Reduction)
/// Arc is thread-safe (Send + Sync) for Bevy's parallel systems
use bevy::prelude::*;
use super::path_request::{PathFailureReason, PathPriority, PathRequestId};
use std::sync::Arc;

/// Component added when entity requests a path
/// Represents pending pathfinding request state
#[derive(Component, Clone, Debug)]
pub struct PathRequested {
    /// Unique identifier for this path request
    pub request_id: PathRequestId,

    /// Target destination tile
    pub target: IVec2,

    /// Priority level for processing (Urgent > Normal > Lazy)
    pub priority: PathPriority,

    /// Simulation tick when path was requested
    pub requested_tick: u64,
}

/// Component added when path computation succeeds
/// Represents a successfully computed path ready for use
/// Uses Arc for cheap cloning - only increments atomic reference count instead of copying entire Vec
#[derive(Component, Clone, Debug)]
pub struct PathReady {
    /// Computed path waypoints from start to goal
    /// Uses Arc for cheap cloning when multiple systems need the path
    pub path: Arc<Vec<IVec2>>,

    /// Simulation tick when path was computed
    pub computed_tick: u64,

    /// Total cost/distance of the path
    pub cost: f32,
}

/// Component added when path computation fails
/// Represents a failed pathfinding attempt with retry information
#[derive(Component, Clone, Debug)]
pub struct PathFailed {
    /// Reason for pathfinding failure
    pub reason: PathFailureReason,

    /// Number of retry attempts made
    pub retry_count: u32,
}

// =============================================================================
// COMPONENT LIFECYCLE NOTES
// =============================================================================
//
// Component State Machine:
// 1. Entity requests path → PathRequested component added
// 2. Path computation completes:
//    - Success: PathReady component added, PathRequested removed
//    - Failure: PathFailed component added, PathRequested removed
// 3. Entity despawns → All components automatically cleaned up by Bevy
//
// Change Detection:
// - Use Changed<PathReady> to react when paths become ready
// - Use Changed<PathFailed> to handle failures reactively
// - Replaces polling pattern: pf_queue.get_result(request_id)
//
// Benefits:
// - Automatic cleanup on entity despawn (no orphaned results)
// - Query-based access (no HashMap lookups)
// - Reactive systems with change detection
// - Component inspector visibility for debugging
