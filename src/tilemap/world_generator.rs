use super::openrct2::{
    generate_simplex_noise, smooth_height_map, HeightMap, OpenRct2Settings,
};
use super::{BiomeType, Chunk, ChunkCoordinate, TerrainType, CHUNK_SIZE};
use bevy::log::debug;
use bevy::math::IVec2;
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ChunkHeightData {
    pub heights: Vec<Vec<u8>>,
    pub slope_masks: Vec<Vec<u8>>,
    pub slope_indices: Vec<Vec<u8>>,
}

const TILE_SLOPE_N_CORNER_UP: u8 = 0b0000_0001;
const TILE_SLOPE_E_CORNER_UP: u8 = 0b0000_0010;
const TILE_SLOPE_S_CORNER_UP: u8 = 0b0000_0100;
const TILE_SLOPE_W_CORNER_UP: u8 = 0b0000_1000;

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

/// Whole-map height storage for OpenRCT2-style generation
/// Stores ALL tile heights before smoothing, allowing cross-chunk propagation
pub struct WholeMapHeights {
    /// All tile heights indexed by world coordinates
    /// Key: (world_x, world_y), Value: height (0-255 units)
    heights: HashMap<(i32, i32), u8>,
    /// Bounding box of the map
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl WholeMapHeights {
    pub fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            heights: HashMap::new(),
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn get_height(&self, x: i32, y: i32) -> i32 {
        self.heights.get(&(x, y)).copied().unwrap_or(0) as i32
    }

    pub fn set_height(&mut self, x: i32, y: i32, height: u8) {
        self.heights.insert((x, y), height);
    }
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
        if let Ok(mut rng) = self.rng.write() {
            *rng = Pcg64::seed_from_u64(seed);
        } else {
            error!("Failed to acquire RNG write lock");
        }
    }

    pub fn generate_chunk(&self, coordinate: ChunkCoordinate) -> Chunk {
        let mut chunk = Chunk::new(coordinate, self.config.seed);

        if self.config.enable_resources {
            self.add_resources_to_chunk(&mut chunk);
        }

        let height_data = self.generate_height_chunk(coordinate.x, coordinate.y);
        for (y, row) in height_data.heights.iter().enumerate().take(CHUNK_SIZE) {
            for (x, value) in row.iter().enumerate().take(CHUNK_SIZE) {
                chunk.heights[y][x] = *value;
            }
        }

        for (y, row) in height_data
            .slope_masks
            .iter()
            .enumerate()
            .take(CHUNK_SIZE)
        {
            for (x, value) in row.iter().enumerate().take(CHUNK_SIZE) {
                chunk.slope_masks[y][x] = *value;
            }
        }

        for (y, row) in height_data
            .slope_indices
            .iter()
            .enumerate()
            .take(CHUNK_SIZE)
        {
            for (x, value) in row.iter().enumerate().take(CHUNK_SIZE) {
                chunk.slope_indices[y][x] = *value;
            }
        }

        chunk
    }

