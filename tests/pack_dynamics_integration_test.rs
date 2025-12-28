//! Integration tests for wolf pack dynamics system
//!
//! Tests the complete pack formation, member management, and cleanup lifecycle

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use life_simulator::entities::{PackLeader, PackMember, TilePosition, GroupType};
use life_simulator::simulation::SimulationTick;
use life_simulator::ai::{
    establish_pack_leadership, add_to_pack, remove_from_pack, dissolve_pack,
    get_pack_members, get_pack_leader,
    is_pack_leader, is_pack_member, is_in_pack, get_pack_size, are_in_same_pack,
};

/// Helper to create a test world with pack-related entities
fn setup_pack_test() -> World {
    let mut world = World::new();
    world.init_resource::<SimulationTick>();
    world
}

/// RED: Test establishing pack leadership creates PackLeader component
#[test]
fn test_pack_formation() {
    let mut world = setup_pack_test();
    let mut queue = CommandQueue::default();

    let leader_entity = world.spawn((
        TilePosition::from_tile(IVec2::ZERO),
    )).id();

    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_pack_leadership(leader_entity, 100, &mut commands);
    }
    queue.apply(&mut world);

    // Verify PackLeader component was added
    assert!(world.get::<PackLeader>(leader_entity).is_some());
    let leader = world.get::<PackLeader>(leader_entity).unwrap();
    assert_eq!(leader.formed_tick, 100);
    assert_eq!(leader.member_count(), 0);
}

/// Test adding members to a pack
#[test]
fn test_add_pack_members() {
    let mut world = setup_pack_test();
    let mut queue = CommandQueue::default();

    let leader_entity = world.spawn((
        TilePosition::from_tile(IVec2::ZERO),
    )).id();

    let member1 = world.spawn((
        TilePosition::from_tile(IVec2::ONE),
    )).id();

    let member2 = world.spawn((
        TilePosition::from_tile(IVec2::new(2, 2)),
    )).id();

    // Establish leadership first
    {
        let mut commands = Commands::new(&mut queue, &world);
        establish_pack_leadership(leader_entity, 100, &mut commands);
    }
    queue.apply(&mut world);

    // Add first member to pack
    {
        let mut commands = Commands::new(&mut queue, &world);
        add_to_pack(member1, leader_entity, 105, &mut commands, &world);
    }
    queue.apply(&mut world);

    // Add second member to pack
    {
        let mut commands = Commands::new(&mut queue, &world);
        add_to_pack(member2, leader_entity, 110, &mut commands, &world);
    }
    queue.apply(&mut world);

    // Verify members are in pack
    assert_eq!(get_pack_size(leader_entity, &world), 3); // leader + 2 members
    assert!(is_pack_member(member1, &world));
    assert!(is_pack_member(member2, &world));
    assert_eq!(get_pack_leader(member1, &world), Some(leader_entity));
    assert_eq!(get_pack_leader(member2, &world), Some(leader_entity));
}

/// Test removing a member from pack
#[test]
fn test_remove_pack_member() {
    let mut world = setup_pack_test();
    let mut queue = CommandQueue::default();

    let leader_entity = world.spawn((
        PackLeader::new(100, GroupType::Pack),
        TilePosition::from_tile(IVec2::ZERO),
    )).id();

    let member1 = world.spawn((
        PackMember::new(leader_entity, 105, GroupType::Pack),
        TilePosition::from_tile(IVec2::ONE),
    )).id();

    let member2 = world.spawn((
        PackMember::new(leader_entity, 110, GroupType::Pack),
        TilePosition::from_tile(IVec2::new(2, 2)),
    )).id();

    // Update leader with members
    if let Some(mut leader) = world.get_mut::<PackLeader>(leader_entity) {
        leader.add_member(member1);
        leader.add_member(member2);
    }

    // Remove one member
    {
        let mut commands = Commands::new(&mut queue, &world);
        remove_from_pack(member1, &mut commands, &world);
    }
    queue.apply(&mut world);

    // Verify member was removed
    let pack = world.get::<PackLeader>(leader_entity).unwrap();
    assert_eq!(pack.member_count(), 1);
    assert!(!world.get::<PackMember>(member1).is_some());
    assert!(world.get::<PackMember>(member2).is_some());
}

