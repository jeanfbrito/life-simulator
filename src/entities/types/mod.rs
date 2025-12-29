pub mod bear;
pub mod deer;
pub mod fox;
/// Entity behavior configuration system
///
/// This module provides modular behavior configuration for different entity types.
/// Each entity can have a BehaviorConfig component attached that defines its AI parameters.
pub mod rabbit;
pub mod raccoon;
pub mod wolf;

use bevy::prelude::*;
use crate::tilemap::TerrainType;

/// Habitat preference weights for different terrain types
///
/// Each weight represents how much a species prefers that terrain for wandering:
/// - 0.0 = Avoid completely (impassable or dangerous)
/// - 0.5 = Tolerable but not preferred
/// - 1.0 = Neutral (baseline)
/// - 1.5+ = Strongly preferred
#[derive(Debug, Clone)]
pub struct HabitatPreference {
    /// Weight for Grass terrain
    pub grass: f32,
    /// Weight for Forest terrain
    pub forest: f32,
    /// Weight for Dirt terrain
    pub dirt: f32,
    /// Weight for Sand terrain
    pub sand: f32,
    /// Weight for Stone terrain
    pub stone: f32,
    /// Weight for Snow terrain
    pub snow: f32,
    /// Weight for Swamp terrain
    pub swamp: f32,
    /// Weight for Desert terrain
    pub desert: f32,
    /// How much to prefer tiles near food (0.0-1.0)
    /// Higher = strongly prefer wandering toward food sources
    pub food_proximity_weight: f32,
    /// How much to prefer tiles near water (0.0-1.0)
    /// Higher = strongly prefer staying near water sources
    pub water_proximity_weight: f32,
}

impl Default for HabitatPreference {
    fn default() -> Self {
        // Generalist default - tolerates most terrain
        Self {
            grass: 1.0,
            forest: 1.0,
            dirt: 0.8,
            sand: 0.6,
            stone: 0.3,
            snow: 0.4,
            swamp: 0.5,
            desert: 0.4,
            food_proximity_weight: 0.3,
            water_proximity_weight: 0.2,
        }
    }
}

impl HabitatPreference {
    /// Get the weight for a specific terrain type
    pub fn get_terrain_weight(&self, terrain: &TerrainType) -> f32 {
        match terrain {
            TerrainType::Grass => self.grass,
            TerrainType::Forest => self.forest,
            TerrainType::Dirt => self.dirt,
            TerrainType::Sand => self.sand,
            TerrainType::Stone => self.stone,
            TerrainType::Snow => self.snow,
            TerrainType::Swamp => self.swamp,
            TerrainType::Desert => self.desert,
            // Impassable terrains - always 0
            TerrainType::Water | TerrainType::DeepWater | TerrainType::ShallowWater => 0.0,
            TerrainType::Mountain => 0.0,
        }
    }

    /// Rabbit habitat: Prefers open grassland, avoids dense forest
    /// Rabbits thrive in meadows with some cover but good visibility
    pub fn rabbit() -> Self {
        Self {
            grass: 1.5,      // Strongly prefer grass
            forest: 0.6,     // Some tolerance for forest edges
            dirt: 1.0,       // Neutral on dirt
            sand: 0.5,       // Not great but tolerable
            stone: 0.2,      // Avoid rocky areas
            snow: 0.3,       // Cold, avoid
            swamp: 0.3,      // Wet, avoid
            desert: 0.2,     // Too harsh
            food_proximity_weight: 0.5,  // Prefer areas with nearby food
            water_proximity_weight: 0.3, // Moderate water preference
        }
    }

    /// Deer habitat: Prefers forest edges with access to meadows
    /// Deer like the boundary between forest and open areas
    pub fn deer() -> Self {
        Self {
            grass: 1.3,      // Like open meadows
            forest: 1.5,     // Love forest for cover
            dirt: 0.9,       // Tolerate dirt paths
            sand: 0.4,       // Not preferred
            stone: 0.3,      // Avoid rocky terrain
            snow: 0.5,       // Can handle some snow
            swamp: 0.4,      // Avoid swampy areas
            desert: 0.2,     // Too harsh
            food_proximity_weight: 0.4,  // Value food sources
            water_proximity_weight: 0.4, // Need water access
        }
    }

    /// Raccoon habitat: Generalist, prefers forest and wetlands
    /// Raccoons are opportunistic and adaptable
    pub fn raccoon() -> Self {
        Self {
            grass: 1.0,      // Neutral on grass
            forest: 1.4,     // Prefer forest for cover
            dirt: 1.0,       // Fine with dirt
            sand: 0.6,       // Tolerable
            stone: 0.5,      // Some tolerance
            snow: 0.4,       // Cold but manageable
            swamp: 1.3,      // Like wetlands
            desert: 0.3,     // Avoid
            food_proximity_weight: 0.5,  // Opportunistic foragers
            water_proximity_weight: 0.5, // Like staying near water
        }
    }
}

/// Foraging search strategies for herbivores
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForagingStrategy {
    /// Search all tiles within radius (thorough, more expensive)
    Exhaustive,
    /// Sample K random tiles within radius (faster, good approximation)
    Sampled { sample_size: usize },
}

impl Default for ForagingStrategy {
    fn default() -> Self {
        Self::Exhaustive // Default to thorough search for correctness
    }
}

