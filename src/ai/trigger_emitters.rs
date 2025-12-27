use crate::ai::queue::ActionQueue;
/// Event-driven trigger emitters for AI replanning
///
/// This module provides systems that monitor various game state changes and
/// emit replanning requests to the ReplanQueue when important stimuli occur.
use crate::ai::replan_queue::{ReplanPriority, ReplanQueue};
use crate::ai::ultrathink::{ThinkQueue, ThinkReason};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::BehaviorConfig;
use crate::simulation::{SimulationTick, TickProfiler};
use bevy::prelude::*;

/// Tracks previous stat states to detect threshold crossings
#[derive(Component, Debug, Clone)]
pub struct StatThresholdTracker {
    pub previous_hunger: f32,
    pub previous_thirst: f32,
    pub previous_energy: f32,
    pub hunger_triggered: bool,
    pub thirst_triggered: bool,
    pub energy_triggered: bool,
}

impl Default for StatThresholdTracker {
    fn default() -> Self {
        Self {
            previous_hunger: 0.0,
            previous_thirst: 0.0,
            previous_energy: 100.0,
            hunger_triggered: false,
            thirst_triggered: false,
            energy_triggered: false,
        }
    }
}

impl StatThresholdTracker {
    pub fn new(hunger: f32, thirst: f32, energy: f32) -> Self {
        Self {
            previous_hunger: hunger,
            previous_thirst: thirst,
            previous_energy: energy,
            hunger_triggered: false,
            thirst_triggered: false,
            energy_triggered: false,
        }
    }
}

/// Tracks entity idle time for long-idle replanning triggers
#[derive(Component, Debug, Clone)]
pub struct IdleTracker {
    pub ticks_since_action: u32,
    pub last_action_tick: u64,
    pub action_completed: bool,
}

impl Default for IdleTracker {
    fn default() -> Self {
        Self {
            ticks_since_action: 0,
            last_action_tick: 0,
            action_completed: false,
        }
    }
}

impl IdleTracker {
    pub fn new(start_tick: u64) -> Self {
        Self {
            ticks_since_action: 0,
            last_action_tick: start_tick,
            action_completed: false,
        }
    }

    pub fn mark_action_completed(&mut self, current_tick: u64) {
        self.action_completed = true;
        self.last_action_tick = current_tick;
        self.ticks_since_action = 0;
    }

    pub fn update_idle_time(&mut self, current_tick: u64) {
        if current_tick > self.last_action_tick {
            self.ticks_since_action = (current_tick - self.last_action_tick) as u32;
        }
    }

    pub fn is_long_idle(&self, config: &BehaviorConfig) -> bool {
        // Balanced idle threshold for stable AI (optimized from 3x to 5x)
        // Consider long idle as 5x the wander radius in ticks
        let idle_threshold = (config.wander_radius * 5) as u32;
        // Minimum threshold of 50 ticks (5 seconds) to prevent excessive replanning
        let min_threshold = 50u32;
        let final_threshold = idle_threshold.max(min_threshold);
        self.ticks_since_action >= final_threshold
    }
}

