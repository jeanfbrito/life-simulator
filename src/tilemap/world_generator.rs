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
    pub terrain_generation_mode: TerrainGenerationMode,
}

/// Terrain generation mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TerrainGenerationMode {
    /// Legacy circular island generation (deprecated)
    CircularIsland,
    /// OpenRCT2-style height-based generation (recommended)
    OpenRCT2Heights,
}

/// OpenRCT2-style terrain generation configuration
/// Based on height thresholds and noise-based variation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRCT2TerrainConfig {
    // Water levels
    pub deep_water_max: u8,      // Below this = DeepWater (default: 35)
    pub shallow_water_max: u8,   // Below this = ShallowWater (default: 48)
    pub beach_max: u8,           // Below this = Sand (beach) (default: 55)

    // Land elevations
    pub plains_max: u8,          // Below this = Grass/Dirt (default: 120)
    pub hills_max: u8,           // Below this = Stone (default: 160)
    pub mountain_min: u8,        // Above this = Mountain (default: 160)

    // Terrain variety parameters
    pub forest_frequency: f64,   // Perlin noise frequency for forests (default: 0.05)
    pub forest_threshold: f64,   // Noise threshold for forest placement (default: 0.3)
    pub desert_frequency: f64,   // Frequency for desert zones (default: 0.03)
    pub desert_threshold: f64,   // Threshold for desert placement (default: 0.5)
    pub snow_altitude: u8,       // Height above which snow appears (default: 180)
}

impl Default for OpenRCT2TerrainConfig {
    fn default() -> Self {
        Self {
            deep_water_max: 35,
            shallow_water_max: 48,
            beach_max: 55,
            plains_max: 120,
            hills_max: 160,
            mountain_min: 160,
            forest_frequency: 0.05,
            forest_threshold: 0.3,
            desert_frequency: 0.03,
            desert_threshold: 0.5,
            snow_altitude: 180,
        }
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            world_size_chunks: 100,
            tile_size: 10.0,
            enable_resources: true,
            resource_density: 0.1,
            terrain_generation_mode: TerrainGenerationMode::OpenRCT2Heights,
        }
    }
}

#[derive(Resource)]
pub struct WorldGenerator {
    config: WorldConfig,
    rng: RwLock<Pcg64>,
    openrct2_config: OpenRCT2TerrainConfig,
}

impl WorldGenerator {
    pub fn new(config: WorldConfig) -> Self {
        let rng = Pcg64::seed_from_u64(config.seed);
        Self {
            config,
            rng: RwLock::new(rng),
            openrct2_config: OpenRCT2TerrainConfig::default(),
        }
    }

    pub fn with_openrct2_config(mut self, openrct2_config: OpenRCT2TerrainConfig) -> Self {
        self.openrct2_config = openrct2_config;
        self
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
        // Dispatch to the appropriate generation method based on config
        match self.config.terrain_generation_mode {
            TerrainGenerationMode::CircularIsland => self.generate_island_chunk(chunk_x, chunk_y),
            TerrainGenerationMode::OpenRCT2Heights => self.generate_openrct2_chunk(chunk_x, chunk_y),
        }
    }

