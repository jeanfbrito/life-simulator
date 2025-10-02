/// Action queue for TQUAI system
/// 
/// Manages queued actions with priorities, executing them synchronously on ticks.
/// Handles multi-tick actions that span across multiple ticks.

use bevy::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use super::action::{Action, ActionResult, ActionType, create_action};

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
                self.utility.partial_cmp(&other.utility)
                    .unwrap_or(Ordering::Equal)
            }
            other => other
        }
    }
}

/// Active action that is executing across multiple ticks
struct ActiveAction {
    entity: Entity,
    action: Box<dyn Action>,
    started_at_tick: u64,
}

/// Resource: Action queue for the TQUAI system
#[derive(Resource)]
pub struct ActionQueue {
    /// Priority queue of pending actions
    pending: BinaryHeap<QueuedAction>,
    /// Currently executing multi-tick actions
    active: HashMap<Entity, ActiveAction>,
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
            active: HashMap::new(),
            stats: QueueStats::default(),
        }
    }
}

impl ActionQueue {
    /// Queue a new action for execution
    pub fn queue_action(&mut self, entity: Entity, action_type: ActionType, utility: f32, priority: i32, tick: u64) {
        // Don't queue if entity already has an active action
        if self.active.contains_key(&entity) {
            return;
        }
        
        // Check if entity already has a pending action - replace if new one is better
        let has_pending = self.pending.iter()
            .any(|qa| qa.entity == entity);
        
        if has_pending {
            // TODO: More sophisticated replacement logic
            // For now, just don't queue duplicate
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
        // First, continue any active multi-tick actions
        self.execute_active_actions(world, tick);
        
        // Then execute new actions from the queue
        self.execute_pending_actions(world, tick);
    }
    
    /// Execute active multi-tick actions
    fn execute_active_actions(&mut self, world: &mut World, tick: u64) {
        let mut to_remove = Vec::new();
        
        for (entity, active) in self.active.iter_mut() {
            // Check if entity still exists
            if world.get_entity(*entity).is_err() {
                to_remove.push(*entity);
                continue;
            }
            
            // Execute the action
            let result = active.action.execute(world, *entity, tick);
            
            match result {
                ActionResult::Success => {
                    debug!(
                        "✅ Entity {:?} completed action '{}' after {} ticks",
                        entity,
                        active.action.name(),
                        tick - active.started_at_tick
                    );
                    to_remove.push(*entity);
                    self.stats.actions_completed += 1;
                }
                ActionResult::Failed => {
                    warn!(
                        "❌ Entity {:?} failed action '{}'",
                        entity,
                        active.action.name()
                    );
                    to_remove.push(*entity);
                    self.stats.actions_failed += 1;
                }
                ActionResult::InProgress => {
                    // Continue next tick
                }
            }
        }
        
        // Remove completed/failed actions
        for entity in to_remove {
            self.active.remove(&entity);
        }
    }
    
    /// Execute pending actions from the queue
    fn execute_pending_actions(&mut self, world: &mut World, tick: u64) {
        let mut executed_this_tick = Vec::new();
        
        // Process actions in priority order
        while let Some(mut queued) = self.pending.pop() {
            // Skip if entity already executed an action this tick
            if executed_this_tick.contains(&queued.entity) {
                continue;
            }
            
            // Skip if entity already has an active action
            if self.active.contains_key(&queued.entity) {
                continue;
            }
            
            // Check if entity still exists
            if world.get_entity(queued.entity).is_err() {
                continue;
            }
            
            // Verify action can still be executed
            if !queued.action.can_execute(world, queued.entity, tick) {
                debug!(
                    "⚠️ Entity {:?} cannot execute '{}' - preconditions failed",
                    queued.entity,
                    queued.action.name()
                );
                self.stats.actions_failed += 1;
                continue;
            }
            
            // Execute the action!
            let result = queued.action.execute(world, queued.entity, tick);
            self.stats.actions_executed += 1;
            
            match result {
                ActionResult::Success => {
                    debug!(
                        "✅ Entity {:?} instantly completed action '{}'",
                        queued.entity,
                        queued.action.name()
                    );
                    self.stats.actions_completed += 1;
                }
                ActionResult::Failed => {
                    warn!(
                        "❌ Entity {:?} failed action '{}'",
                        queued.entity,
                        queued.action.name()
                    );
                    self.stats.actions_failed += 1;
                }
                ActionResult::InProgress => {
                    // Action needs multiple ticks - move to active
                    debug!(
                        "⏳ Entity {:?} started multi-tick action '{}'",
                        queued.entity,
                        queued.action.name()
                    );
                    self.active.insert(queued.entity, ActiveAction {
                        entity: queued.entity,
                        action: queued.action,
                        started_at_tick: tick,
                    });
                }
            }
            
            executed_this_tick.push(queued.entity);
        }
    }
    
    /// Get the number of pending actions
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    
    /// Get the number of active actions
    pub fn active_count(&self) -> usize {
        self.active.len()
    }
    
    /// Check if an entity has any queued or active action
    pub fn has_action(&self, entity: Entity) -> bool {
        self.active.contains_key(&entity) ||
        self.pending.iter().any(|qa| qa.entity == entity)
    }
}
