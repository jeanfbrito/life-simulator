/// Event-driven replan queue for the AI system
///
/// This resource manages entities that need to be replanned due to
/// important stimuli (fear, hunger, thirst, etc.) with priority lanes.
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

/// Priority levels for replanning requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReplanPriority {
    /// High priority: Fear spikes, predator proximity, combat damage, panic effects
    High = 1,
    /// Normal priority: Stat thresholds, action completion, idle timers
    Normal = 0,
}

/// A replanning request for an entity
#[derive(Debug, Clone)]
pub struct ReplanRequest {
    pub entity: Entity,
    pub priority: ReplanPriority,
    pub reason: String,
    pub tick: u64,
}

/// Event-driven replan queue with priority lanes and deduplication
#[derive(Resource, Debug, Default)]
pub struct ReplanQueue {
    /// High priority requests (fear, combat, etc.)
    high_priority: VecDeque<ReplanRequest>,
    /// Normal priority requests (hunger, thirst, etc.)
    normal_priority: VecDeque<ReplanRequest>,
    /// Set of entities already in queue to prevent duplicates
    dedupe_set: HashSet<Entity>,
}

impl ReplanQueue {
    /// Create a new empty replan queue
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entity to the replan queue with specified priority
    /// Returns true if the entity was added, false if already in queue
    pub fn push(
        &mut self,
        entity: Entity,
        priority: ReplanPriority,
        reason: String,
        tick: u64,
    ) -> bool {
        // Check if entity is already in queue
        if self.dedupe_set.contains(&entity) {
            return false;
        }

        // Note: We don't check if entity is alive here because Entity doesn't have is_alive()
        // The cleanup_stale_entities method should be called periodically to remove despawned entities
        self.dedupe_set.insert(entity);
        let request = ReplanRequest {
            entity,
            priority,
            reason,
            tick,
        };

        match priority {
            ReplanPriority::High => self.high_priority.push_back(request),
            ReplanPriority::Normal => self.normal_priority.push_back(request),
        }
        true
    }

    /// Drain up to max_count requests from the queue, respecting priority order
    /// High priority requests are drained first, then normal priority
    pub fn drain(&mut self, max_count: usize) -> Vec<ReplanRequest> {
        let mut result = Vec::with_capacity(max_count);
        let mut remaining = max_count;

        // Drain high priority first
        while remaining > 0 && !self.high_priority.is_empty() {
            if let Some(request) = self.high_priority.pop_front() {
                self.dedupe_set.remove(&request.entity);
                result.push(request);
                remaining -= 1;
            } else {
                break;
            }
        }

        // Then drain normal priority
        while remaining > 0 && !self.normal_priority.is_empty() {
            if let Some(request) = self.normal_priority.pop_front() {
                self.dedupe_set.remove(&request.entity);
                result.push(request);
                remaining -= 1;
            } else {
                break;
            }
        }

        result
    }

    /// Get the number of entities waiting in each priority lane
    pub fn queue_sizes(&self) -> (usize, usize) {
        (self.high_priority.len(), self.normal_priority.len())
    }

    /// Check if an entity is already in the queue
    pub fn contains(&self, entity: Entity) -> bool {
        self.dedupe_set.contains(&entity)
    }

    /// Remove stale entries for entities that no longer exist
    /// Note: In Bevy, we can't directly check if entities are alive without world access
    /// This method should be called from a system that has World access
    pub fn cleanup_stale_entities<F>(&mut self, mut is_alive_fn: F)
    where
        F: FnMut(Entity) -> bool,
    {
        // Clean high priority lane
        self.high_priority.retain(|request| {
            if is_alive_fn(request.entity) {
                true
            } else {
                self.dedupe_set.remove(&request.entity);
                false
            }
        });

        // Clean normal priority lane
        self.normal_priority.retain(|request| {
            if is_alive_fn(request.entity) {
                true
            } else {
                self.dedupe_set.remove(&request.entity);
                false
            }
        });
    }

