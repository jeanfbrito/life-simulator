pub mod deer;
/// Entity behavior configuration system
///
/// This module provides modular behavior configuration for different entity types.
/// Each entity can have a BehaviorConfig component attached that defines its AI parameters.
pub mod rabbit;
pub mod raccoon;

use bevy::prelude::*;

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
        }
    }
}
