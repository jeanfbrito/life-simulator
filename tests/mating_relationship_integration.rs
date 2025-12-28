//! Integration tests for mating relationship system
//!
//! Tests the lifecycle of mating relationships from establishment through cleanup.

use bevy::prelude::*;
use life_simulator::entities::{
    ActiveMate, MatingTarget, TilePosition, Sex, Age, ReproductionCooldown,
    Energy, Health, Hunger, Thirst, WellFedStreak, Creature,
};
use life_simulator::ai::{
    establish_mating_relationship, clear_mating_relationship,
    has_mating_relationship, is_being_courted, get_mating_partner,
};

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

fn create_test_entity(app: &mut App, tile: IVec2) -> Entity {
    let entity = app.world_mut().spawn((
        Creature {
            name: "TestEntity".to_string(),
            species: "TestSpecies".to_string(),
        },
        TilePosition { tile },
        Sex::Male,
        Age {
            ticks_alive: 2000,
            mature_at_ticks: 1000,
        },
        ReproductionCooldown { remaining_ticks: 0 },
        Energy(life_simulator::entities::Stat::new(50.0, 0.0, 100.0, 0.0)),
        Health(life_simulator::entities::Stat::new(80.0, 0.0, 100.0, 0.0)),
        Hunger(life_simulator::entities::Stat::new(30.0, 0.0, 100.0, 0.0)),
        Thirst(life_simulator::entities::Stat::new(25.0, 0.0, 100.0, 0.0)),
        WellFedStreak { ticks: 100 },
    )).id();
    entity
}

#[test]
fn test_establish_mating_relationship() {
    let mut app = setup_app();
    let male = create_test_entity(&mut app, IVec2::new(0, 0));
    let female = create_test_entity(&mut app, IVec2::new(1, 1));
    let meeting_tile = IVec2::new(0, 1);
    let current_tick = 100u64;

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male, female, meeting_tile, current_tick, &mut commands);
    }
    app.update();

    // Check that male has ActiveMate component
    let male_component = app.world().get::<ActiveMate>(male);
    assert!(male_component.is_some(), "Male should have ActiveMate component");
    assert_eq!(male_component.unwrap().partner, female);
    assert_eq!(male_component.unwrap().meeting_tile, meeting_tile);
    assert_eq!(male_component.unwrap().started_tick, current_tick);

    // Check that female has MatingTarget component
    let female_component = app.world().get::<MatingTarget>(female);
    assert!(female_component.is_some(), "Female should have MatingTarget component");
    assert_eq!(female_component.unwrap().suitor, male);
    assert_eq!(female_component.unwrap().meeting_tile, meeting_tile);
    assert_eq!(female_component.unwrap().started_tick, current_tick);
}

#[test]
fn test_has_mating_relationship() {
    let mut app = setup_app();
    let male = create_test_entity(&mut app, IVec2::new(0, 0));
    let female = create_test_entity(&mut app, IVec2::new(1, 1));
    let meeting_tile = IVec2::new(0, 1);

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male, female, meeting_tile, 100, &mut commands);
    }
    app.update();

    let world = app.world();
    assert!(has_mating_relationship(male, world), "Male should have mating relationship");
    assert!(!has_mating_relationship(female, world), "Female should not have ActiveMate");
}

#[test]
fn test_is_being_courted() {
    let mut app = setup_app();
    let male = create_test_entity(&mut app, IVec2::new(0, 0));
    let female = create_test_entity(&mut app, IVec2::new(1, 1));
    let meeting_tile = IVec2::new(0, 1);

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male, female, meeting_tile, 100, &mut commands);
    }
    app.update();

    let world = app.world();
    assert!(is_being_courted(female, world), "Female should be being courted");
    assert!(!is_being_courted(male, world), "Male should not have MatingTarget");
}

#[test]
fn test_get_mating_partner() {
    let mut app = setup_app();
    let male = create_test_entity(&mut app, IVec2::new(0, 0));
    let female = create_test_entity(&mut app, IVec2::new(1, 1));
    let meeting_tile = IVec2::new(0, 1);

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male, female, meeting_tile, 100, &mut commands);
    }
    app.update();

    let world = app.world();
    let partner = get_mating_partner(male, world);
    assert_eq!(partner, Some(female), "Male's partner should be the female");

    let female_partner = get_mating_partner(female, world);
    assert_eq!(female_partner, None, "Female has no ActiveMate component");
}

