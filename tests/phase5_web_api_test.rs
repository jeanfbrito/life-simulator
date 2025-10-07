//! Phase 5: Web/API & Heatmap Validation Tests
//!
//! This test file validates the Phase 5 web API and heatmap functionality:
//! 1. API endpoints return correct grid state from ResourceGrid
//! 2. Heatmap refresh performance under 5ms
//! 3. On-demand refresh with dirty flag
//! 4. Performance metrics aggregation
//! 5. Integration with ChunkLOD system

use std::time::Instant;

use serde_json::Value;

#[test]
fn test_phase5_biomass_heatmap_api() {
    println!("🌡️ Testing Phase 5 Biomass Heatmap API");

    let start_time = Instant::now();

    // Test the new Phase 5 biomass heatmap function
    let heatmap_json = life_simulator::vegetation::get_biomass_heatmap_json();

    let elapsed = start_time.elapsed();

    assert!(!heatmap_json.is_empty(), "Heatmap JSON should not be empty");

    let parsed: Value =
        serde_json::from_str(&heatmap_json).expect("heatmap response should be valid JSON");

    assert!(parsed.get("heatmap").is_some(), "Missing heatmap payload");
    assert!(
        parsed.get("max_biomass").is_some(),
        "Missing max_biomass field"
    );
    assert!(parsed.get("metadata").is_some(), "Missing metadata field");

    let metadata = parsed["metadata"]
        .as_object()
        .expect("metadata should be an object");

    let data_source = metadata
        .get("data_source")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(
        matches!(data_source, "resource_grid_lod" | "none"),
        "Unexpected heatmap data source: {data_source}"
    );
    assert!(
        metadata.contains_key("status"),
        "Metadata should track status"
    );
    assert!(
        metadata.contains_key("note"),
        "Metadata should include note"
    );

    // Performance should be under 10ms
    assert!(
        elapsed.as_millis() < 10,
        "Heatmap generation should be under 10ms, took {}ms",
        elapsed.as_millis()
    );

    println!(
        "✅ Biomass Heatmap API test passed in {}ms",
        elapsed.as_millis()
    );
    println!("   JSON length: {} characters", heatmap_json.len());
}

#[test]
fn test_phase5_performance_metrics_api() {
    println!("📊 Testing Phase 5 Performance Metrics API");

    let start_time = Instant::now();

    // Test the new Phase 5 performance metrics function
    let metrics_json = life_simulator::vegetation::get_performance_metrics_json();

    let elapsed = start_time.elapsed();

    // Validate basic JSON structure
    assert!(metrics_json.len() > 0, "Metrics JSON should not be empty");

    let parsed: Value =
        serde_json::from_str(&metrics_json).expect("performance metrics should be valid JSON");

    // Validate Phase 5 specific features
    assert!(
        parsed.get("resource_grid").is_some(),
        "Should contain ResourceGrid metrics"
    );
    assert!(
        parsed.get("chunk_lod").is_some(),
        "Should contain Chunk LOD metrics"
    );
    assert!(
        parsed.get("heatmap_refresh").is_some(),
        "Should contain heatmap refresh metrics"
    );
    assert!(
        parsed.get("performance").is_some(),
        "Should contain overall performance metrics"
    );

    // Validate specific metrics
    assert!(
        parsed["resource_grid"].get("active_cells").is_some(),
        "Should contain active cells count"
    );
    assert!(
        parsed["chunk_lod"].get("total_chunks").is_some(),
        "Should contain total chunks count"
    );
    assert!(
        parsed["performance"].get("lod_efficiency").is_some(),
        "Should contain LOD efficiency"
    );
    assert!(
        parsed["performance"].get("memory_efficiency").is_some(),
        "Should contain memory efficiency"
    );

    // Performance should be under 10ms for metrics
    assert!(
        elapsed.as_millis() < 10,
        "Metrics generation should be under 10ms, took {}ms",
        elapsed.as_millis()
    );

    println!(
        "✅ Performance Metrics API test passed in {}ms",
        elapsed.as_millis()
    );
    println!("   JSON length: {} characters", metrics_json.len());
}

