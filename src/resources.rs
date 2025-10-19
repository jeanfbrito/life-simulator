use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::collections::HashMap;
use crate::tilemap::biome::BiomeType;

/// Resource categories for behavior differentiation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceCategory {
    Tree,
    Shrub,
    Collectable,
    Rock,
    Flower,
    Bush,
}

/// Consumption kinds for different entity interactions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConsumptionKind {
    HerbivoreBrowse,  // Can be consumed by herbivores (shrubs)
    HumanGather,      // Can be gathered by players (collectables)
    Inedible,         // Cannot be consumed (rocks, some flowers)
}

/// Harvest profile for resource behavior
#[derive(Debug, Clone)]
pub struct HarvestProfile {
    pub category: ResourceCategory,
    pub biomass_cap: f32,
    pub growth_rate_multiplier: f32,
    pub harvest_yield: u32,
    pub regrowth_delay_ticks: u64,
    pub consumption_kind: ConsumptionKind,
    pub nutritional_value: f32,
}

impl HarvestProfile {
    pub fn new(
        category: ResourceCategory,
        biomass_cap: f32,
        growth_rate_multiplier: f32,
        harvest_yield: u32,
        regrowth_delay_ticks: u64,
        consumption_kind: ConsumptionKind,
        nutritional_value: f32,
    ) -> Self {
        Self {
            category,
            biomass_cap,
            growth_rate_multiplier,
            harvest_yield,
            regrowth_delay_ticks,
            consumption_kind,
            nutritional_value,
        }
    }
}

/// Resource definitions metadata map
lazy_static::lazy_static! {
    pub static ref RESOURCE_DEFINITIONS: HashMap<ResourceType, HarvestProfile> = {
        let mut defs = HashMap::new();

        // Trees
        defs.insert(ResourceType::TreeOak, HarvestProfile::new(
            ResourceCategory::Tree,
            100.0,      // High biomass cap
            0.8,        // Slow growth
            5,          // 5 wood units
            1000,       // Long regrowth (1000 ticks)
            ConsumptionKind::Inedible,
            0.0,        // No nutritional value
        ));

        defs.insert(ResourceType::TreePine, HarvestProfile::new(
            ResourceCategory::Tree,
            90.0,
            0.7,
            4,
            1200,
            ConsumptionKind::Inedible,
            0.0,
        ));

        defs.insert(ResourceType::TreeBirch, HarvestProfile::new(
            ResourceCategory::Tree,
            70.0,
            1.0,        // Faster growth
            3,
            800,
            ConsumptionKind::Inedible,
            0.0,
        ));

        // Shrubs - balanced nutritional values for gameplay
        defs.insert(ResourceType::BerryBush, HarvestProfile::new(
            ResourceCategory::Shrub,
            25.0,       // Medium biomass
            1.2,        // Fast growth
            3,          // 3 berry units
            500,        // Medium regrowth (500 ticks)
            ConsumptionKind::HerbivoreBrowse,
            12.0,       // Good nutritional value (reduced from 15.0)
        ));

        defs.insert(ResourceType::HazelShrub, HarvestProfile::new(
            ResourceCategory::Shrub,
            30.0,
            1.0,
            2,          // 2 nut units
            600,
            ConsumptionKind::HerbivoreBrowse,
            18.0,       // High nutritional value (reduced from 20.0)
        ));

        // Collectables - balanced for special discovery feeling
        defs.insert(ResourceType::MushroomPatch, HarvestProfile::new(
            ResourceCategory::Collectable,
            8.0,        // Low biomass
            1.5,        // Very fast growth
            2,          // 2 mushroom units
            300,        // Quick regrowth (300 ticks)
            ConsumptionKind::HumanGather,
            6.0,        // Moderate nutritional value (reduced from 8.0)
        ));

        defs.insert(ResourceType::WildRoot, HarvestProfile::new(
            ResourceCategory::Collectable,
            6.0,
            0.9,
            1,          // 1 root unit
            400,
            ConsumptionKind::HumanGather,
            10.0,       // Good nutritional value (reduced from 12.0)
        ));

        // Existing resources
        defs.insert(ResourceType::Rock, HarvestProfile::new(
            ResourceCategory::Rock,
            150.0,      // Very high biomass ( durability )
            0.0,        // No growth
            0,          // No harvest yield
            u64::MAX,   // Never regrows
            ConsumptionKind::Inedible,
            0.0,
        ));

        defs.insert(ResourceType::Bush, HarvestProfile::new(
            ResourceCategory::Bush,
            20.0,
            1.1,
            1,
            400,
            ConsumptionKind::HerbivoreBrowse,
            10.0,
        ));

        defs.insert(ResourceType::Flower, HarvestProfile::new(
            ResourceCategory::Flower,
            3.0,        // Very low biomass
            1.8,        // Very fast growth
            0,          // No harvest yield
            200,        // Quick regrowth
            ConsumptionKind::Inedible,
            0.0,
        ));

        defs
    };
}

