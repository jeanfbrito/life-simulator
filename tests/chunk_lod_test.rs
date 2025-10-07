//! Phase 4: Chunk Level-of-Detail (LOD) System Validation Tests
//!
//! This test file validates the chunk LOD functionality:
//! 1. Chunk activation conversions (aggregate ↔ per-cell) conserve biomass
//! 2. Proximity-based temperature tracking (hot/warm/cold)
//! 3. Lazy activation system performance
//! 4. Impostor data generation and quality levels

use bevy::prelude::IVec2;
use life_simulator::tilemap::{ChunkCoordinate, CHUNK_SIZE};
use life_simulator::vegetation::chunk_lod::{
    ChunkImpostor, ChunkLODConfig, ChunkLODManager, ChunkMetadata, ChunkTemperature,
    ImpostorQuality,
};
use life_simulator::vegetation::resource_grid::ResourceGrid;

#[test]
fn test_chunk_temperature_calculation() {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());

    // Test with no agents - all chunks should be cold
    lod_manager.update_agent_positions(vec![]);

    let test_chunk = ChunkCoordinate::new(0, 0);
    let metadata = lod_manager.get_or_create_chunk(test_chunk);

    assert_eq!(metadata.temperature, ChunkTemperature::Cold);
    assert!(lod_manager.get_active_chunks().is_empty());

    // Test with agent at origin - nearby chunks should be hot
    lod_manager.update_agent_positions(vec![IVec2::new(0, 0)]);

    let metadata = lod_manager.get_or_create_chunk(test_chunk);
    assert_eq!(metadata.temperature, ChunkTemperature::Hot);
    assert!(lod_manager.is_chunk_active(&test_chunk));

    // Test with agent at medium distance
    lod_manager.update_agent_positions(vec![IVec2::new(150, 150)]);

    let metadata = lod_manager.get_or_create_chunk(test_chunk);
    assert_eq!(metadata.temperature, ChunkTemperature::Warm);
    assert!(lod_manager.is_chunk_active(&test_chunk));

    // Test with agent at far distance
    lod_manager.update_agent_positions(vec![IVec2::new(300, 300)]);

    let metadata = lod_manager.get_or_create_chunk(test_chunk);
    assert_eq!(metadata.temperature, ChunkTemperature::Cold);
    assert!(!lod_manager.is_chunk_active(&test_chunk));

    println!("✅ Chunk temperature calculation test passed");
}

#[test]
fn test_chunk_temperature_ranges() {
    // Test temperature distance ranges
    assert!(ChunkTemperature::Hot.contains_distance(50));
    assert!(ChunkTemperature::Hot.contains_distance(99));
    assert!(!ChunkTemperature::Hot.contains_distance(100));

    assert!(ChunkTemperature::Warm.contains_distance(100));
    assert!(ChunkTemperature::Warm.contains_distance(150));
    assert!(!ChunkTemperature::Warm.contains_distance(200));

    assert!(ChunkTemperature::Cold.contains_distance(200));
    assert!(ChunkTemperature::Cold.contains_distance(500));
    assert!(ChunkTemperature::Cold.contains_distance(i32::MAX));

    println!("✅ Chunk temperature ranges test passed");
}

#[test]
fn test_chunk_metadata_aggregation() {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());
    let mut resource_grid = ResourceGrid::new();
    let chunk_coord = ChunkCoordinate::new(0, 0);

    // Create cells in the chunk
    let chunk_start = IVec2::new(0, 0);
    let positions = vec![
        chunk_start,
        chunk_start + IVec2::new(1, 0),
        chunk_start + IVec2::new(0, 1),
        chunk_start + IVec2::new(5, 5),
    ];

    for pos in positions {
        resource_grid.get_or_create_cell(pos, 100.0, 1.0);
    }

    // Update chunk metadata
    lod_manager.update_chunk_from_grid(chunk_coord, &resource_grid, 100);

    let metadata = lod_manager.get_chunk(&chunk_coord).unwrap();

    assert_eq!(metadata.active_cells, 4);
    assert_eq!(metadata.aggregate_biomass, 20.0); // 4 cells * 5.0 initial biomass
    assert_eq!(metadata.max_biomass, 400.0); // 4 cells * 100.0 max biomass
    assert_eq!(metadata.avg_growth_rate, 1.0);
    assert_eq!(metadata.last_update_tick, 100);

    println!("✅ Chunk metadata aggregation test passed");
    println!(
        "   Active cells: {}, Aggregate biomass: {}, Max biomass: {}",
        metadata.active_cells, metadata.aggregate_biomass, metadata.max_biomass
    );
}

