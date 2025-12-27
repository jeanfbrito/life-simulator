/// Test to verify entity tracker synchronization works correctly
///
/// This test ensures that:
/// 1. Entity tracker syncs all spawned entities
/// 2. Tracker count matches entity count
/// 3. Sync works even when entities are spawned in batches

use bevy::prelude::*;
use life_simulator::entities::{
    Creature, EntityTracker, TilePosition, MovementSpeed, EntityStatsBundle,
    sync_entities_to_tracker, init_entity_tracker, Sex, spawn_creature,
};

#[test]
fn test_entity_tracker_syncs_all_entities() {
    // Create a minimal Bevy app for testing
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Initialize the entity tracker
    app.add_systems(Startup, init_entity_tracker);
    app.add_systems(Update, sync_entities_to_tracker);

    // Run startup to initialize tracker
    app.update();

    // Spawn test entities
    let world = &mut app.world();
    world.resource_scope(|world, _: Mut<Time>| {
        let mut commands = world.commands();

        // Spawn 10 test entities with minimal components
        for i in 0..10 {
            commands.spawn((
                Creature {
                    name: format!("TestEntity{}", i),
                    species: "TestSpecies".to_string(),
                },
                TilePosition::from_tile(IVec2::new(i, i)),
                MovementSpeed::normal(),
            ));
        }
    });

    // Apply commands to actually spawn entities
    app.update();

    // Run another update to ensure sync happens after spawn
    app.update();

    // Verify tracker has all entities
    if let Some(tracker) = EntityTracker::global() {
        let tracker = tracker.read().unwrap();
        let tracked_count = tracker.count();

        println!("Tracked entities: {}", tracked_count);
        assert_eq!(
            tracked_count, 10,
            "Entity tracker should have 10 entities, but has {}",
            tracked_count
        );
    } else {
        panic!("Entity tracker not initialized!");
    }
}

#[test]
fn test_entity_tracker_syncs_with_full_components() {
    // Create a minimal Bevy app for testing
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Initialize the entity tracker
    app.add_systems(Startup, init_entity_tracker);
    app.add_systems(Update, sync_entities_to_tracker);

    // Run startup
    app.update();

    // Spawn entities with full component sets
    let world = &mut app.world();
    world.resource_scope(|world, _: Mut<Time>| {
        let mut commands = world.commands();

        for i in 0..5 {
            commands.spawn((
                Creature {
                    name: format!("FullEntity{}", i),
                    species: "TestSpecies".to_string(),
                },
                TilePosition::from_tile(IVec2::new(i * 10, i * 10)),
                MovementSpeed::normal(),
                EntityStatsBundle::default(),
                Sex::Male,
            ));
        }
    });

    // Apply spawn commands
    app.update();

    // Run sync
    app.update();

    // Verify all entities tracked
    if let Some(tracker) = EntityTracker::global() {
        let tracker = tracker.read().unwrap();
        assert_eq!(
            tracker.count(),
            5,
            "Should track all 5 entities with full components"
        );
    } else {
        panic!("Entity tracker not initialized!");
    }
}

#[test]
fn test_entity_tracker_batch_spawn() {
    // Simulates batch spawning like spawn_entities_from_config
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Startup, init_entity_tracker);
    app.add_systems(Update, sync_entities_to_tracker);

    app.update();

    // Spawn in batches to simulate real spawn behavior
    for batch in 0..5 {
        let world = &mut app.world();
        world.resource_scope(|world, _: Mut<Time>| {
            let mut commands = world.commands();

            for i in 0..20 {
                commands.spawn((
                    Creature {
                        name: format!("Batch{}Entity{}", batch, i),
                        species: "TestSpecies".to_string(),
                    },
                    TilePosition::from_tile(IVec2::new(batch * 100 + i, i)),
                    MovementSpeed::normal(),
                ));
            }
        });

        app.update(); // Apply spawn commands for this batch
    }

    // Final sync
    app.update();

    // Should have 100 entities total (5 batches * 20 entities)
    if let Some(tracker) = EntityTracker::global() {
        let tracker = tracker.read().unwrap();
        assert_eq!(
            tracker.count(),
            100,
            "Should track all 100 entities from batch spawning"
        );
    } else {
        panic!("Entity tracker not initialized!");
    }
}
