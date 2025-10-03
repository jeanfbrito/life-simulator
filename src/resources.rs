use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::collections::HashMap;

/// Resource types that can be placed on the map
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    TreeOak,
    TreePine,
    TreeBirch,
    Rock,
    Bush,
    Flower,
}

impl ResourceType {
    /// Get the string representation for storage
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::TreeOak => "TreeOak",
            ResourceType::TreePine => "TreePine",
            ResourceType::TreeBirch => "TreeBirch",
            ResourceType::Rock => "Rock",
            ResourceType::Bush => "Bush",
            ResourceType::Flower => "Flower",
        }
    }

    /// Create resource from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "TreeOak" => Some(ResourceType::TreeOak),
            "TreePine" => Some(ResourceType::TreePine),
            "TreeBirch" => Some(ResourceType::TreeBirch),
            "Rock" => Some(ResourceType::Rock),
            "Bush" => Some(ResourceType::Bush),
            "Flower" => Some(ResourceType::Flower),
            _ => None,
        }
    }
}

/// Resource generation configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub tree_density: f32,
    pub rock_density: f32,
    pub bush_density: f32,
    pub flower_density: f32,
    pub enable_resources: bool,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            tree_density: 0.08,   // 8% chance of tree per tile
            rock_density: 0.03,   // 3% chance of rock per tile
            bush_density: 0.05,   // 5% chance of bush per tile
            flower_density: 0.04, // 4% chance of flower per tile
            enable_resources: true,
        }
    }
}

/// Resource generator for placing resources on terrain
pub struct ResourceGenerator {
    config: ResourceConfig,
}

impl ResourceGenerator {
    pub fn new(config: ResourceConfig) -> Self {
        Self { config }
    }

    /// Generate resource layer for a chunk based on terrain
    pub fn generate_resource_layer(
        &self,
        terrain_layer: &[Vec<String>],
        chunk_x: i32,
        chunk_y: i32,
        world_seed: u64,
    ) -> Vec<Vec<String>> {
        if !self.config.enable_resources {
            return vec![vec!["".to_string(); 16]; 16];
        }

        let mut resource_layer = vec![vec!["".to_string(); 16]; 16];
        let seed = world_seed
            .wrapping_mul(1000)
            .wrapping_add(chunk_x as u64)
            .wrapping_mul(100)
            .wrapping_add(chunk_y as u64)
            .wrapping_add(9999);
        let mut rng = Pcg64::seed_from_u64(seed);

        for y in 0..16 {
            for x in 0..16 {
                let terrain = &terrain_layer[y][x];

                // Only place resources on suitable terrain
                if let Some(resource_type) = self.determine_resource_for_terrain(terrain, &mut rng)
                {
                    resource_layer[y][x] = resource_type.as_str().to_string();
                }
            }
        }

        resource_layer
    }

    /// Determine what resource (if any) should be placed on given terrain
    fn determine_resource_for_terrain(
        &self,
        terrain: &str,
        rng: &mut Pcg64,
    ) -> Option<ResourceType> {
        match terrain {
            "Grass" => {
                let roll = rng.gen::<f32>();
                if roll < self.config.tree_density {
                    // Choose tree type
                    let tree_roll = rng.gen::<f32>();
                    if tree_roll < 0.4 {
                        Some(ResourceType::TreeOak)
                    } else if tree_roll < 0.7 {
                        Some(ResourceType::TreePine)
                    } else {
                        Some(ResourceType::TreeBirch)
                    }
                } else if roll < self.config.tree_density + self.config.bush_density {
                    Some(ResourceType::Bush)
                } else if roll
                    < self.config.tree_density
                        + self.config.bush_density
                        + self.config.flower_density
                {
                    Some(ResourceType::Flower)
                } else {
                    None
                }
            }
            "Forest" => {
                // Forest terrain has higher tree density
                if rng.gen::<f32>() < self.config.tree_density * 2.0 {
                    let tree_roll = rng.gen::<f32>();
                    if tree_roll < 0.5 {
                        Some(ResourceType::TreeOak)
                    } else if tree_roll < 0.8 {
                        Some(ResourceType::TreePine)
                    } else {
                        Some(ResourceType::TreeBirch)
                    }
                } else {
                    None
                }
            }
            "Mountain" | "Stone" => {
                // Mountain/stone terrain can have rocks
                if rng.gen::<f32>() < self.config.rock_density * 3.0 {
                    Some(ResourceType::Rock)
                } else {
                    None
                }
            }
            "Dirt" => {
                // Dirt can have occasional bushes or rocks
                let roll = rng.gen::<f32>();
                if roll < self.config.bush_density * 0.5 {
                    Some(ResourceType::Bush)
                } else if roll < self.config.bush_density * 0.5 + self.config.rock_density * 0.5 {
                    Some(ResourceType::Rock)
                } else {
                    None
                }
            }
            "Sand" => {
                // Beach sand rarely has anything, maybe occasional bushes
                if rng.gen::<f32>() < self.config.bush_density * 0.1 {
                    Some(ResourceType::Bush)
                } else {
                    None
                }
            }
            _ => {
                // Water terrain and others have no resources
                None
            }
        }
    }

