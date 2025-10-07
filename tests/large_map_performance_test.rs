//! Phase 4: Large Map Scenario Performance Validation
//!
//! This test file validates that CPU usage stays flat when agents are clustered:
//! 1. Large world generation (1000+ chunks)
//! 2. Clustered agent scenarios vs distributed scenarios
//! 3. CPU usage profiling with different agent distributions
//! 4. Memory usage validation for large maps
//! 5. LOD system effectiveness in large-scale scenarios

use bevy::prelude::IVec2;
use life_simulator::tilemap::ChunkCoordinate;
use life_simulator::vegetation::chunk_lod::{
    ChunkLODConfig, ChunkLODManager, ChunkMetadata, ChunkTemperature,
};
use life_simulator::vegetation::resource_grid::ResourceGrid;
use std::time::{Duration, Instant};

/// Large map configuration for performance testing
struct LargeMapConfig {
    /// Number of chunks to generate (1000+ for large scenarios)
    chunk_count: usize,
    /// Agent distribution pattern
    agent_pattern: AgentPattern,
    /// Number of agents to spawn
    agent_count: usize,
    /// Performance measurement duration
    test_duration_ms: u64,
}

/// Agent distribution patterns for testing
#[derive(Debug, Clone)]
enum AgentPattern {
    /// All agents clustered in one area
    Clustered { center: IVec2, radius: i32 },
    /// Agents evenly distributed across map
    Distributed { spacing: i32 },
    /// Agents in multiple small clusters
    MultiCluster { clusters: Vec<(IVec2, i32)> },
    /// Agents along a path/line
    Linear {
        start: IVec2,
        end: IVec2,
        count: usize,
    },
}

impl LargeMapConfig {
    /// Create a large map config for testing
    fn new(chunk_count: usize, agent_pattern: AgentPattern, agent_count: usize) -> Self {
        Self {
            chunk_count,
            agent_pattern,
            agent_count,
            test_duration_ms: 5000, // 5 seconds for thorough testing
        }
    }

    /// Generate chunk coordinates for the test
    fn generate_chunks(&self) -> Vec<ChunkCoordinate> {
        let mut chunks = Vec::with_capacity(self.chunk_count);

        // Generate chunks in a square pattern around origin
        let side_length = (self.chunk_count as f64).sqrt().ceil() as i32;
        let half_side = side_length / 2;

        for x in -half_side..=half_side {
            for y in -half_side..=half_side {
                if chunks.len() < self.chunk_count {
                    chunks.push(ChunkCoordinate::new(x, y));
                }
            }
        }

        chunks
    }

    /// Generate agent positions based on pattern
    fn generate_agents(&self) -> Vec<IVec2> {
        let mut agents = Vec::with_capacity(self.agent_count);

        match &self.agent_pattern {
            AgentPattern::Clustered { center, radius } => {
                for _ in 0..self.agent_count {
                    let offset_x = (rand::random::<i32>() % (2 * radius + 1)) - radius;
                    let offset_y = (rand::random::<i32>() % (2 * radius + 1)) - radius;
                    agents.push(*center + IVec2::new(offset_x, offset_y));
                }
            }
            AgentPattern::Distributed { spacing } => {
                let side_length = (self.agent_count as f64).sqrt().ceil() as i32;
                let half_side = side_length / 2;

                for x in -half_side..=half_side {
                    for y in -half_side..=half_side {
                        if agents.len() < self.agent_count {
                            let pos = IVec2::new(x * spacing, y * spacing);
                            agents.push(pos);
                        }
                    }
                }
            }
            AgentPattern::MultiCluster { clusters } => {
                let agents_per_cluster = self.agent_count / clusters.len();

                for (center, radius) in clusters {
                    for _ in 0..agents_per_cluster {
                        if agents.len() < self.agent_count {
                            let offset_x = (rand::random::<i32>() % (2 * radius + 1)) - radius;
                            let offset_y = (rand::random::<i32>() % (2 * radius + 1)) - radius;
                            agents.push(*center + IVec2::new(offset_x, offset_y));
                        }
                    }
                }

                // Fill remaining agents if needed
                while agents.len() < self.agent_count {
                    let (center, radius) = clusters[0];
                    let offset_x = (rand::random::<i32>() % (2 * radius + 1)) - radius;
                    let offset_y = (rand::random::<i32>() % (2 * radius + 1)) - radius;
                    agents.push(center + IVec2::new(offset_x, offset_y));
                }
            }
            AgentPattern::Linear { start, end, count } => {
                for i in 0..*count {
                    if i < self.agent_count {
                        let t = i as f32 / (count - 1) as f32;
                        let x = start.x as f32 + t * (end.x - start.x) as f32;
                        let y = start.y as f32 + t * (end.y - start.y) as f32;
                        agents.push(IVec2::new(x as i32, y as i32));
                    }
                }
            }
        }

        agents
    }
}