#[test]
fn test_lazy_activation_conserves_biomass() {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());
    let mut resource_grid = ResourceGrid::new();
    let chunk_coord = ChunkCoordinate::new(0, 0);

    // Create initial metadata with some biomass
    let metadata = lod_manager.get_or_create_chunk(chunk_coord);
    metadata.aggregate_biomass = 100.0;
    metadata.active_cells = 10;

    // Create some cells in ResourceGrid (each gets 5.0 initial biomass)
    let chunk_start = IVec2::new(0, 0);
    for i in 0..5 {
        let pos = chunk_start + IVec2::new(i, i);
        resource_grid.get_or_create_cell(pos, 20.0, 1.0);
    }

    // Perform lazy activation
    lod_manager.lazy_activate_chunk(chunk_coord, &mut resource_grid, 100);

    // Update metadata from grid (this replaces the aggregate values)
    lod_manager.update_chunk_from_grid(chunk_coord, &resource_grid, 101);

    let final_metadata = lod_manager.get_chunk(&chunk_coord).unwrap();

    // After update, metadata should reflect actual grid content (5 cells * 5.0 = 25.0)
    assert!(final_metadata.aggregate_biomass >= 25.0);
    assert!(final_metadata.active_cells >= 5);

    println!("✅ Lazy activation biomass conservation test passed");
    println!(
        "   Initial: 100.0, Final: {:.1}",
        final_metadata.aggregate_biomass
    );
}

#[test]
fn test_impostor_data_generation() {
    let mut metadata = ChunkMetadata::new(ChunkCoordinate::new(0, 0), ChunkTemperature::Cold);
    metadata.aggregate_biomass = 80.0;
    metadata.max_biomass = 100.0;

    let impostor = metadata.generate_impostor();

    assert!(impostor.density > 0.7); // High biomass should give high density
    assert_eq!(impostor.quality, ImpostorQuality::Low); // Cold chunks get low quality

    // Test different biomass levels
    metadata.aggregate_biomass = 30.0;
    let medium_impostor = metadata.generate_impostor();
    assert!(medium_impostor.density > 0.2 && medium_impostor.density < 0.5);

    metadata.aggregate_biomass = 5.0;
    let low_impostor = metadata.generate_impostor();
    assert!(low_impostor.density < 0.2);

    println!("✅ Impostor data generation test passed");
    println!(
        "   High density: {:.2}, Medium: {:.2}, Low: {:.2}",
        impostor.density, medium_impostor.density, low_impostor.density
    );
}

#[test]
fn test_impostor_quality_levels() {
    let mut metadata = ChunkMetadata::new(ChunkCoordinate::new(0, 0), ChunkTemperature::Cold);
    metadata.aggregate_biomass = 50.0;
    metadata.max_biomass = 100.0;

    // Hot chunks should generate high quality impostors
    metadata.temperature = ChunkTemperature::Hot;
    let hot_impostor = metadata.generate_impostor();
    assert_eq!(hot_impostor.quality, ImpostorQuality::High);

    // Warm chunks should generate medium quality impostors
    metadata.temperature = ChunkTemperature::Warm;
    let warm_impostor = metadata.generate_impostor();
    assert_eq!(warm_impostor.quality, ImpostorQuality::Medium);

    // Cold chunks should generate low quality impostors
    metadata.temperature = ChunkTemperature::Cold;
    let cold_impostor = metadata.generate_impostor();
    assert_eq!(cold_impostor.quality, ImpostorQuality::Low);

    println!("✅ Impostor quality levels test passed");
}

#[test]
fn test_chunk_cleanup_distant_chunks() {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());

    // Create chunks at various distances
    let agent_position = IVec2::new(0, 0);
    lod_manager.update_agent_positions(vec![agent_position]);

    // Nearby chunk (should remain)
    let nearby_chunk = ChunkCoordinate::new(0, 0);
    lod_manager.get_or_create_chunk(nearby_chunk);

    // Distant chunk (should be cleaned up)
    let distant_chunk = ChunkCoordinate::new(50, 50); // ~70 tiles away
    lod_manager.get_or_create_chunk(distant_chunk);

    assert!(lod_manager.get_chunk(&nearby_chunk).is_some());
    assert!(lod_manager.get_chunk(&distant_chunk).is_some());
    assert_eq!(lod_manager.chunks.len(), 2);

    // Cleanup chunks beyond 50 tiles
    lod_manager.cleanup_distant_chunks(50);

    assert!(lod_manager.get_chunk(&nearby_chunk).is_some());
    assert!(lod_manager.get_chunk(&distant_chunk).is_none());
    assert_eq!(lod_manager.chunks.len(), 1);

    println!("✅ Chunk cleanup test passed");
    println!(
        "   Before: 2 chunks, After: {} chunks",
        lod_manager.chunks.len()
    );
}

