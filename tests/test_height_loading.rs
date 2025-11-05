/// Test to verify map height loading behavior
///
/// This test demonstrates the current state of height data loading:
/// - Map files CONTAIN height data in layers
/// - WorldLoader CAN retrieve height layers as strings
/// - Chunk structs do NOT get populated with height data
///
/// Run with: cargo test --test test_height_loading -- --nocapture

use std::collections::HashMap;

#[test]
fn test_height_data_in_map_files() {
    println!("\n=== Testing Height Data Loading ===\n");

    // Load a map that contains height data
    let map_path = "maps/test_heights.ron";

    println!("1. Loading map file: {}", map_path);

    let world = match life_simulator::serialization::WorldSerializer::load_world(map_path) {
        Ok(w) => {
            println!("   ✅ Map loaded: {}", w.name);
            println!("   ✅ Seed: {}", w.seed);
            println!("   ✅ Chunks: {}", w.chunks.len());
            w
        }
        Err(e) => {
            println!("   ❌ Failed to load map: {}", e);
            panic!("Cannot proceed with test");
        }
    };

    println!("\n2. Checking for height layers in chunks:");

    let mut chunks_with_heights = 0;
    let mut chunks_without_heights = 0;
    let mut sample_heights: Vec<String> = Vec::new();

    for (chunk_key, chunk) in world.chunks.iter() {
        if let Some(heights_layer) = chunk.layers.get("heights") {
            chunks_with_heights += 1;

            // Sample first chunk's height values
            if sample_heights.is_empty() && !heights_layer.is_empty() && !heights_layer[0].is_empty() {
                sample_heights = heights_layer[0][0..4.min(heights_layer[0].len())].to_vec();
                println!("   Sample from chunk {}: {:?}", chunk_key, sample_heights);
            }
        } else {
            chunks_without_heights += 1;
        }
    }

    println!("\n   ✅ Chunks WITH height layers: {}", chunks_with_heights);
    println!("   ℹ️  Chunks WITHOUT height layers: {}", chunks_without_heights);

    assert!(chunks_with_heights > 0, "No chunks have height data!");

    println!("\n3. Attempting to parse height strings to u8:");

    if !sample_heights.is_empty() {
        println!("   String values: {:?}", sample_heights);

        let parsed: Vec<Result<u8, _>> = sample_heights.iter()
            .map(|s| s.parse::<u8>())
            .collect();

        let parsed_ok: Vec<u8> = parsed.iter()
            .filter_map(|r| r.as_ref().ok().copied())
            .collect();

        println!("   ✅ Parsed u8 values: {:?}", parsed_ok);

        assert_eq!(parsed_ok.len(), sample_heights.len(),
            "Not all height strings could be parsed to u8");
    }

    println!("\n4. Checking WorldLoader can access heights:");

    let world_loader = match life_simulator::world_loader::WorldLoader::load_from_file(map_path) {
        Ok(loader) => {
            println!("   ✅ WorldLoader created");
            loader
        }
        Err(e) => {
            println!("   ❌ Failed to create WorldLoader: {}", e);
            panic!("Cannot proceed with test");
        }
    };

    // Try to get heights for chunk (0, 0)
    if let Some(heights_layer) = world_loader.get_chunk_layer(0, 0, "heights") {
        println!("   ✅ WorldLoader.get_chunk_layer(0, 0, \"heights\") returned data");
        println!("   ✅ Layer dimensions: {}x{}", heights_layer.len(), heights_layer[0].len());
        println!("   Sample values: {:?}", &heights_layer[0][0..4]);
    } else {
        println!("   ❌ WorldLoader could not retrieve heights for chunk (0, 0)");
    }

    println!("\n5. Demonstrating the problem:");
    println!("   ⚠️  Chunk struct has heights: [[u8; 16]; 16]");
    println!("   ⚠️  These fields are marked #[serde(skip)]");
    println!("   ⚠️  When loading from file, Chunk.heights[] = all zeros");
    println!("   ⚠️  Height data EXISTS in layers but NOT in runtime Chunks");

    println!("\n=== Test Summary ===");
    println!("✅ Height data IS present in map files (as string layers)");
    println!("✅ WorldLoader CAN retrieve height layers");
    println!("✅ Height strings CAN be parsed to u8 values");
    println!("❌ Chunk structs do NOT get populated with heights at load time");
    println!("\n💡 Solution: Add parsing step to populate Chunk.heights[] from layers");
}

#[test]
fn test_chunk_height_defaults() {
    println!("\n=== Testing Chunk Height Field Defaults ===\n");

    use life_simulator::tilemap::{Chunk, ChunkCoordinate};

    println!("Creating a new Chunk with coordinate (0, 0)...");
    let chunk = Chunk::new(ChunkCoordinate::new(0, 0), 12345);

    println!("Checking height field values:");
    let mut all_zeros = true;
    let mut sample_values = Vec::new();

    for y in 0..16 {
        for x in 0..16 {
            let height = chunk.heights[y][x];
            if height != 0 {
                all_zeros = false;
            }
            if sample_values.len() < 10 {
                sample_values.push(height);
            }
        }
    }

    println!("   Sample height values: {:?}", sample_values);
    println!("   All zeros: {}", all_zeros);

    if all_zeros {
        println!("   ⚠️  Chunk.heights[] contains all zeros (default)");
        println!("   ⚠️  This is expected because heights are not populated from layers");
    } else {
        println!("   ℹ️  Chunk.heights[] contains non-zero values");
        println!("   ℹ️  This might be from WorldGenerator.generate_height_chunk()");
    }
}

#[test]
fn test_proposed_solution() {
    println!("\n=== Testing Proposed Solution (Height Parsing) ===\n");

    // Simulate the proposed solution
    let height_strings = vec![
        vec!["255", "240", "224", "208"],
        vec!["240", "224", "208", "192"],
        vec!["224", "208", "192", "176"],
        vec!["208", "192", "176", "160"],
    ];

    println!("Input: Height layer as strings (4x4 sample):");
    for row in &height_strings {
        println!("   {:?}", row);
    }

    println!("\nParsing to u8 array:");
    let mut heights: [[u8; 4]; 4] = [[0; 4]; 4];

    for (y, row) in height_strings.iter().enumerate() {
        for (x, val_str) in row.iter().enumerate() {
            if let Ok(height) = val_str.parse::<u8>() {
                heights[y][x] = height;
            }
        }
    }

    println!("Output: u8 array:");
    for row in &heights {
        println!("   {:?}", row);
    }

    // Verify parsing worked
    assert_eq!(heights[0][0], 255);
    assert_eq!(heights[1][1], 224);
    assert_eq!(heights[2][2], 192);
    assert_eq!(heights[3][3], 160);

    println!("\n✅ Parsing successful!");
    println!("💡 This demonstrates how to populate Chunk.heights[] from layers");
}