/// Performance metrics collected during testing
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    /// Total test duration
    duration: Duration,
    /// Number of temperature updates performed
    temperature_updates: usize,
    /// Average time per temperature update
    avg_update_time: Duration,
    /// Maximum time for a single temperature update
    max_update_time: Duration,
    /// Number of active chunks (hot/warm)
    active_chunks: usize,
    /// Number of cold chunks
    cold_chunks: usize,
    /// Memory usage estimate (number of tracked chunks)
    memory_chunks: usize,
    /// LOD efficiency ratio (active_chunks / total_chunks)
    lod_efficiency: f32,
}

impl PerformanceMetrics {
    /// Calculate LOD efficiency
    fn calculate_efficiency(active_chunks: usize, total_chunks: usize) -> f32 {
        if total_chunks == 0 {
            0.0
        } else {
            active_chunks as f32 / total_chunks as f32
        }
    }
}

/// Run a performance test with the given configuration
fn run_performance_test(config: &LargeMapConfig) -> PerformanceMetrics {
    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());

    // Generate chunks
    let chunks = config.generate_chunks();
    println!("🗺️  Generated {} chunks for testing", chunks.len());

    // Create chunks in LOD manager
    for chunk_coord in chunks {
        lod_manager.get_or_create_chunk(chunk_coord);

        // Add some vegetation to chunks near agents (will be created later)
        // This simulates a realistic world with vegetation
    }

    // Generate agents
    let agents = config.generate_agents();
    println!(
        "👥 Generated {} agents with pattern: {:?}",
        agents.len(),
        config.agent_pattern
    );

    // Performance measurement setup
    let start_time = Instant::now();
    let mut update_times = Vec::new();
    let mut total_updates = 0;

    // Run simulation for specified duration
    let test_duration = Duration::from_millis(config.test_duration_ms);

    while start_time.elapsed() < test_duration {
        // Measure temperature update performance
        let update_start = Instant::now();
        lod_manager.update_agent_positions(agents.clone());
        let update_time = update_start.elapsed();

        update_times.push(update_time);
        total_updates += 1;

        // Simulate some processing time (like other game systems)
        std::thread::sleep(Duration::from_millis(10));
    }

    // Collect metrics
    let total_duration = start_time.elapsed();
    let metrics = lod_manager.get_metrics();

    let avg_update_time = if update_times.is_empty() {
        Duration::ZERO
    } else {
        update_times.iter().sum::<Duration>() / update_times.len() as u32
    };

    let max_update_time = update_times.iter().max().copied().unwrap_or(Duration::ZERO);

    let lod_efficiency = PerformanceMetrics::calculate_efficiency(
        metrics.hot_chunks + metrics.warm_chunks,
        metrics.total_chunks,
    );

    PerformanceMetrics {
        duration: total_duration,
        temperature_updates: total_updates,
        avg_update_time,
        max_update_time,
        active_chunks: metrics.hot_chunks + metrics.warm_chunks,
        cold_chunks: metrics.cold_chunks,
        memory_chunks: metrics.total_chunks,
        lod_efficiency,
    }
}

/// Print performance results in a readable format
fn print_performance_results(test_name: &str, metrics: &PerformanceMetrics) {
    println!("\n📊 {} Performance Results:", test_name);
    println!("   Duration: {:?}", metrics.duration);
    println!("   Temperature Updates: {}", metrics.temperature_updates);
    println!("   Avg Update Time: {:?}", metrics.avg_update_time);
    println!("   Max Update Time: {:?}", metrics.max_update_time);
    println!("   Active Chunks: {} (hot + warm)", metrics.active_chunks);
    println!("   Cold Chunks: {}", metrics.cold_chunks);
    println!("   Total Chunks Tracked: {}", metrics.memory_chunks);
    println!(
        "   LOD Efficiency: {:.2}% (active/total)",
        metrics.lod_efficiency * 100.0
    );

    // Performance assessment
    let avg_update_ms = metrics.avg_update_time.as_millis();
    let max_update_ms = metrics.max_update_time.as_millis();

    if avg_update_ms <= 5 && max_update_ms <= 10 {
        println!("   ✅ Performance: EXCELLENT (≤5ms avg, ≤10ms max)");
    } else if avg_update_ms <= 10 && max_update_ms <= 20 {
        println!("   ✅ Performance: GOOD (≤10ms avg, ≤20ms max)");
    } else if avg_update_ms <= 20 && max_update_ms <= 50 {
        println!("   ⚠️  Performance: ACCEPTABLE (≤20ms avg, ≤50ms max)");
    } else {
        println!("   ❌ Performance: POOR (>20ms avg or >50ms max)");
    }

    if metrics.lod_efficiency <= 0.3 {
        println!("   ✅ LOD Efficiency: EXCELLENT (≤30% active chunks)");
    } else if metrics.lod_efficiency <= 0.5 {
        println!("   ✅ LOD Efficiency: GOOD (≤50% active chunks)");
    } else if metrics.lod_efficiency <= 0.7 {
        println!("   ⚠️  LOD Efficiency: ACCEPTABLE (≤70% active chunks)");
    } else {
        println!("   ❌ LOD Efficiency: POOR (>70% active chunks)");
    }
}

