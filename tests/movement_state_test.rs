/// TDD Test Suite for Phase 3: Movement State as Component
///
/// These tests define the expected behavior of the new MovementState component
/// that extracts movement logic from action state machines.
///
/// RED PHASE: These tests should FAIL until implementation is complete

use bevy::prelude::*;
use life_simulator::entities::{MovementComponent, TilePosition};
use life_simulator::pathfinding::PathRequestId;

#[test]
fn test_movement_component_exists() {
    // RED: MovementComponent doesn't exist yet
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn((
        TilePosition::new(0, 0),
        MovementComponent::Idle,
    )).id();

    let movement = app.world().get::<MovementComponent>(entity);
    assert!(movement.is_some(), "MovementComponent should be a valid component");
}

#[test]
fn test_movement_component_states() {
    // RED: Test that MovementComponent has all required states
    let idle = MovementComponent::Idle;
    let path_requested = MovementComponent::PathRequested {
        request_id: PathRequestId::new(42),
    };
    let following_path = MovementComponent::FollowingPath {
        path: vec![IVec2::new(1, 1), IVec2::new(2, 2)],
        index: 0,
    };
    let stuck = MovementComponent::Stuck { attempts: 3 };

    // Just ensure these variants compile
    match idle {
        MovementComponent::Idle => {},
        _ => panic!("Should be Idle"),
    }

    match path_requested {
        MovementComponent::PathRequested { request_id } => {
            assert_eq!(request_id.as_u64(), 42);
        },
        _ => panic!("Should be PathRequested"),
    }

    match following_path {
        MovementComponent::FollowingPath { path, index } => {
            assert_eq!(path.len(), 2);
            assert_eq!(index, 0);
        },
        _ => panic!("Should be FollowingPath"),
    }

    match stuck {
        MovementComponent::Stuck { attempts } => {
            assert_eq!(attempts, 3);
        },
        _ => panic!("Should be Stuck"),
    }
}

#[test]
fn test_movement_component_insertion() {
    // RED: Test that MovementComponent can be inserted on entities
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn((
        TilePosition::new(5, 5),
        MovementComponent::Idle,
    )).id();

    // Change state to PathRequested
    app.world_mut().entity_mut(entity).insert(MovementComponent::PathRequested {
        request_id: PathRequestId::new(100),
    });

    let movement = app.world().get::<MovementComponent>(entity).unwrap();
    match movement {
        MovementComponent::PathRequested { request_id } => {
            assert_eq!(request_id.as_u64(), 100);
        },
        _ => panic!("Should be PathRequested"),
    }
}

#[test]
fn test_movement_component_query() {
    // RED: Test that MovementComponent can be queried
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn 3 entities with different movement states
    let entity1 = app.world_mut().spawn((
        TilePosition::new(0, 0),
        MovementComponent::Idle,
    )).id();

    let entity2 = app.world_mut().spawn((
        TilePosition::new(1, 1),
        MovementComponent::PathRequested {
            request_id: PathRequestId::new(1),
        },
    )).id();

    let entity3 = app.world_mut().spawn((
        TilePosition::new(2, 2),
        MovementComponent::FollowingPath {
            path: vec![IVec2::new(3, 3)],
            index: 0,
        },
    )).id();

    // Query all entities with MovementComponent
    let mut query = app.world_mut().query::<(Entity, &MovementComponent)>();
    let results: Vec<_> = query.iter(app.world()).collect();

    assert_eq!(results.len(), 3, "Should have 3 entities with MovementComponent");

    // Verify we can query specific states
    let mut following_query = app.world_mut().query_filtered::<Entity, With<MovementComponent>>();
    let count = following_query.iter(app.world()).count();
    assert_eq!(count, 3, "Should find all entities with MovementComponent");
}

#[test]
fn test_movement_component_default_to_idle() {
    // RED: Test that new entities default to Idle state
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn((
        TilePosition::new(10, 10),
        MovementComponent::Idle,
    )).id();

    let movement = app.world().get::<MovementComponent>(entity).unwrap();
    assert!(matches!(movement, MovementComponent::Idle), "New entities should start Idle");
}

#[test]
fn test_movement_component_path_progression() {
    // RED: Test that path index can progress
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let path = vec![
        IVec2::new(1, 1),
        IVec2::new(2, 2),
        IVec2::new(3, 3),
    ];

    let entity = app.world_mut().spawn((
        TilePosition::new(0, 0),
        MovementComponent::FollowingPath {
            path: path.clone(),
            index: 0,
        },
    )).id();

    // Simulate progression
    app.world_mut().entity_mut(entity).insert(MovementComponent::FollowingPath {
        path: path.clone(),
        index: 1,
    });

    let movement = app.world().get::<MovementComponent>(entity).unwrap();
    match movement {
        MovementComponent::FollowingPath { path: _, index } => {
            assert_eq!(*index, 1, "Index should have progressed to 1");
        },
        _ => panic!("Should be FollowingPath"),
    }

    // Simulate completion
    app.world_mut().entity_mut(entity).insert(MovementComponent::FollowingPath {
        path: path.clone(),
        index: 3,
    });

    let movement = app.world().get::<MovementComponent>(entity).unwrap();
    match movement {
        MovementComponent::FollowingPath { path, index } => {
            assert_eq!(*index, 3, "Index should be at end");
            assert!(index >= &path.len(), "Index should indicate path completion");
        },
        _ => panic!("Should be FollowingPath"),
    }
}