impl ResourceType {
    /// Get the harvest profile for this resource type
    pub fn get_profile(&self) -> Option<&'static HarvestProfile> {
        RESOURCE_DEFINITIONS.get(self)
    }

    /// Get the category of this resource type
    pub fn get_category(&self) -> Option<ResourceCategory> {
        self.get_profile().map(|p| p.category.clone())
    }

    /// Check if this resource can be consumed by herbivores
    pub fn is_herbivore_edible(&self) -> bool {
        matches!(self.get_profile().map(|p| &p.consumption_kind),
                 Some(ConsumptionKind::HerbivoreBrowse))
    }

    /// Check if this resource can be gathered by players
    pub fn is_gatherable(&self) -> bool {
        matches!(self.get_profile().map(|p| &p.consumption_kind),
                 Some(ConsumptionKind::HumanGather))
    }

    /// Get the consumption kind for this resource
    pub fn get_consumption_kind(&self) -> Option<ConsumptionKind> {
        self.get_profile().map(|p| p.consumption_kind.clone())
    }

    /// Get the harvest profile for this resource (alias for get_profile)
    pub fn get_harvest_profile(&self) -> Option<&'static HarvestProfile> {
        self.get_profile()
    }
}

/// Resource types that can be placed on the map
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    // Trees
    TreeOak,
    TreePine,
    TreeBirch,
    // Shrubs
    BerryBush,
    HazelShrub,
    // Collectables
    MushroomPatch,
    WildRoot,
    // Existing resources
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
            ResourceType::BerryBush => "BerryBush",
            ResourceType::HazelShrub => "HazelShrub",
            ResourceType::MushroomPatch => "MushroomPatch",
            ResourceType::WildRoot => "WildRoot",
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
            "BerryBush" => Some(ResourceType::BerryBush),
            "HazelShrub" => Some(ResourceType::HazelShrub),
            "MushroomPatch" => Some(ResourceType::MushroomPatch),
            "WildRoot" => Some(ResourceType::WildRoot),
            "Rock" => Some(ResourceType::Rock),
            "Bush" => Some(ResourceType::Bush),
            "Flower" => Some(ResourceType::Flower),
            _ => None,
        }
    }
}

/// Biome-specific resource density multipliers
#[derive(Debug, Clone)]
pub struct BiomeResourceMultipliers {
    pub tree_multiplier: f32,
    pub shrub_multiplier: f32,
    pub collectable_multiplier: f32,
    pub flower_multiplier: f32,
    pub rock_multiplier: f32,
}

impl Default for BiomeResourceMultipliers {
    fn default() -> Self {
        Self {
            tree_multiplier: 1.0,
            shrub_multiplier: 1.0,
            collectable_multiplier: 1.0,
            flower_multiplier: 1.0,
            rock_multiplier: 1.0,
        }
    }
}

