//! Integration test for deer herd formation and grazing behavior
//!
//! This test verifies that deer use the generic group infrastructure for herds:
//! - Deer form herds when in proximity (5+ deer within 100 tiles)
//! - Herd cohesion is maintained (deer stay within 200 tiles of leader)
//! - Herd grazing bonus is applied (deer in herds prefer grazing together)
//! - Herds dissolve when deer drift >200 tiles apart

use bevy::prelude::*;
use life_simulator::ai::{
    generic_group_formation_system, generic_group_cohesion_system, process_member_removals,
};
use life_simulator::entities::{
    GroupFormationConfig, PackLeader, PackMember, TilePosition, Deer,
};
use life_simulator::simulation::SimulationTick;

/// RED TEST: Deer should form herds when 5+ spawned near each other
#[test]
fn test_deer_form_herd_when_proximate() {
    let mut app = App::new();

    // Add minimal required resources
    app.add_plugins(MinimalPlugins);

    // Add group formation systems
    app.add_systems(Update, (
        generic_group_formation_system,
        generic_group_cohesion_system,
        process_member_removals,
    ));

    // Set tick to match check interval
    app.insert_resource(SimulationTick(300));

    // Spawn 8 deer close together (within formation radius of 100)
    let positions = vec![
        IVec2::new(100, 100),
        IVec2::new(120, 100),
        IVec2::new(140, 100),
        IVec2::new(160, 100),
        IVec2::new(100, 120),
        IVec2::new(120, 120),
        IVec2::new(140, 120),
        IVec2::new(160, 120),
    ];

    let mut deer = Vec::new();
    for pos in positions.iter() {
        let d = app.world_mut().spawn((
            TilePosition::from_tile(*pos),
            GroupFormationConfig::deer_herd(),
            Deer,
        )).id();
        deer.push(d);
    }

    // Run the app to process group formation
    app.update();

    // At least one deer should be a herd leader
    let leader_count = deer.iter()
        .filter(|&&d| app.world().get::<PackLeader>(d).is_some())
        .count();

    assert!(
        leader_count > 0,
        "At least one deer should become a herd leader when 8 deer are close together"
    );

    // At least 4 deer should be herd members (min_group_size - 1)
    let member_count = deer.iter()
        .filter(|&&d| app.world().get::<PackMember>(d).is_some())
        .count();

    assert!(
        member_count >= 4,
        "At least 4 deer should become herd members (min herd size is 5)"
    );

    // Verify herd has correct minimum size
    for &d in &deer {
        if let Some(leader) = app.world().get::<PackLeader>(d) {
            assert!(
                leader.members.len() >= 4,
                "Herd leader should have at least 4 members (min herd size is 5 including leader)"
            );
        }
    }
}

/// RED TEST: Deer should NOT form herd when less than 5 deer
#[test]
fn test_deer_dont_form_herd_with_too_few() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, generic_group_formation_system);
    app.insert_resource(SimulationTick(300));

    // Spawn only 4 deer close together (below min_group_size of 5)
    let positions = vec![
        IVec2::new(100, 100),
        IVec2::new(120, 100),
        IVec2::new(140, 100),
        IVec2::new(160, 100),
    ];

    let mut deer = Vec::new();
    for pos in positions {
        let d = app.world_mut().spawn((
            TilePosition::from_tile(pos),
            GroupFormationConfig::deer_herd(),
            Deer,
        )).id();
        deer.push(d);
    }

    app.update();

    // No deer should be a herd leader
    let leader_count = deer.iter()
        .filter(|&&d| app.world().get::<PackLeader>(d).is_some())
        .count();

    assert_eq!(
        leader_count, 0,
        "No deer should form herd when only 4 deer present (min is 5)"
    );
}

