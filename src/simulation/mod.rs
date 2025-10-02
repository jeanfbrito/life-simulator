/// Core simulation tick system
/// Provides the heartbeat for all discrete game logic
use bevy::prelude::*;
use std::time::Duration;

pub mod tick;

// Re-exports
pub use tick::{
    SimulationTick, SimulationSpeed, TickMetrics, SimulationState,
    increment_tick_counter, log_tick_metrics, every_n_ticks,
};

/// Base tick rate: 10 ticks per second (100ms per tick)
/// This is a good balance between responsiveness and performance
pub const BASE_TICK_RATE: f64 = 10.0;
pub const TICK_DURATION_MS: u64 = 100;

/// Plugin that sets up the core simulation tick system
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(SimulationTick::default())
            .insert_resource(SimulationSpeed::default())
            .insert_resource(SimulationState::default())
            .insert_resource(TickMetrics::default())
            
            // Configure fixed timestep for ticks
            .insert_resource(Time::<Fixed>::from_duration(
                Duration::from_millis(TICK_DURATION_MS)
            ))
            
            // Core tick systems (run in FixedUpdate)
            .add_systems(FixedUpdate, (
                increment_tick_counter,
                log_tick_metrics.run_if(every_n_ticks(100)), // Log every 10 seconds
            ).chain())
            
            // Non-tick systems can be added in Update schedule
            .add_systems(Update, (
                handle_speed_controls,
            ));
    }
}

/// System to handle pause/speed controls (runs every frame)
fn handle_speed_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut speed: ResMut<SimulationSpeed>,
    mut time: ResMut<Time<Fixed>>,
) {
    // Space to pause/unpause
    if keyboard.just_pressed(KeyCode::Space) {
        speed.toggle_pause();
        info!("Simulation {}", if speed.is_paused() { "PAUSED" } else { "RESUMED" });
    }
    
    // Number keys for speed control
    if keyboard.just_pressed(KeyCode::Digit1) {
        speed.set_speed(0.5);
        info!("Speed: 0.5x (Slow)");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        speed.set_speed(1.0);
        info!("Speed: 1.0x (Normal)");
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        speed.set_speed(2.0);
        info!("Speed: 2.0x (Fast)");
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        speed.set_speed(3.0);
        info!("Speed: 3.0x (Ultra)");
    }
    
    // Update fixed timestep based on speed (if not paused)
    if !speed.is_paused() {
        let adjusted_duration = Duration::from_secs_f64(
            TICK_DURATION_MS as f64 / 1000.0 / speed.multiplier as f64
        );
        time.set_timestep(adjusted_duration);
    }
}
