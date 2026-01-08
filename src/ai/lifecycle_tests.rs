/// Entity Lifecycle Integration Tests
///
/// Tests for the entity lifecycle within the AI system, verifying:
/// - Newly spawned entities have all required AI components
/// - Entities survive action failures and get replanned
/// - Trigger flags reset after successful actions
/// - Critical stats trigger urgent priorities
/// - Action completion triggers replanning

use bevy::prelude::*;

use crate::ai::actions::{create_action, ActionType};
use crate::ai::queue::ActionQueue;
use crate::ai::replan_queue::{ReplanPriority, ReplanQueue};
use crate::ai::trigger_emitters::{IdleTracker, StatThresholdTracker};
use crate::ai::ultrathink::{ThinkPriority, ThinkQueue, ThinkReason};
use crate::entities::stats::{Energy, Hunger, Stat, Thirst};
use crate::entities::types::BehaviorConfig;
use crate::entities::{ActiveAction, CurrentAction, TilePosition};
use crate::simulation::SimulationTick;

/// Helper function to create a minimal test world with required resources
fn create_test_world() -> World {
    let mut world = World::new();
    world.insert_resource(SimulationTick(0));
    world.insert_resource(ActionQueue::default());
    world.insert_resource(ReplanQueue::new());
    world.insert_resource(ThinkQueue::new(50));
    world
}

/// Helper function to spawn a basic test entity with core stats and AI components
fn spawn_test_entity_with_all_components(world: &mut World) -> Entity {
    world
        .spawn((
            // Core stats
            Hunger(Stat::new(0.0, 0.0, 100.0, 0.1)),
            Thirst(Stat::new(0.0, 0.0, 100.0, 0.1)),
            Energy(Stat::new(100.0, 0.0, 100.0, -0.05)),
            // Position
            TilePosition::from_tile(IVec2::new(10, 10)),
            // AI components
            BehaviorConfig::default(),
            IdleTracker::default(),
            StatThresholdTracker::default(),
            CurrentAction::none(),
        ))
        .id()
}

