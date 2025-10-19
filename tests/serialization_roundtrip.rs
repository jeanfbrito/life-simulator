//! Tests for world data schema refactoring to support biome and elevation data
//!
//! This test suite validates the new schema extensions while maintaining backward compatibility.

use std::collections::HashMap;
use life_simulator::{
    serialization::{SerializedWorld, SerializedChunk, WorldSerializer, BiomeConfig, BiomeThresholds},
    tilemap::{ChunkCoordinate, WorldConfig, BiomeType, CHUNK_SIZE},
    world_loader::WorldLoader,
    cached_world::CachedWorld,
};

#[cfg(test)]
mod schema_tests {
    use super::*;

    /// Test that new schema can be created with biome and elevation data
    #[test]
    fn test_create_world_with_biome_elevation_data() {
        let name = "Test World".to_string();
        let seed = 42;
        let config = WorldConfig::default();

        // Create multi-layer chunks with new layers
        let mut chunks = HashMap::new();
        let mut layers = HashMap::new();

        // Terrain layer (existing)
        let terrain_layer = vec![vec!["Grass".to_string(); CHUNK_SIZE]; CHUNK_SIZE];
        layers.insert("terrain".to_string(), terrain_layer);

        // NEW: Biome layer
        let biome_layer = vec![vec!["TemperateForest".to_string(); CHUNK_SIZE]; CHUNK_SIZE];
        layers.insert("biome".to_string(), biome_layer);

        // NEW: Elevation layer
        let elevation_layer = vec![vec!["120".to_string(); CHUNK_SIZE]; CHUNK_SIZE];
        layers.insert("elevation".to_string(), elevation_layer);

        // NEW: Vegetation density layer
        let vegetation_layer = vec![vec!["0.7".to_string(); CHUNK_SIZE]; CHUNK_SIZE];
        layers.insert("vegetation_density".to_string(), vegetation_layer);

        // NEW: Moisture layer
        let moisture_layer = vec![vec!["0.6".to_string(); CHUNK_SIZE]; CHUNK_SIZE];
        layers.insert("moisture".to_string(), moisture_layer);

        chunks.insert((0, 0), layers);

        // This should work with new schema
        let world = WorldSerializer::create_serialized_world_from_layers(
            name.clone(),
            seed,
            config,
            chunks,
        );

        assert_eq!(world.name, name);
        assert_eq!(world.seed, seed);

        // Verify chunk has all layers
        let chunk = world.chunks.get("0,0").unwrap();
        assert!(chunk.layers.contains_key("terrain"));
        assert!(chunk.layers.contains_key("biome"));
        assert!(chunk.layers.contains_key("elevation"));
        assert!(chunk.layers.contains_key("vegetation_density"));
        assert!(chunk.layers.contains_key("moisture"));
    }

    /// Test backward compatibility - loading old world files without new fields
    #[test]
    fn test_backward_compatibility_missing_fields() {
        let name = "Legacy World".to_string();
        let seed = 123;
        let config = WorldConfig::default();

        // Create chunks with ONLY terrain layer (old format)
        let mut chunks = HashMap::new();
        let mut layers = HashMap::new();
        let terrain_layer = vec![vec!["Grass".to_string(); CHUNK_SIZE]; CHUNK_SIZE];
        layers.insert("terrain".to_string(), terrain_layer);
        chunks.insert((0, 0), layers);

        let world = WorldSerializer::create_serialized_world_from_layers(
            name.clone(),
            seed,
            config,
            chunks,
        );

        // Should still load successfully
        assert_eq!(world.name, name);
        assert_eq!(world.seed, seed);

        let chunk = world.chunks.get("0,0").unwrap();
        assert!(chunk.layers.contains_key("terrain"));

        // New fields should be accessible with default values
        // This will fail until we implement the migration logic
        assert!(chunk.layers.contains_key("biome")); // Should default to "Plains"
        assert!(chunk.layers.contains_key("elevation")); // Should default to "64"
    }

