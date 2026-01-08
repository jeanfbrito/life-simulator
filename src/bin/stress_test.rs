#![allow(unused_imports, deprecated, static_mut_refs)]
/// Entity Count Stress Test
///
/// This binary runs performance benchmarks with high entity counts (100-700+) to identify
/// bottlenecks in the simulation. It measures TPS, memory usage, and per-tick timings.
///
/// Usage:
///   cargo run --release --bin stress_test
///   DISABLE_WEB_SERVER=1 cargo run --release --bin stress_test
///   STRESS_TEST_DURATION=60 STRESS_TEST_CONFIG=config/spawn_config_stress_100.ron cargo run --release --bin stress_test

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::{Duration, Instant};

// Import life simulator components
use life_simulator::ai::TQUAIPlugin;
use life_simulator::cached_world::CachedWorldPlugin;
use life_simulator::debug::{HealthCheckPlugin, HealthCheckApiPlugin};
use life_simulator::entities::EntitiesPlugin;
use life_simulator::pathfinding::{pathfinding_cache_cleanup_system, process_pathfinding_requests, PathCache, PathfindingGrid};
use life_simulator::simulation::SimulationPlugin;
use life_simulator::tilemap::{TilemapPlugin, WorldConfig, TerrainType};
use life_simulator::vegetation::VegetationPlugin;
use life_simulator::world_loader::WorldLoader;

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        LIFE SIMULATOR - ENTITY STRESS TEST                     ║");
    println!("║        Testing performance with configurable entity counts     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let stress_config = StressTestConfig::from_env();
    println!("📋 Stress Test Configuration:");
    println!("   Duration: {} seconds", stress_config.duration_secs);
    println!("   Target Ticks: {}", stress_config.target_ticks);
    println!("   Config File: {}\n", stress_config.config_file);

    // Set environment variable for spawn config
    std::env::set_var("SPAWN_CONFIG", &stress_config.config_file);
    std::env::set_var("DISABLE_WEB_SERVER", "1"); // Always disable web server for stress tests

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0, // 60 FPS target
            ))),
        )
        .add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::WARN, // Reduce log noise during stress test
            filter: "life_simulator=warn,bevy=warn".to_string(),
            ..default()
        })
        .add_plugins(CachedWorldPlugin)
        .add_plugins((
            SimulationPlugin,
            EntitiesPlugin,
            TQUAIPlugin,
            VegetationPlugin,
            // Skip health check plugins to avoid system conflicts during stress testing
        ))
        .insert_resource(WorldConfig::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<PathfindingGrid>()
        .init_resource::<PathCache>()
        .insert_resource(stress_config)
        .insert_resource(StressTestMetrics::default())
        .add_systems(
            Startup,
            (setup_stress_test, life_simulator::entities::spawn_entities_from_config.after(setup_stress_test)),
        )
        .add_systems(
            Update,
            (
                process_pathfinding_requests,
                pathfinding_cache_cleanup_system,
                measure_tick_performance_system,
                check_completion_system,
            )
                .run_if(resource_exists::<WorldLoader>),
        )
        .run();
}

/// Stress test configuration
#[derive(Resource, Clone)]
struct StressTestConfig {
    duration_secs: u64,
    target_ticks: u64,
    config_file: String,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            duration_secs: 60,
            target_ticks: 600, // 10 TPS * 60 seconds
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
                config.target_ticks = d * 10; // Assume 10 TPS target
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
    tick_times: Vec<u64>, // Microseconds
    entity_count: usize,
    tick_count: u64,
    last_tick_time: Option<Instant>,
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
        self.tick_count as f32 / elapsed_secs as f32
    }
}

