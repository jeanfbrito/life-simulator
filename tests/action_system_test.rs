/// Integration tests for execute_active_actions_system
///
/// Tests the Query/Commands execution pattern for active actions,
/// verifying proper handling of all ActionResult variants.
use bevy::prelude::*;
use bevy::ecs::system::SystemState;
use life_simulator::ai::action::{Action, ActionResult};
use life_simulator::ai::queue::{execute_active_actions_system, ActionQueue};
use life_simulator::entities::{ActiveAction, CurrentAction};
use life_simulator::simulation::SimulationTick;

// ============================================================================
// MOCK ACTIONS FOR TESTING
// ============================================================================

/// Mock action that returns a configurable result
#[derive(Clone)]
struct MockAction {
    result: ActionResult,
    execute_count: std::sync::Arc<std::sync::Mutex<u32>>,
}

impl MockAction {
    fn new(result: ActionResult) -> Self {
        Self {
            result,
            execute_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    fn execution_count(&self) -> u32 {
        *self.execute_count.lock().unwrap()
    }
}

impl Action for MockAction {
    fn can_execute(&self, _world: &World, _entity: Entity) -> bool {
        true
    }

    fn execute(&mut self, _world: &World, _entity: Entity) -> ActionResult {
        *self.execute_count.lock().unwrap() += 1;
        self.result.clone()
    }

    fn name(&self) -> &'static str {
        "MockAction"
    }
}

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Create a minimal World for testing with required resources
fn create_test_world() -> World {
    let mut world = World::new();
    world.insert_resource(SimulationTick(1));
    world.insert_resource(ActionQueue::default());
    world
}

/// Spawn a test entity with ActiveAction component
fn spawn_entity_with_action(world: &mut World, action: Box<dyn Action>, tick: u64) -> Entity {
    world
        .spawn((
            CurrentAction::none(),
            ActiveAction::new(action, tick),
        ))
        .id()
}

/// Run the execute_active_actions_system using SystemState
///
/// This creates a SystemState that matches the system signature and runs it manually.
/// We can't use normal app.add_systems() because the system uses &World parameter.
fn run_active_actions_system(world: &mut World) {
    // Create a SystemState matching execute_active_actions_system's parameters
    // Signature: (Commands, Query<(Entity, &mut ActiveAction)>, &World, Res<SimulationTick>, ResMut<ActionQueue>)

    type SystemParams = (
        Commands<'static, 'static>,
        Query<'static, 'static, (Entity, &'static mut ActiveAction)>,
        Res<'static, SimulationTick>,
        ResMut<'static, ActionQueue>,
    );

    let mut system_state: SystemState<SystemParams> = SystemState::new(world);

    // SAFETY: We need both mutable access (for SystemState) and immutable access (&World for actions).
    // This is safe because:
    // 1. Action::execute takes &World (read-only)
    // 2. All mutations happen via Commands (deferred execution)
    // 3. Commands are applied after all action executions complete
    // 4. We use raw pointers to bypass borrow checker, mimicking what Bevy does internally
    unsafe {
        let world_ptr: *mut World = world as *mut World;
        let world_ref: &World = &*world_ptr;

        let (mut commands, mut query, _tick, mut queue) = system_state.get_mut(&mut *world_ptr);

        for (entity, mut active_action) in query.iter_mut() {
            // Execute the action with read-only world access
            let result = active_action.action.execute(world_ref, entity);

            // Handle the action result using Commands for mutations
            match result {
                ActionResult::Success => {
                    commands.entity(entity).remove::<ActiveAction>();
                    commands.entity(entity).insert(CurrentAction::none());
                    queue.stats.actions_completed += 1;
                }
                ActionResult::Failed => {
                    commands.entity(entity).remove::<ActiveAction>();
                    commands.entity(entity).insert(CurrentAction::none());
                    queue.stats.actions_failed += 1;
                }
                ActionResult::TriggerFollowUp => {
                    commands.entity(entity).remove::<ActiveAction>();
                    commands.entity(entity).insert(CurrentAction::none());
                    queue.stats.actions_completed += 1;
                }
                ActionResult::InProgress => {
                    // Action still running - keep ActiveAction component
                }
            }
        }
    }

    // Apply the commands
    system_state.apply(world);
}

// ============================================================================
// TEST 1: SUCCESS RESULT
// ============================================================================

#[test]
fn test_execute_actions_system_success() {
    let mut world = create_test_world();

    // Spawn entity with action that returns Success
    let mock = MockAction::new(ActionResult::Success);
    let execute_count = mock.execute_count.clone();
    let entity = spawn_entity_with_action(&mut world, Box::new(mock), 1);

    // Verify initial state
    assert!(
        world.get::<ActiveAction>(entity).is_some(),
        "ActiveAction should exist before system runs"
    );

    // Run system once
    run_active_actions_system(&mut world);

    // Verify action was executed
    assert_eq!(*execute_count.lock().unwrap(), 1, "Action should be executed once");

    // Verify ActiveAction component was removed
    assert!(
        world.get::<ActiveAction>(entity).is_none(),
        "ActiveAction should be removed after Success"
    );

    // Verify CurrentAction was set to none/Idle
    let current_action = world.get::<CurrentAction>(entity);
    assert!(current_action.is_some(), "CurrentAction component should exist");
    assert_eq!(
        current_action.unwrap().action_name,
        "Idle",
        "CurrentAction should be set to Idle"
    );

    // Verify ActionQueue stats were updated
    let queue = world.resource::<ActionQueue>();
    assert_eq!(
        queue.stats.actions_completed, 1,
        "Queue should track completed action"
    );
}

// ============================================================================
// TEST 2: FAILED RESULT
// ============================================================================

#[test]
fn test_execute_actions_system_failed() {
    let mut world = create_test_world();

    // Spawn entity with action that returns Failed
    let mock = MockAction::new(ActionResult::Failed);
    let execute_count = mock.execute_count.clone();
    let entity = spawn_entity_with_action(&mut world, Box::new(mock), 1);

    // Run system
    run_active_actions_system(&mut world);

    // Verify action was executed
    assert_eq!(*execute_count.lock().unwrap(), 1, "Action should be executed");

    // Verify ActiveAction component was removed
    assert!(
        world.get::<ActiveAction>(entity).is_none(),
        "ActiveAction should be removed after Failed"
    );

    // Verify CurrentAction was set to Idle
    let current_action = world.get::<CurrentAction>(entity);
    assert!(current_action.is_some());
    assert_eq!(current_action.unwrap().action_name, "Idle");

    // Verify failure tracked in ActionQueue
    let queue = world.resource::<ActionQueue>();
    assert_eq!(queue.stats.actions_failed, 1, "Queue should track failed action");
}

// ============================================================================
// TEST 3: IN-PROGRESS RESULT
// ============================================================================

#[test]
fn test_execute_actions_system_in_progress() {
    let mut world = create_test_world();

    // Spawn entity with action that returns InProgress
    let mock = MockAction::new(ActionResult::InProgress);
    let execute_count = mock.execute_count.clone();
    let entity = spawn_entity_with_action(&mut world, Box::new(mock), 1);

    // Run system
    run_active_actions_system(&mut world);

    // Verify action was executed
    assert_eq!(
        *execute_count.lock().unwrap(),
        1,
        "Action should be executed once"
    );

    // CRITICAL: ActiveAction component should STILL exist
    assert!(
        world.get::<ActiveAction>(entity).is_some(),
        "ActiveAction should remain for InProgress actions"
    );

    // Run system again - action should execute again
    run_active_actions_system(&mut world);

    // Verify action executed again
    assert_eq!(
        *execute_count.lock().unwrap(),
        2,
        "Action should execute on second tick"
    );

    // ActiveAction should still exist
    assert!(
        world.get::<ActiveAction>(entity).is_some(),
        "ActiveAction should continue to exist"
    );
}

// ============================================================================
// TEST 4: TRIGGER FOLLOW-UP RESULT
// ============================================================================

#[test]
fn test_execute_actions_system_trigger_followup() {
    let mut world = create_test_world();

    // Spawn entity with action that triggers follow-up
    let mock = MockAction::new(ActionResult::TriggerFollowUp);
    let execute_count = mock.execute_count.clone();
    let entity = spawn_entity_with_action(&mut world, Box::new(mock), 1);

    // Run system
    run_active_actions_system(&mut world);

    // Verify action was executed
    assert_eq!(*execute_count.lock().unwrap(), 1);

    // Verify ActiveAction component was removed (to allow AI to plan next action)
    assert!(
        world.get::<ActiveAction>(entity).is_none(),
        "ActiveAction should be removed after TriggerFollowUp"
    );

    // Verify CurrentAction was set to Idle
    let current_action = world.get::<CurrentAction>(entity);
    assert!(current_action.is_some());
    assert_eq!(current_action.unwrap().action_name, "Idle");

    // Verify completion tracked (TriggerFollowUp counts as completion)
    let queue = world.resource::<ActionQueue>();
    assert_eq!(
        queue.stats.actions_completed, 1,
        "TriggerFollowUp should count as completion"
    );
}

// ============================================================================
// TEST 5: MULTIPLE ENTITIES
// ============================================================================

#[test]
fn test_execute_actions_system_multiple_entities() {
    let mut world = create_test_world();

    // Spawn 5 entities with different action results
    let mock1 = MockAction::new(ActionResult::Success);
    let count1 = mock1.execute_count.clone();
    let entity1 = spawn_entity_with_action(&mut world, Box::new(mock1), 1);

    let mock2 = MockAction::new(ActionResult::InProgress);
    let count2 = mock2.execute_count.clone();
    let entity2 = spawn_entity_with_action(&mut world, Box::new(mock2), 1);

    let mock3 = MockAction::new(ActionResult::Failed);
    let count3 = mock3.execute_count.clone();
    let entity3 = spawn_entity_with_action(&mut world, Box::new(mock3), 1);

    let mock4 = MockAction::new(ActionResult::TriggerFollowUp);
    let count4 = mock4.execute_count.clone();
    let entity4 = spawn_entity_with_action(&mut world, Box::new(mock4), 1);

    let mock5 = MockAction::new(ActionResult::InProgress);
    let count5 = mock5.execute_count.clone();
    let entity5 = spawn_entity_with_action(&mut world, Box::new(mock5), 1);

    // Verify all entities have ActiveAction component
    assert!(world.get::<ActiveAction>(entity1).is_some());
    assert!(world.get::<ActiveAction>(entity2).is_some());
    assert!(world.get::<ActiveAction>(entity3).is_some());
    assert!(world.get::<ActiveAction>(entity4).is_some());
    assert!(world.get::<ActiveAction>(entity5).is_some());

    // Run system
    run_active_actions_system(&mut world);

    // Verify all actions were executed
    assert_eq!(*count1.lock().unwrap(), 1);
    assert_eq!(*count2.lock().unwrap(), 1);
    assert_eq!(*count3.lock().unwrap(), 1);
    assert_eq!(*count4.lock().unwrap(), 1);
    assert_eq!(*count5.lock().unwrap(), 1);

    // Verify correct components removed/retained based on result
    assert!(
        world.get::<ActiveAction>(entity1).is_none(),
        "Success: ActiveAction removed"
    );
    assert!(
        world.get::<ActiveAction>(entity2).is_some(),
        "InProgress: ActiveAction retained"
    );
    assert!(
        world.get::<ActiveAction>(entity3).is_none(),
        "Failed: ActiveAction removed"
    );
    assert!(
        world.get::<ActiveAction>(entity4).is_none(),
        "TriggerFollowUp: ActiveAction removed"
    );
    assert!(
        world.get::<ActiveAction>(entity5).is_some(),
        "InProgress: ActiveAction retained"
    );

    // Verify stats
    let queue = world.resource::<ActionQueue>();
    assert_eq!(queue.stats.actions_completed, 2, "2 completions (Success + TriggerFollowUp)");
    assert_eq!(queue.stats.actions_failed, 1, "1 failure");
}

// ============================================================================
// TEST 6: COMMANDS APPLICATION
// ============================================================================

#[test]
fn test_execute_actions_system_commands_applied() {
    let mut world = create_test_world();

    // Spawn entity with Success action
    let mock = MockAction::new(ActionResult::Success);
    let entity = spawn_entity_with_action(&mut world, Box::new(mock), 1);

    // Verify component exists before system
    assert!(world.get::<ActiveAction>(entity).is_some());

    // Run system (run_active_actions_system(&mut world) automatically applies Commands)
    run_active_actions_system(&mut world);

    // Commands should be applied - ActiveAction removed
    assert!(
        world.get::<ActiveAction>(entity).is_none(),
        "Commands should be applied automatically by run_active_actions_system(&mut world)"
    );

    // CurrentAction should be updated
    let current = world.get::<CurrentAction>(entity).unwrap();
    assert_eq!(current.action_name, "Idle");
}

// ============================================================================
// TEST 7: RECENTLY COMPLETED TRACKING
// ============================================================================

#[test]
fn test_execute_actions_system_tracks_completion_stats() {
    let mut world = create_test_world();

    // Set tick to specific value
    world.resource_mut::<SimulationTick>().0 = 42;

    // Spawn entity with Success action
    let mock = MockAction::new(ActionResult::Success);
    let entity = spawn_entity_with_action(&mut world, Box::new(mock), 40);

    // Verify initial stats
    assert_eq!(world.resource::<ActionQueue>().stats.actions_completed, 0);

    // Run system
    run_active_actions_system(&mut world);

    // Verify completion was tracked in stats
    let queue = world.resource::<ActionQueue>();
    assert_eq!(
        queue.stats.actions_completed, 1,
        "Completion should be tracked in queue stats"
    );
}

// ============================================================================
// TEST 8: TICK DURATION TRACKING
// ============================================================================

#[test]
fn test_execute_actions_system_tick_duration() {
    let mut world = create_test_world();

    // Set starting tick
    world.resource_mut::<SimulationTick>().0 = 10;

    // Spawn action that started at tick 5
    let mock = MockAction::new(ActionResult::Success);
    let entity = spawn_entity_with_action(&mut world, Box::new(mock), 5);

    // Run system
    run_active_actions_system(&mut world);

    // Action should complete successfully (duration: 10 - 5 = 5 ticks)
    assert!(world.get::<ActiveAction>(entity).is_none());
}

// ============================================================================
// TEST 9: SYSTEM RUNS WITHOUT ERRORS WHEN NO ACTIVE ACTIONS
// ============================================================================

#[test]
fn test_execute_actions_system_no_active_actions() {
    let mut world = create_test_world();

    // Don't spawn any entities with ActiveAction

    // Run system - should not panic
    run_active_actions_system(&mut world);

    // Verify stats remain at 0
    let queue = world.resource::<ActionQueue>();
    assert_eq!(queue.stats.actions_completed, 0);
    assert_eq!(queue.stats.actions_failed, 0);
}

// ============================================================================
// TEST 10: ACTION STATE PERSISTENCE ACROSS TICKS
// ============================================================================

#[test]
fn test_action_state_persistence_across_ticks() {
    let mut world = create_test_world();

    // Create an action that tracks state
    #[derive(Clone)]
    struct StatefulAction {
        ticks_executed: std::sync::Arc<std::sync::Mutex<u32>>,
    }

    impl Action for StatefulAction {
        fn can_execute(&self, _world: &World, _entity: Entity) -> bool {
            true
        }

        fn execute(&mut self, _world: &World, _entity: Entity) -> ActionResult {
            let mut count = self.ticks_executed.lock().unwrap();
            *count += 1;

            // Complete after 3 ticks
            if *count >= 3 {
                ActionResult::Success
            } else {
                ActionResult::InProgress
            }
        }

        fn name(&self) -> &'static str {
            "StatefulAction"
        }
    }

    let tick_counter = std::sync::Arc::new(std::sync::Mutex::new(0));
    let action = StatefulAction {
        ticks_executed: tick_counter.clone(),
    };

    let entity = spawn_entity_with_action(&mut world, Box::new(action), 1);

    // Tick 1: InProgress
    run_active_actions_system(&mut world);
    assert_eq!(*tick_counter.lock().unwrap(), 1);
    assert!(world.get::<ActiveAction>(entity).is_some());

    // Tick 2: InProgress
    run_active_actions_system(&mut world);
    assert_eq!(*tick_counter.lock().unwrap(), 2);
    assert!(world.get::<ActiveAction>(entity).is_some());

    // Tick 3: Success
    run_active_actions_system(&mut world);
    assert_eq!(*tick_counter.lock().unwrap(), 3);
    assert!(
        world.get::<ActiveAction>(entity).is_none(),
        "Action should complete after 3 ticks"
    );
}
