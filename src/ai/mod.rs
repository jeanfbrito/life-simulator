#![allow(unused_imports)]
/// Tick-Queued Utility AI (TQUAI) System
///
/// A deterministic, tick-synchronized AI system where:
/// - Decisions evaluate asynchronously (every frame if needed)
/// - Actions queue with priorities
/// - Execution happens SYNCHRONOUSLY on simulation ticks
/// - Resource contention resolved through queue ordering
pub mod action;
pub mod action_pathfinding_bridge;
pub mod behaviors;
pub mod collectables;
pub mod consideration;
pub mod debug_collectables;
pub mod entity_validator;
pub mod event_driven_planner;
pub mod failure_memory;
pub mod group_cohesion;
pub mod group_coordination;
pub mod group_formation;
pub mod herbivore_toolkit;
pub mod hunting_relationship_system;
pub mod mating_relationship_system;
pub mod pack_relationship_system;
pub mod parent_child_relationship_system;
pub mod planner;
pub mod predator_toolkit;
pub mod queue;
pub mod replan_queue;
pub mod system_params;
pub mod test_collectable_pipeline;
pub mod trigger_emitters;
pub mod ultrathink;
pub mod watchdog;

#[cfg(test)]
pub mod lifecycle_tests;

pub use action::{
    create_action, Action, ActionRequest, ActionResult, ActionType, DrinkWaterAction, GrazeAction,
    HarvestAction, RestAction,
};
pub use collectables::{
    CollectableInfo, CollectableSearchConfig, CollectableStats, debug_list_collectables,
    get_collectable_stats, get_collectable_targets, get_all_collectable_types, is_collectable,
};
pub use hunting_relationship_system::{
    establish_hunting_relationship, clear_hunting_relationship, cleanup_stale_hunting_relationships,
    has_hunting_relationship, is_being_hunted,
};
pub use mating_relationship_system::{
    establish_mating_relationship, clear_mating_relationship, cleanup_stale_mating_relationships,
    has_mating_relationship, is_being_courted, get_mating_partner,
};
pub use pack_relationship_system::{
    establish_pack_leadership, add_to_pack, remove_from_pack, dissolve_pack,
    cleanup_stale_pack_relationships, get_pack_members, get_pack_leader, is_pack_leader,
    is_pack_member, is_in_pack, get_pack_size, are_in_same_pack,
};
pub use parent_child_relationship_system::{
    establish_parent_child_relationship, establish_parent_child_immediate,
    remove_parent_child_relationship, remove_parent_child_immediate,
    cleanup_orphaned_children, get_parent, get_children, has_child, has_parent,
    child_count, child_birth_tick,
};

// Re-export web API functions for easier access
pub use collectables::web_api::{
    get_collectable_stats_json, debug_collectables_json, get_collectable_types_json,
};
pub use debug_collectables::CollectableDebugPlugin;
pub use consideration::{Consideration, ConsiderationSet, ResponseCurve};
pub use event_driven_planner::{EventDrivenPlannerPlugin, NeedsReplanning};
pub use planner::UtilityScore;
pub use queue::{
    ActionQueue, QueuedAction, execute_active_actions_read_only, handle_action_results,
    ActionExecutionResult, handle_action_failure_with_replan, handle_action_failure_exclusive,
    handle_precondition_failure_exclusive,
};
pub use failure_memory::{ActionFailureMemory, apply_failure_penalties, action_type_to_string, action_type_to_base_name};
pub use replan_queue::{ReplanPriority, ReplanQueue, ReplanRequest};
pub use system_params::PlanningResources;
pub use trigger_emitters::{IdleTracker, StatThresholdTracker, TriggerEmittersPlugin};
pub use ultrathink::{ThinkQueue, ThinkRequest, ThinkReason, ThinkPriority, UltraThinkPlugin};
pub use entity_validator::{EntityValidatorPlugin, EntityValidationMetrics};
pub use watchdog::{WatchdogPlugin, WatchdogConfig, WatchdogMetrics, WatchdogHistory, InterventionLevel};
pub use group_formation::generic_group_formation_system;
pub use group_cohesion::{generic_group_cohesion_system, process_member_removals};
pub use group_coordination::apply_group_behavior_bonuses;

use bevy::prelude::*;

/// Plugin that sets up the TQUAI system
pub struct TQUAIPlugin;

impl Plugin for TQUAIPlugin {
    fn build(&self, app: &mut App) {
        use crate::simulation::SimulationSet;

        app
            // Resources
            .init_resource::<ActionQueue>()
            .init_resource::<ReplanQueue>()
            // Plugins
            // IMPORTANT: UltraThinkPlugin MUST come before EventDrivenPlannerPlugin
            // because ultrathink_system (registered in EventDrivenPlannerPlugin)
            // requires ThinkQueue resource (created by UltraThinkPlugin)
            .add_plugins(UltraThinkPlugin::default())
            .add_plugins(TriggerEmittersPlugin)
            .add_plugins(EventDrivenPlannerPlugin)
            .add_plugins(EntityValidatorPlugin)
            .add_plugins(WatchdogPlugin)
            // NOTE: Group dynamics systems (generic_group_formation_system, etc.) are
            // registered in EntitiesPlugin to avoid duplicate execution.
            // === ACTION EXECUTION PHASE ===
            // Tick-synced action execution (must run after Planning, before Movement)
            // Split into multiple systems to avoid &World + Commands parameter conflict:
            // 1. execute_active_actions_read_only: Execute actions with &World (parallelizable)
            // 2. bridge_actions_to_pathfinding: Queue pathfinding for NeedsPathfinding results
            // 3. handle_action_results: Handle results with Commands (mutations)
            // 4. execute_queued_actions: Process pending action queue
            .add_systems(
                Update,
                (
                    execute_active_actions_read_only,
                    ApplyDeferred, // CRITICAL: flush commands between systems
                    action_pathfinding_bridge::bridge_actions_to_pathfinding,
                    ApplyDeferred, // CRITICAL: flush pathfinding queue before result handling
                    handle_action_results,
                )
                    .chain()
                    .in_set(SimulationSet::ActionExecution)
                    .before(execute_queued_actions)
                    .run_if(should_tick),
            )
            .add_systems(
                Update,
                execute_queued_actions
                    .in_set(SimulationSet::ActionExecution)
                    .after(SimulationSet::Planning)
                    .run_if(should_tick),
            )
            // === CLEANUP PHASE ===
            // Clean up stale hunting relationships (prey despawned while being hunted)
            // Clean up stale pack relationships (members despawned while in pack)
            .add_systems(
                Update,
                (
                    cleanup_stale_hunting_relationships,
                    cleanup_stale_pack_relationships,
                    cleanup_stale_mating_relationships,
                )
                    .in_set(SimulationSet::Cleanup)
                    .run_if(should_tick),
            );
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
