//! Integration tests for generic group formation infrastructure
//!
//! These tests verify that the generic group formation, cohesion, and coordination
//! systems work correctly with different species configurations.

use bevy::prelude::*;
use life_simulator::entities::{
    GroupFormationConfig, GroupType, PackLeader, PackMember, TilePosition,
};
use life_simulator::simulation::SimulationTick;
use life_simulator::ai::{
    generic_group_formation_system, generic_group_cohesion_system, process_member_removals,
};

/// Test wolf pack formation using generic system
#[test]
fn test_wolf_pack_formation_generic() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300)); // Tick that matches check interval

    // Spawn 4 wolves close together with wolf pack config
    let config = GroupFormationConfig::wolf_pack();

    let wolves: Vec<Entity> = (0..4)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TilePosition::from_tile(IVec2::new(i * 10, 0)),
                    config.clone(),
                ))
                .id()
        })
        .collect();

    // Run formation system
    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // Should form one pack with 4 members
    let mut pack_count = 0;
    let mut leader_entity = None;

    for &wolf in &wolves {
        if app.world().get::<PackLeader>(wolf).is_some() {
            pack_count += 1;
            leader_entity = Some(wolf);
        }
    }

    assert_eq!(pack_count, 1, "Should form exactly one pack");

    let leader = app
        .world()
        .get::<PackLeader>(leader_entity.unwrap())
        .unwrap();
    assert_eq!(leader.members.len(), 3, "Pack should have 3 members");
    assert_eq!(leader.group_type, GroupType::Pack);
}

/// Test deer herd formation using generic system
#[test]
fn test_deer_herd_formation_generic() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300)); // Tick that matches check interval

    // Spawn 6 deer close together with deer herd config
    let config = GroupFormationConfig::deer_herd();

    let deer: Vec<Entity> = (0..6)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TilePosition::from_tile(IVec2::new(i * 15, 0)),
                    config.clone(),
                ))
                .id()
        })
        .collect();

    // Run formation system
    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // Should form one herd with 6 members
    let mut herd_count = 0;
    let mut leader_entity = None;

    for &deer_entity in &deer {
        if app.world().get::<PackLeader>(deer_entity).is_some() {
            herd_count += 1;
            leader_entity = Some(deer_entity);
        }
    }

    assert_eq!(herd_count, 1, "Should form exactly one herd");

    let leader = app
        .world()
        .get::<PackLeader>(leader_entity.unwrap())
        .unwrap();
    assert_eq!(leader.members.len(), 5, "Herd should have 5 members");
    assert_eq!(leader.group_type, GroupType::Herd);
}

/// Test rabbit warren formation using generic system
#[test]
fn test_rabbit_warren_formation_generic() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(200)); // Tick that matches rabbit check interval

    // Spawn 5 rabbits close together with warren config
    let config = GroupFormationConfig::rabbit_warren();

    let rabbits: Vec<Entity> = (0..5)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TilePosition::from_tile(IVec2::new(i * 5, 0)),
                    config.clone(),
                ))
                .id()
        })
        .collect();

    // Run formation system
    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // Should form one warren with 5 members
    let mut warren_count = 0;
    let mut leader_entity = None;

    for &rabbit in &rabbits {
        if app.world().get::<PackLeader>(rabbit).is_some() {
            warren_count += 1;
            leader_entity = Some(rabbit);
        }
    }

    assert_eq!(warren_count, 1, "Should form exactly one warren");

    let leader = app
        .world()
        .get::<PackLeader>(leader_entity.unwrap())
        .unwrap();
    assert_eq!(leader.members.len(), 4, "Warren should have 4 members");
    assert_eq!(leader.group_type, GroupType::Warren);
}

