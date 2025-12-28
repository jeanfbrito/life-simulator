/// Minimal test to verify ScheduleRunnerPlugin + MinimalPlugins runs Update schedule
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

fn main() {
    println!("🧪 TEST: Verifying Update schedule runs with MinimalPlugins + ScheduleRunnerPlugin");

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_plugins(bevy::log::LogPlugin::default())
        .add_systems(Update, test_update_system)
        .add_systems(Startup, test_startup_system)
        .run();
}

fn test_startup_system() {
    println!("✅ STARTUP: Startup system executed");
}

fn test_update_system() {
    static mut FRAME_COUNT: u32 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT <= 10 {
            println!("✅ UPDATE: Frame {} - Update schedule is running!", FRAME_COUNT);
        } else if FRAME_COUNT == 11 {
            println!("✅ TEST PASSED: Update schedule runs successfully");
            println!("   Exiting after 11 frames...");
            std::process::exit(0);
        }
    }
}
