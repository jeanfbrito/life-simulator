/// Test harness for manually scheduling think requests
/// Used in Phase 1 to verify queue functionality

use super::{ThinkQueue, ThinkReason};
use bevy::prelude::*;

/// System that manually schedules some test think requests on startup
/// This demonstrates that the queue infrastructure is working
pub fn test_schedule_requests(
    mut commands: Commands,
    mut think_queue: ResMut<ThinkQueue>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    // Only run once on first tick
    if tick.0 != 1 {
        return;
    }

    info!("🧪 UltraThink Test Harness: Scheduling test requests...");

    // Spawn some test entities and schedule think requests
    let entity1 = commands.spawn_empty().id();
    let entity2 = commands.spawn_empty().id();
    let entity3 = commands.spawn_empty().id();
    let entity4 = commands.spawn_empty().id();
    let entity5 = commands.spawn_empty().id();

    // Schedule at different priorities
    think_queue.schedule_urgent(entity1, ThinkReason::FearTriggered, tick.0);
    think_queue.schedule_urgent(entity2, ThinkReason::HungerCritical, tick.0);
    think_queue.schedule_normal(entity3, ThinkReason::ActionCompleted, tick.0);
    think_queue.schedule_normal(entity4, ThinkReason::HungerModerate, tick.0);
    think_queue.schedule_low(entity5, ThinkReason::Idle, tick.0);

    let (urgent, normal, low) = think_queue.queue_sizes();
    info!(
        "🧪 Test Harness: Scheduled {} urgent, {} normal, {} low requests",
        urgent, normal, low
    );
}
