use std::collections::HashMap;
use std::env;
use std::process;

// Import the world generation and serialization modules
use life_simulator::{
    tilemap::{WorldGenerator, WorldConfig, CHUNK_SIZE},
    serialization::{WorldSerializer, SerializedWorld},
    resources::ResourceGenerator,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} generate <file_name> <map_name> [seed]", args[0]);
        eprintln!("  {} list", args[0]);
        eprintln!("");
        eprintln!("Examples:");
        eprintln!("  {} generate my_world MyIsland 12345", args[0]);
        eprintln!("  {} generate forest_map Forest", args[0]);
        eprintln!("  {} list", args[0]);
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "generate" => {
            if args.len() < 4 {
                eprintln!("Error: generate command requires file_name and map_name");
                eprintln!("Usage: {} generate <file_name> <map_name> [seed]", args[0]);
                process::exit(1);
            }

            let file_name = &args[2];
            let map_name = &args[3];
            let seed = if args.len() > 4 {
                args[4].parse::<u64>().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid seed '{}', using default", args[4]);
                    12345
                })
            } else {
                12345 // Default seed
            };

            println!("🗺️  Generating world '{}' with seed {}...", map_name, seed);

            match generate_world(file_name, map_name, seed) {
                Ok(()) => {
                    println!("✅ World '{}' successfully generated and saved as '{}'", map_name, file_name);
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate world: {}", e);
                    process::exit(1);
                }
            }
        }
        "list" => {
            match list_saved_worlds() {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("❌ Failed to list worlds: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            eprintln!("Available commands: generate, list");
            process::exit(1);
        }
    }
}

fn generate_world(file_name: &str, map_name: &str, seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Create world generator with specified seed
    let mut world_generator = WorldGenerator::new(WorldConfig::default());
    world_generator.set_seed(seed);

    println!("🌍 Generating terrain chunks...");

    // Generate chunks around center for saving
    let center_x = 0;
    let center_y = 0;
    let radius = 12; // Save 25x25 chunk area around center (400x400 tiles)

    let total_chunks = ((radius * 2 + 1) * (radius * 2 + 1)) as usize;
    let mut generated_chunks = 0;

    // Multi-layer chunks: terrain + resources
    let mut multi_layer_chunks = HashMap::new();

    for chunk_x in (center_x - radius)..=(center_x + radius) {
        for chunk_y in (center_y - radius)..=(center_y + radius) {
            // Generate terrain layer
            let terrain_layer = world_generator.generate_procedural_chunk(chunk_x, chunk_y);

            // Generate resource layer based on terrain
            let resource_layer = ResourceGenerator::create_resources_for_chunk(
                &terrain_layer,
                chunk_x,
                chunk_y,
                world_generator.get_seed()
            );

            // Create multi-layer chunk
            let mut layers = HashMap::new();
            layers.insert("terrain".to_string(), terrain_layer);
            layers.insert("resources".to_string(), resource_layer);

            multi_layer_chunks.insert((chunk_x, chunk_y), layers);
            generated_chunks += 1;

            // Show progress
            print!("\r📦 Progress: {}/{} chunks ({:.1}%)",
                   generated_chunks, total_chunks,
                   (generated_chunks as f32 / total_chunks as f32) * 100.0);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }

    println!("\n🔧 Creating serialized world data with multi-layer support...");

    let serialized_world = WorldSerializer::create_serialized_world_from_layers(
        map_name.to_string(),
        seed,
        WorldConfig::default(),
        multi_layer_chunks,
    );

    println!("💾 Saving world to 'saves/{}.ron'...", file_name);

    let full_path = format!("saves/{}.ron", file_name);
    match WorldSerializer::save_world(&serialized_world, &full_path) {
        Ok(()) => {
            println!("📊 World statistics:");
            println!("   - Name: {}", serialized_world.name);
            println!("   - Seed: {}", serialized_world.seed);
            println!("   - Chunks: {}", serialized_world.chunks.len());
            println!("   - World size: {}x{} tiles",
                     serialized_world.chunks.len() as i32 * (CHUNK_SIZE as i32) * 2 + 1,
                     serialized_world.chunks.len() as i32 * (CHUNK_SIZE as i32) * 2 + 1);
            println!("   - File: {}", full_path);
        }
        Err(e) => {
            return Err(format!("Failed to save world: {}", e).into());
        }
    }

    Ok(())
}

fn list_saved_worlds() -> Result<(), Box<dyn std::error::Error>> {
    println!("📂 Listing saved worlds:");

    let saves_dir = std::path::Path::new("saves");

    // Check if saves directory exists
    if !saves_dir.exists() {
        println!("   No saves directory found. No worlds generated yet.");
        return Ok(());
    }

    let mut worlds = Vec::new();

    for entry in std::fs::read_dir(saves_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ron") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Try to load the world to get more info
                match WorldSerializer::load_world(&path.to_string_lossy()) {
                    Ok(world) => {
                        worlds.push((file_stem.to_string(), world));
                    }
                    Err(_) => {
                        worlds.push((file_stem.to_string(),
                                   SerializedWorld {
                                       name: "Unknown".to_string(),
                                       seed: 0,
                                       config: WorldConfig::default(),
                                       chunks: HashMap::new(),
                                       version: "Unknown".to_string(),
                                   }));
                    }
                }
            }
        }
    }

    if worlds.is_empty() {
        println!("   No saved worlds found.");
    } else {
        worlds.sort_by(|a, b| a.0.cmp(&b.0));
        println!("   {:<15} {:<20} {:<10} {:<15}", "File", "Name", "Seed", "Chunks");
        println!("   {:<15} {:<20} {:<10} {:<15}",
                "----", "----", "----", "------");

        for (file_name, world) in worlds {
            println!("   {:<15} {:<20} {:<10} {:<15}",
                     file_name, world.name, world.seed, world.chunks.len());
        }
    }

    Ok(())
}