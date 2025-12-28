//! Integration tests for Bevy hierarchy migration
//!
//! These tests verify that parent-child relationships work correctly
//! using Bevy's built-in Parent/Children components instead of custom ParentOf/ChildOf.

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use life_simulator::entities::{
    BirthInfo, spawn_rabbit,
};
use life_simulator::ai::{
    establish_parent_child_immediate, remove_parent_child_immediate,
};
use life_simulator::simulation::{SimulationTick, SimulationState};

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
}

fn spawn_test_rabbit(commands: &mut Commands, pos: IVec2) -> Entity {
    spawn_rabbit(commands, "test_rabbit", pos)
}

/// RED: Test that establish_parent_child_immediate adds Bevy's Parent/Children components
#[test]
fn test_bevy_hierarchy_components_added() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let parent;
    let child;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        parent = spawn_test_rabbit(&mut commands, IVec2::new(50, 50));
        child = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    // Pre-check: neither has hierarchy components
    {
        let parent_ref = world.get_entity(parent).unwrap();
        assert!(!parent_ref.contains::<Children>(), "Parent should not have Children component yet");

        let child_ref = world.get_entity(child).unwrap();
        assert!(!child_ref.contains::<Parent>(), "Child should not have Parent component yet");
        assert!(!child_ref.contains::<BirthInfo>(), "Child should not have BirthInfo yet");
    }

    // Establish relationship using Bevy hierarchy
    {
        establish_parent_child_immediate(parent, child, 100, &mut world);
    }

    // Post-check: both have Bevy hierarchy components
    {
        let parent_ref = world.get_entity(parent).unwrap();
        assert!(parent_ref.contains::<Children>(), "Parent should have Children component");

        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 1, "Parent should have 1 child");
            assert!(children.contains(&child), "Children should contain child entity");
        } else {
            panic!("Children component not found on parent");
        }

        let child_ref = world.get_entity(child).unwrap();
        assert!(child_ref.contains::<Parent>(), "Child should have Parent component");
        assert!(child_ref.contains::<BirthInfo>(), "Child should have BirthInfo component");

        if let Some(parent_comp) = child_ref.get::<Parent>() {
            assert_eq!(parent_comp.get(), parent, "Parent component should reference parent entity");
        } else {
            panic!("Parent component not found on child");
        }

        if let Some(birth_info) = child_ref.get::<BirthInfo>() {
            assert_eq!(birth_info.born_tick, 100, "BirthInfo should track birth tick");
        } else {
            panic!("BirthInfo component not found on child");
        }
    }
}

/// RED: Test that multiple children can be added to one parent
#[test]
fn test_bevy_hierarchy_multiple_children() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let parent;
    let child1;
    let child2;
    let child3;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        parent = spawn_test_rabbit(&mut commands, IVec2::new(50, 50));
        child1 = spawn_test_rabbit(&mut commands, IVec2::new(51, 50));
        child2 = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
        child3 = spawn_test_rabbit(&mut commands, IVec2::new(53, 50));
    }
    queue.apply(&mut world);

    // Establish multiple relationships
    {
        establish_parent_child_immediate(parent, child1, 100, &mut world);
        establish_parent_child_immediate(parent, child2, 105, &mut world);
        establish_parent_child_immediate(parent, child3, 110, &mut world);
    }

    // Verify all children are tracked by Bevy's Children component
    {
        let parent_ref = world.get_entity(parent).unwrap();
        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 3, "Parent should have 3 children");
            assert!(children.contains(&child1));
            assert!(children.contains(&child2));
            assert!(children.contains(&child3));
        } else {
            panic!("Children component not found");
        }

        // Verify each child has correct Parent component
        for (child_entity, expected_tick) in [(child1, 100), (child2, 105), (child3, 110)].iter() {
            let child_ref = world.get_entity(*child_entity).unwrap();

            if let Some(parent_comp) = child_ref.get::<Parent>() {
                assert_eq!(parent_comp.get(), parent);
            } else {
                panic!("Parent component not found on child");
            }

            if let Some(birth_info) = child_ref.get::<BirthInfo>() {
                assert_eq!(birth_info.born_tick, *expected_tick);
            } else {
                panic!("BirthInfo not found on child");
            }
        }
    }
}

