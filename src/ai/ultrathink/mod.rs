/// UltraThink: Queue-Based AI Processing System
/// Inspired by Dwarf Fortress LOD Architecture
///
/// Core concept: Thoughts are queued by priority and processed with a fixed budget per tick.
/// Only urgent things (fear, critical hunger) need immediate processing.
/// Everything else can wait in queue and be processed by priority.

pub mod queue;
pub mod request;
pub mod test_harness;

// Re-exports for convenience
pub use queue::{ThinkQueue, ultrathink_system};
pub use request::{ThinkPriority, ThinkReason, ThinkRequest};
pub use test_harness::test_schedule_requests;

use bevy::prelude::*;

/// Plugin that sets up the UltraThink system
pub struct UltraThinkPlugin {
    /// How many think requests to process per tick
    pub thinks_per_tick: usize,
}

impl Default for UltraThinkPlugin {
    fn default() -> Self {
        Self {
            thinks_per_tick: 50,
        }
    }
}

impl Plugin for UltraThinkPlugin {
    fn build(&self, app: &mut App) {
        // Initialize queue with configured budget
        let queue = ThinkQueue::new(self.thinks_per_tick);
        app.insert_resource(queue);

        // Add ultrathink system to FixedUpdate schedule
        // This runs on simulation ticks alongside other AI systems
        app.add_systems(
            FixedUpdate,
            ultrathink_system.run_if(|state: Res<crate::simulation::SimulationState>| state.should_tick),
        );

        // Add test harness if ULTRATHINK_TEST environment variable is set
        if std::env::var("ULTRATHINK_TEST").is_ok() {
            info!("🧪 UltraThink Test Harness enabled");
            app.add_systems(
                FixedUpdate,
                test_schedule_requests.run_if(|state: Res<crate::simulation::SimulationState>| state.should_tick),
            );
        }

        info!(
            "🧠 UltraThink Plugin initialized with {} thinks per tick budget",
            self.thinks_per_tick
        );
    }
}
