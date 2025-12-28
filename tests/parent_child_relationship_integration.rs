//! Integration tests for parent-child relationships using Bevy's hierarchy system
//!
//! Tests that verify parent-child relationships are properly established and cleared
//! during the reproduction lifecycle using Bevy's Parent/Children components + BirthInfo.

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use life_simulator::entities::{
    BirthInfo, spawn_rabbit,
};
use life_simulator::ai::{
    establish_parent_child_immediate, remove_parent_child_immediate,
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
    // WorldLoader is optional for these tests, skip it
}

fn spawn_test_rabbit(commands: &mut Commands, pos: IVec2) -> Entity {
    spawn_rabbit(commands, "test_rabbit", pos)
}

/// RED: Test that establish_parent_child_immediate adds both components
#[test]
fn test_establish_parent_child_relationship_adds_components() {
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

    // Pre-check: neither has parent-child components
    {
        let parent_ref = world.get_entity(parent).unwrap();
        assert!(!parent_ref.contains::<Children>(), "Parent should not have Children component yet");

        let child_ref = world.get_entity(child).unwrap();
        assert!(!child_ref.contains::<ChildOf>(), "Child should not have ChildOf component yet");
        assert!(!child_ref.contains::<BirthInfo>(), "Child should not have BirthInfo yet");
    }

    // Establish relationship using Bevy's hierarchy system
    {
        establish_parent_child_immediate(parent, child, 100, &mut world);
    }

    // Post-check: both have parent-child components (Bevy's hierarchy)
    {
        let parent_ref = world.get_entity(parent).unwrap();
        assert!(parent_ref.contains::<Children>(), "Parent should have Children component");
        if let Some(children) = parent_ref.get::<Children>() {
            assert!(children.contains(&child), "Parent's Children should contain child entity");
        } else {
            panic!("Children component not found on parent");
        }

        let child_ref = world.get_entity(child).unwrap();
        assert!(child_ref.contains::<ChildOf>(), "Child should have ChildOf component");
        if let Some(child_of) = child_ref.get::<ChildOf>() {
            assert_eq!(child_of.parent(), parent, "Child's parent should match");
        } else {
            panic!("ChildOf component not found on child");
        }

        // Check BirthInfo metadata
        assert!(child_ref.contains::<BirthInfo>(), "Child should have BirthInfo component");
        if let Some(birth_info) = child_ref.get::<BirthInfo>() {
            assert_eq!(birth_info.born_tick, 100, "Birth tick should be 100");
        } else {
            panic!("BirthInfo component not found on child");
        }
    }
}

/// RED: Test that remove_parent_child_immediate removes both components
#[test]
fn test_remove_parent_child_relationship_removes_components() {
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

    // First establish relationship
    {
        establish_parent_child_immediate(parent, child, 100, &mut world);
    }

    // Verify components exist (Bevy hierarchy)
    {
        let parent_ref = world.get_entity(parent).unwrap();
        assert!(parent_ref.contains::<Children>());
        let child_ref = world.get_entity(child).unwrap();
        assert!(child_ref.contains::<ChildOf>());
    }

    // Remove relationship
    {
        remove_parent_child_immediate(parent, child, &mut world);
    }

    // Verify components are removed
    {
        let parent_ref = world.get_entity(parent).unwrap();
        // Children component should be removed if no children remain
        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 0);
        }

        let child_ref = world.get_entity(child).unwrap();
        assert!(!child_ref.contains::<ChildOf>());
    }
}

/// RED: Test relationship lifecycle - establish then remove
#[test]
fn test_relationship_lifecycle_establish_and_remove() {
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

    // Step 1: Establish
    {
        establish_parent_child_immediate(parent, child, 50, &mut world);
    }

    // Verify established (Bevy hierarchy + BirthInfo)
    {
        let parent_ref = world.get_entity(parent).unwrap();
        if let Some(children) = parent_ref.get::<Children>() {
            assert!(children.contains(&child), "Parent should have child in Children component");
        } else {
            panic!("Relationship not established");
        }

        let child_ref = world.get_entity(child).unwrap();
        if let Some(birth_info) = child_ref.get::<BirthInfo>() {
            assert_eq!(birth_info.born_tick, 50, "Birth tick should be 50");
        }
    }

    // Step 2: Simulate growth/time passing
    world.resource_mut::<SimulationTick>().0 = 75;

    // Verify still related
    {
        let parent_ref = world.get_entity(parent).unwrap();
        assert!(parent_ref.contains::<Children>(), "Parent should still have Children component");
    }

    // Step 3: Remove after child maturation or death
    {
        remove_parent_child_immediate(parent, child, &mut world);
    }

    // Verify removed
    {
        let child_ref = world.get_entity(child).unwrap();
        assert!(!child_ref.contains::<ChildOf>());
    }
}

/// RED: Test that multiple children can be tracked by one parent
#[test]
fn test_parent_with_multiple_children() {
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
        establish_parent_child_immediate(parent, child2, 100, &mut world);
        establish_parent_child_immediate(parent, child3, 100, &mut world);
    }

    // Verify all children are tracked (Bevy hierarchy)
    {
        let parent_ref = world.get_entity(parent).unwrap();
        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 3, "Parent should have 3 children");
            assert!(children.contains(&child1), "Child1 should be in parent's children");
            assert!(children.contains(&child2), "Child2 should be in parent's children");
            assert!(children.contains(&child3), "Child3 should be in parent's children");
        } else {
            panic!("Children component not found on parent");
        }

        // Verify each child knows its parent
        for child_entity in [child1, child2, child3].iter() {
            let child_ref = world.get_entity(*child_entity).unwrap();
            if let Some(child_of) = child_ref.get::<ChildOf>() {
                assert_eq!(child_of.parent(), parent, "Child's parent should match");
            } else {
                panic!("ChildOf not found on child");
            }
        }
    }
}

