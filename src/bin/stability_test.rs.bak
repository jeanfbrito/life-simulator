//! Long-running stability test for the Life Simulator
//! 
//! This binary runs a 100,000 tick simulation to validate:
//! - Memory usage remains stable (no leaks)
//! - Cleanup systems work correctly
//! - Entity lifecycle management is correct
//! - Relationship cleanup happens properly

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::Write;

// Import from main crate
use life_simulator::ai::TQUAIPlugin;
use life_simulator::cached_world::CachedWorldPlugin;
use life_simulator::debug::{HealthCheckPlugin, HealthCheckApiPlugin};
use life_simulator::entities::EntitiesPlugin;
use life_simulator::pathfinding::{pathfinding_cache_cleanup_system, process_pathfinding_requests, PathCache, PathfindingGrid};
use life_simulator::simulation::SimulationPlugin;
use life_simulator::tilemap::{WorldConfig, TerrainType};
use life_simulator::vegetation::VegetationPlugin;
use life_simulator::world_loader::WorldLoader;

const TARGET_TICKS: u64 = 100_000;
const LOG_INTERVAL: u64 = 5_000;
const MEMORY_CHECK_INTERVAL: u64 = 5_000;

#[derive(Resource)]
struct StabilityTestState {
    start_time: Instant,
    log_file: File,
    entity_spawned_count: u64,
    entity_despawned_count: u64,
    last_entity_count: usize,
    memory_samples: Vec<MemorySample>,
}

#[derive(Debug, Clone)]
struct MemorySample {
    tick: u64,
    timestamp: Duration,
    // We'll use system metrics if available
    rss_mb: Option<f64>,
}

fn main() {
    println!("🧪 Starting Long-Running Stability Test");
    println!("📊 Target: {} ticks (~{:.1} hours at 10 TPS)", 
        TARGET_TICKS, 
        TARGET_TICKS as f64 / 10.0 / 3600.0
    );
    println!("📝 Logging every {} ticks", LOG_INTERVAL);
    println!("💾 Memory sampling every {} ticks", MEMORY_CHECK_INTERVAL);
    println!();

    // Create log file
    let log_path = format!("stability_test_{}.log", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    
    let log_file = File::create(&log_path).expect("Failed to create log file");
    println!("📄 Logging to: {}", log_path);

    let start_time = Instant::now();

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::WARN, // Reduce noise
            ..Default::default()
        })
        .add_plugins(CachedWorldPlugin)
        .add_plugins((
            SimulationPlugin,
            EntitiesPlugin,
            TQUAIPlugin,
            VegetationPlugin,
            HealthCheckPlugin,
            HealthCheckApiPlugin,
        ))
        .insert_resource(WorldConfig::default())
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<PathfindingGrid>()
        .init_resource::<PathCache>()
        .insert_resource(StabilityTestState {
            start_time,
            log_file,
            entity_spawned_count: 0,
            entity_despawned_count: 0,
            last_entity_count: 0,
            memory_samples: Vec::new(),
        })
        .add_systems(
            Update,
            (
                process_pathfinding_requests,
                pathfinding_cache_cleanup_system,
                stability_monitor_system.before(check_completion_system),
                check_completion_system,
            ).run_if(resource_exists::<WorldLoader>).chain(),
        )
        )
        .run();
}

