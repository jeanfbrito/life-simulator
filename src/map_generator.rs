use clap::Parser;
use rand;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

mod cached_world;
mod resources;
mod serialization;
mod tilemap;

use resources::ResourceGenerator;
use serialization::WorldSerializer;
use tilemap::{TerrainGenerationMode, WorldConfig, WorldGenerator};

/// Map Generator for Life Simulator
///
/// A standalone tool to generate complete worlds and save them to files.
/// The generated worlds can then be loaded by the life simulator engine.
#[derive(Parser, Debug)]
#[command(version, about = "Generate complete worlds for the Life Simulator")]
struct Args {
    /// Name of the world to generate
    #[arg(short, long, default_value = "generated_world")]
    name: String,

    /// Seed for world generation (random if not specified)
    #[arg(short, long)]
    seed: Option<u64>,

    /// World size in chunks (radius from center)
    #[arg(short, long, default_value = "5")]
    radius: i32,

    /// Output directory for generated worlds
    #[arg(short, long, default_value = "maps")]
    output_dir: String,

    /// Terrain generation mode: 'openrct2' (default, recommended) or 'island' (legacy)
    #[arg(short = 'm', long, default_value = "openrct2")]
    terrain_mode: String,

    /// Generate preview HTML file
    #[arg(long)]
    preview: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("Map Generator for Life Simulator");
        println!("=====================================");
    }

    // Generate or use provided seed
    let seed = args.seed.unwrap_or_else(|| {
        let random_seed = rand::random::<u64>();
        if args.verbose {
            println!("Using random seed: {}", random_seed);
        }
        random_seed
    });

    if args.verbose {
        println!("Generating world: {}", args.name);
        println!("Seed: {}", seed);
        println!("Radius: {} chunks", args.radius);
        println!("Output: {}", args.output_dir);
    }

    let start_time = Instant::now();

    // Create output directory
    if let Err(e) = fs::create_dir_all(&args.output_dir) {
        eprintln!(
            "Failed to create output directory '{}': {}",
            args.output_dir, e
        );
        std::process::exit(1);
    }

    // Parse terrain generation mode
    let terrain_mode = match args.terrain_mode.to_lowercase().as_str() {
        "openrct2" | "rct2" => TerrainGenerationMode::OpenRCT2Heights,
        "island" | "circular" => TerrainGenerationMode::CircularIsland,
        _ => {
            eprintln!(
                "Invalid terrain mode: '{}'. Valid options: 'openrct2' (default), 'island'",
                args.terrain_mode
            );
            std::process::exit(1);
        }
    };

    if args.verbose {
        let mode_name = match terrain_mode {
            TerrainGenerationMode::OpenRCT2Heights => "OpenRCT2 Heights",
            TerrainGenerationMode::CircularIsland => "Circular Island (legacy)",
        };
        println!("Terrain Mode: {}", mode_name);
    }

    // Initialize world generator
    let mut config = WorldConfig::default();
    config.seed = seed;
    config.terrain_generation_mode = terrain_mode;
    let world_generator = WorldGenerator::new(config);

    // Generate complete world data
    println!("Generating world chunks...");
    let mut multi_layer_chunks = HashMap::new();
    let total_chunks = ((args.radius * 2 + 1) * (args.radius * 2 + 1)) as usize;
    let mut generated_chunks = 0;

    for chunk_x in (-args.radius)..=(args.radius) {
        for chunk_y in (-args.radius)..=(args.radius) {
            // Generate terrain layer
            let terrain_tiles = world_generator.generate_procedural_chunk(chunk_x, chunk_y);

            // Generate resources layer
            let resources_tiles = ResourceGenerator::create_resources_for_chunk(
                &terrain_tiles,
                chunk_x,
                chunk_y,
                seed,
            );

            // Generate height layer
            let height_tiles = world_generator.generate_height_chunk(chunk_x, chunk_y);

            // Convert heights (Vec<Vec<u8>>) to Vec<Vec<String>> for serialization
            let height_tiles_str: Vec<Vec<String>> = height_tiles
                .iter()
                .map(|row| row.iter().map(|h| h.to_string()).collect())
                .collect();

            // Create multi-layer chunk
            let mut chunk_layers = HashMap::new();
            chunk_layers.insert("terrain".to_string(), terrain_tiles);
            chunk_layers.insert("resources".to_string(), resources_tiles);
            chunk_layers.insert("heights".to_string(), height_tiles_str);

            multi_layer_chunks.insert((chunk_x, chunk_y), chunk_layers);
            generated_chunks += 1;

            if args.verbose && generated_chunks % 10 == 0 {
                println!(
                    "Progress: {}/{} chunks ({}%)",
                    generated_chunks,
                    total_chunks,
                    (generated_chunks * 100) / total_chunks
                );
            }
        }
    }

    println!("Generated {} chunks", total_chunks);

    // Create serialized world
    println!("Serializing world data...");
    let serialized_world = WorldSerializer::create_serialized_world_from_layers(
        args.name.clone(),
        seed,
        WorldConfig::default(),
        multi_layer_chunks,
    );

    // Save to file
    let file_name = format!("{}.ron", args.name);
    let file_path = format!("{}/{}", args.output_dir, file_name);

    println!("Saving world to: {}", file_path);
    match WorldSerializer::save_world(&serialized_world, &file_path) {
        Ok(()) => {
            println!("World saved successfully!");
        }
        Err(e) => {
            eprintln!("Failed to save world: {}", e);
            std::process::exit(1);
        }
    }

    // Generate preview if requested
    if args.preview {
        // generate_preview(&args, &serialized_world, &file_path);  // Temporarily disabled due to compilation issues
        println!("Preview generation temporarily disabled");
    }

    let duration = start_time.elapsed();
    println!("Generation completed in: {:?}", duration);

    // Print summary
    println!("\nGeneration Summary:");
    println!("  World file: {}", file_path);
    println!(
        "  Chunks: {} ({}x{} area)",
        total_chunks,
        args.radius * 2 + 1,
        args.radius * 2 + 1
    );
    println!("  Seed: {}", seed);
    println!("  Time: {:?}", duration);

    if args.preview {
        let preview_path = format!("{}/{}_preview.html", args.output_dir, args.name);
        println!("  Preview: {}", preview_path);
    }
}
fn generate_preview(args: &Args, world: &serialization::SerializedWorld, world_file: &str) {
    println!("Preview generation temporarily disabled");
}