    fn add_resources_to_chunk(&self, chunk: &mut Chunk) {
        for y in 0..chunk.tiles.len() {
            for x in 0..chunk.tiles[y].len() {
                if let Ok(rng) = self.rng.write() {
                if rng.gen::<f32>() < self.config.resource_density {
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
            let (chunk_x, chunk_y) = if let Ok(rng) = self.rng.write() {
                (
                    rng.gen_range(min_x..=max_x),
                    rng.gen_range(min_y..=max_y)
                )
            } else {
                error!("Failed to acquire RNG write lock for spawn point");
                return None;
            };
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
        let height_data = self.generate_height_chunk(chunk_x, chunk_y);
        let heights = height_data.heights;

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

    /// Generate terrain types from pre-computed heights (for whole-map generation)
    /// Used after Phase 2 whole-map smoothing to avoid regenerating heights
    pub fn generate_openrct2_chunk_from_heights(&self, chunk_x: i32, chunk_y: i32, heights: &Vec<Vec<u8>>) -> Vec<Vec<String>> {
        let mut chunk = Vec::with_capacity(16);

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

    /// Generate initial heights for ALL chunks BEFORE smoothing
    /// Phase 1 of OpenRCT2-exact generation
    pub fn generate_all_initial_heights(&self, chunks: &[(i32, i32)]) -> WholeMapHeights {
        const DENSITY: i32 = 2;
        const COORDS_Z_STEP: i32 = 8;

        println!("🌍 Phase 1: Generating initial heights for {} chunks...", chunks.len());

        // Calculate bounding box
        let min_chunk_x = chunks.iter().map(|(x, _)| *x).min().unwrap_or(0);
        let max_chunk_x = chunks.iter().map(|(x, _)| *x).max().unwrap_or(0);
        let min_chunk_y = chunks.iter().map(|(_, y)| *y).min().unwrap_or(0);
        let max_chunk_y = chunks.iter().map(|(_, y)| *y).max().unwrap_or(0);

        let min_world_x = min_chunk_x * CHUNK_SIZE as i32;
        let max_world_x = (max_chunk_x + 1) * CHUNK_SIZE as i32;
        let min_world_y = min_chunk_y * CHUNK_SIZE as i32;
        let max_world_y = (max_chunk_y + 1) * CHUNK_SIZE as i32;

        let mut whole_map = WholeMapHeights::new(min_world_x, min_world_y, max_world_x, max_world_y);

        // Generate simplex noise settings
        let mut settings = OpenRct2Settings::default();
        let map_tiles = self.config.world_size_chunks.max(1) * CHUNK_SIZE as i32;
        settings.map_size = IVec2::new(map_tiles, map_tiles);

        // Generate heights for all chunks
        for &(chunk_x, chunk_y) in chunks {
            let world_origin_x = chunk_x * CHUNK_SIZE as i32;
            let world_origin_y = chunk_y * CHUNK_SIZE as i32;

            // Generate height map for this chunk (with simplex noise + blur)
            let width_tiles = CHUNK_SIZE as i32;
            let height_tiles = CHUNK_SIZE as i32;

            let mut height_map = HeightMap::with_density(
                width_tiles as usize,
                height_tiles as usize,
                DENSITY as u8,
            );

            let origin_samples = IVec2::new(world_origin_x * DENSITY, world_origin_y * DENSITY);
            generate_simplex_noise(&settings, self.config.seed, origin_samples, &mut height_map);

            // Blur height map (2-7 iterations like OpenRCT2)
            let chunk_hash = ((chunk_x as i64 as u64) << 32) ^ ((chunk_y as i64 as u64) & 0xFFFF_FFFF);
            let mut rng = Pcg64::seed_from_u64(self.config.seed ^ chunk_hash);
            let smooth_iterations = 2 + rng.gen_range(0..6);
            smooth_height_map(smooth_iterations as u32, &mut height_map);

            // Convert to tile heights
            let water_level = settings.water_level;
            for tile_y in 0..CHUNK_SIZE as i32 {
                let world_y = world_origin_y + tile_y;
                let y_idx = tile_y;

                for tile_x in 0..CHUNK_SIZE as i32 {
                    let world_x = world_origin_x + tile_x;
                    let x_idx = tile_x;

                    let height_x = x_idx * DENSITY;
                    let height_y = y_idx * DENSITY;

                    let q00 = height_map.get(IVec2::new(height_x, height_y)) as i32;
                    let q01 = height_map.get(IVec2::new(height_x, height_y + 1)) as i32;
                    let q10 = height_map.get(IVec2::new(height_x + 1, height_y)) as i32;
                    let q11 = height_map.get(IVec2::new(height_x + 1, height_y + 1)) as i32;

                    let average_height = (q00 + q01 + q10 + q11) / 4;
                    // Note: Don't multiply by 2 here - simplex noise already has proper range
                    // OpenRCT2 divides heightmap_low/high by 2 in noise generation,
                    // so we get values like [7, 30] which map directly to levels
                    let mut base_height = average_height.max(2);

                    if base_height >= 4 && base_height <= water_level {
                        base_height -= 2;
                    }

                    let mut final_height = base_height * COORDS_Z_STEP;
                    if final_height > 255 {
                        final_height = 255;
                    }

                    whole_map.set_height(world_x, world_y, final_height as u8);
                }
            }
        }

        // Sample initial heights for debugging
        let sample_heights: Vec<i32> = whole_map.heights.values().take(100).map(|&h| h as i32).collect();
        let min_h = sample_heights.iter().min().unwrap_or(&0);
        let max_h = sample_heights.iter().max().unwrap_or(&0);
        let avg_h = if !sample_heights.is_empty() {
            sample_heights.iter().sum::<i32>() / sample_heights.len() as i32
        } else {
            0
        };

        println!("✅ Phase 1 complete: {} tiles initialized", whole_map.heights.len());
        println!("   Initial height range: min={}, max={}, avg={} (sample of 100 tiles)", min_h, max_h, avg_h);
        whole_map
    }

    /// Run whole-map smoothing EXACTLY like OpenRCT2's smoothMap function
    /// Phase 2 of OpenRCT2-exact generation
    pub fn smooth_whole_map(&self, whole_map: &mut WholeMapHeights) {
        println!("🏔️ Phase 2: Smoothing entire map (OpenRCT2 exact algorithm)...");

        let mut iteration = 0;
        loop {
            iteration += 1;
            let mut num_tiles_changed = 0;

            // Process all tiles (OpenRCT2: for y in 1..mapSize.y-1, for x in 1..mapSize.x-1)
            for world_y in (whole_map.min_y + 1)..(whole_map.max_y - 1) {
                for world_x in (whole_map.min_x + 1)..(whole_map.max_x - 1) {
                    let old_height = whole_map.get_height(world_x, world_y);

                    // Get 8 neighbors
                    let h_n = whole_map.get_height(world_x, world_y - 1);
                    let h_s = whole_map.get_height(world_x, world_y + 1);
                    let h_e = whole_map.get_height(world_x + 1, world_y);
                    let h_w = whole_map.get_height(world_x - 1, world_y);
                    let h_nw = whole_map.get_height(world_x - 1, world_y - 1);
                    let h_ne = whole_map.get_height(world_x + 1, world_y - 1);
                    let h_sw = whole_map.get_height(world_x - 1, world_y + 1);
                    let h_se = whole_map.get_height(world_x + 1, world_y + 1);

                    // Apply smoothTileStrong HEIGHT RAISING logic (no slopes yet)
                    let new_height = self.smooth_tile_strong_height_only(
                        old_height, h_n, h_s, h_e, h_w, h_nw, h_ne, h_sw, h_se
                    );

                    if new_height != old_height {
                        whole_map.set_height(world_x, world_y, new_height as u8);
                        num_tiles_changed += 1;
                    }
                }
            }

            if iteration == 1 || num_tiles_changed > 0 {
                println!("  Iteration {}: {} tiles changed", iteration, num_tiles_changed);
            }

            // Converged when no tiles change (OpenRCT2: if (numTilesChanged == 0) break)
            if num_tiles_changed == 0 {
                // Sample final heights for debugging
                let sample_heights: Vec<i32> = whole_map.heights.values().take(100).map(|&h| h as i32).collect();
                let min_h = sample_heights.iter().min().unwrap_or(&0);
                let max_h = sample_heights.iter().max().unwrap_or(&0);
                let avg_h = if !sample_heights.is_empty() {
                    sample_heights.iter().sum::<i32>() / sample_heights.len() as i32
                } else {
                    0
                };

                println!("✅ Phase 2 complete: Converged after {} iterations", iteration);
                println!("   Final height range: min={}, max={}, avg={} (sample of 100 tiles)", min_h, max_h, avg_h);
                break;
            }

            // Safety limit
            if iteration >= 100 {
                println!("⚠️  WARNING: Did not converge after 100 iterations!");
                break;
            }
        }
    }

    /// Apply smoothTile Strong height raising logic ONLY (no slope calculation)
    /// Used during Phase 2 whole-map smoothing
    fn smooth_tile_strong_height_only(
        &self,
        mut current_height: i32,
        h_n: i32, h_s: i32, h_e: i32, h_w: i32,
        h_nw: i32, h_ne: i32, h_sw: i32, h_se: i32,
    ) -> i32 {
        // EXACT copy of lines 695-773 from generate_height_chunk_openrct2
        // but ONLY the height raising parts, NO slope calculation

        // Step 1: Raise to edge height - 2 levels (16 units)
        let mut highest_orthogonal = current_height;
        highest_orthogonal = highest_orthogonal.max(h_w);
        highest_orthogonal = highest_orthogonal.max(h_e);
        highest_orthogonal = highest_orthogonal.max(h_n);
        highest_orthogonal = highest_orthogonal.max(h_s);

        if current_height < highest_orthogonal - 16 {
            current_height = (highest_orthogonal - 16).max(0);
        }

        // Step 2: Check diagonal corners
        let corner_heights = [h_nw, h_ne, h_se, h_sw];

        let mut highest_corner = current_height;
        for &corner_h in &corner_heights {
            highest_corner = highest_corner.max(corner_h);
        }

        // Step 3: If highest corner >= current + 4 levels (32 units), check for diagonal
        if highest_corner >= current_height + 32 {
            let mut count = 0;
            let mut corner_idx = 0;
            let mut can_compensate = true;

            for (i, &corner_h) in corner_heights.iter().enumerate() {
                if corner_h == highest_corner {
                    count += 1;
                    corner_idx = i;

                    let highest_on_lowest_side = match i {
                        0 => h_e.max(h_s),
                        1 => h_w.max(h_s),
                        2 => h_w.max(h_n),
                        3 => h_e.max(h_n),
                        _ => current_height,
                    };

                    if highest_on_lowest_side > current_height {
                        current_height = highest_on_lowest_side;
                        can_compensate = false;
                    }
                }
            }

            if count == 1 && can_compensate {
                if current_height < highest_corner - 32 {
                    current_height = (highest_corner - 32).max(0);
                }
                // Note: We DON'T set diagonal flag here - that's done in finalization
            } else {
                if current_height < highest_corner - 16 {
                    current_height = (highest_corner - 16).max(0);
                }
            }
        }

        // Clamp to valid range [0, 255]
        current_height.clamp(0, 255)
    }

    /// Extract final chunk data from whole map after smoothing (Phase 3)
    pub fn finalize_chunk_from_whole_map(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        whole_map: &WholeMapHeights,
    ) -> ChunkHeightData {
        let mut heights = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];
        let mut slope_masks = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];
        let mut slope_indices = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];

        let world_origin_x = chunk_x * CHUNK_SIZE as i32;
        let world_origin_y = chunk_y * CHUNK_SIZE as i32;

        // Copy final heights from whole map
        for local_y in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = world_origin_x + local_x as i32;
                let world_y = world_origin_y + local_y as i32;
                heights[local_y][local_x] = whole_map.get_height(world_x, world_y) as u8;
            }
        }

        // Calculate slopes based on final smoothed heights
        for local_y in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = world_origin_x + local_x as i32;
                let world_y = world_origin_y + local_y as i32;
                let final_height = whole_map.get_height(world_x, world_y);

                // Get all 8 neighbors
                let h_n = whole_map.get_height(world_x, world_y - 1);
                let h_s = whole_map.get_height(world_x, world_y + 1);
                let h_e = whole_map.get_height(world_x + 1, world_y);
                let h_w = whole_map.get_height(world_x - 1, world_y);
                let h_nw = whole_map.get_height(world_x - 1, world_y - 1);
                let h_ne = whole_map.get_height(world_x + 1, world_y - 1);
                let h_sw = whole_map.get_height(world_x - 1, world_y + 1);
                let h_se = whole_map.get_height(world_x + 1, world_y + 1);

                // Calculate slope using the FULL logic (both PATH 1 diagonal + PATH 2 normal)
                let slope_bits = self.calculate_slope_for_tile(
                    final_height, h_n, h_s, h_e, h_w, h_nw, h_ne, h_sw, h_se
                );

                slope_masks[local_y][local_x] = slope_bits;
                slope_indices[local_y][local_x] = slope_mask_to_index(slope_bits);
            }
        }

