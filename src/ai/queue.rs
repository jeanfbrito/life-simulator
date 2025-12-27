/// Action queue for TQUAI system
///
/// Manages queued actions with priorities, executing them synchronously on ticks.
/// Handles multi-tick actions that span across multiple ticks.
///
/// DEAD ENTITY MANAGEMENT:
/// Dead entities are automatically skipped during action execution via validation checks.
/// Additionally, the queue periodically cleans up accumulated dead entity references
/// (every 100 ticks) to prevent HashMap iteration slowdown. This prevents memory
/// accumulation when entities are despawned but their references remain in queue
/// data structures (active, pending, recently_completed, pending_cancellations).
use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use super::action::{create_action, Action, ActionResult, ActionType};
use crate::entities::{ActiveAction, CurrentAction};
use crate::simulation::SimulationTick;

/// A queued action waiting to be executed
pub struct QueuedAction {
    pub entity: Entity,
    pub action: Box<dyn Action>,
    pub utility: f32,
    pub priority: i32,
    pub queued_at_tick: u64,
}

impl PartialEq for QueuedAction {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.utility == other.utility
    }
}

impl Eq for QueuedAction {}

impl PartialOrd for QueuedAction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedAction {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority executes first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // If same priority, higher utility executes first
                self.utility
                    .partial_cmp(&other.utility)
                    .unwrap_or(Ordering::Equal)
            }
            other => other,
        }
    }
}

/// Resource: Action queue for the TQUAI system
#[derive(Resource)]
pub struct ActionQueue {
    /// Priority queue of pending actions
    pending: BinaryHeap<QueuedAction>,
    /// Recently completed actions (for trigger system)
    recently_completed: Vec<(Entity, u64)>, // (entity, completed_tick)
    /// Entities scheduled for cancellation before next tick execution
    pending_cancellations: HashSet<Entity>,
    /// Statistics
    pub stats: QueueStats,
}

#[derive(Debug, Default)]
pub struct QueueStats {
    pub actions_queued: u64,
    pub actions_executed: u64,
    pub actions_completed: u64,
    pub actions_failed: u64,
}

impl Default for ActionQueue {
    fn default() -> Self {
        Self {
            pending: BinaryHeap::new(),
            recently_completed: Vec::new(),
            pending_cancellations: HashSet::new(),
            stats: QueueStats::default(),
        }
    }
}

impl ActionQueue {
    /// Queue a new action for execution
    ///
    /// Note: This method needs World access to check for ActiveAction component.
    /// The signature will be updated in the refactored execute_tick flow.
    /// For now, we keep the old signature for compatibility.
    pub fn queue_action(
        &mut self,
        entity: Entity,
        action_type: ActionType,
        utility: f32,
        priority: i32,
        tick: u64,
    ) {
        let awaiting_cancellation = self.pending_cancellations.contains(&entity);

        // Note: We can't check for ActiveAction component here without World access
        // This check will be handled in execute_pending_actions instead

        // Check if entity already has a pending action - replace if new one is better
        let has_pending = self.pending.iter().any(|qa| qa.entity == entity);

        if has_pending && !awaiting_cancellation {
            // Simple replacement logic for now - don't queue duplicates
            // This could be enhanced with priority-based replacement later
            return;
        }

        let action = create_action(action_type);

        self.pending.push(QueuedAction {
            entity,
            action,
            utility,
            priority,
            queued_at_tick: tick,
        });

        self.stats.actions_queued += 1;
    }

    /// Execute all queued and active actions for this tick
    pub fn execute_tick(&mut self, world: &mut World, tick: u64) {
        self.process_pending_cancellations(world);

        // Active actions are now handled by execute_active_actions_system (registered in schedule)
        // This method only processes the pending action queue

        // Execute new actions from the queue
        self.execute_pending_actions(world, tick);
    }

}

