//! Phase 4: Required Components Test Suite (RED PHASE - TDD)
//!
//! This test suite verifies that components using #[require(...)] attributes
//! automatically include their required components when spawned.
//!
//! Test Strategy:
//! 1. Spawn entities with required components
//! 2. Assert that automatically required components are present
//! 3. Verify compilation ensures component dependencies are met

use bevy::prelude::*;
use life_simulator::entities::{
    Creature, TilePosition, MovementComponent,
    Health, Hunger, Thirst, Energy,
    FearState,
    Age, Sex, ReproductionCooldown, WellFedStreak, Pregnancy,
};

/// Test that MovementComponent requires TilePosition
#[test]
fn test_movement_component_requires_tile_position() {
    let mut world = World::new();

    // Spawn MovementComponent - should automatically include TilePosition
    let entity = world.spawn(MovementComponent::idle()).id();

    // Verify TilePosition is automatically present
    assert!(
        world.entity(entity).get::<TilePosition>().is_some(),
        "MovementComponent should require TilePosition"
    );
}

/// Test that Health requires Creature
#[test]
fn test_health_requires_creature() {
    let mut world = World::new();

    // Spawn Health alone - should automatically include Creature
    let entity = world.spawn(Health::new()).id();

    // Verify Creature is automatically present
    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "Health should require Creature"
    );
}

/// Test that Hunger requires Creature
#[test]
fn test_hunger_requires_creature() {
    let mut world = World::new();

    let entity = world.spawn(Hunger::new()).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "Hunger should require Creature"
    );
}

/// Test that Thirst requires Creature
#[test]
fn test_thirst_requires_creature() {
    let mut world = World::new();

    let entity = world.spawn(Thirst::new()).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "Thirst should require Creature"
    );
}

/// Test that Energy requires Creature
#[test]
fn test_energy_requires_creature() {
    let mut world = World::new();

    let entity = world.spawn(Energy::new()).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "Energy should require Creature"
    );
}

/// Test that FearState requires Creature and TilePosition
#[test]
fn test_fearstate_requires_creature_and_tile_position() {
    let mut world = World::new();

    let entity = world.spawn(FearState::new()).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "FearState should require Creature"
    );
    assert!(
        world.entity(entity).get::<TilePosition>().is_some(),
        "FearState should require TilePosition"
    );
}

/// Test that Age requires Creature
#[test]
fn test_age_requires_creature() {
    let mut world = World::new();

    let entity = world.spawn(Age {
        ticks_alive: 0,
        mature_at_ticks: 100,
    }).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "Age should require Creature"
    );
}

/// Test that Sex requires Creature
#[test]
fn test_sex_requires_creature() {
    let mut world = World::new();

    let entity = world.spawn(Sex::Male).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "Sex should require Creature"
    );
}

/// Test that ReproductionCooldown requires Creature
#[test]
fn test_reproduction_cooldown_requires_creature() {
    let mut world = World::new();

    let entity = world.spawn(ReproductionCooldown::default()).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "ReproductionCooldown should require Creature"
    );
}

/// Test that WellFedStreak requires Creature
#[test]
fn test_well_fed_streak_requires_creature() {
    let mut world = World::new();

    let entity = world.spawn(WellFedStreak::default()).id();

    assert!(
        world.entity(entity).get::<Creature>().is_some(),
        "WellFedStreak should require Creature"
    );
}

/// Test that Pregnancy requires Age and Sex
#[test]
fn test_pregnancy_requires_age_and_sex() {
    let mut world = World::new();

    let entity = world.spawn(Pregnancy {
        remaining_ticks: 100,
        litter_size: 2,
        father: None,
    }).id();

    assert!(
        world.entity(entity).get::<Age>().is_some(),
        "Pregnancy should require Age"
    );
    assert!(
        world.entity(entity).get::<Sex>().is_some(),
        "Pregnancy should require Sex"
    );
}

/// Test that explicit component insertion still works (doesn't duplicate)
#[test]
fn test_explicit_component_insertion_doesnt_duplicate() {
    let mut world = World::new();

    // Spawn with explicit components - should NOT auto-add them again
    let creature = Creature {
        name: "Test".to_string(),
        species: "Test".to_string(),
    };
    let tile_pos = TilePosition::new(5, 5);

    let entity = world.spawn((
        MovementComponent::idle(),
        creature,
        tile_pos,
    )).id();

    // Verify we only have ONE of each component
    assert_eq!(
        world.entity(entity).get::<TilePosition>().map(|tp| tp.tile),
        Some(IVec2::new(5, 5)),
        "Explicit TilePosition should be used, not auto-inserted"
    );
}