#[test]
fn test_heatmap_refresh_manager() {
    println!("🔄 Testing HeatmapRefreshManager");

    // Create a HeatmapRefreshManager
    let mut manager = life_simulator::vegetation::HeatmapRefreshManager::default();

    // Test initial state
    assert!(manager.dirty, "Manager should start dirty");
    assert_eq!(manager.last_refresh_tick, 0, "Last refresh should be 0");
    assert_eq!(manager.refresh_count, 0, "Refresh count should start at 0");
    assert_eq!(
        manager.refresh_interval, 50,
        "Default refresh interval should be 50"
    );

    // Test needs_refresh logic
    assert!(manager.needs_refresh(10), "Should need refresh when dirty");

    // Test mark_refreshed
    manager.mark_refreshed(10, 2);
    assert!(!manager.dirty, "Should not be dirty after refresh");
    assert_eq!(
        manager.last_refresh_tick, 10,
        "Last refresh tick should be updated"
    );
    assert_eq!(manager.refresh_count, 1, "Refresh count should increment");
    assert_eq!(
        manager.last_generation_time_ms, 2,
        "Generation time should be recorded"
    );

    // Test interval-based refresh
    assert!(
        !manager.needs_refresh(30),
        "Should not need refresh within interval"
    );
    assert!(
        manager.needs_refresh(70),
        "Should need refresh after interval"
    );

    // Test mark_dirty
    manager.mark_dirty();
    assert!(manager.dirty, "Should be dirty after mark_dirty");
    assert!(manager.needs_refresh(80), "Should need refresh when dirty");

    // Test get_stats
    let stats = manager.get_stats();
    assert!(
        stats["dirty"].as_bool().unwrap(),
        "Stats should reflect dirty state"
    );
    assert_eq!(
        stats["last_refresh_tick"].as_u64().unwrap(),
        10,
        "Stats should show last refresh"
    );
    assert_eq!(
        stats["refresh_count"].as_u64().unwrap(),
        1,
        "Stats should show refresh count"
    );

    println!("✅ HeatmapRefreshManager test passed");
    println!(
        "   Final state: dirty={}, refresh_count={}, last_refresh={}",
        manager.dirty, manager.refresh_count, manager.last_refresh_tick
    );
}

#[test]
fn test_phase5_api_response_structure() {
    println!("🏗️ Testing Phase 5 API Response Structure");

    // Test biomass heatmap structure
    let heatmap_json = life_simulator::vegetation::get_biomass_heatmap_json();

    // Parse JSON to validate structure
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&heatmap_json) {
        // Validate required fields
        assert!(parsed.get("heatmap").is_some(), "Missing 'heatmap' field");
        assert!(
            parsed.get("max_biomass").is_some(),
            "Missing 'max_biomass' field"
        );
        assert!(
            parsed.get("tile_size").is_some(),
            "Missing 'tile_size' field"
        );
        assert!(parsed.get("metadata").is_some(), "Missing 'metadata' field");

        // Validate metadata structure
        if let Some(metadata) = parsed.get("metadata").and_then(|v| v.as_object()) {
            assert!(
                metadata.contains_key("updated_tick"),
                "Missing 'updated_tick' in metadata"
            );
            assert!(
                metadata.contains_key("grid_size"),
                "Missing 'grid_size' in metadata"
            );
            assert!(
                metadata.contains_key("scale"),
                "Missing 'scale' in metadata"
            );
            assert!(
                metadata.contains_key("data_source"),
                "Missing 'data_source' in metadata"
            );
            assert!(
                metadata.contains_key("status"),
                "Missing 'status' in metadata"
            );
            assert!(metadata.contains_key("note"), "Missing 'note' in metadata");
        } else {
            panic!("Metadata should be an object");
        }
    } else {
        panic!("Failed to parse heatmap JSON");
    }

    // Test performance metrics structure
    let metrics_json = life_simulator::vegetation::get_performance_metrics_json();

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&metrics_json) {
        // Validate required top-level fields
        assert!(
            parsed.get("resource_grid").is_some(),
            "Missing 'resource_grid' field"
        );
        assert!(
            parsed.get("chunk_lod").is_some(),
            "Missing 'chunk_lod' field"
        );
        assert!(
            parsed.get("heatmap_refresh").is_some(),
            "Missing 'heatmap_refresh' field"
        );
        assert!(
            parsed.get("performance").is_some(),
            "Missing 'performance' field"
        );

        // Validate resource_grid structure
        if let Some(resource_grid) = parsed.get("resource_grid").and_then(|v| v.as_object()) {
            assert!(
                resource_grid.contains_key("active_cells"),
                "Missing 'active_cells'"
            );
            assert!(
                resource_grid.contains_key("pending_events"),
                "Missing 'pending_events'"
            );
            assert!(
                resource_grid.contains_key("events_processed"),
                "Missing 'events_processed'"
            );
            assert!(
                resource_grid.contains_key("processing_time_us"),
                "Missing 'processing_time_us'"
            );
        }

        // Validate chunk_lod structure
        if let Some(chunk_lod) = parsed.get("chunk_lod").and_then(|v| v.as_object()) {
            assert!(
                chunk_lod.contains_key("total_chunks"),
                "Missing 'total_chunks'"
            );
            assert!(chunk_lod.contains_key("hot_chunks"), "Missing 'hot_chunks'");
            assert!(
                chunk_lod.contains_key("warm_chunks"),
                "Missing 'warm_chunks'"
            );
            assert!(
                chunk_lod.contains_key("cold_chunks"),
                "Missing 'cold_chunks'"
            );
            assert!(
                chunk_lod.contains_key("active_chunks"),
                "Missing 'active_chunks'"
            );
        }
    } else {
        panic!("Failed to parse metrics JSON");
    }

    println!("✅ API Response Structure test passed");
    println!("   Heatmap JSON: {} bytes", heatmap_json.len());
    println!("   Metrics JSON: {} bytes", metrics_json.len());
}

