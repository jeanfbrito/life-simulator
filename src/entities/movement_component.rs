/// Movement State Component - Phase 3 ECS Improvement
///
/// Extracts movement logic from action state machines into a dedicated component.
/// This provides:
/// - Separation of concerns (movement separate from action logic)
/// - Reusability (any system can query movement state)
/// - Visibility (movement state in component inspector)
/// - Single source of truth for entity movement
///
/// Previous architecture embedded movement in action states:
/// ```rust
/// enum WanderState {
///     NeedPath,
///     WaitingForPath { request_id: PathRequestId },
///     Moving { path: Vec<IVec2>, current_index: usize },  // ❌ Embedded
/// }
/// ```
///
/// New architecture uses dedicated component:
/// ```rust
/// #[derive(Component)]
/// pub enum MovementComponent {
///     Idle,
///     PathRequested { request_id: PathRequestId },
///     FollowingPath { path: Vec<IVec2>, index: usize },  // ✅ Separate
///     Stuck { attempts: u32 },
/// }
/// ```

use bevy::prelude::*;
use crate::pathfinding::PathRequestId;
use std::sync::Arc;

/// Movement state component - tracks entity movement independently from actions
/// Uses Arc<Vec<IVec2>> for path to enable cheap cloning (Phase 3: Clone Reduction)
/// Arc is used instead of Rc for thread-safety (Bevy components must be Send + Sync)
///
/// Phase 4: Required Components
/// MovementComponent automatically requires TilePosition - compile-time guarantee
/// that any entity with movement also has a position.
#[derive(Component, Debug, Clone)]
#[require(crate::entities::TilePosition)]
pub enum MovementComponent {
    /// Entity is not moving (default state)
    Idle,

    /// Entity has requested a path and is waiting for pathfinding result
    PathRequested {
        request_id: PathRequestId,
    },

    /// Entity is following a computed path
    /// Uses Arc for cheap cloning - only increments atomic reference count instead of copying entire Vec
    FollowingPath {
        path: Arc<Vec<IVec2>>,
        index: usize,
    },

    /// Entity is stuck and cannot make progress
    Stuck {
        attempts: u32,
    },
}

impl Default for MovementComponent {
    fn default() -> Self {
        MovementComponent::Idle
    }
}

impl MovementComponent {
    /// Create a new idle movement state
    pub fn idle() -> Self {
        MovementComponent::Idle
    }

    /// Create a path requested state
    pub fn path_requested(request_id: PathRequestId) -> Self {
        MovementComponent::PathRequested { request_id }
    }

    /// Create a following path state
    pub fn following_path(path: Vec<IVec2>) -> Self {
        MovementComponent::FollowingPath {
            path: Arc::new(path),
            index: 0
        }
    }

    /// Create a stuck state
    pub fn stuck(attempts: u32) -> Self {
        MovementComponent::Stuck { attempts }
    }

    /// Check if entity is idle
    pub fn is_idle(&self) -> bool {
        matches!(self, MovementComponent::Idle)
    }

    /// Check if entity is waiting for a path
    pub fn is_path_requested(&self) -> bool {
        matches!(self, MovementComponent::PathRequested { .. })
    }

    /// Check if entity is following a path
    pub fn is_following_path(&self) -> bool {
        matches!(self, MovementComponent::FollowingPath { .. })
    }

    /// Check if entity is stuck
    pub fn is_stuck(&self) -> bool {
        matches!(self, MovementComponent::Stuck { .. })
    }

    /// Get the current path if following one
    pub fn get_path(&self) -> Option<&Arc<Vec<IVec2>>> {
        match self {
            MovementComponent::FollowingPath { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Get the current path index if following one
    pub fn get_path_index(&self) -> Option<usize> {
        match self {
            MovementComponent::FollowingPath { index, .. } => Some(*index),
            _ => None,
        }
    }

    /// Get the request ID if waiting for path
    pub fn get_request_id(&self) -> Option<PathRequestId> {
        match self {
            MovementComponent::PathRequested { request_id } => Some(*request_id),
            _ => None,
        }
    }

    /// Get stuck attempts if stuck
    pub fn get_stuck_attempts(&self) -> Option<u32> {
        match self {
            MovementComponent::Stuck { attempts } => Some(*attempts),
            _ => None,
        }
    }
}
