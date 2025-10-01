use std::collections::HashMap;
use std::fs;
use std::path::Path;

use bevy::prelude::*;
use ron::error::SpannedError;
use serde::{Deserialize, Serialize};

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

/// Serializable chunk data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedChunk {
    pub coordinate: ChunkCoordinate,
    pub tiles: Vec<Vec<String>>,
    pub biome: String,
}

/// Simple world serialization system
pub struct WorldSerializer;

impl WorldSerializer {
    /// Save world data to RON file
    pub fn save_world(world: &SerializedWorld, path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
            .map(|((x, y), tiles)| {
                let chunk_key = format!("{},{}", x, y);
                let serialized_chunk = SerializedChunk {
                    coordinate: ChunkCoordinate::new(x, y),
                    tiles,
                    biome: "Plains".to_string(), // Default biome for now
                };
                (chunk_key, serialized_chunk)
            })
            .collect();

        SerializedWorld {
            name,
            seed,
            config,
            chunks: serialized_chunks,
            version: "0.1.0".to_string(),
        }
    }

    /// Convert serialized chunks back to HashMap format
    pub fn chunks_to_hashmap(
        chunks: HashMap<String, SerializedChunk>,
    ) -> HashMap<(i32, i32), Vec<Vec<String>>> {
        chunks
            .into_iter()
            .filter_map(|(key, chunk)| {
                if let Some((x_str, y_str)) = key.split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                        return Some(((x, y), chunk.tiles));
                    }
                }
                None
            })
            .collect()
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

        // Generate chunks around center for saving
        let mut chunks = HashMap::new();
        let center_x = 0;
        let center_y = 0;
        let radius = 3; // Save 7x7 chunk area around center

        for chunk_x in (center_x - radius)..=(center_x + radius) {
            for chunk_y in (center_y - radius)..=(center_y + radius) {
                let coord = ChunkCoordinate::new(chunk_x, chunk_y);
                let chunk_tiles = world_generator.generate_procedural_chunk(chunk_x, chunk_y);
                chunks.insert((chunk_x, chunk_y), chunk_tiles);
            }
        }

        let serialized_world = WorldSerializer::create_serialized_world(
            request.name.clone(),
            world_generator.get_seed(),
            config.clone(),
            chunks,
        );

        match WorldSerializer::save_world(&serialized_world, &request.file_path) {
            Ok(()) => {
                info!("World saved successfully!");
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
) {
    for (entity, request) in load_requests.iter() {
        info!("Loading world from {}", request.file_path);

        match WorldSerializer::load_world(&request.file_path) {
            Ok(serialized_world) => {
                info!("World loaded successfully: {}", serialized_world.name);
                // Here you could spawn entities or update world state based on loaded data
                // For now, we'll just log the success and remove the component
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