/// RED: Test that remove_parent_child_immediate removes hierarchy components
#[test]
fn test_bevy_hierarchy_removal() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let parent;
    let child;

    // Spawn and establish relationship
    {
        let mut commands = Commands::new(&mut queue, &world);
        parent = spawn_test_rabbit(&mut commands, IVec2::new(50, 50));
        child = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    establish_parent_child_immediate(parent, child, 100, &mut world);

    // Verify components exist
    {
        let parent_ref = world.get_entity(parent).unwrap();
        assert!(parent_ref.contains::<Children>());
        let child_ref = world.get_entity(child).unwrap();
        assert!(child_ref.contains::<Parent>());
    }

    // Remove relationship
    {
        remove_parent_child_immediate(parent, child, &mut world);
    }

    // Verify components are removed
    {
        let child_ref = world.get_entity(child).unwrap();
        assert!(!child_ref.contains::<Parent>(), "Parent component should be removed from child");

        let parent_ref = world.get_entity(parent).unwrap();
        // Parent's Children component should either be removed or empty
        if let Some(children) = parent_ref.get::<Children>() {
            assert!(!children.contains(&child), "Child should be removed from parent's children list");
        }
    }
}

/// RED: Test that removing one child doesn't affect others
#[test]
fn test_bevy_hierarchy_remove_one_preserves_others() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let parent;
    let child1;
    let child2;
    let child3;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        parent = spawn_test_rabbit(&mut commands, IVec2::new(50, 50));
        child1 = spawn_test_rabbit(&mut commands, IVec2::new(51, 50));
        child2 = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
        child3 = spawn_test_rabbit(&mut commands, IVec2::new(53, 50));
    }
    queue.apply(&mut world);

    // Establish relationships
    {
        establish_parent_child_immediate(parent, child1, 100, &mut world);
        establish_parent_child_immediate(parent, child2, 100, &mut world);
        establish_parent_child_immediate(parent, child3, 100, &mut world);
    }

    // Verify all 3 children
    {
        let parent_ref = world.get_entity(parent).unwrap();
        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 3);
        }
    }

    // Remove one child
    {
        remove_parent_child_immediate(parent, child2, &mut world);
    }

    // Verify others remain
    {
        let parent_ref = world.get_entity(parent).unwrap();
        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 2, "Should have 2 children remaining");
            assert!(children.contains(&child1));
            assert!(!children.contains(&child2));
            assert!(children.contains(&child3));
        }

        // Removed child should not have Parent
        let c2_ref = world.get_entity(child2).unwrap();
        assert!(!c2_ref.contains::<Parent>());

        // Other children should still have Parent
        let c1_ref = world.get_entity(child1).unwrap();
        assert!(c1_ref.contains::<Parent>());

        let c3_ref = world.get_entity(child3).unwrap();
        assert!(c3_ref.contains::<Parent>());
    }
}

/// RED: Test BirthInfo component tracks birth tick correctly
#[test]
fn test_birth_info_tracking() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let parent;
    let child;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        parent = spawn_test_rabbit(&mut commands, IVec2::new(50, 50));
        child = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    // Establish at specific tick
    {
        establish_parent_child_immediate(parent, child, 50, &mut world);
    }

    // Verify tick is recorded in BirthInfo
    {
        let child_ref = world.get_entity(child).unwrap();
        if let Some(birth_info) = child_ref.get::<BirthInfo>() {
            assert_eq!(birth_info.born_tick, 50);
        }
    }

    // Advance time
    world.resource_mut::<SimulationTick>().0 = 150;

    // Calculate age using BirthInfo
    {
        let current_tick = world.get_resource::<SimulationTick>().unwrap().0;
        let child_ref = world.get_entity(child).unwrap();
        if let Some(birth_info) = child_ref.get::<BirthInfo>() {
            let age = current_tick - birth_info.born_tick;
            assert_eq!(age, 100);
        }
    }
}

/// RED: Test that despawning parent doesn't break (Bevy handles this automatically)
#[test]
fn test_bevy_hierarchy_despawn_safety() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let parent;
    let child;

    // Spawn and establish relationship
    {
        let mut commands = Commands::new(&mut queue, &world);
        parent = spawn_test_rabbit(&mut commands, IVec2::new(50, 50));
        child = spawn_test_rabbit(&mut commands, IVec2::new(52, 50));
    }
    queue.apply(&mut world);

    establish_parent_child_immediate(parent, child, 100, &mut world);

    // Verify relationship exists
    {
        let child_ref = world.get_entity(child).unwrap();
        assert!(child_ref.contains::<Parent>());
    }

    // Despawn parent (Bevy should handle cleanup automatically)
    world.despawn(parent);

    // Child should still exist but orphaned
    {
        let child_ref = world.get_entity(child);
        assert!(child_ref.is_some(), "Child should still exist after parent despawn");
        // Note: Bevy's hierarchy system handles orphaned entities gracefully
    }
}
