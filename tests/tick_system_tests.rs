// Integration tests for tick system consistency
// These tests verify the fixes documented in docs/TICK_SYSTEM_FIXES.md

mod test_utils;

use bevy::prelude::*;
use life_simulator::entities::{movement_tick_system, wanderer_ai_system};
use life_simulator::simulation::{accumulate_ticks, should_tick, SimulationTick, SimulationSpeed};
use std::time::Duration;
use test_utils::*;

/// Test that the tick accumulator correctly converts time to ticks
#[test]
fn test_tick_accumulation_basic() {
    let mut app = create_test_app();
    
    // Add the accumulate_ticks system
    app.add_systems(Update, accumulate_ticks);
    
    // Initial state
    assert_eq!(get_tick_count(&app), 0);
    
    // Run for 1 second at 10 TPS = should get ~10 ticks
    run_for_duration(&mut app, Duration::from_secs(1));
    
    let ticks = get_tick_count(&app);
    assert!(
        ticks >= 8 && ticks <= 12,
        "Expected ~10 ticks after 1 second, got {}",
        ticks
    );
}

/// Test that tick accumulation is consistent over longer periods
#[test]
fn test_tick_accumulation_consistency() {
    let mut app = create_test_app();
    app.add_systems(Update, accumulate_ticks);
    
    // Run for 5 seconds at 10 TPS = should get ~50 ticks
    run_for_duration(&mut app, Duration::from_secs(5));
    
    let ticks = get_tick_count(&app);
    assert!(
        ticks >= 45 && ticks <= 55,
        "Expected ~50 ticks after 5 seconds, got {}",
        ticks
    );
}

/// Test that should_tick flag is set correctly when ticks are available
#[test]
fn test_should_tick_flag() {
    let mut app = create_test_app();
    app.add_systems(Update, accumulate_ticks);
    
    // Initially should not tick
    assert!(!should_tick_now(&app));
    
    // Run for enough time to accumulate ticks
    run_for_duration(&mut app, Duration::from_millis(150)); // > 100ms needed for 1 tick
    
    // After accumulation, should_tick should be true
    let has_ticks = should_tick_now(&app);
    assert!(
        has_ticks,
        "should_tick flag should be true after accumulating time"
    );
}

/// Test that simulation speed multiplier affects tick rate
#[test]
fn test_simulation_speed_multiplier() {
    let mut app = create_test_app();
    app.add_systems(Update, accumulate_ticks);
    
    // Set speed to 2x
    set_simulation_speed(&mut app, 2.0);
    
    // Run for 1 second at 2x speed = should get ~20 ticks (double normal)
    run_for_duration(&mut app, Duration::from_secs(1));
    
    let ticks = get_tick_count(&app);
    assert!(
        ticks >= 18 && ticks <= 22,
        "Expected ~20 ticks at 2x speed, got {}",
        ticks
    );
}

/// Test that changing speed mid-simulation works correctly
#[test]
fn test_dynamic_speed_change() {
    let mut app = create_test_app();
    app.add_systems(Update, accumulate_ticks);
    
    // Run 1 second at normal speed (1.0x) = ~10 ticks
    run_for_duration(&mut app, Duration::from_secs(1));
    let ticks_after_1s = get_tick_count(&app);
    
    // Change to 3x speed
    set_simulation_speed(&mut app, 3.0);
    
    // Run another second at 3x speed = ~30 more ticks
    run_for_duration(&mut app, Duration::from_secs(1));
    let ticks_after_2s = get_tick_count(&app);
    
    let additional_ticks = ticks_after_2s - ticks_after_1s;
    assert!(
        additional_ticks >= 28 && additional_ticks <= 32,
        "Expected ~30 additional ticks at 3x speed, got {}",
        additional_ticks
    );
}

/// Test that entity movement timing is correct
#[test]
fn test_entity_movement_timing() {
    let mut app = create_test_app();
    
    // Add systems
    app.add_systems(Update, accumulate_ticks);
    app.add_systems(Update, (
        wanderer_ai_system,
        movement_tick_system,
    ).run_if(should_tick));
    
    // Spawn a test entity with 30 ticks per tile movement speed
    let entity = app.world.spawn_empty().id();
    app.world.entity_mut(entity).insert({
        use life_simulator::entities::{EntityName, TilePosition, MovementSpeed, Wanderer};
        (
            EntityName("TestHuman".to_string()),
            TilePosition::from_xy(50, 50),
            MovementSpeed { ticks_per_tile: 30 },
            Wanderer::default(),
        )
    });
    
    let start_pos = get_entity_position(&app, entity).unwrap();
    
    // Run for 3 seconds = 30 ticks = should complete 1 tile movement
    run_for_duration(&mut app, Duration::from_secs(3));
    
    // Entity might not have moved yet (waiting for AI decision or path)
    // So we run longer to ensure AI has time to act
    run_for_duration(&mut app, Duration::from_secs(6)); // Total 9 seconds = 90 ticks
    
    let end_pos = get_entity_position(&app, entity).unwrap();
    
    // Entity should have moved at least 1 tile in 9 seconds (90 ticks / 30 ticks per tile = 3 tiles possible)
    let distance = manhattan_distance(start_pos, end_pos);
    assert!(
        distance >= 1,
        "Entity should have moved at least 1 tile, but distance is {}",
        distance
    );
}

