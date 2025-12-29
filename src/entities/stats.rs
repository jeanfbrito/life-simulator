/// Entity statistics system for Tick-Queued Utility AI (TQUAI)
///
/// This module provides a flexible, component-based stats system where:
/// - Each stat has min/max bounds and current value
/// - Stats decay/regenerate at configurable rates per tick
/// - Stats are normalized (0.0-1.0) for utility calculations
/// - Thresholds (critical/low/normal) trigger AI decisions
use bevy::prelude::*;

use crate::entities::{Carcass, Creature, SpeciesNeeds, TilePosition};

// ============================================================================
// STAT COMPONENTS
// ============================================================================

/// Generic stat with bounds, decay rate, and current value
#[derive(Component, Debug, Clone)]
pub struct Stat {
    pub current: f32,
    pub min: f32,
    pub max: f32,
    /// Change per tick (negative = decay, positive = regen)
    pub rate_per_tick: f32,
}

impl Stat {
    /// Create a new stat with initial value
    pub fn new(initial: f32, min: f32, max: f32, rate_per_tick: f32) -> Self {
        Self {
            current: initial.clamp(min, max),
            min,
            max,
            rate_per_tick,
        }
    }

    /// Create a stat starting at max
    pub fn new_full(max: f32, rate_per_tick: f32) -> Self {
        Self::new(max, 0.0, max, rate_per_tick)
    }

    /// Get normalized value (0.0 = min, 1.0 = max)
    #[inline(always)]
    pub fn normalized(&self) -> f32 {
        if self.max == self.min {
            1.0
        } else {
            (self.current - self.min) / (self.max - self.min)
        }
    }

    /// Get inverted normalized value (1.0 = min, 0.0 = max)
    /// Useful for needs like hunger where higher = worse
    #[inline(always)]
    pub fn normalized_inverted(&self) -> f32 {
        1.0 - self.normalized()
    }

    /// Update stat by delta (clamped to bounds)
    pub fn change(&mut self, delta: f32) {
        self.current = (self.current + delta).clamp(self.min, self.max);
    }

    /// Apply tick-based decay/regen
    pub fn tick(&mut self) {
        self.change(self.rate_per_tick);
    }

    /// Set to a specific value (clamped)
    pub fn set(&mut self, value: f32) {
        self.current = value.clamp(self.min, self.max);
    }

    /// Check if stat is at or below critical threshold (10%)
    #[inline(always)]
    pub fn is_critical(&self) -> bool {
        self.normalized() <= 0.1
    }

    /// Check if stat is below low threshold (30%)
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        self.normalized() <= 0.3
    }

    /// Check if stat is above high threshold (70%)
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        self.normalized() >= 0.7
    }

    /// Check if stat is full
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Check if stat is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.current <= self.min
    }

    /// Get percentage (0-100)
    pub fn percentage(&self) -> f32 {
        self.normalized() * 100.0
    }
}

// ============================================================================
// SPECIFIC STAT COMPONENTS
// ============================================================================

/// Hunger stat - increases over time, needs food
///
/// Phase 4: Required Components
/// Hunger automatically requires Creature - compile-time guarantee
/// that any entity with hunger is a creature.
#[derive(Component, Debug, Clone)]
#[require(crate::entities::Creature)]
pub struct Hunger(pub Stat);

impl Hunger {
    pub fn new() -> Self {
        // Starts at 0 (not hungry), max 100, increases by 0.1 per tick
        // At 10 TPS, fully hungry in ~16.6 minutes
        Self(Stat::new(0.0, 0.0, 100.0, 0.1))
    }

    /// Get hunger urgency for utility AI (0.0 = not hungry, 1.0 = starving)
    #[inline(always)]
    pub fn urgency(&self) -> f32 {
        self.0.normalized()
    }
}

/// Thirst stat - increases faster than hunger
///
/// Phase 4: Required Components
/// Thirst automatically requires Creature - compile-time guarantee.
#[derive(Component, Debug, Clone)]
#[require(crate::entities::Creature)]
pub struct Thirst(pub Stat);

impl Thirst {
    pub fn new() -> Self {
        // Starts at 0 (not thirsty), max 100, increases by 0.15 per tick
        // At 10 TPS, fully thirsty in ~11 minutes
        Self(Stat::new(0.0, 0.0, 100.0, 0.15))
    }

    /// Get thirst urgency for utility AI
    #[inline(always)]
    pub fn urgency(&self) -> f32 {
        self.0.normalized()
    }
}

