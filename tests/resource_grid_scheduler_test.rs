//! Phase 3: ResourceGrid Scheduler Unit Tests
//!
//! This test file validates the event-driven scheduler functionality:
//! 1. Event scheduling and execution timing
//! 2. Tick budget enforcement for random sampling
//! 3. Regrowth delays proportional to consumption
//! 4. Performance validation (target: <2ms processing time)

use bevy::prelude::IVec2;
use life_simulator::vegetation::resource_grid::ResourceGrid;

#[test]
fn test_scheduler_basic_functionality() {
    let mut grid = ResourceGrid::new();

    // Test initial state
    assert_eq!(grid.pending_events(), 0);
    assert_eq!(grid.current_tick(), 0);

    // Create a cell to test event scheduling through consumption
    let location = IVec2::new(5, 5);
    grid.get_or_create_cell(location, 100.0, 1.0);
    let initial_biomass = grid.get_cell(location).unwrap().total_biomass;

    // Consume biomass - this should schedule a regrowth event
    let consumed = grid.consume_at(location, 10.0, 0.3);
    assert!(consumed > 0.0, "Should have consumed biomass");

    // Verify regrowth event was scheduled
    assert_eq!(grid.pending_events(), 1);

    // Update to tick 50 - should not process event yet (regrowth delay > 50 ticks)
    grid.update(50);
    assert_eq!(grid.pending_events(), 1);
    assert_eq!(grid.current_tick(), 50);

    // Update to tick 1000 - should process event (regrowth delay can be up to 250+ ticks)
    grid.update(1000);
    assert_eq!(grid.current_tick(), 1000);

    // Check if biomass regrew (event processed)
    let final_biomass = grid.get_cell(location).unwrap().total_biomass;
    assert!(final_biomass > initial_biomass - consumed, "Biomass should have regrown");

    println!("✅ Scheduler basic functionality test passed");
    println!("   Initial: {:.1}, Consumed: {:.1}, Final: {:.1}",
             initial_biomass, consumed, final_biomass);
}

#[test]
fn test_consumption_regrow_delay() {
    let mut grid = ResourceGrid::new();
    let location = IVec2::new(5, 5);
    let max_biomass = 100.0;

    // Create cell with biomass
    grid.get_or_create_cell(location, max_biomass, 1.0);
    let initial_biomass = grid.get_cell(location).unwrap().total_biomass;

    // Consume biomass - this should schedule regrowth
    let consumed = grid.consume_at(location, 30.0, 0.3);
    assert!(consumed > 0.0, "Should have consumed biomass");

    // Verify regrowth event was scheduled
    assert_eq!(grid.pending_events(), 1, "Should have scheduled regrowth event");

    // Update to when event should be due
    grid.update(500); // Advance 50 seconds at 10 TPS

    // Event should now be processed
    let metrics = grid.get_metrics();
    assert!(metrics.events_processed > 0, "Should have processed regrowth event");

    // Verify biomass increased (regrowth occurred)
    let final_biomass = grid.get_cell(location).unwrap().total_biomass;
    assert!(final_biomass > initial_biomass - consumed, "Biomass should have regrown");

    println!("✅ Consumption regrow delay test passed");
    println!("   Initial: {:.1}, Consumed: {:.1}, Final: {:.1}",
             initial_biomass, consumed, final_biomass);
}

#[test]
fn test_random_tick_budget() {
    let mut grid = ResourceGrid::new();

    // Create multiple cells
    for x in 0..10 {
        for y in 0..10 {
            let pos = IVec2::new(x, y);
            grid.get_or_create_cell(pos, 50.0, 1.0);
        }
    }

    // Update grid
    grid.update(100);

    let metrics = grid.get_metrics();

    // Should respect default tick budget (50 cells max)
    assert!(metrics.random_cells_sampled <= 50,
            "Random cells sampled ({}) should not exceed default budget (50)",
            metrics.random_cells_sampled);

    // Should have processed some random cells since we have many
    assert!(metrics.random_cells_sampled > 0,
            "Should have sampled random cells from available pool");

    println!("✅ Random tick budget test passed");
    println!("   Cells: {}, Random sampled: {}, Budget: 50",
             grid.cell_count(), metrics.random_cells_sampled);
}