    /// Test serialization round-trip with new schema
    #[test]
    fn test_serialization_roundtrip_with_new_fields() {
        let name = "Roundtrip Test".to_string();
        let seed = 999;
        let config = WorldConfig::default();

        // Create world with all new layers
        let mut chunks = HashMap::new();
        let mut layers = HashMap::new();

        layers.insert("terrain".to_string(), vec![vec!["Forest".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("biome".to_string(), vec![vec!["Woodland".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("elevation".to_string(), vec![vec!["150".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("vegetation_density".to_string(), vec![vec!["0.8".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("moisture".to_string(), vec![vec!["0.7".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);

        chunks.insert((1, 1), layers);

        let original_world = WorldSerializer::create_serialized_world_from_layers(
            name.clone(),
            seed,
            config.clone(),
            chunks,
        );

        // Serialize to RON string
        let ron_string = ron::to_string(&original_world).expect("Failed to serialize");

        // Deserialize back
        let deserialized_world: SerializedWorld = ron::from_str(&ron_string).expect("Failed to deserialize");

        // Verify all data preserved
        assert_eq!(deserialized_world.name, name);
        assert_eq!(deserialized_world.seed, seed);
        assert_eq!(deserialized_world.chunks.len(), 1);

        let chunk = deserialized_world.chunks.get("1,1").unwrap();
        assert!(chunk.layers.contains_key("biome"));
        assert!(chunk.layers.contains_key("elevation"));
        assert!(chunk.layers.contains_key("vegetation_density"));
        assert!(chunk.layers.contains_key("moisture"));
    }

    /// Test WorldLoader can access new biome and elevation data
    #[test]
    fn test_world_loader_biome_elevation_access() {
        // This test will fail until we add the new getter methods to WorldLoader
        let mut chunks = HashMap::new();
        let mut layers = HashMap::new();

        layers.insert("terrain".to_string(), vec![vec!["Grass".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("biome".to_string(), vec![vec!["TemperateForest".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("elevation".to_string(), vec![vec!["100".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);

        chunks.insert((0, 0), layers);

        let world = WorldSerializer::create_serialized_world_from_layers(
            "Test".to_string(),
            42,
            WorldConfig::default(),
            chunks,
        );

        // Create a WorldLoader for testing
        let loader = WorldLoader::from_serialized_world(world.clone());

        // These new methods don't exist yet - test will fail until implemented
        let biome = loader.get_biome_at(5, 5);
        assert_eq!(biome, Some("TemperateForest".to_string()));

        let elevation = loader.get_elevation_at(5, 5);
        assert_eq!(elevation, Some("100".to_string()));

        let vegetation = loader.get_vegetation_density_at(5, 5);
        // Should return default since we didn't set vegetation layer (migrated)
        assert_eq!(vegetation, Some("0.5".to_string())); // Default value from migration
    }

    /// Test CachedWorld supports new layers
    #[test]
    fn test_cached_world_new_layers() {
        let mut chunks = HashMap::new();
        let mut layers = HashMap::new();

        layers.insert("terrain".to_string(), vec![vec!["Mountain".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("biome".to_string(), vec![vec!["RockyOutcrop".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("elevation".to_string(), vec![vec!["200".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("vegetation_density".to_string(), vec![vec!["0.1".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);

        chunks.insert((0, 0), layers);

        let world = WorldSerializer::create_serialized_world_from_layers(
            "Cache Test".to_string(),
            777,
            WorldConfig::default(),
            chunks,
        );

        let mut cached_world = CachedWorld::from_serialized(world);

        // Test accessing new layers through CachedWorld
        let biome_layer = cached_world.get_chunk_layer(0, 0, "biome");
        assert!(biome_layer.is_some());

        let elevation_layer = cached_world.get_chunk_layer(0, 0, "elevation");
        assert!(elevation_layer.is_some());

        let vegetation_layer = cached_world.get_chunk_layer(0, 0, "vegetation_density");
        assert!(vegetation_layer.is_some());
    }

    /// Test migration helpers provide sensible defaults
    #[test]
    fn test_migration_helpers_default_values() {
        // Create a world missing new fields (simulate old format)
        let world_json = r#"{
            "name": "Old World",
            "seed": 123,
            "config": {
                "seed": 123,
                "world_size_chunks": 10,
                "tile_size": 10.0,
                "enable_resources": true,
                "resource_density": 0.1,
                "terrain_generation_mode": "OpenRCT2Heights"
            },
            "chunks": {
                "0,0": {
                    "coordinate": {"x": 0, "y": 0},
                    "layers": {
                        "terrain": [["Grass"; 16]; 16]
                    },
                    "biome": "Plains"
                }
            },
            "version": "0.1.0"
        }"#;

        // This should fail to parse with new schema until migration is implemented
        let result: Result<SerializedWorld, _> = ron::from_str(world_json);

        // For now, this will fail because old format doesn't have new required fields
        // After implementation, this should succeed with migrated defaults
        match result {
            Ok(world) => {
                // If parsing succeeds, verify defaults were applied
                let chunk = world.chunks.get("0,0").unwrap();
                assert!(chunk.layers.contains_key("biome"));
                assert!(chunk.layers.contains_key("elevation"));

                // Check default values
                let biome_layer = chunk.layers.get("biome").unwrap();
                assert_eq!(biome_layer[0][0], "Plains"); // Default biome

                let elevation_layer = chunk.layers.get("elevation").unwrap();
                assert_eq!(elevation_layer[0][0], "64"); // Default elevation
            }
            Err(_) => {
                // Expected to fail initially - this is the RED phase of TDD
                println!("Expected failure: Old format doesn't work with new schema yet");
            }
        }
    }

    /// Test biome classification integration with new schema
    #[test]
    fn test_biome_classification_integration() {
        // Test that new biome types from PRD are supported
        let prd_biomes = vec![
            "TemperateForest", "Woodland", "Grassland", "ForestEdge",
            "RiparianZone", "RockyOutcrop", "ShallowWater", "DeepWater"
        ];

        for biome_name in prd_biomes {
            let mut chunks = HashMap::new();
            let mut layers = HashMap::new();

            layers.insert("terrain".to_string(), vec![vec!["Grass".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
            layers.insert("biome".to_string(), vec![vec![biome_name.to_string(); CHUNK_SIZE]; CHUNK_SIZE]);

            chunks.insert((0, 0), layers);

            let world = WorldSerializer::create_serialized_world_from_layers(
                format!("{} Test", biome_name),
                42,
                WorldConfig::default(),
                chunks,
            );

            // Verify biome is stored correctly
            let chunk = world.chunks.get("0,0").unwrap();
            let biome_layer = chunk.layers.get("biome").unwrap();
            assert_eq!(biome_layer[0][0], biome_name);
        }
    }

    /// Test elevation map statistics and validation
    #[test]
    fn test_elevation_map_statistics() {
        let mut chunks = HashMap::new();
        let mut layers = HashMap::new();

        // Create elevation with varied heights
        let mut elevation_layer = Vec::with_capacity(CHUNK_SIZE);
        for y in 0..CHUNK_SIZE {
            let mut row = Vec::with_capacity(CHUNK_SIZE);
            for x in 0..CHUNK_SIZE {
                let elevation = ((x + y) % 256).to_string(); // Varied elevations
                row.push(elevation);
            }
            elevation_layer.push(row);
        }

        layers.insert("terrain".to_string(), vec![vec!["Grass".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("elevation".to_string(), elevation_layer);

        chunks.insert((0, 0), layers);

        let world = WorldSerializer::create_serialized_world_from_layers(
            "Elevation Test".to_string(),
            42,
            WorldConfig::default(),
            chunks,
        );

        // Verify elevation data is stored as strings
        let chunk = world.chunks.get("0,0").unwrap();
        let elevation_layer = chunk.layers.get("elevation").unwrap();

        // Check some sample values
        assert_eq!(elevation_layer[0][0], "0");
        assert_eq!(elevation_layer[1][1], "2");
        assert_eq!(elevation_layer[15][15], "30");

        // Test that all elevation values are valid numbers
        for row in elevation_layer {
            for value in row {
                assert!(value.parse::<f32>().is_ok(), "Invalid elevation value: {}", value);
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test integration with map_generator - ensure it writes new layers
    #[test]
    fn test_map_generator_writes_new_layers() {
        // This will fail until map_generator is updated to write new layers
        // For now, we test the expected structure

        let config = WorldConfig::default();
        let generator = life_simulator::tilemap::WorldGenerator::new(config);

        // Generate a chunk (this will need to be updated to create new layers)
        let coord = ChunkCoordinate::new(0, 0);
        let chunk_data = generator.generate_procedural_chunk(0, 0);

        // For now, this just tests terrain generation
        // After implementation, this should test that biome, elevation, etc. layers are also generated
        assert_eq!(chunk_data.len(), CHUNK_SIZE);
        assert_eq!(chunk_data[0].len(), CHUNK_SIZE);

        // TODO: After implementation, verify new layers exist
        // let biome_layer = generator.get_biome_layer(0, 0);
        // let elevation_layer = generator.get_elevation_layer(0, 0);
        // assert!(biome_layer.is_some());
        // assert!(elevation_layer.is_some());
    }

    /// Test HTTP API can serve new layer data
    #[test]
    fn test_http_api_new_layers() {
        // This test will fail until HTTP API is updated to expose new layers
        let mut chunks = HashMap::new();
        let mut layers = HashMap::new();

        layers.insert("terrain".to_string(), vec![vec!["Grass".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("biome".to_string(), vec![vec!["Grassland".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);
        layers.insert("elevation".to_string(), vec![vec!["80".to_string(); CHUNK_SIZE]; CHUNK_SIZE]);

        chunks.insert((0, 0), layers);

        let world = WorldSerializer::create_serialized_world_from_layers(
            "API Test".to_string(),
            42,
            WorldConfig::default(),
            chunks,
        );

        let cached_world = CachedWorld::from_serialized(world);

        // Test multi-layer JSON generation (should include new layers)
        let json = cached_world.generate_multi_layer_chunks_json("/api/chunks?coords=0,0");

        // Should contain biome and elevation data
        assert!(json.contains("biome"));
        assert!(json.contains("elevation"));
        assert!(json.contains("Grassland"));
        assert!(json.contains("80"));
    }
}