/// System to detect stat threshold crossings and emit normal priority replanning requests
///
/// This system monitors hunger, thirst, and energy levels and triggers replanning
/// when entities cross their configured thresholds for the first time.
pub fn stat_threshold_system(
    mut commands: Commands,
    mut replan_queue: ResMut<ReplanQueue>,
    mut think_queue: ResMut<ThinkQueue>,
    mut query: Query<(
        Entity,
        &Hunger,
        &Thirst,
        &Energy,
        &BehaviorConfig,
        Option<&mut StatThresholdTracker>,
    )>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "trigger_stats");

    for (entity, hunger, thirst, energy, behavior_config, tracker_opt) in query.iter_mut() {
        let current_hunger = hunger.0.normalized();
        let current_thirst = thirst.0.normalized();
        let current_energy = energy.0.normalized();

        // Initialize tracker if missing
        let mut tracker = if let Some(tracker) = tracker_opt {
            tracker
        } else {
            commands.entity(entity).insert(StatThresholdTracker::new(
                current_hunger,
                current_thirst,
                current_energy,
            ));
            continue;
        };

        let mut needs_replan = false;
        let mut reason = String::new();

        // Check hunger threshold crossing
        let hunger_threshold = behavior_config.hunger_threshold;
        if current_hunger >= hunger_threshold && !tracker.hunger_triggered {
            tracker.hunger_triggered = true;
            needs_replan = true;
            reason = format!(
                "Hunger threshold: {:.1}% >= {:.1}%",
                current_hunger * 100.0,
                hunger_threshold * 100.0
            );

            // ThinkQueue scheduling based on severity
            if current_hunger >= 80.0 {
                // Critical hunger (>= 80%)
                debug!("🧠 ThinkQueue: Scheduling URGENT for critical hunger: {:.1}%", current_hunger * 100.0);
                think_queue.schedule_urgent(entity, ThinkReason::HungerCritical, tick.0);
            } else if current_hunger >= 50.0 {
                // Moderate hunger (50-79%)
                debug!("🧠 ThinkQueue: Scheduling NORMAL for moderate hunger: {:.1}%", current_hunger * 100.0);
                think_queue.schedule_normal(entity, ThinkReason::HungerModerate, tick.0);
            }
        } else if current_hunger < hunger_threshold {
            tracker.hunger_triggered = false; // Reset when below threshold
        }

        // Check thirst threshold crossing
        let thirst_threshold = behavior_config.thirst_threshold;
        if current_thirst >= thirst_threshold && !tracker.thirst_triggered {
            tracker.thirst_triggered = true;
            needs_replan = true;
            reason = format!(
                "Thirst threshold: {:.1}% >= {:.1}%",
                current_thirst * 100.0,
                thirst_threshold * 100.0
            );

            // ThinkQueue scheduling based on severity
            if current_thirst >= 80.0 {
                // Critical thirst (>= 80%)
                debug!("🧠 ThinkQueue: Scheduling URGENT for critical thirst: {:.1}%", current_thirst * 100.0);
                think_queue.schedule_urgent(entity, ThinkReason::ThirstCritical, tick.0);
            } else if current_thirst >= 50.0 {
                // Moderate thirst (50-79%)
                debug!("🧠 ThinkQueue: Scheduling NORMAL for moderate thirst: {:.1}%", current_thirst * 100.0);
                think_queue.schedule_normal(entity, ThinkReason::ThirstModerate, tick.0);
            }
        } else if current_thirst < thirst_threshold {
            tracker.thirst_triggered = false; // Reset when below threshold
        }

        // Check energy threshold crossing (low energy triggers replanning)
        let energy_threshold = 0.3; // 30% energy is concerning
        if current_energy <= energy_threshold && !tracker.energy_triggered {
            tracker.energy_triggered = true;
            needs_replan = true;
            reason = format!(
                "Energy threshold: {:.1}% <= {:.1}%",
                current_energy * 100.0,
                energy_threshold * 100.0
            );

            // ThinkQueue: Low energy is moderate priority
            if current_energy <= 20.0 {
                debug!("🧠 ThinkQueue: Scheduling URGENT for critical energy: {:.1}%", current_energy * 100.0);
                think_queue.schedule_urgent(entity, ThinkReason::HungerCritical, tick.0);
            } else {
                debug!("🧠 ThinkQueue: Scheduling NORMAL for low energy: {:.1}%", current_energy * 100.0);
                think_queue.schedule_normal(entity, ThinkReason::HungerModerate, tick.0);
            }
        } else if current_energy > energy_threshold {
            tracker.energy_triggered = false; // Reset when above threshold
        }

        // Emit replanning request if needed
        if needs_replan {
            debug!("📊 Entity {:?} stat threshold trigger: {}", entity, reason);
            replan_queue.push(entity, ReplanPriority::Normal, reason, tick.0);
        }

        // Update previous values
        tracker.previous_hunger = current_hunger;
        tracker.previous_thirst = current_thirst;
        tracker.previous_energy = current_energy;
    }
}