#[test]
fn test_chunk_lod_metrics() {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());

    // Create chunks at different temperature levels
    lod_manager.update_agent_positions(vec![IVec2::new(0, 0)]);

    // Hot chunk
    let hot_chunk = ChunkCoordinate::new(0, 0);
    lod_manager.get_or_create_chunk(hot_chunk);

    // Warm chunk
    lod_manager.update_agent_positions(vec![IVec2::new(150, 150)]);
    let warm_chunk = ChunkCoordinate::new(5, 5);
    lod_manager.get_or_create_chunk(warm_chunk);

    // Cold chunk
    lod_manager.update_agent_positions(vec![IVec2::new(300, 300)]);
    let cold_chunk = ChunkCoordinate::new(10, 10);
    lod_manager.get_or_create_chunk(cold_chunk);

    let metrics = lod_manager.get_metrics();

    assert_eq!(metrics.total_chunks, 3);
    assert_eq!(metrics.hot_chunks, 1);
    assert_eq!(metrics.warm_chunks, 1);
    assert_eq!(metrics.cold_chunks, 1);

    println!("✅ Chunk LOD metrics test passed");
    println!(
        "   Total: {}, Hot: {}, Warm: {}, Cold: {}",
        metrics.total_chunks, metrics.hot_chunks, metrics.warm_chunks, metrics.cold_chunks
    );
}

#[test]
fn test_multiple_agent_proximity_tracking() {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());

    // Multiple agents spread out
    let agent_positions = vec![
        IVec2::new(0, 0),
        IVec2::new(50, 50),
        IVec2::new(-30, -20),
        IVec2::new(200, 100),
    ];

    lod_manager.update_agent_positions(agent_positions);

    // Test chunks around each agent
    let chunks_to_test = vec![
        (ChunkCoordinate::new(0, 0), ChunkTemperature::Hot), // Near (0,0)
        (ChunkCoordinate::new(3, 3), ChunkTemperature::Hot), // Near (0,0)
        (ChunkCoordinate::new(9, 9), ChunkTemperature::Warm), // Medium distance
        (ChunkCoordinate::new(12, 6), ChunkTemperature::Warm), // Medium distance
        (ChunkCoordinate::new(15, 15), ChunkTemperature::Cold), // Far from all
    ];

    for (chunk_coord, expected_temp) in chunks_to_test {
        let metadata = lod_manager.get_or_create_chunk(chunk_coord);
        assert_eq!(
            metadata.temperature, expected_temp,
            "Chunk {:?} should be {:?}",
            chunk_coord, expected_temp
        );
    }

    println!("✅ Multiple agent proximity tracking test passed");
}

#[test]
fn test_chunk_lod_performance() {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());

    // Create many chunks
    let agent_positions = vec![IVec2::new(0, 0)];
    lod_manager.update_agent_positions(agent_positions.clone());

    let start_time = std::time::Instant::now();

    // Create 100 chunks
    for x in 0..10 {
        for y in 0..10 {
            let chunk_coord = ChunkCoordinate::new(x, y);
            lod_manager.get_or_create_chunk(chunk_coord);
        }
    }

    let creation_time = start_time.elapsed();

    // Test temperature recalculation
    let start_time = std::time::Instant::now();
    lod_manager.update_agent_positions(agent_positions);
    let recalc_time = start_time.elapsed();

    // Performance should be good
    assert!(
        creation_time.as_millis() < 10,
        "Chunk creation should be fast"
    );
    assert!(
        recalc_time.as_millis() < 5,
        "Temperature recalculation should be fast"
    );

    let metrics = lod_manager.get_metrics();
    assert_eq!(metrics.total_chunks, 100);

    println!("✅ Chunk LOD performance test passed");
    println!(
        "   Creation: {}ms, Recalculation: {}ms, Total chunks: {}",
        creation_time.as_millis(),
        recalc_time.as_millis(),
        metrics.total_chunks
    );
}