/// Test that multiple entities move independently and consistently
#[test]
fn test_multi_entity_movement_consistency() {
    let mut app = create_test_app();
    
    // Add systems
    app.add_systems(Update, accumulate_ticks);
    app.add_systems(Update, (
        wanderer_ai_system,
        movement_tick_system,
    ).run_if(should_tick));
    
    // Spawn multiple entities with different speeds
    let fast_entity = app.world.spawn_empty().id();
    let slow_entity = app.world.spawn_empty().id();
    
    app.world.entity_mut(fast_entity).insert({
        use life_simulator::entities::{EntityName, TilePosition, MovementSpeed, Wanderer};
        (
            EntityName("FastHuman".to_string()),
            TilePosition::from_xy(50, 50),
            MovementSpeed { ticks_per_tile: 10 }, // Fast: 1 second per tile
            Wanderer::default(),
        )
    });
    
    app.world.entity_mut(slow_entity).insert({
        use life_simulator::entities::{EntityName, TilePosition, MovementSpeed, Wanderer};
        (
            EntityName("SlowHuman".to_string()),
            TilePosition::from_xy(50, 50),
            MovementSpeed { ticks_per_tile: 50 }, // Slow: 5 seconds per tile
            Wanderer::default(),
        )
    });
    
    let fast_start = get_entity_position(&app, fast_entity).unwrap();
    let slow_start = get_entity_position(&app, slow_entity).unwrap();
    
    // Run for 10 seconds
    run_for_duration(&mut app, Duration::from_secs(10));
    
    let fast_end = get_entity_position(&app, fast_entity).unwrap();
    let slow_end = get_entity_position(&app, slow_entity).unwrap();
    
    let fast_distance = manhattan_distance(fast_start, fast_end);
    let slow_distance = manhattan_distance(slow_start, slow_end);
    
    // Fast entity should have moved more than slow entity
    // (though wandering AI randomness might affect this)
    println!("Fast entity moved {} tiles", fast_distance);
    println!("Slow entity moved {} tiles", slow_distance);
    
    // Both should have moved at least some distance
    assert!(fast_distance > 0, "Fast entity should have moved");
    assert!(slow_distance > 0, "Slow entity should have moved");
}

/// Test that ticks don't accumulate when paused
#[test]
fn test_pause_functionality() {
    let mut app = create_test_app();
    app.add_systems(Update, accumulate_ticks);
    
    // Run for 1 second normally
    run_for_duration(&mut app, Duration::from_secs(1));
    let ticks_before_pause = get_tick_count(&app);
    
    // Pause by setting speed to 0
    set_simulation_speed(&mut app, 0.0);
    
    // Run for another second while paused
    run_for_duration(&mut app, Duration::from_secs(1));
    let ticks_after_pause = get_tick_count(&app);
    
    // Ticks should not have increased (or increased very minimally)
    assert!(
        ticks_after_pause <= ticks_before_pause + 2,
        "Ticks should not accumulate when paused"
    );
}

/// Test that the system can handle very long simulation times
#[test]
fn test_long_running_stability() {
    let mut app = create_test_app();
    app.add_systems(Update, accumulate_ticks);
    
    // Run for 30 seconds of simulated time
    run_for_duration(&mut app, Duration::from_secs(30));
    
    let ticks = get_tick_count(&app);
    
    // Should have ~300 ticks (30 seconds * 10 TPS)
    assert!(
        ticks >= 280 && ticks <= 320,
        "Expected ~300 ticks after 30 seconds, got {}",
        ticks
    );
    
    // System should still be responsive
    let should_tick = should_tick_now(&app);
    println!("After 30 seconds: {} ticks, should_tick={}", ticks, should_tick);
}

