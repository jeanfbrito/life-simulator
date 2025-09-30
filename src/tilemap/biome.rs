use super::terrain::TerrainType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Plains,
    Forest,
    Desert,
    Tundra,
    Swamp,
    Mountain,
    Ocean,
    Taiga,
    Savanna,
    Rainforest,
}

impl BiomeType {
    pub fn from_climate(temperature: f32, moisture: f32, elevation: f32) -> Self {
        if elevation < 0.1 {
            return BiomeType::Ocean;
        }

        if elevation > 0.7 {
            return BiomeType::Mountain;
        }

        match (temperature, moisture) {
            (t, m) if t < 0.2 && m < 0.3 => BiomeType::Tundra,
            (t, m) if t < 0.3 && m > 0.5 => BiomeType::Taiga,
            (t, m) if t > 0.7 && m < 0.3 => BiomeType::Desert,
            (t, m) if t > 0.6 && m < 0.5 => BiomeType::Savanna,
            (t, m) if t > 0.6 && m > 0.7 => BiomeType::Rainforest,
            (t, m) if t > 0.4 && m > 0.6 => BiomeType::Swamp,
            (_t, m) if m > 0.5 => BiomeType::Forest,
            _ => BiomeType::Plains,
        }
    }

    pub fn get_dominant_terrain(&self) -> Vec<(TerrainType, f32)> {
        match self {
            BiomeType::Plains => vec![
                (TerrainType::Grass, 0.7),
                (TerrainType::Dirt, 0.2),
                (TerrainType::Stone, 0.1),
            ],
            BiomeType::Forest => vec![
                (TerrainType::Forest, 0.6),
                (TerrainType::Grass, 0.3),
                (TerrainType::Dirt, 0.1),
            ],
            BiomeType::Desert => vec![
                (TerrainType::Desert, 0.6),
                (TerrainType::Sand, 0.3),
                (TerrainType::Stone, 0.1),
            ],
            BiomeType::Tundra => vec![
                (TerrainType::Snow, 0.5),
                (TerrainType::Stone, 0.3),
                (TerrainType::Dirt, 0.2),
            ],
            BiomeType::Swamp => vec![
                (TerrainType::Swamp, 0.5),
                (TerrainType::ShallowWater, 0.3),
                (TerrainType::Grass, 0.2),
            ],
            BiomeType::Mountain => vec![
                (TerrainType::Mountain, 0.5),
                (TerrainType::Stone, 0.4),
                (TerrainType::Snow, 0.1),
            ],
            BiomeType::Ocean => vec![
                (TerrainType::DeepWater, 0.7),
                (TerrainType::Water, 0.2),
                (TerrainType::ShallowWater, 0.1),
            ],
            BiomeType::Taiga => vec![
                (TerrainType::Snow, 0.3),
                (TerrainType::Forest, 0.5),
                (TerrainType::Dirt, 0.2),
            ],
            BiomeType::Savanna => vec![
                (TerrainType::Grass, 0.5),
                (TerrainType::Sand, 0.3),
                (TerrainType::Dirt, 0.2),
            ],
            BiomeType::Rainforest => vec![
                (TerrainType::Forest, 0.7),
                (TerrainType::Swamp, 0.2),
                (TerrainType::Grass, 0.1),
            ],
        }
    }

    pub fn select_terrain(&self, random_value: f32) -> TerrainType {
        let terrains = self.get_dominant_terrain();
        let mut cumulative = 0.0;

        for (terrain, probability) in terrains {
            cumulative += probability;
            if random_value <= cumulative {
                return terrain;
            }
        }

        // Fallback
        TerrainType::Grass
    }

    pub fn get_resource_potential(&self) -> Vec<&'static str> {
        match self {
            BiomeType::Forest => vec!["wood", "herbs", "berries", "wildlife"],
            BiomeType::Plains => vec!["grain", "herbs", "wildlife"],
            BiomeType::Desert => vec!["sand", "rare_minerals", "cacti"],
            BiomeType::Tundra => vec!["ice", "cold_resistant_plants", "fur"],
            BiomeType::Swamp => vec!["herbs", "clay", "reeds"],
            BiomeType::Mountain => vec!["stone", "ore", "minerals", "mountain_herbs"],
            BiomeType::Ocean => vec!["fish", "seaweed", "pearls"],
            BiomeType::Taiga => vec!["wood", "pine_cones", "cold_resistant_herbs"],
            BiomeType::Savanna => vec!["grain", "wildlife", "acacia"],
            BiomeType::Rainforest => vec!["exotic_wood", "rare_herbs", "exotic_fruits", "wildlife"],
        }
    }
}

pub struct BiomeGenerator {
    seed: u64,
}

impl BiomeGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn generate_biome(&self, chunk_x: i32, chunk_y: i32) -> BiomeType {
        // Large-scale biome generation based on chunk coordinates
        let scale = 0.01;
        let x = chunk_x as f32 * scale;
        let y = chunk_y as f32 * scale;

        // Generate climate values using simple noise functions
        let temperature = self.noise_2d(x, y, 1.0);
        let moisture = self.noise_2d(x * 1.3, y * 1.3, 2.0);
        let elevation = self.noise_2d(x * 0.7, y * 0.7, 3.0);

        BiomeType::from_climate(temperature, moisture, elevation)
    }

    fn noise_2d(&self, x: f32, y: f32, offset: f32) -> f32 {
        // Simple pseudo-noise function
        let seed_factor = (self.seed as f32 * 0.001) + offset;
        let value = ((x + seed_factor).sin() * (y + seed_factor).cos() + 1.0) / 2.0;
        value.clamp(0.0, 1.0)
    }
}