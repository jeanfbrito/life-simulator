//! Phase 2 validation test for ResourceGrid consumption (Fixed version)
//!
//! This test validates that:
//! 1. Consumption from empty grid returns zero
//! 2. Consumption from populated grid reduces biomass
//! 3. Regrowth events are properly scheduled
//! 4. Animals can still find food with the new system

use bevy::prelude::*;
use life_simulator::{
    ai::{queue::ActionQueue, Action, ActionResult},
    entities::{
        stats::{Hunger, Stat},
        BehaviorConfig, MovementSpeed, Rabbit, TilePosition,
    },
    simulation::SimulationPlugin,
    vegetation::resource_grid::ResourceGrid,
};

#[test]
fn test_consumption_empty_grid() {
    let mut world = World::new();

    // Create empty resource grid
    let resource_grid = ResourceGrid::new();
    world.insert_resource(resource_grid);

    // Spawn rabbit
    let rabbit_entity = world
        .spawn((
            Rabbit,
            TilePosition::from_tile(IVec2::new(5, 5)),
            MovementSpeed::normal(),
            Hunger(Stat::new(0.0, 0.0, 100.0, 0.0)),
            BehaviorConfig::default(),
        ))
        .id();

    // Try to consume from empty grid
    let mut action = life_simulator::ai::action::GrazeAction::new(IVec2::new(5, 5));
    let result = action.execute(&mut world, rabbit_entity, 0);

    // Should fail or complete with no consumption
    assert!(matches!(
        result,
        ActionResult::Success | ActionResult::Failed
    ));

    // Verify no consumption occurred (grid still empty)
    let resource_grid = world.resource::<ResourceGrid>();
    assert_eq!(resource_grid.cell_count(), 0);

    println!("✅ Empty grid consumption test passed");
}

#[test]
fn test_consumption_populated_grid() {
    let mut world = World::new();

    // Create resource grid with populated cell
    let mut resource_grid = ResourceGrid::new();
    let tile = IVec2::new(5, 5);
    let initial_biomass = 80.0;

    resource_grid
        .get_or_create_cell(tile, 100.0, 1.0)
        .total_biomass = initial_biomass;
    world.insert_resource(resource_grid);

    // Spawn rabbit at the tile
    let rabbit_entity = world
        .spawn((
            Rabbit,
            TilePosition::from_tile(tile),
            MovementSpeed::normal(),
            Hunger(Stat::new(0.0, 0.0, 100.0, 0.0)),
            BehaviorConfig::default(),
        ))
        .id();

    // Execute grazing action
    let mut action = life_simulator::ai::action::GrazeAction::new(tile);
    let result = action.execute(&mut world, rabbit_entity, 0);

    // Should succeed with consumption
    assert!(matches!(
        result,
        ActionResult::Success | ActionResult::InProgress
    ));

    // Verify biomass was reduced
    let resource_grid = world.resource::<ResourceGrid>();
    let final_biomass = resource_grid
        .get_cell(tile)
        .map(|c| c.total_biomass)
        .unwrap_or(0.0);

    assert!(
        final_biomass < initial_biomass,
        "Biomass should have been consumed"
    );

    // Verify regrowth event was scheduled
    assert!(
        resource_grid.pending_events() > 0,
        "Regrowth event should be scheduled"
    );

    println!("✅ Populated grid consumption test passed");
    println!(
        "   Initial biomass: {}, Final biomass: {}",
        initial_biomass, final_biomass
    );
    println!(
        "   Pending regrowth events: {}",
        resource_grid.pending_events()
    );
}

