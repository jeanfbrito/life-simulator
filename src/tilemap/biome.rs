use super::terrain::TerrainType;
use serde::{Deserialize, Serialize};
use noise::{NoiseFn, Perlin, Simplex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    DeepWater,
    ShallowWater,
    RiparianZone,
    TemperateForest,
    Woodland,
    Grassland,
    ForestEdge,
    RockyOutcrop,
}

impl BiomeType {
    pub fn from_climate(temperature: f32, moisture: f32, elevation: f32) -> Self {
        // Rocky outcrop - highest elevation
        if elevation > 0.8 {
            return BiomeType::RockyOutcrop;
        }

        // Deep water - lowest elevation
        if elevation <= 0.05 {
            return BiomeType::DeepWater;
        }

        // Shallow water - low elevation but not deep water
        if elevation <= 0.1 {
            return BiomeType::ShallowWater;
        }

        // Riparian zone - very high moisture near water level
        if elevation < 0.2 && moisture > 0.7 {
            return BiomeType::RiparianZone;
        }

        // Temperate biome classification based on moisture and temperature
        match (temperature, moisture) {
            // High moisture, moderate temperature -> Temperate Forest
            (t, m) if t >= 0.5 && t <= 0.7 && m > 0.7 => BiomeType::TemperateForest,

            // Edge conditions - transition zones (check before general woodland/grassland)
            (t, m) if t >= 0.3 && t <= 0.8 && m >= 0.4 && m <= 0.6 => BiomeType::ForestEdge,

            // Moderate moisture and temperature -> Woodland
            (t, m) if t >= 0.4 && t <= 0.8 && m >= 0.5 && m <= 0.7 => BiomeType::Woodland,

            // Lower moisture -> Grassland
            (t, m) if t >= 0.4 && t <= 0.8 && m < 0.5 => BiomeType::Grassland,

            // Fallback to woodland for temperate climates
            (t, _m) if t >= 0.3 && t <= 0.8 => BiomeType::Woodland,

            // Extreme conditions but not water or rock
            _ => BiomeType::Grassland,
        }
    }

    pub fn get_dominant_terrain(&self) -> Vec<(TerrainType, f32)> {
        match self {
            BiomeType::DeepWater => vec![
                (TerrainType::DeepWater, 0.8),
                (TerrainType::Water, 0.2),
            ],
            BiomeType::ShallowWater => vec![
                (TerrainType::ShallowWater, 0.6),
                (TerrainType::Water, 0.3),
                (TerrainType::Grass, 0.1),
            ],
            BiomeType::RiparianZone => vec![
                (TerrainType::Grass, 0.4),
                (TerrainType::ShallowWater, 0.3),
                (TerrainType::Forest, 0.2),
                (TerrainType::Dirt, 0.1),
            ],
            BiomeType::TemperateForest => vec![
                (TerrainType::Forest, 0.7),
                (TerrainType::Grass, 0.2),
                (TerrainType::Dirt, 0.1),
            ],
            BiomeType::Woodland => vec![
                (TerrainType::Forest, 0.4),
                (TerrainType::Grass, 0.4),
                (TerrainType::Dirt, 0.2),
            ],
            BiomeType::Grassland => vec![
                (TerrainType::Grass, 0.8),
                (TerrainType::Dirt, 0.15),
                (TerrainType::Stone, 0.05),
            ],
            BiomeType::ForestEdge => vec![
                (TerrainType::Grass, 0.4),
                (TerrainType::Forest, 0.3),
                (TerrainType::Dirt, 0.2),
                (TerrainType::Stone, 0.1),
            ],
            BiomeType::RockyOutcrop => vec![
                (TerrainType::Stone, 0.6),
                (TerrainType::Grass, 0.2),
                (TerrainType::Dirt, 0.1),
                (TerrainType::Mountain, 0.1),
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
            BiomeType::DeepWater => vec!["fish", "seaweed", "pearls"],
            BiomeType::ShallowWater => vec!["fish", "reeds", "clay"],
            BiomeType::RiparianZone => vec!["herbs", "clay", "reeds", "fish"],
            BiomeType::TemperateForest => vec!["wood", "herbs", "berries", "wildlife"],
            BiomeType::Woodland => vec!["wood", "herbs", "wildlife"],
            BiomeType::Grassland => vec!["grain", "herbs", "wildlife"],
            BiomeType::ForestEdge => vec!["herbs", "berries", "wood"],
            BiomeType::RockyOutcrop => vec!["stone", "ore", "minerals"],
        }
    }
}

pub struct BiomeGenerator {
    seed: u64,
    moisture_noise: Simplex,
    temperature_noise: Perlin,
    elevation_noise: Simplex,
}

impl BiomeGenerator {
    pub fn new(seed: u64) -> Self {
        let moisture_seed = seed;
        let temperature_seed = seed.wrapping_add(1000);
        let elevation_seed = seed.wrapping_add(2000);

        // Convert u64 to u32 by XORing upper and lower bits to preserve entropy
        let to_u32 = |s: u64| ((s >> 32) as u32) ^ (s as u32);

        Self {
            seed,
            moisture_noise: Simplex::new(to_u32(moisture_seed)),
            temperature_noise: Perlin::new(to_u32(temperature_seed)),
            elevation_noise: Simplex::new(to_u32(elevation_seed)),
        }
    }

