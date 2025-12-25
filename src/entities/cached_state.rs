/// Cached entity state for performance optimization
///
/// This component caches frequently-accessed entity data to reduce
/// query overhead and eliminate repeated calculations in AI planning.

use bevy::prelude::*;
use crate::entities::{TilePosition, Hunger, Thirst, Energy, Age, ReproductionConfig};

/// Cached entity state bundle
#[derive(Component, Debug, Clone)]
pub struct CachedEntityState {
    /// Current tile position (avoid HashMap lookup)
    pub tile: IVec2,

    /// Pre-computed stat urgencies (0.0-1.0)
    pub hunger_urgency: f32,
    pub thirst_urgency: f32,
    pub energy_urgency: f32,

    /// Pre-computed flags for fast decision making
    pub is_emergency: bool,      // Any stat in critical range
    pub is_juvenile: bool,        // Age-based flag
    pub can_mate: bool,           // Reproduction eligible

    /// Cache invalidation
    pub dirty: bool,
    pub last_update_tick: u64,
}

impl Default for CachedEntityState {
    fn default() -> Self {
        Self {
            tile: IVec2::ZERO,
            hunger_urgency: 0.0,
            thirst_urgency: 0.0,
            energy_urgency: 0.0,
            is_emergency: false,
            is_juvenile: false,
            can_mate: false,
            dirty: true,
            last_update_tick: 0,
        }
    }
}

impl CachedEntityState {
    /// Create new cached state from entity components
    pub fn from_components(
        pos: &TilePosition,
        hunger: &Hunger,
        thirst: &Thirst,
        energy: &Energy,
        age: Option<&Age>,
        repro_config: Option<&ReproductionConfig>,
        current_tick: u64,
    ) -> Self {
        let hunger_urgency = hunger.urgency();
        let thirst_urgency = thirst.urgency();
        let energy_urgency = energy.urgency();

        // Emergency if any stat in critical range (matches planner.rs thresholds)
        let is_emergency = hunger.0.normalized() >= 0.85
                        || thirst.0.normalized() >= 0.85
                        || energy.0.normalized() <= 0.15;

        // Juvenile check (inverse of is_adult)
        let is_juvenile = age.map(|a| !a.is_adult()).unwrap_or(false);

        // Mating eligibility
        let can_mate = repro_config.is_some() && !is_juvenile && !is_emergency;

        Self {
            tile: pos.tile,
            hunger_urgency,
            thirst_urgency,
            energy_urgency,
            is_emergency,
            is_juvenile,
            can_mate,
            dirty: false,
            last_update_tick: current_tick,
        }
    }

    /// Mark cache as needing update
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if cache needs update
    pub fn needs_update(&self, current_tick: u64) -> bool {
        self.dirty || current_tick > self.last_update_tick
    }
}