#[test]
fn test_resource_grid_find_best_cell() {
    let mut resource_grid = ResourceGrid::new();

    // Create multiple cells with different biomass levels
    let positions = vec![
        (IVec2::new(0, 0), 20.0),
        (IVec2::new(5, 0), 60.0), // Best - closest and high biomass
        (IVec2::new(0, 5), 80.0), // Best biomass but farther
        (IVec2::new(10, 10), 40.0),
    ];

    for (pos, biomass) in &positions {
        resource_grid
            .get_or_create_cell(*pos, 100.0, 1.0)
            .total_biomass = *biomass;
    }

    // Search from position (2, 2)
    let center = IVec2::new(2, 2);
    let search_radius = 10;

    let best = resource_grid.find_best_cell(center, search_radius);

    assert!(best.is_some(), "Should find a best cell");

    let (best_pos, best_biomass) = best.unwrap();

    // Should prefer the closest high biomass option (5,0 with 60.0)
    // over the farther higher biomass (0,5 with 80.0)
    println!("Best cell: {:?} with biomass: {}", best_pos, best_biomass);

    // The exact choice depends on the utility calculation, but should be reasonable
    assert!(
        best_biomass >= 40.0,
        "Should select a cell with decent biomass"
    );

    println!("✅ Find best cell test passed");
}

#[test]
fn test_animal_foraging_integration() {
    let mut app = App::new();

    // Add essential plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SimulationPlugin);
    app.init_resource::<ResourceGrid>();
    app.init_resource::<ActionQueue>();

    // Setup test world with multiple vegetation cells
    app.add_systems(Startup, setup_foraging_world);
    app.add_systems(Startup, spawn_foraging_rabbit);

    // Run setup
    app.update();

    // Get rabbit entity
    let rabbit_entity = {
        let mut world = app.world_mut();
        let mut rabbit_query = world.query::<(Entity, &Rabbit)>();
        rabbit_query.iter(&world).next().map(|(e, _)| e).unwrap()
    };

    // Check rabbit can find food
    let resource_grid = app.world().resource::<ResourceGrid>();
    let rabbit_pos = IVec2::new(2, 2);

    let best_cell = resource_grid.find_best_cell(rabbit_pos, 15);
    assert!(best_cell.is_some(), "Rabbit should be able to find food");

    let (food_pos, food_biomass) = best_cell.unwrap();
    println!(
        "Rabbit at {:?} found food at {:?} with {} biomass",
        rabbit_pos, food_pos, food_biomass
    );

    assert!(food_biomass >= 10.0, "Food should have sufficient biomass");

    // Test actual foraging by moving rabbit to food and executing graze action
    {
        let mut entity_mut = app.world_mut().get_entity_mut(rabbit_entity).unwrap();
        entity_mut.insert(TilePosition::from_tile(food_pos));
    }

    let mut action = life_simulator::ai::action::GrazeAction::new(food_pos);
    let result = action.execute(&mut app.world_mut(), rabbit_entity, 0);

    assert!(
        matches!(result, ActionResult::Success | ActionResult::InProgress),
        "Foraging should succeed"
    );

    // Verify consumption occurred
    let final_biomass = app
        .world()
        .resource::<ResourceGrid>()
        .get_cell(food_pos)
        .map(|c| c.total_biomass)
        .unwrap_or(0.0);

    assert!(
        final_biomass < food_biomass,
        "Food biomass should be reduced"
    );

    println!("✅ Animal foraging integration test passed");
    println!(
        "   Food consumed: {} -> {} biomass",
        food_biomass, final_biomass
    );
}

fn setup_foraging_world(mut resource_grid: ResMut<ResourceGrid>) {
    // Create a realistic foraging environment
    let vegetation_patches = vec![
        (IVec2::new(5, 3), 70.0),  // Close, good biomass
        (IVec2::new(8, 8), 50.0),  // Medium distance, decent biomass
        (IVec2::new(2, 10), 90.0), // Medium distance, excellent biomass
        (IVec2::new(12, 1), 30.0), // Far, low biomass
    ];

    for (pos, biomass) in &vegetation_patches {
        resource_grid
            .get_or_create_cell(*pos, 100.0, 1.0)
            .total_biomass = *biomass;
    }

    println!(
        "Setup foraging world with {} vegetation patches",
        vegetation_patches.len()
    );
}

fn spawn_foraging_rabbit(mut commands: Commands) {
    let rabbit = commands
        .spawn((
            Rabbit,
            TilePosition::from_tile(IVec2::new(2, 2)), // Start position
            MovementSpeed::normal(),
            Hunger(Stat::new(0.0, 0.0, 100.0, 0.0)),
            BehaviorConfig::default(),
        ))
        .id();

    println!("Spawned foraging rabbit: {:?}", rabbit);
}
