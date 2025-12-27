/// Event System TDD Tests
/// Tests for event-driven communication replacing polling patterns
///
/// Testing priority:
/// 1. EntityDied event (replaces death polling)
/// 2. ActionCompleted event (notifies when actions finish)
/// 3. PathCompleted event (notifies when movement finishes)
/// 4. StatCritical event (hunger/thirst critical levels)
use bevy::prelude::*;
use life_simulator::events::{
    ActionCompletedEvent, EntityDied, EntityDiedEvent, PathCompletedEvent, StatCritical,
    StatCriticalEvent,
};
use life_simulator::entities::{Health, Hunger, Stat, Thirst, TilePosition};

// ============================================================================
// TEST 1: EntityDied Event Detection and Emission
// ============================================================================

#[test]
fn test_entity_died_event_emitted_when_health_zero() {
    // ARRANGE: Create app with event system
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register EntityDied event
    app.add_event::<EntityDiedEvent>();

    // Add producer system (detects death and emits event)
    app.add_systems(Update, life_simulator::events::detect_entity_death);

    // Spawn entity with zero health and critical hunger (starvation)
    let entity = app
        .world_mut()
        .spawn((
            Health(Stat::new(0.0, 0.0, 100.0, 0.01)), // Dead entity
            Hunger(Stat::new(95.0, 0.0, 100.0, 0.1)), // Critical hunger (>90%)
            Thirst(Stat::new(20.0, 0.0, 100.0, 0.15)), // Normal thirst
            TilePosition::new(0, 0),
        ))
        .id();

    // ACT: Run the system
    app.update();

    // ASSERT: EntityDied event was emitted
    let events: Vec<EntityDiedEvent> = app
        .world_mut()
        .resource_mut::<Events<EntityDiedEvent>>()
        .drain()
        .collect();

    assert_eq!(events.len(), 1, "Should emit one EntityDied event");
    assert_eq!(events[0].entity, entity, "Event should reference dead entity");
    assert_eq!(
        events[0].cause,
        EntityDied::Starvation,
        "Death cause should be tracked"
    );
}

#[test]
fn test_entity_died_event_not_emitted_for_living_entities() {
    // ARRANGE: Create app with event system
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<EntityDiedEvent>();
    app.add_systems(Update, life_simulator::events::detect_entity_death);

    // Spawn living entity with high health
    app.world_mut().spawn((
        Health(Stat::new(80.0, 0.0, 100.0, 0.01)),
        TilePosition::new(0, 0),
    ));

    // ACT: Run the system
    app.update();

    // ASSERT: No events emitted
    let events: Vec<EntityDiedEvent> = app
        .world_mut()
        .resource_mut::<Events<EntityDiedEvent>>()
        .drain()
        .collect();

    assert_eq!(events.len(), 0, "Should not emit event for living entities");
}

// ============================================================================
// TEST 2: ActionCompleted Event
// ============================================================================

#[test]
fn test_action_completed_event_emitted() {
    // ARRANGE: Create app with ActionCompleted event
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<ActionCompletedEvent>();
    app.add_systems(Update, life_simulator::events::detect_action_completion);

    // Spawn entity with completed action (ActiveAction removed indicates completion)
    let entity = app
        .world_mut()
        .spawn(TilePosition::new(0, 0))
        .id();

    // Simulate action completion by inserting marker component
    app.world_mut()
        .entity_mut(entity)
        .insert(life_simulator::events::ActionJustCompleted {
            action_type: life_simulator::ai::ActionType::Wander {
                target_tile: IVec2::new(0, 0),
            },
            success: true,
        });

    // ACT: Run the system
    app.update();

    // ASSERT: ActionCompleted event was emitted
    let events: Vec<ActionCompletedEvent> = app
        .world_mut()
        .resource_mut::<Events<ActionCompletedEvent>>()
        .drain()
        .collect();

    assert_eq!(events.len(), 1, "Should emit ActionCompleted event");
    assert_eq!(events[0].entity, entity);
    assert!(events[0].success, "Should track success status");
}

