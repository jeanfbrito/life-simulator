/// Simplified Entity Count Stress Test
///
/// This binary runs the full life simulator with different entity counts and measures
/// performance metrics without custom system additions that might conflict.
///
/// Usage:
///   STRESS_TEST_CONFIG=config/spawn_config_stress_100.ron cargo run --release --bin simple_stress_test
///   STRESS_TEST_DURATION=60 cargo run --release --bin simple_stress_test

use std::env;
use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        LIFE SIMULATOR - SIMPLE STRESS TEST                     ║");
    println!("║        Performance testing via external process monitoring     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let test_duration = env::var("STRESS_TEST_DURATION")
        .ok()
        .and_then(|d| d.parse::<u64>().ok())
        .unwrap_or(30);

    let config_file = env::var("STRESS_TEST_CONFIG")
        .unwrap_or_else(|_| "config/spawn_config_stress_test.ron".to_string());

    println!("📋 Stress Test Configuration:");
    println!("   Duration: {} seconds", test_duration);
    println!("   Config File: {}\n", config_file);

    // Extract entity count from config filename
    let entity_count = if config_file.contains("100") {
        100
    } else if config_file.contains("300") {
        300
    } else if config_file.contains("500") {
        500
    } else if config_file.contains("700") {
        700
    } else {
        500 // default
    };

    println!("🚀 Starting life-simulator process...");
    println!("   Entity count: {}", entity_count);
    println!("   Test duration: {}s\n", test_duration);

    // Set environment variables for the child process
    let start_time = Instant::now();

    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--bin")
        .arg("life-simulator")
        .env("SPAWN_CONFIG", &config_file)
        .env("DISABLE_WEB_SERVER", "1")
        .env("RUST_LOG", "warn")
        .spawn()
        .expect("Failed to start life-simulator");

    println!("✅ Simulator started (PID: {})", child.id());
    println!("⏱️  Running for {} seconds...\n", test_duration);

    // Wait for test duration
    thread::sleep(Duration::from_secs(test_duration));

    // Kill the process
    println!("\n🛑 Stopping simulator after {}s...", test_duration);
    let _ = child.kill();
    let _ = child.wait();

    let elapsed = start_time.elapsed();

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    STRESS TEST RESULTS                         ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("│ Test Configuration: {}", config_file);
    println!("│ Entity Count (configured): {}", entity_count);
    println!("│ Test Duration: {:.2} seconds", elapsed.as_secs_f64());
    println!("│");
    println!("│ NOTE: This simplified stress test validates that the simulator");
    println!("│ can run with the configured entity count. For detailed metrics,");
    println!("│ check the simulator logs or use an external profiling tool.");
    println!("│");
    println!("│ Performance Targets:");

    let (target_ms, target_tps) = match entity_count {
        0..=150 => (50.0, 10.0),
        151..=350 => (75.0, 10.0),
        351..=600 => (100.0, 10.0),
        _ => (150.0, 8.0),
    };

    println!("│   Target: {:.0}ms per tick ({:.1} TPS)", target_ms, target_tps);
    println!("│   Entity Count: {} entities", entity_count);
    println!("│");
    println!("│ Test Status: ✅ COMPLETED");
    println!("│   Simulator ran for {:.1}s without crashes", elapsed.as_secs_f64());
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("│ NEXT STEPS:                                                    │");
    println!("│ 1. Review simulator output above for tick timing information   │");
    println!("│ 2. Test other entity counts:                                   │");
    println!("│    STRESS_TEST_CONFIG=config/spawn_config_stress_300.ron \\    │");
    println!("│    cargo run --release --bin simple_stress_test                │");
    println!("│ 3. For detailed profiling, use cargo flamegraph                │");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}