/// Energy stat - depletes during activity, regenerates during rest
///
/// Phase 4: Required Components
/// Energy automatically requires Creature - compile-time guarantee.
#[derive(Component, Debug, Clone)]
#[require(crate::entities::Creature)]
pub struct Energy(pub Stat);

impl Energy {
    pub fn new() -> Self {
        // Starts at 100 (full energy), min 0, decreases by 0.05 per tick
        // At 10 TPS, fully depleted in ~33 minutes
        Self(Stat::new(100.0, 0.0, 100.0, -0.05))
    }

    /// Get tiredness urgency for utility AI (0.0 = full energy, 1.0 = exhausted)
    #[inline(always)]
    pub fn urgency(&self) -> f32 {
        self.0.normalized_inverted()
    }

    /// Set to resting rate (faster regen)
    pub fn set_resting(&mut self) {
        self.0.rate_per_tick = 0.35; // Faster regen while resting (sleep-like)
    }

    /// Set to active rate (slower decay)
    pub fn set_active(&mut self) {
        self.0.rate_per_tick = -0.05; // Decay while active
    }
}

/// Health stat - damaged by hazards, regenerates slowly
///
/// Phase 4: Required Components
/// Health automatically requires Creature - compile-time guarantee.
#[derive(Component, Debug, Clone)]
#[require(crate::entities::Creature)]
pub struct Health(pub Stat);

impl Health {
    pub fn new() -> Self {
        // Starts at 100 (full health), min 0, slow natural regen
        Self(Stat::new(100.0, 0.0, 100.0, 0.01))
    }

    /// Get health urgency for utility AI (0.0 = healthy, 1.0 = dying)
    #[inline(always)]
    pub fn urgency(&self) -> f32 {
        self.0.normalized_inverted()
    }

    /// Check if entity is dead
    #[inline(always)]
    pub fn is_dead(&self) -> bool {
        self.0.is_empty()
    }

    /// Apply progressive health damage based on hunger and thirst levels
    /// Only applies damage if either need >= 90%
    /// Uses the WORSE of hunger/thirst damage (they don't stack)
    pub fn apply_need_damage(&mut self, hunger: &Hunger, thirst: &Thirst) {
        let hunger_norm = hunger.0.normalized();
        let thirst_norm = thirst.0.normalized();

        // Calculate damage from each need
        let hunger_damage = Self::calculate_need_damage(hunger_norm);
        let thirst_damage = Self::calculate_need_damage(thirst_norm);

        // Apply the worse damage (they don't stack)
        let damage = hunger_damage.max(thirst_damage);
        if damage > 0.0 {
            self.0.change(-damage);
        }
    }

    /// Calculate progressive damage based on normalized need level (0.0-1.0)
    /// Returns damage per tick:
    /// - 90-95% hungry/thirsty: 0.05 health/tick (warning)
    /// - 95-98%: 0.2 health/tick (danger)
    /// - 98%+: 0.5 health/tick (critical)
    fn calculate_need_damage(need_normalized: f32) -> f32 {
        if need_normalized >= 0.98 {
            0.5 // Critical: death in ~200 ticks = 20 seconds
        } else if need_normalized >= 0.95 {
            0.2 // Danger: death in ~500 ticks = 50 seconds
        } else if need_normalized >= 0.90 {
            0.05 // Warning: can survive indefinitely
        } else {
            0.0 // No damage
        }
    }
}

// ============================================================================
// BUNDLE FOR COMPLETE ENTITY STATS
// ============================================================================

/// Bundle containing all basic needs stats
#[derive(Bundle)]
pub struct EntityStatsBundle {
    pub hunger: Hunger,
    pub thirst: Thirst,
    pub energy: Energy,
    pub health: Health,
    pub cached_state: crate::entities::CachedEntityState,
}

impl Default for EntityStatsBundle {
    fn default() -> Self {
        Self {
            hunger: Hunger::new(),
            thirst: Thirst::new(),
            energy: Energy::new(),
            health: Health::new(),
            cached_state: crate::entities::CachedEntityState::default(),
        }
    }
}

// ============================================================================
// SYSTEMS (TICK-SYNCED)
// ============================================================================

