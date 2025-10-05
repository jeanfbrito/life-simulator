/// Tick-Queued Utility AI (TQUAI) System
///
/// A deterministic, tick-synchronized AI system where:
/// - Decisions evaluate asynchronously (every frame if needed)
/// - Actions queue with priorities
/// - Execution happens SYNCHRONOUSLY on simulation ticks
/// - Resource contention resolved through queue ordering
pub mod action;
pub mod behaviors;
pub mod consideration;
pub mod herbivore_toolkit;
pub mod planner;
pub mod queue;

pub use action::{
    create_action, Action, ActionRequest, ActionResult, ActionType, DrinkWaterAction, GrazeAction,
    RestAction,
};
pub use consideration::{Consideration, ConsiderationSet, ResponseCurve};
pub use planner::UtilityScore;
pub use queue::{ActionQueue, QueuedAction};

use bevy::prelude::*;

/// Plugin that sets up the TQUAI system
pub struct TQUAIPlugin;

impl Plugin for TQUAIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ActionQueue>()
            // Tick-synced systems (run when should_tick == true)
            .add_systems(Update, (execute_queued_actions,).run_if(should_tick))
            // Frame-rate systems (run every frame for responsiveness)
            // Wait for WorldLoader resource to be available before planning
            .add_systems(
                Update,
                (
                    crate::entities::types::rabbit::plan_rabbit_actions,
                    crate::entities::types::deer::plan_deer_actions,
                    crate::entities::types::raccoon::plan_raccoon_actions,
                )
                    .run_if(resource_exists::<crate::world_loader::WorldLoader>)
                    .run_if(resource_exists::<crate::vegetation::VegetationGrid>),
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
    world.resource_scope(|world, mut queue: Mut<ActionQueue>| {
        let tick = world.resource::<crate::simulation::SimulationTick>().0;
        queue.execute_tick(world, tick);
    });
}
