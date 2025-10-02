/// Rabbit-specific behavior configuration
/// 
/// Defines behavior parameters optimized for rabbit entities.

use super::BehaviorConfig;

/// Rabbit behavior preset
pub struct RabbitBehavior;

impl RabbitBehavior {
    /// Get the default behavior configuration for rabbits
    /// 
    /// Rabbit characteristics:
    /// - Proactive drinkers: Seek water at 15% thirsty to avoid dehydration
    /// - Moderate eaters: Eat grass when 40% hungry
    /// - Light sleepers: Rest when energy drops below 30%
    /// - Short-range grazers: Prefer to stay close (3-8 tiles) when eating
    /// - Moderate search radius: 100 tiles for resources
    /// - Small territory: 15 tile wander radius
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new(
            0.15,       // thirst_threshold: Drink when 15% thirsty
            0.4,        // hunger_threshold: Eat when 40% hungry
            0.3,        // energy_threshold: Rest when energy drops below 30%
            (3, 8),     // graze_range: Short-range grazing (3-8 tiles)
            100,        // water_search_radius: Wide water search
            100,        // food_search_radius: Wide food search
            15,         // wander_radius: Small territory
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rabbit_config() {
        let config = RabbitBehavior::config();
        assert_eq!(config.thirst_threshold, 0.6);
        assert_eq!(config.graze_range, (3, 8));
        assert_eq!(config.water_search_radius, 100);
    }
}