    /// Legacy circular island generation (kept for backward compatibility)
    fn generate_island_chunk(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<String>> {
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

    /// OpenRCT2-style terrain generation based on height and noise
    fn generate_openrct2_chunk(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<String>> {
        let mut chunk = Vec::with_capacity(16);

        // Generate heights first (using existing fBm method)
        let heights = self.generate_height_chunk(chunk_x, chunk_y);

        // Create noise generators for terrain variety
        let perlin = Perlin::new(self.config.seed as u32);

        for y in 0..16 {
            let mut row = Vec::with_capacity(16);
            for x in 0..16 {
                let world_x = chunk_x * 16 + x;
                let world_y = chunk_y * 16 + y;
                let height = heights[y as usize][x as usize];

                // Generate terrain type from height and noise
                let terrain_type = self.generate_terrain_from_height(
                    height,
                    world_x,
                    world_y,
                    &perlin,
                );
                row.push(terrain_type);
            }
            chunk.push(row);
        }

        chunk
    }

    /// Map height to terrain type using OpenRCT2-style thresholds
    /// Uses additional noise layers for natural terrain variety
    fn generate_terrain_from_height(
        &self,
        height: u8,
        world_x: i32,
        world_y: i32,
        perlin: &Perlin,
    ) -> String {
        let cfg = &self.openrct2_config;

        // Water levels (lowest priority - height-based only)
        if height <= cfg.deep_water_max {
            return "DeepWater".to_string();
        }
        if height <= cfg.shallow_water_max {
            return "ShallowWater".to_string();
        }
        if height <= cfg.beach_max {
            return "Sand".to_string();
        }

        // Snow at high altitudes (highest priority on land)
        if height >= cfg.snow_altitude {
            return "Snow".to_string();
        }

        // Mountains (high elevation)
        if height >= cfg.mountain_min {
            return "Mountain".to_string();
        }

        // Hills/Stone (medium-high elevation)
        if height >= cfg.hills_max {
            return "Stone".to_string();
        }

        // Plains (medium elevation) - use noise for variety
        // Sample terrain variety noise
        let forest_noise = perlin.get([
            world_x as f64 * cfg.forest_frequency,
            world_y as f64 * cfg.forest_frequency,
        ]);
        let desert_noise = perlin.get([
            world_x as f64 * cfg.desert_frequency,
            world_y as f64 * cfg.desert_frequency,
        ]);

        // Normalize noise to [0, 1]
        let forest_value = (forest_noise + 1.0) / 2.0;
        let desert_value = (desert_noise + 1.0) / 2.0;

        // Apply terrain types based on noise thresholds
        if desert_value > cfg.desert_threshold && height > 60 && height < 100 {
            // Desert zones in mid-elevation dry areas
            return "Desert".to_string();
        }

        if forest_value > cfg.forest_threshold && height > 65 && height < 140 {
            // Forests in suitable elevation range
            return "Forest".to_string();
        }

        // Height-based terrain for remaining tiles
        if height > 100 {
            // Higher plains - more varied
            if (world_x + world_y) % 7 == 0 {
                "Dirt".to_string()
            } else if (world_x + world_y) % 11 == 0 {
                "Stone".to_string()
            } else {
                "Grass".to_string()
            }
        } else if height > 70 {
            // Mid plains - mostly grass
            if (world_x * 3 + world_y * 2) % 13 == 0 {
                "Dirt".to_string()
            } else {
                "Grass".to_string()
            }
        } else {
            // Lower plains near beach
            if (world_x + world_y) % 5 == 0 {
                "Sand".to_string()
            } else {
                "Grass".to_string()
            }
        }
    }

    /// Generate height map for a chunk using Perlin noise
    ///
    /// OpenRCT2 mode: Pure Fractional Brownian Motion noise (0-255 range)
    /// Island mode: Noise + circular island pattern for backward compatibility
    ///
    /// Perlin noise adds natural variation to create hills/valleys
    pub fn generate_height_chunk(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<u8>> {
        match self.config.terrain_generation_mode {
            TerrainGenerationMode::CircularIsland => {
                self.generate_height_chunk_island(chunk_x, chunk_y)
            }
            TerrainGenerationMode::OpenRCT2Heights => {
                self.generate_height_chunk_openrct2(chunk_x, chunk_y)
            }
        }
    }

    /// Pure OpenRCT2-style height generation using Fractional Brownian Motion
    /// No circular island bias - pure procedural terrain
    fn generate_height_chunk_openrct2(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<u8>> {
        let mut heights = Vec::with_capacity(16);
        let perlin = Perlin::new(self.config.seed as u32);

        // OpenRCT2-style Fractional Brownian Motion (fBm) parameters
        // Reference: OpenRCT2/src/openrct2/world/map_generator/SimplexNoise.cpp:194
        let base_freq = 0.015; // Base frequency (lower = larger features)
        let octaves = 6; // Number of noise layers (OpenRCT2 uses 6)
        let lacunarity = 2.0; // Frequency multiplier per octave
        let persistence = 0.65; // Amplitude multiplier per octave

        // Height range for normalization
        let min_height = 0.0;
        let max_height = 255.0;

        // Generate heights with fBm
        for y in 0..16 {
            let mut row = Vec::with_capacity(16);
            for x in 0..16 {
                let world_x = chunk_x * 16 + x;
                let world_y = chunk_y * 16 + y;

                // Apply Fractional Brownian Motion (multiple octaves)
                let mut noise_value = 0.0;
                let mut freq = base_freq;
                let mut amp = 1.0;
                let mut total_amp = 0.0;

                for _ in 0..octaves {
                    let sample_x = (world_x as f64) * freq;
                    let sample_y = (world_y as f64) * freq;
                    noise_value += perlin.get([sample_x, sample_y]) * amp;
                    total_amp += amp;
                    freq *= lacunarity;
                    amp *= persistence;
                }

                // Normalize from [-total_amp, total_amp] to [0, 1]
                let normalized = (noise_value / total_amp + 1.0) / 2.0;

                // Map to height range [0, 255]
                let height = min_height + (normalized * (max_height - min_height));

                // Clamp to u8 range
                let final_height = height.max(0.0).min(255.0) as u8;
                row.push(final_height);
            }
            heights.push(row);
        }

        // Apply smoothing (OpenRCT2 does 2-7 passes, we'll do 3)
        // Reference: OpenRCT2/src/openrct2/world/map_generator/SimplexNoise.cpp:212
        for _ in 0..3 {
            heights = self.smooth_heights(heights);
        }

        heights
    }

    /// Legacy height generation with circular island pattern
    fn generate_height_chunk_island(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<u8>> {
        let mut heights = Vec::with_capacity(16);
        let perlin = Perlin::new(self.config.seed as u32);

        let base_freq = 0.015;
        let octaves = 4;
        let lacunarity = 2.0;
        let persistence = 0.65;
        let amplitude = 8.0;

        for y in 0..16 {
            let mut row = Vec::with_capacity(16);
            for x in 0..16 {
                let world_x = chunk_x * 16 + x;
                let world_y = chunk_y * 16 + y;

                // Get base height from island pattern
                let base = self.calculate_base_height(world_x, world_y) as f64;

                // Apply noise variation
                let mut noise_value = 0.0;
                let mut freq = base_freq;
                let mut amp = 1.0;
                let mut total_amp = 0.0;

                for _ in 0..octaves {
                    let sample_x = (world_x as f64) * freq;
                    let sample_y = (world_y as f64) * freq;
                    noise_value += perlin.get([sample_x, sample_y]) * amp;
                    total_amp += amp;
                    freq *= lacunarity;
                    amp *= persistence;
                }

                noise_value /= total_amp;
                let height = base + (noise_value * amplitude);

                let final_height = height.max(0.0).min(255.0) as u8;
                row.push(final_height);
            }
            heights.push(row);
        }

        for _ in 0..3 {
            heights = self.smooth_heights(heights);
        }

        heights
    }

    /// Smooth height map using 3×3 box filter (OpenRCT2 style)
    /// Reference: OpenRCT2/src/openrct2/world/map_generator/SimplexNoise.cpp:166-174
    fn smooth_heights(&self, heights: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let mut smoothed = Vec::with_capacity(16);

        for y in 0..16 {
            let mut row = Vec::with_capacity(16);
            for x in 0..16 {
                let mut sum = 0;
                let mut count = 0;

                // 3×3 box filter
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let ny = (y as i32 + dy).clamp(0, 15) as usize;
                        let nx = (x as i32 + dx).clamp(0, 15) as usize;
                        sum += heights[ny][nx] as u32;
                        count += 1;
                    }
                }

                row.push((sum / count) as u8);
            }
            smoothed.push(row);
        }

        smoothed
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
