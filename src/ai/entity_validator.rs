/// Entity Validator System
///
/// DEFENSIVE AI VALIDATION: Catches problems before they cause entity death.
///
/// This module provides periodic validation of AI entities to:
/// 1. Detect entities with BehaviorConfig but missing required tracker components
/// 2. Auto-fix by adding missing IdleTracker and StatThresholdTracker components
/// 3. Detect "stuck" entities (high hunger but Idle for too long)
/// 4. Log warnings/errors for broken entity states
///
/// Runs every 50 ticks to catch configuration issues early.
use bevy::prelude::*;

use crate::ai::trigger_emitters::{IdleTracker, StatThresholdTracker};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::BehaviorConfig;
use crate::entities::CurrentAction;
use crate::simulation::SimulationTick;

/// Statistics for entity validation
#[derive(Debug, Default)]
pub struct ValidationStats {
    /// Number of entities validated
    pub entities_checked: u32,
    /// Number of entities with missing components that were fixed
    pub components_auto_fixed: u32,
    /// Number of stuck entities detected
    pub stuck_entities: u32,
    /// Number of completely broken entities (unfixable)
    pub broken_entities: u32,
}

/// Resource to track validation statistics across ticks
#[derive(Resource, Default, Debug)]
pub struct EntityValidationMetrics {
    /// Stats from last validation run
    pub last_run_stats: ValidationStats,
    /// Tick of last validation run
    pub last_run_tick: u64,
    /// Total entities fixed across all runs
    pub total_fixed: u64,
    /// Total stuck entities detected across all runs
    pub total_stuck_detected: u64,
}

/// Validation interval in ticks (every 50 ticks = 5 seconds at 10 TPS)
const VALIDATION_INTERVAL_TICKS: u64 = 50;

/// Stuck entity threshold: high hunger (>80%) + idle for >100 ticks
const STUCK_HUNGER_THRESHOLD: f32 = 0.80;
const STUCK_IDLE_TICKS_THRESHOLD: u32 = 100;

