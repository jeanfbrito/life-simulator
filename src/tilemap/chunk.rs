use super::biome::{BiomeGenerator, BiomeType};
use super::terrain::TerrainType;
use bevy::log::info;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub y: i32,
}

impl ChunkCoordinate {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn from_world_position(world_x: f32, world_y: f32, tile_size: f32) -> Self {
        let tile_x = (world_x / tile_size).floor() as i32;
        let tile_y = (world_y / tile_size).floor() as i32;
        Self::from_tile_position(tile_x, tile_y)
    }

    pub fn from_tile_position(tile_x: i32, tile_y: i32) -> Self {
        Self {
            x: tile_x.div_euclid(CHUNK_SIZE as i32),
            y: tile_y.div_euclid(CHUNK_SIZE as i32),
        }
    }

    pub fn to_world_position(&self, tile_size: f32) -> (f32, f32) {
        let world_x = self.x as f32 * CHUNK_SIZE as f32 * tile_size;
        let world_y = self.y as f32 * CHUNK_SIZE as f32 * tile_size;
        (world_x, world_y)
    }

    pub fn distance_to(&self, other: &ChunkCoordinate) -> i32 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx.max(dy)
    }

    pub fn neighbors(&self) -> Vec<ChunkCoordinate> {
        vec![
            ChunkCoordinate::new(self.x - 1, self.y - 1),
            ChunkCoordinate::new(self.x, self.y - 1),
            ChunkCoordinate::new(self.x + 1, self.y - 1),
            ChunkCoordinate::new(self.x - 1, self.y),
            ChunkCoordinate::new(self.x + 1, self.y),
            ChunkCoordinate::new(self.x - 1, self.y + 1),
            ChunkCoordinate::new(self.x, self.y + 1),
            ChunkCoordinate::new(self.x + 1, self.y + 1),
        ]
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub coordinate: ChunkCoordinate,
    pub tiles: [[TerrainType; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip)]
    pub heights: [[u8; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip)]
    pub slope_masks: [[u8; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip)]
    pub slope_indices: [[u8; CHUNK_SIZE]; CHUNK_SIZE],
    pub biome: BiomeType,
    pub is_dirty: bool,
    pub generation_seed: u64,
}

impl Chunk {
    pub fn new(coordinate: ChunkCoordinate, seed: u64) -> Self {
        // Generate biome for this chunk
        let biome_gen = BiomeGenerator::new(seed);
        let biome = biome_gen.generate_biome(coordinate.x, coordinate.y);

        let mut chunk = Self {
            coordinate,
            tiles: [[TerrainType::Grass; CHUNK_SIZE]; CHUNK_SIZE],
            heights: [[0; CHUNK_SIZE]; CHUNK_SIZE],
            slope_masks: [[0; CHUNK_SIZE]; CHUNK_SIZE],
            slope_indices: [[0; CHUNK_SIZE]; CHUNK_SIZE],
            biome,
            is_dirty: true,
            generation_seed: seed,
        };
        chunk.generate_terrain();
        chunk
    }

    fn generate_terrain(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = self.coordinate.x * CHUNK_SIZE as i32 + x as i32;
                let world_y = self.coordinate.y * CHUNK_SIZE as i32 + y as i32;

                // Add local variation within the chunk
                let local_noise =
                    ((world_x as f32 * 0.1).sin() + (world_y as f32 * 0.1).cos()) * 0.1;
                let random_factor = rng.gen::<f32>() + local_noise;

                // Select terrain based on biome
                self.tiles[y][x] = self.biome.select_terrain(random_factor.clamp(0.0, 1.0));
            }
        }

        info!(
            "[CHUNK] Generated {:?} chunk ({}, {}) with {} tiles",
            self.biome,
            self.coordinate.x,
            self.coordinate.y,
            CHUNK_SIZE * CHUNK_SIZE
        );
    }

    pub fn get_tile(&self, local_x: usize, local_y: usize) -> Option<TerrainType> {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            Some(self.tiles[local_y][local_x])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, local_x: usize, local_y: usize, tile_type: TerrainType) -> bool {
        if local_x < CHUNK_SIZE && local_y < CHUNK_SIZE {
            self.tiles[local_y][local_x] = tile_type;
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn world_to_local(world_x: i32, world_y: i32) -> (usize, usize) {
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;
        (local_x, local_y)
    }

    pub fn get_tile_at_world_position(&self, world_x: i32, world_y: i32) -> Option<TerrainType> {
        let chunk_world_x = self.coordinate.x * CHUNK_SIZE as i32;
        let chunk_world_y = self.coordinate.y * CHUNK_SIZE as i32;

        let local_x = world_x - chunk_world_x;
        let local_y = world_y - chunk_world_y;

        if (0..CHUNK_SIZE as i32).contains(&local_x) && (0..CHUNK_SIZE as i32).contains(&local_y) {
            Some(self.tiles[local_y as usize][local_x as usize])
        } else {
            None
        }
    }

    pub fn get_terrain_summary(&self) -> std::collections::HashMap<TerrainType, usize> {
        let mut summary = std::collections::HashMap::new();

        for row in &self.tiles {
            for terrain in row {
                *summary.entry(*terrain).or_insert(0) += 1;
            }
        }

        summary
    }

    pub fn is_walkable_percentage(&self) -> f32 {
        let mut walkable_count = 0;
        let total_tiles = CHUNK_SIZE * CHUNK_SIZE;

        for row in &self.tiles {
            for terrain in row {
                if terrain.is_walkable() {
                    walkable_count += 1;
                }
            }
        }

        walkable_count as f32 / total_tiles as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coordinate_conversion() {
        let coord = ChunkCoordinate::from_tile_position(17, -5);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, -1);

        let coord = ChunkCoordinate::from_tile_position(15, 15);
        assert_eq!(coord.x, 0);
        assert_eq!(coord.y, 0);

        let coord = ChunkCoordinate::from_tile_position(-1, -1);
        assert_eq!(coord.x, -1);
        assert_eq!(coord.y, -1);
    }

    #[test]
    fn test_world_to_local() {
        let (x, y) = Chunk::world_to_local(17, 5);
        assert_eq!(x, 1);
        assert_eq!(y, 5);

        let (x, y) = Chunk::world_to_local(-1, -1);
        assert_eq!(x, 15);
        assert_eq!(y, 15);
    }

    #[test]
    fn test_chunk_distance() {
        let coord1 = ChunkCoordinate::new(0, 0);
        let coord2 = ChunkCoordinate::new(3, 4);
        assert_eq!(coord1.distance_to(&coord2), 4);

        let coord3 = ChunkCoordinate::new(-2, -2);
        assert_eq!(coord1.distance_to(&coord3), 2);
    }

    #[test]
    fn test_chunk_creation() {
        let coord = ChunkCoordinate::new(0, 0);
        let chunk = Chunk::new(coord, 12345);

        assert_eq!(chunk.coordinate, coord);
        assert_eq!(chunk.tiles.len(), CHUNK_SIZE);
        assert_eq!(chunk.tiles[0].len(), CHUNK_SIZE);
        assert!(chunk.is_dirty);
    }
}