/// Executes all active actions using proper Bevy system parameters.
///
/// This system replaces the old execute_active_actions method which used
/// exclusive &mut World access. By using Query and Commands, Bevy can
/// run this system in parallel with other non-conflicting systems.
///
/// TECHNICAL DETAILS:
/// - Uses Query<(Entity, &mut ActiveAction)> for safe concurrent access
/// - Uses Commands for deferred structural changes (removes components)
/// - Uses &World for read-only action execution (Action::execute takes &World)
/// - Gets current tick from SimulationTick resource
/// - Accesses ActionQueue stats via ResMut for tracking
pub fn execute_active_actions_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ActiveAction)>,
    world: &World,
    tick: Res<SimulationTick>,
    mut queue: ResMut<ActionQueue>,
) {
    for (entity, mut active_action) in query.iter_mut() {
        let action_name = active_action.action.name().to_string();
        let started_at_tick = active_action.started_at_tick;
        let current_tick = tick.0;

        // Execute the action with read-only world access
        let result = active_action.action.execute(world, entity);

        // Handle the action result using Commands for mutations
        match result {
            ActionResult::Success => {
                debug!(
                    "✅ Entity {:?} completed action '{}' after {} ticks",
                    entity,
                    action_name,
                    current_tick - started_at_tick
                );
                // Clear current action and remove ActiveAction component
                commands.entity(entity).remove::<ActiveAction>();
                commands.entity(entity).insert(CurrentAction::none());

                // Track recently completed action for trigger system
                queue.recently_completed.push((entity, current_tick));
                queue.stats.actions_completed += 1;
            }
            ActionResult::Failed => {
                warn!(
                    "❌ Entity {:?} failed action '{}'",
                    entity,
                    action_name
                );
                // Clear current action and remove ActiveAction component
                commands.entity(entity).remove::<ActiveAction>();
                commands.entity(entity).insert(CurrentAction::none());

                // Track failed action for trigger system
                queue.recently_completed.push((entity, current_tick));
                queue.stats.actions_failed += 1;
            }
            ActionResult::TriggerFollowUp => {
                debug!(
                    "🔄 Entity {:?} completed action '{}' with follow-up needed after {} ticks",
                    entity,
                    action_name,
                    current_tick - started_at_tick
                );
                // Clear current action to allow AI to plan next action
                commands.entity(entity).remove::<ActiveAction>();
                commands.entity(entity).insert(CurrentAction::none());

                // Trigger follow-up also counts as completion for trigger system
                queue.recently_completed.push((entity, current_tick));
                queue.stats.actions_completed += 1;
            }
            ActionResult::InProgress => {
                // Action still running - ActiveAction component stays (no Commands needed)
                // The &mut ActiveAction keeps the action state for next tick
            }
        }
    }
}