#[test]
fn test_movement_component_stuck_retry() {
    // RED: Test stuck state retry mechanism
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn((
        TilePosition::new(5, 5),
        MovementComponent::Stuck { attempts: 0 },
    )).id();

    // Increment retry attempts
    for i in 1..=3 {
        app.world_mut().entity_mut(entity).insert(MovementComponent::Stuck {
            attempts: i,
        });

        let movement = app.world().get::<MovementComponent>(entity).unwrap();
        match movement {
            MovementComponent::Stuck { attempts } => {
                assert_eq!(*attempts, i, "Attempts should be {}", i);
            },
            _ => panic!("Should be Stuck"),
        }
    }
}

#[test]
fn test_movement_component_state_transitions() {
    // RED: Test typical state transition flow
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn((
        TilePosition::new(0, 0),
        MovementComponent::Idle,
    )).id();

    // Idle -> PathRequested
    app.world_mut().entity_mut(entity).insert(MovementComponent::PathRequested {
        request_id: PathRequestId::new(5),
    });

    let movement = app.world().get::<MovementComponent>(entity).unwrap();
    assert!(matches!(movement, MovementComponent::PathRequested { .. }), "Should transition to PathRequested");

    // PathRequested -> FollowingPath
    app.world_mut().entity_mut(entity).insert(MovementComponent::FollowingPath {
        path: vec![IVec2::new(1, 1)],
        index: 0,
    });

    let movement = app.world().get::<MovementComponent>(entity).unwrap();
    assert!(matches!(movement, MovementComponent::FollowingPath { .. }), "Should transition to FollowingPath");

    // FollowingPath -> Idle (when complete)
    app.world_mut().entity_mut(entity).insert(MovementComponent::Idle);

    let movement = app.world().get::<MovementComponent>(entity).unwrap();
    assert!(matches!(movement, MovementComponent::Idle), "Should transition back to Idle");
}

#[test]
fn test_movement_component_cleanup_on_despawn() {
    // RED: Test that MovementComponent is cleaned up when entity despawns
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity = app.world_mut().spawn((
        TilePosition::new(0, 0),
        MovementComponent::FollowingPath {
            path: vec![IVec2::new(1, 1)],
            index: 0,
        },
    )).id();

    // Verify entity exists
    assert!(app.world().get_entity(entity).is_ok(), "Entity should exist");

    // Despawn entity
    app.world_mut().despawn(entity);

    // Verify entity is gone (component automatically cleaned up)
    assert!(app.world().get_entity(entity).is_err(), "Entity should be despawned");
}

#[test]
fn test_movement_component_multiple_entities_independent() {
    // RED: Test that multiple entities can have independent MovementComponent states
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let entity1 = app.world_mut().spawn((
        TilePosition::new(0, 0),
        MovementComponent::Idle,
    )).id();

    let entity2 = app.world_mut().spawn((
        TilePosition::new(5, 5),
        MovementComponent::FollowingPath {
            path: vec![IVec2::new(6, 6)],
            index: 0,
        },
    )).id();

    let entity3 = app.world_mut().spawn((
        TilePosition::new(10, 10),
        MovementComponent::PathRequested {
            request_id: PathRequestId::new(99),
        },
    )).id();

    // Verify all states are independent
    let m1 = app.world().get::<MovementComponent>(entity1).unwrap();
    let m2 = app.world().get::<MovementComponent>(entity2).unwrap();
    let m3 = app.world().get::<MovementComponent>(entity3).unwrap();

    assert!(matches!(m1, MovementComponent::Idle));
    assert!(matches!(m2, MovementComponent::FollowingPath { .. }));
    assert!(matches!(m3, MovementComponent::PathRequested { .. }));

    // Change one entity's state
    app.world_mut().entity_mut(entity1).insert(MovementComponent::Stuck { attempts: 1 });

    // Verify others are unaffected
    let m1 = app.world().get::<MovementComponent>(entity1).unwrap();
    let m2 = app.world().get::<MovementComponent>(entity2).unwrap();
    let m3 = app.world().get::<MovementComponent>(entity3).unwrap();

    assert!(matches!(m1, MovementComponent::Stuck { .. }));
    assert!(matches!(m2, MovementComponent::FollowingPath { .. })); // Unchanged
    assert!(matches!(m3, MovementComponent::PathRequested { .. })); // Unchanged
}
