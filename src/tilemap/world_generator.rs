use super::{BiomeType, Chunk, ChunkCoordinate, CHUNK_SIZE, TerrainType};
use bevy::log::debug;
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        rng: RwLock::new(rng)
    }
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
                            chunk.coordinate.x,
                            chunk.coordinate.y,
                            x,
                            y,
                            terrain
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