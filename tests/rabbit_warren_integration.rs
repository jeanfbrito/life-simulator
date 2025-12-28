//! Integration tests for rabbit warren formation and defense behaviors
//!
//! This test suite validates that:
//! 1. Rabbits spawn with GroupFormationConfig::rabbit_warren()
//! 2. Rabbits form warrens when 4+ rabbits are within 30 tiles
//! 3. Warren cohesion maintains groups within 100 tiles
//! 4. Warren defense bonuses apply to flee/movement actions
//! 5. Warrens dissolve when members drift beyond cohesion radius

use bevy::prelude::*;
use life_simulator::ai::{ActionType, UtilityScore};
use life_simulator::entities::{
    GroupFormationConfig, GroupType, PackLeader, PackMember, Rabbit, TilePosition,
};
use life_simulator::entities::stats::{Energy, Health, Hunger, Thirst};
use life_simulator::simulation::SimulationTick;

/// RED TEST: Verify rabbits spawn with warren formation config
#[test]
fn test_rabbit_spawns_with_warren_config() {
    let mut app = App::new();

    // Spawn a rabbit using the standard spawn function
    let rabbit = life_simulator::entities::entity_types::spawn_rabbit(
        &mut app.world_mut().commands(),
        "Test Rabbit".to_string(),
        IVec2::new(0, 0),
    );

    app.world_mut().flush();

    // Verify rabbit has GroupFormationConfig
    let config = app.world().get::<GroupFormationConfig>(rabbit);
    assert!(config.is_some(), "Rabbit should have GroupFormationConfig");

    let config = config.unwrap();
    assert!(config.enabled, "Warren formation should be enabled");
    assert_eq!(config.group_type, GroupType::Warren, "Should be Warren type");
    assert_eq!(config.min_group_size, 4, "Min warren size should be 4");
    assert_eq!(config.max_group_size, 15, "Max warren size should be 15");
    assert_eq!(config.formation_radius, 30.0, "Formation radius should be 30");
    assert_eq!(config.cohesion_radius, 100.0, "Cohesion radius should be 100");
}

/// RED TEST: Verify 4+ rabbits within 30 tiles form a warren
#[test]
fn test_rabbits_form_warren() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SimulationTick>();
    app.add_systems(Update, life_simulator::ai::generic_group_formation_system);

    let config = GroupFormationConfig::rabbit_warren();

    // Spawn 5 rabbits close together (within 30 tiles)
    let rabbits: Vec<Entity> = (0..5)
        .map(|i| {
            app.world_mut().spawn((
                Rabbit,
                TilePosition { tile: IVec2::new(i * 5, 0) }, // 5 tiles apart
                config.clone(),
            )).id()
        })
        .collect();

    app.world_mut().flush();

    // Run formation system
    app.update();

    app.world_mut().flush();

    // Verify warren was formed
    let leaders: Vec<Entity> = {
        let world = app.world_mut();
        world
            .query::<(Entity, &PackLeader)>()
            .iter(world)
            .map(|(e, _)| e)
            .collect()
    };

    assert!(!leaders.is_empty(), "Warren should have been formed");

    // Verify warren has correct type
    let leader_comp = app.world().get::<PackLeader>(leaders[0]).unwrap();
    assert_eq!(leader_comp.group_type, GroupType::Warren);
    assert!(leader_comp.members.len() >= 3, "Warren should have at least 3 members besides leader");
}

/// RED TEST: Verify warren cohesion maintains groups within 100 tiles
#[test]
fn test_warren_cohesion_maintained() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SimulationTick>();
    app.add_systems(Update, (
        life_simulator::ai::generic_group_cohesion_system,
        life_simulator::ai::process_member_removals,
    ).chain());

    let config = GroupFormationConfig::rabbit_warren();

    // Create a warren with leader and enough members (min_group_size = 4)
    let leader = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(0, 0) },
        config.clone(),
        PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    // Add 4 members close to leader (within 100 tiles) to meet min_group_size
    let member1 = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(50, 0) }, // 50 tiles away
        config.clone(),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    let member2 = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(0, 80) }, // 80 tiles away
        config.clone(),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    let member3 = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(30, 30) }, // ~42 tiles away
        config.clone(),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    let member4 = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(-20, 20) }, // ~28 tiles away
        config.clone(),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    // Update leader's member list
    if let Some(mut leader_comp) = app.world_mut().get_mut::<PackLeader>(leader) {
        leader_comp.members = vec![member1, member2, member3, member4];
    }

    app.world_mut().flush();

    // Run cohesion system
    {
        let mut tick = app.world_mut().resource_mut::<SimulationTick>();
        tick.0 = 600; // Divisible by check_interval_ticks (200)
    }

    app.update();

    app.world_mut().flush();

    // Verify warren is still intact (members within cohesion radius)
    let leader_comp = app.world().get::<PackLeader>(leader);
    assert!(leader_comp.is_some(), "Leader should still exist with members within cohesion radius");
    assert_eq!(leader_comp.unwrap().members.len(), 4, "All 4 members should still be in warren");
}