/// System to detect fear spikes and emit high priority replanning requests
///
/// This system monitors fear states and triggers immediate replanning
/// when entities detect predators or experience fear spikes.
pub fn fear_trigger_system(
    mut replan_queue: ResMut<ReplanQueue>,
    mut think_queue: ResMut<ThinkQueue>,
    mut query: Query<(
        Entity,
        &crate::entities::FearState,
        Option<&mut IdleTracker>,
    )>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "trigger_fear");
    for (entity, fear_state, mut idle_tracker) in query.iter_mut() {
        // Check for fear spike (rapid increase in fear level)
        if fear_state.fear_level > 0.3 && fear_state.nearby_predators > 0 {
            let reason = format!(
                "Fear spike: {:.2} fear, {} predators",
                fear_state.fear_level, fear_state.nearby_predators
            );

            debug!(
                "😱 Entity {:?} high priority fear trigger: {}",
                entity, reason
            );
            replan_queue.push(entity, ReplanPriority::High, reason, tick.0);

            // ThinkQueue: Fear is always URGENT
            debug!(
                "🧠 ThinkQueue: Scheduling URGENT for fear: {:.2} fear, {} predators",
                fear_state.fear_level, fear_state.nearby_predators
            );
            think_queue.schedule_urgent(entity, ThinkReason::FearTriggered, tick.0);

            // Reset idle timer when fear triggered
            if let Some(ref mut idle) = idle_tracker {
                idle.mark_action_completed(tick.0);
            }
        }
    }
}

/// System to detect action completion/failure and emit normal priority replanning requests
///
/// This system monitors the ActionQueue and triggers replanning when entities
/// complete actions or when actions fail.
pub fn action_completion_system(
    mut replan_queue: ResMut<ReplanQueue>,
    mut think_queue: ResMut<ThinkQueue>,
    mut action_queue: ResMut<ActionQueue>,
    mut query: Query<(Entity, &mut IdleTracker)>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "trigger_actions");
    // Get entities that just completed actions from the queue
    let recently_completed = action_queue.get_recently_completed(tick.0 - 1);

    for entity in recently_completed {
        if let Ok((_, mut idle_tracker)) = query.get_mut(entity) {
            idle_tracker.mark_action_completed(tick.0);

            debug!(
                "✅ Entity {:?} action completed, triggering replanning",
                entity
            );
            replan_queue.push(
                entity,
                ReplanPriority::Normal,
                "Action completed".to_string(),
                tick.0,
            );

            // ThinkQueue: Action completion is NORMAL priority
            debug!("🧠 ThinkQueue: Scheduling NORMAL for action completion");
            think_queue.schedule_normal(entity, ThinkReason::ActionCompleted, tick.0);
        }
    }
}

/// System to detect long idle periods and emit normal priority replanning requests
///
/// This system prevents entities from getting stuck by triggering replanning
/// when they haven't performed actions for too long.
/// Modified for UltraThink: Runs every 20 ticks and schedules LOW priority think requests.
pub fn long_idle_system(
    mut replan_queue: ResMut<ReplanQueue>,
    mut think_queue: ResMut<ThinkQueue>,
    mut query: Query<(Entity, &BehaviorConfig, &mut IdleTracker)>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "trigger_idle");

    // UltraThink optimization: Only check idle every 20 ticks
    if tick.0 % 20 != 0 {
        return;
    }

    for (entity, behavior_config, mut idle_tracker) in query.iter_mut() {
        idle_tracker.update_idle_time(tick.0);

        if idle_tracker.is_long_idle(behavior_config) {
            let reason = format!(
                "Long idle: {} ticks since last action",
                idle_tracker.ticks_since_action
            );

            debug!("⏰ Entity {:?} long idle trigger: {}", entity, reason);
            replan_queue.push(entity, ReplanPriority::Normal, reason, tick.0);

            // ThinkQueue: Long idle is LOW priority (can wait)
            debug!("🧠 ThinkQueue: Scheduling LOW for long idle: {} ticks", idle_tracker.ticks_since_action);
            think_queue.schedule_low(entity, ThinkReason::Idle, tick.0);

            // Reset the timer to avoid spam
            idle_tracker.mark_action_completed(tick.0);
        }
    }
}

