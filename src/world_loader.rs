use bevy::log::info;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::WorldConfig;
use crate::serialization::{SerializedWorld, WorldSerializer};

/// Iterator over chunks in a world
pub struct ChunkIterator<'a> {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub chunk_data: &'a HashMap<String, Vec<Vec<String>>>,
}

/// Parse a chunk key "x,y" into coordinates
fn parse_chunk_key(key: &str) -> (i32, i32) {
    let parts: Vec<&str> = key.split(',').collect();
    if parts.len() != 2 {
        return (0, 0);
    }
    
    let x = parts[0].parse().unwrap_or(0);
    let y = parts[1].parse().unwrap_or(0);
    (x, y)
}

/// World loader for Life Simulator
///
/// Replaces WorldGenerator to only load pre-generated worlds.
/// The engine should always start with a loaded world instead of generating on-the-fly.
#[derive(Debug, Clone, bevy::prelude::Resource)]
pub struct WorldLoader {
    world: SerializedWorld,
    config: WorldConfig,
}

impl WorldLoader {
    /// Create a WorldLoader from an existing SerializedWorld (for testing)
    pub fn from_serialized_world(world: SerializedWorld) -> Self {
        let config = WorldConfig {
            seed: world.seed,
            ..WorldConfig::default()
        };

        Self { world, config }
    }

    /// Load a world from a file
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🗺️ Loading world from: {}", file_path);

        let world = WorldSerializer::load_world(file_path)?;
        let config = WorldConfig {
            seed: world.seed,
            ..WorldConfig::default()
        };

        info!("✅ World loaded: {} (seed: {})", world.name, world.seed);

