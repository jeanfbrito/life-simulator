use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

mod ai;
mod cached_world;
mod debug;
mod entities;
mod errors;
mod pathfinding;
mod resources;
mod serialization;
mod simulation;
mod tilemap;
mod types;
mod vegetation;
mod web;
mod world_loader;

use ai::TQUAIPlugin;
use cached_world::CachedWorldPlugin;
use debug::{HealthCheckPlugin, HealthCheckApiPlugin};
use entities::EntitiesPlugin;
use pathfinding::{pathfinding_cache_cleanup_system, process_pathfinding_requests, PathCache, PathfindingGrid, PathfindingQueuePlugin};
use serialization::{WorldLoadRequest, WorldSaveRequest, WorldSerializationPlugin};
use simulation::SimulationPlugin;
use tilemap::{TilemapPlugin, WorldConfig};
use vegetation::VegetationPlugin;
use world_loader::WorldLoader;

mod web_server_simple;

fn main() {
    println!("🚀 Starting Life Simulator (Headless Mode)");
    println!("🔧 Configuring Bevy app with MinimalPlugins...");

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_plugins(bevy::log::LogPlugin::default()) // Enable logging!
        // TilemapPlugin removed - we're loading a world, not generating one
        // WorldSerializationPlugin removed - not needed for running simulation
        .add_plugins(CachedWorldPlugin)
        .add_plugins((
            SimulationPlugin,
            EntitiesPlugin,
            TQUAIPlugin,
            PathfindingQueuePlugin,
            VegetationPlugin,
            HealthCheckPlugin,
            HealthCheckApiPlugin,
        )) // Core plugins
        .insert_resource(WorldConfig::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<PathfindingGrid>()
        .init_resource::<PathCache>()
        .add_systems(
            Startup,
            (setup, entities::spawn_entities_from_config.after(setup)),
        )
        .add_systems(
            Update,
            (
                process_pathfinding_requests, // Async pathfinding with cache
                pathfinding_cache_cleanup_system, // Periodic cache cleanup
                simulation_system,
                save_load_system.after(simulation_system),
            )
                .run_if(resource_exists::<WorldLoader>),
        )
        .run();
}

fn setup(mut commands: Commands, mut pathfinding_grid: ResMut<PathfindingGrid>) {
    println!("🔧 LIFE_SIMULATOR: Setting up headless life simulation");

    // Load the world
    println!("🗺️ LIFE_SIMULATOR: Loading world...");
    let requested_map_name =
        std::env::var("WORLD_MAP_NAME").unwrap_or_else(|_| "slopes_demo".to_string());
    println!(
        "🗺️ LIFE_SIMULATOR: Attempting to load world '{}'",
        requested_map_name
    );

    let world_loader = match WorldLoader::load_by_name(&requested_map_name) {
        Ok(loader) => {
            println!(
                "✅ LIFE_SIMULATOR: World loaded: {} (seed: {})",
                loader.get_name(),
                loader.get_seed()
            );
            loader
        }
        Err(err) => {
            eprintln!(
                "⚠️ LIFE_SIMULATOR: Could not load '{}': {}. Falling back to most recent map...",
                requested_map_name, err
            );
            match WorldLoader::load_default() {
                Ok(loader) => {
                    println!(
                        "✅ LIFE_SIMULATOR: World loaded: {} (seed: {})",
                        loader.get_name(),
                        loader.get_seed()
                    );
                    loader
                }
                Err(e) => {
                    eprintln!("❌ LIFE_SIMULATOR: Failed to load world: {}", e);
                    eprintln!(
                        "💡 LIFE_SIMULATOR: Please generate a world first using: cargo run --bin map_generator"
                    );
                    std::process::exit(1);
                }
            }
        }
    };

    // Build pathfinding grid from terrain and resources
    println!("🧭 LIFE_SIMULATOR: Building pathfinding grid...");

    use tilemap::TerrainType;

    // Get world bounds
    let ((min_x, min_y), (max_x, max_y)) = world_loader.get_world_bounds();
    let tile_min_x = min_x * 16 - 16; // Extra padding
    let tile_min_y = min_y * 16 - 16;
    let tile_max_x = (max_x + 1) * 16 + 16;
    let tile_max_y = (max_y + 1) * 16 + 16;

    let mut tiles_processed = 0;
    let mut tiles_blocked = 0;

    for y in tile_min_y..=tile_max_y {
        for x in tile_min_x..=tile_max_x {
            let pos = bevy::math::IVec2::new(x, y);

            // Get terrain at this position
            let terrain_str = world_loader.get_terrain_at(x, y);
            let terrain_cost = if let Some(terrain_str) = terrain_str {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    let cost = terrain.movement_cost();
                    if cost >= 1000.0 {
                        u32::MAX // Impassable terrain
                    } else {
                        cost as u32
                    }
                } else {
                    // Unknown terrain type, assume impassable
                    u32::MAX
                }
            } else {
                // No terrain data, assume impassable (outside world bounds)
                u32::MAX
            };

            // Check if there's a BLOCKING resource on this tile (Trees and Rocks only)
            // Bushes, Flowers, and Shrubs do NOT block movement
            let has_blocking_resource = world_loader
                .get_resource_at(x, y)
                .and_then(|r| {
                    use crate::resources::{ResourceType, ResourceCategory};
                    ResourceType::from_str(&r).and_then(|rt| rt.get_category())
                })
                .map(|cat| matches!(cat, crate::resources::ResourceCategory::Tree | crate::resources::ResourceCategory::Rock))
                .unwrap_or(false);

            // If terrain is passable but has blocking resource, make it impassable
            let final_cost = if has_blocking_resource && terrain_cost != u32::MAX {
                tiles_blocked += 1;
                u32::MAX // Trees and Rocks block movement
            } else {
                terrain_cost
            };

            pathfinding_grid.set_cost(pos, final_cost);
            tiles_processed += 1;
        }
    }

    println!("✅ LIFE_SIMULATOR: Pathfinding grid ready");
    println!(
        "   📊 Processed {} tiles, {} blocked by resources",
        tiles_processed, tiles_blocked
    );

    // Start the web server
    let disable_web = std::env::var("DISABLE_WEB_SERVER").ok() == Some("1".to_string());
    if disable_web {
        println!("🌐 LIFE_SIMULATOR: Web server disabled via DISABLE_WEB_SERVER=1");
    } else {
        println!("🌐 LIFE_SIMULATOR: Starting web server...");
        let web_server_port = web_server_simple::start_simple_web_server();
        println!(
            "✅ LIFE_SIMULATOR: Web server started at http://127.0.0.1:{}",
            web_server_port
        );
    }

    // Insert world loader as a resource for systems to use
    commands.insert_resource(world_loader);
}