impl ActionQueue {
    /// Execute pending actions from the queue
    fn execute_pending_actions(&mut self, world: &mut World, tick: u64) {
        let mut executed_this_tick = Vec::new();

        // Process actions in priority order
        while let Some(mut queued) = self.pending.pop() {
            // Skip if entity already executed an action this tick
            if executed_this_tick.contains(&queued.entity) {
                continue;
            }

            // Skip if entity already has an active action (check component)
            if let Ok(entity_ref) = world.get_entity(queued.entity) {
                if entity_ref.contains::<ActiveAction>() {
                    continue;
                }
            } else {
                // Entity doesn't exist
                continue;
            }

            // Verify action can still be executed
            if !queued.action.can_execute(world, queued.entity) {
                debug!(
                    "⚠️ Entity {:?} cannot execute '{}' - preconditions failed",
                    queued.entity,
                    queued.action.name()
                );
                self.stats.actions_failed += 1;
                continue;
            }

            // Execute the action!
            let result = queued.action.execute(world, queued.entity);
            self.stats.actions_executed += 1;

            // Set current action on entity
            let action_name = queued.action.name().to_string();
            if let Ok(mut entity_mut) = world.get_entity_mut(queued.entity) {
                entity_mut.insert(CurrentAction::new(action_name.clone()));
            }

            match result {
                ActionResult::Success => {
                    debug!(
                        "✅ Entity {:?} instantly completed action '{}'",
                        queued.entity,
                        queued.action.name()
                    );
                    // Clear current action
                    if let Ok(mut entity_mut) = world.get_entity_mut(queued.entity) {
                        entity_mut.insert(CurrentAction::none());
                    }
                    // Track instant completion for trigger system
                    self.recently_completed.push((queued.entity, tick));
                    self.stats.actions_completed += 1;
                }
                ActionResult::Failed => {
                    warn!(
                        "❌ Entity {:?} failed action '{}'",
                        queued.entity,
                        queued.action.name()
                    );
                    // Clear current action
                    if let Ok(mut entity_mut) = world.get_entity_mut(queued.entity) {
                        entity_mut.insert(CurrentAction::none());
                    }
                    // Track failed action for trigger system
                    self.recently_completed.push((queued.entity, tick));
                    self.stats.actions_failed += 1;
                }
                ActionResult::TriggerFollowUp => {
                    debug!(
                        "🔄 Entity {:?} completed action '{}' with follow-up needed",
                        queued.entity,
                        queued.action.name()
                    );
                    // Clear current action to allow AI to plan next action
                    if let Ok(mut entity_mut) = world.get_entity_mut(queued.entity) {
                        entity_mut.insert(CurrentAction::none());
                    }
                    // Track follow-up completion for trigger system
                    self.recently_completed.push((queued.entity, tick));
                    self.stats.actions_completed += 1;
                }
                ActionResult::InProgress => {
                    // Action needs multiple ticks - insert as component
                    debug!(
                        "⏳ Entity {:?} started multi-tick action '{}'",
                        queued.entity,
                        queued.action.name()
                    );
                    if let Ok(mut entity_mut) = world.get_entity_mut(queued.entity) {
                        entity_mut.insert(ActiveAction::new(queued.action, tick));
                    }
                }
            }

            executed_this_tick.push(queued.entity);
        }
    }

    /// Get the number of pending actions
    #[inline(always)]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get the number of active actions
    /// This method requires World access to query components
    pub fn active_count_with_world(&self, world: &mut World) -> usize {
        let mut query = world.query::<&ActiveAction>();
        query.iter(world).count()
    }

    /// Get the number of active actions (no World access - approximate)
    /// NOTE: This is deprecated and should be replaced with active_count_with_world
    /// For now, returns 0 as we can't count components without World access
    #[deprecated(note = "Use active_count_with_world instead")]
    pub fn active_count(&self) -> usize {
        0 // Can't count components without World access
    }

    /// Check if an entity has any queued or active action
    /// This method requires World access to check for ActiveAction component
    pub fn has_action_with_world(&self, world: &World, entity: Entity) -> bool {
        // Check for ActiveAction component
        let has_active = if let Ok(entity_ref) = world.get_entity(entity) {
            entity_ref.contains::<ActiveAction>()
        } else {
            false
        };

        has_active || self.pending.iter().any(|qa| qa.entity == entity)
    }

    /// Check if an entity has any queued or pending action (no active check)
    /// NOTE: This doesn't check for ActiveAction component - use has_action_with_world for complete check
    #[inline]
    pub fn has_action(&self, entity: Entity) -> bool {
        self.pending.iter().any(|qa| qa.entity == entity)
    }

    /// Schedule cancellation of an entity's current action (processed at next tick)
    #[inline(always)]
    pub fn schedule_cancellation(&mut self, entity: Entity) {
        self.pending_cancellations.insert(entity);
    }

    /// Get entities that recently completed actions since the given tick
    pub fn get_recently_completed(&mut self, since_tick: u64) -> Vec<Entity> {
        // Clean up old entries (keep only last 100 ticks worth)
        self.recently_completed
            .retain(|(_, tick)| *tick >= since_tick.saturating_sub(100));

        // Get entities that completed since the given tick
        self.recently_completed
            .iter()
            .filter(|(_, tick)| *tick > since_tick)
            .map(|(entity, _)| *entity)
            .collect()
    }

