/// Phase 1 UltraThink Queue Tests
/// RED PHASE: These tests should fail initially
use bevy::prelude::*;
use life_simulator::ai::ultrathink::{ThinkQueue, ThinkRequest, ThinkReason, ThinkPriority};
use life_simulator::simulation::SimulationTick;

#[test]
fn test_schedule_requests_to_correct_priority_queues() {
    // Test that requests are scheduled to the correct priority queues
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();

    // Spawn entities first (separate borrow scope)
    let entity_urgent = app.world_mut().spawn_empty().id();
    let entity_normal = app.world_mut().spawn_empty().id();
    let entity_low = app.world_mut().spawn_empty().id();

    // Then get queue and schedule (new borrow scope)
    {
        let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
        queue.schedule_urgent(entity_urgent, ThinkReason::FearTriggered, 1);
        queue.schedule_normal(entity_normal, ThinkReason::ActionCompleted, 1);
        queue.schedule_low(entity_low, ThinkReason::Idle, 1);
    }

    // Finally check results (new borrow scope)
    let queue = app.world_mut().resource::<ThinkQueue>();
    let (urgent, normal, low) = queue.queue_sizes();
    assert_eq!(urgent, 1, "Should have 1 urgent request");
    assert_eq!(normal, 1, "Should have 1 normal request");
    assert_eq!(low, 1, "Should have 1 low request");
}

#[test]
fn test_drain_respects_priority_order() {
    // Test that drain processes urgent → normal → low in correct order
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();

    // Spawn entities first
    let entity_low = app.world_mut().spawn_empty().id();
    let entity_urgent = app.world_mut().spawn_empty().id();
    let entity_normal = app.world_mut().spawn_empty().id();

    // Schedule in reverse priority order to test sorting
    {
        let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
        queue.schedule_low(entity_low, ThinkReason::Idle, 1);
        queue.schedule_urgent(entity_urgent, ThinkReason::FearTriggered, 1);
        queue.schedule_normal(entity_normal, ThinkReason::ActionCompleted, 1);
    }

    // Drain all 3 requests
    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
    let requests = queue.drain(3);

    assert_eq!(requests.len(), 3, "Should drain 3 requests");
    assert!(matches!(requests[0].priority, ThinkPriority::Urgent), "First should be urgent");
    assert!(matches!(requests[1].priority, ThinkPriority::Normal), "Second should be normal");
    assert!(matches!(requests[2].priority, ThinkPriority::Low), "Third should be low");
}

#[test]
fn test_drain_respects_budget_limit() {
    // Test that drain respects the budget limit
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();

    // Spawn 5 entities first
    let mut entities = Vec::new();
    for _i in 0..5 {
        entities.push(app.world_mut().spawn_empty().id());
    }

    // Schedule all 5
    {
        let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
        for entity in entities {
            queue.schedule_normal(entity, ThinkReason::ActionCompleted, 1);
        }
    }

    // Drain only 3
    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
    let requests = queue.drain(3);

    assert_eq!(requests.len(), 3, "Should only drain 3 requests despite 5 available");

    let (_, normal, _) = queue.queue_sizes();
    assert_eq!(normal, 2, "Should have 2 requests remaining");
}

#[test]
fn test_contains_detects_queued_entities() {
    // Test that contains() correctly detects if an entity is already queued
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();

    // Spawn entities first
    let entity_queued = app.world_mut().spawn_empty().id();
    let entity_not_queued = app.world_mut().spawn_empty().id();

    // Schedule and check
    {
        let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
        queue.schedule_normal(entity_queued, ThinkReason::ActionCompleted, 1);
    }

    let queue = app.world_mut().resource::<ThinkQueue>();
    assert!(queue.contains(entity_queued), "Should detect queued entity");
    assert!(!queue.contains(entity_not_queued), "Should not detect non-queued entity");
}

#[test]
fn test_queue_empty_returns_empty_vec() {
    // Test that draining an empty queue returns empty vec
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();

    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();

    let requests = queue.drain(10);

    assert_eq!(requests.len(), 0, "Empty queue should return empty vec");
}
