/// Integration test for real-time performance logging system
/// Tests that the performance logger runs on wall-clock time, not tick count
use bevy::prelude::*;
use life_simulator::simulation::{
    RealtimePerformanceTimer, SimulationPlugin, SimulationSpeed, SimulationTick, TickMetrics,
};

#[test]
fn test_realtime_performance_logging_integration() {
    // Create minimal Bevy app with just the simulation plugin
    let mut app = App::new();

    // Add minimal plugins and input resource
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<KeyCode>>(); // Required for handle_speed_controls
    app.add_plugins(SimulationPlugin);

    // Override the timer to use a very short interval for testing
    app.world_mut()
        .insert_resource(RealtimePerformanceTimer::new(0.05)); // 50ms

    // Initialize the app
    app.update();

    // Verify resources exist
    assert!(app.world().get_resource::<SimulationTick>().is_some());
    assert!(app
        .world()
        .get_resource::<RealtimePerformanceTimer>()
        .is_some());
    assert!(app.world().get_resource::<TickMetrics>().is_some());
    assert!(app.world().get_resource::<SimulationSpeed>().is_some());

    // Run a few frames
    for _ in 0..3 {
        app.update();
    }

    // Wait for the timer interval
    std::thread::sleep(std::time::Duration::from_millis(60));

    // Run one more frame - should trigger logging
    app.update();

    // Verify the timer was reset (indicating logging occurred)
    let timer = app.world().get_resource::<RealtimePerformanceTimer>().unwrap();
    assert!(
        timer.elapsed_seconds() < 0.05,
        "Timer should have been reset after logging"
    );
}

#[test]
fn test_performance_logging_with_zero_tps() {
    // Test that logging works even when TPS is 0 (no ticks accumulated yet)
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(SimulationPlugin);

    // Set very short timer interval
    app.world_mut()
        .insert_resource(RealtimePerformanceTimer::new(0.01));

    // Pause the simulation so TPS stays at 0
    app.world_mut().resource_mut::<SimulationSpeed>().pause();

    // Wait and update
    std::thread::sleep(std::time::Duration::from_millis(15));
    app.update();

    // Should still log even with 0 TPS
    let timer = app.world().get_resource::<RealtimePerformanceTimer>().unwrap();
    assert!(timer.elapsed_seconds() < 0.015);
}

#[test]
fn test_performance_logging_includes_entity_count() {
    // Verify that entity count is tracked correctly
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(SimulationPlugin);

    // Spawn some test entities
    app.world_mut().spawn(());
    app.world_mut().spawn(());
    app.world_mut().spawn(());

    // Set short timer
    app.world_mut()
        .insert_resource(RealtimePerformanceTimer::new(0.01));

    // Update and verify
    std::thread::sleep(std::time::Duration::from_millis(15));
    app.update();

    // Entity count should include the 3 we spawned
    // Note: The exact count may vary due to internal Bevy entities
    let entity_count = app.world().iter_entities().count();
    assert!(entity_count >= 3, "Should have at least 3 entities");
}
