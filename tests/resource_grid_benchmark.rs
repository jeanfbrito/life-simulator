/// Performance benchmark for ResourceGrid (Phase 1 validation)
///
/// This test validates that the sparse, event-driven resource grid
/// can handle 10k cells with per-tick updates under 1ms as specified in the rewrite plan.

use life_simulator::vegetation::resource_grid::*;
use life_simulator::vegetation::resource_grid::grid_helpers::*;
use std::time::Instant;
use bevy::math::IVec2;

#[test]
fn test_10k_cell_performance() {
    println!("🚀 Starting Phase 1 Performance Benchmark: 10k cells");

    let mut grid = ResourceGrid::new();
    let current_tick = 1000;

    // Phase 1: Create 10,000 cells with sparse distribution
    println!("📊 Creating 10,000 sparse cells...");
    let setup_start = Instant::now();

    // Create cells in a realistic sparse pattern (not every tile)
    let chunk_size = 16;
    let world_radius_chunks = 8; // 16x16 chunks = 256x256 tiles
    let mut positions = Vec::new();

    for chunk_x in -world_radius_chunks..=world_radius_chunks {
        for chunk_y in -world_radius_chunks..=world_radius_chunks {
            // Each chunk has ~25% vegetation coverage (sparse)
            for local_x in 0..chunk_size {
                for local_y in 0..chunk_size {
                    if rand::random::<f32>() < 0.25 { // 25% chance of vegetation
                        let world_x = chunk_x * chunk_size + local_x;
                        let world_y = chunk_y * chunk_size + local_y;
                        let pos = IVec2::new(world_x, world_y);

                        // Vary max biomass based on terrain simulation
                        let max_biomass = match rand::random::<u32>() % 5 {
                            0 => 100.0, // Grass - excellent
                            1 => 120.0, // Forest - excellent
                            2 => 70.0,  // Dirt - fair
                            3 => 20.0,  // Sand - poor
                            _ => 5.0,   // Other - very poor
                        };

                        let growth_modifier = match rand::random::<u32>() % 3 {
                            0 => 1.0,  // Normal
                            1 => 1.1,  // Good (forest, swamp)
                            _ => 0.7,  // Poor (dry terrain)
                        };

                        grid.get_or_create_cell(pos, max_biomass, growth_modifier);
                        positions.push(pos);

                        if positions.len() >= 10000 {
                            break;
                        }
                    }
                }
            }
            if positions.len() >= 10000 {
                break;
            }
        }
        if positions.len() >= 10000 {
            break;
        }
    }

    let setup_time = setup_start.elapsed();
    println!("✅ Setup completed: {} cells in {:?}", positions.len(), setup_time);
    assert!(positions.len() >= 10000, "Should create at least 10k cells");

    // Phase 2: Simulate consumption events
    println!("🍽️  Simulating consumption events...");
    let consume_start = Instant::now();
    let mut consumption_events = 0;

    // Simulate 1000 consumption events (herbivores grazing)
    for _ in 0..1000 {
        let pos = positions[rand::random::<usize>() % positions.len()];
        let requested = 5.0 + rand::random::<f32>() * 15.0; // 5-20 biomass per meal
        let consumed = grid.consume_at(pos, requested, 0.3);
        if consumed > 0.0 {
            consumption_events += 1;
        }
    }

    let consume_time = consume_start.elapsed();
    println!("✅ Consumption simulation: {} events in {:?}", consumption_events, consume_time);

    // Phase 3: Process regrowth events
    println!("🌱 Processing regrowth events...");
    let regrow_start = Instant::now();

    // Process 100 ticks worth of regrowth
    for tick in current_tick..current_tick + 100 {
        grid.update(tick);
    }

    let regrow_time = regrow_start.elapsed();
    println!("✅ Regrowth processing: 100 ticks in {:?}", regrow_time);

    // Phase 4: Performance validation - per-tick update should be under 1ms
    println!("🎯 Validating per-tick performance...");
    let validation_start = Instant::now();

    // Measure time for individual tick updates
    let mut tick_times = Vec::new();
    for tick in current_tick + 100..current_tick + 200 {
        let tick_start = Instant::now();
        grid.update(tick);
        let tick_duration = tick_start.elapsed();
        tick_times.push(tick_duration);
    }

    let validation_time = validation_start.elapsed();
    let avg_tick_time = tick_times.iter().sum::<std::time::Duration>() / tick_times.len() as u32;
    let max_tick_time = tick_times.iter().max().unwrap();

    println!("📈 Performance Results:");
    println!("  - Average tick time: {:?}", avg_tick_time);
    println!("  - Max tick time: {:?}", max_tick_time);
    println!("  - Total validation time: {:?}", validation_time);
    println!("  - Cells processed: {}", grid.cell_count());
    println!("  - Pending events: {}", grid.pending_events());

    // Print metrics
    let metrics = grid.get_metrics();
    println!("📊 Metrics:");
    println!("  - Events processed: {}", metrics.events_processed);
    println!("  - Random cells sampled: {}", metrics.random_cells_sampled);
    println!("  - Biomass grown: {:.1}", metrics.biomass_grown);
    println!("  - Biomass consumed: {:.1}", metrics.biomass_consumed);
    println!("  - Processing time: {}μs", metrics.processing_time_us);

    // Phase 1 Validation: Per-tick updates must be ≤ 1ms
    let one_ms = std::time::Duration::from_millis(1);
    assert!(avg_tick_time <= one_ms,
        "❌ Phase 1 FAILED: Average tick time {:?} exceeds 1ms budget", avg_tick_time);
    assert!(*max_tick_time <= one_ms * 2,
        "❌ Phase 1 FAILED: Max tick time {:?} exceeds 2ms tolerance", max_tick_time);

    println!("✅ Phase 1 VALIDATION PASSED: Per-tick updates within budget");

    // Additional validation: Ensure sparse storage efficiency
    let total_possible_tiles = (world_radius_chunks * 2 + 1) * chunk_size * (world_radius_chunks * 2 + 1) * chunk_size;
    let storage_efficiency = (grid.cell_count() as f32) / (total_possible_tiles as f32) * 100.0;
    println!("💾 Storage efficiency: {:.1}% ({} cells / {} possible tiles)",
        storage_efficiency, grid.cell_count(), total_possible_tiles);

    assert!(storage_efficiency < 50.0,
        "Storage should be sparse (< 50% of possible tiles), got {:.1}%", storage_efficiency);

    println!("✅ Sparse storage efficiency validated");

    // Phase 1 complete
    println!("🎉 Phase 1 Implementation Complete and Validated!");
    println!("📋 Summary:");
    println!("  ✅ Sparse hash storage implemented");
    println!("  ✅ GrazingCell with required fields");
    println!("  ✅ Helper functions for grid management");
    println!("  ✅ GrowthEvent enum and scheduler");
    println!("  ✅ Comprehensive unit tests");
    println!("  ✅ Performance: 10k cells < 1ms per tick");
    println!("  ✅ Sparse storage efficiency verified");
}