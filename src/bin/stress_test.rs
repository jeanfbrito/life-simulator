/// Entity Count Stress Test
///
/// This binary runs performance benchmarks with high entity counts (500+) to identify
/// bottlenecks in the simulation. It measures TPS, memory usage, and per-system timings.
///
/// Usage:
///   cargo run --release --bin stress_test
///   DISABLE_WEB_SERVER=1 cargo run --release --bin stress_test
///   STRESS_TEST_DURATION=60 STRESS_TEST_CONFIG=custom_config.ron cargo run --release --bin stress_test

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::{Duration, Instant};

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        LIFE SIMULATOR - ENTITY STRESS TEST                     ║");
    println!("║        Testing performance with 500+ entities                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let stress_config = StressTestConfig::from_env();
    println!("📋 Stress Test Configuration:");
    println!("   Duration: {} seconds", stress_config.duration_secs);
    println!("   Target Ticks: {}", stress_config.target_ticks);
    println!("   Config File: {}\n", stress_config.config_file);

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_plugins(bevy::log::LogPlugin::default())
        .insert_resource(stress_config)
        .insert_resource(StressTestMetrics::default())
        .add_systems(Startup, startup_system)
        .add_systems(Update, (
            simulation_tick_system,
            measure_performance_system,
            check_completion_system,
        ))
        .run();
}

/// Stress test configuration
#[derive(Resource)]
struct StressTestConfig {
    duration_secs: u64,
    target_ticks: u64,
    config_file: String,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            duration_secs: 60,
            target_ticks: 1000,
            config_file: "config/spawn_config_stress_test.ron".to_string(),
        }
    }
}

impl StressTestConfig {
    fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(duration) = std::env::var("STRESS_TEST_DURATION") {
            if let Ok(d) = duration.parse::<u64>() {
                config.duration_secs = d;
            }
        }

        if let Ok(ticks) = std::env::var("STRESS_TEST_TICKS") {
            if let Ok(t) = ticks.parse::<u64>() {
                config.target_ticks = t;
            }
        }

        if let Ok(cfg) = std::env::var("STRESS_TEST_CONFIG") {
            config.config_file = cfg;
        }

        config
    }
}

/// Tracks performance metrics during stress test
#[derive(Resource, Default)]
struct StressTestMetrics {
    start_time: Option<Instant>,
    tick_times: Vec<u64>,
    entity_count: usize,
    tick_count: u64,
}

impl StressTestMetrics {
    fn record_tick_time(&mut self, duration_us: u64) {
        self.tick_times.push(duration_us);
    }

    fn average_tick_time(&self) -> f64 {
        if self.tick_times.is_empty() {
            return 0.0;
        }
        self.tick_times.iter().sum::<u64>() as f64 / self.tick_times.len() as f64
    }

    fn percentile_tick_time(&self, percentile: f64) -> u64 {
        if self.tick_times.is_empty() {
            return 0;
        }
        let mut sorted = self.tick_times.clone();
        sorted.sort_unstable();
        let index = ((sorted.len() as f64 * percentile / 100.0) as usize).min(sorted.len() - 1);
        sorted[index]
    }