impl BiomeResourceMultipliers {
    pub fn for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::DeepWater => Self {
                tree_multiplier: 0.0,
                shrub_multiplier: 0.0,
                collectable_multiplier: 0.0,
                flower_multiplier: 0.0,
                rock_multiplier: 0.5,
            },
            BiomeType::ShallowWater => Self {
                tree_multiplier: 0.0,
                shrub_multiplier: 0.0,
                collectable_multiplier: 0.1, // Some aquatic resources
                flower_multiplier: 0.0,
                rock_multiplier: 0.3,
            },
            BiomeType::RiparianZone => Self {
                tree_multiplier: 1.5,   // Riverside trees
                shrub_multiplier: 1.8,  // Dense riverside vegetation
                collectable_multiplier: 2.5, // Rich in mushrooms and herbs
                flower_multiplier: 1.2,  // Riverside flowers
                rock_multiplier: 0.2,
            },
            BiomeType::TemperateForest => Self {
                tree_multiplier: 2.5,   // Dense forest
                shrub_multiplier: 1.2,  // Moderate undergrowth
                collectable_multiplier: 2.2, // Good mushroom habitat
                flower_multiplier: 0.4,  // Shaded forest floor
                rock_multiplier: 0.3,
            },
            BiomeType::Woodland => Self {
                tree_multiplier: 1.8,   // Moderate tree density
                shrub_multiplier: 1.4,  // Good shrub coverage
                collectable_multiplier: 1.5, // Some foraging resources
                flower_multiplier: 0.8,  // More light than forest
                rock_multiplier: 0.4,
            },
            BiomeType::Grassland => Self {
                tree_multiplier: 0.3,   // Few trees
                shrub_multiplier: 1.5,  // Grasses and small shrubs
                collectable_multiplier: 0.8, // Some foraging
                flower_multiplier: 2.0,  // Meadow flowers
                rock_multiplier: 0.5,
            },
            BiomeType::ForestEdge => Self {
                tree_multiplier: 1.2,   // Edge trees
                shrub_multiplier: 1.6,  // Edge vegetation
                collectable_multiplier: 1.8, // Good foraging
                flower_multiplier: 1.5,  // Edge flowers
                rock_multiplier: 0.3,
            },
            BiomeType::RockyOutcrop => Self {
                tree_multiplier: 0.1,   // Few trees
                shrub_multiplier: 0.4,  // Some hardy shrubs
                collectable_multiplier: 0.3, // Limited resources
                flower_multiplier: 0.2,  // Some alpine flowers
                rock_multiplier: 3.0,   // Rocky
            },
        }
    }
}