/// Species-level needs and consumption profile
#[derive(Component, Debug, Clone)]
pub struct SpeciesNeeds {
    /// Maximum hunger capacity (absolute units). Higher for larger animals.
    pub hunger_max: f32,
    /// Maximum thirst capacity (absolute units). Higher for larger animals.
    pub thirst_max: f32,
    /// How much hunger is reduced by a single "eat" event (absolute units)
    pub eat_amount: f32,
    /// How much thirst is reduced by a single "drink" event (absolute units)
    pub drink_amount: f32,
}

/// Component that defines an entity's behavior parameters
///
/// This is attached to entities at spawn time and queried by the AI planner
/// to determine behavior thresholds and ranges.
#[derive(Component, Debug, Clone)]
pub struct BehaviorConfig {
    /// Minimum thirst level (0.0-1.0) before seeking water
    /// Example: 0.6 = seek water when 60% thirsty
    pub thirst_threshold: f32,

    /// Minimum hunger level (0.0-1.0) before seeking food
    /// Example: 0.5 = seek food when 50% hungry
    pub hunger_threshold: f32,

    /// Maximum energy level (0.0-1.0) before resting
    /// Example: 0.3 = rest when energy drops below 30%
    /// Note: Lower energy = more tired, so rest when energy is LOW
    pub energy_threshold: f32,

    /// Grazing/foraging range (min_tiles, max_tiles)
    /// Entity will wander this distance to find food
    pub graze_range: (i32, i32),

    /// Maximum search radius for water sources
    pub water_search_radius: i32,

    /// Maximum search radius for food sources
    pub food_search_radius: i32,

    /// General wander radius when no needs are urgent
    pub wander_radius: i32,

    /// Foraging search strategy
    /// Determines how entities search for food: exhaustive vs. sampling
    pub foraging_strategy: ForagingStrategy,

    /// Biomass level that satisfies this species (when at full energy)
    /// Low energy reduces this threshold proportionally, making hungry/tired animals less picky.
    /// Example: 15.0 for rabbits (not picky), 40.0 for deer (selective)
    pub satisfaction_biomass: f32,

    /// Habitat preferences for wandering behavior
    /// Determines which terrain types and conditions the species prefers
    pub habitat_preference: HabitatPreference,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            thirst_threshold: 0.3,
            hunger_threshold: 0.3,
            energy_threshold: 0.3,
            graze_range: (5, 15),
            water_search_radius: 50,
            food_search_radius: 50,
            wander_radius: 20,
            foraging_strategy: ForagingStrategy::default(),
            satisfaction_biomass: 20.0, // Moderate default
            habitat_preference: HabitatPreference::default(),
        }
    }
}

impl BehaviorConfig {
    /// Create a custom behavior configuration
    pub fn new(
        thirst_threshold: f32,
        hunger_threshold: f32,
        energy_threshold: f32,
        graze_range: (i32, i32),
        water_search_radius: i32,
        food_search_radius: i32,
        wander_radius: i32,
    ) -> Self {
        Self {
            thirst_threshold,
            hunger_threshold,
            energy_threshold,
            graze_range,
            water_search_radius,
            food_search_radius,
            wander_radius,
            foraging_strategy: ForagingStrategy::default(),
            satisfaction_biomass: 20.0, // Default moderate pickiness
            habitat_preference: HabitatPreference::default(),
        }
    }

    /// Create a custom behavior configuration with foraging strategy
    pub fn new_with_foraging(
        thirst_threshold: f32,
        hunger_threshold: f32,
        energy_threshold: f32,
        graze_range: (i32, i32),
        water_search_radius: i32,
        food_search_radius: i32,
        wander_radius: i32,
        foraging_strategy: ForagingStrategy,
    ) -> Self {
        Self {
            thirst_threshold,
            hunger_threshold,
            energy_threshold,
            graze_range,
            water_search_radius,
            food_search_radius,
            wander_radius,
            foraging_strategy,
            satisfaction_biomass: 20.0, // Default moderate pickiness
            habitat_preference: HabitatPreference::default(),
        }
    }

    /// Set the satisfaction biomass threshold (fluent builder pattern)
    pub fn with_satisfaction(mut self, satisfaction_biomass: f32) -> Self {
        self.satisfaction_biomass = satisfaction_biomass;
        self
    }

    /// Set the habitat preference (fluent builder pattern)
    pub fn with_habitat(mut self, habitat_preference: HabitatPreference) -> Self {
        self.habitat_preference = habitat_preference;
        self
    }

    /// Create a custom behavior configuration with foraging strategy and satisfaction
    pub fn new_with_satisfaction(
        thirst_threshold: f32,
        hunger_threshold: f32,
        energy_threshold: f32,
        graze_range: (i32, i32),
        water_search_radius: i32,
        food_search_radius: i32,
        wander_radius: i32,
        foraging_strategy: ForagingStrategy,
        satisfaction_biomass: f32,
    ) -> Self {
        Self {
            thirst_threshold,
            hunger_threshold,
            energy_threshold,
            graze_range,
            water_search_radius,
            food_search_radius,
            wander_radius,
            foraging_strategy,
            satisfaction_biomass,
            habitat_preference: HabitatPreference::default(),
        }
    }
}