/// Update all stats by their tick rates
/// MUST run in Update schedule with `run_if(should_run_tick_systems)` guard (tick-synced at 10 TPS)
/// Only processes entities with changed stats (using Changed<T> filters)
pub fn tick_stats_system(
    mut query: Query<(
        Entity,
        Option<&mut Hunger>,
        Option<&mut Thirst>,
        Option<&mut Energy>,
        Option<&mut Health>,
    )>,
    tick: Res<crate::simulation::SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    use crate::simulation::profiler::end_timing_resource;
    use crate::simulation::profiler::start_timing_resource;

    start_timing_resource(&mut profiler, "stats");
    for (entity, hunger, thirst, energy, health) in query.iter_mut() {
        // Update each stat if present
        if let Some(mut h) = hunger {
            h.0.tick();

            // Log critical states for debugging
            if h.0.is_critical() && tick.0 % 10 == 0 {
                debug!(
                    "Entity {:?} hunger CRITICAL: {:.1}%",
                    entity,
                    h.0.percentage()
                );
            }
        }

        if let Some(mut t) = thirst {
            t.0.tick();

            if t.0.is_critical() && tick.0 % 10 == 0 {
                debug!(
                    "Entity {:?} thirst CRITICAL: {:.1}%",
                    entity,
                    t.0.percentage()
                );
            }
        }

        if let Some(mut e) = energy {
            e.0.tick();

            if e.0.is_critical() && tick.0 % 10 == 0 {
                debug!(
                    "Entity {:?} energy CRITICAL: {:.1}%",
                    entity,
                    e.0.percentage()
                );
            }
        }

        if let Some(mut h) = health {
            h.0.tick();

            if h.0.is_critical() && tick.0 % 10 == 0 {
                warn!(
                    "Entity {:?} health CRITICAL: {:.1}%",
                    entity,
                    h.0.percentage()
                );
            }
        }
    }

    end_timing_resource(&mut profiler, "stats");
}

/// Apply progressive damage from starvation and dehydration
/// MUST run in Update schedule with `run_if(should_run_tick_systems)` guard (tick-synced at 10 TPS)
/// Only processes entities with changed hunger, thirst, or health
pub fn need_damage_system(
    mut query: Query<(Entity, &mut Health, &Hunger, &Thirst), Or<(Changed<Hunger>, Changed<Thirst>, Changed<Health>)>>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    for (entity, mut health, hunger, thirst) in query.iter_mut() {
        let old_health = health.0.current;

        health.apply_need_damage(hunger, thirst);

        // Log damage events every 10 ticks to avoid spam
        if tick.0 % 10 == 0 && health.0.current < old_health {
            let hunger_norm = hunger.0.normalized();
            let thirst_norm = thirst.0.normalized();

            warn!(
                "Entity {:?} taking need damage: health {:.1} (hunger {:.1}%, thirst {:.1}%)",
                entity, health.0.current, hunger_norm * 100.0, thirst_norm * 100.0
            );
        }
    }
}