    pub fn generate_biome(&self, chunk_x: i32, chunk_y: i32) -> BiomeType {
        // Generate climate values using proper noise functions
        let temperature = self.get_temperature(chunk_x, chunk_y);
        let moisture = self.get_moisture(chunk_x, chunk_y);
        let elevation = self.get_elevation(chunk_x, chunk_y);

        BiomeType::from_climate(temperature, moisture, elevation)
    }

    pub fn get_moisture(&self, x: i32, y: i32) -> f32 {
        // Multi-octave moisture generation
        let scale = 0.02;
        let nx = x as f64 * scale;
        let ny = y as f64 * scale;

        // Primary moisture layer (large scale)
        let primary = self.moisture_noise.get([nx, ny]) as f32;

        // Secondary detail layer (smaller scale)
        let detail_scale = 0.08;
        let detail_x = x as f64 * detail_scale;
        let detail_y = y as f64 * detail_scale;
        let detail = self.moisture_noise.get([detail_x, detail_y]) as f32 * 0.3;

        // Combine and normalize to 0..1
        let combined = primary * 0.7 + detail;
        ((combined + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    pub fn get_temperature(&self, x: i32, y: i32) -> f32 {
        // Multi-octave temperature generation with latitude influence
        let scale = 0.015;
        let nx = x as f64 * scale;
        let ny = y as f64 * scale;

        // Primary temperature layer
        let primary = self.temperature_noise.get([nx, ny]) as f32;

        // Add latitude gradient (colder in north/south)
        let latitude_factor = (y as f32 / 100.0).abs().min(1.0) * 0.3;
        let latitude_adjustment = 1.0 - latitude_factor;

        // Combine and normalize
        let combined = primary * 0.6 + latitude_adjustment * 0.4;
        combined.clamp(0.0, 1.0)
    }

    pub fn get_elevation(&self, x: i32, y: i32) -> f32 {
        // Multi-octave elevation generation
        let scale = 0.025;
        let nx = x as f64 * scale;
        let ny = y as f64 * scale;

        // Primary elevation layer (large scale features)
        let primary = self.elevation_noise.get([nx, ny]) as f32;

        // Ridge noise for mountains
        let ridge_scale = 0.04;
        let ridge_x = x as f64 * ridge_scale;
        let ridge_y = y as f64 * ridge_scale;
        let ridge = (self.elevation_noise.get([ridge_x, ridge_y]).abs() as f32) * 0.5;

        // Combine and normalize with bias toward mid-elevations
        let combined = primary * 0.6 + ridge * 0.4;
        let normalized = ((combined + 1.0) * 0.5).powf(1.2); // Bias toward mid-values
        normalized.clamp(0.0, 1.0)
    }

    // Helper method to get biome for specific world coordinates (useful for simulation systems)
    pub fn get_biome_at(&self, world_x: i32, world_y: i32) -> BiomeType {
        self.generate_biome(world_x, world_y)
    }

    // Generate weighted biome blends for transition tiles
    pub fn get_biome_blend(&self, x: i32, y: i32) -> Vec<(BiomeType, f32)> {
        let center_biome = self.generate_biome(x, y);
        let mut blends = vec![(center_biome, 0.5)];

        // Sample neighboring biomes for smooth transitions
        let neighbors = [
            (x - 1, y), (x + 1, y),
            (x, y - 1), (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            let neighbor_biome = self.generate_biome(nx, ny);
            if neighbor_biome != center_biome {
                blends.push((neighbor_biome, 0.125));
            }
        }

        // Normalize weights
        let total_weight: f32 = blends.iter().map(|(_, w)| w).sum();
        if total_weight > 0.0 {
            blends.iter_mut().for_each(|(_, w)| *w /= total_weight);
        }

        blends
    }
}

// Include the test module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tilemap::biome::{BiomeType, BiomeGenerator};

    #[test]
    fn test_temperate_biome_type_from_climate() {
        // Test deep water - lowest elevation
        assert_eq!(BiomeType::from_climate(0.5, 0.5, 0.05), BiomeType::DeepWater);

        // Test shallow water
        assert_eq!(BiomeType::from_climate(0.5, 0.5, 0.08), BiomeType::ShallowWater);

        // Test riparian zone - near water with high moisture
        assert_eq!(BiomeType::from_climate(0.6, 0.8, 0.15), BiomeType::RiparianZone);

        // Test temperate forest - moderate temperature, high moisture, mid elevation
        assert_eq!(BiomeType::from_climate(0.6, 0.75, 0.3), BiomeType::TemperateForest);

        // Test woodland - moderate temperature and moisture
        assert_eq!(BiomeType::from_climate(0.6, 0.65, 0.25), BiomeType::Woodland);

        // Test grassland - moderate temperature, low moisture
        assert_eq!(BiomeType::from_climate(0.6, 0.3, 0.2), BiomeType::Grassland);

        // Test forest edge - transition areas
        assert_eq!(BiomeType::from_climate(0.65, 0.45, 0.25), BiomeType::ForestEdge);

        // Test rocky outcrop - high elevation, any moisture
        assert_eq!(BiomeType::from_climate(0.4, 0.5, 0.85), BiomeType::RockyOutcrop);
        assert_eq!(BiomeType::from_climate(0.7, 0.2, 0.85), BiomeType::RockyOutcrop);
    }

    #[test]
    fn test_biome_type_serialization() {
        // Test all biome types can be serialized/deserialized
        let biomes = vec![
            BiomeType::DeepWater,
            BiomeType::ShallowWater,
            BiomeType::RiparianZone,
            BiomeType::TemperateForest,
            BiomeType::Woodland,
            BiomeType::Grassland,
            BiomeType::ForestEdge,
            BiomeType::RockyOutcrop,
        ];

        for biome in biomes {
            let serialized = serde_json::to_string(&biome).unwrap();
            let deserialized: BiomeType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(biome, deserialized);
        }
    }

    #[test]
    fn test_biome_generator_deterministic() {
        let seed = 12345;
        let generator1 = BiomeGenerator::new(seed);
        let generator2 = BiomeGenerator::new(seed);

        // Same seed should produce same biomes
        for x in 0..10 {
            for y in 0..10 {
                let biome1 = generator1.generate_biome(x, y);
                let biome2 = generator2.generate_biome(x, y);
                assert_eq!(biome1, biome2, "Biomes differ at ({}, {})", x, y);
            }
        }
    }

    #[test]
    fn test_biome_generator_different_seeds() {
        let generator1 = BiomeGenerator::new(12345);
        let generator2 = BiomeGenerator::new(54321);

        let mut differences = 0;
        for x in 0..10 {
            for y in 0..10 {
                let biome1 = generator1.generate_biome(x, y);
                let biome2 = generator2.generate_biome(x, y);
                if biome1 != biome2 {
                    differences += 1;
                }
            }
        }

        // Should have significant differences with different seeds
        assert!(differences > 50, "Different seeds should produce different biomes");
    }

    #[test]
    fn test_biome_dominant_terrain() {
        let forest_terrain = BiomeType::TemperateForest.get_dominant_terrain();
        assert!(!forest_terrain.is_empty());

        // Check probabilities sum to approximately 1.0
        let total_probability: f32 = forest_terrain.iter().map(|(_, prob)| prob).sum();
        assert!((total_probability - 1.0).abs() < 0.01);

        // Forest should have forest terrain as dominant
        let has_forest = forest_terrain.iter().any(|(terrain, _)| {
            matches!(terrain, super::super::terrain::TerrainType::Forest)
        });
        assert!(has_forest);
    }

    #[test]
    fn test_biome_resource_potential() {
        let forest_resources = BiomeType::TemperateForest.get_resource_potential();
        assert!(!forest_resources.is_empty());

        // Forest should have wood resources
        assert!(forest_resources.contains(&"wood"));

        let water_resources = BiomeType::DeepWater.get_resource_potential();
        assert!(water_resources.contains(&"fish"));
    }

    #[test]
    fn test_terrain_selection() {
        let biome = BiomeType::TemperateForest;

        // Test multiple random values produce consistent results
        let mut terrain_counts = std::collections::HashMap::new();

        for i in 0..1000 {
            let random_value = i as f32 / 1000.0;
            let terrain = biome.select_terrain(random_value);
            *terrain_counts.entry(terrain).or_insert(0) += 1;
        }

        // Should produce variety but biased toward dominant terrain
        assert!(terrain_counts.len() > 1);
    }

    #[test]
    fn test_moisture_temperature_noise_generation() {
        let generator = BiomeGenerator::new(42);

        // Test noise values are in valid range
        for x in 0..100 {
            for y in 0..100 {
                let moisture = generator.get_moisture(x, y);
                let temperature = generator.get_temperature(x, y);

                assert!(moisture >= 0.0 && moisture <= 1.0, "Moisture out of range: {}", moisture);
                assert!(temperature >= 0.0 && temperature <= 1.0, "Temperature out of range: {}", temperature);
            }
        }
    }

    #[test]
    fn test_biome_climate_integration() {
        // Use a larger seed to benefit from entropy-preserving XOR conversion
        let generator = BiomeGenerator::new(9876543210);

        // Generate a biome map with sparser sampling to get variety
        // (noise scales are 0.015-0.025, so we need larger coordinate ranges)
        let mut biome_set = std::collections::HashSet::new();

        for x in (0..200).step_by(10) {
            for y in (0..200).step_by(10) {
                let biome = generator.generate_biome(x, y);
                biome_set.insert(biome);
            }
        }

        // Should generate variety of biomes
        assert!(biome_set.len() > 3, "Should generate multiple biome types");

        // All biomes should be from our temperate set
        for biome in biome_set {
            // All our temperate biomes are valid
            assert!(matches!(biome,
                BiomeType::DeepWater |
                BiomeType::ShallowWater |
                BiomeType::RiparianZone |
                BiomeType::TemperateForest |
                BiomeType::Woodland |
                BiomeType::Grassland |
                BiomeType::ForestEdge |
                BiomeType::RockyOutcrop
            ), "Invalid biome type generated: {:?}", biome);
        }
    }
}