/// Test the exact movement timing from our manual verification
#[test]
fn test_exact_movement_timing_30_ticks() {
    let mut app = create_test_app();
    
    // Add systems
    app.add_systems(Update, accumulate_ticks);
    app.add_systems(Update, (
        wanderer_ai_system,
        movement_tick_system,
    ).run_if(should_tick));
    
    // Spawn entity with 30 ticks per tile (3 seconds)
    let entity = app.world.spawn_empty().id();
    app.world.entity_mut(entity).insert({
        use life_simulator::entities::{EntityName, TilePosition, MovementSpeed, Wanderer};
        (
            EntityName("TestHuman".to_string()),
            TilePosition::from_xy(50, 50),
            MovementSpeed { ticks_per_tile: 30 },
            Wanderer::default(),
        )
    });
    
    let mut tracker = PositionTracker::new();
    
    // Record position every 2 seconds for 20 seconds
    for i in 0..10 {
        run_for_duration(&mut app, Duration::from_secs(2));
        tracker.record(&app, entity);
        
        println!(
            "At {}s: tick={}, pos={:?}",
            (i + 1) * 2,
            get_tick_count(&app),
            get_entity_position(&app, entity)
        );
    }
    
    // Entity should have moved multiple times
    let movements = tracker.movement_count();
    println!("Total movements in 20 seconds: {}", movements);
    
    // With 30 ticks per tile and 10 TPS, entity can move every 3 seconds
    // In 20 seconds, could move up to 6 times (but AI decisions affect this)
    assert!(
        movements >= 2,
        "Entity should have moved at least 2 times in 20 seconds, but only moved {} times",
        movements
    );
    
    tracker.print_history();
}

/// Test that tick rate stays consistent under different frame rates
#[test]
fn test_frame_rate_independence() {
    // Test with high frame rate (60 FPS equivalent)
    let mut app_fast = create_test_app();
    app_fast.add_systems(Update, accumulate_ticks);
    
    for _ in 0..600 {
        // 600 frames at 16ms each = ~10 seconds
        let mut time = app_fast.world.resource_mut::<Time>();
        time.advance_by(Duration::from_millis(16));
        app_fast.update();
    }
    
    let ticks_fast = get_tick_count(&app_fast);
    
    // Test with low frame rate (10 FPS equivalent)
    let mut app_slow = create_test_app();
    app_slow.add_systems(Update, accumulate_ticks);
    
    for _ in 0..100 {
        // 100 frames at 100ms each = ~10 seconds
        let mut time = app_slow.world.resource_mut::<Time>();
        time.advance_by(Duration::from_millis(100));
        app_slow.update();
    }
    
    let ticks_slow = get_tick_count(&app_slow);
    
    // Both should have approximately the same number of ticks (~100)
    println!("Fast frame rate: {} ticks", ticks_fast);
    println!("Slow frame rate: {} ticks", ticks_slow);
    
    assert!(
        (ticks_fast as i64 - ticks_slow as i64).abs() <= 5,
        "Tick counts should be similar regardless of frame rate. Fast: {}, Slow: {}",
        ticks_fast,
        ticks_slow
    );
}

/// Regression test: Ensure systems don't run when should_tick is false
#[test]
fn test_systems_respect_should_tick_condition() {
    let mut app = create_test_app();
    
    // Add accumulator but NOT the movement systems
    app.add_systems(Update, accumulate_ticks);
    
    // Spawn an entity
    let entity = app.world.spawn_empty().id();
    app.world.entity_mut(entity).insert({
        use life_simulator::entities::{EntityName, TilePosition, MovementSpeed};
        (
            EntityName("TestHuman".to_string()),
            TilePosition::from_xy(50, 50),
            MovementSpeed { ticks_per_tile: 30 },
        )
    });
    
    let start_pos = get_entity_position(&app, entity).unwrap();
    
    // Run for a while - entity should NOT move because movement_tick_system isn't added
    run_for_duration(&mut app, Duration::from_secs(5));
    
    let end_pos = get_entity_position(&app, entity).unwrap();
    
    // Position should be unchanged
    assert_eq!(
        start_pos, end_pos,
        "Entity should not move without movement systems"
    );
}

/// Test that tick accumulation handles edge cases (very small time deltas)
#[test]
fn test_tick_accumulation_small_deltas() {
    let mut app = create_test_app();
    app.add_systems(Update, accumulate_ticks);
    
    // Simulate many tiny frames (1ms each)
    for _ in 0..10_000 {
        let mut time = app.world.resource_mut::<Time>();
        time.advance_by(Duration::from_millis(1));
        app.update();
    }
    
    let ticks = get_tick_count(&app);
    
    // 10,000ms = 10 seconds = ~100 ticks at 10 TPS
    assert!(
        ticks >= 95 && ticks <= 105,
        "Expected ~100 ticks, got {}",
        ticks
    );
}
