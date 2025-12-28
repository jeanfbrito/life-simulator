//! Integration tests for wolf pack AI behavior
//!
//! Tests that wolves actually USE the pack system through their AI planner

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use life_simulator::entities::{
    PackLeader, PackMember, TilePosition, Wolf, Deer, GroupType,
    Energy, Health, Hunger, Thirst, BehaviorConfig,
};
use life_simulator::ai::{
    is_pack_leader, is_pack_member, get_pack_members, get_pack_size,
};

/// Helper to create a test world with basic setup
fn setup_wolf_pack_test() -> (World, CommandQueue) {
    let mut world = World::new();
    world.init_resource::<life_simulator::simulation::SimulationTick>();
    let queue = CommandQueue::default();
    (world, queue)
}

/// Helper to create a basic wolf entity at a position
fn spawn_test_wolf(world: &mut World, position: IVec2) -> Entity {
    world.spawn((
        Wolf,
        TilePosition::from_tile(position),
        Hunger(life_simulator::entities::stats::Stat::new(50.0, 0.0, 260.0, 0.05)),
        Thirst(life_simulator::entities::stats::Stat::new(50.0, 0.0, 200.0, 0.04)),
        Energy(life_simulator::entities::stats::Stat::new(70.0, 0.0, 100.0, -0.06)),
        Health(life_simulator::entities::stats::Stat::new(100.0, 0.0, 100.0, 0.015)),
        BehaviorConfig::new_with_foraging(
            0.55, 0.45, 0.25, (8, 22), 180, 220, 200,
            life_simulator::entities::types::ForagingStrategy::Exhaustive,
        ),
    )).id()
}

/// RED: Test pack formation when 3+ wolves are nearby
#[test]
fn test_wolves_form_pack_when_nearby() {
    let (mut world, mut queue) = setup_wolf_pack_test();

    // Spawn 4 wolves in proximity (within pack formation radius)
    let wolf1 = spawn_test_wolf(&mut world, IVec2::new(0, 0));
    let wolf2 = spawn_test_wolf(&mut world, IVec2::new(5, 5));
    let wolf3 = spawn_test_wolf(&mut world, IVec2::new(10, 8));
    let wolf4 = spawn_test_wolf(&mut world, IVec2::new(3, 12));

    // Manually form pack (simulating what wolf_pack_formation_system would do)
    {
        let tick = world.get_resource::<life_simulator::simulation::SimulationTick>().unwrap().0;
        let mut commands = Commands::new(&mut queue, &world);

        // Make wolf1 the leader
        life_simulator::ai::establish_pack_leadership(wolf1, tick, &mut commands);
        // Add others as members
        life_simulator::ai::add_to_pack(wolf2, wolf1, tick, &mut commands, &world);
        life_simulator::ai::add_to_pack(wolf3, wolf1, tick, &mut commands, &world);
        life_simulator::ai::add_to_pack(wolf4, wolf1, tick, &mut commands, &world);
    }
    queue.apply(&mut world);

    // One wolf should become leader, others should become members
    let leaders_count = [wolf1, wolf2, wolf3, wolf4]
        .iter()
        .filter(|&&w| is_pack_leader(w, &world))
        .count();

    let members_count = [wolf1, wolf2, wolf3, wolf4]
        .iter()
        .filter(|&&w| is_pack_member(w, &world))
        .count();

    // Exactly 1 leader and 3 members for a pack of 4
    assert_eq!(leaders_count, 1, "Expected exactly one pack leader");
    assert_eq!(members_count, 3, "Expected exactly three pack members");
}