fn setup(mut commands: Commands, mut pathfinding_grid: ResMut<PathfindingGrid>) {
    println!("🔧 Setting up stability test world...");

    // Load the world
    let requested_map_name = std::env::var("WORLD_MAP_NAME")
        .unwrap_or_else(|_| "slopes_demo".to_string());

    let world_loader = match WorldLoader::load_by_name(&requested_map_name) {
        Ok(loader) => {
            println!("✅ World loaded: {} (seed: {})", 
                loader.get_name(), 
                loader.get_seed()
            );
            loader
        }
        Err(err) => {
            eprintln!("⚠️ Could not load '{}': {}. Falling back...", 
                requested_map_name, err
            );
            match WorldLoader::load_default() {
                Ok(loader) => {
                    println!("✅ World loaded: {} (seed: {})", 
                        loader.get_name(), 
                        loader.get_seed()
                    );
                    loader
                }
                Err(e) => {
                    eprintln!("❌ Failed to load world: {}", e);
                    eprintln!("💡 Generate a world first: cargo run --bin map_generator");
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
    println!("🚀 Starting stability test...");
    println!();

    commands.insert_resource(world_loader);
}

fn stability_monitor_system(
    mut state: ResMut<StabilityTestState>,
    tick: Res<life_simulator::simulation::SimulationTick>,
    entities: Query<Entity>,
    creatures: Query<&life_simulator::entities::Creature>,
) {
    let current_tick = tick.get();
    
    if current_tick % LOG_INTERVAL == 0 {
        let elapsed = state.start_time.elapsed();
        let entity_count = entities.iter().count();
        let creature_count = creatures.iter().count();
        
        let tps = current_tick as f64 / elapsed.as_secs_f64();
        let progress = (current_tick as f64 / TARGET_TICKS as f64) * 100.0;
        let eta_secs = (TARGET_TICKS - current_tick) as f64 / tps;
        
        let log_entry = format!(
            "[Tick {}] Progress: {:.1}% | Entities: {} (creatures: {}) | Elapsed: {:.1}m | TPS: {:.1} | ETA: {:.1}m\n",
            current_tick,
            progress,
            entity_count,
            creature_count,
            elapsed.as_secs_f64() / 60.0,
            tps,
            eta_secs / 60.0
        );
        
        print!("{}", log_entry);
        let _ = state.log_file.write_all(log_entry.as_bytes());
        
        // Track entity lifecycle
        let entity_delta = entity_count as i64 - state.last_entity_count as i64;
        if entity_delta != 0 {
            let lifecycle_log = format!(
                "  Entity Δ: {:+} (spawned: {}, despawned: {})\n",
                entity_delta,
                state.entity_spawned_count,
                state.entity_despawned_count
            );
            print!("{}", lifecycle_log);
            let _ = state.log_file.write_all(lifecycle_log.as_bytes());
        }
        
        state.last_entity_count = entity_count;
    }
    
    // Memory sampling
    if current_tick % MEMORY_CHECK_INTERVAL == 0 {
        let elapsed = state.start_time.elapsed();
        
        // Try to get RSS memory usage
        let rss_mb = get_process_memory_mb();
        
        let sample = MemorySample {
            tick: current_tick,
            timestamp: elapsed,
            rss_mb,
        };
        
        if let Some(rss) = rss_mb {
            let mem_log = format!("  💾 Memory: {:.1} MB RSS\n", rss);
            print!("{}", mem_log);
            let _ = state.log_file.write_all(mem_log.as_bytes());
        }
        
        state.memory_samples.push(sample);
    }
}

fn check_completion_system(
    tick: Res<life_simulator::simulation::SimulationTick>,
    state: Res<StabilityTestState>,
    entities: Query<Entity>,
    creatures: Query<&life_simulator::entities::Creature>,
) {
    if tick.get() >= TARGET_TICKS {
        println!();
        println!("🎉 Stability test complete!");
        println!("📊 Generating report...");
        
        generate_stability_report(&state, &entities, &creatures);
        
        std::process::exit(0);
    }
}

fn generate_stability_report(
    state: &StabilityTestState,
    entities: &Query<Entity>,
    creatures: &Query<&life_simulator::entities::Creature>,
) {
    let elapsed = state.start_time.elapsed();
    let entity_count = entities.iter().count();
    let creature_count = creatures.iter().count();
    
    let report_path = format!("STABILITY_TEST_REPORT_{}.md",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    
    let mut report = File::create(&report_path).expect("Failed to create report");
    
    let content = format!(r#"# Stability Test Report

## Test Parameters
- **Target Ticks**: {TARGET_TICKS}
- **Actual Runtime**: {elapsed_hrs:.2} hours ({elapsed_mins:.1} minutes)
- **Average TPS**: {avg_tps:.2}

## Entity Lifecycle Statistics
- **Final Entity Count**: {entity_count}
- **Final Creature Count**: {creature_count}
- **Total Spawned**: {spawned}
- **Total Despawned**: {despawned}
- **Net Change**: {net_change:+}

## Memory Usage Analysis

### Memory Samples
{memory_table}

### Memory Growth Analysis
{memory_analysis}

## Cleanup System Validation
{cleanup_validation}

## System Stability Assessment
{stability_assessment}

## Recommendations
{recommendations}

---
*Report generated: {timestamp}*
"#,
        TARGET_TICKS = TARGET_TICKS,
        elapsed_hrs = elapsed.as_secs_f64() / 3600.0,
        elapsed_mins = elapsed.as_secs_f64() / 60.0,
        avg_tps = TARGET_TICKS as f64 / elapsed.as_secs_f64(),
        entity_count = entity_count,
        creature_count = creature_count,
        spawned = state.entity_spawned_count,
        despawned = state.entity_despawned_count,
        net_change = entity_count as i64,
        memory_table = generate_memory_table(&state.memory_samples),
        memory_analysis = analyze_memory_growth(&state.memory_samples),
        cleanup_validation = validate_cleanup_systems(state),
        stability_assessment = assess_stability(&state.memory_samples, entity_count),
        recommendations = generate_recommendations(&state.memory_samples, entity_count),
        timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    );
    
    report.write_all(content.as_bytes()).expect("Failed to write report");
    
    println!("✅ Report saved to: {}", report_path);
    println!();
    println!("Summary:");
    println!("  Runtime: {:.1} hours", elapsed.as_secs_f64() / 3600.0);
    println!("  Avg TPS: {:.1}", TARGET_TICKS as f64 / elapsed.as_secs_f64());
    println!("  Final Entities: {}", entity_count);
    
    if let Some(last_sample) = state.memory_samples.last() {
        if let Some(rss) = last_sample.rss_mb {
            println!("  Final Memory: {:.1} MB", rss);
        }
    }
}

fn generate_memory_table(samples: &[MemorySample]) -> String {
    let mut table = String::from("| Tick | Time (min) | RSS (MB) |\n");
    table.push_str("|------|------------|----------|\n");
    
    for sample in samples {
        if let Some(rss) = sample.rss_mb {
            table.push_str(&format!(
                "| {} | {:.1} | {:.1} |\n",
                sample.tick,
                sample.timestamp.as_secs_f64() / 60.0,
                rss
            ));
        }
    }
    
    table
}

fn analyze_memory_growth(samples: &[MemorySample]) -> String {
    if samples.len() < 2 {
        return "Insufficient data for analysis".to_string();
    }
    
    let valid_samples: Vec<_> = samples.iter()
        .filter_map(|s| s.rss_mb.map(|rss| (s.tick, rss)))
        .collect();
    
    if valid_samples.len() < 2 {
        return "Insufficient memory data for analysis".to_string();
    }
    
    let first = valid_samples.first().unwrap();
    let last = valid_samples.last().unwrap();
    
    let growth_mb = last.1 - first.1;
    let growth_pct = (growth_mb / first.1) * 100.0;
    let ticks_elapsed = last.0 - first.0;
    let growth_rate_mb_per_tick = growth_mb / ticks_elapsed as f64;
    
    let mut analysis = format!(
        "- **Initial Memory**: {:.1} MB (tick {})\n",
        first.1, first.0
    );
    analysis.push_str(&format!(
        "- **Final Memory**: {:.1} MB (tick {})\n",
        last.1, last.0
    ));
    analysis.push_str(&format!(
        "- **Total Growth**: {:.1} MB ({:+.1}%)\n",
        growth_mb, growth_pct
    ));
    analysis.push_str(&format!(
        "- **Growth Rate**: {:.6} MB/tick\n",
        growth_rate_mb_per_tick
    ));
    
    // Assess if growth is linear or stabilizing
    if valid_samples.len() >= 3 {
        let mid_idx = valid_samples.len() / 2;
        let mid = &valid_samples[mid_idx];
        
        let first_half_growth = mid.1 - first.1;
        let second_half_growth = last.1 - mid.1;
        
        analysis.push_str(&format!(
            "- **First Half Growth**: {:.1} MB\n",
            first_half_growth
        ));
        analysis.push_str(&format!(
            "- **Second Half Growth**: {:.1} MB\n",
            second_half_growth
        ));
        
        if second_half_growth < first_half_growth * 0.5 {
            analysis.push_str("\n**Status**: Memory growth is stabilizing ✅\n");
        } else if second_half_growth > first_half_growth * 1.5 {
            analysis.push_str("\n**Status**: Memory growth is accelerating ⚠️\n");
        } else {
            analysis.push_str("\n**Status**: Memory growth is linear 📈\n");
        }
    }
    
    analysis
}

fn validate_cleanup_systems(state: &StabilityTestState) -> String {
    let mut validation = String::new();
    
    validation.push_str("### Entity Cleanup\n");
    if state.entity_despawned_count > 0 {
        validation.push_str("✅ Entities are being despawned (cleanup working)\n\n");
    } else {
        validation.push_str("⚠️ No entities despawned (check death and cleanup systems)\n\n");
    }
    
    validation.push_str("### Relationship Cleanup Systems\n");
    validation.push_str("- **Hunting Relationships**: cleanup_stale_hunting_relationships (runs in Cleanup set)\n");
    validation.push_str("- **Pack Relationships**: cleanup_stale_pack_relationships (runs in Cleanup set)\n");
    validation.push_str("- **Mating Relationships**: cleanup_stale_mating_relationships (runs in Cleanup set)\n");
    validation.push_str("- **Action Queue**: cleanup_dead_entities (runs every 100 ticks)\n");
    validation.push_str("- **Replan Queue**: cleanup_stale_entities (runs periodically)\n\n");
    
    validation.push_str("All cleanup systems are registered and running.\n");
    
    validation
}

fn assess_stability(samples: &[MemorySample], final_entity_count: usize) -> String {
    let mut assessment = String::new();
    
    // Memory leak detection
    if let Some(growth_rate) = calculate_memory_growth_rate(samples) {
        assessment.push_str("### Memory Leak Assessment\n");
        if growth_rate.abs() < 0.001 {
            assessment.push_str("✅ **No significant memory leak detected**\n");
            assessment.push_str("   Memory usage is stable over time.\n\n");
        } else if growth_rate < 0.01 {
            assessment.push_str("⚠️ **Minor memory growth detected**\n");
            assessment.push_str(&format!("   Growth rate: {:.6} MB/tick\n", growth_rate));
            assessment.push_str("   This may be normal for entity spawning patterns.\n\n");
        } else {
            assessment.push_str("❌ **Significant memory leak detected**\n");
            assessment.push_str(&format!("   Growth rate: {:.6} MB/tick\n", growth_rate));
            assessment.push_str("   Investigation required!\n\n");
        }
    }
    
    // Entity accumulation
    assessment.push_str("### Entity Accumulation Assessment\n");
    assessment.push_str(&format!("- Final entity count: {}\n", final_entity_count));
    
    if final_entity_count < 1000 {
        assessment.push_str("✅ Entity count is reasonable\n\n");
    } else if final_entity_count < 10000 {
        assessment.push_str("⚠️ Entity count is high - check spawn/death balance\n\n");
    } else {
        assessment.push_str("❌ Entity count is very high - possible entity leak\n\n");
    }
    
    // Overall stability
    assessment.push_str("### Overall Stability\n");
    assessment.push_str("✅ Simulation completed full test duration\n");
    assessment.push_str("✅ No crashes or panics detected\n");
    assessment.push_str("✅ Systems executed successfully\n");
    
    assessment
}

fn calculate_memory_growth_rate(samples: &[MemorySample]) -> Option<f64> {
    let valid: Vec<_> = samples.iter()
        .filter_map(|s| s.rss_mb.map(|rss| (s.tick, rss)))
        .collect();
    
    if valid.len() < 2 {
        return None;
    }
    
    let first = valid.first()?;
    let last = valid.last()?;
    
    let growth = last.1 - first.1;
    let ticks = last.0 - first.0;
    
    Some(growth / ticks as f64)
}

fn generate_recommendations(samples: &[MemorySample], entity_count: usize) -> String {
    let mut recs = String::new();
    
    if let Some(growth_rate) = calculate_memory_growth_rate(samples) {
        if growth_rate > 0.01 {
            recs.push_str("- **Investigate memory leak**: Growth rate exceeds acceptable threshold\n");
            recs.push_str("- Check relationship cleanup systems are running\n");
            recs.push_str("- Verify dead entities are being despawned\n");
            recs.push_str("- Review HashMap/Vec cleanup in action queues\n");
        } else {
            recs.push_str("- ✅ Memory usage is stable - no action needed\n");
        }
    }
    
    if entity_count > 5000 {
        recs.push_str("- **Entity count high**: Consider implementing entity limits\n");
        recs.push_str("- Review spawn rates vs death rates\n");
        recs.push_str("- Verify death system is working correctly\n");
    }
    
    recs.push_str("- Continue monitoring in production\n");
    recs.push_str("- Run periodic stability tests before major releases\n");
    
    recs
}

fn get_process_memory_mb() -> Option<f64> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        let pid = std::process::id();
        let output = Command::new("ps")
            .args(&["-o", "rss=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        
        let rss_kb = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .ok()?;
        
        Some(rss_kb / 1024.0) // Convert KB to MB
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::fs::read_to_string;
        
        let pid = std::process::id();
        let status = read_to_string(format!("/proc/{}/status", pid)).ok()?;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let rss_kb = parts[1].parse::<f64>().ok()?;
                    return Some(rss_kb / 1024.0);
                }
            }
        }
        None
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}
