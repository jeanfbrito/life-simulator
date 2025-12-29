/// AI Watchdog System
///
/// A critical safety system that prevents entities from dying due to stuck AI.
/// Monitors entities in critical states (high hunger/thirst + long idle) and applies
/// escalating interventions to force replanning.
///
/// Intervention Levels:
/// - Level 1 (Replan): Schedule urgent think request via ThinkQueue
/// - Level 2 (Cancel): Cancel current action + force replan
/// - Level 3 (Emergency): Remove ActiveAction component + insert NeedsReplanning + reset IdleTracker
use bevy::prelude::*;

use crate::ai::event_driven_planner::NeedsReplanning;
use crate::ai::queue::ActionQueue;
use crate::ai::trigger_emitters::IdleTracker;
use crate::ai::ultrathink::{ThinkQueue, ThinkReason};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{ActiveAction, BehaviorConfig, Creature, CurrentAction};
use crate::simulation::SimulationTick;

// ============================================================================
// COMPONENTS AND RESOURCES
// ============================================================================

/// Intervention levels for escalation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterventionLevel {
    #[default]
    None,
    Level1Replan,
    Level2Cancel,
    Level3Emergency,
}

impl InterventionLevel {
    /// Get the next escalation level
    pub fn escalate(&self) -> Self {
        match self {
            InterventionLevel::None => InterventionLevel::Level1Replan,
            InterventionLevel::Level1Replan => InterventionLevel::Level2Cancel,
            InterventionLevel::Level2Cancel => InterventionLevel::Level3Emergency,
            InterventionLevel::Level3Emergency => InterventionLevel::Level3Emergency, // Max level
        }
    }
}

/// Tracks intervention history per entity to prevent thrashing
#[derive(Component, Debug, Clone, Default)]
pub struct WatchdogHistory {
    pub last_intervention_tick: u64,
    pub last_emergency_tick: u64,
    pub intervention_count: u32,
    pub last_level: InterventionLevel,
}

impl WatchdogHistory {
    /// Check if we're in cooldown period
    pub fn in_cooldown(&self, current_tick: u64, cooldown: u32) -> bool {
        current_tick < self.last_intervention_tick + cooldown as u64
    }

    /// Check if we should escalate (previous intervention didn't work)
    pub fn should_escalate(&self, current_tick: u64, escalation_wait: u32) -> bool {
        current_tick >= self.last_intervention_tick + escalation_wait as u64
            && self.last_level != InterventionLevel::None
    }

    /// Record an intervention
    pub fn record_intervention(&mut self, tick: u64, level: InterventionLevel) {
        self.last_intervention_tick = tick;
        self.intervention_count += 1;
        self.last_level = level;

        if level == InterventionLevel::Level3Emergency {
            self.last_emergency_tick = tick;
        }
    }

    /// Reset after entity recovers (hunger/thirst below critical)
    pub fn reset(&mut self) {
        self.last_level = InterventionLevel::None;
        // Keep intervention_count for statistics
    }
}

/// Metrics for monitoring
#[derive(Resource, Debug, Default)]
pub struct WatchdogMetrics {
    pub level1_count: u64,
    pub level2_count: u64,
    pub level3_count: u64,
    pub throttled_count: u64,
    pub last_run_tick: u64,
}

impl WatchdogMetrics {
    pub fn record_intervention(&mut self, level: InterventionLevel) {
        match level {
            InterventionLevel::Level1Replan => self.level1_count += 1,
            InterventionLevel::Level2Cancel => self.level2_count += 1,
            InterventionLevel::Level3Emergency => self.level3_count += 1,
            InterventionLevel::None => {}
        }
    }

    pub fn record_throttled(&mut self) {
        self.throttled_count += 1;
    }
}

/// Configuration for the watchdog system
#[derive(Resource, Clone)]
pub struct WatchdogConfig {
    /// Critical hunger threshold (0.0-1.0, default 0.85 = 85%)
    pub critical_hunger: f32,
    /// Critical thirst threshold (0.0-1.0, default 0.85 = 85%)
    pub critical_thirst: f32,
    /// Critical idle ticks threshold (default 100 ticks = 10 seconds at 10 TPS)
    pub critical_idle_ticks: u32,
    /// Cooldown between interventions (default 50 ticks = 5 seconds)
    pub intervention_cooldown: u32,
    /// Wait time before escalating (default 30 ticks = 3 seconds)
    pub escalation_wait: u32,
    /// How often to run the watchdog (default 10 ticks = 1 second)
    pub run_interval: u64,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            critical_hunger: 0.85,
            critical_thirst: 0.85,
            critical_idle_ticks: 100,
            intervention_cooldown: 50,
            escalation_wait: 30,
            run_interval: 10,
        }
    }
}