/// Additional fallback system for more aggressive replanning of idle entities
///
/// This system ensures that idle entities get replanned frequently even if
/// action completion detection fails, providing a safety net for the AI system.
pub fn aggressive_idle_fallback_system(
    mut replan_queue: ResMut<ReplanQueue>,
    mut think_queue: ResMut<ThinkQueue>,
    mut query: Query<(Entity, &mut IdleTracker)>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer =
        crate::simulation::profiler::ScopedTimer::new(&mut profiler, "trigger_aggressive_idle");

    // Only run every 30 ticks (3 seconds) to avoid excessive overhead
    if tick.0 % 30 != 0 {
        return;
    }

    for (entity, mut idle_tracker) in query.iter_mut() {
        // If entity has been idle for more than 30 ticks (3 seconds), trigger replanning
        if idle_tracker.ticks_since_action >= 30 {
            let reason = format!(
                "Aggressive fallback: {} ticks since last action",
                idle_tracker.ticks_since_action
            );

            debug!(
                "🔄 Entity {:?} aggressive fallback trigger: {}",
                entity, reason
            );
            replan_queue.push(entity, ReplanPriority::Normal, reason, tick.0);

            // ThinkQueue: Fallback idle is LOW priority
            debug!("🧠 ThinkQueue: Scheduling LOW for aggressive idle fallback: {} ticks", idle_tracker.ticks_since_action);
            think_queue.schedule_low(entity, ThinkReason::Idle, tick.0);

            // Reset the timer to avoid spam
            idle_tracker.mark_action_completed(tick.0);
        }
    }
}

/// Cleanup system to remove stale entries from the ReplanQueue
///
/// This system periodically cleans up the replan queue to remove entries
/// for entities that no longer exist.
pub fn replan_queue_cleanup_system(
    mut replan_queue: ResMut<ReplanQueue>,
    entities: Query<Entity>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "trigger_cleanup");
    // Create a set of all valid entities
    let valid_entities: std::collections::HashSet<Entity> = entities.iter().collect();

    // Clean up the queue using a closure that checks if entities are still valid
    replan_queue.cleanup_stale_entities(|entity| valid_entities.contains(&entity));
}

/// Plugin to register all trigger emitter systems
pub struct TriggerEmittersPlugin;

impl Plugin for TriggerEmittersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                // High priority systems (run first)
                fear_trigger_system,
                // Normal priority systems
                stat_threshold_system,
                action_completion_system,
                long_idle_system,
                aggressive_idle_fallback_system,
                // Cleanup system (run last)
                replan_queue_cleanup_system,
            )
                .chain() // Run in order
                .run_if(crate::ai::should_tick),
        );

        // Initialize trackers for existing entities
        app.add_systems(Startup, initialize_trackers);
    }
}

/// System to initialize tracking components for existing entities
fn initialize_trackers(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    entities_with_behavior: Query<(Entity, &Hunger, &Thirst, &Energy), With<BehaviorConfig>>,
) {
    for (entity, hunger, thirst, energy) in entities_with_behavior.iter() {
        // Initialize stat threshold tracker
        commands.entity(entity).insert(StatThresholdTracker::new(
            hunger.0.normalized(),
            thirst.0.normalized(),
            energy.0.normalized(),
        ));

        // Initialize idle tracker
        commands.entity(entity).insert(IdleTracker::new(tick.0));

        debug!("🔧 Initialized trigger trackers for entity {:?}", entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::replan_queue::ReplanQueue;
    use crate::entities::stats::{Energy, Hunger, Thirst};
    use crate::entities::types::rabbit::RabbitBehavior;
    use crate::simulation::{SimulationState, SimulationTick};

    #[test]
    fn test_stat_threshold_tracker() {
        let tracker = StatThresholdTracker::new(0.1, 0.1, 0.9);

        assert!(!tracker.hunger_triggered);
        assert!(!tracker.thirst_triggered);
        assert!(!tracker.energy_triggered);
    }

    #[test]
    fn test_idle_tracker() {
        let mut tracker = IdleTracker::new(100);

        assert_eq!(tracker.ticks_since_action, 0);

        tracker.update_idle_time(105);
        assert_eq!(tracker.ticks_since_action, 5);

        tracker.mark_action_completed(105);
        assert_eq!(tracker.ticks_since_action, 0);
        assert!(tracker.action_completed);
    }

    #[test]
    fn test_replan_queue_triggers() {
        let mut app = App::new();
        app.init_resource::<ReplanQueue>()
            .add_systems(Update, stat_threshold_system)
            .add_plugins(TriggerEmittersPlugin);

        // This test would need more setup with actual entities
        // For now, just ensure the systems compile
    }
}