fn simulation_system(world_loader: Res<WorldLoader>) {
    // Basic simulation loop - runs once per frame
    // In a full implementation, this would handle entity updates, AI, etc.

    static mut FRAME_COUNT: u64 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT % 300 == 0 {
            // Every 5 seconds at 60 FPS
            println!(
                "🔄 LIFE_SIMULATOR: Simulation running - frame {} (world: {} chunks)",
                FRAME_COUNT,
                world_loader.get_chunk_count()
            );
        }
    }
}

fn save_load_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    world_loader: Res<WorldLoader>,
    world_config: Res<WorldConfig>,
) {
    // Save system - Press key 1 to save
    if keyboard.just_pressed(KeyCode::Digit1) {
        println!("💾 LIFE_SIMULATOR: Saving world...");

        // Create a unique save name with timestamp and world name
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let world_name = world_loader.get_name();
        let save_name = format!("{}_save_{}", world_name, timestamp);
        let file_path = format!("saves/{}.ron", save_name);

        commands.spawn(WorldSaveRequest {
            file_path: file_path.clone(),
            name: save_name.clone(),
        });

        println!(
            "✅ LIFE_SIMULATOR: Save request queued for '{}' -> {}",
            save_name, file_path
        );
    }

    // Load system - Press key 2 to load most recent save
    if keyboard.just_pressed(KeyCode::Digit2) {
        println!("📂 LIFE_SIMULATOR: Loading most recent world...");

        // Find the most recent save file
        if let Some(recent_save) = find_most_recent_save() {
            commands.spawn(WorldLoadRequest {
                file_path: recent_save.clone(),
            });

            println!("✅ LIFE_SIMULATOR: Load request queued for {}", recent_save);
        } else {
            println!("⚠️ LIFE_SIMULATOR: No save files found in saves/ directory");
        }
    }

    // List saves - Press key 3 to list available saves
    if keyboard.just_pressed(KeyCode::Digit3) {
        println!("📋 LIFE_SIMULATOR: Available save files:");
        list_save_files();
    }
}

fn find_most_recent_save() -> Option<String> {
    use std::fs;
    use std::path::Path;

    let saves_dir = Path::new("saves");
    if !saves_dir.exists() {
        return None;
    }

    let mut saves = Vec::new();

    if let Ok(entries) = fs::read_dir(saves_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "ron") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        saves.push((path.to_string_lossy().to_string(), modified));
                    }
                }
            }
        }
    }

    // Sort by modification time (newest first)
    saves.sort_by(|a, b| b.1.cmp(&a.1));

    saves.into_iter().map(|(path, _)| path).next()
}

fn list_save_files() {
    use std::fs;
    use std::path::Path;

    let saves_dir = Path::new("saves");
    if !saves_dir.exists() {
        println!("  No saves directory found");
        return;
    }

    if let Ok(entries) = fs::read_dir(saves_dir) {
        let mut count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "ron") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Some(filename) = path.file_stem() {
                            println!(
                                "  {} (modified: {:?})",
                                filename.to_string_lossy(),
                                modified
                            );
                            count += 1;
                        }
                    }
                }
            }
        }

        if count == 0 {
            println!("  No save files found");
        }
    }
}