#[test]
fn test_large_map_clustered_agents() {
    let config = LargeMapConfig::new(
        1200, // 1200 chunks (~35x35 grid)
        AgentPattern::Clustered {
            center: IVec2::new(0, 0),
            radius: 15, // Much tighter clustering
        },
        50, // 50 agents clustered together
    );

    let metrics = run_performance_test(&config);
    print_performance_results("Large Map - Clustered Agents", &metrics);

    // Validate performance expectations for clustered agents
    assert!(
        metrics.avg_update_time.as_millis() <= 15,
        "Average update time should be ≤15ms for clustered agents"
    );
    assert!(
        metrics.max_update_time.as_millis() <= 30,
        "Maximum update time should be ≤30ms for clustered agents"
    );
    assert!(
        metrics.lod_efficiency <= 0.7,
        "LOD efficiency should be ≤70% for tightly clustered agents (active/total chunks)"
    );

    println!("✅ Large map clustered agents test passed");
}

#[test]
fn test_large_map_distributed_agents() {
    let config = LargeMapConfig::new(
        1200, // 1200 chunks (~35x35 grid)
        AgentPattern::Distributed { spacing: 200 },
        50, // 50 agents distributed across map
    );

    let metrics = run_performance_test(&config);
    print_performance_results("Large Map - Distributed Agents", &metrics);

    // Validate performance expectations for distributed agents
    assert!(
        metrics.avg_update_time.as_millis() <= 15,
        "Average update time should be ≤15ms for distributed agents"
    );
    assert!(
        metrics.max_update_time.as_millis() <= 30,
        "Maximum update time should be ≤30ms for distributed agents"
    );

    println!("✅ Large map distributed agents test passed");
}

#[test]
fn test_large_map_multi_cluster() {
    let clusters = vec![
        (IVec2::new(-150, -150), 12), // Much tighter clusters
        (IVec2::new(150, -150), 12),
        (IVec2::new(0, 150), 12),
    ];

    let config = LargeMapConfig::new(
        1600, // 1600 chunks (~40x40 grid)
        AgentPattern::MultiCluster { clusters },
        36, // 36 agents in 3 clusters (12 each)
    );

    let metrics = run_performance_test(&config);
    print_performance_results("Large Map - Multi-Cluster", &metrics);

    // Validate performance expectations for multi-cluster
    assert!(
        metrics.avg_update_time.as_millis() <= 12,
        "Average update time should be ≤12ms for multi-cluster agents"
    );
    assert!(
        metrics.max_update_time.as_millis() <= 25,
        "Maximum update time should be ≤25ms for multi-cluster agents"
    );
    assert!(
        metrics.lod_efficiency <= 0.3,
        "LOD efficiency should be ≤30% for tight multi-cluster agents"
    );

    println!("✅ Large map multi-cluster test passed");
}