/// Entity Validator System
///
/// Runs every 50 ticks to:
/// 1. Find entities with BehaviorConfig but missing IdleTracker or StatThresholdTracker
/// 2. Auto-fix by adding missing components
/// 3. Detect stuck entities (high hunger >80% but Idle for >100 ticks)
/// 4. Log appropriate warnings/errors
pub fn entity_validation_system(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    mut metrics: ResMut<EntityValidationMetrics>,
    // Query for entities that have BehaviorConfig (should have tracker components)
    behavior_entities: Query<(
        Entity,
        &BehaviorConfig,
        Option<&Hunger>,
        Option<&Thirst>,
        Option<&Energy>,
        Option<&IdleTracker>,
        Option<&StatThresholdTracker>,
        Option<&CurrentAction>,
        Option<&crate::entities::Creature>,
    )>,
) {
    // Only run every N ticks
    if tick.0 % VALIDATION_INTERVAL_TICKS != 0 {
        return;
    }

    let mut stats = ValidationStats::default();

    for (
        entity,
        _behavior,
        hunger_opt,
        thirst_opt,
        energy_opt,
        idle_opt,
        threshold_opt,
        action_opt,
        creature_opt,
    ) in behavior_entities.iter()
    {
        stats.entities_checked += 1;
        let name = creature_opt.map(|c| c.name.as_str()).unwrap_or("Unknown");

        // === CHECK 1: Missing IdleTracker ===
        if idle_opt.is_none() {
            warn!(
                "[EntityValidator] {} (Entity {:?}) has BehaviorConfig but MISSING IdleTracker - auto-fixing",
                name, entity
            );
            commands.entity(entity).insert(IdleTracker::new(tick.0));
            stats.components_auto_fixed += 1;
        }

        // === CHECK 2: Missing StatThresholdTracker ===
        if threshold_opt.is_none() {
            // Get current stat values for initialization
            let hunger_norm = hunger_opt.map(|h| h.0.normalized()).unwrap_or(0.0);
            let thirst_norm = thirst_opt.map(|t| t.0.normalized()).unwrap_or(0.0);
            let energy_norm = energy_opt.map(|e| e.0.normalized()).unwrap_or(1.0);

            warn!(
                "[EntityValidator] {} (Entity {:?}) has BehaviorConfig but MISSING StatThresholdTracker - auto-fixing (H:{:.1}% T:{:.1}% E:{:.1}%)",
                name, entity, hunger_norm * 100.0, thirst_norm * 100.0, energy_norm * 100.0
            );
            commands.entity(entity).insert(StatThresholdTracker::new(
                hunger_norm,
                thirst_norm,
                energy_norm,
            ));
            stats.components_auto_fixed += 1;
        }

        // === CHECK 3: Stuck Entity Detection ===
        // An entity is "stuck" if:
        // - Hunger is high (>80%)
        // - Has been idle for too long (>100 ticks)
        // - But isn't actively doing anything
        if let (Some(hunger), Some(idle_tracker)) = (hunger_opt, idle_opt) {
            let hunger_norm = hunger.0.normalized();
            let is_high_hunger = hunger_norm >= STUCK_HUNGER_THRESHOLD;
            let is_long_idle = idle_tracker.ticks_since_action >= STUCK_IDLE_TICKS_THRESHOLD;

            // Check if entity has a current action
            let is_doing_action = action_opt
                .map(|action| action.action_name != "None" && action.action_name != "Idle")
                .unwrap_or(false);

            if is_high_hunger && is_long_idle && !is_doing_action {
                stats.stuck_entities += 1;

                error!(
                    "[EntityValidator] STUCK ENTITY DETECTED: {} (Entity {:?}) - Hunger {:.1}%, Idle for {} ticks, Action: {:?}. Entity may die soon!",
                    name,
                    entity,
                    hunger_norm * 100.0,
                    idle_tracker.ticks_since_action,
                    action_opt.map(|a| a.action_name.as_str()).unwrap_or("None")
                );

                // Force a replan by resetting the idle tracker
                // This gives the entity another chance to find food
                if idle_tracker.ticks_since_action > STUCK_IDLE_TICKS_THRESHOLD * 2 {
                    warn!(
                        "[EntityValidator] Force-resetting IdleTracker for stuck entity {} to trigger replan",
                        name
                    );
                    commands.entity(entity).insert(IdleTracker::new(tick.0));
                }
            }
        }

        // === CHECK 4: Missing Stats Components ===
        // BehaviorConfig entities should have hunger/thirst/energy
        if hunger_opt.is_none() || thirst_opt.is_none() || energy_opt.is_none() {
            stats.broken_entities += 1;
            error!(
                "[EntityValidator] BROKEN ENTITY: {} (Entity {:?}) has BehaviorConfig but missing stats components (H:{} T:{} E:{})",
                name,
                entity,
                if hunger_opt.is_some() { "OK" } else { "MISSING" },
                if thirst_opt.is_some() { "OK" } else { "MISSING" },
                if energy_opt.is_some() { "OK" } else { "MISSING" }
            );
        }
    }

    // Update metrics
    if stats.entities_checked > 0 {
        metrics.last_run_stats = ValidationStats {
            entities_checked: stats.entities_checked,
            components_auto_fixed: stats.components_auto_fixed,
            stuck_entities: stats.stuck_entities,
            broken_entities: stats.broken_entities,
        };
        metrics.last_run_tick = tick.0;
        metrics.total_fixed += stats.components_auto_fixed as u64;
        metrics.total_stuck_detected += stats.stuck_entities as u64;

        // Log summary if any issues were found
        if stats.components_auto_fixed > 0 || stats.stuck_entities > 0 || stats.broken_entities > 0
        {
            info!(
                "[EntityValidator] Tick {} - Checked {} entities: {} components auto-fixed, {} stuck, {} broken",
                tick.0,
                stats.entities_checked,
                stats.components_auto_fixed,
                stats.stuck_entities,
                stats.broken_entities
            );
        }
    }
}

/// Plugin to register the entity validation system
pub struct EntityValidatorPlugin;