/// Test group cohesion system dissolves groups when members drift
#[test]
fn test_group_cohesion_dissolves_groups() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300));

    let config = GroupFormationConfig::wolf_pack();

    // Create a pack with leader and 2 members
    let leader = app
        .world_mut()
        .spawn((
            TilePosition::from_tile(IVec2::new(0, 0)),
            config.clone(),
        ))
        .id();

    let member1 = app
        .world_mut()
        .spawn(TilePosition::from_tile(IVec2::new(50, 0)))
        .id();

    let member2 = app
        .world_mut()
        .spawn(TilePosition::from_tile(IVec2::new(200, 0))) // Too far!
        .id();

    // Set up pack relationships
    app.world_mut().entity_mut(leader).insert(PackLeader {
        members: vec![member1, member2],
        formed_tick: 0,
        group_type: GroupType::Pack,
    });

    app.world_mut().entity_mut(member1).insert(PackMember {
        leader,
        joined_tick: 0,
        group_type: GroupType::Pack,
    });

    app.world_mut().entity_mut(member2).insert(PackMember {
        leader,
        joined_tick: 0,
        group_type: GroupType::Pack,
    });

    // Run cohesion and member removal systems
    app.add_systems(
        Update,
        (generic_group_cohesion_system, process_member_removals).chain(),
    );
    app.update();

    // member2 should be removed, group should dissolve (only 1 member left, min is 3)
    assert!(
        app.world().get::<PackLeader>(leader).is_none(),
        "Group should be dissolved"
    );
    assert!(
        app.world().get::<PackMember>(member1).is_none(),
        "Member1 should no longer be in pack"
    );
}

/// Test that species don't form mixed groups
#[test]
fn test_no_mixed_species_groups() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300));

    let wolf_config = GroupFormationConfig::wolf_pack();
    let deer_config = GroupFormationConfig::deer_herd();

    // Spawn 3 wolves and 3 deer at same location
    let wolves: Vec<Entity> = (0..3)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TilePosition::from_tile(IVec2::new(i * 10, 0)),
                    wolf_config.clone(),
                ))
                .id()
        })
        .collect();

    let deer: Vec<Entity> = (0..3)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TilePosition::from_tile(IVec2::new(i * 10, 10)),
                    deer_config.clone(),
                ))
                .id()
        })
        .collect();

    // Run formation system
    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // Count groups by type
    let mut pack_leaders = 0;
    let mut herd_leaders = 0;

    for &wolf in &wolves {
        if let Some(leader) = app.world().get::<PackLeader>(wolf) {
            if leader.group_type == GroupType::Pack {
                pack_leaders += 1;
            }
        }
    }

    for &deer_entity in &deer {
        if let Some(leader) = app.world().get::<PackLeader>(deer_entity) {
            if leader.group_type == GroupType::Herd {
                herd_leaders += 1;
            }
        }
    }

    // Wolves should form their own pack (3 wolves = min size)
    assert_eq!(pack_leaders, 1, "Wolves should form one pack");

    // Deer should NOT form a herd (only 3, min is 5)
    assert_eq!(herd_leaders, 0, "Deer should not form herd (too few)");
}

/// Test disabled group formation
#[test]
fn test_disabled_group_formation() {
    let mut app = App::new();
    app.insert_resource(SimulationTick(300));

    let mut config = GroupFormationConfig::wolf_pack();
    config.enabled = false; // Disable formation

    // Spawn 4 wolves close together
    let wolves: Vec<Entity> = (0..4)
        .map(|i| {
            app.world_mut()
                .spawn((
                    TilePosition::from_tile(IVec2::new(i * 10, 0)),
                    config.clone(),
                ))
                .id()
        })
        .collect();

    // Run formation system
    app.add_systems(Update, generic_group_formation_system);
    app.update();

    // Should NOT form any groups
    for &wolf in &wolves {
        assert!(
            app.world().get::<PackLeader>(wolf).is_none(),
            "No groups should form when disabled"
        );
        assert!(
            app.world().get::<PackMember>(wolf).is_none(),
            "No groups should form when disabled"
        );
    }
}