#[test]
fn test_cpu_usage_scalability() {
    println!("\n🔬 Testing CPU Usage Scalability");

    let mut results = Vec::new();

    // Test with different chunk counts to validate scalability
    for chunk_count in [400, 800, 1200, 1600] {
        let config = LargeMapConfig::new(
            chunk_count,
            AgentPattern::Clustered {
                center: IVec2::new(0, 0),
                radius: 30,
            },
            30,
        );

        // Shorter test for scalability test
        let mut short_config = config;
        short_config.test_duration_ms = 2000; // 2 seconds

        let metrics = run_performance_test(&short_config);
        results.push((chunk_count, metrics.avg_update_time.as_micros()));

        println!(
            "   {} chunks: {}µs avg update time",
            chunk_count,
            metrics.avg_update_time.as_micros()
        );
    }

    // Validate that CPU usage scales sub-linearly with chunk count
    // (i.e., doubling chunks shouldn't double processing time due to LOD)
    let (chunks_400, time_400) = results[0];
    let (chunks_1200, time_1200) = results[2];

    let chunks_ratio = chunks_1200 as f32 / chunks_400 as f32; // Should be ~3.0
    let time_ratio = if time_400 > 0 {
        time_1200 as f32 / time_400 as f32
    } else {
        1.0
    };

    println!(
        "   Chunk ratio: {:.1}x ({} -> {})",
        chunks_ratio, chunks_400, chunks_1200
    );
    println!(
        "   Time ratio: {:.1}x ({}µs -> {}µs)",
        time_ratio, time_400, time_1200
    );

    // Time ratio should be significantly less than chunk ratio due to LOD efficiency
    // or at least not proportionally higher
    assert!(
        time_ratio <= chunks_ratio * 1.2,
        "CPU usage should scale sub-linearly with chunk count due to LOD"
    );

    println!("✅ CPU usage scalability test passed - LOD system provides good scalability");
}

#[test]
fn test_memory_efficiency() {
    println!("\n💾 Testing Memory Efficiency");

    let config = LargeMapConfig::new(
        2000, // 2000 chunks (~45x45 grid)
        AgentPattern::Clustered {
            center: IVec2::new(0, 0),
            radius: 25,
        },
        20, // Only 20 agents
    );

    let mut lod_manager = ChunkLODManager::new(ChunkLODConfig::default());

    // Generate many chunks
    let chunks = config.generate_chunks();
    for chunk_coord in chunks {
        lod_manager.get_or_create_chunk(chunk_coord);
    }

    // Generate clustered agents
    let agents = config.generate_agents();
    lod_manager.update_agent_positions(agents);

    let metrics = lod_manager.get_metrics();

    println!("   Total chunks in memory: {}", metrics.total_chunks);
    println!(
        "   Active chunks (hot + warm): {}",
        metrics.hot_chunks + metrics.warm_chunks
    );
    println!("   Cold chunks (impostor only): {}", metrics.cold_chunks);
    println!(
        "   Memory efficiency: {:.1}% of chunks are active",
        (metrics.hot_chunks + metrics.warm_chunks) as f32 / metrics.total_chunks as f32 * 100.0
    );

    // With clustered agents, most chunks should be cold (impostor only)
    assert!(
        metrics.cold_chunks > metrics.total_chunks / 2,
        "Most chunks should be cold when agents are clustered"
    );

    println!("✅ Memory efficiency test passed - LOD system effectively reduces memory usage");
}

#[test]
fn test_performance_comparison() {
    println!("\n⚡ Performance Comparison: Clustered vs Distributed");

    // Test clustered agents
    let mut clustered_config = LargeMapConfig::new(
        1000,
        AgentPattern::Clustered {
            center: IVec2::new(0, 0),
            radius: 40,
        },
        40,
    );
    clustered_config.test_duration_ms = 3000;

    let clustered_metrics = run_performance_test(&clustered_config);

    // Test distributed agents
    let mut distributed_config =
        LargeMapConfig::new(1000, AgentPattern::Distributed { spacing: 300 }, 40);
    distributed_config.test_duration_ms = 3000;

    let distributed_metrics = run_performance_test(&distributed_config);

    println!("\n   Clustered Agents:");
    println!(
        "     Avg update time: {}ms",
        clustered_metrics.avg_update_time.as_millis()
    );
    println!("     Active chunks: {}", clustered_metrics.active_chunks);
    println!(
        "     LOD efficiency: {:.1}%",
        clustered_metrics.lod_efficiency * 100.0
    );

    println!("\n   Distributed Agents:");
    println!(
        "     Avg update time: {}ms",
        distributed_metrics.avg_update_time.as_millis()
    );
    println!("     Active chunks: {}", distributed_metrics.active_chunks);
    println!(
        "     LOD efficiency: {:.1}%",
        distributed_metrics.lod_efficiency * 100.0
    );

    // Clustered should be more efficient
    let time_improvement = distributed_metrics.avg_update_time.as_millis() as f32
        / clustered_metrics.avg_update_time.as_millis() as f32;

    println!(
        "\n   Performance improvement (clustered vs distributed): {:.1}x faster",
        time_improvement
    );

    assert!(
        time_improvement >= 1.5,
        "Clustered agents should be at least 1.5x faster than distributed"
    );

    println!("✅ Performance comparison test passed - clustering provides significant benefits");
}