        Ok(Self { world, config })
    }

    /// Load the default world (most recent in maps/ directory)
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        let maps_dir = Path::new("maps");
        if !maps_dir.exists() {
            return Err("No maps directory found. Please generate a world first using the map_generator tool.".into());
        }

        // Find the most recent RON file in maps directory
        let most_recent_file = find_most_recent_map_file(maps_dir)?;
        WorldLoader::load_from_file(&most_recent_file)
    }

    /// Load world by name
    pub fn load_by_name(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = format!("maps/{}.ron", name);
        WorldLoader::load_from_file(&file_path)
    }

    /// Get the world configuration
    pub fn get_config(&self) -> &WorldConfig {
        &self.config
    }

    /// Get the world seed
    pub fn get_seed(&self) -> u64 {
        self.world.seed
    }

    /// Get the world name
    pub fn get_name(&self) -> &str {
        &self.world.name
    }

    /// Find a spawn point in the loaded world
    pub fn find_spawn_point(&self) -> Option<(i32, i32)> {
        // Look for a suitable spawn point near center
        // For now, use center (0, 0) as default spawn point
        Some((0, 0))
    }

    /// Get terrain data for a specific chunk
    pub fn get_chunk_data(&self, chunk_x: i32, chunk_y: i32) -> Option<Vec<Vec<String>>> {
        let chunk_key = format!("{},{}", chunk_x, chunk_y);

        if let Some(chunk) = self.world.chunks.get(&chunk_key) {
            chunk.get_terrain().cloned()
        } else {
            // Return empty terrain chunk if not found
            Some(vec![vec!["Grass".to_string(); 16]; 16])
        }
    }

    /// Get all terrain layers for a chunk
    pub fn get_chunk_layers(
        &self,
        chunk_x: i32,
        chunk_y: i32,
    ) -> Option<HashMap<String, Vec<Vec<String>>>> {
        let chunk_key = format!("{},{}", chunk_x, chunk_y);

        if let Some(chunk) = self.world.chunks.get(&chunk_key) {
            Some(chunk.layers.clone())
        } else {
            // Return empty layers if chunk not found
            let mut layers = HashMap::new();
            layers.insert(
                "terrain".to_string(),
                vec![vec!["Grass".to_string(); 16]; 16],
            );
            Some(layers)
        }
    }

    /// Get a specific layer for a chunk
    pub fn get_chunk_layer(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        layer_name: &str,
    ) -> Option<Vec<Vec<String>>> {
        self.get_chunk_layers(chunk_x, chunk_y)
            .and_then(|layers| layers.get(layer_name).cloned())
    }

    /// Iterate over all chunks in the world
    pub fn iter_chunks(&self) -> impl Iterator<Item = ChunkIterator> {
        self.world.chunks.iter().map(|(key, chunk_data)| {
            let (chunk_x, chunk_y) = parse_chunk_key(key);
            ChunkIterator {
                chunk_x,
                chunk_y,
                chunk_data: &chunk_data.layers,
            }
        })
    }

    /// Get the world's metadata
    pub fn get_world_info(&self) -> &SerializedWorld {
        &self.world
    }

    /// Check if a chunk exists in the loaded world
    pub fn has_chunk(&self, chunk_x: i32, chunk_y: i32) -> bool {
        let chunk_key = format!("{},{}", chunk_x, chunk_y);
        self.world.chunks.contains_key(&chunk_key)
    }

    /// Get all available chunk coordinates
    pub fn get_chunk_coordinates(&self) -> Vec<(i32, i32)> {
        self.world
            .chunks
            .keys()
            .filter_map(|key| {
                if let Some((x_str, y_str)) = key.split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                        return Some((x, y));
                    }
                }
                None
            })
            .collect()
    }

    /// Get the total number of chunks in the world
    pub fn get_chunk_count(&self) -> usize {
        self.world.chunks.len()
    }

    /// Get the world bounds (min/max chunk coordinates)
    pub fn get_world_bounds(&self) -> ((i32, i32), (i32, i32)) {
        let coords = self.get_chunk_coordinates();
        if coords.is_empty() {
            return ((0, 0), (0, 0));
        }

        let min_x = coords.iter().map(|(x, _)| *x).min().unwrap_or(0);
        let max_x = coords.iter().map(|(x, _)| *x).max().unwrap_or(0);
        let min_y = coords.iter().map(|(_, y)| *y).min().unwrap_or(0);
        let max_y = coords.iter().map(|(_, y)| *y).max().unwrap_or(0);

        ((min_x, min_y), (max_x, max_y))
    }

    /// Check if the world has resources layer data
    pub fn has_resources(&self) -> bool {
        self.world
            .chunks
            .values()
            .any(|chunk| chunk.layers.contains_key("resources"))
    }

    /// Check if the world has terrain layer data
    pub fn has_terrain(&self) -> bool {
        self.world
            .chunks
            .values()
            .any(|chunk| chunk.layers.contains_key("terrain"))
    }

    /// Get available layer names in the world
    pub fn get_available_layers(&self) -> Vec<String> {
        let mut layers = std::collections::HashSet::new();
        for chunk in self.world.chunks.values() {
            for layer_name in chunk.layers.keys() {
                layers.insert(layer_name.clone());
            }
        }
        layers.into_iter().collect()
    }

    /// Get terrain type at a specific world tile position
    pub fn get_terrain_at(&self, world_x: i32, world_y: i32) -> Option<String> {
        // Convert world coordinates to chunk coordinates
        let chunk_x = world_x.div_euclid(16);
        let chunk_y = world_y.div_euclid(16);

        // Get local tile coordinates within chunk
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;

        // Get terrain layer for this chunk
        self.get_chunk_layer(chunk_x, chunk_y, "terrain")
            .and_then(|terrain| {
                terrain
                    .get(local_y)
                    .and_then(|row| row.get(local_x))
                    .cloned()
            })
    }

    /// Get resource at a specific world tile position (empty string if no resource)
    pub fn get_resource_at(&self, world_x: i32, world_y: i32) -> Option<String> {
        // Convert world coordinates to chunk coordinates
        let chunk_x = world_x.div_euclid(16);
        let chunk_y = world_y.div_euclid(16);

        // Get local tile coordinates within chunk
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;

        // Get resources layer for this chunk
        self.get_chunk_layer(chunk_x, chunk_y, "resources")
            .and_then(|resources| {
                resources
                    .get(local_y)
                    .and_then(|row| row.get(local_x))
                    .cloned()
            })
    }

    /// Get biome at a specific world tile position
    pub fn get_biome_at(&self, world_x: i32, world_y: i32) -> Option<String> {
        // Convert world coordinates to chunk coordinates
        let chunk_x = world_x.div_euclid(16);
        let chunk_y = world_y.div_euclid(16);

        // Get local tile coordinates within chunk
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;

        // Get biome layer for this chunk
        self.get_chunk_layer(chunk_x, chunk_y, "biome")
            .and_then(|biome| {
                biome
                    .get(local_y)
                    .and_then(|row| row.get(local_x))
                    .cloned()
            })
    }

    /// Get elevation at a specific world tile position
    pub fn get_elevation_at(&self, world_x: i32, world_y: i32) -> Option<String> {
        // Convert world coordinates to chunk coordinates
        let chunk_x = world_x.div_euclid(16);
        let chunk_y = world_y.div_euclid(16);

        // Get local tile coordinates within chunk
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;

        // Get elevation layer for this chunk
        self.get_chunk_layer(chunk_x, chunk_y, "elevation")
            .and_then(|elevation| {
                elevation
                    .get(local_y)
                    .and_then(|row| row.get(local_x))
                    .cloned()
            })
    }

    /// Get vegetation density at a specific world tile position
    pub fn get_vegetation_density_at(&self, world_x: i32, world_y: i32) -> Option<String> {
        // Convert world coordinates to chunk coordinates
        let chunk_x = world_x.div_euclid(16);
        let chunk_y = world_y.div_euclid(16);

        // Get local tile coordinates within chunk
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;

        // Get vegetation_density layer for this chunk
        self.get_chunk_layer(chunk_x, chunk_y, "vegetation_density")
            .and_then(|vegetation| {
                vegetation
                    .get(local_y)
                    .and_then(|row| row.get(local_x))
                    .cloned()
            })
    }

    /// Get moisture at a specific world tile position
    pub fn get_moisture_at(&self, world_x: i32, world_y: i32) -> Option<String> {
        // Convert world coordinates to chunk coordinates
        let chunk_x = world_x.div_euclid(16);
        let chunk_y = world_y.div_euclid(16);

        // Get local tile coordinates within chunk
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;

        // Get moisture layer for this chunk
        self.get_chunk_layer(chunk_x, chunk_y, "moisture")
            .and_then(|moisture| {
                moisture
                    .get(local_y)
                    .and_then(|row| row.get(local_x))
                    .cloned()
            })
    }

    /// Get the world's biome configuration
    pub fn get_biome_config(&self) -> &crate::serialization::BiomeConfig {
        &self.world.biome_config
    }

    /// Get the world's elevation map
    pub fn get_elevation_map(&self) -> &std::collections::HashMap<String, f32> {
        &self.world.elevation_map
    }
}

