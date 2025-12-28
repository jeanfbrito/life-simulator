use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

fn main() {
    println!("🧪 Testing minimal ScheduleRunnerPlugin setup");

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 10.0, // 10 FPS
            ))),
        )
        .add_plugins(bevy::log::LogPlugin::default())
        .add_systems(Update, test_system)
        .run();
}

fn test_system() {
    static mut FRAME: u32 = 0;
    unsafe {
        FRAME += 1;
        println!("✅ Frame {} - Update system is running!", FRAME);

        if FRAME >= 5 {
            println!("🎯 Test PASSED - Update schedule is running");
            std::process::exit(0);
        }
    }
}