/// System to update cached entity state
/// Runs early in the tick to ensure fresh data for planning
pub fn update_cached_entity_state_system(
    mut query: Query<(
        &TilePosition,
        &Hunger,
        &Thirst,
        &Energy,
        Option<&Age>,
        Option<&ReproductionConfig>,
        &mut CachedEntityState,
    )>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    let current_tick = tick.0;

    for (pos, hunger, thirst, energy, age, repro_config, mut cached) in query.iter_mut() {
        if cached.needs_update(current_tick) {
            *cached = CachedEntityState::from_components(
                pos,
                hunger,
                thirst,
                energy,
                age,
                repro_config,
                current_tick,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Stat, Hunger, Thirst, Energy};

    #[test]
    fn test_cached_state_creation() {
        let pos = TilePosition { tile: IVec2::new(10, 20) };
        let hunger = Hunger(Stat::new(50.0, 0.0, 100.0, 0.1));
        let thirst = Thirst(Stat::new(30.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(70.0, 0.0, 100.0, -0.05));

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            None,
            None,
            100,
        );

        assert_eq!(cached.tile, IVec2::new(10, 20));
        assert_eq!(cached.hunger_urgency, 0.5); // 50/100
        assert_eq!(cached.thirst_urgency, 0.3); // 30/100
        assert!(!cached.is_emergency); // None critical
        assert!(!cached.dirty);
    }

    #[test]
    fn test_emergency_detection_hunger() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(90.0, 0.0, 100.0, 0.1)); // Critical!
        let thirst = Thirst(Stat::new(30.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(70.0, 0.0, 100.0, -0.05));

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            None,
            None,
            100,
        );

        assert!(cached.is_emergency); // Hunger >= 85%
    }

    #[test]
    fn test_emergency_detection_thirst() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(50.0, 0.0, 100.0, 0.1));
        let thirst = Thirst(Stat::new(87.0, 0.0, 100.0, 0.15)); // Critical!
        let energy = Energy(Stat::new(70.0, 0.0, 100.0, -0.05));

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            None,
            None,
            100,
        );

        assert!(cached.is_emergency); // Thirst >= 85%
    }

    #[test]
    fn test_emergency_detection_energy() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(50.0, 0.0, 100.0, 0.1));
        let thirst = Thirst(Stat::new(30.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(10.0, 0.0, 100.0, -0.05)); // Critical!

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            None,
            None,
            100,
        );

        assert!(cached.is_emergency); // Energy <= 15%
    }

    #[test]
    fn test_juvenile_detection() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(50.0, 0.0, 100.0, 0.1));
        let thirst = Thirst(Stat::new(30.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(70.0, 0.0, 100.0, -0.05));

        // Create juvenile age (below maturity threshold)
        let age = Age {
            ticks_alive: 500,
            mature_at_ticks: 1000,
        };

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            Some(&age),
            None,
            100,
        );

        assert!(cached.is_juvenile); // Age below maturity
        assert!(!cached.can_mate); // Juveniles cannot mate
    }

    #[test]
    fn test_adult_detection() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(50.0, 0.0, 100.0, 0.1));
        let thirst = Thirst(Stat::new(30.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(70.0, 0.0, 100.0, -0.05));

        // Create adult age (above maturity threshold)
        let age = Age {
            ticks_alive: 1500,
            mature_at_ticks: 1000,
        };

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            Some(&age),
            None,
            100,
        );

        assert!(!cached.is_juvenile); // Age above maturity
    }

    #[test]
    fn test_can_mate_requires_reproduction_config() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(50.0, 0.0, 100.0, 0.1));
        let thirst = Thirst(Stat::new(30.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(70.0, 0.0, 100.0, -0.05));

        let age = Age {
            ticks_alive: 1500,
            mature_at_ticks: 1000,
        };

        // Without ReproductionConfig
        let cached_no_config = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            Some(&age),
            None,
            100,
        );

        assert!(!cached_no_config.can_mate); // No reproduction config

        // With ReproductionConfig
        let repro_config = ReproductionConfig {
            maturity_ticks: 1000,
            gestation_ticks: 200,
            mating_cooldown_ticks: 100,
            postpartum_cooldown_ticks: 50,
            litter_size_range: (1, 4),
            mating_search_radius: 5,
            well_fed_hunger_norm: 0.5,
            well_fed_thirst_norm: 0.5,
            well_fed_required_ticks: 10,
            matching_interval_ticks: 10,
            mating_duration_ticks: 5,
            min_energy_norm: 0.3,
            min_health_norm: 0.5,
        };

        let cached_with_config = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            Some(&age),
            Some(&repro_config),
            100,
        );

        assert!(cached_with_config.can_mate); // Has reproduction config and is adult
    }

    #[test]
    fn test_can_mate_blocked_by_emergency() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(90.0, 0.0, 100.0, 0.1)); // Emergency!
        let thirst = Thirst(Stat::new(30.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(70.0, 0.0, 100.0, -0.05));

        let age = Age {
            ticks_alive: 1500,
            mature_at_ticks: 1000,
        };

        let repro_config = ReproductionConfig {
            maturity_ticks: 1000,
            gestation_ticks: 200,
            mating_cooldown_ticks: 100,
            postpartum_cooldown_ticks: 50,
            litter_size_range: (1, 4),
            mating_search_radius: 5,
            well_fed_hunger_norm: 0.5,
            well_fed_thirst_norm: 0.5,
            well_fed_required_ticks: 10,
            matching_interval_ticks: 10,
            mating_duration_ticks: 5,
            min_energy_norm: 0.3,
            min_health_norm: 0.5,
        };

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            Some(&age),
            Some(&repro_config),
            100,
        );

        assert!(cached.is_emergency);
        assert!(!cached.can_mate); // Emergency blocks mating
    }

    #[test]
    fn test_dirty_flag() {
        let mut cached = CachedEntityState::default();
        assert!(cached.dirty);

        cached.dirty = false;
        assert!(!cached.needs_update(0));

        cached.mark_dirty();
        assert!(cached.needs_update(0));
    }

    #[test]
    fn test_needs_update_tick_advance() {
        let mut cached = CachedEntityState::default();
        cached.dirty = false;
        cached.last_update_tick = 100;

        // Same tick - no update needed
        assert!(!cached.needs_update(100));

        // Next tick - update needed
        assert!(cached.needs_update(101));
    }

    #[test]
    fn test_urgency_calculations() {
        let pos = TilePosition { tile: IVec2::ZERO };
        let hunger = Hunger(Stat::new(75.0, 0.0, 100.0, 0.1));
        let thirst = Thirst(Stat::new(60.0, 0.0, 100.0, 0.15));
        let energy = Energy(Stat::new(40.0, 0.0, 100.0, -0.05));

        let cached = CachedEntityState::from_components(
            &pos,
            &hunger,
            &thirst,
            &energy,
            None,
            None,
            100,
        );

        // Check urgency calculations
        assert_eq!(cached.hunger_urgency, 0.75); // 75/100
        assert_eq!(cached.thirst_urgency, 0.6);  // 60/100
        assert_eq!(cached.energy_urgency, 0.6);  // (100-40)/100 inverted
    }
}
