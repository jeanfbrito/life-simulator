/// Core simulation tick system
/// Provides the heartbeat for all discrete game logic
use bevy::prelude::*;

pub mod profiler;
pub mod system_sets;
pub mod tick;

// Re-exports
pub use tick::{
    every_n_ticks, increment_tick_counter, log_realtime_performance, log_tick_metrics,
    RealtimePerformanceTimer, SimulationSpeed, SimulationState, SimulationTick, TickAccumulator,
    TickMetrics,
};

pub use profiler::{
    end_timing_resource, start_timing_resource, ScopedTimer, TickProfiler, TickProfilerPlugin,
};

pub use system_sets::SimulationSet;

/// Base tick rate: 10 ticks per second (100ms per tick)
/// This is a good balance between responsiveness and performance
pub const BASE_TICK_RATE: f64 = 10.0;
pub const TICK_DURATION_MS: u64 = 100;
const METRICS_LOG_INTERVAL_TICKS: u64 = 50;

/// Plugin that sets up the core simulation tick system
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        info!("🔌 SimulationPlugin: Installing tick systems...");
        app
            // Resources
            .insert_resource(SimulationTick::default())
            .insert_resource(SimulationSpeed::default())
            .insert_resource(SimulationState { should_tick: true })
            .insert_resource(TickMetrics::default())
            .insert_resource(TickAccumulator::default())
            .insert_resource(RealtimePerformanceTimer::default())
            // Core tick systems run in Update schedule
            .add_systems(
                Update,
                (
                    diagnostic_heartbeat,
                    accumulate_ticks.before(run_simulation_ticks),
                    run_simulation_ticks,
                    handle_speed_controls,
                    log_realtime_performance, // Runs every frame, checks timer internally
                ),
            )
            .add_plugins(TickProfilerPlugin)
            .add_systems(
                Update,
                log_tick_metrics
                    .after(run_simulation_ticks)
                    .run_if(every_n_ticks(METRICS_LOG_INTERVAL_TICKS)),
            );
        info!("✅ SimulationPlugin: Tick systems installed");
    }
}

/// Diagnostic system to verify Update schedule is running
fn diagnostic_heartbeat() {
    static mut HEARTBEAT: u32 = 0;
    unsafe {
        HEARTBEAT += 1;
        if HEARTBEAT <= 3 || HEARTBEAT % 600 == 0 {
            info!("💓 Heartbeat #{} - Update schedule is running", HEARTBEAT);
        }
    }
}

/// System that accumulates frame time and determines when ticks should run
fn accumulate_ticks(
    time: Res<Time>,
    mut accumulator: ResMut<TickAccumulator>,
    speed: Res<SimulationSpeed>,
    mut state: ResMut<SimulationState>,
) {
    if speed.is_paused() {
        accumulator.pending_ticks = 0;
        state.should_tick = false;
        return;
    }

    let tick_duration = 1.0 / BASE_TICK_RATE as f32;
    let delta = time.delta_secs();
    let ticks = accumulator.update(delta, tick_duration, speed.multiplier);
    state.should_tick = ticks > 0;

    // Debug: Log first few frames
    static mut FRAME_COUNT: u32 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT <= 5 || (FRAME_COUNT % 100 == 0) {
            info!(
                "🔍 Frame {}: delta={:.4}s, ticks={}, accumulated={:.4}",
                FRAME_COUNT, delta, ticks, accumulator.accumulated
            );
        }
    }
}

/// System that runs simulation ticks when accumulated
fn run_simulation_ticks(
    mut tick: ResMut<SimulationTick>,
    mut metrics: ResMut<TickMetrics>,
    mut accumulator: ResMut<TickAccumulator>,
    mut profiler: ResMut<TickProfiler>,
    state: Res<SimulationState>,
) {
    let ticks_to_run = accumulator.pending_ticks;

    if ticks_to_run == 0 {
        return;
    }

    // Run each tick
    for _ in 0..ticks_to_run {
        // End previous tick timing
        metrics.end_tick();

        // Start new tick timing
        metrics.start_tick();

        // Start profiler frame
        profiler.start_frame();

        // Increment counter
        tick.increment();

        // Log every 100 ticks
        if tick.get() % 100 == 0 {
            info!(
                "🎯 Tick #{} | TPS: {:.1} | Avg duration: {:?}",
                tick.get(),
                metrics.actual_tps(),
                metrics.average_duration()
            );
        }

        // Check if profiler should report this tick
        if profiler.should_report(tick.get()) {
            let report = profiler.generate_report(tick.get());
            info!("{}", report);
            profiler.reset_period();
            profiler.last_report_tick = tick.get();
        }

        // End profiler frame
        profiler.end_frame();
    }

    // Clear pending ticks
    accumulator.pending_ticks = 0;
}

/// System to handle pause/speed controls (runs every frame)
fn handle_speed_controls(keyboard: Res<ButtonInput<KeyCode>>, mut speed: ResMut<SimulationSpeed>) {
    // Space to pause/unpause
    if keyboard.just_pressed(KeyCode::Space) {
        speed.toggle_pause();
        info!(
            "Simulation {}",
            if speed.is_paused() {
                "PAUSED"
            } else {
                "RESUMED"
            }
        );
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
}