/// RED: Test that multiple parents can have different children
#[test]
fn test_multiple_parents_different_children() {
    let (mut world, mut queue) = create_test_world();
    setup_resources(&mut world);

    let parent1;
    let parent2;
    let child1;
    let child2;

    // Spawn entities
    {
        let mut commands = Commands::new(&mut queue, &world);
        parent1 = spawn_test_rabbit(&mut commands, IVec2::new(50, 50));
        parent2 = spawn_test_rabbit(&mut commands, IVec2::new(60, 50));
        child1 = spawn_test_rabbit(&mut commands, IVec2::new(51, 50));
        child2 = spawn_test_rabbit(&mut commands, IVec2::new(61, 50));
    }
    queue.apply(&mut world);

    // Both establish relationships
    {
        establish_parent_child_immediate(parent1, child1, 100, &mut world);
        establish_parent_child_immediate(parent2, child2, 100, &mut world);
    }

    // Verify each parent has their own child (Bevy hierarchy)
    {
        let p1_ref = world.get_entity(parent1).unwrap();
        if let Some(children1) = p1_ref.get::<Children>() {
            assert!(children1.contains(&child1), "Parent1 should have child1");
            assert!(!children1.contains(&child2), "Parent1 should not have child2");
        }

        let p2_ref = world.get_entity(parent2).unwrap();
        if let Some(children2) = p2_ref.get::<Children>() {
            assert!(children2.contains(&child2), "Parent2 should have child2");
            assert!(!children2.contains(&child1), "Parent2 should not have child1");
        }

        // Check children point to correct parents
        let c1_ref = world.get_entity(child1).unwrap();
        if let Some(child_of1) = c1_ref.get::<ChildOf>() {
            assert_eq!(child_of1.parent(), parent1, "Child1's parent should be parent1");
        }

        let c2_ref = world.get_entity(child2).unwrap();
        if let Some(child_of2) = c2_ref.get::<ChildOf>() {
            assert_eq!(child_of2.parent(), parent2, "Child2's parent should be parent2");
        }
    }
}

/// RED: Test birth tick tracking
#[test]
fn test_birth_tick_tracking() {
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

    // Verify tick is recorded (Bevy hierarchy uses BirthInfo)
    {
        let child_ref = world.get_entity(child).unwrap();
        if let Some(birth_info) = child_ref.get::<BirthInfo>() {
            assert_eq!(birth_info.born_tick, 50, "Birth tick should be 50");
        } else {
            panic!("BirthInfo not found on child");
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
            assert_eq!(age, 100, "Age calculation should work with BirthInfo");
        }
    }
}

/// RED: Test relationship validation (bidirectional consistency)
#[test]
fn test_relationship_bidirectional_consistency() {
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

    // Establish relationship
    {
        establish_parent_child_immediate(parent, child, 100, &mut world);
    }

    // Verify bidirectional consistency (Bevy hierarchy + BirthInfo)
    {
        let parent_ref = world.get_entity(parent).unwrap();
        let child_ref = world.get_entity(child).unwrap();

        if let (Some(children), Some(child_of)) = (parent_ref.get::<Children>(), child_ref.get::<ChildOf>()) {
            // Parent's children should include child
            assert!(children.contains(&child), "Parent's Children should contain child");
            // Child's parent should match parent entity
            assert_eq!(child_of.parent(), parent, "Child's ChildOf should point to parent");
        } else {
            panic!("Relationship components not properly established");
        }

        // Verify BirthInfo on child
        if let Some(birth_info) = child_ref.get::<BirthInfo>() {
            assert_eq!(birth_info.born_tick, 100, "Birth tick should be 100");
        } else {
            panic!("BirthInfo not found on child");
        }
    }
}

/// RED: Test removing one child doesn't affect others
#[test]
fn test_remove_one_child_preserves_others() {
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

    // Verify all 3 children (Bevy hierarchy)
    {
        let parent_ref = world.get_entity(parent).unwrap();
        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 3, "Parent should have 3 children");
        }
    }

    // Remove one child
    {
        remove_parent_child_immediate(parent, child2, &mut world);
    }

    // Verify others remain (Bevy hierarchy)
    {
        let parent_ref = world.get_entity(parent).unwrap();
        if let Some(children) = parent_ref.get::<Children>() {
            assert_eq!(children.len(), 2, "Parent should have 2 children after removal");
            assert!(children.contains(&child1), "Parent should still have child1");
            assert!(!children.contains(&child2), "Parent should not have child2");
            assert!(children.contains(&child3), "Parent should still have child3");
        }

        // Removed child should not have ChildOf
        let c2_ref = world.get_entity(child2).unwrap();
        assert!(!c2_ref.contains::<ChildOf>(), "Removed child should not have ChildOf");

        // Other children should still have ChildOf
        let c1_ref = world.get_entity(child1).unwrap();
        assert!(c1_ref.contains::<ChildOf>(), "Child1 should still have ChildOf");

        let c3_ref = world.get_entity(child3).unwrap();
        assert!(c3_ref.contains::<ChildOf>(), "Child3 should still have ChildOf");
    }
}