    /// Clear all entries from the queue (useful for testing)
    pub fn clear(&mut self) {
        self.high_priority.clear();
        self.normal_priority.clear();
        self.dedupe_set.clear();
    }

    /// Get all requests in the queue (for debugging)
    pub fn debug_snapshot(&self) -> Vec<&ReplanRequest> {
        let mut result: Vec<&ReplanRequest> = self.high_priority.iter().collect();
        result.extend(self.normal_priority.iter());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut queue = ReplanQueue::new();
        let world = &mut World::new();
        let entity = world.spawn_empty().id();

        // First push should succeed
        assert!(queue.push(entity, ReplanPriority::Normal, "test".to_string(), 0));
        assert_eq!(queue.queue_sizes(), (0, 1));

        // Second push should fail due to deduplication
        assert!(!queue.push(entity, ReplanPriority::High, "test2".to_string(), 1));
        assert_eq!(queue.queue_sizes(), (0, 1));

        // Drain should remove the entity
        let drained = queue.drain(5);
        assert_eq!(drained.len(), 1);
        assert_eq!(queue.queue_sizes(), (0, 0));

        // Now we can push again
        assert!(queue.push(entity, ReplanPriority::High, "test3".to_string(), 2));
        assert_eq!(queue.queue_sizes(), (1, 0));
    }

    #[test]
    fn test_priority_ordering() {
        let mut queue = ReplanQueue::new();
        let world = &mut World::new();

        let entity1 = world.spawn_empty().id();
        let entity2 = world.spawn_empty().id();
        let entity3 = world.spawn_empty().id();

        // Add normal priority first
        queue.push(entity1, ReplanPriority::Normal, "normal1".to_string(), 0);

        // Add high priority
        queue.push(entity2, ReplanPriority::High, "high1".to_string(), 1);

        // Add another normal
        queue.push(entity3, ReplanPriority::Normal, "normal2".to_string(), 2);

        // Drain should return high priority first
        let drained = queue.drain(3);
        assert_eq!(drained.len(), 3);

        // High priority should be first
        assert_eq!(drained[0].entity, entity2);
        assert_eq!(drained[0].priority, ReplanPriority::High);

        // Normal priorities should follow in insertion order
        assert_eq!(drained[1].entity, entity1);
        assert_eq!(drained[2].entity, entity3);
    }

    #[test]
    fn test_drain_budget() {
        let mut queue = ReplanQueue::new();
        let world = &mut World::new();

        let entities: Vec<Entity> = (0..5).map(|_| world.spawn_empty().id()).collect();

        // Add 5 entities
        for (i, entity) in entities.iter().enumerate() {
            queue.push(
                *entity,
                ReplanPriority::Normal,
                format!("entity_{}", i),
                i as u64,
            );
        }

        // Drain with budget of 3
        let drained = queue.drain(3);
        assert_eq!(drained.len(), 3);
        assert_eq!(queue.queue_sizes(), (0, 2));

        // Drain remaining
        let remaining = queue.drain(10);
        assert_eq!(remaining.len(), 2);
        assert_eq!(queue.queue_sizes(), (0, 0));
    }

    #[test]
    fn test_cleanup_stale_entities() {
        let mut queue = ReplanQueue::new();
        let world = &mut World::new();

        let entity1 = world.spawn_empty().id();
        let entity2 = world.spawn_empty().id();

        queue.push(entity1, ReplanPriority::High, "high".to_string(), 0);
        queue.push(entity2, ReplanPriority::Normal, "normal".to_string(), 1);

        assert_eq!(queue.queue_sizes(), (1, 1));

        // Despawn entity1
        world.entity_mut(entity1).despawn();

        // Cleanup should remove despawned entity
        queue.cleanup_stale_entities(|entity| world.get_entity(entity).is_ok());
        assert_eq!(queue.queue_sizes(), (0, 1));

        // Should still contain entity2
        assert!(queue.contains(entity2));
        assert!(!queue.contains(entity1));
    }
}