    /// Create a resource layer from existing terrain and chunk coordinates
    pub fn create_resources_for_chunk(
        terrain_layer: &[Vec<String>],
        chunk_x: i32,
        chunk_y: i32,
        world_seed: u64,
    ) -> Vec<Vec<String>> {
        let config = ResourceConfig::default();
        let generator = ResourceGenerator::new(config);
        generator.generate_resource_layer(terrain_layer, chunk_x, chunk_y, world_seed)
    }
}

/// Utility functions for resource layer management
pub struct ResourceUtils;

impl ResourceUtils {
    /// Create an empty resource layer
    pub fn empty_resource_layer() -> Vec<Vec<String>> {
        vec![vec!["".to_string(); 16]; 16]
    }

    /// Check if a tile has a resource
    pub fn has_resource(resource_layer: &[Vec<String>], x: usize, y: usize) -> bool {
        if y < resource_layer.len() && x < resource_layer[y].len() {
            !resource_layer[y][x].is_empty()
        } else {
            false
        }
    }

    /// Get resource type at specific position
    pub fn get_resource_at(
        resource_layer: &[Vec<String>],
        x: usize,
        y: usize,
    ) -> Option<ResourceType> {
        if y < resource_layer.len() && x < resource_layer[y].len() {
            ResourceType::from_str(&resource_layer[y][x])
        } else {
            None
        }
    }

    /// Count resources in a layer by type
    pub fn count_resources_by_type(resource_layer: &[Vec<String>]) -> HashMap<ResourceType, usize> {
        let mut counts = HashMap::new();

        for row in resource_layer {
            for tile in row {
                if let Some(resource_type) = ResourceType::from_str(tile) {
                    *counts.entry(resource_type).or_insert(0) += 1;
                }
            }
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_conversion() {
        assert_eq!(
            ResourceType::from_str("TreeOak"),
            Some(ResourceType::TreeOak)
        );
        assert_eq!(ResourceType::TreeOak.as_str(), "TreeOak");
        assert_eq!(ResourceType::from_str("Invalid"), None);
    }

    #[test]
    fn test_empty_resource_layer() {
        let layer = ResourceUtils::empty_resource_layer();
        assert_eq!(layer.len(), 16);
        assert_eq!(layer[0].len(), 16);
        assert!(!ResourceUtils::has_resource(&layer, 0, 0));
    }

    #[test]
    fn test_resource_generation() {
        let terrain = vec![vec!["Grass".to_string(); 16]; 16];

        let resources = ResourceGenerator::create_resources_for_chunk(&terrain, 0, 0, 12345);
        assert_eq!(resources.len(), 16);
        assert_eq!(resources[0].len(), 16);

        // Should have some resources in grass terrain
        let has_any_resource = resources
            .iter()
            .any(|row| row.iter().any(|tile| !tile.is_empty()));
        assert!(has_any_resource);
    }

    #[test]
    fn test_resource_counting() {
        let mut resources = ResourceUtils::empty_resource_layer();
        resources[0][0] = "TreeOak".to_string();
        resources[0][1] = "TreeOak".to_string();
        resources[1][0] = "Rock".to_string();

        let counts = ResourceUtils::count_resources_by_type(&resources);
        assert_eq!(counts.get(&ResourceType::TreeOak), Some(&2));
        assert_eq!(counts.get(&ResourceType::Rock), Some(&1));
    }
}