/// RED TEST: Verify warren dissolves when members drift beyond 100 tiles
#[test]
fn test_warren_dissolution_on_distance() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SimulationTick>();
    app.add_systems(Update, (
        life_simulator::ai::generic_group_cohesion_system,
        life_simulator::ai::process_member_removals,
    ).chain());

    let config = GroupFormationConfig::rabbit_warren();

    // Create a warren with leader and a distant member
    let leader = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(0, 0) },
        config.clone(),
        PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    // Add member far from leader (beyond 100 tiles)
    let member_far = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(150, 0) }, // 150 tiles away (beyond cohesion)
        config.clone(),
        PackMember {
            leader,
            joined_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    // Update leader's member list
    if let Some(mut leader_comp) = app.world_mut().get_mut::<PackLeader>(leader) {
        leader_comp.members = vec![member_far];
    }

    app.world_mut().flush();

    // Run cohesion system at the right tick interval
    {
        let mut tick = app.world_mut().resource_mut::<SimulationTick>();
        tick.0 = 600; // Divisible by check_interval_ticks for warren (200)
    }

    // Run update to trigger cohesion check
    app.update();

    app.world_mut().flush();

    // Verify warren dissolved (leader no longer has PackLeader component or members removed)
    let leader_comp = app.world().get::<PackLeader>(leader);
    // Cohesion system should either:
    // 1. Remove PackLeader component entirely (warren dissolved)
    // 2. Remove the distant member from the members list
    // The test passes if either condition is true
    let warren_dissolved = leader_comp.is_none() ||
                          leader_comp.unwrap().members.is_empty() ||
                          !leader_comp.unwrap().members.contains(&member_far);

    assert!(warren_dissolved,
            "Warren should dissolve or remove distant members. Leader exists: {}, Members: {:?}",
            leader_comp.is_some(),
            leader_comp.map(|c| c.members.clone()).unwrap_or_default());
}

/// RED TEST: Verify warren defense bonus increases flee utility
#[test]
fn test_warren_defense_bonus_applied() {
    let mut app = App::new();

    // Create warren leader with members
    let member1 = Entity::from_raw(10);
    let member2 = Entity::from_raw(11);

    let leader = app.world_mut().spawn((
        Rabbit,
        TilePosition { tile: IVec2::new(0, 0) },
        PackLeader {
            members: vec![member1, member2],
            formed_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    // Create actions with Graze (used for movement/fleeing)
    let mut actions = vec![
        UtilityScore {
            action_type: ActionType::Graze {
                target_tile: IVec2::new(10, 0),
            },
            utility: 0.5,
            priority: 100,
        },
        UtilityScore {
            action_type: ActionType::Wander {
                target_tile: IVec2::new(5, 5),
            },
            utility: 0.3,
            priority: 50,
        },
    ];

    let original_graze_utility = actions[0].utility;
    let original_wander_utility = actions[1].utility;

    // Apply warren defense bonuses
    life_simulator::ai::apply_group_behavior_bonuses(leader, &mut actions, app.world());

    // Verify bonuses were applied (warren defense adds +0.20 to movement actions)
    assert!(
        actions[0].utility > original_graze_utility,
        "Graze utility should increase with warren defense bonus"
    );
    assert!(
        actions[1].utility > original_wander_utility,
        "Wander utility should increase with warren defense bonus"
    );

    // Verify bonus amount is approximately +0.20
    let graze_bonus = actions[0].utility - original_graze_utility;
    assert!(
        (graze_bonus - 0.20).abs() < 0.01,
        "Warren defense bonus should be approximately +0.20, got {}",
        graze_bonus
    );
}

/// RED TEST: Verify warren defense bonus doesn't affect non-movement actions
#[test]
fn test_warren_defense_bonus_specific_to_movement() {
    let mut app = App::new();

    let leader = app.world_mut().spawn((
        Rabbit,
        PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Warren,
        },
    )).id();

    // Create actions including non-movement types
    let mut actions = vec![
        UtilityScore {
            action_type: ActionType::DrinkWater {
                target_tile: IVec2::new(10, 0),
            },
            utility: 0.7,
            priority: 200,
        },
        UtilityScore {
            action_type: ActionType::Rest {
                duration_ticks: 10,
            },
            utility: 0.6,
            priority: 150,
        },
    ];

    let original_drink = actions[0].utility;
    let original_eat = actions[1].utility;

    // Apply warren defense bonuses
    life_simulator::ai::apply_group_behavior_bonuses(leader, &mut actions, app.world());

    // Verify non-movement actions are unchanged
    assert_eq!(
        actions[0].utility, original_drink,
        "Drink action should not be affected by warren defense bonus"
    );
    assert_eq!(
        actions[1].utility, original_eat,
        "Rest action should not be affected by warren defense bonus"
    );
}

/// RED TEST: Verify multiple warrens can form from large rabbit populations
#[test]
fn test_multiple_warrens_formation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SimulationTick>();
    app.add_systems(Update, life_simulator::ai::generic_group_formation_system);

    let config = GroupFormationConfig::rabbit_warren();

    // Spawn 20 rabbits in two clusters (10 each, separated by 200 tiles)
    let mut rabbits = Vec::new();

    // Cluster 1: around (0, 0)
    for i in 0..10 {
        let rabbit = app.world_mut().spawn((
            Rabbit,
            TilePosition { tile: IVec2::new(i * 3, 0) }, // 3 tiles apart
            config.clone(),
        )).id();
        rabbits.push(rabbit);
    }

    // Cluster 2: around (200, 0)
    for i in 0..10 {
        let rabbit = app.world_mut().spawn((
            Rabbit,
            TilePosition { tile: IVec2::new(200 + i * 3, 0) },
            config.clone(),
        )).id();
        rabbits.push(rabbit);
    }

    app.world_mut().flush();

    // Run formation system
    app.update();

    app.world_mut().flush();

    // Count number of warrens formed
    let warren_count = {
        let world = app.world_mut();
        world
            .query::<&PackLeader>()
            .iter(world)
            .filter(|leader| leader.group_type == GroupType::Warren)
            .count()
    };

    // Should form at least 2 warrens (one per cluster)
    assert!(
        warren_count >= 2,
        "Should form at least 2 warrens from separated clusters, found {}",
        warren_count
    );
}