/// Test dissolving entire pack
#[test]
fn test_pack_dissolution() {
    let mut world = setup_pack_test();
    let mut queue = CommandQueue::default();

    let leader_entity = world.spawn((
        PackLeader::new(100, GroupType::Pack),
        TilePosition::from_tile(IVec2::ZERO),
    )).id();

    let member1 = world.spawn((
        PackMember::new(leader_entity, 105, GroupType::Pack),
        TilePosition::from_tile(IVec2::ONE),
    )).id();

    let member2 = world.spawn((
        PackMember::new(leader_entity, 110, GroupType::Pack),
        TilePosition::from_tile(IVec2::new(2, 2)),
    )).id();

    // Update leader with members
    if let Some(mut leader) = world.get_mut::<PackLeader>(leader_entity) {
        leader.add_member(member1);
        leader.add_member(member2);
    }

    // Dissolve pack
    {
        let mut commands = Commands::new(&mut queue, &world);
        dissolve_pack(leader_entity, &mut commands, &world);
    }
    queue.apply(&mut world);

    // Verify pack was dissolved
    assert!(!world.get::<PackLeader>(leader_entity).is_some());
    assert!(!world.get::<PackMember>(member1).is_some());
    assert!(!world.get::<PackMember>(member2).is_some());
}

/// Test pack helper functions
#[test]
fn test_pack_query_functions() {
    let mut world = setup_pack_test();
    let leader_entity = world.spawn((
        PackLeader::new(100, GroupType::Pack),
        TilePosition::from_tile(IVec2::ZERO),
    )).id();

    let member1 = world.spawn((
        PackMember::new(leader_entity, 105, GroupType::Pack),
        TilePosition::from_tile(IVec2::ONE),
    )).id();

    let member2 = world.spawn((
        PackMember::new(leader_entity, 110, GroupType::Pack),
        TilePosition::from_tile(IVec2::new(2, 2)),
    )).id();

    // Update leader with members
    if let Some(mut leader) = world.get_mut::<PackLeader>(leader_entity) {
        leader.add_member(member1);
        leader.add_member(member2);
    }

    // Test query functions
    assert!(is_pack_leader(leader_entity, &world));
    assert!(!is_pack_leader(member1, &world));
    assert!(!is_pack_leader(member2, &world));

    assert!(!is_pack_member(leader_entity, &world));
    assert!(is_pack_member(member1, &world));
    assert!(is_pack_member(member2, &world));

    assert!(is_in_pack(leader_entity, &world));
    assert!(is_in_pack(member1, &world));
    assert!(is_in_pack(member2, &world));

    assert_eq!(get_pack_size(leader_entity, &world), 3);
    assert_eq!(get_pack_members(leader_entity, &world).len(), 2);

    assert!(are_in_same_pack(member1, member2, &world));
    assert!(are_in_same_pack(leader_entity, member1, &world));
}

/// Test independent packs don't interfere
#[test]
fn test_multiple_independent_packs() {
    let mut world = setup_pack_test();

    // Create two packs
    let leader1 = world.spawn((
        PackLeader::new(100, GroupType::Pack),
        TilePosition::from_tile(IVec2::ZERO),
    )).id();

    let leader2 = world.spawn((
        PackLeader::new(100, GroupType::Pack),
        TilePosition::from_tile(IVec2::new(10, 10)),
    )).id();

    let member1_1 = world.spawn((
        PackMember::new(leader1, 105, GroupType::Pack),
        TilePosition::from_tile(IVec2::ONE),
    )).id();

    let member2_1 = world.spawn((
        PackMember::new(leader2, 105, GroupType::Pack),
        TilePosition::from_tile(IVec2::new(11, 11)),
    )).id();

    // Update packs
    if let Some(mut pack) = world.get_mut::<PackLeader>(leader1) {
        pack.add_member(member1_1);
    }
    if let Some(mut pack) = world.get_mut::<PackLeader>(leader2) {
        pack.add_member(member2_1);
    }

    // Verify packs are independent
    assert_eq!(get_pack_leader(member1_1, &world), Some(leader1));
    assert_eq!(get_pack_leader(member2_1, &world), Some(leader2));
    assert!(!are_in_same_pack(member1_1, member2_1, &world));
}

/// Test pack size tracking
#[test]
fn test_pack_size_tracking() {
    let mut world = setup_pack_test();
    let leader = world.spawn((
        PackLeader::new(100, GroupType::Pack),
        TilePosition::from_tile(IVec2::ZERO),
    )).id();

    assert_eq!(get_pack_size(leader, &world), 1); // Just leader

    let member1 = world.spawn((
        PackMember::new(leader, 105, GroupType::Pack),
        TilePosition::from_tile(IVec2::ONE),
    )).id();

    if let Some(mut pack) = world.get_mut::<PackLeader>(leader) {
        pack.add_member(member1);
    }

    assert_eq!(get_pack_size(leader, &world), 2); // leader + 1 member

    let member2 = world.spawn((
        PackMember::new(leader, 110, GroupType::Pack),
        TilePosition::from_tile(IVec2::new(2, 2)),
    )).id();

    if let Some(mut pack) = world.get_mut::<PackLeader>(leader) {
        pack.add_member(member2);
    }

    assert_eq!(get_pack_size(leader, &world), 3); // leader + 2 members
}