/// Resource generation configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    // Tree densities
    pub tree_density: f32,
    // Shrub densities
    pub berry_bush_density: f32,
    pub hazel_shrub_density: f32,
    // Collectable densities
    pub mushroom_patch_density: f32,
    pub wild_root_density: f32,
    // Existing resource densities
    pub rock_density: f32,
    pub bush_density: f32,
    pub flower_density: f32,
    pub enable_resources: bool,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            // Tree densities - reduced from 8% for better natural distribution
            tree_density: 0.05,   // 5% chance of tree per tile (down from 8%)
            // Shrub densities - balanced for gameplay
            berry_bush_density: 0.015,   // 1.5% chance of berry bush per tile (down from 3%)
            hazel_shrub_density: 0.01,  // 1% chance of hazel shrub per tile (down from 2%)
            // Collectable densities - kept rare for special discovery feeling
            mushroom_patch_density: 0.008, // 0.8% chance of mushroom patch per tile (down from 1.5%)
            wild_root_density: 0.006,     // 0.6% chance of wild root per tile (down from 1%)
            // Existing resource densities - balanced for natural appearance
            rock_density: 0.03,   // 3% chance of rock per tile (unchanged)
            bush_density: 0.025,  // 2.5% chance of bush per tile (down from 5%)
            flower_density: 0.025, // 2.5% chance of flower per tile (down from 4%)
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

    /// Generate resource layer for a chunk based on terrain and biome
    pub fn generate_resource_layer(
        &self,
        terrain_layer: &[Vec<String>],
        chunk_x: i32,
        chunk_y: i32,
        world_seed: u64,
    ) -> Vec<Vec<String>> {
        // Generate biome for this chunk
        let biome = BiomeType::from_climate(0.5, 0.5, 0.5); // Default biome for backward compatibility
        self.generate_resource_layer_with_biome(terrain_layer, chunk_x, chunk_y, world_seed, biome)
    }

    /// Generate resource layer for a chunk based on terrain and biome context
    pub fn generate_resource_layer_with_biome(
        &self,
        terrain_layer: &[Vec<String>],
        chunk_x: i32,
        chunk_y: i32,
        world_seed: u64,
        biome: BiomeType,
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

        let biome_multipliers = BiomeResourceMultipliers::for_biome(biome);

        for y in 0..16 {
            for x in 0..16 {
                let terrain = &terrain_layer[y][x];

                // Only place resources on suitable terrain with biome context
                if let Some(resource_type) = self.determine_resource_for_terrain_with_biome(
                    terrain,
                    &mut rng,
                    &biome_multipliers,
                ) {
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
        // Use default biome multipliers for backward compatibility
        let biome_multipliers = BiomeResourceMultipliers::default();
        self.determine_resource_for_terrain_with_biome(terrain, rng, &biome_multipliers)
    }

    /// Determine what resource (if any) should be placed on given terrain with biome context
    fn determine_resource_for_terrain_with_biome(
        &self,
        terrain: &str,
        rng: &mut Pcg64,
        biome_multipliers: &BiomeResourceMultipliers,
    ) -> Option<ResourceType> {
        match terrain {
            "Grass" => {
                let roll = rng.gen::<f32>();

                // Apply biome multipliers to densities
                let tree_chance = self.config.tree_density * biome_multipliers.tree_multiplier;
                let shrub_chance = (self.config.berry_bush_density + self.config.hazel_shrub_density) * biome_multipliers.shrub_multiplier;
                let collectable_chance = (self.config.mushroom_patch_density + self.config.wild_root_density) * biome_multipliers.collectable_multiplier;
                let flower_chance = self.config.flower_density * biome_multipliers.flower_multiplier;

                // Priority order: Trees > Shrubs > Collectables > Flowers > Bushes
                if roll < tree_chance {
                    // Choose tree type - adjusted distribution for more oak dominance
                    let tree_roll = rng.gen::<f32>();
                    if tree_roll < 0.5 { // Increased from 0.4
                        Some(ResourceType::TreeOak)
                    } else if tree_roll < 0.8 { // Adjusted from 0.7
                        Some(ResourceType::TreePine)
                    } else {
                        Some(ResourceType::TreeBirch) // Reduced from 0.3 to 0.2
                    }
                } else if roll < tree_chance + shrub_chance {
                    // Choose shrub type - favor berry bushes
                    let shrub_roll = rng.gen::<f32>();
                    if shrub_roll < 0.7 { // Increased from 0.6
                        Some(ResourceType::BerryBush)
                    } else {
                        Some(ResourceType::HazelShrub) // Reduced from 0.4 to 0.3
                    }
                } else if roll < tree_chance + shrub_chance + collectable_chance {
                    // Choose collectable type - adjusted for rarity balance
                    let collectable_roll = rng.gen::<f32>();
                    if collectable_roll < 0.7 { // Keep mushrooms more common
                        Some(ResourceType::MushroomPatch)
                    } else {
                        Some(ResourceType::WildRoot) // Keep roots rarer
                    }
                } else if roll < tree_chance + shrub_chance + collectable_chance + flower_chance {
                    Some(ResourceType::Flower)
                } else if roll < tree_chance + shrub_chance + collectable_chance + flower_chance + self.config.bush_density {
                    Some(ResourceType::Bush)
                } else {
                    None
                }
            }
            "Forest" => {
                // Forest terrain has higher tree density and more collectables (balanced)
                let tree_chance = self.config.tree_density * 1.8 * biome_multipliers.tree_multiplier; // Reduced from 2.0
                let shrub_chance = self.config.berry_bush_density * 0.6 * biome_multipliers.shrub_multiplier; // Reduced from 0.8
                let collectable_chance = self.config.mushroom_patch_density * 3.0 * biome_multipliers.collectable_multiplier; // Increased from 2.5

                let roll = rng.gen::<f32>();
                if roll < tree_chance {
                    // Choose tree type (strongly favor oak in forests)
                    let tree_roll = rng.gen::<f32>();
                    if tree_roll < 0.7 { // Increased from 0.6
                        Some(ResourceType::TreeOak)
                    } else if tree_roll < 0.9 { // Increased from 0.85
                        Some(ResourceType::TreePine)
                    } else {
                        Some(ResourceType::TreeBirch) // Reduced from 0.15 to 0.1
                    }
                } else if roll < tree_chance + shrub_chance {
                    Some(ResourceType::BerryBush) // Berry bushes more common in forests
                } else if roll < tree_chance + shrub_chance + collectable_chance {
                    Some(ResourceType::MushroomPatch) // Mushrooms thrive in forests
                } else {
                    None
                }
            }
            "Mountain" | "Stone" => {
                // Mountain/stone terrain can have rocks
                let rock_chance = self.config.rock_density * 3.0 * biome_multipliers.rock_multiplier;
                if rng.gen::<f32>() < rock_chance {
                    Some(ResourceType::Rock)
                } else {
                    None
                }
            }
            "Dirt" => {
                // Dirt can have occasional shrubs, collectables, or rocks (balanced)
                let shrub_chance = self.config.hazel_shrub_density * 0.5 * biome_multipliers.shrub_multiplier; // Reduced from 0.6
                let collectable_chance = self.config.wild_root_density * 2.0 * biome_multipliers.collectable_multiplier; // Increased from 1.5
                let rock_chance = self.config.rock_density * 0.3 * biome_multipliers.rock_multiplier; // Reduced from 0.5

                let roll = rng.gen::<f32>();
                if roll < shrub_chance {
                    Some(ResourceType::HazelShrub)
                } else if roll < shrub_chance + collectable_chance {
                    Some(ResourceType::WildRoot)
                } else if roll < shrub_chance + collectable_chance + rock_chance {
                    Some(ResourceType::Rock)
                } else {
                    None
                }
            }
            "Sand" => {
                // Beach sand rarely has anything, maybe occasional bushes
                let shrub_chance = self.config.bush_density * 0.1 * biome_multipliers.shrub_multiplier;
                if rng.gen::<f32>() < shrub_chance {
                    Some(ResourceType::Bush)
                } else {
                    None
                }
            }
            "Swamp" => {
                // Swamps are great for mushrooms and some berries
                let collectable_chance = self.config.mushroom_patch_density * 3.0 * biome_multipliers.collectable_multiplier;
                let shrub_chance = self.config.berry_bush_density * 0.5 * biome_multipliers.shrub_multiplier;

                let roll = rng.gen::<f32>();
                if roll < collectable_chance {
                    Some(ResourceType::MushroomPatch)
                } else if roll < collectable_chance + shrub_chance {
                    Some(ResourceType::BerryBush)
                } else {
                    None
                }
            }
            "Snow" => {
                // Snow has very few resources
                let collectable_chance = self.config.wild_root_density * 0.2 * biome_multipliers.collectable_multiplier;
                if rng.gen::<f32>() < collectable_chance {
                    Some(ResourceType::WildRoot) // Hardy roots can survive snow
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
        // Test existing resources
        assert_eq!(
            ResourceType::from_str("TreeOak"),
            Some(ResourceType::TreeOak)
        );
        assert_eq!(ResourceType::TreeOak.as_str(), "TreeOak");

        // Test new shrub resources
        assert_eq!(
            ResourceType::from_str("BerryBush"),
            Some(ResourceType::BerryBush)
        );
        assert_eq!(ResourceType::BerryBush.as_str(), "BerryBush");

        assert_eq!(
            ResourceType::from_str("HazelShrub"),
            Some(ResourceType::HazelShrub)
        );
        assert_eq!(ResourceType::HazelShrub.as_str(), "HazelShrub");

        // Test new collectable resources
        assert_eq!(
            ResourceType::from_str("MushroomPatch"),
            Some(ResourceType::MushroomPatch)
        );
        assert_eq!(ResourceType::MushroomPatch.as_str(), "MushroomPatch");

        assert_eq!(
            ResourceType::from_str("WildRoot"),
            Some(ResourceType::WildRoot)
        );
        assert_eq!(ResourceType::WildRoot.as_str(), "WildRoot");

        assert_eq!(ResourceType::from_str("Invalid"), None);
    }

    #[test]
    fn test_resource_categories() {
        // Test tree categories
        assert_eq!(
            ResourceType::TreeOak.get_category(),
            Some(ResourceCategory::Tree)
        );

        // Test shrub categories
        assert_eq!(
            ResourceType::BerryBush.get_category(),
            Some(ResourceCategory::Shrub)
        );

        // Test collectable categories
        assert_eq!(
            ResourceType::MushroomPatch.get_category(),
            Some(ResourceCategory::Collectable)
        );
    }

    #[test]
    fn test_herbivore_edible() {
        // Shrubs should be herbivore edible
        assert!(ResourceType::BerryBush.is_herbivore_edible());
        assert!(ResourceType::HazelShrub.is_herbivore_edible());
        assert!(ResourceType::Bush.is_herbivore_edible());

        // Trees and rocks should not be edible
        assert!(!ResourceType::TreeOak.is_herbivore_edible());
        assert!(!ResourceType::Rock.is_herbivore_edible());

        // Collectables should not be herbivore edible
        assert!(!ResourceType::MushroomPatch.is_herbivore_edible());
        assert!(!ResourceType::WildRoot.is_herbivore_edible());
    }

    #[test]
    fn test_gatherable_resources() {
        // Collectables should be gatherable
        assert!(ResourceType::MushroomPatch.is_gatherable());
        assert!(ResourceType::WildRoot.is_gatherable());

        // Other resources should not be gatherable
        assert!(!ResourceType::TreeOak.is_gatherable());
        assert!(!ResourceType::BerryBush.is_gatherable());
        assert!(!ResourceType::Rock.is_gatherable());
    }

    #[test]
    fn test_resource_profiles() {
        // Test that all resource types have profiles
        let resource_types = vec![
            ResourceType::TreeOak,
            ResourceType::TreePine,
            ResourceType::TreeBirch,
            ResourceType::BerryBush,
            ResourceType::HazelShrub,
            ResourceType::MushroomPatch,
            ResourceType::WildRoot,
            ResourceType::Rock,
            ResourceType::Bush,
            ResourceType::Flower,
        ];

        for resource_type in resource_types {
            assert!(
                resource_type.get_profile().is_some(),
                "Missing profile for {:?}",
                resource_type
            );
        }
    }

    #[test]
    fn test_default_config_densities() {
        let config = ResourceConfig::default();

        // Verify all density values are non-negative
        assert!(config.tree_density >= 0.0);
        assert!(config.berry_bush_density >= 0.0);
        assert!(config.hazel_shrub_density >= 0.0);
        assert!(config.mushroom_patch_density >= 0.0);
        assert!(config.wild_root_density >= 0.0);
        assert!(config.rock_density >= 0.0);
        assert!(config.bush_density >= 0.0);
        assert!(config.flower_density >= 0.0);

        // Verify resources are enabled
        assert!(config.enable_resources);
    }

    #[test]
    fn test_biome_multipliers() {
        // Test temperate forest biome multipliers
        let forest_multipliers = BiomeResourceMultipliers::for_biome(BiomeType::TemperateForest);
        assert!(forest_multipliers.tree_multiplier > 1.0);
        assert!(forest_multipliers.collectable_multiplier > 1.0);
        assert!(forest_multipliers.rock_multiplier < 1.0);

        // Test grassland biome multipliers
        let grassland_multipliers = BiomeResourceMultipliers::for_biome(BiomeType::Grassland);
        assert!(grassland_multipliers.tree_multiplier < 0.5);
        assert!(grassland_multipliers.flower_multiplier > 1.0);

        // Test riparian zone biome multipliers
        let riparian_multipliers = BiomeResourceMultipliers::for_biome(BiomeType::RiparianZone);
        assert!(riparian_multipliers.collectable_multiplier > 2.0);
        assert!(riparian_multipliers.tree_multiplier > 1.0);

        // Test rocky outcrop biome multipliers
        let rocky_multipliers = BiomeResourceMultipliers::for_biome(BiomeType::RockyOutcrop);
        assert!(rocky_multipliers.rock_multiplier > 2.0);
        assert!(rocky_multipliers.tree_multiplier < 0.5);
    }

    #[test]
    fn test_biome_aware_resource_generation() {
        let config = ResourceConfig::default();
        let generator = ResourceGenerator::new(config);

        // Test temperate forest biome
        let forest_terrain = vec![vec!["Forest".to_string(); 16]; 16];
        let forest_resources = generator.generate_resource_layer_with_biome(
            &forest_terrain, 0, 0, 12345, BiomeType::TemperateForest
        );

        // Should have some resources in forest
        let forest_resource_count = forest_resources.iter()
            .flatten()
            .filter(|tile| !tile.is_empty())
            .count();
        assert!(forest_resource_count > 0);

        // Test riparian zone should have more mushrooms
        let riparian_terrain = vec![vec!["Grass".to_string(); 16]; 16];
        let riparian_resources = generator.generate_resource_layer_with_biome(
            &riparian_terrain, 0, 0, 12345, BiomeType::RiparianZone
        );

        // Should have some mushroom patches in riparian zone
        let has_mushrooms = riparian_resources.iter()
            .flatten()
            .any(|tile| tile == "MushroomPatch");
        assert!(has_mushrooms);

        // Test rocky outcrop should have more rocks
        let rocky_terrain = vec![vec!["Stone".to_string(); 16]; 16];
        let rocky_resources = generator.generate_resource_layer_with_biome(
            &rocky_terrain, 0, 0, 12345, BiomeType::RockyOutcrop
        );

        // Should have some rocks in rocky outcrop
        let has_rocks = rocky_resources.iter()
            .flatten()
            .any(|tile| tile == "Rock");
        assert!(has_rocks);
    }

    #[test]
    fn test_resource_placement_priority() {
        let config = ResourceConfig {
            tree_density: 0.5,      // High density for testing
            berry_bush_density: 0.3,
            mushroom_patch_density: 0.3,
            bush_density: 0.2,
            flower_density: 0.1,
            ..ResourceConfig::default()
        };
        let generator = ResourceGenerator::new(config);

        let grass_terrain = vec![vec!["Grass".to_string(); 16]; 16];
        let resources = generator.generate_resource_layer_with_biome(
            &grass_terrain, 0, 0, 12345, BiomeType::Grassland
        );

        // Count resource types
        let mut tree_count = 0;
        let mut shrub_count = 0;
        let mut collectable_count = 0;
        let mut flower_count = 0;
        let mut bush_count = 0;

        for row in &resources {
            for tile in row {
                match tile.as_str() {
                    "TreeOak" | "TreePine" | "TreeBirch" => tree_count += 1,
                    "BerryBush" | "HazelShrub" => shrub_count += 1,
                    "MushroomPatch" | "WildRoot" => collectable_count += 1,
                    "Flower" => flower_count += 1,
                    "Bush" => bush_count += 1,
                    _ => {}
                }
            }
        }

        // With high densities, should have multiple resource types
        let total_resources = tree_count + shrub_count + collectable_count + flower_count + bush_count;
        assert!(total_resources > 0);

        // Trees should be most common due to high density and priority
        assert!(tree_count > 0);
    }

    #[test]
    fn test_biome_specific_terrain_handling() {
        let config = ResourceConfig::default();
        let generator = ResourceGenerator::new(config);

        // Test riparian terrain handling
        let riparian_terrain = vec![vec!["Grass".to_string(); 16]; 16];
        let riparian_resources = generator.generate_resource_layer_with_biome(
            &riparian_terrain, 0, 0, 12345, BiomeType::RiparianZone
        );

        // Should primarily have mushrooms and berry bushes
        let has_riparian_resources = riparian_resources.iter()
            .flatten()
            .any(|tile| tile == "MushroomPatch" || tile == "BerryBush");
        assert!(has_riparian_resources);

        // Test rocky terrain handling
        let rocky_terrain = vec![vec!["Stone".to_string(); 16]; 16];
        let rocky_resources = generator.generate_resource_layer_with_biome(
            &rocky_terrain, 0, 0, 12345, BiomeType::RockyOutcrop
        );

        // Should rarely have resources
        let rocky_resource_count = rocky_resources.iter()
            .flatten()
            .filter(|tile| !tile.is_empty())
            .count();
        assert!(rocky_resource_count <= 2); // Very few resources in rocky areas
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
