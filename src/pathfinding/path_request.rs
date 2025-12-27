/// Path Request Types for PathfindingQueue System
/// Following UltraThink patterns: priority-based requests, result caching
use bevy::prelude::*;
use std::fmt;

/// Unique identifier for a path request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathRequestId(u64);

impl PathRequestId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Priority level for pathfinding requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathPriority {
    /// Process within 1-2 ticks - fleeing from predators (life-threatening)
    Urgent,
    /// Process within 3-5 ticks - moving to food/water (needs-based)
    Normal,
    /// Process within 10-20 ticks - wandering exploration (idle activity)
    Lazy,
}

/// Reason why a path is needed (for debugging and metrics)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathReason {
    // Urgent reasons
    FleeingPredator,

    // Normal priority reasons
    MovingToFood,
    MovingToWater,
    MovingToMate,
    Hunting,

    // Lazy priority reasons
    Wandering,
}

impl PathReason {
    /// Get the default priority for this reason
    pub fn default_priority(&self) -> PathPriority {
        match self {
            PathReason::FleeingPredator => PathPriority::Urgent,

            PathReason::MovingToFood
            | PathReason::MovingToWater
            | PathReason::MovingToMate
            | PathReason::Hunting => PathPriority::Normal,

            PathReason::Wandering => PathPriority::Lazy,
        }
    }
}

impl fmt::Display for PathReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathReason::FleeingPredator => write!(f, "FleeingPredator"),
            PathReason::MovingToFood => write!(f, "MovingToFood"),
            PathReason::MovingToWater => write!(f, "MovingToWater"),
            PathReason::MovingToMate => write!(f, "MovingToMate"),
            PathReason::Hunting => write!(f, "Hunting"),
            PathReason::Wandering => write!(f, "Wandering"),
        }
    }
}

/// A pathfinding request in the queue
#[derive(Debug, Clone)]
pub struct PathRequest {
    pub id: PathRequestId,
    pub entity: Entity,
    pub from: IVec2,
    pub to: IVec2,
    pub priority: PathPriority,
    pub reason: PathReason,
    pub requested_tick: u64,
}

impl PathRequest {
    pub fn new(
        id: PathRequestId,
        entity: Entity,
        from: IVec2,
        to: IVec2,
        priority: PathPriority,
        reason: PathReason,
        requested_tick: u64,
    ) -> Self {
        Self {
            id,
            entity,
            from,
            to,
            priority,
            reason,
            requested_tick,
        }
    }
}

/// Result of a pathfinding computation
#[derive(Debug, Clone)]
pub enum PathResult {
    Success {
        path: Vec<IVec2>,
        computed_tick: u64,
    },
    Failed {
        reason: PathFailureReason,
        retry_count: u32,
    },
}

/// Why a pathfinding request failed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathFailureReason {
    Unreachable,
    Timeout,
    InvalidStart,
    InvalidGoal,
}

impl fmt::Display for PathFailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathFailureReason::Unreachable => write!(f, "Unreachable"),
            PathFailureReason::Timeout => write!(f, "Timeout"),
            PathFailureReason::InvalidStart => write!(f, "InvalidStart"),
            PathFailureReason::InvalidGoal => write!(f, "InvalidGoal"),
        }
    }
}
