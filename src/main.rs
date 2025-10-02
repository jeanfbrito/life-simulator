use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;

mod tilemap;
mod web;
mod serialization;
mod cached_world;
mod resources;
mod world_loader;
mod pathfinding;
mod entities;
mod simulation;

use tilemap::{TilemapPlugin, WorldConfig};
use serialization::{WorldSerializationPlugin, WorldSaveRequest, WorldLoadRequest};
use cached_world::CachedWorldPlugin;
use world_loader::WorldLoader;
use pathfinding::{PathfindingGrid, process_pathfinding_requests};
use entities::{EntitiesPlugin, spawn_wandering_people};
use simulation::SimulationPlugin;

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
        .add_plugins(SimulationPlugin)  // Tick system
        .add_plugins(EntitiesPlugin)     // Movement & AI
        .insert_resource(WorldConfig::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<PathfindingGrid>()
        .add_systems(Startup, (setup, spawn_wanderers.after(setup)))
        .add_systems(Update, (
            run_fixed_update_schedule,     // Manually run FixedUpdate
            process_pathfinding_requests,  // Async pathfinding
            simulation_system,
            save_load_system.after(simulation_system),
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
) {
    println!("🔧 LIFE_SIMULATOR: Setting up headless life simulation");

    // Load the world
    println!("🗺️ LIFE_SIMULATOR: Loading world...");
    let world_loader = match WorldLoader::load_default() {
        Ok(loader) => {
            println!("✅ LIFE_SIMULATOR: World loaded: {} (seed: {})", loader.get_name(), loader.get_seed());
            loader
        }
        Err(e) => {
            eprintln!("❌ LIFE_SIMULATOR: Failed to load world: {}", e);
            eprintln!("💡 LIFE_SIMULATOR: Please generate a world first using: cargo run --bin map_generator");
            std::process::exit(1);
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
            
            // Check if there's a resource blocking this tile
            let has_resource = world_loader.get_resource_at(x, y)
                .map(|r| !r.is_empty())
                .unwrap_or(false);
            
            // If terrain is passable but has resource, make it impassable
            let final_cost = if has_resource && terrain_cost != u32::MAX {
                tiles_blocked += 1;
                u32::MAX // Resources block movement
            } else {
                terrain_cost
            };
            
            pathfinding_grid.set_cost(pos, final_cost);
            tiles_processed += 1;
        }
    }
    
    println!("✅ LIFE_SIMULATOR: Pathfinding grid ready");
    println!("   📊 Processed {} tiles, {} blocked by resources", tiles_processed, tiles_blocked);

    // Start the web server
    println!("🌐 LIFE_SIMULATOR: Starting web server...");
    web_server_simple::start_simple_web_server();
    println!("✅ LIFE_SIMULATOR: Web server started at http://127.0.0.1:54321");

    // Insert world loader as a resource for systems to use
    commands.insert_resource(world_loader);
}

fn spawn_wanderers(
    mut commands: Commands,
    pathfinding_grid: Res<PathfindingGrid>,
) {
    println!("🚶 LIFE_SIMULATOR: Spawning wandering people...");
    
    // Spawn 3 wandering people around the origin
    let entities = spawn_wandering_people(
        &mut commands,
        3,                    // Count
        bevy::math::IVec2::ZERO,  // Center
        20,                   // Spawn radius
        30,                   // Wander radius
        &pathfinding_grid,
    );
    
    println!("✅ LIFE_SIMULATOR: Spawned {} wandering people", entities.len());
    println!("🌐 LIFE_SIMULATOR: View them at http://127.0.0.1:54321/viewer.html");
    println!("🌐 LIFE_SIMULATOR: Entity API at http://127.0.0.1:54321/api/entities");
}

/// System that manually runs the FixedUpdate schedule
/// This is needed because ScheduleRunnerPlugin doesn't run it automatically
fn run_fixed_update_schedule(world: &mut World) {
    world.run_schedule(FixedUpdate);
}

fn simulation_system(
    world_loader: Res<WorldLoader>,
) {
    // Basic simulation loop - runs once per frame
    // In a full implementation, this would handle entity updates, AI, etc.

    static mut FRAME_COUNT: u64 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT % 300 == 0 { // Every 5 seconds at 60 FPS
            println!("🔄 LIFE_SIMULATOR: Simulation running - frame {} (world: {} chunks)",
                FRAME_COUNT, world_loader.get_chunk_count());
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

        println!("✅ LIFE_SIMULATOR: Save request queued for '{}' -> {}", save_name, file_path);
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
                            println!("  {} (modified: {:?})",
                                filename.to_string_lossy(),
                                modified);
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