// ============================================================================
// MAIN WATCHDOG SYSTEM
// ============================================================================

/// Watchdog system that monitors entities for stuck AI and applies interventions
///
/// Runs every N ticks (configured via WatchdogConfig::run_interval)
/// Detects critical state: hunger >= 85% OR thirst >= 85% AND idle >= 100 ticks
/// Applies escalating interventions to prevent entity death
pub fn watchdog_system(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    config: Res<WatchdogConfig>,
    mut metrics: ResMut<WatchdogMetrics>,
    mut think_queue: ResMut<ThinkQueue>,
    mut action_queue: ResMut<ActionQueue>,
    mut query: Query<(
        Entity,
        &BehaviorConfig,
        &Hunger,
        &Thirst,
        &Energy,
        &mut IdleTracker,
        Option<&mut WatchdogHistory>,
        Option<&Creature>,
        Option<&ActiveAction>,
    )>,
) {
    // Only run every N ticks
    if tick.0 % config.run_interval != 0 {
        return;
    }

    metrics.last_run_tick = tick.0;

    for (
        entity,
        _behavior_config,
        hunger,
        thirst,
        _energy,
        mut idle_tracker,
        history_opt,
        creature_opt,
        active_action_opt,
    ) in query.iter_mut()
    {
        let hunger_norm = hunger.0.normalized();
        let thirst_norm = thirst.0.normalized();
        let idle_ticks = idle_tracker.ticks_since_action;
        let name = creature_opt.map(|c| c.name.as_str()).unwrap_or("Unknown");

        // Check if in critical state: (hunger >= 85% OR thirst >= 85%) AND idle >= 100 ticks
        let is_critical = (hunger_norm >= config.critical_hunger
            || thirst_norm >= config.critical_thirst)
            && idle_ticks >= config.critical_idle_ticks;

        if !is_critical {
            // Entity is not in critical state - reset history if we have one
            if let Some(mut history) = history_opt {
                if history.last_level != InterventionLevel::None {
                    debug!(
                        "[Watchdog] {} recovered from critical state, resetting history",
                        name
                    );
                    history.reset();
                }
            }
            continue;
        }

        // Entity is in critical state - determine intervention level
        let history = if let Some(history) = history_opt {
            history.into_inner()
        } else {
            // Add WatchdogHistory component if missing
            commands.entity(entity).insert(WatchdogHistory::default());
            continue; // Will process on next run with history component
        };

        // Check cooldown
        if history.in_cooldown(tick.0, config.intervention_cooldown) {
            metrics.record_throttled();
            debug!(
                "[Watchdog] {} in cooldown (tick {}, last intervention tick {})",
                name, tick.0, history.last_intervention_tick
            );
            continue;
        }

        // Determine intervention level
        let level = if history.should_escalate(tick.0, config.escalation_wait) {
            history.last_level.escalate()
        } else if history.last_level == InterventionLevel::None {
            InterventionLevel::Level1Replan
        } else {
            // Still in escalation wait period, skip
            continue;
        };

        // Apply intervention
        match level {
            InterventionLevel::Level1Replan => {
                // Schedule urgent think request
                warn!(
                    "[Watchdog] LEVEL 1 intervention for {} (H:{:.1}% T:{:.1}% Idle:{} ticks) - scheduling urgent replan",
                    name,
                    hunger_norm * 100.0,
                    thirst_norm * 100.0,
                    idle_ticks
                );

                think_queue.schedule_urgent(entity, ThinkReason::WatchdogIntervention, tick.0);
            }

            InterventionLevel::Level2Cancel => {
                // Cancel action + force replan
                warn!(
                    "[Watchdog] LEVEL 2 intervention for {} (H:{:.1}% T:{:.1}% Idle:{} ticks) - cancelling action + replan",
                    name,
                    hunger_norm * 100.0,
                    thirst_norm * 100.0,
                    idle_ticks
                );

                action_queue.schedule_cancellation(entity);
                think_queue.schedule_urgent(entity, ThinkReason::WatchdogIntervention, tick.0);
            }

            InterventionLevel::Level3Emergency => {
                // Emergency intervention: remove ActiveAction, insert NeedsReplanning, reset IdleTracker
                error!(
                    "[Watchdog] LEVEL 3 EMERGENCY for {} (H:{:.1}% T:{:.1}% Idle:{} ticks) - forcing complete reset",
                    name,
                    hunger_norm * 100.0,
                    thirst_norm * 100.0,
                    idle_ticks
                );

                // Remove ActiveAction component if present
                if active_action_opt.is_some() {
                    commands.entity(entity).remove::<ActiveAction>();
                    commands.entity(entity).insert(CurrentAction::none());
                }

                // Insert NeedsReplanning directly (bypass queue)
                commands.entity(entity).insert(NeedsReplanning {
                    reason: format!(
                        "Watchdog EMERGENCY: H:{:.1}% T:{:.1}% Idle:{}",
                        hunger_norm * 100.0,
                        thirst_norm * 100.0,
                        idle_ticks
                    ),
                });

                // Reset idle tracker
                idle_tracker.mark_action_completed(tick.0);
            }

            InterventionLevel::None => unreachable!(),
        }

        // Record intervention
        history.record_intervention(tick.0, level);
        metrics.record_intervention(level);
    }

    // Log metrics every 100 ticks
    if tick.0 % 100 == 0
        && (metrics.level1_count > 0 || metrics.level2_count > 0 || metrics.level3_count > 0)
    {
        info!(
            "[Watchdog] Metrics @ tick {}: L1:{} L2:{} L3:{} Throttled:{}",
            tick.0, metrics.level1_count, metrics.level2_count, metrics.level3_count, metrics.throttled_count
        );
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

/// Plugin that sets up the AI Watchdog system
pub struct WatchdogPlugin;

impl Plugin for WatchdogPlugin {
    fn build(&self, app: &mut App) {
        // TICK-SYNCHRONIZED SYSTEMS
        // Watchdog now runs on Update schedule with tick guards
        // to ensure it only executes during simulation ticks (10 TPS)
        // Previously used FixedUpdate which runs at ~64Hz independently
        app.init_resource::<WatchdogConfig>()
            .init_resource::<WatchdogMetrics>()
            .add_systems(
                Update,
                watchdog_system
                    .after(crate::ai::entity_validator::entity_validation_system)
                    .run_if(should_tick),
            );

        info!("[Watchdog] Plugin initialized - monitoring for stuck AI");
    }
}

/// Run condition that checks if a tick should happen
fn should_tick(state: Res<crate::simulation::SimulationState>) -> bool {
    state.should_tick
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intervention_level_escalation() {
        let level = InterventionLevel::None;
        assert_eq!(level.escalate(), InterventionLevel::Level1Replan);

        let level = InterventionLevel::Level1Replan;
        assert_eq!(level.escalate(), InterventionLevel::Level2Cancel);

        let level = InterventionLevel::Level2Cancel;
        assert_eq!(level.escalate(), InterventionLevel::Level3Emergency);

        let level = InterventionLevel::Level3Emergency;
        assert_eq!(level.escalate(), InterventionLevel::Level3Emergency); // Max
    }

    #[test]
    fn test_watchdog_history_cooldown() {
        let mut history = WatchdogHistory::default();
        history.record_intervention(100, InterventionLevel::Level1Replan);

        // During cooldown (50 ticks default)
        assert!(history.in_cooldown(120, 50));
        assert!(history.in_cooldown(149, 50));

        // After cooldown
        assert!(!history.in_cooldown(150, 50));
        assert!(!history.in_cooldown(200, 50));
    }

    #[test]
    fn test_watchdog_history_escalation() {
        let mut history = WatchdogHistory::default();
        history.record_intervention(100, InterventionLevel::Level1Replan);

        // Before escalation wait (30 ticks default)
        assert!(!history.should_escalate(120, 30));

        // After escalation wait
        assert!(history.should_escalate(130, 30));
        assert!(history.should_escalate(200, 30));
    }

    #[test]
    fn test_watchdog_history_reset() {
        let mut history = WatchdogHistory::default();
        history.record_intervention(100, InterventionLevel::Level2Cancel);
        assert_eq!(history.last_level, InterventionLevel::Level2Cancel);
        assert_eq!(history.intervention_count, 1);

        history.reset();
        assert_eq!(history.last_level, InterventionLevel::None);
        assert_eq!(history.intervention_count, 1); // Count preserved for stats
    }

    #[test]
    fn test_watchdog_config_defaults() {
        let config = WatchdogConfig::default();
        assert_eq!(config.critical_hunger, 0.85);
        assert_eq!(config.critical_thirst, 0.85);
        assert_eq!(config.critical_idle_ticks, 100);
        assert_eq!(config.intervention_cooldown, 50);
        assert_eq!(config.escalation_wait, 30);
        assert_eq!(config.run_interval, 10);
    }

    #[test]
    fn test_watchdog_metrics_recording() {
        let mut metrics = WatchdogMetrics::default();

        metrics.record_intervention(InterventionLevel::Level1Replan);
        assert_eq!(metrics.level1_count, 1);

        metrics.record_intervention(InterventionLevel::Level2Cancel);
        assert_eq!(metrics.level2_count, 1);

        metrics.record_intervention(InterventionLevel::Level3Emergency);
        assert_eq!(metrics.level3_count, 1);

        metrics.record_throttled();
        assert_eq!(metrics.throttled_count, 1);
    }
}