/// Handle death when health reaches zero
pub fn death_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Health,
        Option<&TilePosition>,
        Option<&SpeciesNeeds>,
        Option<&Creature>,
    )>,
) {
    for (entity, health, position, needs, creature) in query.iter() {
        if health.is_dead() {
            info!("Entity {:?} has died!", entity);

            if let Some(pos) = position {
                let species_label = creature
                    .map(|c| c.species.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let base_nutrition = needs
                    .map(|n| (n.eat_amount * 3.0).max(20.0))
                    .unwrap_or(50.0);
                let decay_ticks = 7_200; // ~12 minutes at 10 TPS

                commands.spawn((
                    Carcass::new(species_label, base_nutrition, decay_ticks),
                    TilePosition::from_tile(pos.tile),
                ));
            }

            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS FOR TQUAI
// ============================================================================

/// Calculate utility score for "eat food" action
/// Returns 0.0-1.0 where higher = more urgent
pub fn utility_eat(hunger: &Hunger) -> f32 {
    let base = hunger.urgency();

    // Exponential urgency when critical
    if hunger.0.is_critical() {
        base * base // Square it for extra urgency
    } else {
        base
    }
}

/// Calculate utility score for "drink water" action
pub fn utility_drink(thirst: &Thirst) -> f32 {
    let base = thirst.urgency();

    // Thirst is more urgent than hunger
    if thirst.0.is_critical() {
        (base * base * 1.2).min(1.0)
    } else {
        base * 1.1
    }
}

/// Calculate utility score for "sleep/rest" action
pub fn utility_rest(energy: &Energy) -> f32 {
    energy.urgency()
}

/// Calculate utility score for "seek healing" action
pub fn utility_heal(health: &Health) -> f32 {
    let base = health.urgency();

    // Health is critical priority
    if health.0.is_critical() {
        1.0 // Max priority
    } else {
        base * 1.5
    }
}

/// Get highest priority need for an entity
pub fn get_most_urgent_need(
    hunger: Option<&Hunger>,
    thirst: Option<&Thirst>,
    energy: Option<&Energy>,
    health: Option<&Health>,
) -> Option<(String, f32)> {
    let mut needs = Vec::new();

    if let Some(h) = hunger {
        needs.push(("hunger".to_string(), utility_eat(h)));
    }
    if let Some(t) = thirst {
        needs.push(("thirst".to_string(), utility_drink(t)));
    }
    if let Some(e) = energy {
        needs.push(("energy".to_string(), utility_rest(e)));
    }
    if let Some(h) = health {
        needs.push(("health".to_string(), utility_heal(h)));
    }

    needs
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_bounds() {
        let mut stat = Stat::new(50.0, 0.0, 100.0, 0.0);

        stat.change(100.0);
        assert_eq!(stat.current, 100.0); // Clamped to max

        stat.change(-200.0);
        assert_eq!(stat.current, 0.0); // Clamped to min
    }

    #[test]
    fn test_stat_normalized() {
        let stat = Stat::new(50.0, 0.0, 100.0, 0.0);
        assert_eq!(stat.normalized(), 0.5);

        let stat = Stat::new(0.0, 0.0, 100.0, 0.0);
        assert_eq!(stat.normalized(), 0.0);

        let stat = Stat::new(100.0, 0.0, 100.0, 0.0);
        assert_eq!(stat.normalized(), 1.0);
    }

    #[test]
    fn test_hunger_decay() {
        let mut hunger = Hunger::new();
        assert_eq!(hunger.0.current, 0.0);

        // Simulate 100 ticks
        for _ in 0..100 {
            hunger.0.tick();
        }

        // Use approximate comparison due to floating point precision
        assert!((hunger.0.current - 10.0).abs() < 0.001); // 0.1 per tick * 100
    }

    #[test]
    fn test_utility_calculations() {
        let hunger = Hunger::new();
        assert!(utility_eat(&hunger) < 0.1); // Not hungry yet

        let mut critical_hunger = Hunger::new();
        critical_hunger.0.set(95.0);
        assert!(utility_eat(&critical_hunger) > 0.9); // Very urgent
    }

    #[test]
    fn test_starvation_damage_warning() {
        let mut health = Health::new();
        let mut hunger = Hunger::new();
        let thirst = Thirst::new();

        hunger.0.set(92.0);  // 92% = warning zone
        health.apply_need_damage(&hunger, &thirst);

        assert!(health.0.current < 100.0);  // Damage applied
        assert!(health.0.current > 99.0);   // But very small (0.05)
    }

    #[test]
    fn test_starvation_damage_critical() {
        let mut health = Health::new();
        let mut hunger = Hunger::new();
        let thirst = Thirst::new();

        hunger.0.set(99.0);  // 99% = critical
        health.apply_need_damage(&hunger, &thirst);

        assert_eq!(health.0.current, 99.5);  // Lost 0.5 health
    }

    #[test]
    fn test_thirst_overrides_hunger() {
        let mut health = Health::new();
        let mut hunger = Hunger::new();
        let mut thirst = Thirst::new();

        hunger.0.set(92.0);  // 92% hunger = 0.05 damage
        thirst.0.set(99.0);  // 99% thirst = 0.5 damage
        health.apply_need_damage(&hunger, &thirst);

        assert_eq!(health.0.current, 99.5);  // Uses worse damage (0.5)
    }

    #[test]
    fn test_no_damage_below_threshold() {
        let mut health = Health::new();
        let mut hunger = Hunger::new();
        let thirst = Thirst::new();

        hunger.0.set(85.0);  // Below 90% threshold
        health.apply_need_damage(&hunger, &thirst);

        assert_eq!(health.0.current, 100.0);  // No damage
    }

    #[test]
    fn test_entity_stats_bundle_includes_cached_state() {
        let bundle = EntityStatsBundle::default();

        // Verify all stats are present
        assert_eq!(bundle.hunger.0.current, 0.0);
        assert_eq!(bundle.thirst.0.current, 0.0);
        assert_eq!(bundle.energy.0.current, 100.0);
        assert_eq!(bundle.health.0.current, 100.0);

        // Verify cached state component exists in bundle
        assert_eq!(bundle.cached_state.tile, bevy::math::IVec2::ZERO);
        // Default cached state starts with all urgencies at 0.0
        assert_eq!(bundle.cached_state.hunger_urgency, 0.0);
        assert_eq!(bundle.cached_state.thirst_urgency, 0.0);
        assert_eq!(bundle.cached_state.energy_urgency, 0.0);
        // Should be dirty on creation (needs update on first use)
        assert!(bundle.cached_state.dirty);
    }
}
