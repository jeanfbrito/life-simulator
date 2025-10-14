use super::{BiomeType, Chunk, ChunkCoordinate, TerrainType, CHUNK_SIZE};
use bevy::log::debug;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Simplex};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, Component, Resource)]
pub struct WorldConfig {
    pub seed: u64,
    pub world_size_chunks: i32,
    pub tile_size: f32,
    pub enable_resources: bool,
    pub resource_density: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            world_size_chunks: 100,
            tile_size: 10.0,
            enable_resources: true,
            resource_density: 0.1,
        }
    }
}

#[derive(Resource)]
pub struct WorldGenerator {
    config: WorldConfig,
    rng: RwLock<Pcg64>,
}

impl WorldGenerator {
    pub fn new(config: WorldConfig) -> Self {
        let rng = Pcg64::seed_from_u64(config.seed);
        Self {
            config,
            rng: RwLock::new(rng),
        }
    }

    pub fn get_seed(&self) -> u64 {
        self.config.seed
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.config.seed = seed;
        *self.rng.write().unwrap() = Pcg64::seed_from_u64(seed);
    }

    pub fn generate_chunk(&self, coordinate: ChunkCoordinate) -> Chunk {
        let mut chunk = Chunk::new(coordinate, self.config.seed);

        if self.config.enable_resources {
            self.add_resources_to_chunk(&mut chunk);
        }

        chunk
    }

    fn add_resources_to_chunk(&self, chunk: &mut Chunk) {
        for y in 0..chunk.tiles.len() {
            for x in 0..chunk.tiles[y].len() {
                if self.rng.write().unwrap().gen::<f32>() < self.config.resource_density {
                    // Add resource deposits based on terrain type
                    let terrain = chunk.tiles[y][x];
                    if self.can_spawn_resource_on_terrain(terrain) {
                        // In a full implementation, this would add resource entities
                        // For now, we just mark the terrain as having resource potential
                        debug!(
                            "Resource potential at chunk ({}, {}) local ({}, {}) on {:?}",
                            chunk.coordinate.x, chunk.coordinate.y, x, y, terrain
                        );
                    }
                }
            }
        }
    }

    fn can_spawn_resource_on_terrain(&self, terrain: TerrainType) -> bool {
        matches!(
            terrain,
            TerrainType::Forest | TerrainType::Mountain | TerrainType::Stone | TerrainType::Grass
        )
    }

    pub fn get_world_bounds(&self) -> (i32, i32, i32, i32) {
        let half_size = self.config.world_size_chunks / 2;
        let min_x = -half_size;
        let max_x = half_size;
        let min_y = -half_size;
        let max_y = half_size;
        (min_x, max_x, min_y, max_y)
    }

    pub fn is_within_world_bounds(&self, coord: &ChunkCoordinate) -> bool {
        let (min_x, max_x, min_y, max_y) = self.get_world_bounds();
        coord.x >= min_x && coord.x <= max_x && coord.y >= min_y && coord.y <= max_y
    }

    pub fn generate_world_statistics(&self) -> WorldStatistics {
        let mut stats = WorldStatistics::default();
        let (min_x, max_x, min_y, max_y) = self.get_world_bounds();

        for chunk_y in min_y..=max_y {
            for chunk_x in min_x..=max_x {
                let coord = ChunkCoordinate::new(chunk_x, chunk_y);
                let chunk = self.generate_chunk(coord);

                stats.total_chunks += 1;
                stats.total_tiles += CHUNK_SIZE * CHUNK_SIZE;

                // Count terrain types
                for row in chunk.tiles {
                    for terrain in row {
                        *stats.terrain_distribution.entry(terrain).or_insert(0) += 1;

                        if terrain.is_walkable() {
                            stats.walkable_tiles += 1;
                        }
                    }
                }

                // Count biomes
                *stats.biome_distribution.entry(chunk.biome).or_insert(0) += 1;
            }
        }

        stats
    }