        ChunkHeightData {
            heights,
            slope_masks,
            slope_indices,
        }
    }

    /// Calculate slope bits for a tile (both diagonal and normal slopes)
    /// Used during Phase 3 finalization
    fn calculate_slope_for_tile(
        &self,
        final_height: i32,
        h_n: i32, h_s: i32, h_e: i32, h_w: i32,
        h_nw: i32, h_ne: i32, h_sw: i32, h_se: i32,
    ) -> u8 {
        const TILE_SLOPE_DIAGONAL_FLAG: u8 = 0b0001_0000;
        const TILE_SLOPE_RAISED_CORNERS_MASK: u8 = 0b0000_1111;
        const TILE_SLOPE_N_CORNER_UP: u8 = 0b0000_0001;
        const TILE_SLOPE_E_CORNER_UP: u8 = 0b0000_0010;
        const TILE_SLOPE_S_CORNER_UP: u8 = 0b0000_0100;
        const TILE_SLOPE_W_CORNER_UP: u8 = 0b0000_1000;
        const TILE_SLOPE_W_CORNER_DOWN: u8 = TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_W_CORNER_UP;
        const TILE_SLOPE_S_CORNER_DOWN: u8 = TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_S_CORNER_UP;
        const TILE_SLOPE_E_CORNER_DOWN: u8 = TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_E_CORNER_UP;
        const TILE_SLOPE_N_CORNER_DOWN: u8 = TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_N_CORNER_UP;
        const TILE_SLOPE_NE_SIDE_UP: u8 = TILE_SLOPE_N_CORNER_UP | TILE_SLOPE_E_CORNER_UP;
        const TILE_SLOPE_SE_SIDE_UP: u8 = TILE_SLOPE_S_CORNER_UP | TILE_SLOPE_E_CORNER_UP;
        const TILE_SLOPE_SW_SIDE_UP: u8 = TILE_SLOPE_S_CORNER_UP | TILE_SLOPE_W_CORNER_UP;
        const TILE_SLOPE_NW_SIDE_UP: u8 = TILE_SLOPE_N_CORNER_UP | TILE_SLOPE_W_CORNER_UP;

        // Check for diagonal slope first (PATH 1)
        let corner_heights = [h_nw, h_ne, h_se, h_sw];
        let mut highest_corner = final_height;
        for &corner_h in &corner_heights {
            highest_corner = highest_corner.max(corner_h);
        }

        if highest_corner >= final_height + 32 {
            let mut count = 0;
            let mut corner_idx = 0;

            for (i, &corner_h) in corner_heights.iter().enumerate() {
                if corner_h == highest_corner {
                    count += 1;
                    corner_idx = i;
                }
            }

            if count == 1 {
                // Check opposite corner
                let opposite_corner_low = match corner_idx {
                    0 => corner_heights[2] <= corner_heights[0] - 32,
                    1 => corner_heights[3] <= corner_heights[1] - 32,
                    2 => corner_heights[0] <= corner_heights[2] - 32,
                    3 => corner_heights[1] <= corner_heights[3] - 32,
                    _ => false,
                };

                if opposite_corner_low {
                    // This is a diagonal slope!
                    let mut slope_bits = TILE_SLOPE_DIAGONAL_FLAG;
                    match corner_idx {
                        0 => slope_bits |= TILE_SLOPE_N_CORNER_DOWN,
                        1 => slope_bits |= TILE_SLOPE_W_CORNER_DOWN,
                        2 => slope_bits |= TILE_SLOPE_S_CORNER_DOWN,
                        3 => slope_bits |= TILE_SLOPE_E_CORNER_DOWN,
                        _ => {}
                    }
                    return slope_bits;
                }
            }
        }

        // PATH 2: Normal slope calculation
        let mut slope_bits = 0u8;

        // Diagonal neighbors raise individual corners
        if h_se > final_height {
            slope_bits |= TILE_SLOPE_N_CORNER_UP;
        }
        if h_sw > final_height {
            slope_bits |= TILE_SLOPE_W_CORNER_UP;
        }
        if h_ne > final_height {
            slope_bits |= TILE_SLOPE_E_CORNER_UP;
        }
        if h_nw > final_height {
            slope_bits |= TILE_SLOPE_S_CORNER_UP;
        }

        // Orthogonal neighbors raise sides
        if h_e > final_height {
            slope_bits |= TILE_SLOPE_NE_SIDE_UP;
        }
        if h_w > final_height {
            slope_bits |= TILE_SLOPE_SW_SIDE_UP;
        }
        if h_n > final_height {
            slope_bits |= TILE_SLOPE_SE_SIDE_UP;
        }
        if h_s > final_height {
            slope_bits |= TILE_SLOPE_NW_SIDE_UP;
        }

        // If all corners raised, this should have been raised during smoothing
        // (we don't raise here because heights are already final)
        if slope_bits == TILE_SLOPE_RAISED_CORNERS_MASK {
            slope_bits = 0; // Flat (height should already be raised)
        }

        slope_bits
    }

    /// Generate height map for a chunk using Perlin noise
    ///
    /// OpenRCT2 mode: Pure Fractional Brownian Motion noise (0-255 range)
    /// Island mode: Noise + circular island pattern for backward compatibility
    ///
    /// Perlin noise adds natural variation to create hills/valleys
    pub fn generate_height_chunk(&self, chunk_x: i32, chunk_y: i32) -> ChunkHeightData {
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
    fn generate_height_chunk_openrct2(&self, chunk_x: i32, chunk_y: i32) -> ChunkHeightData {
        const DENSITY: i32 = 2;
        const COORDS_Z_STEP: i32 = 8;

        let world_origin_x = chunk_x * CHUNK_SIZE as i32;
        let world_origin_y = chunk_y * CHUNK_SIZE as i32;
        let world_min_x = world_origin_x - 1;
        let world_min_y = world_origin_y - 1;

        let width_tiles = CHUNK_SIZE as i32 + 2;
        let height_tiles = CHUNK_SIZE as i32 + 2;

        let mut settings = OpenRct2Settings::default();
        let map_tiles = self.config.world_size_chunks.max(1) * CHUNK_SIZE as i32;
        settings.map_size = IVec2::new(map_tiles, map_tiles);

        let mut height_map = HeightMap::with_density(
            width_tiles as usize,
            height_tiles as usize,
            DENSITY as u8,
        );

        let origin_samples = IVec2::new(world_min_x * DENSITY, world_min_y * DENSITY);
        generate_simplex_noise(
            &settings,
            self.config.seed,
            origin_samples,
            &mut height_map,
        );

        let chunk_hash =
            ((chunk_x as i64 as u64) << 32) ^ ((chunk_y as i64 as u64) & 0xFFFF_FFFF);
        let mut rng = Pcg64::seed_from_u64(self.config.seed ^ chunk_hash);
        let smooth_iterations = 2 + rng.gen_range(0..6);
        smooth_height_map(smooth_iterations as u32, &mut height_map);

        let grid_size = (CHUNK_SIZE as i32 + 2) as usize;
        let mut final_heights_border = vec![vec![0i32; grid_size]; grid_size];
        let water_level = settings.water_level;

        for tile_y in -1..=CHUNK_SIZE as i32 {
            let world_y = world_origin_y + tile_y;
            let y_idx = world_y - world_min_y;
            let height_y = y_idx * DENSITY;

            for tile_x in -1..=CHUNK_SIZE as i32 {
                let world_x = world_origin_x + tile_x;
                let x_idx = world_x - world_min_x;
                let height_x = x_idx * DENSITY;

                let q00 = height_map.get(IVec2::new(height_x, height_y)) as i32;
                let q01 = height_map.get(IVec2::new(height_x, height_y + 1)) as i32;
                let q10 = height_map.get(IVec2::new(height_x + 1, height_y)) as i32;
                let q11 = height_map.get(IVec2::new(height_x + 1, height_y + 1)) as i32;

                let average_height = (q00 + q01 + q10 + q11) / 4;
                let mut base_height = (average_height * 2).max(2);

                if base_height >= 4 && base_height <= water_level {
                    base_height -= 2;
                }

                let mut final_height = base_height * COORDS_Z_STEP;
                if final_height > 255 {
                    final_height = 255;
                }

                let array_y = (tile_y + 1) as usize;
                let array_x = (tile_x + 1) as usize;
                final_heights_border[array_y][array_x] = final_height;
            }
        }

        let mut heights = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];
        let mut slope_masks = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];
        let mut slope_indices = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];

        // Initialize heights from the height map first
        for local_y in 0..CHUNK_SIZE {
            let border_y = local_y + 1;
            for local_x in 0..CHUNK_SIZE {
                let border_x = local_x + 1;
                heights[local_y][local_x] = final_heights_border[border_y][border_x] as u8;
            }
        }

        // Constants for slope detection
        const TILE_SLOPE_DIAGONAL_FLAG: u8 = 0b0001_0000;
        const TILE_SLOPE_RAISED_CORNERS_MASK: u8 = 0b0000_1111;
        const TILE_SLOPE_W_CORNER_DOWN: u8 =
            TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_W_CORNER_UP;
        const TILE_SLOPE_S_CORNER_DOWN: u8 =
            TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_S_CORNER_UP;
        const TILE_SLOPE_E_CORNER_DOWN: u8 =
            TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_E_CORNER_UP;
        const TILE_SLOPE_N_CORNER_DOWN: u8 =
            TILE_SLOPE_RAISED_CORNERS_MASK & !TILE_SLOPE_N_CORNER_UP;

        // Side constants (orthogonal neighbors raise two corners)
        const TILE_SLOPE_NE_SIDE_UP: u8 = TILE_SLOPE_N_CORNER_UP | TILE_SLOPE_E_CORNER_UP;
        const TILE_SLOPE_SE_SIDE_UP: u8 = TILE_SLOPE_S_CORNER_UP | TILE_SLOPE_E_CORNER_UP;
        const TILE_SLOPE_SW_SIDE_UP: u8 = TILE_SLOPE_S_CORNER_UP | TILE_SLOPE_W_CORNER_UP;
        const TILE_SLOPE_NW_SIDE_UP: u8 = TILE_SLOPE_N_CORNER_UP | TILE_SLOPE_W_CORNER_UP;

        // OpenRCT2 iterative smoothing: run until no tiles change
        // This allows terrain to "propagate" and settle
        let mut iteration = 0;
        loop {
            iteration += 1;
            let mut num_tiles_changed = 0;
            let mut heights_raised = 0;
            let mut slopes_changed = 0;

        for local_y in 0..CHUNK_SIZE {
            let tile_y = local_y as i32;
            let world_y = world_origin_y + tile_y;
            let y_idx = world_y - world_min_y;
            let height_y = y_idx * DENSITY;
            let border_y = (tile_y + 1) as usize;

            for local_x in 0..CHUNK_SIZE {
                let tile_x = local_x as i32;
                let world_x = world_origin_x + tile_x;
                let x_idx = world_x - world_min_x;
                let height_x = x_idx * DENSITY;
                let border_x = (tile_x + 1) as usize;

                // Use the potentially-modified height from previous iterations
                let final_height = heights[local_y][local_x] as i32;

                // Get all 8 neighbor heights from the potentially-modified heights array
                // Need to check bounds and fall back to border heights for edge tiles
                let h_n = if local_y > 0 {
                    heights[local_y - 1][local_x] as i32
                } else {
                    final_heights_border[border_y.saturating_sub(1)][border_x]
                };
                let h_s = if local_y < CHUNK_SIZE - 1 {
                    heights[local_y + 1][local_x] as i32
                } else {
                    final_heights_border[border_y + 1][border_x]
                };
                let h_w = if local_x > 0 {
                    heights[local_y][local_x - 1] as i32
                } else {
                    final_heights_border[border_y][border_x.saturating_sub(1)]
                };
                let h_e = if local_x < CHUNK_SIZE - 1 {
                    heights[local_y][local_x + 1] as i32
                } else {
                    final_heights_border[border_y][border_x + 1]
                };

                let h_nw = if local_y > 0 && local_x > 0 {
                    heights[local_y - 1][local_x - 1] as i32
                } else {
                    final_heights_border[border_y.saturating_sub(1)][border_x.saturating_sub(1)]
                };
                let h_ne = if local_y > 0 && local_x < CHUNK_SIZE - 1 {
                    heights[local_y - 1][local_x + 1] as i32
                } else {
                    final_heights_border[border_y.saturating_sub(1)][border_x + 1]
                };
                let h_sw = if local_y < CHUNK_SIZE - 1 && local_x > 0 {
                    heights[local_y + 1][local_x - 1] as i32
                } else {
                    final_heights_border[border_y + 1][border_x.saturating_sub(1)]
                };
                let h_se = if local_y < CHUNK_SIZE - 1 && local_x < CHUNK_SIZE - 1 {
                    heights[local_y + 1][local_x + 1] as i32
                } else {
                    final_heights_border[border_y + 1][border_x + 1]
                };

                // OpenRCT2 EXACT smoothTileStrong algorithm (MapHelpers.cpp lines 35-194)
                //
                // UNIT SYSTEM CRITICAL INFO:
                // - OpenRCT2 BaseHeight: in "levels" (0-255, each level is a discrete height)
                // - Our heights array: in "units" (0-255, where 8 units = 1 level)
                // - COORDS_Z_STEP = 8 (units per level)
                // - All OpenRCT2 operations like "-2" must be multiplied by 8 in our code!
                //   Example: OpenRCT2 "highest - 2" = Our "highest - 16" (2 levels × 8 units/level)
                //
                // Two-path algorithm:
                // PATH 1 (lines 40-149): Diagonal detection via corner analysis
                // PATH 2 (lines 151-194): Normal slope calculation

                let old_height = heights[local_y][local_x];
                let old_slope = slope_masks[local_y][local_x];  // For statistics tracking

                // OpenRCT2 Line 35: Reset slope to flat before processing
                // CRITICAL: Always recalculate slope from scratch based on current heights
                slope_masks[local_y][local_x] = 0;

                // PATH 1: Diagonal slope detection (lines 40-149)

                // Step 1: Raise to edge height - 2 (lines 42-50)
                // OpenRCT2: "highest - 2" (in levels)
                // Our code: "highest - 16" (2 levels × 8 units/level = 16 units)
                let mut highest_orthogonal = final_height;
                highest_orthogonal = highest_orthogonal.max(h_w);
                highest_orthogonal = highest_orthogonal.max(h_e);
                highest_orthogonal = highest_orthogonal.max(h_n);
                highest_orthogonal = highest_orthogonal.max(h_s);

                if final_height < highest_orthogonal - 16 {
                    heights[local_y][local_x] = ((highest_orthogonal - 16).max(0)) as u8;
                }

                // Update final_height after potential raise
                let final_height = heights[local_y][local_x] as i32;

                // Step 2: Check 4 diagonal corners (lines 52-58)
                let corner_heights = [h_nw, h_ne, h_se, h_sw];  // NW=0, NE=1, SE=2, SW=3

                // Step 3: Find highest corner (lines 60-62)
                let mut highest_corner = final_height;
                for &corner_h in &corner_heights {
                    highest_corner = highest_corner.max(corner_h);
                }

                // Step 4: If highest corner >= current + 4, check for diagonal (lines 64-129)
                let mut double_corner: Option<usize> = None;

                if highest_corner >= final_height + 32 {  // +4 in OpenRCT2 units = +32 in our units (×8)
                    // Count how many corners are at highest AND check canCompensate (lines 67-103)
                    let mut count = 0;
                    let mut corner_idx = 0;
                    let mut can_compensate = true;

                    for (i, &corner_h) in corner_heights.iter().enumerate() {
                        if corner_h == highest_corner {
                            count += 1;
                            corner_idx = i;

                            // CRITICAL: Check if surrounding corners aren't too high (lines 75-102)
                            // Get the two orthogonal neighbors on the OPPOSITE side of this corner
                            // If they're higher than current tile, we can't use diagonal compensation
                            let highest_on_lowest_side = match i {
                                0 => h_e.max(h_s),  // NW corner highest → check E, S (opposite side)
                                1 => h_w.max(h_s),  // NE corner highest → check W, S
                                2 => h_w.max(h_n),  // SE corner highest → check W, N
                                3 => h_e.max(h_n),  // SW corner highest → check E, N
                                _ => final_height,
                            };

                            if highest_on_lowest_side > final_height {
                                // Opposite side is too high - raise tile and disable diagonal
                                heights[local_y][local_x] = highest_on_lowest_side as u8;
                                can_compensate = false;
                            }
                        }
                    }

                    // Update final_height after potential raise from canCompensate check
                    let final_height = heights[local_y][local_x] as i32;

                    // Lines 105-120: Try diagonal if count == 1 AND canCompensate
                    if count == 1 && can_compensate {
                        // Raise tile to highest - 4 (lines 107-110)
                        if final_height < highest_corner - 32 {
                            heights[local_y][local_x] = ((highest_corner - 32).max(0)) as u8;
                        }

                        // CRITICAL: Verify opposite diagonal corner is low enough (lines 112-119)
                        // The opposite corner must be ≤ highest - 32 for diagonal to work
                        let opposite_corner_low = match corner_idx {
                            0 => corner_heights[2] <= corner_heights[0] - 32,  // NW high → SE must be low
                            1 => corner_heights[3] <= corner_heights[1] - 32,  // NE high → SW must be low
                            2 => corner_heights[0] <= corner_heights[2] - 32,  // SE high → NW must be low
                            3 => corner_heights[1] <= corner_heights[3] - 32,  // SW high → NE must be low
                            _ => false,
                        };

                        if opposite_corner_low {
                            double_corner = Some(corner_idx);
                        }
                        // If opposite corner not low enough, doubleCorner stays None
                        // and we fall through to Path 2 (normal slopes)
                    } else {
                        // Lines 121-128: Fallback when count != 1 OR can't compensate
                        // Raise to highest - 2 (not highest - 4!)
                        let final_height = heights[local_y][local_x] as i32;
                        if final_height < highest_corner - 16 {  // -2 in OpenRCT2 = -16 in our units
                            heights[local_y][local_x] = ((highest_corner - 16).max(0)) as u8;
                        }
                    }
                }

                let mut slope_bits = 0u8;

                // Step 5: If diagonal detected, set flags and skip normal slope logic (lines 131-149)
                if let Some(corner_idx) = double_corner {
                    slope_bits = TILE_SLOPE_DIAGONAL_FLAG;

                    // Set the corner DOWN flag based on which corner is highest
                    // When NW (0) is highest, N corner is DOWN, etc.
                    match corner_idx {
                        0 => slope_bits |= TILE_SLOPE_N_CORNER_DOWN,  // NW highest → N down
                        1 => slope_bits |= TILE_SLOPE_W_CORNER_DOWN,  // NE highest → W down
                        2 => slope_bits |= TILE_SLOPE_S_CORNER_DOWN,  // SE highest → S down
                        3 => slope_bits |= TILE_SLOPE_E_CORNER_DOWN,  // SW highest → E down
                        _ => {}
                    }

                    slope_masks[local_y][local_x] = slope_bits;
                } else {
                    // PATH 2: Normal slope calculation (lines 151-194)
                    // Only runs if NO diagonal was detected

                    // Update final_height again (may have been raised in Path 1)
                    let final_height = heights[local_y][local_x] as i32;

                    // Step 1: Diagonal neighbors raise individual corners
                    // (x+1, y+1) = SE diagonal
                    if h_se > final_height {
                        slope_bits |= TILE_SLOPE_N_CORNER_UP;
                    }
                    // (x-1, y+1) = SW diagonal
                    if h_sw > final_height {
                        slope_bits |= TILE_SLOPE_W_CORNER_UP;
                    }
                    // (x+1, y-1) = NE diagonal
                    if h_ne > final_height {
                        slope_bits |= TILE_SLOPE_E_CORNER_UP;
                    }
                    // (x-1, y-1) = NW diagonal
                    if h_nw > final_height {
                        slope_bits |= TILE_SLOPE_S_CORNER_UP;
                    }

                    // Step 2: Orthogonal neighbors raise SIDES (two corners each)
                    // OpenRCT2 compares BASE HEIGHTS only, not corner/slope heights!
                    // (x+1, y+0) = East neighbor base height
                    if h_e > final_height {
                        slope_bits |= TILE_SLOPE_NE_SIDE_UP;
                    }
                    // (x-1, y+0) = West neighbor base height
                    if h_w > final_height {
                        slope_bits |= TILE_SLOPE_SW_SIDE_UP;
                    }
                    // (x+0, y-1) = North neighbor base height
                    if h_n > final_height {
                        slope_bits |= TILE_SLOPE_SE_SIDE_UP;
                    }
                    // (x+0, y+1) = South neighbor base height
                    if h_s > final_height {
                        slope_bits |= TILE_SLOPE_NW_SIDE_UP;
                    }

                    // Step 3: If all four corners are raised, raise the base height and flatten
                    // This is EXACTLY what OpenRCT2 does in smoothTileStrong:
                    // if (slope == kTileSlopeRaisedCornersMask) {
                    //     slope = kTileSlopeFlat;
                    //     surfaceElement->BaseHeight = surfaceElement->ClearanceHeight += 2;
                    // }
                    if slope_bits == TILE_SLOPE_RAISED_CORNERS_MASK {
                        // Raise base height by 2 LEVELS (OpenRCT2 exact behavior)
                        // OpenRCT2: "+= 2" (in levels)
                        // Our code: "+= 16" (2 levels × 8 units/level = 16 units)
                        // This prevents holes when surrounded by higher terrain
                        heights[local_y][local_x] = heights[local_y][local_x].saturating_add(16);
                        slope_bits = 0; // Flat
                    }

                    slope_masks[local_y][local_x] = slope_bits;
                }

                // Track if this tile changed (for iteration convergence)
                // CRITICAL: OpenRCT2 only counts HEIGHT changes (raisedLand flag), NOT slope changes!
                // Slopes are always recalculated, but only height raises trigger another iteration.
                if heights[local_y][local_x] != old_height {
                    heights_raised += 1;
                    num_tiles_changed += 1;  // Only height changes count for convergence!
                }

                // Track slope changes for statistics, but don't affect convergence
                if slope_masks[local_y][local_x] != old_slope {
                    slopes_changed += 1;
                }

                let slope_index = slope_mask_to_index(slope_bits);
                slope_indices[local_y][local_x] = slope_index;
            }
        }

            // Log iteration statistics
            if iteration == 1 || num_tiles_changed > 0 {
                println!(
                    "  Chunk ({}, {}) iteration {}: {} tiles changed ({} heights raised, {} slopes changed)",
                    chunk_x, chunk_y, iteration, num_tiles_changed, heights_raised, slopes_changed
                );
            }

            // Check if we've converged (no more changes)
            if num_tiles_changed == 0 {
                println!(
                    "  Chunk ({}, {}) converged after {} iterations",
                    chunk_x, chunk_y, iteration
                );
                break;
            }

            // Safety limit: prevent infinite loops
            if iteration >= 100 {
                println!(
                    "  WARNING: Chunk ({}, {}) did not converge after 100 iterations!",
                    chunk_x, chunk_y
                );
                break;
            }
        } // end of smoothing loop

        ChunkHeightData {
            heights,
            slope_masks,
            slope_indices,
        }
    }

    /// Legacy height generation with circular island pattern
    fn generate_height_chunk_island(&self, chunk_x: i32, chunk_y: i32) -> ChunkHeightData {
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

        let slope_masks = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];
        let slope_indices = vec![vec![0u8; CHUNK_SIZE]; CHUNK_SIZE];

        ChunkHeightData {
            heights,
            slope_masks,
            slope_indices,
        }
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

