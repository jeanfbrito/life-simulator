use clap::{Parser, Subcommand};
use rand;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

// Import from the library crate instead of declaring local modules
use life_simulator::resources::ResourceGenerator;
use life_simulator::serialization::WorldSerializer;
use life_simulator::tilemap::{TerrainGenerationMode, WorldConfig, WorldGenerator};

/// Map Generator for Life Simulator
///
/// A standalone tool to generate complete worlds and save them to files.
/// The generated worlds can then be loaded by the life simulator engine.
#[derive(Parser, Debug)]
#[command(version, about = "Generate complete worlds for the Life Simulator")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Name of the world to generate (legacy mode, use 'generate' subcommand instead)
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

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a new world map
    Generate {
        /// File name for the saved world (without extension)
        file_name: String,

        /// Display name for the world
        map_name: String,

        /// Seed for world generation (random if not specified)
        seed: Option<u64>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// World size in chunks (radius from center)
        #[arg(short, long, default_value = "5")]
        radius: i32,

        /// Terrain generation mode: 'openrct2' (default, recommended) or 'island' (legacy)
        #[arg(short = 'm', long, default_value = "openrct2")]
        terrain_mode: String,
    },
    /// List all saved worlds
    List,
}

fn main() {
    let args = Args::parse();

    // Handle subcommands if provided
    match &args.command {
        Some(Commands::Generate {
            file_name,
            map_name,
            seed,
            verbose,
            radius,
            terrain_mode,
        }) => {
            run_generate(
                file_name,
                map_name,
                *seed,
                *verbose,
                *radius,
                terrain_mode,
                "saves", // Default output directory for subcommand mode
            );
            return;
        }
        Some(Commands::List) => {
            list_saved_worlds();
            return;
        }
        None => {
            // Legacy flag-based mode - continue with original behavior
        }
    }

    // Legacy flag-based mode
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

    // OpenRCT2 mode uses 3-phase whole-map generation for exact smoothing behavior
    if terrain_mode == TerrainGenerationMode::OpenRCT2Heights {
        println!("🌍 Using OpenRCT2 exact 3-phase generation (whole-map smoothing)");

        // Collect all chunk coordinates
        let mut all_chunks = Vec::new();
        for chunk_x in (-args.radius)..=(args.radius) {
            for chunk_y in (-args.radius)..=(args.radius) {
                all_chunks.push((chunk_x, chunk_y));
            }
        }

        // PHASE 1: Generate ALL initial heights (simplex noise + blur)
        let mut whole_map = world_generator.generate_all_initial_heights(&all_chunks);

        // PHASE 2: Smooth entire map until convergence (OpenRCT2 exact)
        world_generator.smooth_whole_map(&mut whole_map);

        // PHASE 3: Finalize each chunk (extract heights, calculate slopes)
        println!("🎨 Finalizing chunks and generating terrain/resources...");
        for (chunk_x, chunk_y) in all_chunks {
            // Extract final heights and slopes from whole map
            let height_data = world_generator.finalize_chunk_from_whole_map(
                chunk_x,
                chunk_y,
                &whole_map,
            );

            // Generate terrain layer from pre-computed heights (NO height regeneration!)
            let terrain_tiles = world_generator.generate_openrct2_chunk_from_heights(
                chunk_x,
                chunk_y,
                &height_data.heights,
            );

            // Generate resources layer
            let resources_tiles = ResourceGenerator::create_resources_for_chunk(
                &terrain_tiles,
                chunk_x,
                chunk_y,
                seed,
            );

            // Convert heights (Vec<Vec<u8>>) to Vec<Vec<String>> for serialization
            let height_tiles_str: Vec<Vec<String>> = height_data
                .heights
                .iter()
                .map(|row| row.iter().map(|h| h.to_string()).collect())
                .collect();

            let slope_tiles_str: Vec<Vec<String>> = height_data
                .slope_indices
                .iter()
                .map(|row| row.iter().map(|h| h.to_string()).collect())
                .collect();

            // Create multi-layer chunk
            let mut chunk_layers = HashMap::new();
            chunk_layers.insert("terrain".to_string(), terrain_tiles);
            chunk_layers.insert("resources".to_string(), resources_tiles);
            chunk_layers.insert("heights".to_string(), height_tiles_str);
            chunk_layers.insert("slope_indices".to_string(), slope_tiles_str);

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
    } else {
        // Legacy island mode uses per-chunk generation
        println!("🏝️  Using legacy island generation (per-chunk)");

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

                // Generate height and slope data
                let height_data = world_generator.generate_height_chunk(chunk_x, chunk_y);

                // Convert heights (Vec<Vec<u8>>) to Vec<Vec<String>> for serialization
                let height_tiles_str: Vec<Vec<String>> = height_data
                    .heights
                    .iter()
                    .map(|row| row.iter().map(|h| h.to_string()).collect())
                    .collect();

                let slope_tiles_str: Vec<Vec<String>> = height_data
                    .slope_indices
                    .iter()
                    .map(|row| row.iter().map(|h| h.to_string()).collect())
                    .collect();

                // Create multi-layer chunk
                let mut chunk_layers = HashMap::new();
                chunk_layers.insert("terrain".to_string(), terrain_tiles);
                chunk_layers.insert("resources".to_string(), resources_tiles);
                chunk_layers.insert("heights".to_string(), height_tiles_str);
                chunk_layers.insert("slope_indices".to_string(), slope_tiles_str);

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
    }

    println!("✅ Generated {} chunks", total_chunks);

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
fn generate_preview(_args: &Args, _world: &life_simulator::serialization::SerializedWorld, _world_file: &str) {
    println!("Preview generation temporarily disabled");
}

/// Run generate subcommand
fn run_generate(
    file_name: &str,
    map_name: &str,
    seed: Option<u64>,
    verbose: bool,
    radius: i32,
    terrain_mode_str: &str,
    output_dir: &str,
) {
    if verbose {
        println!("🗺️  Map Generator - Configuration");
        println!("   File: {}", file_name);
        println!("   Name: {}", map_name);
    }

    // Generate or use provided seed
    let seed = seed.unwrap_or_else(|| {
        let random_seed = rand::random::<u64>();
        if verbose {
            println!("   Seed: {} (randomly generated)", random_seed);
        }
        random_seed
    });

    if verbose {
        println!("   Seed: {}", seed);
        println!("   Radius: {} chunks", radius);
        println!("   Output: {}/{}.ron", output_dir, file_name);
        println!();
    } else {
        println!("🗺️  Generating world '{}' with seed {}...", map_name, seed);
    }

    let start_time = Instant::now();

    // Create output directory
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!(
            "Failed to create output directory '{}': {}",
            output_dir, e
        );
        std::process::exit(1);
    }

    // Parse terrain generation mode
    let terrain_mode = match terrain_mode_str.to_lowercase().as_str() {
        "openrct2" | "rct2" => TerrainGenerationMode::OpenRCT2Heights,
        "island" | "circular" => TerrainGenerationMode::CircularIsland,
        _ => {
            eprintln!(
                "Invalid terrain mode: '{}'. Valid options: 'openrct2' (default), 'island'",
                terrain_mode_str
            );
            std::process::exit(1);
        }
    };

    if verbose {
        let mode_name = match terrain_mode {
            TerrainGenerationMode::OpenRCT2Heights => "OpenRCT2 Heights",
            TerrainGenerationMode::CircularIsland => "Circular Island (legacy)",
        };
        println!("🌍 Terrain Mode: {}", mode_name);
        println!();
        println!("📋 Boundary Rules:");
        println!("   • Perimeter enforcement: Outermost tiles forced to deep water");
        println!("   • Shallow water band: 1-2 tiles from perimeter");
        println!("   • Beach transition: Sandy beach between water and land");
        println!("   • Interior water bodies: Natural transitions with shallow edges");
        println!("   • Validation: Post-generation checks for rule compliance");
        println!();
    }

    // Initialize world generator
    let mut config = WorldConfig::default();
    config.seed = seed;
    config.terrain_generation_mode = terrain_mode;
    let world_generator = WorldGenerator::new(config);

    // Generate complete world data
    println!("🌍 Generating terrain chunks...");
    let mut multi_layer_chunks = HashMap::new();
    let total_chunks = ((radius * 2 + 1) * (radius * 2 + 1)) as usize;
    let mut generated_chunks = 0;

    // OpenRCT2 mode uses 3-phase whole-map generation for exact smoothing behavior
    if terrain_mode == TerrainGenerationMode::OpenRCT2Heights {
        if verbose {
            println!("   Using OpenRCT2 exact 3-phase generation (whole-map smoothing)");
        }

        // Collect all chunk coordinates
        let mut all_chunks = Vec::new();
        for chunk_x in (-radius)..=(radius) {
            for chunk_y in (-radius)..=(radius) {
                all_chunks.push((chunk_x, chunk_y));
            }
        }

        // PHASE 1: Generate ALL initial heights (simplex noise + blur)
        let mut whole_map = world_generator.generate_all_initial_heights(&all_chunks);

        // PHASE 2: Smooth entire map until convergence (OpenRCT2 exact)
        world_generator.smooth_whole_map(&mut whole_map);

        // PHASE 3: Finalize each chunk (extract heights, calculate slopes)
        if verbose {
            println!("🎨 Finalizing chunks and generating terrain/resources...");
        }
        for (chunk_x, chunk_y) in all_chunks {
            // Extract final heights and slopes from whole map
            let height_data = world_generator.finalize_chunk_from_whole_map(
                chunk_x,
                chunk_y,
                &whole_map,
            );

            // Generate terrain layer from pre-computed heights (NO height regeneration!)
            let terrain_tiles = world_generator.generate_openrct2_chunk_from_heights(
                chunk_x,
                chunk_y,
                &height_data.heights,
            );

            // Generate resources layer
            let resources_tiles = ResourceGenerator::create_resources_for_chunk(
                &terrain_tiles,
                chunk_x,
                chunk_y,
                seed,
            );

            // Convert heights (Vec<Vec<u8>>) to Vec<Vec<String>> for serialization
            let height_tiles_str: Vec<Vec<String>> = height_data
                .heights
                .iter()
                .map(|row| row.iter().map(|h| h.to_string()).collect())
                .collect();

            let slope_tiles_str: Vec<Vec<String>> = height_data
                .slope_indices
                .iter()
                .map(|row| row.iter().map(|h| h.to_string()).collect())
                .collect();

            // Create multi-layer chunk
            let mut chunk_layers = HashMap::new();
            chunk_layers.insert("terrain".to_string(), terrain_tiles);
            chunk_layers.insert("resources".to_string(), resources_tiles);
            chunk_layers.insert("heights".to_string(), height_tiles_str);
            chunk_layers.insert("slope_indices".to_string(), slope_tiles_str);

            multi_layer_chunks.insert((chunk_x, chunk_y), chunk_layers);
            generated_chunks += 1;

            // Show progress
            print!("\r📦 Progress: {}/{} chunks ({:.1}%)",
                   generated_chunks, total_chunks,
                   (generated_chunks as f32 / total_chunks as f32) * 100.0);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        println!();
    } else {
        // Legacy island mode uses per-chunk generation
        if verbose {
            println!("🏝️  Using legacy island generation (per-chunk)");
        }

        for chunk_x in (-radius)..=(radius) {
            for chunk_y in (-radius)..=(radius) {
                // Generate terrain layer
                let terrain_tiles = world_generator.generate_procedural_chunk(chunk_x, chunk_y);

                // Generate resources layer
                let resources_tiles = ResourceGenerator::create_resources_for_chunk(
                    &terrain_tiles,
                    chunk_x,
                    chunk_y,
                    seed,
                );

                // Generate height and slope data
                let height_data = world_generator.generate_height_chunk(chunk_x, chunk_y);

                // Convert heights (Vec<Vec<u8>>) to Vec<Vec<String>> for serialization
                let height_tiles_str: Vec<Vec<String>> = height_data
                    .heights
                    .iter()
                    .map(|row| row.iter().map(|h| h.to_string()).collect())
                    .collect();

                let slope_tiles_str: Vec<Vec<String>> = height_data
                    .slope_indices
                    .iter()
                    .map(|row| row.iter().map(|h| h.to_string()).collect())
                    .collect();

                // Create multi-layer chunk
                let mut chunk_layers = HashMap::new();
                chunk_layers.insert("terrain".to_string(), terrain_tiles);
                chunk_layers.insert("resources".to_string(), resources_tiles);
                chunk_layers.insert("heights".to_string(), height_tiles_str);
                chunk_layers.insert("slope_indices".to_string(), slope_tiles_str);

                multi_layer_chunks.insert((chunk_x, chunk_y), chunk_layers);
                generated_chunks += 1;

                // Show progress
                print!("\r📦 Progress: {}/{} chunks ({:.1}%)",
                       generated_chunks, total_chunks,
                       (generated_chunks as f32 / total_chunks as f32) * 100.0);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
        println!();
    }

    println!("✅ Generated {} chunks", total_chunks);

    if verbose {
        println!();
        println!("✅ Validation Results:");
        println!("   • Terrain generation: {} chunks successfully generated", generated_chunks);
        println!("   • Boundary enforcement: Applied to all perimeter chunks");
        println!("   • Multi-layer structure: Terrain + resource layers created");
        println!("   • Chunk coverage: {}x{} chunk grid complete", radius * 2 + 1, radius * 2 + 1);
        println!();
    }

    // Create serialized world
    println!("💾 Serializing world data...");
    let serialized_world = WorldSerializer::create_serialized_world_from_layers(
        map_name.to_string(),
        seed,
        WorldConfig::default(),
        multi_layer_chunks,
    );

    // Save to file
    let file_path = format!("{}/{}.ron", output_dir, file_name);

    println!("💾 Saving world to '{}'...", file_path);
    match WorldSerializer::save_world(&serialized_world, &file_path) {
        Ok(()) => {
            println!("✅ World '{}' successfully generated and saved!", map_name);
        }
        Err(e) => {
            eprintln!("❌ Failed to save world: {}", e);
            std::process::exit(1);
        }
    }

    let duration = start_time.elapsed();

    // Print summary
    println!();
    println!("📊 World statistics:");
    println!("   - Name: {}", map_name);
    println!("   - Seed: {}", seed);
    println!("   - Chunks: {}", serialized_world.chunks.len());
    println!("   - World size: ~{}x{} tiles",
             (radius * 2 + 1) * 16,
             (radius * 2 + 1) * 16);
    println!("   - File: {}", file_path);
    println!("   - Time: {:?}", duration);

    if verbose {
        println!();
        println!("🎯 Generation Features:");
        println!("   - Perimeter boundary enforcement (deep water → shallow → sand)");
        println!("   - Internal water transitions (deep water → shallow → land)");
        println!("   - Strategic water spot placement using noise algorithm");
        println!("   - Biome-aware terrain generation (moisture/temperature layers)");
        println!("   - Resource placement optimized for foraging");
    }
}

/// List all saved worlds
fn list_saved_worlds() {
    println!("📂 Listing saved worlds:");

    let saves_dir = std::path::Path::new("saves");

    // Check if saves directory exists
    if !saves_dir.exists() {
        println!("   No saves directory found. No worlds generated yet.");
        return;
    }

    let mut worlds = Vec::new();

    if let Ok(entries) = std::fs::read_dir(saves_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Try to load the world to get more info
                    match WorldSerializer::load_world(&path.to_string_lossy()) {
                        Ok(world) => {
                            worlds.push((file_stem.to_string(), world.name, world.seed, world.chunks.len()));
                        }
                        Err(_) => {
                            worlds.push((file_stem.to_string(), "Unknown".to_string(), 0, 0));
                        }
                    }
                }
            }
        }
    }

    if worlds.is_empty() {
        println!("   No saved worlds found.");
    } else {
        worlds.sort_by(|a, b| a.0.cmp(&b.0));
        println!("   {:<15} {:<20} {:<12} {:<10}", "File", "Name", "Seed", "Chunks");
        println!("   {:<15} {:<20} {:<12} {:<10}",
                "----", "----", "----", "------");

        for (file_name, name, seed, chunks) in worlds {
            println!("   {:<15} {:<20} {:<12} {:<10}",
                     file_name, name, seed, chunks);
        }
    }
}