    pub fn find_spawn_point(&self) -> Option<(i32, i32)> {
        let (min_x, max_x, min_y, max_y) = self.get_world_bounds();
        let mut attempts = 0;
        let max_attempts = 100;

        while attempts < max_attempts {
            let chunk_x = self.rng.write().unwrap().gen_range(min_x..=max_x);
            let chunk_y = self.rng.write().unwrap().gen_range(min_y..=max_y);
            let coord = ChunkCoordinate::new(chunk_x, chunk_y);

            let chunk = self.generate_chunk(coord);

            // Check if chunk has good spawn conditions (mostly walkable terrain)
            if chunk.is_walkable_percentage() > 0.7 {
                // Find a walkable tile within the chunk
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        if chunk.tiles[y][x].is_walkable() {
                            let world_x = coord.x * CHUNK_SIZE as i32 + x as i32;
                            let world_y = coord.y * CHUNK_SIZE as i32 + y as i32;
                            return Some((world_x, world_y));
                        }
                    }
                }
            }

            attempts += 1;
        }

        None
    }

    // Circular island terrain generation methods for web API
    pub fn generate_chunks_json(&self, path: &str) -> String {
        // Parse coordinates from path like /api/chunks?coords=0,0&coords=1,0
        let coords = self.parse_chunk_coords(path);
        let mut chunk_data = HashMap::new();

        for &(chunk_x, chunk_y) in &coords {
            let chunk_key = format!("{},{}", chunk_x, chunk_y);
            let terrain_data = self.generate_procedural_chunk(chunk_x, chunk_y);
            chunk_data.insert(chunk_key, terrain_data);
        }

        // Convert to JSON string
        let mut json_parts = Vec::new();
        for (key, data) in chunk_data {
            let data_str = data
                .iter()
                .map(|row| {
                    format!(
                        "[{}]",
                        row.iter()
                            .map(|tile| format!("\"{}\"", tile))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            json_parts.push(format!("\"{}\": [{}]", key, data_str));
        }

        format!("{{\"chunk_data\": {{{}}}}}", json_parts.join(", "))
    }

    fn parse_chunk_coords(&self, path: &str) -> Vec<(i32, i32)> {
        // Extract coordinates from path like /api/chunks?coords=0,0&coords=1,0
        if let Some(query_part) = path.split('?').nth(1) {
            let mut coords = Vec::new();
            for param in query_part.split('&') {
                if let Some(coord_part) = param.strip_prefix("coords=") {
                    if let Some((x_str, y_str)) = coord_part.split_once(',') {
                        if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                            coords.push((x, y));
                        }
                    }
                }
            }
            return coords;
        }
        // Default to center chunk (0, 0)
        vec![(0, 0)]
    }

    pub fn generate_procedural_chunk(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<String>> {
        let mut chunk = Vec::with_capacity(16);
        let seed = (chunk_x as u64)
            .wrapping_mul(1000)
            .wrapping_add(chunk_y as u64)
            .wrapping_add(self.config.seed);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        for y in 0..16 {
            let mut row = Vec::with_capacity(16);
            for x in 0..16 {
                let world_x = chunk_x * 16 + x;
                let world_y = chunk_y * 16 + y;

                // Generate terrain based on circular island pattern
                let terrain_type = self.generate_terrain_type(world_x, world_y, &mut rng);
                row.push(terrain_type);
            }
            chunk.push(row);
        }

        chunk
    }

    /// Generate height map for a chunk using Perlin noise
    ///
    /// Heights follow the circular island pattern:
    /// - Center (island): Higher elevation (80-120)
    /// - Beach zone: Mid elevation (45-55)
    /// - Water zones: Sea level (30-40)
    /// - Deep water: Lower (10-20)
    ///
    /// Perlin noise adds natural variation to create hills/valleys
    pub fn generate_height_chunk(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<u8>> {
        let mut heights = Vec::with_capacity(16);
        let perlin = Perlin::new(self.config.seed as u32);

        // Height map parameters
        let base_height = 50; // Sea level
        let noise_scale = 0.05; // Frequency of height variation
        let noise_amplitude = 30.0; // Height variation range

        for y in 0..16 {
            let mut row = Vec::with_capacity(16);
            for x in 0..16 {
                let world_x = chunk_x * 16 + x;
                let world_y = chunk_y * 16 + y;

                // Get base height from island pattern
                let base = self.calculate_base_height(world_x, world_y);

                // Add Perlin noise for natural variation
                let noise_x = (world_x as f64) * noise_scale;
                let noise_y = (world_y as f64) * noise_scale;
                let noise_value = perlin.get([noise_x, noise_y]);

                // Combine base height with noise
                let height = base as f64 + (noise_value * noise_amplitude);

                // Clamp to u8 range (0-255)
                let final_height = height.max(0.0).min(255.0) as u8;

                row.push(final_height);
            }
            heights.push(row);
        }

        heights
    }

    /// Calculate base height for a tile based on circular island pattern
    ///
    /// Matches terrain generation: center is elevated, edges are water
    fn calculate_base_height(&self, world_x: i32, world_y: i32) -> u8 {
        let distance_from_center = ((world_x * world_x + world_y * world_y) as f32).sqrt();

        // Match terrain generation parameters
        let island_radius = 35.0;
        let beach_width = 4.0;
        let shallow_water_width = 6.0;

        // Add same variations as terrain
        let angle = (world_y as f32).atan2(world_x as f32);
        let island_variation = (angle * 2.0).sin() * 1.5 + (angle * 3.0).cos() * 1.0;
        let effective_island_radius = island_radius + island_variation;

        let beach_variation = (angle * 4.0).sin() * 0.8;
        let effective_beach_width = beach_width + beach_variation;

        // Height zones matching terrain types
        if distance_from_center
            > effective_island_radius + effective_beach_width + shallow_water_width
        {
            // Deep water - lowest elevation
            15
        } else if distance_from_center > effective_island_radius + effective_beach_width {
            // Shallow water - below sea level
            35
        } else if distance_from_center > effective_island_radius {
            // Beach - at sea level
            50
        } else {
            // Island interior - elevated terrain
            let center_distance = distance_from_center;
            let inner_radius = effective_island_radius * 0.7;

            if center_distance < inner_radius {
                // Inner island - highest elevation
                100
            } else {
                // Transition zone - gradient from center to beach
                let transition_factor = (distance_from_center - inner_radius)
                    / (effective_island_radius - inner_radius);
                let height = 100.0 - (transition_factor * 50.0); // 100 → 50
                height as u8
            }
        }
    }

    fn generate_terrain_type(
        &self,
        world_x: i32,
        world_y: i32,
        rng: &mut rand::rngs::StdRng,
    ) -> String {
        // Circular island generation with clean beach edges
        let distance_from_center = ((world_x * world_x + world_y * world_y) as f32).sqrt();

        // Main island parameters
        let island_radius = 35.0; // Main island radius
        let beach_width = 4.0; // Beach width around island
        let shallow_water_width = 6.0; // Shallow water zone

        // Create circular island with reduced irregularity for more consistency
        let angle = (world_y as f32).atan2(world_x as f32);
        let island_variation = (angle * 2.0).sin() * 1.5 + (angle * 3.0).cos() * 1.0;
        let effective_island_radius = island_radius + island_variation;

        // Beach variation - reduced for consistency
        let beach_variation = (angle * 4.0).sin() * 0.8;
        let effective_beach_width = beach_width + beach_variation;

        // Add small random variation for texture
        let texture_noise = (world_x as f32 * 0.1).sin() * (world_y as f32 * 0.1).cos() * 0.5 + 0.5;

        if distance_from_center
            > effective_island_radius + effective_beach_width + shallow_water_width
        {
            // Deep water - outer ocean
            "DeepWater".to_string()
        } else if distance_from_center > effective_island_radius + effective_beach_width {
            // Shallow water - between beach and deep water
            "ShallowWater".to_string()
        } else if distance_from_center > effective_island_radius {
            // Beach sand - ring around island
            if texture_noise < 0.1 && distance_from_center - effective_island_radius > 1.0 {
                // Some shallow water patches in beach for variety
                "ShallowWater".to_string()
            } else {
                "Sand".to_string()
            }
        } else {
            // Island interior - mostly grass with some variation
            let center_distance = distance_from_center;
            let inner_radius = effective_island_radius * 0.7; // Inner grass circle

            if center_distance < inner_radius {
                // Inner area - guaranteed grass
                if texture_noise < 0.05 {
                    // Small dirt patches for variety
                    "Dirt".to_string()
                } else if texture_noise > 0.95 {
                    // occasional forest patches
                    "Forest".to_string()
                } else {
                    "Grass".to_string()
                }
            } else {
                // Outer island area - transition zone
                let transition_factor = (distance_from_center - inner_radius)
                    / (effective_island_radius - inner_radius);

                if rng.gen::<f32>() < transition_factor * 0.3 {
                    // Some sand patches near beach
                    "Sand".to_string()
                } else if texture_noise > 0.8 && transition_factor < 0.5 {
                    // Forest patches in middle areas
                    "Forest".to_string()
                } else if texture_noise < 0.1 && transition_factor > 0.3 {
                    // Dirt patches near edges
                    "Dirt".to_string()
                } else {
                    // Mostly grass
                    "Grass".to_string()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStatistics {
    pub total_chunks: i32,
    pub total_tiles: usize,
    pub walkable_tiles: usize,
    pub terrain_distribution: std::collections::HashMap<TerrainType, usize>,
    pub biome_distribution: std::collections::HashMap<BiomeType, i32>,
}

impl Default for WorldStatistics {
    fn default() -> Self {
        Self {
            total_chunks: 0,
            total_tiles: 0,
            walkable_tiles: 0,
            terrain_distribution: std::collections::HashMap::new(),
            biome_distribution: std::collections::HashMap::new(),
        }
    }
}

impl WorldStatistics {
    pub fn get_walkable_percentage(&self) -> f32 {
        if self.total_tiles == 0 {
            0.0
        } else {
            self.walkable_tiles as f32 / self.total_tiles as f32
        }
    }

    pub fn get_most_common_terrain(&self) -> Option<TerrainType> {
        self.terrain_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(terrain, _)| *terrain)
    }

    pub fn get_most_common_biome(&self) -> Option<BiomeType> {
        self.biome_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(biome, _)| *biome)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub config: WorldConfig,
    pub statistics: WorldStatistics,
    pub generated_at: std::time::SystemTime,
}

impl WorldMetadata {
    pub fn new(config: WorldConfig, statistics: WorldStatistics) -> Self {
        Self {
            config,
            statistics,
            generated_at: std::time::SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_generator_creation() {
        let config = WorldConfig::default();
        let generator = WorldGenerator::new(config);

        // Test that generator was created successfully
        assert_eq!(generator.config.seed, 12345);
    }

    #[test]
    fn test_chunk_generation() {
        let config = WorldConfig::default();
        let mut generator = WorldGenerator::new(config);
        let coord = ChunkCoordinate::new(0, 0);

        let chunk = generator.generate_chunk(coord);

        assert_eq!(chunk.coordinate, coord);
        assert_eq!(chunk.tiles.len(), CHUNK_SIZE);
        assert_eq!(chunk.tiles[0].len(), CHUNK_SIZE);
    }

    #[test]
    fn test_world_bounds() {
        let config = WorldConfig {
            world_size_chunks: 10,
            ..Default::default()
        };
        let generator = WorldGenerator::new(config);

        let (min_x, max_x, min_y, max_y) = generator.get_world_bounds();

        assert_eq!(min_x, -5);
        assert_eq!(max_x, 5);
        assert_eq!(min_y, -5);
        assert_eq!(max_y, 5);
    }

    #[test]
    fn test_spawn_point_finding() {
        let config = WorldConfig {
            world_size_chunks: 4,
            ..Default::default()
        };
        let mut generator = WorldGenerator::new(config);

        let spawn_point = generator.find_spawn_point();

        assert!(spawn_point.is_some());
        let (x, y) = spawn_point.unwrap();

        // Check that spawn point is within world bounds
        let (min_x, max_x, min_y, max_y) = generator.get_world_bounds();
        let world_min_x = min_x * CHUNK_SIZE as i32;
        let world_max_x = (max_x + 1) * CHUNK_SIZE as i32;
        let world_min_y = min_y * CHUNK_SIZE as i32;
        let world_max_y = (max_y + 1) * CHUNK_SIZE as i32;

        assert!(x >= world_min_x && x < world_max_x);
        assert!(y >= world_min_y && y < world_max_y);
    }
}
