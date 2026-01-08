#![allow(static_mut_refs)]
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

fn main() {
    println!("🧪 Testing plugin interaction with ScheduleRunnerPlugin");

    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        ))),
    )
    .add_plugins(bevy::log::LogPlugin::default());

    println!("✅ Added MinimalPlugins + LogPlugin");

    // Add SimulationPlugin
    app.add_plugins(life_simulator::simulation::SimulationPlugin);
    println!("✅ Added SimulationPlugin");

    // Add test system
    app.add_systems(Update, test_system);
    println!("✅ Added test_system");

    println!("🚀 Running app...");
    app.run();
}

fn test_system() {
    static mut FRAME: u32 = 0;
    unsafe {
        FRAME += 1;
        println!("✅ TEST Frame {} - Update is running!", FRAME);

        if FRAME >= 3 {
            println!("🎯 Test PASSED");
            std::process::exit(0);
        }
    }
}

use life_simulator;
