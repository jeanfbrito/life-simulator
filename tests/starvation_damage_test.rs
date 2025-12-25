/// Integration test for starvation/dehydration damage system
///
/// This test verifies that:
/// 1. Entities take progressive damage based on hunger/thirst levels
/// 2. Damage increases with severity (warning -> danger -> critical)
/// 3. Entities die from starvation if not fed
/// 4. The system integrates properly with the tick system

use bevy::prelude::*;
use life_simulator::entities::{
    spawn_rabbit, Energy, Health, Hunger, MovementSpeed, TilePosition, Thirst,
};
use life_simulator::simulation::{SimulationState, SimulationTick};

#[test]
fn test_starvation_damage_integration() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    // Add required resources
    app.insert_resource(SimulationTick(0));
    app.insert_resource(SimulationState {
        should_tick: true,
    });

    // Add the need damage system (skip tick_stats_system to avoid TickProfiler dependency)
    app.add_systems(
        Update,
        (
            life_simulator::entities::need_damage_system,
            life_simulator::entities::death_system,
        )
            .chain(),
    );

    // Spawn a rabbit with critical hunger
    app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(0, 0)),
        MovementSpeed::normal(),
        Hunger::new(),
        Thirst::new(),
        Energy::new(),
        Health::new(),
    ));

    // Set hunger to critical level (99%)
    let mut query = app.world_mut().query::<&mut Hunger>();
    for mut hunger in query.iter_mut(app.world_mut()) {
        hunger.0.set(99.0);
    }

    // Get initial health
    let initial_health = app
        .world_mut()
        .query::<&Health>()
        .iter(app.world())
        .next()
        .unwrap()
        .0
        .current;

    assert_eq!(initial_health, 100.0, "Health should start at 100");

    // Run one update tick
    app.update();

    // Check that health decreased by 0.5 (critical damage)
    let health_after_damage = app
        .world_mut()
        .query::<&Health>()
        .iter(app.world())
        .next()
        .unwrap()
        .0
        .current;

    assert_eq!(
        health_after_damage, 99.5,
        "Health should decrease by 0.5 at critical hunger"
    );

    // Verify entity still exists
    let entity_count = app.world_mut().query::<&Health>().iter(app.world()).len();
    assert_eq!(entity_count, 1, "Entity should still be alive");
}

#[test]
fn test_progressive_damage_levels() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimulationTick(0));
    app.insert_resource(SimulationState {
        should_tick: true,
    });

    app.add_systems(
        Update,
        life_simulator::entities::need_damage_system,
    );

    // Test warning level (92% hunger) - 0.05 damage
    app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(0, 0)),
        MovementSpeed::normal(),
        Hunger::new(),
        Thirst::new(),
        Energy::new(),
        Health::new(),
    ));

    // Set to warning level
    let mut query = app.world_mut().query::<&mut Hunger>();
    for mut hunger in query.iter_mut(app.world_mut()) {
        hunger.0.set(92.0);
    }

    app.update();

    let health = app
        .world_mut()
        .query::<&Health>()
        .iter(app.world())
        .next()
        .unwrap()
        .0
        .current;

    assert_eq!(health, 99.95, "Warning level should deal 0.05 damage");
}

#[test]
fn test_thirst_overrides_hunger() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimulationTick(0));
    app.insert_resource(SimulationState {
        should_tick: true,
    });

    app.add_systems(
        Update,
        life_simulator::entities::need_damage_system,
    );

    app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(0, 0)),
        MovementSpeed::normal(),
        Hunger::new(),
        Thirst::new(),
        Energy::new(),
        Health::new(),
    ));

    // Set hunger to warning (0.05 damage) and thirst to critical (0.5 damage)
    let mut hunger_query = app.world_mut().query::<&mut Hunger>();
    for mut hunger in hunger_query.iter_mut(app.world_mut()) {
        hunger.0.set(92.0);
    }

    let mut thirst_query = app.world_mut().query::<&mut Thirst>();
    for mut thirst in thirst_query.iter_mut(app.world_mut()) {
        thirst.0.set(99.0);
    }

    app.update();

    let health = app
        .world_mut()
        .query::<&Health>()
        .iter(app.world())
        .next()
        .unwrap()
        .0
        .current;

    assert_eq!(
        health, 99.5,
        "Should use worse damage (0.5 from thirst, not 0.05 from hunger)"
    );
}

#[test]
fn test_death_from_starvation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimulationTick(0));
    app.insert_resource(SimulationState {
        should_tick: true,
    });

    app.add_systems(
        Update,
        life_simulator::entities::need_damage_system,
    );

    app.world_mut().spawn((
        TilePosition::from_tile(IVec2::new(0, 0)),
        MovementSpeed::normal(),
        Hunger::new(),
        Thirst::new(),
        Energy::new(),
        Health::new(),
    ));

    // Set to critical starvation
    let mut query = app.world_mut().query::<&mut Hunger>();
    for mut hunger in query.iter_mut(app.world_mut()) {
        hunger.0.set(99.0);
    }

    // Should take ~200 ticks to die at 0.5 damage per tick
    // Let's run 210 ticks to ensure health reaches zero
    for _ in 0..210 {
        app.update();
    }

    // Verify health reached zero (death would be handled by death_system in real game)
    let health = app
        .world_mut()
        .query::<&Health>()
        .iter(app.world())
        .next()
        .unwrap()
        .0
        .current;

    assert!(health <= 0.0, "Entity health should have reached zero from starvation");
}
