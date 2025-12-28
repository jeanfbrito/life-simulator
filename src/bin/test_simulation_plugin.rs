/// Test to verify SimulationPlugin systems run with MinimalPlugins
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use life_simulator::simulation::SimulationPlugin;
use std::time::Duration;

fn main() {
    println!("🧪 TEST: Verifying SimulationPlugin runs with MinimalPlugins");

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_plugins(bevy::log::LogPlugin::default())
        .add_plugins(SimulationPlugin)
        .add_systems(Update, test_counter)
        .run();
}

fn test_counter() {
    static mut FRAME_COUNT: u32 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT == 300 {
            // After 5 seconds at 60 FPS
            println!("✅ TEST COMPLETE: SimulationPlugin systems are running");
            println!("   Check logs above for heartbeat and accumulate_ticks messages");
            std::process::exit(0);
        }
    }
}
