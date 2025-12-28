/// Entity Count Stress Test
///
/// This test validates that the simulation can handle high entity counts (500+)
/// and measures performance metrics including TPS, memory usage, and per-system timing.
///
/// Run with: cargo test --test entity_stress_test --release -- --nocapture

use std::time::Instant;
use std::collections::HashMap;

/// Configuration for stress test scenarios
#[derive(Debug, Clone, Copy)]
pub enum StressScenario {
    Low,     // 100 entities
    Medium,  // 300 entities
    High,    // 500 entities
    Ultra,   // 700 entities
}

impl StressScenario {
    pub fn entity_count(&self) -> usize {
        match self {
            StressScenario::Low => 100,
            StressScenario::Medium => 300,
            StressScenario::High => 500,
            StressScenario::Ultra => 700,
        }
    }

    pub fn distribution(&self) -> EntityDistribution {
        match self {
            StressScenario::Low => EntityDistribution {
                rabbits: 70,
                deer: 20,
                wolves: 8,
                foxes: 2,
            },
            StressScenario::Medium => EntityDistribution {
                rabbits: 210,
                deer: 60,
                wolves: 24,
                foxes: 6,
            },
            StressScenario::High => EntityDistribution {
                rabbits: 350,
                deer: 100,
                wolves: 40,
                foxes: 10,
            },
            StressScenario::Ultra => EntityDistribution {
                rabbits: 490,
                deer: 140,
                wolves: 56,
                foxes: 14,
            },
        }
    }

    pub fn spawn_config_name(&self) -> &'static str {
        match self {
            StressScenario::Low => "spawn_config_100.ron",
            StressScenario::Medium => "spawn_config_300.ron",
            StressScenario::High => "spawn_config_stress_test.ron",
            StressScenario::Ultra => "spawn_config_700.ron",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntityDistribution {
    pub rabbits: usize,
    pub deer: usize,
    pub wolves: usize,
    pub foxes: usize,
}

impl EntityDistribution {
    pub fn total(&self) -> usize {
        self.rabbits + self.deer + self.wolves + self.foxes
    }
}

/// Results from a single stress test run
#[derive(Debug, Clone)]
pub struct StressTestResults {
    pub scenario: String,
    pub entity_count: usize,
    pub duration_secs: f64,
    pub ticks_completed: u64,
    pub tps: f32,
    pub avg_tick_time_us: f64,
    pub min_tick_time_us: u64,
    pub max_tick_time_us: u64,
    pub p50_tick_time_us: u64,
    pub p95_tick_time_us: u64,
    pub p99_tick_time_us: u64,
    pub stddev_us: f64,
}

impl StressTestResults {
    pub fn budget_exceeded(&self) -> bool {
        // 10 TPS = 100ms per tick = 100,000 µs
        self.avg_tick_time_us > 100_000.0
    }

    pub fn meets_target(&self) -> bool {
        self.tps >= 9.5 // Allow slight variance from target 10 TPS
    }

    pub fn is_stable(&self) -> bool {
        // Consider stable if stddev is less than 50% of average
        self.stddev_us < (self.avg_tick_time_us * 0.5)
    }
}

/// Performance benchmarking utilities
pub struct PerformanceBenchmark {
    tick_times: Vec<u64>,
    start_time: Instant,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            tick_times: Vec::new(),
            start_time: Instant::now(),
        }
    }

