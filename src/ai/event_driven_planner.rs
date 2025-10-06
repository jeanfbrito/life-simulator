use crate::ai::queue::ActionQueue;
/// Event-driven planner drain system
///
/// This module implements the tick-scheduled planner that drains the ReplanQueue
/// and runs planning for entities that need replanning due to important stimuli.
use crate::ai::replan_queue::{ReplanPriority, ReplanQueue};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::BehaviorConfig;
use crate::simulation::{SimulationTick, TickProfiler};
use bevy::prelude::*;

/// Per-tick budget for replanning to prevent starvation under heavy load
/// Limits how many entities can be replanned per tick
const REPLAN_BUDGET_PER_TICK: usize = 10;

/// System that drains the ReplanQueue and triggers replanning for entities
///
/// This system runs on every tick and processes entities that need replanning,
/// respecting priority order and per-tick budget constraints.
pub fn event_driven_planner_system(
    mut commands: Commands,
    mut replan_queue: ResMut<ReplanQueue>,
    mut action_queue: ResMut<ActionQueue>,
    tick: Res<SimulationTick>,
    query: Query<(Entity, &BehaviorConfig, &Hunger, &Thirst, &Energy)>,
    mut profiler: ResMut<TickProfiler>,
) {
    // Only process if there are entities in the replan queue
    let queue_sizes = replan_queue.queue_sizes();
    if queue_sizes.0 == 0 && queue_sizes.1 == 0 {
        return;
    }

    // Drain up to budget entries from the replan queue
    let replan_requests = replan_queue.drain(REPLAN_BUDGET_PER_TICK);

    if replan_requests.is_empty() {
        return;
    }

    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "ai_event_replan");

    let start_time = std::time::Instant::now();
    let mut processed_high = 0;
    let mut processed_normal = 0;

    // Process each replan request
    for request in replan_requests {
        let entity = request.entity;
        let priority = request.priority;
        let reason = request.reason.clone();

        // Check if entity still exists and has required components
        let can_replan = query.get(entity).is_ok();

        if !can_replan {
            debug!(
                "🚫 Entity {:?} no longer has required components, skipping replan: {}",
                entity, reason
            );
            continue;
        }

        if action_queue.has_action(entity) {
            action_queue.schedule_cancellation(entity);
        }

        // Run the planner for this entity (mark as needing replanning)
        commands.entity(entity).insert(NeedsReplanning {
            reason: reason.clone(),
        });

        // Track processing statistics
        match priority {
            ReplanPriority::High => processed_high += 1,
            ReplanPriority::Normal => processed_normal += 1,
        }

        debug!(
            "🧠 Entity {:?} replanned due to {:?}: {}",
            entity, priority, reason
        );
    }

    // Log performance metrics
    let duration = start_time.elapsed();
    if duration.as_millis() > 5 {
        // Only log if it took more than 5ms
        info!(
            "📊 Event-driven planner tick {}: processed {} high, {} normal priority in {:?}ms",
            tick.0,
            processed_high,
            processed_normal,
            duration.as_millis()
        );
    }
}

/// Marker component for entities that need replanning
#[derive(Component, Debug)]
pub struct NeedsReplanning {
    pub reason: String,
}

/// System that clears the NeedsReplanning marker after planning is complete
pub fn cleanup_replanning_markers(
    mut commands: Commands,
    query: Query<(Entity, &NeedsReplanning)>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<TickProfiler>,
) {
    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "ai_replan_cleanup");

    for (entity, needs_replan) in query.iter() {
        // Remove the marker after one tick (planning should have happened)
        commands.entity(entity).remove::<NeedsReplanning>();

        if tick.0 % 10 == 0 {
            debug!(
                "✅ Completed replanning for entity {:?}: {}",
                entity, needs_replan.reason
            );
        }
    }
}

/// Plugin to register the event-driven planner systems
pub struct EventDrivenPlannerPlugin;

impl Plugin for EventDrivenPlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                event_driven_planner_system,
                crate::entities::types::rabbit::plan_rabbit_actions,
                crate::entities::types::deer::plan_deer_actions,
                crate::entities::types::raccoon::plan_raccoon_actions,
                cleanup_replanning_markers,
            )
                .chain()
                .run_if(crate::ai::should_tick)
                .run_if(resource_exists::<crate::world_loader::WorldLoader>)
                .run_if(resource_exists::<crate::vegetation::ResourceGrid>),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::replan_queue::{ReplanPriority, ReplanQueue};

    #[test]
    fn test_replan_budget() {
        assert!(REPLAN_BUDGET_PER_TICK > 0);
        assert!(REPLAN_BUDGET_PER_TICK <= 20); // Reasonable upper bound
    }

    #[test]
    fn test_needs_replanning_component() {
        let marker = NeedsReplanning {
            reason: "test trigger".to_string(),
        };

        assert_eq!(marker.reason, "test trigger");
    }

    #[test]
    fn test_replan_queue_drain() {
        let mut queue = ReplanQueue::new();
        let mut world = World::new();
        let entity_high = world.spawn_empty().id();
        let entity_normal = world.spawn_empty().id();

        // Add some requests with distinct entities so dedupe doesn't collapse them
        queue.push(entity_high, ReplanPriority::High, "test1".to_string(), 1);
        queue.push(
            entity_normal,
            ReplanPriority::Normal,
            "test2".to_string(),
            1,
        );

        // Test draining with budget
        let drained = queue.drain(1);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].priority, ReplanPriority::High); // High priority first

        // Drain remaining
        let remaining = queue.drain(10);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].priority, ReplanPriority::Normal);
    }
}