fn setup_stress_test(mut commands: Commands, mut pathfinding_grid: ResMut<PathfindingGrid>) {
    println!("🔧 Setting up stress test environment...");

    // Load the world
    let requested_map_name =
        std::env::var("WORLD_MAP_NAME").unwrap_or_else(|_| "slopes_demo".to_string());

    let world_loader = match WorldLoader::load_by_name(&requested_map_name) {
        Ok(loader) => {
            println!("✅ World loaded: {} (seed: {})", loader.get_name(), loader.get_seed());
            loader
        }
        Err(_) => {
            match WorldLoader::load_default() {
                Ok(loader) => {
                    println!("✅ World loaded: {} (seed: {})", loader.get_name(), loader.get_seed());
                    loader
                }
                Err(e) => {
                    eprintln!("❌ Failed to load world: {}", e);
                    eprintln!("💡 Please generate a world first: cargo run --bin map_generator");
                    std::process::exit(1);
                }
            }
        }
    };

    // Build pathfinding grid
    println!("🧭 Building pathfinding grid...");
    let ((min_x, min_y), (max_x, max_y)) = world_loader.get_world_bounds();
    let tile_min_x = min_x * 16 - 16;
    let tile_min_y = min_y * 16 - 16;
    let tile_max_x = (max_x + 1) * 16 + 16;
    let tile_max_y = (max_y + 1) * 16 + 16;

    for y in tile_min_y..=tile_max_y {
        for x in tile_min_x..=tile_max_x {
            let pos = bevy::math::IVec2::new(x, y);
            let terrain_str = world_loader.get_terrain_at(x, y);
            let terrain_cost = if let Some(terrain_str) = terrain_str {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    let cost = terrain.movement_cost();
                    if cost >= 1000.0 { u32::MAX } else { cost as u32 }
                } else {
                    u32::MAX
                }
            } else {
                u32::MAX
            };

            let has_resource = world_loader
                .get_resource_at(x, y)
                .map(|r| !r.is_empty())
                .unwrap_or(false);

            let final_cost = if has_resource && terrain_cost != u32::MAX {
                u32::MAX
            } else {
                terrain_cost
            };

            pathfinding_grid.set_cost(pos, final_cost);
        }
    }

    println!("✅ Pathfinding grid ready");
    println!("📊 Starting performance measurement...\n");

    // Initialize metrics
    let mut metrics = StressTestMetrics::default();
    metrics.start_time = Some(Instant::now());
    metrics.last_tick_time = Some(Instant::now());

    commands.insert_resource(world_loader);
    commands.insert_resource(metrics);
}

fn measure_tick_performance_system(
    mut metrics: ResMut<StressTestMetrics>,
    query: Query<Entity, With<life_simulator::entities::Creature>>,
) {
    let now = Instant::now();

    if let Some(last_tick) = metrics.last_tick_time {
        let tick_duration = now.duration_since(last_tick);
        let tick_us = tick_duration.as_micros() as u64;

        metrics.record_tick_time(tick_us);
        metrics.tick_count += 1;
        metrics.entity_count = query.iter().count();

        // Log every 60 ticks (roughly every second at 60 FPS)
        if metrics.tick_count % 60 == 0 {
            let avg = metrics.average_tick_time();
            println!(
                "Tick {} - {} entities - {:.2}ms avg ({:.2} TPS)",
                metrics.tick_count,
                metrics.entity_count,
                avg / 1000.0,
                60.0 / (avg / 1_000_000.0)
            );
        }
    }

    metrics.last_tick_time = Some(now);
}

fn check_completion_system(
    metrics: Res<StressTestMetrics>,
    stress_config: Res<StressTestConfig>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
) {
    if let Some(start) = metrics.start_time {
        let elapsed = start.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        // Check if we've reached target ticks or duration
        let completed = elapsed.as_secs() >= stress_config.duration_secs
            || metrics.tick_count >= stress_config.target_ticks;

        if completed && !metrics.tick_times.is_empty() {
            println!("\n");
            print_stress_test_results(&metrics, elapsed_secs, &stress_config);
            app_exit_events.write(bevy::app::AppExit::Success);
        }
    }
}

