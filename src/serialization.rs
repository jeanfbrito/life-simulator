use std::collections::HashMap;
use std::fs;
use std::path::Path;

use bevy::prelude::*;
use ron::error::SpannedError;
use serde::{Deserialize, Serialize};

use crate::cached_world::CachedWorld;
use crate::tilemap::{ChunkCoordinate, WorldConfig, CHUNK_SIZE};

/// Serializable world data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedWorld {
    pub name: String,
    pub seed: u64,
    pub config: WorldConfig,
    pub chunks: HashMap<String, SerializedChunk>,
    pub version: String,
}

/// Serializable chunk data structure with multi-layer support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedChunk {
    pub coordinate: ChunkCoordinate,
    pub layers: HashMap<String, Vec<Vec<String>>>,
    pub biome: String,
}

impl SerializedChunk {
    /// Create a new chunk with terrain layer
    pub fn new(coordinate: ChunkCoordinate, terrain: Vec<Vec<String>>) -> Self {
        let mut layers = HashMap::new();
        layers.insert("terrain".to_string(), terrain);

        Self {
            coordinate,
            layers,
            biome: "Plains".to_string(),
        }
    }

    /// Get terrain layer data
    pub fn get_terrain(&self) -> Option<&Vec<Vec<String>>> {
        self.layers.get("terrain")
    }

    /// Get resources layer data
    pub fn get_resources(&self) -> Option<&Vec<Vec<String>>> {
        self.layers.get("resources")
    }

    /// Set terrain layer data
    pub fn set_terrain(&mut self, terrain: Vec<Vec<String>>) {
        self.layers.insert("terrain".to_string(), terrain);
    }

    /// Set resources layer data
    pub fn set_resources(&mut self, resources: Vec<Vec<String>>) {
        self.layers.insert("resources".to_string(), resources);
    }

    /// Get any layer by name
    pub fn get_layer(&self, layer_name: &str) -> Option<&Vec<Vec<String>>> {
        self.layers.get(layer_name)
    }

    /// Set any layer by name
    pub fn set_layer(&mut self, layer_name: String, data: Vec<Vec<String>>) {
        self.layers.insert(layer_name, data);
    }
}

/// Simple world serialization system
pub struct WorldSerializer;

impl WorldSerializer {
    /// Save world data to RON file
    pub fn save_world(
        world: &SerializedWorld,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let ron_string = ron::to_string(world)?;
        fs::write(path, ron_string)?;
        Ok(())
    }

    /// Load world data from RON file
    pub fn load_world(path: &str) -> Result<SerializedWorld, Box<dyn std::error::Error>> {
        let ron_string = fs::read_to_string(path)?;
        let world: SerializedWorld = ron::from_str(&ron_string)?;
        Ok(world)
    }

    /// Create a serialized world from current state
    pub fn create_serialized_world(
        name: String,
        seed: u64,
        config: WorldConfig,
        chunks: HashMap<(i32, i32), Vec<Vec<String>>>,
    ) -> SerializedWorld {
        let serialized_chunks = chunks
            .into_iter()
            .map(|((x, y), terrain_tiles)| {
                let chunk_key = format!("{},{}", x, y);
                let serialized_chunk =
                    SerializedChunk::new(ChunkCoordinate::new(x, y), terrain_tiles);
                (chunk_key, serialized_chunk)
            })
            .collect();

        SerializedWorld {
            name,
            seed,
            config,
            chunks: serialized_chunks,
            version: "0.2.0".to_string(), // Updated version for multi-layer support
        }
    }

    /// Convert serialized chunks back to HashMap format (terrain layer only for compatibility)
    pub fn chunks_to_hashmap(
        chunks: HashMap<String, SerializedChunk>,
    ) -> HashMap<(i32, i32), Vec<Vec<String>>> {
        chunks
            .into_iter()
            .filter_map(|(key, chunk)| {
                if let Some((x_str, y_str)) = key.split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                        // Extract terrain layer for backward compatibility
                        return Some((
                            (x, y),
                            chunk.get_terrain().cloned().unwrap_or_else(|| {
                                // Create empty terrain layer if none exists
                                vec![vec!["Grass".to_string(); 16]; 16]
                            }),
                        ));
                    }
                }
                None
            })
            .collect()
    }

    /// Convert serialized chunks to multi-layer HashMap format
    pub fn chunks_to_multi_layer_hashmap(
        chunks: HashMap<String, SerializedChunk>,
    ) -> HashMap<(i32, i32), HashMap<String, Vec<Vec<String>>>> {
        chunks
            .into_iter()
            .filter_map(|(key, chunk)| {
                if let Some((x_str, y_str)) = key.split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                        return Some(((x, y), chunk.layers));
                    }
                }
                None
            })
            .collect()
    }

    /// Create a serialized world from multi-layer chunks
    pub fn create_serialized_world_from_layers(
        name: String,
        seed: u64,
        config: WorldConfig,
        multi_layer_chunks: HashMap<(i32, i32), HashMap<String, Vec<Vec<String>>>>,
    ) -> SerializedWorld {
        let serialized_chunks = multi_layer_chunks
            .into_iter()
            .map(|((x, y), layers)| {
                let chunk_key = format!("{},{}", x, y);
                let coordinate = ChunkCoordinate::new(x, y);

                // Extract terrain layer to create the SerializedChunk
                let serialized_chunk = if let Some(terrain_layer) = layers.get("terrain") {
                    SerializedChunk::new(coordinate, terrain_layer.clone())
                } else {
                    // Fallback to empty terrain layer
                    SerializedChunk::new(coordinate, vec![vec!["Grass".to_string(); 16]; 16])
                };

                // Add all layers to the chunk
                let mut final_chunk = serialized_chunk;
                for (layer_name, layer_data) in layers {
                    if layer_name != "terrain" {
                        final_chunk.set_layer(layer_name, layer_data);
                    }
                }

                (chunk_key, final_chunk)
            })
            .collect();

        SerializedWorld {
            name,
            seed,
            config,
            chunks: serialized_chunks,
            version: "0.2.0".to_string(),
        }
    }
}

