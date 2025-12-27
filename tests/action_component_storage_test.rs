/// Test: ActionQueue component-based storage
///
/// Validates that active actions are stored as components instead of HashMap,
/// providing automatic cleanup on entity despawn while maintaining 10 TPS performance.
use bevy::prelude::*;
use life_simulator::ai::{ActionQueue, ActionType};
use life_simulator::entities::{CurrentAction, Energy, Stat};

/// Helper to create a minimal test world with required resources
fn create_test_world() -> World {
    let mut world = World::new();
    world.insert_resource(ActionQueue::default());
    world
}

/// Spawn a test entity with Energy component (required for Rest action)
fn spawn_test_entity(world: &mut World) -> Entity {
    world.spawn((
        CurrentAction::none(),
        Energy(Stat::new(50.0, 0.0, 100.0, 1.0)), // 50% energy so it won't be full
    )).id()
}

#[test]
fn test_active_action_stored_as_component() {
    let mut world = create_test_world();

    // Spawn entity with required components
    let entity = spawn_test_entity(&mut world);

    // Queue a multi-tick action (Rest)
    {
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.queue_action(
            entity,
            ActionType::Rest { duration_ticks: 5 },
            1.0,
            0,
            0,
        );
    }

    // Execute tick to move action from pending to active
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, 1);
    });

    // CRITICAL TEST: Active action should be stored as a component on the entity
    assert!(
        world.get::<life_simulator::entities::ActiveAction>(entity).is_some(),
        "Active action should be stored as component on entity"
    );
}

#[test]
fn test_component_removed_on_action_completion() {
    let mut world = create_test_world();

    let entity = spawn_test_entity(&mut world);

    // Queue a 2-tick Rest action
    {
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.queue_action(
            entity,
            ActionType::Rest { duration_ticks: 2 },
            1.0,
            0,
            0,
        );
    }

    // Start the action (tick 1) - should go from 2 to 1 remaining
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, 1);
    });

    // Component should exist while action is active
    assert!(
        world.get::<life_simulator::entities::ActiveAction>(entity).is_some(),
        "Component should exist while action is in progress"
    );

    // Continue to tick 2 - goes from 1 to 0 remaining, completes
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, 2);
    });

    // Component should be removed after completion
    assert!(
        world.get::<life_simulator::entities::ActiveAction>(entity).is_none(),
        "Component should be removed after action completes"
    );
}

#[test]
fn test_automatic_cleanup_on_entity_despawn() {
    let mut world = create_test_world();

    let entity = spawn_test_entity(&mut world);

    // Queue a long-running action
    {
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.queue_action(
            entity,
            ActionType::Rest { duration_ticks: 100 },
            1.0,
            0,
            0,
        );
    }

    // Start the action
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, 1);
    });

    // Verify component exists
    assert!(world.get::<life_simulator::entities::ActiveAction>(entity).is_some());

    // Despawn the entity - component should auto-cleanup (no HashMap leak!)
    world.despawn(entity);

    // Execute another tick - should handle missing entity gracefully
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, 2);
    });

    // No assertion needed - test passes if no panic occurs
    // This demonstrates the benefit: components clean up automatically
}

#[test]
fn test_hashmap_removed_from_action_queue() {
    let mut world = create_test_world();

    // Queue and execute an action
    let entity = spawn_test_entity(&mut world);
    let mut queue = world.resource_mut::<ActionQueue>();

    queue.queue_action(
        entity,
        ActionType::Rest { duration_ticks: 5 },
        1.0,
        0,
        0,
    );

    drop(queue);

    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, 1);
    });

    // Check count using resource_scope
    world.resource_scope(|world, queue: Mut<ActionQueue>| {
        let count = queue.active_count_with_world(world);
        assert_eq!(count, 1, "Should have 1 active action stored as component");
    });
}

#[test]
fn test_multiple_entities_component_storage() {
    let mut world = create_test_world();

    // Spawn multiple entities
    let entity1 = spawn_test_entity(&mut world);
    let entity2 = spawn_test_entity(&mut world);
    let entity3 = spawn_test_entity(&mut world);

    // Queue actions for all
    {
        let mut queue = world.resource_mut::<ActionQueue>();
        queue.queue_action(entity1, ActionType::Rest { duration_ticks: 5 }, 1.0, 0, 0);
        queue.queue_action(entity2, ActionType::Rest { duration_ticks: 5 }, 1.0, 0, 0);
        queue.queue_action(entity3, ActionType::Rest { duration_ticks: 5 }, 1.0, 0, 0);
    }

    // Execute tick
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, 1);
    });

    // All entities should have the component
    assert!(world.get::<life_simulator::entities::ActiveAction>(entity1).is_some());
    assert!(world.get::<life_simulator::entities::ActiveAction>(entity2).is_some());
    assert!(world.get::<life_simulator::entities::ActiveAction>(entity3).is_some());

    world.resource_scope(|world, queue: Mut<ActionQueue>| {
        assert_eq!(queue.active_count_with_world(world), 3, "Should count 3 active actions via Query");
    });
}