fn print_stress_test_results(metrics: &StressTestMetrics, elapsed_secs: f64, config: &StressTestConfig) {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    STRESS TEST RESULTS                         ║");
    println!("╠════════════════════════════════════════════════════════════════╣");

    println!("│ Test Configuration: {}", config.config_file);
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

    // Determine target based on entity count
    let (target_ms, target_tps) = match metrics.entity_count {
        0..=150 => (50.0, 10.0),   // 100 entities: 50ms per tick
        151..=350 => (75.0, 10.0),  // 300 entities: 75ms per tick
        351..=600 => (100.0, 10.0), // 500 entities: 100ms per tick
        _ => (150.0, 8.0),          // 700 entities: 150ms per tick (8 TPS)
    };

    let target_tick_us = target_ms * 1000.0;
    let budget_percent = (avg_tick_time / target_tick_us * 100.0).min(999.9);
    println!("│   Budget Target: {:.0}ms per tick ({:.1} TPS)", target_ms, target_tps);
    println!("│   Budget Used: {:.1}%", budget_percent);

    let status = if budget_percent > 100.0 {
        "⚠️ EXCEEDING TARGET (too slow)"
    } else if budget_percent > 80.0 {
        "⚡ HIGH (approaching limit)"
    } else {
        "✅ GOOD (within budget)"
    };
    println!("│   Status: {}", status);

    println!("│");
    println!("│ TICK TIME DISTRIBUTION:");
    if !metrics.tick_times.is_empty() {
        let min_tick = metrics.tick_times.iter().copied().min().unwrap_or(0);
        let max_tick = metrics.tick_times.iter().copied().max().unwrap_or(0);
        println!("│   Min:  {:.2} µs ({:.3} ms)", min_tick, min_tick as f64 / 1000.0);
        println!("│   Max:  {:.2} µs ({:.3} ms)", max_tick, max_tick as f64 / 1000.0);
        println!("│   Range: {:.2} µs ({:.3} ms)", max_tick - min_tick, (max_tick - min_tick) as f64 / 1000.0);
    }

    // Analysis
    println!("│");
    println!("│ ANALYSIS:");

    if tps < target_tps * 0.8 {
        println!("│   ⚠️ BOTTLENECK DETECTED: TPS is significantly below target ({:.1} vs {:.1})", tps, target_tps);
        println!("│   Recommendation: Run with flamegraph to identify hot paths");
    } else if tps < target_tps {
        println!("│   ⚡ MARGINAL: TPS slightly below target ({:.1} vs {:.1})", tps, target_tps);
        println!("│   Recommendation: Monitor hottest systems");
    } else {
        println!("│   ✅ PASS: Meets target throughput ({:.1} vs {:.1} TPS)", tps, target_tps);
    }

    if stddev > avg_tick_time * 0.5 {
        println!("│   ⚠️ HIGH VARIANCE: Tick times are inconsistent (σ={:.1}%)",
                 (stddev / avg_tick_time * 100.0));
        println!("│   Recommendation: Check for frame drops or scheduling issues");
    } else {
        println!("│   ✅ STABLE: Consistent tick performance (σ={:.1}%)",
                 (stddev / avg_tick_time * 100.0));
    }

    if p99_tick_time as f64 > target_tick_us * 2.0 {
        println!("│   ⚠️ OUTLIERS: P99 significantly exceeds target ({:.1}ms vs {:.1}ms)",
                 p99_tick_time as f64 / 1000.0, target_ms);
        println!("│   Recommendation: Investigate spike causes");
    }

    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("│ NEXT STEPS:                                                    │");
    println!("│ 1. Test other entity counts:                                   │");
    println!("│    STRESS_TEST_CONFIG=config/spawn_config_stress_100.ron \\    │");
    println!("│    cargo run --release --bin stress_test                       │");
    println!("│ 2. Run extended tests: STRESS_TEST_DURATION=120                │");
    println!("│ 3. Profile with flamegraph:                                    │");
    println!("│    cargo flamegraph --bin stress_test                          │");
    println!("│ 4. Compare results across entity counts                        │");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}
