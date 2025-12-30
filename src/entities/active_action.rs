/// ActiveAction component - stores the executing action as a component
///
/// This replaces the HashMap-based storage in ActionQueue, providing:
/// - Automatic cleanup when entities despawn (no manual tracking needed)
/// - Better ECS architecture (data on entities, not in global HashMap)
/// - Same 10 TPS performance with cleaner code
///
/// Separation of concerns:
/// - ActiveAction: Execution state (trait object, started_at_tick)
/// - CurrentAction: Visualization state (action name string for web API)
use bevy::prelude::*;
use crate::ai::actions::Action;

/// Component that stores an active multi-tick action
#[derive(Component)]
pub struct ActiveAction {
    /// The action being executed (trait object)
    pub action: Box<dyn Action>,
    /// Tick when action started
    pub started_at_tick: u64,
}

impl ActiveAction {
    pub fn new(action: Box<dyn Action>, started_at_tick: u64) -> Self {
        Self {
            action,
            started_at_tick,
        }
    }
}
