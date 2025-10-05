//! Integration tests for predator fear system
//!
//! Tests that predator proximity detection and fear-based behavior modification
//! work correctly as part of Phase 3 implementation.

use life_simulator::entities::{FearState, FearPlugin};
use life_simulator::entities::entity_types::{spawn_rabbit, spawn_wolf};
use life_simulator::entities::{TilePosition, MovementSpeed};
use bevy::prelude::*;
use life_simulator::vegetation::constants::predator_effects::*;

#[test]
fn test_fear_state_creation() {
    let fear_state = FearState::new();

    assert_eq!(fear_state.fear_level, 0.0);
    assert_eq!(fear_state.nearby_predators, 0);
    assert_eq!(fear_state.ticks_since_danger, 0);
    assert_eq!(fear_state.peak_fear, 0.0);
    assert!(!fear_state.is_fearful());
}

#[test]
fn test_fear_stimulus_application() {
    let mut fear_state = FearState::new();

    // Apply fear stimulus from 1 predator
    fear_state.apply_fear_stimulus(1);
    assert_eq!(fear_state.nearby_predators, 1);
    assert_eq!(fear_state.fear_level, 0.4); // 1 * 0.4, capped at 1.0
    assert_eq!(fear_state.peak_fear, 0.4);
    assert!(fear_state.is_fearful());

    // Apply fear stimulus from 2 predators
    fear_state.apply_fear_stimulus(2);
    assert_eq!(fear_state.nearby_predators, 2);
    assert_eq!(fear_state.fear_level, 0.8); // 2 * 0.4, capped at 1.0
    assert_eq!(fear_state.peak_fear, 0.8);

    // Apply fear stimulus from many predators
    fear_state.apply_fear_stimulus(5);
    assert_eq!(fear_state.nearby_predators, 5);
    assert_eq!(fear_state.fear_level, 1.0); // Capped at 1.0
    assert_eq!(fear_state.peak_fear, 1.0);
}

#[test]
fn test_fear_decay() {
    let mut fear_state = FearState::new();

    // Apply high fear
    fear_state.apply_fear_stimulus(3);
    assert!(fear_state.is_fearful());

    // Decay over time
    for _ in 0..50 {
        fear_state.decay_fear();
    }

    // Should be significantly decayed
    assert!(fear_state.fear_level < 0.2);
    assert!(!fear_state.is_fearful());
}

#[test]
fn test_utility_modifier() {
    let mut fear_state = FearState::new();

    // No fear = no modifier
    assert_eq!(fear_state.get_utility_modifier(), 1.0);

    // High fear = reduced utility
    fear_state.apply_fear_stimulus(2); // fear_level = 0.8
    let modifier = fear_state.get_utility_modifier();
    assert!(modifier < 1.0);
    assert!(modifier > 0.5); // Should be 1.0 - (0.8 * 0.5) = 0.6
}

#[test]
fn test_speed_modifier() {
    let mut fear_state = FearState::new();

    // No fear = no speed boost
    assert_eq!(fear_state.get_speed_modifier(), 1.0);

    // High fear = speed boost
    fear_state.apply_fear_stimulus(2); // fear_level = 0.8
    let modifier = fear_state.get_speed_modifier();
    assert!(modifier > 1.0);
    assert!(modifier <= FEAR_SPEED_BOOST); // Should be <= 1.5
}

#[test]
fn test_feeding_reduction() {
    let mut fear_state = FearState::new();

    // No fear = no feeding reduction
    assert_eq!(fear_state.get_feeding_reduction(), 0.0);

    // High fear = feeding reduction
    fear_state.apply_fear_stimulus(2); // fear_level = 0.8
    let reduction = fear_state.get_feeding_reduction();
    assert!(reduction > 0.0);
    assert!(reduction <= FEAR_FEEDING_REDUCTION); // Should be <= 0.3
}

#[test]
fn test_biomass_tolerance() {
    let mut fear_state = FearState::new();

    // No fear = no tolerance increase
    assert_eq!(fear_state.get_biomass_tolerance(), 0.0);

    // High fear = biomass tolerance increase
    fear_state.apply_fear_stimulus(2); // fear_level = 0.8
    let tolerance = fear_state.get_biomass_tolerance();
    assert!(tolerance > 0.0);
    assert!(tolerance <= FEAR_BIOMASS_TOLERANCE); // Should be <= 0.2
}

#[test]
fn test_predator_proximity_detection() {
    let mut app = App::new();
    app.add_plugins(FearPlugin);

    // Create test world
    let world = &mut app.world_mut();

    // Spawn wolf (predator) at (0, 0)
    let wolf = spawn_wolf(world, "TestWolf", IVec2::new(0, 0));

    // Spawn rabbit (prey) within fear radius
    let rabbit_near = spawn_rabbit(world, "NearRabbit", IVec2::new(10, 10));

    // Spawn rabbit (prey) outside fear radius
    let rabbit_far = spawn_rabbit(world, "FarRabbit", IVec2::new(50, 50));

    // Run one update to process fear detection
    app.update();

    // Check fear states
    let near_fear = world.entity(rabbit_near).get::<FearState>().unwrap();
    let far_fear = world.entity(rabbit_far).get::<FearState>().unwrap();

    // Near rabbit should detect predator
    assert!(near_fear.nearby_predators > 0);
    assert!(near_fear.is_fearful());

    // Far rabbit should not detect predator
    assert_eq!(far_fear.nearby_predators, 0);
    assert!(!far_fear.is_fearful());

    // Verify distance calculations
    let near_distance = IVec2::new(10, 10).as_vec2().distance(IVec2::new(0, 0).as_vec2());
    let far_distance = IVec2::new(50, 50).as_vec2().distance(IVec2::new(0, 0).as_vec2());

    assert!(near_distance <= FEAR_RADIUS as f32);
    assert!(far_distance > FEAR_RADIUS as f32);
}

#[test]
fn test_fear_constants() {
    // Verify fear system constants are reasonable
    assert!(FEAR_FEEDING_REDUCTION > 0.0 && FEAR_FEEDING_REDUCTION <= 1.0);
    assert!(FEAR_RADIUS > 0);
    assert!(FEAR_BIOMASS_TOLERANCE > 0.0 && FEAR_BIOMASS_TOLERANCE <= 1.0);
    assert!(FEAR_SPEED_BOOST >= 1.0);

    // Test specific values from the plan
    assert_eq!(FEAR_FEEDING_REDUCTION, 0.3); // 30% shorter feeding
    assert_eq!(FEAR_RADIUS, 20); // 20 tiles radius
    assert_eq!(FEAR_BIOMASS_TOLERANCE, 0.2); // 20% lower threshold
    assert_eq!(FEAR_SPEED_BOOST, 1.5); // 1.5x normal speed
}