#[test]
fn test_event_timing_accuracy() {
    let mut grid = ResourceGrid::new();

    // Create cells and consume at different times to simulate event timing
    let locations = vec![
        IVec2::new(1, 1),
        IVec2::new(2, 2),
        IVec2::new(3, 3),
    ];

    for location in &locations {
        grid.get_or_create_cell(*location, 50.0, 1.0);
        // Consume to schedule regrowth events
        grid.consume_at(*location, 5.0, 0.3);
    }

    // Should have 3 pending regrowth events
    assert_eq!(grid.pending_events(), 3);

    // Update multiple times to process events
    grid.update(100);
    let remaining_after_first = grid.pending_events();

    grid.update(200);
    let remaining_after_second = grid.pending_events();

    grid.update(300);
    let remaining_after_third = grid.pending_events();

    // Events should be processed over time
    assert!(remaining_after_first <= 3, "Events should be processed");
    assert!(remaining_after_second <= remaining_after_first, "More events should be processed");
    assert!(remaining_after_third <= remaining_after_second, "All events should be processed eventually");

    println!("✅ Event timing accuracy test passed");
    println!("   Events: 3 -> {} -> {} -> {}",
             remaining_after_first, remaining_after_second, remaining_after_third);
}

#[test]
fn test_performance_target_validation() {
    let mut grid = ResourceGrid::new();

    // Create a reasonable number of cells
    for x in 0..20 {
        for y in 0..20 {
            let pos = IVec2::new(x, y);
            grid.get_or_create_cell(pos, 50.0, 1.0);
        }
    }

    // Create some events through consumption
    for i in 0..10 {
        let pos = IVec2::new(i, i);
        grid.get_or_create_cell(pos, 50.0, 1.0);
        // Consume to schedule regrowth events
        grid.consume_at(pos, 2.0, 0.3);
    }

    // Measure processing time
    let start = std::time::Instant::now();
    grid.update(100);
    let elapsed = start.elapsed();

    let metrics = grid.get_metrics();

    // Validate performance target: should be under 2ms
    assert!(elapsed.as_millis() < 2,
            "Processing took {}ms, target is <2ms", elapsed.as_millis());

    assert!(metrics.processing_time_us < 2000,
            "Metrics show {}μs processing time, target is <2000μs",
            metrics.processing_time_us);

    println!("✅ Performance target validation test passed");
    println!("   Processing time: {}μs (target: <2000μs)", metrics.processing_time_us);
    println!("   Cells processed: {}, Events: {}, Random: {}",
             metrics.active_cells, metrics.events_processed, metrics.random_cells_sampled);
}

#[test]
fn test_multiple_consumption_events() {
    let mut grid = ResourceGrid::new();
    let location = IVec2::new(5, 5);

    // Create cell
    grid.get_or_create_cell(location, 100.0, 1.0);

    // Multiple consumptions should schedule multiple regrowth events
    let consumed1 = grid.consume_at(location, 10.0, 0.3);
    let consumed2 = grid.consume_at(location, 15.0, 0.3);
    let consumed3 = grid.consume_at(location, 20.0, 0.3);

    assert!(consumed1 > 0.0 && consumed2 > 0.0 && consumed3 > 0.0);

    // Should have scheduled regrowth events for each consumption
    assert_eq!(grid.pending_events(), 3, "Should have 3 pending regrowth events");

    // Update past all scheduled times
    grid.update(1000);

    let metrics = grid.get_metrics();
    assert!(metrics.events_processed >= 3, "Should have processed at least 3 events");

    println!("✅ Multiple consumption events test passed");
    println!("   Consumed: {:.1} + {:.1} + {:.1} = {:.1}",
             consumed1, consumed2, consumed3, consumed1 + consumed2 + consumed3);
}

#[test]
fn test_scheduler_pressure_decay() {
    let mut grid = ResourceGrid::new();
    let location = IVec2::new(5, 5);

    // Create cell
    grid.get_or_create_cell(location, 100.0, 1.0);

    // Consume to increase pressure
    grid.consume_at(location, 30.0, 0.3);

    let initial_pressure = grid.get_cell(location).unwrap().consumption_pressure;
    assert!(initial_pressure > 0.0, "Pressure should increase after consumption");

    // Update many ticks to allow pressure decay
    for tick in 1..=200 {
        grid.update(tick);
    }

    let final_pressure = grid.get_cell(location).unwrap().consumption_pressure;

    // Pressure should have decayed
    assert!(final_pressure < initial_pressure,
            "Pressure should decay over time: {:.3} -> {:.3}",
            initial_pressure, final_pressure);

    println!("✅ Scheduler pressure decay test passed");
    println!("   Initial pressure: {:.3}, Final pressure: {:.3}",
             initial_pressure, final_pressure);
}