#[test]
fn test_clear_mating_relationship() {
    let mut app = setup_app();
    let male = create_test_entity(&mut app, IVec2::new(0, 0));
    let female = create_test_entity(&mut app, IVec2::new(1, 1));
    let meeting_tile = IVec2::new(0, 1);

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male, female, meeting_tile, 100, &mut commands);
    }
    app.update();

    // Verify relationship exists
    assert!(has_mating_relationship(male, app.world()), "Relationship should exist");
    assert!(is_being_courted(female, app.world()), "Female should be courted");

    // Clear relationship
    {
        let mut commands = app.world_mut().commands();
        clear_mating_relationship(male, female, &mut commands);
    }
    app.update();

    // Verify relationship is gone
    assert!(!has_mating_relationship(male, app.world()), "ActiveMate should be removed");
    assert!(!is_being_courted(female, app.world()), "MatingTarget should be removed");
}

#[test]
fn test_mating_duration_calculation() {
    let mut app = setup_app();
    let male = create_test_entity(&mut app, IVec2::new(0, 0));
    let female = create_test_entity(&mut app, IVec2::new(1, 1));
    let meeting_tile = IVec2::new(0, 1);
    let start_tick = 100u64;

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male, female, meeting_tile, start_tick, &mut commands);
    }
    app.update();

    let world = app.world();
    let male_mate = world.get::<ActiveMate>(male).unwrap();
    let current_tick = 150u64;
    let duration = current_tick - male_mate.started_tick;

    assert_eq!(duration, 50, "Mating should have lasted 50 ticks");
}

#[test]
fn test_multiple_mating_pairs() {
    let mut app = setup_app();
    let male1 = create_test_entity(&mut app, IVec2::new(0, 0));
    let female1 = create_test_entity(&mut app, IVec2::new(1, 1));
    let male2 = create_test_entity(&mut app, IVec2::new(10, 10));
    let female2 = create_test_entity(&mut app, IVec2::new(11, 11));

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male1, female1, IVec2::new(0, 1), 100, &mut commands);
        establish_mating_relationship(male2, female2, IVec2::new(10, 11), 100, &mut commands);
    }
    app.update();

    let world = app.world();

    // Verify first pair
    assert_eq!(get_mating_partner(male1, world), Some(female1));
    assert!(is_being_courted(female1, world));

    // Verify second pair
    assert_eq!(get_mating_partner(male2, world), Some(female2));
    assert!(is_being_courted(female2, world));

    // Verify pairs are separate
    assert_ne!(get_mating_partner(male1, world), get_mating_partner(male2, world));
}

#[test]
fn test_different_meeting_tiles() {
    let mut app = setup_app();
    let male1 = create_test_entity(&mut app, IVec2::new(0, 0));
    let female1 = create_test_entity(&mut app, IVec2::new(1, 1));
    let male2 = create_test_entity(&mut app, IVec2::new(10, 10));
    let female2 = create_test_entity(&mut app, IVec2::new(11, 11));

    let tile1 = IVec2::new(5, 5);
    let tile2 = IVec2::new(15, 15);

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male1, female1, tile1, 100, &mut commands);
        establish_mating_relationship(male2, female2, tile2, 100, &mut commands);
    }
    app.update();

    let world = app.world();
    let mate1 = world.get::<ActiveMate>(male1).unwrap();
    let mate2 = world.get::<ActiveMate>(male2).unwrap();

    assert_eq!(mate1.meeting_tile, tile1);
    assert_eq!(mate2.meeting_tile, tile2);
    assert_ne!(mate1.meeting_tile, mate2.meeting_tile);
}

#[test]
fn test_bidirectional_consistency() {
    let mut app = setup_app();
    let male = create_test_entity(&mut app, IVec2::new(0, 0));
    let female = create_test_entity(&mut app, IVec2::new(1, 1));
    let meeting_tile = IVec2::new(0, 1);
    let tick = 100u64;

    {
        let mut commands = app.world_mut().commands();
        establish_mating_relationship(male, female, meeting_tile, tick, &mut commands);
    }
    app.update();

    let world = app.world();
    let male_mate = world.get::<ActiveMate>(male).unwrap();
    let female_target = world.get::<MatingTarget>(female).unwrap();

    // Both should reference each other
    assert_eq!(male_mate.partner, female);
    assert_eq!(female_target.suitor, male);

    // Both should have same meeting location
    assert_eq!(male_mate.meeting_tile, female_target.meeting_tile);

    // Both should have same start tick
    assert_eq!(male_mate.started_tick, female_target.started_tick);
}
