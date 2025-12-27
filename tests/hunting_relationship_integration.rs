//! Integration tests for hunting relationships with HuntAction
//!
//! Tests that verify hunting relationships are properly established and cleared
//! during the hunt action lifecycle.

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use life_simulator::entities::{
    ActiveHunter, HuntingTarget, spawn_fox, spawn_rabbit,
};
use life_simulator::ai::{
    ActionQueue, establish_hunting_relationship,
    clear_hunting_relationship,
};
use life_simulator::simulation::{SimulationTick, SimulationState};
use life_simulator::world_loader::WorldLoader;

fn create_test_world() -> (World, CommandQueue) {
    let world = World::new();
    let queue = CommandQueue::default();
    (world, queue)
}

fn setup_resources(world: &mut World) {
    world.insert_resource(SimulationTick(0));
    world.insert_resource(SimulationState {
        should_tick: true,
    });
    world.insert_resource(ActionQueue::default());
    // WorldLoader is optional for these tests, skip it
}

fn spawn_test_fox(commands: &mut Commands) -> Entity {
    spawn_fox(commands, "test_fox", IVec2::new(50, 50))
}

fn spawn_test_rabbit(commands: &mut Commands, pos: IVec2) -> Entity {
    spawn_rabbit(commands, "test_rabbit", pos)
}

/// RED: Test that establish_hunting_relationship adds both components
#[test]
fn test_establish_hunting_relationship_adds_components() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let predator;
    let prey;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        predator = spawn_test_fox(&mut commands);
        prey = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    // Pre-check: neither has hunting components
    {
        let predator_ref = world.get_entity(predator).unwrap();
        assert!(!predator_ref.contains::<ActiveHunter>());

        let prey_ref = world.get_entity(prey).unwrap();
        assert!(!prey_ref.contains::<HuntingTarget>());
    }

    // Establish relationship
    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_hunting_relationship(predator, prey, 100, &mut commands);
    }
    queue.apply(&mut world);

    // Post-check: both have hunting components
    {
        let predator_ref = world.get_entity(predator).unwrap();
        assert!(predator_ref.contains::<ActiveHunter>());
        if let Some(hunter) = predator_ref.get::<ActiveHunter>() {
            assert_eq!(hunter.target, prey);
            assert_eq!(hunter.started_tick, 100);
        } else {
            panic!("ActiveHunter component not found on predator");
        }

        let prey_ref = world.get_entity(prey).unwrap();
        assert!(prey_ref.contains::<HuntingTarget>());
        if let Some(target) = prey_ref.get::<HuntingTarget>() {
            assert_eq!(target.predator, predator);
            assert_eq!(target.started_tick, 100);
        } else {
            panic!("HuntingTarget component not found on prey");
        }
    }
}

/// RED: Test that clear_hunting_relationship removes both components
#[test]
fn test_clear_hunting_relationship_removes_components() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let predator;
    let prey;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        predator = spawn_test_fox(&mut commands);
        prey = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    // First establish relationship
    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_hunting_relationship(predator, prey, 100, &mut commands);
    }
    queue.apply(&mut world);

    // Verify components exist
    {
        let predator_ref = world.get_entity(predator).unwrap();
        assert!(predator_ref.contains::<ActiveHunter>());
        let prey_ref = world.get_entity(prey).unwrap();
        assert!(prey_ref.contains::<HuntingTarget>());
    }

    // Clear relationship
    {
        let mut commands = Commands::new(&mut queue, &world);
        clear_hunting_relationship(predator, prey, &mut commands);
    }
    queue.apply(&mut world);

    // Verify components are removed
    {
        let predator_ref = world.get_entity(predator).unwrap();
        assert!(!predator_ref.contains::<ActiveHunter>());

        let prey_ref = world.get_entity(prey).unwrap();
        assert!(!prey_ref.contains::<HuntingTarget>());
    }
}

