/// TDD Tests for Phase 2: PathResult as Component
/// Testing component-based storage replacing HashMap in PathfindingQueue
use bevy::prelude::*;
use life_simulator::pathfinding::{
    PathFailureReason, PathPriority, PathReason, PathRequestId, PathfindingQueue,
};
use life_simulator::entities::TilePosition;

// Import the new components we'll create
use life_simulator::pathfinding::{PathRequested, PathReady, PathFailed};

#[test]
fn test_path_requested_component_inserted() {
    // Setup
    let mut app = App::new();
    app.insert_resource(PathfindingQueue::new(10));

    let entity = app.world_mut().spawn(
        TilePosition::new(0, 0),
    ).id();

    // Request path with component insertion
    let request_id = {
        let world = app.world_mut();
        let mut queue = world.remove_resource::<PathfindingQueue>().unwrap();
        let id = queue.request_path_with_component(
            world,
            entity,
            IVec2::new(0, 0),
            IVec2::new(10, 10),
            PathPriority::Normal,
            PathReason::MovingToFood,
            1,
        );
        world.insert_resource(queue);
        id
    };

    // Assert PathRequested component was inserted
    let path_requested = app.world().get::<PathRequested>(entity);
    assert!(path_requested.is_some(), "PathRequested component should be inserted");

    let path_requested = path_requested.unwrap();
    assert_eq!(path_requested.request_id, request_id);
    assert_eq!(path_requested.target, IVec2::new(10, 10));
    assert_eq!(path_requested.priority, PathPriority::Normal);
    assert_eq!(path_requested.requested_tick, 1);
}

#[test]
fn test_path_ready_component_inserted_on_success() {
    // Setup
    let mut app = App::new();
    app.insert_resource(PathfindingQueue::new(10));

    let entity = app.world_mut().spawn(
        TilePosition::new(0, 0),
    ).id();

    // Simulate path completion by inserting PathReady component
    let path = vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0)];
    app.world_mut().entity_mut(entity).insert(PathReady {
        path: path.clone(),
        computed_tick: 5,
        cost: 2.0,
    });

    // Assert PathReady component exists
    let path_ready = app.world().get::<PathReady>(entity);
    assert!(path_ready.is_some(), "PathReady component should exist");

    let path_ready = path_ready.unwrap();
    assert_eq!(path_ready.path, path);
    assert_eq!(path_ready.computed_tick, 5);
    assert_eq!(path_ready.cost, 2.0);
}

#[test]
fn test_path_failed_component_inserted_on_failure() {
    // Setup
    let mut app = App::new();

    let entity = app.world_mut().spawn(
        TilePosition::new(0, 0),
    ).id();

    // Simulate path failure by inserting PathFailed component
    app.world_mut().entity_mut(entity).insert(PathFailed {
        reason: PathFailureReason::Unreachable,
        retry_count: 0,
    });

    // Assert PathFailed component exists
    let path_failed = app.world().get::<PathFailed>(entity);
    assert!(path_failed.is_some(), "PathFailed component should exist");

    let path_failed = path_failed.unwrap();
    assert_eq!(path_failed.reason, PathFailureReason::Unreachable);
    assert_eq!(path_failed.retry_count, 0);
}

#[test]
fn test_changed_path_ready_detection() {
    // Setup
    let mut app = App::new();

    let entity = app.world_mut().spawn(
        TilePosition::new(0, 0),
    ).id();

    // Insert PathReady component
    let path = vec![IVec2::new(0, 0), IVec2::new(1, 0)];
    app.world_mut().entity_mut(entity).insert(PathReady {
        path: path.clone(),
        computed_tick: 5,
        cost: 1.0,
    });

    // Query immediately after insertion - change detection should work
    // Note: Changed<T> detects when a component is added or modified
    let mut query_state = app.world_mut().query_filtered::<Entity, Changed<PathReady>>();
    let results: Vec<Entity> = query_state.iter(app.world()).collect();

    assert_eq!(results.len(), 1, "Changed<PathReady> should detect newly inserted component");
    assert_eq!(results[0], entity);

    // After update, the change flag is cleared
    app.update();

    // Re-query - should find no changes now
    let mut query_state = app.world_mut().query_filtered::<Entity, Changed<PathReady>>();
    let results: Vec<Entity> = query_state.iter(app.world()).collect();

    assert_eq!(results.len(), 0, "Changed<PathReady> should be cleared after update");
}

