//! Integration test for herbivore consumption with vegetation system
//!
//! This test verifies that the GrazeAction correctly consumes biomass
//! from the vegetation grid and reduces rabbit hunger.

use bevy::prelude::*;
use life_simulator::{
    ai::{Action, queue::ActionQueue},
    entities::{Rabbit, stats::{Hunger, Stat}, BehaviorConfig, TilePosition, MovementSpeed},
    simulation::SimulationPlugin,
    vegetation::{VegetationGrid, VegetationPlugin},
};

#[test]
fn test_herbivore_consumption_integration() {
    let mut app = App::new();

    // Add essential plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin);
    app.add_plugins(VegetationPlugin);

    // Add world setup
    app.init_resource::<VegetationGrid>();
    app.init_resource::<ActionQueue>();

    // Setup test world and spawn rabbit
    app.add_systems(Startup, setup_test_world);
    app.add_systems(Startup, spawn_test_rabbit);

    // Run one update to setup everything
    app.update();

    // Get initial state
    let initial_vegetation = {
        let vegetation_grid = app.world().resource::<VegetationGrid>();
        let test_tile = IVec2::new(5, 5);
        vegetation_grid.get(test_tile).map(|v| v.biomass).unwrap_or(0.0)
    };

    println!("Initial vegetation biomass at (5,5): {}", initial_vegetation);

    // Get rabbit entity
    let rabbit_entity = {
        let mut rabbit_query = app.world_mut().query::<(Entity, &Rabbit)>();
        rabbit_query.iter(app.world()).next().unwrap().0
    };

    println!("Spawned rabbit entity: {:?}", rabbit_entity);

    // Check rabbit's initial hunger
    let initial_hunger = {
        let mut hunger_query = app.world_mut().query::<(&Hunger, &Rabbit)>();
        hunger_query.iter(app.world()).next().unwrap().0.current
    };

    println!("Initial rabbit hunger: {}", initial_hunger);

    // Position rabbit at the vegetation tile
    {
        let mut entity_mut = app.world_mut().get_entity_mut(rabbit_entity).unwrap();
        entity_mut.insert(TilePosition::from_tile(IVec2::new(5, 5)));
    }

    // Manually create and execute a GrazeAction
    let mut action = life_simulator::ai::action::GrazeAction::new(IVec2::new(5, 5));

    // Execute the action
    let result = action.execute(&mut app.world_mut(), rabbit_entity, 0);

    println!("GrazeAction result: {:?}", result);

    // Check the vegetation after grazing
    let final_vegetation = {
        let vegetation_grid = app.world().resource::<VegetationGrid>();
        let test_tile = IVec2::new(5, 5);
        vegetation_grid.get(test_tile).map(|v| v.biomass).unwrap_or(0.0)
    };

    println!("Final vegetation biomass at (5,5): {}", final_vegetation);

    // Check rabbit's final hunger
    let final_hunger = {
        let mut hunger_query = app.world_mut().query::<(&Hunger, &Rabbit)>();
        hunger_query.iter(app.world()).next().unwrap().0.current
    };

    println!("Final rabbit hunger: {}", final_hunger);

    // Verify consumption occurred
    assert!(initial_vegetation > final_vegetation, "Vegetation should have been consumed");
    assert!(final_hunger > initial_hunger, "Rabbit hunger should have been reduced");

    // Calculate expected consumption (30% rule)
    let expected_consumption = (initial_vegetation * 0.3).min(30.0); // MAX_MEAL_ABSOLUTE = 30
    let actual_consumption = initial_vegetation - final_vegetation;
    let hunger_reduction = final_hunger - initial_hunger;

    println!("Expected consumption: {}, Actual consumption: {}, Hunger reduction: {}",
             expected_consumption, actual_consumption, hunger_reduction);

    // Allow for small floating point differences
    assert!((actual_consumption - expected_consumption).abs() < 0.1,
              "Consumed amount should match 30% rule");
    assert!((hunger_reduction - actual_consumption).abs() < 0.1,
              "Hunger reduction should match consumed amount");
}

fn setup_test_world(mut vegetation_grid: ResMut<VegetationGrid>) {
    // Create a simple test world with grass at position (5,5)
    let test_tile = IVec2::new(5, 5);

    // Add vegetation to the test tile
    vegetation_grid.get_or_create(test_tile, 1.0); // Normal terrain multiplier

    // Set high initial biomass for testing
    {
        let vegetation = vegetation_grid.get_mut(test_tile).unwrap();
        vegetation.biomass = 80.0; // High biomass
    }

    println!("Setup test world with vegetation at (5,5)");
}

fn spawn_test_rabbit(mut commands: Commands) {
    // Spawn a rabbit with hunger
    let rabbit = commands.spawn((
        Rabbit,
        TilePosition::from_tile(IVec2::new(3, 5)),
        MovementSpeed::normal(),
        Hunger(Stat::new(0.0, 0.0, 100.0, 0.0)), // Start with no hunger reduction
        BehaviorConfig::default(),
    )).id();

    println!("Spawned test rabbit: {:?}", rabbit);
}