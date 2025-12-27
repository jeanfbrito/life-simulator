/// Integration tests for UltraThink -> AI Planning connection
/// Tests that ThinkQueue requests actually trigger replanning
use bevy::prelude::*;
use life_simulator::ai::event_driven_planner::NeedsReplanning;
use life_simulator::ai::ultrathink::{ThinkQueue, ThinkReason, ultrathink_system};
use life_simulator::simulation::{SimulationTick, TickProfiler};

/// Test 1: UltraThink processes urgent requests and marks entities for replanning
#[test]
fn test_ultrathink_marks_entities_for_replanning() {
    let mut app = App::new();

    // Setup minimal world
    app.insert_resource(ThinkQueue::new(10));
    app.insert_resource(SimulationTick(1));
    app.insert_resource(TickProfiler::default());

    // Create test entity
    let entity = app.world_mut().spawn_empty().id();

    // Schedule urgent think request
    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
    queue.schedule_urgent(entity, ThinkReason::FearTriggered, 1);

    // Run ultrathink system
    app.add_systems(Update, ultrathink_system);
    app.update();

    // Verify entity was marked for replanning
    let has_marker = app.world().get::<NeedsReplanning>(entity).is_some();
    assert!(has_marker, "Entity should have NeedsReplanning component after UltraThink processing");
}

/// Test 2: UltraThink respects priority ordering
#[test]
fn test_ultrathink_processes_by_priority() {
    let mut app = App::new();

    app.insert_resource(ThinkQueue::new(2)); // Limited budget
    app.insert_resource(SimulationTick(1));
    app.insert_resource(TickProfiler::default());

    let entity_urgent = app.world_mut().spawn_empty().id();
    let entity_normal = app.world_mut().spawn_empty().id();
    let entity_low = app.world_mut().spawn_empty().id();

    // Schedule in reverse priority order
    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
    queue.schedule_low(entity_low, ThinkReason::Idle, 1);
    queue.schedule_normal(entity_normal, ThinkReason::ActionCompleted, 1);
    queue.schedule_urgent(entity_urgent, ThinkReason::FearTriggered, 1);

    // Run system with limited budget (only 2 should process)
    app.add_systems(Update, ultrathink_system);
    app.update();

    // Urgent and normal should be marked, low should not
    assert!(app.world().get::<NeedsReplanning>(entity_urgent).is_some(), "Urgent should be processed");
    assert!(app.world().get::<NeedsReplanning>(entity_normal).is_some(), "Normal should be processed");
    assert!(app.world().get::<NeedsReplanning>(entity_low).is_none(), "Low should not be processed (budget exhausted)");
}

/// Test 3: UltraThink includes reason in NeedsReplanning
#[test]
fn test_ultrathink_includes_reason() {
    let mut app = App::new();

    app.insert_resource(ThinkQueue::new(10));
    app.insert_resource(SimulationTick(1));
    app.insert_resource(TickProfiler::default());

    let entity = app.world_mut().spawn_empty().id();

    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
    queue.schedule_urgent(entity, ThinkReason::FearTriggered, 1);

    app.add_systems(Update, ultrathink_system);
    app.update();

    // Verify reason is included
    if let Some(marker) = app.world().get::<NeedsReplanning>(entity) {
        assert!(marker.reason.contains("Fear"), "Reason should reference fear trigger");
    } else {
        panic!("Entity should have NeedsReplanning marker");
    }
}

/// Test 4: UltraThink handles empty queue gracefully
#[test]
fn test_ultrathink_handles_empty_queue() {
    let mut app = App::new();

    app.insert_resource(ThinkQueue::new(10));
    app.insert_resource(SimulationTick(1));
    app.insert_resource(TickProfiler::default());

    // Don't add any requests
    app.add_systems(Update, ultrathink_system);
    app.update();

    // Should complete without panic
    // No entities to check, just verify system runs
}

/// Test 5: UltraThink processes multiple entities in single tick
#[test]
fn test_ultrathink_batch_processing() {
    let mut app = App::new();

    app.insert_resource(ThinkQueue::new(50));
    app.insert_resource(SimulationTick(1));
    app.insert_resource(TickProfiler::default());

    // Create multiple entities
    let entities: Vec<Entity> = (0..10)
        .map(|_| app.world_mut().spawn_empty().id())
        .collect();

    // Schedule all for thinking
    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
    for (i, &entity) in entities.iter().enumerate() {
        let reason = if i % 3 == 0 {
            ThinkReason::FearTriggered
        } else if i % 3 == 1 {
            ThinkReason::ActionCompleted
        } else {
            ThinkReason::Idle
        };

        if i % 3 == 0 {
            queue.schedule_urgent(entity, reason, 1);
        } else {
            queue.schedule_normal(entity, reason, 1);
        }
    }

    app.add_systems(Update, ultrathink_system);
    app.update();

    // All should be marked
    let marked_count = entities.iter()
        .filter(|&&e| app.world().get::<NeedsReplanning>(e).is_some())
        .count();

    assert_eq!(marked_count, 10, "All 10 entities should be marked for replanning");
}

/// Test 6: UltraThink handles despawned entities gracefully
#[test]
fn test_ultrathink_handles_despawned_entities() {
    let mut app = App::new();

    app.insert_resource(ThinkQueue::new(10));
    app.insert_resource(SimulationTick(1));
    app.insert_resource(TickProfiler::default());

    // Create entity and schedule it
    let entity = app.world_mut().spawn_empty().id();

    let mut queue = app.world_mut().resource_mut::<ThinkQueue>();
    queue.schedule_urgent(entity, ThinkReason::FearTriggered, 1);
    drop(queue);

    // Despawn the entity before processing
    app.world_mut().despawn(entity);

    // Run system - should handle despawned entity gracefully
    app.add_systems(Update, ultrathink_system);
    app.update();

    // Should not panic - test passes if we reach this point
    assert!(true, "System should handle despawned entities without panic");
}
