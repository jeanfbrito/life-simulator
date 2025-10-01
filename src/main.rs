use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;

mod tilemap;
mod web;
mod serialization;
mod cached_world;
mod resources;

use tilemap::{TilemapPlugin, WorldGenerator, WorldConfig};
use serialization::WorldSerializationPlugin;
use cached_world::CachedWorldPlugin;

mod web_server_simple;

fn main() {
    println!("🚀 Starting Life Simulator (Headless Mode)");

    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ))
        .add_plugins(TilemapPlugin)
        .add_plugins(WorldSerializationPlugin)
        .add_plugins(CachedWorldPlugin)
        .insert_resource(WorldConfig::default())
        .add_systems(Startup, setup)
        .add_systems(Update, simulation_system)
        .run();
}

fn setup(
    mut commands: Commands,
    world_generator: Res<WorldGenerator>,
) {
    println!("🔧 LIFE_SIMULATOR: Setting up headless life simulation");

    // Start the web server
    println!("🌐 LIFE_SIMULATOR: Starting web server...");
    web_server_simple::start_simple_web_server();
    println!("✅ LIFE_SIMULATOR: Web server started at http://127.0.0.1:54321");

    // Create a test entity at spawn point
    if let Some((spawn_x, spawn_y)) = world_generator.find_spawn_point() {
        let _spawn_entity = commands.spawn_empty().id();
        println!("📍 LIFE_SIMULATOR: Spawn entity created at ({}, {})", spawn_x, spawn_y);

        // Log terrain information at spawn point
        println!("🌍 LIFE_SIMULATOR: Ready to simulate life at world coordinates ({}, {})", spawn_x, spawn_y);
    } else {
        println!("⚠️ LIFE_SIMULATOR: Could not find spawn point, using origin");
    }
}

fn simulation_system(
    _world_generator: Res<WorldGenerator>,
) {
    // Basic simulation loop - runs once per frame
    // In a full implementation, this would handle entity updates, AI, etc.

    static mut FRAME_COUNT: u64 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT % 300 == 0 { // Every 5 seconds at 60 FPS
            println!("🔄 LIFE_SIMULATOR: Simulation running - frame {}", FRAME_COUNT);
        }
    }
}