impl Plugin for EntityValidatorPlugin {
    fn build(&self, app: &mut App) {
        // TICK-SYNCHRONIZED SYSTEMS
        // Entity validator now runs on Update schedule with tick guards
        // to ensure it only executes during simulation ticks (10 TPS)
        // Previously used FixedUpdate which runs at ~64Hz independently
        app.init_resource::<EntityValidationMetrics>().add_systems(
            Update,
            entity_validation_system.run_if(crate::ai::should_tick),
        );
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::trigger_emitters::{IdleTracker, StatThresholdTracker};
    use crate::entities::stats::{Energy, Hunger, Thirst};
    use crate::entities::{BehaviorConfig, CurrentAction};
    use crate::simulation::{SimulationState, SimulationTick};
    use bevy::prelude::*;

    /// Helper to create a test app with required resources
    fn create_test_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTick(50)) // Runs on tick 50
            .insert_resource(SimulationState { should_tick: true })
            .init_resource::<EntityValidationMetrics>()
            .add_systems(Update, entity_validation_system);
        app
    }

    #[test]
    fn test_detects_missing_idle_tracker() {
        let mut app = create_test_app();

        // Spawn entity with BehaviorConfig but NO IdleTracker
        let entity = app
            .world_mut()
            .spawn((
                BehaviorConfig::default(),
                StatThresholdTracker::default(),
                Hunger::new(),
                Thirst::new(),
                Energy::new(),
                CurrentAction::none(),
            ))
            .id();

        // Verify no IdleTracker initially
        assert!(app.world().get::<IdleTracker>(entity).is_none());

        // Run the system
        app.update();

        // Should auto-fix by adding IdleTracker
        assert!(
            app.world().get::<IdleTracker>(entity).is_some(),
            "IdleTracker should be auto-added"
        );

        // Check metrics
        let metrics = app.world().resource::<EntityValidationMetrics>();
        assert_eq!(metrics.last_run_stats.components_auto_fixed, 1);
    }

    #[test]
    fn test_detects_missing_stat_threshold_tracker() {
        let mut app = create_test_app();

        // Spawn entity with BehaviorConfig but NO StatThresholdTracker
        let entity = app
            .world_mut()
            .spawn((
                BehaviorConfig::default(),
                IdleTracker::default(),
                Hunger::new(),
                Thirst::new(),
                Energy::new(),
                CurrentAction::none(),
            ))
            .id();

        // Verify no StatThresholdTracker initially
        assert!(app.world().get::<StatThresholdTracker>(entity).is_none());

        // Run the system
        app.update();

        // Should auto-fix by adding StatThresholdTracker
        assert!(
            app.world().get::<StatThresholdTracker>(entity).is_some(),
            "StatThresholdTracker should be auto-added"
        );

        // Check metrics
        let metrics = app.world().resource::<EntityValidationMetrics>();
        assert_eq!(metrics.last_run_stats.components_auto_fixed, 1);
    }

    #[test]
    fn test_detects_stuck_entity() {
        let mut app = create_test_app();

        // Create a stuck entity: high hunger + long idle + no action
        let mut hunger = Hunger::new();
        hunger.0.set(85.0); // 85% hunger (above 80% threshold)

        let mut idle_tracker = IdleTracker::default();
        idle_tracker.ticks_since_action = 150; // Above 100 ticks threshold

        app.world_mut().spawn((
            BehaviorConfig::default(),
            StatThresholdTracker::default(),
            hunger,
            Thirst::new(),
            Energy::new(),
            idle_tracker,
            CurrentAction::none(),
        ));

        // Run the system
        app.update();

        // Check metrics - should detect stuck entity
        let metrics = app.world().resource::<EntityValidationMetrics>();
        assert_eq!(
            metrics.last_run_stats.stuck_entities, 1,
            "Should detect 1 stuck entity"
        );
    }

    #[test]
    fn test_does_not_flag_active_entity_as_stuck() {
        let mut app = create_test_app();

        // Create entity with high hunger but actively doing something
        let mut hunger = Hunger::new();
        hunger.0.set(90.0); // Very high hunger

        let mut idle_tracker = IdleTracker::default();
        idle_tracker.ticks_since_action = 5; // Recently active (low idle time)

        app.world_mut().spawn((
            BehaviorConfig::default(),
            StatThresholdTracker::default(),
            hunger,
            Thirst::new(),
            Energy::new(),
            idle_tracker,
            CurrentAction::new("Graze"), // Actively grazing
        ));

        // Run the system
        app.update();

        // Should NOT be flagged as stuck
        let metrics = app.world().resource::<EntityValidationMetrics>();
        assert_eq!(
            metrics.last_run_stats.stuck_entities, 0,
            "Active entity should not be flagged as stuck"
        );
    }

    #[test]
    fn test_validation_only_runs_every_50_ticks() {
        let mut app = App::new();
        app.insert_resource(SimulationTick(25)) // Not a multiple of 50
            .insert_resource(SimulationState { should_tick: true })
            .init_resource::<EntityValidationMetrics>()
            .add_systems(Update, entity_validation_system);

        // Spawn entity with missing components
        app.world_mut().spawn((
            BehaviorConfig::default(),
            Hunger::new(),
            Thirst::new(),
            Energy::new(),
        ));

        // Run the system at tick 25
        app.update();

        // Should NOT run at tick 25 (only runs at 0, 50, 100, etc.)
        let metrics = app.world().resource::<EntityValidationMetrics>();
        assert_eq!(
            metrics.last_run_stats.entities_checked, 0,
            "Validation should not run at tick 25"
        );
    }
}
