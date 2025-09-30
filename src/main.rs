use bevy::prelude::*;
use bevy::log::LogPlugin;

mod tilemap;

use tilemap::{TilemapPlugin, WorldGenerator, WorldConfig};

fn main() {
    println!("🚀 Starting Life Simulator (Headless Mode)");

    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: bevy::log::Level::INFO,
                filter: "info,warn".to_string(),
                ..default()
            }),
            TilemapPlugin,
        ))
          .add_systems(Startup, setup)
        .add_systems(Update, simulation_system)
        .run();
}

fn setup(
    mut commands: Commands,
    world_generator: Res<WorldGenerator>,
) {
    info!("LIFE_SIMULATOR: Setting up headless life simulation");

    // Create a test entity at spawn point
    if let Some((spawn_x, spawn_y)) = world_generator.find_spawn_point() {
        let spawn_entity = commands.spawn_empty().id();
        info!("LIFE_SIMULATOR: Spawn entity created at ({}, {})", spawn_x, spawn_y);

        // Log terrain information at spawn point
        info!("LIFE_SIMULATOR: Ready to simulate life at world coordinates ({}, {})", spawn_x, spawn_y);
    } else {
        warn!("LIFE_SIMULATOR: Could not find spawn point, using origin");
    }
}

fn simulation_system(
    world_generator: Res<WorldGenerator>,
    time: Res<Time>,
) {
    // Basic simulation loop - runs once per frame
    // In a full implementation, this would handle entity updates, AI, etc.

    static mut LAST_LOG: f32 = 0.0;
    let current_time = time.elapsed_secs();

    unsafe {
        if current_time - LAST_LOG > 5.0 {
            info!("LIFE_SIMULATOR: Simulation running - {:.1}s elapsed", current_time);
            LAST_LOG = current_time;
        }
    }
}
