/// Think Queue Implementation for UltraThink System
///
/// Phase 5 Complete: UltraThink now invokes actual AI planning by marking entities
/// with NeedsReplanning components. The event_driven_planner_system picks up these
/// markers and executes the full planning pipeline.
///
/// This provides intelligent budget-controlled thinking where urgent stimuli (fear,
/// critical needs) get processed within 1-2 ticks, while normal activities can wait
/// in queue based on available budget.
use super::request::{ThinkPriority, ThinkReason, ThinkRequest};
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

/// Global queue of thinking tasks with priority-based processing
#[derive(Resource)]
pub struct ThinkQueue {
    /// High priority queue - urgent thoughts (fear, critical hunger/thirst)
    urgent_queue: VecDeque<ThinkRequest>,

    /// Medium priority queue - normal activities (moderate needs, action completion)
    normal_queue: VecDeque<ThinkRequest>,

    /// Low priority queue - idle activities (wandering, exploration)
    low_queue: VecDeque<ThinkRequest>,

    /// Processing budget per tick
    pub thinks_per_tick: usize,

    /// Track which entities are already queued to prevent duplicates
    queued_entities: HashSet<Entity>,

    /// Statistics
    total_thoughts_processed: u64,
}

impl Default for ThinkQueue {
    fn default() -> Self {
        Self::new(50) // Default to 50 thinks per tick
    }
}

impl ThinkQueue {
    /// Create a new think queue with specified budget
    pub fn new(thinks_per_tick: usize) -> Self {
        Self {
            urgent_queue: VecDeque::new(),
            normal_queue: VecDeque::new(),
            low_queue: VecDeque::new(),
            thinks_per_tick,
            queued_entities: HashSet::new(),
            total_thoughts_processed: 0,
        }
    }

    /// Schedule an urgent think request (processed within 1-2 ticks)
    pub fn schedule_urgent(&mut self, entity: Entity, reason: ThinkReason, tick: u64) {
        if self.queued_entities.insert(entity) {
            let request = ThinkRequest::new_with_priority(entity, reason, tick, ThinkPriority::Urgent);
            self.urgent_queue.push_back(request);
        }
    }

    /// Schedule a normal priority think request (processed within 5-10 ticks)
    pub fn schedule_normal(&mut self, entity: Entity, reason: ThinkReason, tick: u64) {
        if self.queued_entities.insert(entity) {
            let request = ThinkRequest::new_with_priority(entity, reason, tick, ThinkPriority::Normal);
            self.normal_queue.push_back(request);
        }
    }

    /// Schedule a low priority think request (processed within 20-50 ticks)
    pub fn schedule_low(&mut self, entity: Entity, reason: ThinkReason, tick: u64) {
        if self.queued_entities.insert(entity) {
            let request = ThinkRequest::new_with_priority(entity, reason, tick, ThinkPriority::Low);
            self.low_queue.push_back(request);
        }
    }

    /// Drain up to N requests from queues, prioritizing urgent → normal → low
    pub fn drain(&mut self, max_count: usize) -> Vec<ThinkRequest> {
        let mut requests = Vec::with_capacity(max_count);
        let mut processed = 0;

        // Process urgent queue first
        while processed < max_count {
            if let Some(request) = self.urgent_queue.pop_front() {
                self.queued_entities.remove(&request.entity);
                requests.push(request);
                processed += 1;
            } else {
                break;
            }
        }

        // Then normal queue
        while processed < max_count {
            if let Some(request) = self.normal_queue.pop_front() {
                self.queued_entities.remove(&request.entity);
                requests.push(request);
                processed += 1;
            } else {
                break;
            }
        }

        // Finally low queue
        while processed < max_count {
            if let Some(request) = self.low_queue.pop_front() {
                self.queued_entities.remove(&request.entity);
                requests.push(request);
                processed += 1;
            } else {
                break;
            }
        }

        self.total_thoughts_processed += requests.len() as u64;
        requests
    }

    /// Check if an entity is already queued
    pub fn contains(&self, entity: Entity) -> bool {
        self.queued_entities.contains(&entity)
    }

    /// Get queue sizes for monitoring (urgent, normal, low)
    pub fn queue_sizes(&self) -> (usize, usize, usize) {
        (
            self.urgent_queue.len(),
            self.normal_queue.len(),
            self.low_queue.len(),
        )
    }

    /// Get total queue depth
    pub fn total_queued(&self) -> usize {
        self.urgent_queue.len() + self.normal_queue.len() + self.low_queue.len()
    }

    /// Get statistics
    pub fn total_processed(&self) -> u64 {
        self.total_thoughts_processed
    }
}

/// System that processes think requests from the queue
///
/// Phase 5: Actually invokes AI planning by marking entities with NeedsReplanning.
/// The event_driven_planner_system picks up these markers and runs the actual planning logic.
pub fn ultrathink_system(
    mut commands: Commands,
    mut think_queue: ResMut<ThinkQueue>,
    tick: Res<crate::simulation::SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    let budget = think_queue.thinks_per_tick;
    let requests = think_queue.drain(budget);

    if requests.is_empty() {
        return;
    }

    let _timer = crate::simulation::profiler::ScopedTimer::new(&mut profiler, "ultrathink_process");

    let mut processed_count = 0;
    let mut skipped_count = 0;

    // Process each think request by marking entity for replanning
    for request in &requests {
        // Try to mark entity - this will safely handle despawned entities
        if let Ok(mut entity_cmd) = commands.get_entity(request.entity) {
            entity_cmd.insert(crate::ai::event_driven_planner::NeedsReplanning {
                reason: format!("UltraThink: {} (priority: {:?})", request.reason, request.priority),
            });

            processed_count += 1;

            debug!(
                "🧠 Processing think request: entity={:?}, reason={}, priority={:?}, wait_time={} ticks",
                request.entity,
                request.reason,
                request.priority,
                tick.0.saturating_sub(request.scheduled_tick)
            );
        } else {
            skipped_count += 1;
            debug!(
                "⚠️ Skipped think request for despawned entity {:?}, reason: {}",
                request.entity,
                request.reason
            );
        }
    }

    // Log processing summary if we had any skipped entities
    if skipped_count > 0 {
        debug!(
            "🧠 UltraThink tick {}: processed {}, skipped {} (despawned)",
            tick.0, processed_count, skipped_count
        );
    }

    // Log metrics every 50 ticks
    if tick.0 % 50 == 0 && think_queue.total_queued() > 0 {
        let (urgent, normal, low) = think_queue.queue_sizes();
        info!(
            "🧠 ThinkQueue depth: {} urgent, {} normal, {} low | Processed {}/{} | Total processed: {}",
            urgent,
            normal,
            low,
            requests.len(),
            budget,
            think_queue.total_processed()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_creation() {
        let queue = ThinkQueue::new(50);
        assert_eq!(queue.thinks_per_tick, 50);
        assert_eq!(queue.total_queued(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut queue = ThinkQueue::new(10);
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        // Add in reverse priority
        queue.schedule_low(entity3, ThinkReason::Idle, 1);
        queue.schedule_urgent(entity1, ThinkReason::FearTriggered, 1);
        queue.schedule_normal(entity2, ThinkReason::ActionCompleted, 1);

        let requests = queue.drain(3);

        assert_eq!(requests.len(), 3);
        assert!(matches!(requests[0].priority, ThinkPriority::Urgent));
        assert!(matches!(requests[1].priority, ThinkPriority::Normal));
        assert!(matches!(requests[2].priority, ThinkPriority::Low));
    }
}
