/// Tick-Queued Utility AI (TQUAI) System
///
/// A deterministic, tick-synchronized AI system where:
/// - Decisions evaluate asynchronously (every frame if needed)
/// - Actions queue with priorities
/// - Execution happens SYNCHRONOUSLY on simulation ticks
/// - Resource contention resolved through queue ordering
pub mod action;
pub mod behaviors;
pub mod collectables;
pub mod consideration;
pub mod debug_collectables;
pub mod event_driven_planner;
pub mod herbivore_toolkit;
pub mod planner;
pub mod predator_toolkit;
pub mod queue;
pub mod replan_queue;
pub mod test_collectable_pipeline;
pub mod trigger_emitters;

pub use action::{
    create_action, Action, ActionRequest, ActionResult, ActionType, DrinkWaterAction, GrazeAction,
    HarvestAction, RestAction,
};
pub use collectables::{
    CollectableInfo, CollectableSearchConfig, CollectableStats, debug_list_collectables,
    get_collectable_stats, get_collectable_targets, get_all_collectable_types, is_collectable,
};

// Re-export web API functions for easier access
pub use collectables::web_api::{
    get_collectable_stats_json, debug_collectables_json, get_collectable_types_json,
};
pub use debug_collectables::CollectableDebugPlugin;
pub use consideration::{Consideration, ConsiderationSet, ResponseCurve};
pub use event_driven_planner::{EventDrivenPlannerPlugin, NeedsReplanning};
pub use planner::UtilityScore;
pub use queue::{ActionQueue, QueuedAction};
pub use replan_queue::{ReplanPriority, ReplanQueue, ReplanRequest};
pub use trigger_emitters::{IdleTracker, StatThresholdTracker, TriggerEmittersPlugin};

use bevy::prelude::*;

/// Plugin that sets up the TQUAI system
pub struct TQUAIPlugin;

impl Plugin for TQUAIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ActionQueue>()
            .init_resource::<ReplanQueue>()
            // Plugins
            .add_plugins(TriggerEmittersPlugin)
            .add_plugins(EventDrivenPlannerPlugin)
            // Tick-synced systems (run on simulation ticks)
            .add_systems(FixedUpdate, (execute_queued_actions,).run_if(should_tick));
    }
}

/// Run condition that checks if a tick should happen
fn should_tick(state: Res<crate::simulation::SimulationState>) -> bool {
    state.should_tick
}

/// System that executes queued actions on each tick
/// CRITICAL: This runs synchronously with other tick systems
/// Uses exclusive system to get mutable World access
fn execute_queued_actions(world: &mut World) {
    let tick = world.resource::<crate::simulation::SimulationTick>().0;

    // Start profiling before executing queued actions
    if let Some(mut profiler) = world.get_resource_mut::<crate::simulation::TickProfiler>() {
        crate::simulation::profiler::start_timing_resource(&mut profiler, "ai_actions");
    }

    // Execute the AI actions
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        queue.execute_tick(world, tick);

        // Periodic cleanup of dead entities every 100 ticks to prevent HashMap slowdown
        if tick % 100 == 0 {
            queue.cleanup_dead_entities(world);
        }
    });

    // End profiling after execution completes
    if let Some(mut profiler) = world.get_resource_mut::<crate::simulation::TickProfiler>() {
        crate::simulation::profiler::end_timing_resource(&mut profiler, "ai_actions");
    }
}