/// Find the most recent map file in a directory
fn find_most_recent_map_file(dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut most_recent_file = None;
    let mut most_recent_time = std::time::SystemTime::UNIX_EPOCH;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ron") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified > most_recent_time {
                        most_recent_time = modified;
                        most_recent_file = Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    most_recent_file.ok_or_else(|| "No RON files found in maps directory".into())
}

/// Validate that a world file exists and is valid
pub fn validate_world_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(file_path).exists() {
        return Err(format!("World file not found: {}", file_path).into());
    }

    // Try to load the file to validate it
    let _ = WorldSerializer::load_world(file_path)?;

    info!("✅ World file is valid: {}", file_path);
    Ok(())
}

/// Get information about all available worlds
pub fn list_available_worlds() -> Result<Vec<WorldInfo>, Box<dyn std::error::Error>> {
    let maps_dir = Path::new("maps");
    if !maps_dir.exists() {
        return Ok(vec![]);
    }

    let mut worlds = Vec::new();

    for entry in fs::read_dir(maps_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ron") {
            let file_path = path.to_string_lossy();

            // Try to load the world to get metadata
            if let Ok(world) = WorldSerializer::load_world(&file_path) {
                worlds.push(WorldInfo {
                    name: world.name.clone(),
                    file_path: file_path.to_string(),
                    seed: world.seed,
                    chunk_count: world.chunks.len(),
                    version: world.version,
                    file_size: path.metadata().map(|m| m.len()).unwrap_or(0),
                });
            }
        }
    }

    // Sort by modification time (newest first)
    worlds.sort_by(|a, b| b.file_path.cmp(&a.file_path));

    Ok(worlds)
}

#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub file_path: String,
    pub seed: u64,
    pub chunk_count: usize,
    pub version: String,
    pub file_size: u64,
}
