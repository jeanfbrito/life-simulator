//! Generic group formation configuration for all species.
//!
//! This module provides data-driven configuration for group formation behavior.
//! Each species can customize group formation by providing a GroupFormationConfig component.

use bevy::prelude::*;

// Re-export GroupType from pack_relationships
// We'll expand it there to include all group types
pub use crate::entities::pack_relationships::GroupType;

/// Configuration for how a species forms and maintains groups
#[derive(Component, Clone, Debug)]
pub struct GroupFormationConfig {
    /// Is group formation enabled for this species?
    pub enabled: bool,

    /// Type of group this species forms
    pub group_type: GroupType,

    /// Minimum entities required to form a group (3 for packs, 5 for herds)
    pub min_group_size: usize,

    /// Maximum entities in a single group (8 for packs, 20 for herds)
    pub max_group_size: usize,

    /// Radius to search for potential group members (tiles)
    pub formation_radius: f32,

    /// Maximum distance members can drift before group dissolves (tiles)
    pub cohesion_radius: f32,

    /// How often to check for formation opportunities (ticks)
    pub check_interval_ticks: u64,

    /// Minimum time before a dissolved group can reform (ticks)
    pub reformation_cooldown_ticks: u64,
}

impl GroupFormationConfig {
    /// Wolf pack configuration
    pub fn wolf_pack() -> Self {
        Self {
            enabled: true,
            group_type: GroupType::Pack,
            min_group_size: 3,
            max_group_size: 8,
            formation_radius: 50.0,
            cohesion_radius: 150.0,
            check_interval_ticks: 300,
            reformation_cooldown_ticks: 600,
        }
    }

    /// Deer herd configuration
    pub fn deer_herd() -> Self {
        Self {
            enabled: true,
            group_type: GroupType::Herd,
            min_group_size: 5,
            max_group_size: 20,
            formation_radius: 100.0,
            cohesion_radius: 200.0,
            check_interval_ticks: 300,
            reformation_cooldown_ticks: 400,
        }
    }

    /// Rabbit warren configuration
    pub fn rabbit_warren() -> Self {
        Self {
            enabled: true,
            group_type: GroupType::Warren,
            min_group_size: 4,
            max_group_size: 15,
            formation_radius: 30.0,  // Tighter formation
            cohesion_radius: 100.0,
            check_interval_ticks: 200,
            reformation_cooldown_ticks: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RED: Test GroupType has correct name mapping
    #[test]
    fn test_group_type_names() {
        assert_eq!(GroupType::Pack.name(), "pack");
        assert_eq!(GroupType::Herd.name(), "herd");
        assert_eq!(GroupType::Flock.name(), "flock");
        assert_eq!(GroupType::Warren.name(), "warren");
        assert_eq!(GroupType::Colony.name(), "colony");
        assert_eq!(GroupType::School.name(), "school");
    }

    /// RED: Test wolf pack configuration has correct values
    #[test]
    fn test_wolf_pack_config() {
        let config = GroupFormationConfig::wolf_pack();

        assert!(config.enabled);
        assert_eq!(config.group_type, GroupType::Pack);
        assert_eq!(config.min_group_size, 3);
        assert_eq!(config.max_group_size, 8);
        assert_eq!(config.formation_radius, 50.0);
        assert_eq!(config.cohesion_radius, 150.0);
        assert_eq!(config.check_interval_ticks, 300);
        assert_eq!(config.reformation_cooldown_ticks, 600);
    }

    /// RED: Test deer herd configuration has correct values
    #[test]
    fn test_deer_herd_config() {
        let config = GroupFormationConfig::deer_herd();

        assert!(config.enabled);
        assert_eq!(config.group_type, GroupType::Herd);
        assert_eq!(config.min_group_size, 5);
        assert_eq!(config.max_group_size, 20);
        assert_eq!(config.formation_radius, 100.0);
        assert_eq!(config.cohesion_radius, 200.0);
        assert_eq!(config.check_interval_ticks, 300);
        assert_eq!(config.reformation_cooldown_ticks, 400);
    }

    /// RED: Test rabbit warren configuration has correct values
    #[test]
    fn test_rabbit_warren_config() {
        let config = GroupFormationConfig::rabbit_warren();

        assert!(config.enabled);
        assert_eq!(config.group_type, GroupType::Warren);
        assert_eq!(config.min_group_size, 4);
        assert_eq!(config.max_group_size, 15);
        assert_eq!(config.formation_radius, 30.0);
        assert_eq!(config.cohesion_radius, 100.0);
        assert_eq!(config.check_interval_ticks, 200);
        assert_eq!(config.reformation_cooldown_ticks, 300);
    }

    /// RED: Test GroupType equality
    #[test]
    fn test_group_type_equality() {
        assert_eq!(GroupType::Pack, GroupType::Pack);
        assert_ne!(GroupType::Pack, GroupType::Herd);
        assert_ne!(GroupType::Herd, GroupType::Warren);
    }

    /// RED: Test config can be cloned
    #[test]
    fn test_config_clone() {
        let config1 = GroupFormationConfig::wolf_pack();
        let config2 = config1.clone();

        assert_eq!(config1.group_type, config2.group_type);
        assert_eq!(config1.min_group_size, config2.min_group_size);
        assert_eq!(config1.formation_radius, config2.formation_radius);
    }
}