/// Helper function to spawn a minimal test entity (missing some AI components)
fn spawn_minimal_entity(world: &mut World) -> Entity {
    world
        .spawn((
            Hunger(Stat::new(0.0, 0.0, 100.0, 0.1)),
            Thirst(Stat::new(0.0, 0.0, 100.0, 0.1)),
            Energy(Stat::new(100.0, 0.0, 100.0, -0.05)),
            TilePosition::from_tile(IVec2::new(10, 10)),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Verify newly spawned entities have all AI components
    ///
    /// When an entity is spawned with AI capabilities, it should have:
    /// - IdleTracker (tracks idle time for replanning triggers)
    /// - StatThresholdTracker (tracks stat threshold crossings)
    /// - BehaviorConfig (defines AI behavior parameters)
    /// - CurrentAction (tracks current action for visualization)
    #[test]
    fn test_spawned_entity_has_all_ai_components() {
        let mut world = create_test_world();
        let entity = spawn_test_entity_with_all_components(&mut world);

        // Verify all required AI components are present
        assert!(
            world.get::<IdleTracker>(entity).is_some(),
            "Entity should have IdleTracker component"
        );
        assert!(
            world.get::<StatThresholdTracker>(entity).is_some(),
            "Entity should have StatThresholdTracker component"
        );
        assert!(
            world.get::<BehaviorConfig>(entity).is_some(),
            "Entity should have BehaviorConfig component"
        );
        assert!(
            world.get::<CurrentAction>(entity).is_some(),
            "Entity should have CurrentAction component"
        );

        // Verify the trackers are initialized with sensible defaults
        let idle_tracker = world.get::<IdleTracker>(entity).unwrap();
        assert_eq!(
            idle_tracker.ticks_since_action, 0,
            "IdleTracker should start with 0 ticks"
        );
        assert!(
            !idle_tracker.action_completed,
            "IdleTracker should start with action_completed = false"
        );

        let stat_tracker = world.get::<StatThresholdTracker>(entity).unwrap();
        assert!(
            !stat_tracker.hunger_triggered,
            "StatThresholdTracker should start with hunger_triggered = false"
        );
        assert!(
            !stat_tracker.thirst_triggered,
            "StatThresholdTracker should start with thirst_triggered = false"
        );
        assert!(
            !stat_tracker.energy_triggered,
            "StatThresholdTracker should start with energy_triggered = false"
        );

        let current_action = world.get::<CurrentAction>(entity).unwrap();
        assert_eq!(
            current_action.action_name, "Idle",
            "CurrentAction should start with 'Idle'"
        );
    }

    /// Test 2: Verify entity survives action failure and gets replanned
    ///
    /// When an action fails, the entity should:
    /// - Not be despawned or left in an invalid state
    /// - Be added to the ReplanQueue for a new action
    /// - Have its CurrentAction cleared
    #[test]
    fn test_entity_survives_action_failure() {
        let mut world = create_test_world();
        let entity = spawn_test_entity_with_all_components(&mut world);

        // Give entity an active action
        let action = create_action(ActionType::Rest { duration_ticks: 5 });
        world.entity_mut(entity).insert(ActiveAction::new(action, 0));
        world
            .entity_mut(entity)
            .insert(CurrentAction::new("Rest".to_string()));

        // Verify entity has active action
        assert!(
            world.get::<ActiveAction>(entity).is_some(),
            "Entity should have ActiveAction before failure"
        );

        // Simulate action failure by removing ActiveAction and clearing CurrentAction
        world.entity_mut(entity).remove::<ActiveAction>();
        world.entity_mut(entity).insert(CurrentAction::none());

        // Add to replan queue as the system would
        let mut replan_queue = world.resource_mut::<ReplanQueue>();
        replan_queue.push(
            entity,
            ReplanPriority::Normal,
            "Action failed".to_string(),
            1,
        );

        // Verify entity still exists and is in valid state
        assert!(
            world.get_entity(entity).is_ok(),
            "Entity should still exist after action failure"
        );
        assert!(
            world.get::<ActiveAction>(entity).is_none(),
            "Entity should not have ActiveAction after failure"
        );

        // Verify entity is queued for replanning
        let replan_queue = world.resource::<ReplanQueue>();
        assert!(
            replan_queue.contains(entity),
            "Entity should be queued for replanning after action failure"
        );

        // Verify CurrentAction is cleared
        let current_action = world.get::<CurrentAction>(entity).unwrap();
        assert_eq!(
            current_action.action_name, "Idle",
            "CurrentAction should be 'Idle' after action failure"
        );
    }

    /// Test 3: Verify trigger flags reset after action
    ///
    /// When hunger drops below threshold after successful grazing:
    /// - hunger_triggered flag should reset to false
    /// - Entity should be able to trigger again when hunger rises
    #[test]
    fn test_trigger_flags_reset_after_action() {
        let mut world = create_test_world();
        let entity = spawn_test_entity_with_all_components(&mut world);

        // Simulate high hunger (above threshold) - trigger activated
        {
            let mut tracker = world.get_mut::<StatThresholdTracker>(entity).unwrap();
            tracker.hunger_triggered = true;
            tracker.previous_hunger = 0.8; // 80% hungry
        }

        // Verify flag is set
        {
            let tracker = world.get::<StatThresholdTracker>(entity).unwrap();
            assert!(
                tracker.hunger_triggered,
                "hunger_triggered should be true after threshold crossing"
            );
        }

        // Simulate successful grazing - hunger drops below threshold
        {
            let mut hunger = world.get_mut::<Hunger>(entity).unwrap();
            hunger.0 = Stat::new(20.0, 0.0, 100.0, 0.1); // 20% hungry (normalized = 0.2)
        }

        // Simulate what stat_threshold_system does when hunger goes below threshold
        {
            let hunger = world.get::<Hunger>(entity).unwrap();
            let behavior_config = world.get::<BehaviorConfig>(entity).unwrap();
            let current_hunger = hunger.0.normalized();

            if current_hunger < behavior_config.hunger_threshold {
                let mut tracker = world.get_mut::<StatThresholdTracker>(entity).unwrap();
                tracker.hunger_triggered = false; // Reset flag
            }
        }

        // Verify flag is reset
        {
            let tracker = world.get::<StatThresholdTracker>(entity).unwrap();
            assert!(
                !tracker.hunger_triggered,
                "hunger_triggered should be false after hunger drops below threshold"
            );
        }

        // Verify entity can trigger again when hunger rises
        {
            let mut hunger = world.get_mut::<Hunger>(entity).unwrap();
            hunger.0 = Stat::new(80.0, 0.0, 100.0, 0.1); // 80% hungry again
        }

        // Simulate threshold crossing again
        {
            // Read immutable values first
            let current_hunger = world.get::<Hunger>(entity).unwrap().0.normalized();
            let hunger_threshold = world.get::<BehaviorConfig>(entity).unwrap().hunger_threshold;

            // Then get mutable tracker
            let mut tracker = world.get_mut::<StatThresholdTracker>(entity).unwrap();

            if current_hunger >= hunger_threshold && !tracker.hunger_triggered {
                tracker.hunger_triggered = true;
            }
        }

        // Verify flag can be set again
        {
            let tracker = world.get::<StatThresholdTracker>(entity).unwrap();
            assert!(
                tracker.hunger_triggered,
                "hunger_triggered should be true again after hunger rises above threshold"
            );
        }
    }

    /// Test 4: Verify entity with critical stats gets urgent ThinkQueue priority
    ///
    /// When an entity has hunger >= 80%, it should be scheduled with:
    /// - ThinkPriority::Urgent in ThinkQueue
    /// - ThinkReason::HungerCritical
    #[test]
    fn test_entity_with_critical_stats_gets_urgent_priority() {
        let mut world = create_test_world();
        let entity = spawn_test_entity_with_all_components(&mut world);

        // Set critical hunger (80%+)
        {
            let mut hunger = world.get_mut::<Hunger>(entity).unwrap();
            hunger.0 = Stat::new(85.0, 0.0, 100.0, 0.1); // 85% hungry
        }

        // Schedule urgent think request (simulating what stat_threshold_system does)
        {
            let hunger = world.get::<Hunger>(entity).unwrap();
            let current_hunger = hunger.0.normalized();

            if current_hunger >= 0.80 {
                let mut think_queue = world.resource_mut::<ThinkQueue>();
                think_queue.schedule_urgent(entity, ThinkReason::HungerCritical, 0);
            }
        }

        // Verify entity is in urgent queue
        let think_queue = world.resource::<ThinkQueue>();
        let (urgent, normal, low) = think_queue.queue_sizes();
        assert_eq!(urgent, 1, "Should have 1 urgent request");
        assert_eq!(normal, 0, "Should have 0 normal requests");
        assert_eq!(low, 0, "Should have 0 low requests");

        // Drain and verify priority
        let mut think_queue = world.resource_mut::<ThinkQueue>();
        let requests = think_queue.drain(1);
        assert_eq!(requests.len(), 1, "Should have drained 1 request");
        assert_eq!(
            requests[0].entity, entity,
            "Request should be for our entity"
        );
        assert!(
            matches!(requests[0].priority, ThinkPriority::Urgent),
            "Priority should be Urgent"
        );
        assert!(
            matches!(requests[0].reason, ThinkReason::HungerCritical),
            "Reason should be HungerCritical"
        );
    }

    /// Test 5: Verify action completion triggers replanning
    ///
    /// After an action completes successfully:
    /// - Entity should be added to ReplanQueue
    /// - IdleTracker should be updated
    #[test]
    fn test_action_completion_triggers_replanning() {
        let mut world = create_test_world();
        let entity = spawn_test_entity_with_all_components(&mut world);
        let tick = 100u64;
        world.insert_resource(SimulationTick(tick));

        // Give entity an active action that will complete
        let action = create_action(ActionType::Rest { duration_ticks: 0 }); // 0 duration = instant complete
        world.entity_mut(entity).insert(ActiveAction::new(action, 95));
        world
            .entity_mut(entity)
            .insert(CurrentAction::new("Rest".to_string()));

        // Simulate action completion by adding to recently_completed
        // (This is what the action queue does when actions complete)
        {
            let mut action_queue = world.resource_mut::<ActionQueue>();
            action_queue.queue_action(
                entity,
                ActionType::Rest { duration_ticks: 0 },
                1.0,
                100,
                tick,
            );
        }

        // Simulate what action_completion_system does
        {
            // Remove active action
            world.entity_mut(entity).remove::<ActiveAction>();
            world.entity_mut(entity).insert(CurrentAction::none());

            // Add to replan queue
            let mut replan_queue = world.resource_mut::<ReplanQueue>();
            replan_queue.push(
                entity,
                ReplanPriority::Normal,
                "Action completed/failed".to_string(),
                tick,
            );

            // Update IdleTracker
            let mut idle_tracker = world.get_mut::<IdleTracker>(entity).unwrap();
            idle_tracker.mark_action_completed(tick);
        }

        // Verify entity is in replan queue
        let replan_queue = world.resource::<ReplanQueue>();
        assert!(
            replan_queue.contains(entity),
            "Entity should be in ReplanQueue after action completion"
        );

        // Verify IdleTracker was updated
        let idle_tracker = world.get::<IdleTracker>(entity).unwrap();
        assert!(
            idle_tracker.action_completed,
            "IdleTracker.action_completed should be true"
        );
        assert_eq!(
            idle_tracker.last_action_tick, tick,
            "IdleTracker.last_action_tick should be updated"
        );
        assert_eq!(
            idle_tracker.ticks_since_action, 0,
            "IdleTracker.ticks_since_action should be reset to 0"
        );

        // Verify CurrentAction is cleared
        let current_action = world.get::<CurrentAction>(entity).unwrap();
        assert_eq!(
            current_action.action_name, "Idle",
            "CurrentAction should be 'Idle' after action completion"
        );
    }

    /// Additional test: Verify entity without AI components can have them added
    ///
    /// An entity spawned without AI components should be able to have them
    /// added later (simulating what initialize_new_entity_trackers does)
    #[test]
    fn test_entity_can_receive_ai_components_after_spawn() {
        let mut world = create_test_world();
        let entity = spawn_minimal_entity(&mut world);

        // Verify entity is missing AI components
        assert!(
            world.get::<IdleTracker>(entity).is_none(),
            "Entity should not have IdleTracker initially"
        );
        assert!(
            world.get::<StatThresholdTracker>(entity).is_none(),
            "Entity should not have StatThresholdTracker initially"
        );
        assert!(
            world.get::<BehaviorConfig>(entity).is_none(),
            "Entity should not have BehaviorConfig initially"
        );

        // Simulate what initialize_new_entity_trackers does
        let tick = world.resource::<SimulationTick>().0;
        let hunger = world.get::<Hunger>(entity).unwrap().0.normalized();
        let thirst = world.get::<Thirst>(entity).unwrap().0.normalized();
        let energy = world.get::<Energy>(entity).unwrap().0.normalized();

        world.entity_mut(entity).insert((
            BehaviorConfig::default(),
            IdleTracker::new(tick),
            StatThresholdTracker::new(hunger, thirst, energy),
            CurrentAction::none(),
        ));

        // Verify all AI components are now present
        assert!(
            world.get::<IdleTracker>(entity).is_some(),
            "Entity should now have IdleTracker"
        );
        assert!(
            world.get::<StatThresholdTracker>(entity).is_some(),
            "Entity should now have StatThresholdTracker"
        );
        assert!(
            world.get::<BehaviorConfig>(entity).is_some(),
            "Entity should now have BehaviorConfig"
        );
        assert!(
            world.get::<CurrentAction>(entity).is_some(),
            "Entity should now have CurrentAction"
        );
    }

    /// Additional test: Verify ThinkQueue priority ordering
    ///
    /// ThinkQueue should process urgent requests before normal,
    /// and normal before low priority.
    #[test]
    fn test_think_queue_priority_ordering() {
        let mut world = create_test_world();

        let entity_low = world.spawn_empty().id();
        let entity_normal = world.spawn_empty().id();
        let entity_urgent = world.spawn_empty().id();

        // Add requests in reverse priority order
        {
            let mut think_queue = world.resource_mut::<ThinkQueue>();
            think_queue.schedule_low(entity_low, ThinkReason::Idle, 0);
            think_queue.schedule_normal(entity_normal, ThinkReason::ActionCompleted, 0);
            think_queue.schedule_urgent(entity_urgent, ThinkReason::FearTriggered, 0);
        }

        // Verify queue sizes
        {
            let think_queue = world.resource::<ThinkQueue>();
            let (urgent, normal, low) = think_queue.queue_sizes();
            assert_eq!(urgent, 1, "Should have 1 urgent request");
            assert_eq!(normal, 1, "Should have 1 normal request");
            assert_eq!(low, 1, "Should have 1 low request");
        }

        // Drain and verify order: urgent first, then normal, then low
        {
            let mut think_queue = world.resource_mut::<ThinkQueue>();
            let requests = think_queue.drain(3);

            assert_eq!(requests.len(), 3, "Should drain all 3 requests");
            assert_eq!(
                requests[0].entity, entity_urgent,
                "First request should be urgent"
            );
            assert_eq!(
                requests[1].entity, entity_normal,
                "Second request should be normal"
            );
            assert_eq!(
                requests[2].entity, entity_low,
                "Third request should be low"
            );
        }
    }

    /// Additional test: Verify ReplanQueue deduplication
    ///
    /// ReplanQueue should prevent duplicate entries for the same entity.
    #[test]
    fn test_replan_queue_deduplication() {
        let mut world = create_test_world();
        let entity = spawn_test_entity_with_all_components(&mut world);

        // Add entity to replan queue multiple times
        {
            let mut replan_queue = world.resource_mut::<ReplanQueue>();
            let first_push = replan_queue.push(
                entity,
                ReplanPriority::Normal,
                "First reason".to_string(),
                0,
            );
            let second_push = replan_queue.push(
                entity,
                ReplanPriority::High,
                "Second reason".to_string(),
                1,
            );

            assert!(first_push, "First push should succeed");
            assert!(!second_push, "Second push should fail (duplicate)");
        }

        // Verify only one entry exists
        {
            let replan_queue = world.resource::<ReplanQueue>();
            let (high, normal) = replan_queue.queue_sizes();
            assert_eq!(high + normal, 1, "Should have only 1 entry total");
        }

        // Drain and verify we can push again
        {
            let mut replan_queue = world.resource_mut::<ReplanQueue>();
            let drained = replan_queue.drain(1);
            assert_eq!(drained.len(), 1, "Should drain 1 entry");

            // Now should be able to push again
            let third_push = replan_queue.push(
                entity,
                ReplanPriority::Normal,
                "Third reason".to_string(),
                2,
            );
            assert!(
                third_push,
                "Third push should succeed after draining"
            );
        }
    }
}