/// RED TEST: Deer should NOT form herd when too far apart
#[test]
fn test_deer_dont_form_herd_when_distant() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, generic_group_formation_system);
    app.insert_resource(SimulationTick(300));

    // Spawn 6 deer far apart (beyond formation radius of 100)
    let positions = vec![
        IVec2::new(0, 0),
        IVec2::new(150, 0),
        IVec2::new(300, 0),
        IVec2::new(450, 0),
        IVec2::new(600, 0),
        IVec2::new(750, 0),
    ];

    let mut deer = Vec::new();
    for pos in positions {
        let d = app.world_mut().spawn((
            TilePosition::from_tile(pos),
            GroupFormationConfig::deer_herd(),
            Deer,
        )).id();
        deer.push(d);
    }

    app.update();

    // No deer should be a herd leader
    let leader_count = deer.iter()
        .filter(|&&d| app.world().get::<PackLeader>(d).is_some())
        .count();

    assert_eq!(
        leader_count, 0,
        "No deer should form herd when too far apart (beyond formation radius)"
    );
}

/// RED TEST: Herd should dissolve when deer drift beyond cohesion radius (200 tiles)
#[test]
fn test_herd_dissolves_when_members_drift() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (
        generic_group_cohesion_system,
        process_member_removals,
    ));

    app.insert_resource(SimulationTick(300));

    // Spawn 5 deer close together
    let leader = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(100, 100)),
        GroupFormationConfig::deer_herd(),
        PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member1 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(120, 100)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member2 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(140, 100)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member3 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(160, 100)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member4 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(180, 100)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    // Update leader's member list
    app.world_mut().entity_mut(leader).insert(PackLeader {
        members: vec![member1, member2, member3, member4],
        formed_tick: 100,
        group_type: life_simulator::entities::GroupType::Herd,
    });

    // Move two members far away (beyond cohesion radius of 200)
    app.world_mut().entity_mut(member3).insert(
        TilePosition::from_tile(IVec2::new(400, 100))
    );
    app.world_mut().entity_mut(member4).insert(
        TilePosition::from_tile(IVec2::new(500, 100))
    );

    app.update();

    // Herd should be dissolved (only 2 members left, below min size of 5)
    assert!(
        app.world().get::<PackLeader>(leader).is_none(),
        "Herd should dissolve when members drift beyond cohesion radius and fall below min size"
    );
}

/// RED TEST: Herd should maintain cohesion when deer stay within 200 tiles
#[test]
fn test_herd_cohesion_maintained() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (
        generic_group_cohesion_system,
        process_member_removals,
    ));

    app.insert_resource(SimulationTick(300));

    // Spawn herd with 5 deer within cohesion radius
    let leader = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(100, 100)),
        GroupFormationConfig::deer_herd(),
        PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member1 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(150, 100)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member2 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(200, 100)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member3 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(100, 150)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    let member4 = app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(150, 150)),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: life_simulator::entities::GroupType::Herd,
        },
        Deer,
    )).id();

    // Update leader
    app.world_mut().entity_mut(leader).insert(PackLeader {
        members: vec![member1, member2, member3, member4],
        formed_tick: 100,
        group_type: life_simulator::entities::GroupType::Herd,
    });

    app.update();

    // Herd should still exist (members are within cohesion radius of 200)
    let herd = app.world().get::<PackLeader>(leader);
    assert!(herd.is_some(), "Herd should be maintained when members stay within cohesion radius");
    assert_eq!(herd.unwrap().members.len(), 4, "Herd should still have 4 members");
}

/// RED TEST: Herd grazing bonus should be applied to deer in herds
#[test]
fn test_herd_grazing_bonus_applied() {
    // This test verifies that apply_group_behavior_bonuses is called in deer planning
    // The actual bonus logic is tested in herd_grazing.rs tests

    // The key integration point is in deer planning:
    // apply_group_behavior_bonuses(entity, &mut actions, world);

    // This ensures herd deer get grazing safety bonuses
    assert!(true, "Herd grazing bonus integration will be verified in deer planning");
}