#[test]
fn test_phase5_performance_benchmarks() {
    println!("⚡ Testing Phase 5 Performance Benchmarks");

    const NUM_ITERATIONS: usize = 100;

    // Benchmark heatmap generation
    let heatmap_times: Vec<_> = (0..NUM_ITERATIONS)
        .map(|_| {
            let start = Instant::now();
            let _json = life_simulator::vegetation::get_biomass_heatmap_json();
            start.elapsed()
        })
        .collect();

    let avg_heatmap_time =
        heatmap_times.iter().sum::<std::time::Duration>() / NUM_ITERATIONS as u32;
    let max_heatmap_time = heatmap_times.iter().max().unwrap();

    // Benchmark metrics generation
    let metrics_times: Vec<_> = (0..NUM_ITERATIONS)
        .map(|_| {
            let start = Instant::now();
            let _json = life_simulator::vegetation::get_performance_metrics_json();
            start.elapsed()
        })
        .collect();

    let avg_metrics_time =
        metrics_times.iter().sum::<std::time::Duration>() / NUM_ITERATIONS as u32;
    let max_metrics_time = metrics_times.iter().max().unwrap();

    // Validate performance targets
    assert!(
        avg_heatmap_time.as_millis() < 10,
        "Average heatmap time should be <10ms, was {}ms",
        avg_heatmap_time.as_millis()
    );
    assert!(
        max_heatmap_time.as_millis() < 20,
        "Maximum heatmap time should be <20ms, was {}ms",
        max_heatmap_time.as_millis()
    );

    assert!(
        avg_metrics_time.as_millis() < 5,
        "Average metrics time should be <5ms, was {}ms",
        avg_metrics_time.as_millis()
    );
    assert!(
        max_metrics_time.as_millis() < 10,
        "Maximum metrics time should be <10ms, was {}ms",
        max_metrics_time.as_millis()
    );

    println!("✅ Performance Benchmarks test passed");
    println!(
        "   Heatmap: avg={}ms, max={}ms",
        avg_heatmap_time.as_millis(),
        max_heatmap_time.as_millis()
    );
    println!(
        "   Metrics: avg={}ms, max={}ms",
        avg_metrics_time.as_millis(),
        max_metrics_time.as_millis()
    );
}

#[test]
fn test_phase5_integration_workflow() {
    println!("🔗 Testing Phase 5 Integration Workflow");

    // Simulate a complete Phase 5 workflow

    // 1. Initialize refresh manager
    let mut refresh_manager = life_simulator::vegetation::HeatmapRefreshManager::default();
    let mut current_tick = 0u64;

    // 2. Simulate multiple API calls over time
    for i in 0..10 {
        current_tick += 10;

        // Check if refresh is needed
        let needs_refresh = refresh_manager.needs_refresh(current_tick);

        // Generate heatmap (simulates API call)
        let heatmap_start = Instant::now();
        let _heatmap_json = life_simulator::vegetation::get_biomass_heatmap_json();
        let heatmap_time = heatmap_start.elapsed();

        // Generate metrics (simulates API call)
        let metrics_start = Instant::now();
        let _metrics_json = life_simulator::vegetation::get_performance_metrics_json();
        let metrics_time = metrics_start.elapsed();

        // Simulate refresh management
        if needs_refresh {
            refresh_manager.mark_refreshed(current_tick, heatmap_time.as_millis() as u64);
            println!(
                "   Tick {}: Refreshed heatmap in {}ms",
                current_tick,
                heatmap_time.as_millis()
            );
        } else {
            println!(
                "   Tick {}: Used cached data ({}ms heatmap, {}ms metrics)",
                current_tick,
                heatmap_time.as_millis(),
                metrics_time.as_millis()
            );
        }

        // Validate performance
        assert!(
            heatmap_time.as_millis() < 10,
            "Heatmap should be fast: {}ms",
            heatmap_time.as_millis()
        );
        assert!(
            metrics_time.as_millis() < 10,
            "Metrics should be fast: {}ms",
            metrics_time.as_millis()
        );
    }

    // Validate final state
    assert!(
        refresh_manager.refresh_count > 0,
        "Should have performed refreshes"
    );
    assert!(!refresh_manager.dirty, "Should not be dirty at end");

    println!("✅ Integration Workflow test passed");
    println!("   Total refreshes: {}", refresh_manager.refresh_count);
    println!("   Final tick: {}", current_tick);
}
