//! Direct test of ResourceGrid consumption without action complexity
//!
//! This test directly validates the ResourceGrid::consume_at method
//! which is the core of Phase 2 integration.

use bevy::prelude::IVec2;
use life_simulator::vegetation::resource_grid::ResourceGrid;

#[test]
fn test_direct_resource_grid_consumption() {
    let mut resource_grid = ResourceGrid::new();
    let tile = IVec2::new(5, 5);
    let initial_biomass = 80.0;

    // Create a cell with biomass
    resource_grid
        .get_or_create_cell(tile, 100.0, 1.0)
        .total_biomass = initial_biomass;

    // Verify initial state
    assert_eq!(resource_grid.cell_count(), 1);
    let cell = resource_grid.get_cell(tile).unwrap();
    assert_eq!(cell.total_biomass, initial_biomass);

    // Test consumption with demand and max fraction
    let demand = 100.0; // High demand
    let max_fraction = 0.3; // 30% max consumption like the old system
    let consumed = resource_grid.consume_at(tile, demand, max_fraction);

    // Verify consumption occurred
    let expected_consumption = (initial_biomass * max_fraction).min(30.0); // MAX_MEAL_ABSOLUTE = 30
    assert!(consumed > 0.0, "Should have consumed some biomass");
    assert!(
        (consumed - expected_consumption).abs() < 0.1,
        "Consumed {:.1} but expected {:.1}",
        consumed,
        expected_consumption
    );

    // Verify biomass was reduced
    let final_biomass = resource_grid.get_cell(tile).unwrap().total_biomass;
    assert!(
        final_biomass < initial_biomass,
        "Biomass should have been reduced"
    );
    assert!(
        (final_biomass - (initial_biomass - consumed)).abs() < 0.1,
        "Final biomass {:.1} should be initial {:.1} minus consumed {:.1}",
        final_biomass,
        initial_biomass,
        consumed
    );

    // Verify regrowth event was scheduled
    assert!(
        resource_grid.pending_events() > 0,
        "Regrowth event should be scheduled"
    );

    println!("✅ Direct ResourceGrid consumption test passed");
    println!("   Initial biomass: {}", initial_biomass);
    println!("   Consumed: {}", consumed);
    println!("   Final biomass: {}", final_biomass);
    println!(
        "   Pending regrowth events: {}",
        resource_grid.pending_events()
    );
}

#[test]
fn test_resource_grid_empty_consumption() {
    let mut resource_grid = ResourceGrid::new();
    let tile = IVec2::new(5, 5);

    // Try to consume from empty grid
    let consumed = resource_grid.consume_at(tile, 100.0, 0.3);
    assert_eq!(consumed, 0.0, "Should not consume from empty grid");
    assert_eq!(
        resource_grid.pending_events(),
        0,
        "Should not schedule regrowth for empty grid"
    );

    println!("✅ Empty grid consumption test passed");
}

#[test]
fn test_resource_grid_multiple_consumptions() {
    let mut resource_grid = ResourceGrid::new();
    let tile = IVec2::new(5, 5);
    let initial_biomass = 80.0;

    // Create a cell with biomass
    resource_grid
        .get_or_create_cell(tile, 100.0, 1.0)
        .total_biomass = initial_biomass;

    let total_consumed = 0.0;
    let num_consumptions = 3;

    // Multiple consumptions
    for i in 0..num_consumptions {
        let before = resource_grid.get_cell(tile).unwrap().total_biomass;
        let consumed = resource_grid.consume_at(tile, 100.0, 0.3);
        let after = resource_grid.get_cell(tile).unwrap().total_biomass;

        println!(
            "   Consumption {}: {:.1} -> {:.1} (consumed {:.1})",
            i + 1,
            before,
            after,
            consumed
        );

        assert!(
            consumed > 0.0,
            "Should consume biomass on iteration {}",
            i + 1
        );
        assert!(
            after < before,
            "Biomass should decrease on iteration {}",
            i + 1
        );
    }

    // Verify multiple regrowth events scheduled
    assert_eq!(
        resource_grid.pending_events(),
        num_consumptions,
        "Should have scheduled regrowth for each consumption"
    );

    println!("✅ Multiple consumptions test passed");
}