fn slope_mask_to_index(mask: u8) -> u8 {
    // OpenRCT2 slope index system:
    // - Bits 0-3: Corner flags (N/E/S/W raised)
    // - Bit 4: Diagonal flag (steep slope)
    //
    // Without diagonal flag (0-15): Maps directly to index
    // With diagonal flag (16-31): Different slope types
    //
    // The mask already contains the correct bits from our threshold calculation

    const DIAGONAL_FLAG: u8 = 0b0001_0000;
    const CORNER_MASK: u8 = 0b0000_1111;

    let corners = mask & CORNER_MASK;
    let is_diagonal = (mask & DIAGONAL_FLAG) != 0;

    if !is_diagonal {
        // Normal slopes (0-15): Direct mapping
        corners
    } else {
        // Diagonal/steep slopes (16-31)
        // In OpenRCT2, these have special rendering for double-height corners
        // The Godot viewer uses indices 16-18 for special cases:
        // - 16: Diagonal NE-SW
        // - 17: Diagonal NW-SE
        // - 18: Peak
        //
        // For now, map diagonal flag to index 16/17 based on which corner is down
        match corners {
            0b0111 => 16, // W corner down -> NE-SW diagonal
            0b1011 => 16, // S corner down -> NE-SW diagonal
            0b1101 => 17, // E corner down -> NW-SE diagonal
            0b1110 => 17, // N corner down -> NW-SE diagonal
            _ => corners, // Fallback to normal slope
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