    fn process_pending_cancellations(&mut self, world: &mut World) {
        let entities: Vec<Entity> = self.pending_cancellations.drain().collect();
        for entity in entities {
            self.cancel_action(world, entity);
        }
    }

    /// Cancel any active or pending action for the given entity
    /// Returns true if an action was cancelled
    pub fn cancel_action(&mut self, world: &mut World, entity: Entity) -> bool {
        let mut cancelled = false;

        // Check if entity has an active action component
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            if let Some(mut active_action) = entity_mut.take::<ActiveAction>() {
                let action_name = active_action.action.name().to_string();
                debug!(
                    "🚫 Cancelling active action '{}' for entity {:?}",
                    action_name,
                    entity
                );

                // Call the action's cancel method for cleanup
                active_action.action.cancel(world, entity);

                // Clear current action from entity
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(crate::entities::CurrentAction::none());
                }

                cancelled = true;
            }
        }

        // Remove any pending actions for this entity
        let original_len = self.pending.len();
        self.pending.retain(|qa| qa.entity != entity);
        let removed_pending = original_len > self.pending.len();

        if removed_pending {
            debug!("🚫 Cancelled pending actions for entity {:?}", entity);
            cancelled = true;
        }

        if cancelled {
            self.pending_cancellations.remove(&entity);
        }

        cancelled
    }

    /// Remove references to dead entities from all queue data structures
    /// NOTE: Active actions are now stored as components and clean up automatically
    /// when entities despawn - no manual cleanup needed!
    pub fn cleanup_dead_entities(&mut self, world: &World) {
        let mut recently_completed_removed = 0;
        let mut pending_removed = 0;
        let mut pending_cancellations_removed = 0;

        // Active actions are now components - they auto-cleanup on entity despawn!
        // No manual cleanup needed for active actions anymore.

        // Remove dead entities from recently_completed
        let original_len = self.recently_completed.len();
        self.recently_completed
            .retain(|(entity, _)| world.get_entity(*entity).is_ok());
        recently_completed_removed = original_len - self.recently_completed.len();

        // Remove dead entities from pending (requires collecting and rebuilding heap)
        let valid_pending: Vec<_> = self
            .pending
            .drain()
            .filter(|qa| {
                let is_alive = world.get_entity(qa.entity).is_ok();
                if !is_alive {
                    pending_removed += 1;
                }
                is_alive
            })
            .collect();
        self.pending = valid_pending.into_iter().collect();

        // Clean pending cancellations
        let original_len = self.pending_cancellations.len();
        self.pending_cancellations
            .retain(|entity| world.get_entity(*entity).is_ok());
        pending_cancellations_removed = original_len - self.pending_cancellations.len();

        if pending_removed > 0 || recently_completed_removed > 0 {
            debug!(
                "Cleaned dead entities from ActionQueue: {} pending, {} recently_completed, {} pending_cancellations (active actions auto-cleanup via components)",
                pending_removed, recently_completed_removed, pending_cancellations_removed
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a minimal test world
    fn create_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_cleanup_removes_dead_entities_from_active() {
        let mut world = create_test_world();
        let mut queue = ActionQueue::default();

        // Spawn two entities
        let entity1 = world.spawn((crate::entities::CurrentAction::none(),)).id();
        let entity2 = world.spawn((crate::entities::CurrentAction::none(),)).id();

        // Insert active actions as components
        world.entity_mut(entity1).insert(ActiveAction::new(
            create_action(ActionType::Rest { duration_ticks: 5 }),
            0,
        ));
        world.entity_mut(entity2).insert(ActiveAction::new(
            create_action(ActionType::Rest { duration_ticks: 5 }),
            0,
        ));

        assert_eq!(
            queue.active_count_with_world(&mut world),
            2,
            "Should have 2 active actions"
        );

        // Despawn entity1
        world.despawn(entity1);

        // Components auto-cleanup - no manual cleanup needed!
        // (But we still call cleanup_dead_entities for other data structures)
        queue.cleanup_dead_entities(&world);

        assert_eq!(
            queue.active_count_with_world(&mut world),
            1,
            "Should have 1 active action after despawn (auto-cleanup)"
        );
        assert!(
            world.get::<ActiveAction>(entity2).is_some(),
            "entity2 should still have active action"
        );
    }

    #[test]
    fn test_cleanup_removes_dead_entities_from_recently_completed() {
        let mut world = create_test_world();
        let mut queue = ActionQueue::default();

        // Spawn entities
        let entity1 = world.spawn((crate::entities::CurrentAction::none(),)).id();
        let entity2 = world.spawn((crate::entities::CurrentAction::none(),)).id();

        // Add to recently completed
        queue.recently_completed.push((entity1, 100));
        queue.recently_completed.push((entity2, 105));

        assert_eq!(
            queue.recently_completed.len(),
            2,
            "Should have 2 recently completed"
        );

        // Despawn entity1
        world.despawn(entity1);

        // Cleanup should remove the dead entity
        queue.cleanup_dead_entities(&world);

        assert_eq!(
            queue.recently_completed.len(),
            1,
            "Should have 1 recently completed after cleanup"
        );
        assert_eq!(
            queue.recently_completed[0].0, entity2,
            "entity2 should still be in recently completed"
        );
    }

    #[test]
    fn test_cleanup_removes_dead_entities_from_pending_cancellations() {
        let mut world = create_test_world();
        let mut queue = ActionQueue::default();

        // Spawn entities
        let entity1 = world.spawn((crate::entities::CurrentAction::none(),)).id();
        let entity2 = world.spawn((crate::entities::CurrentAction::none(),)).id();

        // Add to pending cancellations
        queue.pending_cancellations.insert(entity1);
        queue.pending_cancellations.insert(entity2);

        assert_eq!(
            queue.pending_cancellations.len(),
            2,
            "Should have 2 pending cancellations"
        );

        // Despawn entity1
        world.despawn(entity1);

        // Cleanup should remove the dead entity
        queue.cleanup_dead_entities(&world);

        assert_eq!(
            queue.pending_cancellations.len(),
            1,
            "Should have 1 pending cancellation after cleanup"
        );
        assert!(
            queue.pending_cancellations.contains(&entity2),
            "entity2 should still be in pending cancellations"
        );
    }

    #[test]
    fn test_cleanup_comprehensive_dead_entity_removal() {
        let mut world = create_test_world();
        let mut queue = ActionQueue::default();

        // Spawn 3 entities
        let alive1 = world.spawn((crate::entities::CurrentAction::none(),)).id();
        let dead1 = world.spawn((crate::entities::CurrentAction::none(),)).id();
        let dead2 = world.spawn((crate::entities::CurrentAction::none(),)).id();

        // Add active actions as components
        world.entity_mut(alive1).insert(ActiveAction::new(
            create_action(ActionType::Rest { duration_ticks: 5 }),
            0,
        ));
        world.entity_mut(dead1).insert(ActiveAction::new(
            create_action(ActionType::Rest { duration_ticks: 5 }),
            0,
        ));

        queue.recently_completed.push((alive1, 100));
        queue.recently_completed.push((dead1, 105));
        queue.recently_completed.push((dead2, 110));

        queue.pending_cancellations.insert(dead1);
        queue.pending_cancellations.insert(dead2);

        // Despawn dead entities
        world.despawn(dead1);
        world.despawn(dead2);

        // Cleanup
        queue.cleanup_dead_entities(&world);

        // Verify only alive entities remain
        assert_eq!(
            queue.active_count_with_world(&mut world),
            1,
            "Should have only alive1 active (auto-cleanup)"
        );
        assert!(world.get::<ActiveAction>(alive1).is_some());

        assert_eq!(
            queue.recently_completed.len(),
            1,
            "Should have only alive1 in recently_completed"
        );
        assert_eq!(queue.recently_completed[0].0, alive1);

        assert!(
            queue.pending_cancellations.is_empty(),
            "Should have no pending cancellations for dead entities"
        );
    }
}