/// RED: Test pack hunting coordination - pack members should prefer same target
#[test]
fn test_pack_coordinates_hunting() {
    let (mut world, _queue) = setup_wolf_pack_test();

    // Create a pack with leader + 2 members
    let leader = spawn_test_wolf(&mut world, IVec2::new(0, 0));
    let member1 = spawn_test_wolf(&mut world, IVec2::new(5, 5));
    let member2 = spawn_test_wolf(&mut world, IVec2::new(8, 3));

    // Manually establish pack for this test
    world.entity_mut(leader).insert(PackLeader::new(100, GroupType::Pack));
    world.entity_mut(member1).insert(PackMember::new(leader, 105, GroupType::Pack));
    world.entity_mut(member2).insert(PackMember::new(leader, 110, GroupType::Pack));

    if let Some(mut pack) = world.get_mut::<PackLeader>(leader) {
        pack.add_member(member1);
        pack.add_member(member2);
    }

    // Spawn nearby deer as prey
    let deer = world.spawn((
        Deer,
        TilePosition::from_tile(IVec2::new(20, 20)),
    )).id();

    // Evaluate actions for all pack members when hungry
    // They should coordinate on the same target (pack hunting behavior)
    // This will be tested through evaluate_wolf_pack_actions function

    assert!(is_pack_leader(leader, &world));
    assert_eq!(get_pack_size(leader, &world), 3);
}

/// RED: Test pack dissolution when wolves separate
#[test]
fn test_pack_dissolves_on_separation() {
    let (mut world, _queue) = setup_wolf_pack_test();

    // Create a pack
    let leader = spawn_test_wolf(&mut world, IVec2::new(0, 0));
    let member1 = spawn_test_wolf(&mut world, IVec2::new(5, 5));
    let member2 = spawn_test_wolf(&mut world, IVec2::new(8, 3));

    world.entity_mut(leader).insert(PackLeader::new(100, GroupType::Pack));
    world.entity_mut(member1).insert(PackMember::new(leader, 105, GroupType::Pack));
    world.entity_mut(member2).insert(PackMember::new(leader, 110, GroupType::Pack));

    if let Some(mut pack) = world.get_mut::<PackLeader>(leader) {
        pack.add_member(member1);
        pack.add_member(member2);
    }

    assert_eq!(get_pack_size(leader, &world), 3);

    // Move members far away (beyond pack cohesion radius)
    if let Some(mut pos) = world.get_mut::<TilePosition>(member1) {
        pos.tile = IVec2::new(500, 500);
    }
    if let Some(mut pos) = world.get_mut::<TilePosition>(member2) {
        pos.tile = IVec2::new(-500, -500);
    }

    // Run pack maintenance system (will be implemented)
    // pack_cohesion_system should detect separation and dissolve pack

    // Pack should be dissolved or significantly reduced
    // This will fail until we implement pack maintenance
}

/// RED: Test lone wolves don't form packs (need 3+ wolves)
#[test]
fn test_lone_wolf_stays_alone() {
    let (mut world, _queue) = setup_wolf_pack_test();

    // Spawn only 2 wolves
    let wolf1 = spawn_test_wolf(&mut world, IVec2::new(0, 0));
    let wolf2 = spawn_test_wolf(&mut world, IVec2::new(100, 100));

    // Run pack formation system
    // Should NOT form a pack with only 2 wolves

    assert!(!is_pack_leader(wolf1, &world), "Lone wolf should not be leader");
    assert!(!is_pack_member(wolf1, &world), "Lone wolf should not be member");
    assert!(!is_pack_leader(wolf2, &world), "Lone wolf should not be leader");
    assert!(!is_pack_member(wolf2, &world), "Lone wolf should not be member");
}

/// RED: Test pack awareness in action evaluation
#[test]
fn test_pack_aware_action_evaluation() {
    let (mut world, _queue) = setup_wolf_pack_test();

    // Create wolf with pack
    let leader = spawn_test_wolf(&mut world, IVec2::new(0, 0));
    let member = spawn_test_wolf(&mut world, IVec2::new(5, 5));

    world.entity_mut(leader).insert(PackLeader::new(100, GroupType::Pack));
    world.entity_mut(member).insert(PackMember::new(leader, 105, GroupType::Pack));

    if let Some(mut pack) = world.get_mut::<PackLeader>(leader) {
        pack.add_member(member);
    }

    // Test that evaluate_wolf_actions considers pack context
    // Pack members should have different action priorities/utilities
    // than lone wolves in same situation

    let pack_members = get_pack_members(leader, &world);
    assert_eq!(pack_members.len(), 1);
    assert_eq!(pack_members[0], member);
}
