use crate::ai::queue::ActionQueue;
/// Event-driven trigger emitters for AI replanning
///
/// This module provides systems that monitor various game state changes and
/// emit replanning requests to the ReplanQueue when important stimuli occur.
use crate::ai::replan_queue::{ReplanPriority, ReplanQueue};
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
        // Consider long idle as 10x the wander radius in ticks
        let idle_threshold = (config.wander_radius * 10) as u32;
        self.ticks_since_action >= idle_threshold
    }
}

/// System to detect stat threshold crossings and emit normal priority replanning requests
///
/// This system monitors hunger, thirst, and energy levels and triggers replanning
/// when entities cross their configured thresholds for the first time.
pub fn stat_threshold_system(
    mut commands: Commands,
    mut replan_queue: ResMut<ReplanQueue>,
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
        }
    }
}

/// System to detect long idle periods and emit normal priority replanning requests
///
/// This system prevents entities from getting stuck by triggering replanning
/// when they haven't performed actions for too long.
pub fn long_idle_system(
    mut replan_queue: ResMut<ReplanQueue>,
    mut query: Query<(Entity, &BehaviorConfig, &mut IdleTracker)>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "trigger_idle");
    for (entity, behavior_config, mut idle_tracker) in query.iter_mut() {
        idle_tracker.update_idle_time(tick.0);

        if idle_tracker.is_long_idle(behavior_config) {
            let reason = format!(
                "Long idle: {} ticks since last action",
                idle_tracker.ticks_since_action
            );

            debug!("⏰ Entity {:?} long idle trigger: {}", entity, reason);
            replan_queue.push(entity, ReplanPriority::Normal, reason, tick.0);

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
            Update,
            (
                // High priority systems (run first)
                fear_trigger_system,
                // Normal priority systems
                stat_threshold_system,
                action_completion_system,
                long_idle_system,
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