/// Component to mark worlds that should be saved
#[derive(Component)]
pub struct WorldSaveRequest {
    pub file_path: String,
    pub name: String,
}

/// Component to mark worlds that should be loaded
#[derive(Component)]
pub struct WorldLoadRequest {
    pub file_path: String,
}

/// System to handle world save requests
pub fn handle_world_save_requests(
    mut commands: Commands,
    save_requests: Query<(Entity, &WorldSaveRequest, &WorldConfig)>,
    world_generator: Res<crate::tilemap::WorldGenerator>,
) {
    for (entity, request, config) in save_requests.iter() {
        info!("Saving world '{}' to {}", request.name, request.file_path);

        // Generate multi-layer chunks around center for saving
        let mut multi_layer_chunks = HashMap::new();
        let center_x = 0;
        let center_y = 0;
        let radius = 3; // Save 7x7 chunk area around center

        for chunk_x in (center_x - radius)..=(center_x + radius) {
            for chunk_y in (center_y - radius)..=(center_y + radius) {
                // Generate terrain layer
                let terrain_tiles = world_generator.generate_procedural_chunk(chunk_x, chunk_y);

                // Generate resources layer using the existing resource generation system
                let resources_tiles =
                    crate::resources::ResourceGenerator::create_resources_for_chunk(
                        &terrain_tiles,
                        chunk_x,
                        chunk_y,
                        world_generator.get_seed(),
                    );

                // Create multi-layer chunk with both terrain and resources
                let mut chunk_layers = HashMap::new();
                chunk_layers.insert("terrain".to_string(), terrain_tiles);
                chunk_layers.insert("resources".to_string(), resources_tiles);

                multi_layer_chunks.insert((chunk_x, chunk_y), chunk_layers);
            }
        }

        let serialized_world = WorldSerializer::create_serialized_world_from_layers(
            request.name.clone(),
            world_generator.get_seed(),
            config.clone(),
            multi_layer_chunks,
        );

        match WorldSerializer::save_world(&serialized_world, &request.file_path) {
            Ok(()) => {
                info!(
                    "World '{}' saved successfully to {}",
                    request.name, request.file_path
                );
                commands.entity(entity).remove::<WorldSaveRequest>();
            }
            Err(e) => {
                error!("Failed to save world: {}", e);
            }
        }
    }
}

/// System to handle world load requests
pub fn handle_world_load_requests(
    mut commands: Commands,
    load_requests: Query<(Entity, &WorldLoadRequest)>,
    mut world_generator: ResMut<crate::tilemap::WorldGenerator>,
) {
    for (entity, request) in load_requests.iter() {
        info!("Loading world from {}", request.file_path);

        match WorldSerializer::load_world(&request.file_path) {
            Ok(serialized_world) => {
                info!(
                    "World '{}' loaded successfully (seed: {})",
                    serialized_world.name, serialized_world.seed
                );

                // Update the world generator with the loaded seed
                world_generator.set_seed(serialized_world.seed);

                // Create and populate the cached world with loaded data
                let cached_world = CachedWorld::from_serialized(serialized_world.clone());
                CachedWorld::global_set(cached_world);

                // Log chunk loading details
                let loaded_chunks_count = serialized_world.chunks.len();
                info!("Loaded {} chunks into cached world", loaded_chunks_count);

                // Log available layers in loaded world
                if let Some((_, first_chunk)) = serialized_world.chunks.iter().next() {
                    let layer_names: Vec<&String> = first_chunk.layers.keys().collect();
                    info!("Available layers: {:?}", layer_names);
                }

                commands.entity(entity).remove::<WorldLoadRequest>();
            }
            Err(e) => {
                error!("Failed to load world: {}", e);
            }
        }
    }
}

/// Plugin to add world serialization systems
pub struct WorldSerializationPlugin;

impl Plugin for WorldSerializationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_world_save_requests)
            .add_systems(Update, handle_world_load_requests);
    }
}
