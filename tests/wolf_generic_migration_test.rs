//! Wolf Generic Group Migration Test
//!
//! Tests that wolves with GroupFormationConfig can use the generic group formation system.
//! This validates the migration from wolf-specific pack formation to generic system.

use bevy::prelude::*;
use life_simulator::ai::{generic_group_formation_system, generic_group_cohesion_system, process_member_removals};
use life_simulator::entities::{
    GroupFormationConfig, PackLeader, PackMember, TilePosition, Wolf, GroupType,
};
use life_simulator::simulation::SimulationTick;

/// Helper to spawn a wolf with GroupFormationConfig
fn spawn_wolf_with_config(world: &mut World, position: IVec2) -> Entity {
    world.spawn((
        Wolf,
        TilePosition::from_tile(position),
        GroupFormationConfig::wolf_pack(),
    )).id()
}

/// RED: Test wolves spawn with GroupFormationConfig
#[test]
fn test_wolves_have_group_formation_config() {
    let mut world = World::new();

    let wolf = spawn_wolf_with_config(&mut world, IVec2::new(0, 0));

    let config = world.get::<GroupFormationConfig>(wolf);
    assert!(
        config.is_some(),
        "Wolves should have GroupFormationConfig component"
    );

    let config = config.unwrap();
    assert!(config.enabled, "Wolf group formation should be enabled");
    assert_eq!(
        config.min_group_size, 3,
        "Wolves should require 3 for pack formation"
    );
    assert_eq!(
        config.max_group_size, 8,
        "Wolves should have max pack size of 8"
    );
}

/// RED: Test generic system forms wolf packs
#[test]
fn test_generic_system_forms_wolf_packs() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300)); // Matches check interval

    // Spawn 3 wolves close together (within formation_radius of 50)
    let wolf1 = spawn_wolf_with_config(app.world_mut(), IVec2::new(0, 0));
    let wolf2 = spawn_wolf_with_config(app.world_mut(), IVec2::new(20, 0));
    let wolf3 = spawn_wolf_with_config(app.world_mut(), IVec2::new(40, 0));

    // Run generic group formation system
    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // At least one wolf should be a pack leader
    let has_leader = app
        .world()
        .get::<PackLeader>(wolf1)
        .or_else(|| app.world().get::<PackLeader>(wolf2))
        .or_else(|| app.world().get::<PackLeader>(wolf3))
        .is_some();

    assert!(has_leader, "Generic system should form wolf pack");

    // At least one wolf should be a pack member
    let has_member = app
        .world()
        .get::<PackMember>(wolf1)
        .or_else(|| app.world().get::<PackMember>(wolf2))
        .or_else(|| app.world().get::<PackMember>(wolf3))
        .is_some();

    assert!(has_member, "Generic system should create pack members");
}

/// RED: Test pack has correct GroupType::Pack
#[test]
fn test_wolf_pack_has_correct_group_type() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300));

    // Spawn 3 wolves
    let wolf1 = spawn_wolf_with_config(app.world_mut(), IVec2::new(0, 0));
    let wolf2 = spawn_wolf_with_config(app.world_mut(), IVec2::new(20, 0));
    let wolf3 = spawn_wolf_with_config(app.world_mut(), IVec2::new(40, 0));

    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // Find the leader
    let leader = if let Some(leader) = app.world().get::<PackLeader>(wolf1) {
        Some((wolf1, leader))
    } else if let Some(leader) = app.world().get::<PackLeader>(wolf2) {
        Some((wolf2, leader))
    } else if let Some(leader) = app.world().get::<PackLeader>(wolf3) {
        Some((wolf3, leader))
    } else {
        None
    };

    assert!(leader.is_some(), "Should have formed a pack");

    let (_, pack_leader) = leader.unwrap();
    assert_eq!(
        pack_leader.group_type,
        GroupType::Pack,
        "Wolf groups should have GroupType::Pack"
    );
}

/// RED: Test pack formation respects formation radius
#[test]
fn test_wolf_pack_formation_radius() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300));

    // Spawn 3 wolves - 2 close, 1 far away (beyond 50 tile radius)
    let wolf1 = spawn_wolf_with_config(app.world_mut(), IVec2::new(0, 0));
    let wolf2 = spawn_wolf_with_config(app.world_mut(), IVec2::new(20, 0));
    let wolf3 = spawn_wolf_with_config(app.world_mut(), IVec2::new(200, 0)); // Far away

    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // Wolves 1 and 2 should NOT form pack (need 3 minimum)
    // Wolf 3 should be alone
    assert!(
        app.world().get::<PackLeader>(wolf1).is_none(),
        "Should not form pack with only 2 wolves"
    );
    assert!(
        app.world().get::<PackLeader>(wolf2).is_none(),
        "Should not form pack with only 2 wolves"
    );
    assert!(
        app.world().get::<PackLeader>(wolf3).is_none(),
        "Distant wolf should not be in pack"
    );
}

/// RED: Test cohesion system works with wolves
#[test]
fn test_wolf_pack_cohesion() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300));

    let config = GroupFormationConfig::wolf_pack();

    // Create a pack manually
    let leader = app
        .world_mut()
        .spawn((
            Wolf,
            PackLeader {
                members: vec![],
                formed_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(0, 0)),
            config.clone(),
        ))
        .id();

    let member_close = app
        .world_mut()
        .spawn((
            Wolf,
            PackMember {
                leader,
                joined_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(50, 0)), // Within cohesion radius
        ))
        .id();

    let member_far = app
        .world_mut()
        .spawn((
            Wolf,
            PackMember {
                leader,
                joined_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(200, 0)), // Beyond cohesion radius (150)
        ))
        .id();

    // Update leader's member list
    app.world_mut().entity_mut(leader).insert(PackLeader {
        members: vec![member_close, member_far],
        formed_tick: 0,
        group_type: GroupType::Pack,
    });

    app.add_systems(Update, (generic_group_cohesion_system, process_member_removals));
    app.update();

    // Far member should be marked for removal
    // Group should be dissolved (only 1 member remaining < min_group_size - 1)
    assert!(
        app.world().get::<PackLeader>(leader).is_none(),
        "Pack should be dissolved when members drift too far"
    );
}