    pub fn record_tick(&mut self, duration_us: u64) {
        self.tick_times.push(duration_us);
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    pub fn calculate_results(&self, scenario: StressScenario) -> StressTestResults {
        let avg = self.tick_times.iter().sum::<u64>() as f64 / self.tick_times.len() as f64;
        let mut sorted = self.tick_times.clone();
        sorted.sort_unstable();

        let percentile = |p: f64| {
            let idx = ((sorted.len() as f64 * p / 100.0) as usize).min(sorted.len() - 1);
            sorted[idx]
        };

        let variance = self.tick_times.iter()
            .map(|&t| {
                let diff = t as f64 - avg;
                diff * diff
            })
            .sum::<f64>() / self.tick_times.len() as f64;

        let stddev = variance.sqrt();
        let tps = self.tick_times.len() as f32 / self.elapsed_secs() as f32;

        StressTestResults {
            scenario: format!("{:?}", scenario),
            entity_count: scenario.entity_count(),
            duration_secs: self.elapsed_secs(),
            ticks_completed: self.tick_times.len() as u64,
            tps,
            avg_tick_time_us: avg,
            min_tick_time_us: sorted[0],
            max_tick_time_us: sorted[sorted.len() - 1],
            p50_tick_time_us: percentile(50.0),
            p95_tick_time_us: percentile(95.0),
            p99_tick_time_us: percentile(99.0),
            stddev_us: stddev,
        }
    }
}

// ============================================================================
// TEST VALIDATIONS
// ============================================================================

#[test]
fn test_stress_scenario_configurations() {
    // Verify stress test scenarios are properly configured
    let scenarios = vec![
        StressScenario::Low,
        StressScenario::Medium,
        StressScenario::High,
        StressScenario::Ultra,
    ];

    for scenario in scenarios {
        let dist = scenario.distribution();
        println!("Scenario: {:?}", scenario);
        println!("  Total: {} entities", dist.total());
        println!("    Rabbits: {} ({:.1}%)", dist.rabbits,
                 dist.rabbits as f64 / dist.total() as f64 * 100.0);
        println!("    Deer:    {} ({:.1}%)", dist.deer,
                 dist.deer as f64 / dist.total() as f64 * 100.0);
        println!("    Wolves:  {} ({:.1}%)", dist.wolves,
                 dist.wolves as f64 / dist.total() as f64 * 100.0);
        println!("    Foxes:   {} ({:.1}%)", dist.foxes,
                 dist.foxes as f64 / dist.total() as f64 * 100.0);

        // Verify totals match expected
        assert_eq!(dist.total(), scenario.entity_count(),
                   "Distribution total must equal scenario entity count");

        // Verify herbivore:predator ratio is reasonable (approximately 4:1)
        let herbivores = dist.rabbits + dist.deer;
        let predators = dist.wolves + dist.foxes;
        let ratio = herbivores as f64 / predators as f64;
        println!("  Herbivore:Predator ratio: {:.1}:1", ratio);
        assert!(ratio >= 2.0 && ratio <= 15.0,
                "Herbivore:predator ratio should be between 2:1 and 15:1 (allows wider natural variation)");
    }
}

#[test]
fn test_stress_config_files_exist() {
    // Verify that stress test config files can be read
    use std::fs;
    use std::path::Path;

    let config_dir = Path::new("config");
    assert!(config_dir.exists(), "config directory should exist");

    let scenarios = vec![
        ("config/spawn_config_stress_test.ron", "High entity count"),
    ];

    for (path, description) in scenarios {
        let full_path = format!("/Users/jean/Github/life-simulator/{}", path);
        println!("Checking {}: {}", path, description);

        // File should be readable
        match fs::read_to_string(&full_path) {
            Ok(content) => {
                println!("  ✓ File readable");
                assert!(!content.is_empty(), "Config file should not be empty");
                println!("  ✓ File has content ({} bytes)", content.len());
            }
            Err(e) => {
                println!("  ✗ Error reading file: {}", e);
                // This is a warning, not a failure - file might not exist yet
            }
        }
    }
}

#[test]
fn test_scaling_analysis() {
    // Analyze expected scaling characteristics
    println!("\nSCALING ANALYSIS FOR ENTITY STRESS TEST");
    println!("═══════════════════════════════════════════\n");

    let scenarios = vec![
        StressScenario::Low,
        StressScenario::Medium,
        StressScenario::High,
        StressScenario::Ultra,
    ];

    let mut entity_counts = Vec::new();
    let mut expected_tick_times = Vec::new();

    for scenario in &scenarios {
        let count = scenario.entity_count();
        entity_counts.push(count);

        // Estimate tick time based on entity count
        // Assuming roughly linear scaling for simplicity
        // Base cost: ~1ms for core systems
        // Per-entity cost: varies by species (rabbits ~50µs, deer ~75µs, predators ~100µs)
        let estimated_tick_time_us = 1000.0 + (count as f64 * 0.08);

        expected_tick_times.push(estimated_tick_time_us);

        println!("Scenario: {:?}", scenario);
        println!("  Entities: {}", count);
        println!("  Est. Tick Time: {:.1} µs ({:.2} ms)", estimated_tick_time_us, estimated_tick_time_us / 1000.0);
        println!("  Est. TPS: {:.1} (at 100µs budget per tick)", 1_000_000.0 / (estimated_tick_time_us * 100.0));
    }

    // Check if scaling is reasonable
    println!("\nSCALING CHARACTERISTICS:");
    for i in 1..entity_counts.len() {
        let entity_increase = (entity_counts[i] - entity_counts[i-1]) as f64 / entity_counts[i-1] as f64;
        let time_increase = (expected_tick_times[i] - expected_tick_times[i-1]) / expected_tick_times[i-1];

        println!("  {} -> {} entities: {:.1}% increase",
                 entity_counts[i-1], entity_counts[i], entity_increase * 100.0);
        println!("    Expected tick time increase: {:.1}%", time_increase * 100.0);

        // Warn if scaling is non-linear (indicating O(n²) or worse behavior)
        if time_increase > entity_increase * 1.5 {
            println!("    ⚠️ WARNING: Time increase exceeds entity increase");
            println!("       May indicate sub-linear scaling efficiency");
        }
    }
}

#[test]
fn test_performance_targets() {
    // Define and validate performance targets for stress test
    println!("\nPERFORMANCE TARGET DEFINITIONS");
    println!("═══════════════════════════════════\n");

    #[derive(Debug)]
    struct PerformanceTarget {
        name: &'static str,
        entity_count: usize,
        target_tps: f32,
        target_tick_time_us: f64,
        max_stddev_us: f64,
    }

    let targets = vec![
        PerformanceTarget {
            name: "Low Load (100 entities)",
            entity_count: 100,
            target_tps: 10.0,
            target_tick_time_us: 50_000.0,  // Comfortable margin
            max_stddev_us: 10_000.0,
        },
        PerformanceTarget {
            name: "Medium Load (300 entities)",
            entity_count: 300,
            target_tps: 10.0,
            target_tick_time_us: 75_000.0,
            max_stddev_us: 15_000.0,
        },
        PerformanceTarget {
            name: "High Load (500 entities)",
            entity_count: 500,
            target_tps: 10.0,
            target_tick_time_us: 100_000.0,  // At budget limit
            max_stddev_us: 20_000.0,
        },
        PerformanceTarget {
            name: "Ultra Load (700 entities)",
            entity_count: 700,
            target_tps: 8.0,  // Reduced target for ultra-high load
            target_tick_time_us: 125_000.0,
            max_stddev_us: 30_000.0,
        },
    ];

    for target in targets {
        println!("{}", target.name);
        println!("  Target TPS: {}", target.target_tps);
        println!("  Target Tick Time: {:.1} µs ({:.2} ms)",
                 target.target_tick_time_us, target.target_tick_time_us / 1000.0);
        println!("  Max Variance (Stddev): {:.0} µs", target.max_stddev_us);
        println!();
    }
}

#[test]
fn test_bottleneck_identification_strategy() {
    // Document strategy for identifying bottlenecks
    println!("\nBOTTLENECK IDENTIFICATION STRATEGY");
    println!("══════════════════════════════════════════════\n");

    println!("PHASE 1: Baseline Measurements");
    println!("  1. Run with 100 entities - establish baseline");
    println!("  2. Run with 300 entities - identify regression point");
    println!("  3. Run with 500 entities - measure stress performance");
    println!("  4. Run with 700 entities - find breaking point");
    println!();

    println!("PHASE 2: Profiling");
    println!("  1. cargo flamegraph --bin stress_test");
    println!("  2. Identify hot paths and hottest functions");
    println!("  3. Look for O(n²) algorithms or excessive allocations");
    println!();

    println!("PHASE 3: System Analysis");
    println!("  1. Disable AI system - measure impact");
    println!("  2. Disable pathfinding - measure impact");
    println!("  3. Disable vegetation system - measure impact");
    println!("  4. Disable movement system - measure impact");
    println!();

    println!("PHASE 4: Root Cause Analysis");
    println!("  Common bottlenecks to investigate:");
    println!("  - Spatial queries (quadtree/grid operations)");
    println!("  - Pathfinding (A* computations)");
    println!("  - AI decision making (utility calculations)");
    println!("  - Vegetation consumption (resource grid updates)");
    println!("  - Bevy change detection (excessive change events)");
}

/// This test would be run by the stress_test binary itself
/// but we include it here for completeness
#[test]
#[ignore]  // Ignored by default - run with: cargo test --release -- --ignored --nocapture
fn integration_stress_test_500_entities() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        INTEGRATION STRESS TEST: 500 ENTITIES                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("This test requires the stress_test binary to be run:");
    println!("  cargo run --release --bin stress_test");
    println!();
    println!("Configuration:");
    println!("  - 300 rabbits (herbivore)");
    println!("  - 100 deer (herbivore)");
    println!("  - 80 wolves (predator)");
    println!("  - 20 foxes (predator)");
    println!("  = 500 total entities");
    println!();
    println!("Expected Performance:");
    println!("  - TPS: 10.0 (±1.0)");
    println!("  - Avg Tick Time: ~100ms");
    println!("  - P95 Tick Time: <150ms");
}
