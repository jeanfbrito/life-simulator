//! Simple integration test for herbivore consumption

use bevy::prelude::*;
use life_simulator::{
    ai::Action,
    entities::{Rabbit, stats::{Hunger, Stat}, BehaviorConfig, TilePosition, MovementSpeed},
    vegetation::VegetationGrid,
};

#[test]
fn test_simple_grazing() {
    let mut world = World::new();

    // Create vegetation grid
    let mut vegetation_grid = VegetationGrid::new();

    // Create vegetation at (5,5)
    let tile = IVec2::new(5, 5);
    vegetation_grid.get_or_create(tile, 1.0);

    // Set high biomass
    {
        let vegetation = vegetation_grid.get_mut(tile).unwrap();
        vegetation.biomass = 80.0;
    }

    // Insert resources
    world.insert_resource(vegetation_grid);

    // Spawn rabbit
    let rabbit_entity = world.spawn((
        Rabbit,
        TilePosition::from_tile(IVec2::new(5, 5)), // Position at vegetation tile
        MovementSpeed::normal(),
        Hunger(Stat::new(0.0, 0.0, 100.0, 0.0)),
        BehaviorConfig::default(),
    )).id();

    println!("Setup complete");

    // Check initial state
    let initial_vegetation = {
        let vegetation_grid = world.resource::<VegetationGrid>();
        vegetation_grid.get(IVec2::new(5, 5)).map(|v| v.biomass).unwrap_or(0.0)
    };

    println!("Initial vegetation biomass: {}", initial_vegetation);

    // Execute grazing action
    let mut action = life_simulator::ai::action::GrazeAction::new(IVec2::new(5, 5));
    let result = action.execute(&mut world, rabbit_entity, 0);

    println!("GrazeAction result: {:?}", result);

    // Check final state
    let final_vegetation = {
        let vegetation_grid = world.resource::<VegetationGrid>();
        vegetation_grid.get(IVec2::new(5, 5)).map(|v| v.biomass).unwrap_or(0.0)
    };

    println!("Final vegetation biomass: {}", final_vegetation);

    // Verify consumption occurred
    assert!(initial_vegetation > final_vegetation, "Vegetation should have been consumed");

    let consumed = initial_vegetation - final_vegetation;
    let expected = (initial_vegetation * 0.3).min(30.0);

    println!("Consumed: {}, Expected: {}", consumed, expected);
    assert!((consumed - expected).abs() < 0.1, "Should consume 30% of biomass");

    // Verify rabbit hunger was reduced
    let final_hunger = {
        let mut hunger_query = world.query::<&Hunger>();
        let hunger_stat = &hunger_query.iter(&world).next().unwrap().0;
        hunger_stat.current
    };

    println!("Final rabbit hunger: {}", final_hunger);
    assert!(final_hunger > 0.0, "Rabbit hunger should have been reduced");
    assert!((final_hunger - consumed).abs() < 0.1, "Hunger reduction should match consumption");
}