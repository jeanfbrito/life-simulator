/// PathfindingQueue Implementation
/// Following UltraThink proven patterns: priority queues, budget control, deduplication
use super::path_request::{PathPriority, PathReason, PathRequest, PathRequestId};
use super::path_components::PathRequested;
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

/// Global queue of pathfinding requests with priority-based processing
#[derive(Resource)]
pub struct PathfindingQueue {
    /// High priority queue - fleeing from predators
    urgent_queue: VecDeque<PathRequest>,

    /// Medium priority queue - moving to food/water/mates
    normal_queue: VecDeque<PathRequest>,

    /// Low priority queue - wandering exploration
    lazy_queue: VecDeque<PathRequest>,

    /// Processing budget per tick (default: 40-50)
    paths_per_tick: usize,

    /// Track pending requests for deduplication (entity, from, to)
    pending_requests: HashSet<(Entity, IVec2, IVec2)>,

    /// Counter for unique request IDs
    next_id: u64,

    /// Statistics
    total_paths_processed: u64,
}

impl Default for PathfindingQueue {
    fn default() -> Self {
        Self::new(40) // Default budget: 40 paths per tick
    }
}

impl PathfindingQueue {
    /// Create a new PathfindingQueue with specified budget
    pub fn new(paths_per_tick: usize) -> Self {
        Self {
            urgent_queue: VecDeque::new(),
            normal_queue: VecDeque::new(),
            lazy_queue: VecDeque::new(),
            paths_per_tick,
            pending_requests: HashSet::new(),
            next_id: 1,
            total_paths_processed: 0,
        }
    }

    /// Request a path to be computed
    /// Returns unique PathRequestId for retrieving result later
    /// Deduplicates requests for same (entity, from, to)
    ///
    /// Note: This method only queues the request. To insert the PathRequested component,
    /// use request_path_with_component() or manually insert the component after calling this.
    pub fn request_path(
        &mut self,
        entity: Entity,
        from: IVec2,
        to: IVec2,
        priority: PathPriority,
        reason: PathReason,
        requested_tick: u64,
    ) -> PathRequestId {
        let id = PathRequestId::new(self.next_id);
        self.next_id += 1;

        // Check for duplicate request
        let key = (entity, from, to);
        if !self.pending_requests.insert(key) {
            // Already queued - return new ID but don't queue again
            debug!(
                "🗺️ Deduplicating path request: entity={:?}, {:?} → {:?}",
                entity, from, to
            );
            return id;
        }

        // Create request
        let request = PathRequest::new(id, entity, from, to, priority, reason, requested_tick);

        // Queue based on priority
        match priority {
            PathPriority::Urgent => self.urgent_queue.push_back(request),
            PathPriority::Normal => self.normal_queue.push_back(request),
            PathPriority::Lazy => self.lazy_queue.push_back(request),
        }

        debug!(
            "🗺️ Queued path request {}: entity={:?}, {:?} → {:?}, priority={:?}, reason={}",
            id.as_u64(),
            entity,
            from,
            to,
            priority,
            reason
        );

        id
    }

    /// Request a path and insert PathRequested component on the entity
    /// This is the component-based version that should be preferred going forward
    pub fn request_path_with_component(
        &mut self,
        world: &mut World,
        entity: Entity,
        from: IVec2,
        to: IVec2,
        priority: PathPriority,
        reason: PathReason,
        requested_tick: u64,
    ) -> PathRequestId {
        let id = self.request_path(entity, from, to, priority, reason, requested_tick);

        // Insert PathRequested component
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(PathRequested {
                request_id: id,
                target: to,
                priority,
                requested_tick,
            });
        }

        id
    }

    /// Drain up to N requests from queues, prioritizing urgent → normal → lazy
    /// Returns vector of requests to process
    pub fn drain(&mut self, max_count: usize) -> Vec<PathRequest> {
        let mut requests = Vec::with_capacity(max_count);
        let mut processed = 0;

        // Process urgent queue first
        while processed < max_count {
            if let Some(request) = self.urgent_queue.pop_front() {
                self.pending_requests
                    .remove(&(request.entity, request.from, request.to));
                requests.push(request);
                processed += 1;
            } else {
                break;
            }
        }

        // Then normal queue
        while processed < max_count {
            if let Some(request) = self.normal_queue.pop_front() {
                self.pending_requests
                    .remove(&(request.entity, request.from, request.to));
                requests.push(request);
                processed += 1;
            } else {
                break;
            }
        }

        // Finally lazy queue
        while processed < max_count {
            if let Some(request) = self.lazy_queue.pop_front() {
                self.pending_requests
                    .remove(&(request.entity, request.from, request.to));
                requests.push(request);
                processed += 1;
            } else {
                break;
            }
        }

        self.total_paths_processed += requests.len() as u64;
        requests
    }

    /// Get queue sizes for monitoring (urgent, normal, lazy)
    pub fn queue_sizes(&self) -> (usize, usize, usize) {
        (
            self.urgent_queue.len(),
            self.normal_queue.len(),
            self.lazy_queue.len(),
        )
    }

    /// Get total number of queued requests
    pub fn total_queued(&self) -> usize {
        self.urgent_queue.len() + self.normal_queue.len() + self.lazy_queue.len()
    }

    /// Get paths per tick budget
    pub fn paths_per_tick(&self) -> usize {
        self.paths_per_tick
    }

    /// Get total paths processed (statistics)
    pub fn total_processed(&self) -> u64 {
        self.total_paths_processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_creation() {
        let queue = PathfindingQueue::new(40);
        assert_eq!(queue.paths_per_tick(), 40);
        assert_eq!(queue.total_queued(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut queue = PathfindingQueue::new(10);
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        // Add in reverse priority
        queue.request_path(
            entity3,
            IVec2::new(0, 0),
            IVec2::new(10, 10),
            PathPriority::Lazy,
            PathReason::Wandering,
            1,
        );
        queue.request_path(
            entity1,
            IVec2::new(0, 0),
            IVec2::new(5, 5),
            PathPriority::Urgent,
            PathReason::FleeingPredator,
            1,
        );
        queue.request_path(
            entity2,
            IVec2::new(0, 0),
            IVec2::new(7, 7),
            PathPriority::Normal,
            PathReason::MovingToFood,
            1,
        );

        let requests = queue.drain(3);

        assert_eq!(requests.len(), 3);
        assert!(matches!(requests[0].priority, PathPriority::Urgent));
        assert!(matches!(requests[1].priority, PathPriority::Normal));
        assert!(matches!(requests[2].priority, PathPriority::Lazy));
    }

    #[test]
    fn test_deduplication() {
        let mut queue = PathfindingQueue::new(10);
        let entity = Entity::from_raw(1);
        let from = IVec2::new(0, 0);
        let to = IVec2::new(10, 10);

        // Request same path twice
        queue.request_path(entity, from, to, PathPriority::Normal, PathReason::MovingToFood, 1);
        queue.request_path(entity, from, to, PathPriority::Normal, PathReason::MovingToFood, 2);

        // Should only have one request queued
        assert_eq!(queue.total_queued(), 1);
    }
}
