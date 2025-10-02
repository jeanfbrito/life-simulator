/// Entity behavior configuration system
/// 
/// This module provides modular behavior configuration for different entity types.
/// Each entity can have a BehaviorConfig component attached that defines its AI parameters.

pub mod rabbit;
pub mod deer;

use bevy::prelude::*;

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
        }
    }
}