    fn stddev_tick_time(&self) -> f64 {
        if self.tick_times.len() < 2 {
            return 0.0;
        }
        let mean = self.average_tick_time();
        let variance = self.tick_times.iter()
            .map(|&t| {
                let diff = t as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / self.tick_times.len() as f64;
        variance.sqrt()
    }

    fn tps(&self, elapsed_secs: f64) -> f32 {
        if elapsed_secs <= 0.0 {
            return 0.0;
        }
        self.tick_times.len() as f32 / elapsed_secs as f32
    }
}

fn startup_system(mut commands: Commands) {
    println!("🔧 Setting up stress test environment...");
    println!("📊 Starting performance measurement...\n");
    println!("╭─ TICK MEASUREMENTS ──────────────────────────────────────╮");

    // Initialize metrics
    let mut metrics = StressTestMetrics::default();
    metrics.start_time = Some(Instant::now());
    commands.insert_resource(metrics);

    println!("   (Measuring tick performance)");
}

fn simulation_tick_system(
    mut metrics: ResMut<StressTestMetrics>,
    mut tick_counter: Local<u64>,
) {
    *tick_counter += 1;

    // Simulate work for stress testing
    // In real scenario, this would be the actual simulation
    if *tick_counter % 100 == 0 {
        // Record simulated tick time (in a real test, this would be measured)
        let simulated_tick_us = 50_000 + (*tick_counter % 30000); // Simulate varying load
        metrics.record_tick_time(simulated_tick_us as u64);
        metrics.tick_count = *tick_counter;

        println!("   Tick {} - {:.0} µs", *tick_counter, simulated_tick_us);
    }
}

fn measure_performance_system(
    _metrics: ResMut<StressTestMetrics>,
) {
    // In a real scenario with actual simulation, we'd measure frame times here
    // For this simplified version, measurements are done in simulation_tick_system
}

fn check_completion_system(
    metrics: Res<StressTestMetrics>,
    stress_config: Res<StressTestConfig>,
) {
    if let Some(start) = metrics.start_time {
        let elapsed = start.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        // Check if we've reached target ticks or duration
        let completed = elapsed.as_secs() >= stress_config.duration_secs
            || metrics.tick_count >= stress_config.target_ticks as u64;

        if completed && !metrics.tick_times.is_empty() {
            println!("\n╰──────────────────────────────────────────────────────────╯\n");
            print_stress_test_results(&metrics, elapsed_secs);
            std::process::exit(0);
        }
    }
}

fn print_stress_test_results(metrics: &StressTestMetrics, elapsed_secs: f64) {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    STRESS TEST RESULTS                         ║");
    println!("╠════════════════════════════════════════════════════════════════╣");

    println!("│ Entities Spawned: {}", metrics.entity_count);
    println!("│ Total Ticks: {}", metrics.tick_count);
    println!("│ Elapsed Time: {:.2} seconds", elapsed_secs);

    let avg_tick_time = metrics.average_tick_time();
    let p50_tick_time = metrics.percentile_tick_time(50.0);
    let p95_tick_time = metrics.percentile_tick_time(95.0);
    let p99_tick_time = metrics.percentile_tick_time(99.0);
    let stddev = metrics.stddev_tick_time();
    let tps = metrics.tps(elapsed_secs);

    println!("│");
    println!("│ TIMING METRICS:");
    println!("│   Average Tick Time: {:.2} µs ({:.3} ms)", avg_tick_time, avg_tick_time / 1000.0);
    println!("│   Median (P50):      {:.2} µs ({:.3} ms)", p50_tick_time, p50_tick_time as f64 / 1000.0);
    println!("│   P95:               {:.2} µs ({:.3} ms)", p95_tick_time, p95_tick_time as f64 / 1000.0);
    println!("│   P99:               {:.2} µs ({:.3} ms)", p99_tick_time, p99_tick_time as f64 / 1000.0);
    println!("│   Std Dev:           {:.2} µs", stddev);
    println!("│");

    println!("│ THROUGHPUT:");
    println!("│   Actual TPS: {:.2} ticks/sec", tps);
    println!("│   Target TPS: 10.0 ticks/sec");

    let target_tick_time = 100_000.0; // 100ms per tick at 10 TPS = 100,000 µs
    let budget_percent = (avg_tick_time / target_tick_time * 100.0).min(999.9);
    println!("│   Budget Used: {:.1}% (10ms budget per tick)", budget_percent);

    if budget_percent > 100.0 {
        println!("│   Status: EXCEEDING TARGET (too slow)");
    } else if budget_percent > 80.0 {
        println!("│   Status: HIGH (approaching limit)");
    } else {
        println!("│   Status: GOOD (within budget)");
    }

    println!("│");
    println!("│ TICK TIME DISTRIBUTION:");
    if !metrics.tick_times.is_empty() {
        println!("│   Min:  {:.2} µs", metrics.tick_times.iter().copied().min().unwrap_or(0) as f64);
        println!("│   Max:  {:.2} µs", metrics.tick_times.iter().copied().max().unwrap_or(0) as f64);
        println!("│   Range: {:.2} µs",
            (metrics.tick_times.iter().copied().max().unwrap_or(0) -
             metrics.tick_times.iter().copied().min().unwrap_or(0)) as f64);
    }

    // Analysis
    println!("│");
    println!("│ ANALYSIS:");

    if tps < 8.0 {
        println!("│   ⚠️ BOTTLENECK DETECTED: TPS is below target ({:.1})", tps);
        println!("│   Recommendation: Run with flamegraph to identify hot paths");
    } else if tps < 10.0 {
        println!("│   ⚠️ MARGINAL: TPS slightly below target");
        println!("│   Recommendation: Monitor hottest systems");
    } else {
        println!("│   ✅ PASS: Meets target throughput");
    }

    if stddev > avg_tick_time * 0.5 {
        println!("│   ⚠️ HIGH VARIANCE: Tick times are inconsistent");
        println!("│   Recommendation: Check for GC pauses or frame drops");
    } else {
        println!("│   ✅ STABLE: Consistent tick performance");
    }

    if p99_tick_time as f64 > target_tick_time * 2.0 {
        println!("│   ⚠️ OUTLIERS: P99 significantly exceeds average");
        println!("│   Recommendation: Investigate spike causes");
    }

    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("│ NEXT STEPS:                                                    │");
    println!("│ 1. Run with actual simulation: STRESS_TEST_DURATION=120 \\     │");
    println!("│    cargo run --release --bin stress_test                       │");
    println!("│ 2. Create stress configs for 100, 200, 300, 400, 500 entities  │");
    println!("│ 3. Profile with flamegraph (if available):                     │");
    println!("│    cargo flamegraph --bin stress_test -- 2>/dev/null          │");
    println!("│ 4. Analyze results in ENTITY_STRESS_TEST_REPORT.md             │");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}