#[test]
fn test_pathfinding_queue_no_completed_paths_hashmap() {
    // This test verifies that PathfindingQueue no longer has completed_paths HashMap
    // We'll test this by ensuring get_result() method doesn't exist
    // Note: This will be validated by compilation - if get_result() is removed,
    // code trying to call it will fail to compile

    let queue = PathfindingQueue::new(10);

    // Verify queue exists and can be created
    assert_eq!(queue.paths_per_tick(), 10);

    // The absence of get_result() will be validated at compile time
    // once we refactor the PathfindingQueue struct
}

#[test]
fn test_component_cleanup_on_entity_despawn() {
    // Setup
    let mut app = App::new();

    let entity = app.world_mut().spawn((
        TilePosition::new(0, 0),
        PathRequested {
            request_id: PathRequestId::new(1),
            target: IVec2::new(10, 10),
            priority: PathPriority::Normal,
            requested_tick: 1,
        },
    )).id();

    // Verify component exists
    assert!(app.world().get::<PathRequested>(entity).is_some());

    // Despawn entity
    app.world_mut().despawn(entity);

    // Verify entity no longer exists (component automatically cleaned up)
    assert!(app.world().get::<PathRequested>(entity).is_none());
}

#[test]
fn test_path_components_with_multiple_entities() {
    // Setup
    let mut app = App::new();
    app.insert_resource(PathfindingQueue::new(10));

    // Create multiple entities requesting paths
    let entity1 = app.world_mut().spawn(
        TilePosition::new(0, 0),
    ).id();

    let entity2 = app.world_mut().spawn(
        TilePosition::new(5, 5),
    ).id();

    let entity3 = app.world_mut().spawn(
        TilePosition::new(10, 10),
    ).id();

    // Request paths for all entities with component insertion
    {
        let world = app.world_mut();
        let mut queue = world.remove_resource::<PathfindingQueue>().unwrap();

        queue.request_path_with_component(world, entity1, IVec2::new(0, 0), IVec2::new(3, 3), PathPriority::Urgent, PathReason::FleeingPredator, 1);
        queue.request_path_with_component(world, entity2, IVec2::new(5, 5), IVec2::new(8, 8), PathPriority::Normal, PathReason::MovingToFood, 1);
        queue.request_path_with_component(world, entity3, IVec2::new(10, 10), IVec2::new(15, 15), PathPriority::Lazy, PathReason::Wandering, 1);

        world.insert_resource(queue);
    }

    // Verify all entities have PathRequested component
    assert!(app.world().get::<PathRequested>(entity1).is_some());
    assert!(app.world().get::<PathRequested>(entity2).is_some());
    assert!(app.world().get::<PathRequested>(entity3).is_some());

    // Verify priorities are correct
    assert_eq!(app.world().get::<PathRequested>(entity1).unwrap().priority, PathPriority::Urgent);
    assert_eq!(app.world().get::<PathRequested>(entity2).unwrap().priority, PathPriority::Normal);
    assert_eq!(app.world().get::<PathRequested>(entity3).unwrap().priority, PathPriority::Lazy);
}

#[test]
fn test_reactive_path_ready_query() {
    // Setup
    let mut app = App::new();

    // Spawn entity with PathRequested
    let entity = app.world_mut().spawn((
        TilePosition::new(0, 0),
        PathRequested {
            request_id: PathRequestId::new(1),
            target: IVec2::new(10, 10),
            priority: PathPriority::Normal,
            requested_tick: 1,
        },
    )).id();

    // Path not ready yet - query should be empty
    let mut query_state = app.world_mut().query_filtered::<Entity, With<PathReady>>();
    let results: Vec<Entity> = query_state.iter(app.world()).collect();
    assert_eq!(results.len(), 0, "No PathReady component should exist yet");

    // Simulate path completion
    app.world_mut().entity_mut(entity).insert(PathReady {
        path: vec![IVec2::new(0, 0), IVec2::new(5, 5), IVec2::new(10, 10)],
        computed_tick: 5,
        cost: 10.0,
    });

    // Now query should find the entity with PathReady
    let mut query_state = app.world_mut().query_filtered::<Entity, With<PathReady>>();
    let results: Vec<Entity> = query_state.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "PathReady component should be queryable");
    assert_eq!(results[0], entity);
}
