/// Integration tests for PathfindingQueue (TDD - Phase 1)
/// Following UltraThink proven patterns: priority queues, budget control, deduplication
use bevy::prelude::*;
use life_simulator::pathfinding::{
    PathPriority, PathReason, PathfindingQueue,
};

#[test]
fn test_queue_creation() {
    let queue = PathfindingQueue::new(40);
    assert_eq!(queue.paths_per_tick(), 40);
    assert_eq!(queue.total_queued(), 0);
    assert_eq!(queue.queue_sizes(), (0, 0, 0));
}

#[test]
fn test_priority_ordering() {
    let mut queue = PathfindingQueue::new(10);
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);

    // Add in reverse priority order
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

    // Drain all requests
    let requests = queue.drain(3);

    // Should be ordered: Urgent -> Normal -> Lazy
    assert_eq!(requests.len(), 3);
    assert!(matches!(requests[0].priority, PathPriority::Urgent));
    assert!(matches!(requests[1].priority, PathPriority::Normal));
    assert!(matches!(requests[2].priority, PathPriority::Lazy));
}

#[test]
fn test_budget_limits() {
    let mut queue = PathfindingQueue::new(2); // Budget: only 2 paths per tick
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);

    // Add 3 requests
    queue.request_path(
        entity1,
        IVec2::new(0, 0),
        IVec2::new(5, 5),
        PathPriority::Normal,
        PathReason::MovingToFood,
        1,
    );
    queue.request_path(
        entity2,
        IVec2::new(1, 1),
        IVec2::new(6, 6),
        PathPriority::Normal,
        PathReason::MovingToWater,
        1,
    );
    queue.request_path(
        entity3,
        IVec2::new(2, 2),
        IVec2::new(7, 7),
        PathPriority::Normal,
        PathReason::Wandering,
        1,
    );

    assert_eq!(queue.total_queued(), 3);

    // Drain with budget limit
    let batch1 = queue.drain(2);
    assert_eq!(batch1.len(), 2); // Only 2 returned due to budget
    assert_eq!(queue.total_queued(), 1); // 1 still queued

    // Drain again
    let batch2 = queue.drain(2);
    assert_eq!(batch2.len(), 1); // Remaining request
    assert_eq!(queue.total_queued(), 0);
}

#[test]
fn test_deduplication() {
    let mut queue = PathfindingQueue::new(10);
    let entity = Entity::from_raw(1);
    let from = IVec2::new(0, 0);
    let to = IVec2::new(10, 10);

    // Request same path twice
    let id1 = queue.request_path(
        entity,
        from,
        to,
        PathPriority::Normal,
        PathReason::MovingToFood,
        1,
    );
    let id2 = queue.request_path(
        entity,
        from,
        to,
        PathPriority::Normal,
        PathReason::MovingToFood,
        2,
    );

    // Should be deduplicated - only one request queued
    assert_eq!(queue.total_queued(), 1);

    // IDs should be different (second is new allocation even if deduplicated)
    assert_ne!(id1, id2);
}

#[test]
fn test_queue_sizes() {
    let mut queue = PathfindingQueue::new(10);
    let entity1 = Entity::from_raw(1);
    let entity2 = Entity::from_raw(2);
    let entity3 = Entity::from_raw(3);

    // Add to different priority queues
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
    queue.request_path(
        entity3,
        IVec2::new(0, 0),
        IVec2::new(10, 10),
        PathPriority::Lazy,
        PathReason::Wandering,
        1,
    );

    let (urgent, normal, lazy) = queue.queue_sizes();
    assert_eq!(urgent, 1);
    assert_eq!(normal, 1);
    assert_eq!(lazy, 1);
    assert_eq!(queue.total_queued(), 3);
}

#[test]
fn test_request_id_uniqueness() {
    let mut queue = PathfindingQueue::new(10);
    let entity = Entity::from_raw(1);

    let id1 = queue.request_path(
        entity,
        IVec2::new(0, 0),
        IVec2::new(5, 5),
        PathPriority::Normal,
        PathReason::MovingToFood,
        1,
    );
    let id2 = queue.request_path(
        entity,
        IVec2::new(5, 5),
        IVec2::new(10, 10),
        PathPriority::Normal,
        PathReason::MovingToWater,
        1,
    );

    // Different requests should have unique IDs
    assert_ne!(id1, id2);
}