/// RED: Test relationship lifecycle - establish then clear
#[test]
fn test_relationship_lifecycle_establish_and_clear() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let predator;
    let prey;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        predator = spawn_test_fox(&mut commands);
        prey = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    // Step 1: Establish
    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_hunting_relationship(predator, prey, 50, &mut commands);
    }
    queue.apply(&mut world);

    // Verify established
    {
        let predator_ref = world.get_entity(predator).unwrap();
        if let Some(hunter) = predator_ref.get::<ActiveHunter>() {
            assert_eq!(hunter.started_tick, 50);
        } else {
            panic!("Hunt not established");
        }
    }

    // Step 2: Simulate hunt progress (would normally happen during action execution)
    world.resource_mut::<SimulationTick>().0 = 75;

    // Verify still hunting
    {
        let predator_ref = world.get_entity(predator).unwrap();
        assert!(predator_ref.contains::<ActiveHunter>());
    }

    // Step 3: Clear after successful hunt
    {
        let mut commands = Commands::new(&mut queue, &world);
        clear_hunting_relationship(predator, prey, &mut commands);
    }
    queue.apply(&mut world);

    // Verify cleared
    {
        let predator_ref = world.get_entity(predator).unwrap();
        assert!(!predator_ref.contains::<ActiveHunter>());
    }
}

/// RED: Test that multiple predators can have different hunts
#[test]
fn test_multiple_predators_different_hunts() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let predator1;
    let predator2;
    let prey1;
    let prey2;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        predator1 = spawn_test_fox(&mut commands);
        predator2 = spawn_test_fox(&mut commands);
        prey1 = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
        prey2 = spawn_test_rabbit(&mut commands, IVec2::new(48, 50));
    }
    queue.apply(&mut world);

    // Both establish hunts
    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_hunting_relationship(predator1, prey1, 100, &mut commands);
        establish_hunting_relationship(predator2, prey2, 100, &mut commands);
    }
    queue.apply(&mut world);

    // Verify each hunts their own prey
    {
        let p1_ref = world.get_entity(predator1).unwrap();
        if let Some(hunter1) = p1_ref.get::<ActiveHunter>() {
            assert_eq!(hunter1.target, prey1);
        }

        let p2_ref = world.get_entity(predator2).unwrap();
        if let Some(hunter2) = p2_ref.get::<ActiveHunter>() {
            assert_eq!(hunter2.target, prey2);
        }

        let prey1_ref = world.get_entity(prey1).unwrap();
        if let Some(target1) = prey1_ref.get::<HuntingTarget>() {
            assert_eq!(target1.predator, predator1);
        }

        let prey2_ref = world.get_entity(prey2).unwrap();
        if let Some(target2) = prey2_ref.get::<HuntingTarget>() {
            assert_eq!(target2.predator, predator2);
        }
    }
}

/// RED: Test that hunt duration tracking works
#[test]
fn test_hunt_duration_tracking() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let predator;
    let prey;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        predator = spawn_test_fox(&mut commands);
        prey = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    // Establish hunt at tick 50
    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_hunting_relationship(predator, prey, 50, &mut commands);
    }
    queue.apply(&mut world);

    // Verify start tick is recorded
    {
        let predator_ref = world.get_entity(predator).unwrap();
        if let Some(hunter) = predator_ref.get::<ActiveHunter>() {
            assert_eq!(hunter.started_tick, 50);
        }

        let prey_ref = world.get_entity(prey).unwrap();
        if let Some(target) = prey_ref.get::<HuntingTarget>() {
            assert_eq!(target.started_tick, 50);
        }
    }

    // Advance time to tick 150
    world.resource_mut::<SimulationTick>().0 = 150;

    // Calculate hunt duration
    {
        let current_tick = world.get_resource::<SimulationTick>().unwrap().0;
        let predator_ref = world.get_entity(predator).unwrap();
        if let Some(hunter) = predator_ref.get::<ActiveHunter>() {
            let duration = current_tick - hunter.started_tick;
            assert_eq!(duration, 100);
        }
    }
}

/// RED: Test relationship validation (bidirectional consistency)
#[test]
fn test_relationship_bidirectional_consistency() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let predator;
    let prey;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        predator = spawn_test_fox(&mut commands);
        prey = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    // Establish relationship
    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_hunting_relationship(predator, prey, 100, &mut commands);
    }
    queue.apply(&mut world);

    // Verify bidirectional consistency
    {
        let predator_ref = world.get_entity(predator).unwrap();
        let prey_ref = world.get_entity(prey).unwrap();

        if let (Some(hunter), Some(target)) = (predator_ref.get::<ActiveHunter>(), prey_ref.get::<HuntingTarget>()) {
            // Predator's target should match prey entity
            assert_eq!(hunter.target, prey);
            // Prey's predator should match predator entity
            assert_eq!(target.predator, predator);
            // Both should have same start tick
            assert_eq!(hunter.started_tick, target.started_tick);
        } else {
            panic!("Relationship components not properly established");
        }
    }
}