// ============================================================================
// TEST 3: PathCompleted Event
// ============================================================================

#[test]
fn test_path_completed_event_emitted_when_path_finishes() {
    // ARRANGE: Create app with PathCompleted event
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PathCompletedEvent>();
    app.add_systems(Update, life_simulator::events::detect_path_completion);

    // Spawn entity with completed path
    let entity = app
        .world_mut()
        .spawn((
            TilePosition::new(5, 5),
            // PathReady removed = path completed
            life_simulator::events::PathJustCompleted {
                destination: IVec2::new(5, 5),
                success: true,
            },
        ))
        .id();

    // ACT: Run the system
    app.update();

    // ASSERT: PathCompleted event was emitted
    let events: Vec<PathCompletedEvent> = app
        .world_mut()
        .resource_mut::<Events<PathCompletedEvent>>()
        .drain()
        .collect();

    assert_eq!(events.len(), 1, "Should emit PathCompleted event");
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].destination, IVec2::new(5, 5));
    assert!(events[0].success);
}

// ============================================================================
// TEST 4: StatCritical Event
// ============================================================================

#[test]
fn test_stat_critical_event_emitted_for_critical_hunger() {
    // ARRANGE: Create app with StatCritical event
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<StatCriticalEvent>();
    app.add_systems(Update, life_simulator::events::detect_stat_critical);

    // Spawn entity with critical hunger (90%+)
    let entity = app
        .world_mut()
        .spawn((
            Hunger(Stat::new(95.0, 0.0, 100.0, 0.1)), // 95% hungry (critical)
            Thirst(Stat::new(20.0, 0.0, 100.0, 0.15)),
            TilePosition::new(0, 0),
        ))
        .id();

    // ACT: Run the system
    app.update();

    // ASSERT: StatCritical event was emitted
    let events: Vec<StatCriticalEvent> = app
        .world_mut()
        .resource_mut::<Events<StatCriticalEvent>>()
        .drain()
        .collect();

    assert_eq!(events.len(), 1, "Should emit StatCritical event");
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].stat_type, StatCritical::Hunger);
    assert!(events[0].value >= 0.9, "Critical threshold is 90%");
}

#[test]
fn test_stat_critical_event_not_emitted_for_normal_stats() {
    // ARRANGE: Create app with StatCritical event
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<StatCriticalEvent>();
    app.add_systems(Update, life_simulator::events::detect_stat_critical);

    // Spawn entity with normal stats
    app.world_mut().spawn((
        Hunger(Stat::new(30.0, 0.0, 100.0, 0.1)),
        Thirst(Stat::new(40.0, 0.0, 100.0, 0.15)),
        TilePosition::new(0, 0),
    ));

    // ACT: Run the system
    app.update();

    // ASSERT: No events emitted
    let events: Vec<StatCriticalEvent> = app
        .world_mut()
        .resource_mut::<Events<StatCriticalEvent>>()
        .drain()
        .collect();

    assert_eq!(events.len(), 0, "Should not emit event for normal stats");
}

// ============================================================================
// TEST 5: Event Consumer - Death Handler
// ============================================================================

#[test]
fn test_death_event_consumer_despawns_entity() {
    // ARRANGE: Create app with death event system
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<EntityDiedEvent>();

    // Add consumer system (handles death events)
    app.add_systems(Update, life_simulator::events::handle_entity_death);

    // Spawn entity
    let entity = app
        .world_mut()
        .spawn((
            Health(Stat::new(0.0, 0.0, 100.0, 0.01)),
            TilePosition::new(0, 0),
        ))
        .id();

    // Manually emit death event
    app.world_mut()
        .resource_mut::<Events<EntityDiedEvent>>()
        .send(EntityDiedEvent {
            entity,
            cause: EntityDied::Starvation,
        });

    // ACT: Run the consumer system
    app.update();

    // ASSERT: Entity was despawned
    assert!(
        app.world().get_entity(entity).is_err(),
        "Entity should be despawned after death event"
    );
}
