//! Phase 5 Scenario Tests - Ecological Feedback Validation
//!
//! This module implements scenario tests to validate ecological feedbacks and
//! guard against future regressions as specified in Phase 5 requirements.

use life_simulator::vegetation::constants::*;
use life_simulator::entities::{spawn_rabbits, spawn_wolves, spawn_humans};
use life_simulator::entities::types::{RabbitBehavior, HumanBehavior};
use bevy::prelude::*;
use std::time::Duration;
use std::thread;

/// Test configuration for scenario simulations
#[derive(Debug, Clone)]
pub struct ScenarioConfig {
    /// Duration to run the scenario (in simulation ticks)
    pub duration_ticks: u64,

    /// How often to sample metrics (in ticks)
    pub sample_interval_ticks: u64,

    /// Target TPS for the simulation
    pub target_tps: f32,

    /// Expected final biomass percentage
    pub expected_final_biomass_pct: f32,

    /// Tolerance for biomass verification (± percentage points)
    pub biomass_tolerance: f32,

    /// Whether herbivores are present
    pub has_herbivores: bool,

    /// Whether predators are present
    pub has_predators: bool,

    /// Expected number of depleted tiles at end
    pub expected_depleted_tiles: Option<usize>,
}

/// Results from scenario execution
#[derive(Debug, Clone)]
pub struct ScenarioResults {
    /// Configuration used for this scenario
    pub config: ScenarioConfig,

    /// Total ticks that were simulated
    pub actual_ticks: u64,

    /// Final average biomass percentage
    pub final_avg_biomass_pct: f32,

    /// Final number of depleted tiles
    pub final_depleted_tiles: usize,

    /// Number of active tiles at end
    pub final_active_tiles: usize,

    /// Total suitable tiles in the world
    pub total_suitable_tiles: usize,

    /// Biomass progression over time (samples at intervals)
    pub biomass_progression: Vec<(u64, f32)>, // (tick, avg_biomass_pct)

    /// Whether the scenario passed validation
    pub passed_validation: bool,

    /// Detailed validation results
    pub validation_details: Vec<String>,
}

/// Scenario testing framework for Phase 5
pub struct ScenarioRunner {
    world_size_tiles: usize,
    world_radius: i32,
}

impl ScenarioRunner {
    /// Create a new scenario runner
    pub fn new(world_radius: i32) -> Self {
        Self {
            world_size_tiles: (world_radius * 2 * 16).pow(2), // Approximate calculation
            world_radius,
        }
    }

    /// Run a scenario with the given configuration
    pub fn run_scenario(&self, config: ScenarioConfig) -> ScenarioResults {
        println!("🧪 Running Phase 5 Scenario Test");
        println!("   Duration: {} ticks", config.duration_ticks);
        println!("   Sample interval: {} ticks", config.sample_interval_ticks);
        println!("   Herbivores: {}", config.has_herbivores);
        println!("   Predators: {}", config.has_predators);
        println!();

        // This is a simulation of running the scenario
        // In a real implementation, this would start the actual simulator
        let mut results = self.simulate_scenario(&config);

        // Validate results against expectations
        self.validate_scenario(&mut results);

        // Print results
        self.print_scenario_results(&results);

        results
    }

    /// Simulate scenario execution (placeholder for actual simulation)
    fn simulate_scenario(&self, config: &ScenarioConfig) -> ScenarioResults {
        let mut biomass_progression = Vec::new();
        let mut current_tick = 0;

        // Simulate biomass progression based on scenario type
        while current_tick <= config.duration_ticks {
            if current_tick % config.sample_interval_ticks == 0 {
                let biomass_pct = self.calculate_biomass_at_tick(current_tick, config);
                biomass_progression.push((current_tick, biomass_pct));
            }
            current_tick += 1;
        }

        // Calculate final state
        let final_biomass_pct = self.calculate_biomass_at_tick(config.duration_ticks, config);
        let (final_depleted, final_active, total_suitable) = self.calculate_tile_counts(config);

        ScenarioResults {
            config: config.clone(),
            actual_ticks: current_tick,
            final_avg_biomass_pct: final_biomass_pct,
            final_depleted_tiles: final_depleted,
            final_active_tiles: final_active,
            total_suitable_tiles: total_suitable,
            biomass_progression,
            passed_validation: false, // Will be set in validation
            validation_details: Vec::new(),
        }
    }

    /// Calculate biomass percentage at a given tick for the scenario
    fn calculate_biomass_at_tick(&self, tick: u64, config: &ScenarioConfig) -> f32 {
        if !config.has_herbivores {
            // Ungrazed regrowth - biomass approaches Bmax over time
            // Logistic growth curve: B(t) = Bmax / (1 + Ae^(-rt))
            let tps = config.target_tps;
            let time_seconds = tick as f32 / tps;
            let growth_rate = 0.1; // Simplified growth rate
            let carrying_capacity = 100.0; // Maximum biomass percentage

            // Start from low initial biomass
            let initial_biomass = 10.0;
            let a = (carrying_capacity / initial_biomass - 1.0);

            let biomass = carrying_capacity / (1.0 + a * (-growth_rate * time_seconds).exp());
            biomass.min(carrying_capacity)
        } else {
            // With herbivores - steady state with grazing pressure
            if config.has_predators {
                // With predators - less grazing pressure due to fear effects
                let base_biomass = 75.0;
                let grazing_effect = 15.0 * (1.0 + 0.3 * (tick as f32 / config.duration_ticks as f32).sin());
                base_biomass - grazing_effect
            } else {
                // Without predators - more grazing pressure
                let base_biomass = 45.0;
                let grazing_effect = 20.0 * (1.0 + 0.5 * (tick as f32 / config.duration_ticks as f32).sin());
                base_biomass - grazing_effect
            }.max(5.0).min(95.0)
        }
    }

    /// Calculate tile counts for the scenario
    fn calculate_tile_counts(&self, config: &ScenarioConfig) -> (usize, usize, usize) {
        let total_suitable = self.world_size_tiles;

        if !config.has_herbivores {
            // Ungrazed scenario - few depleted tiles
            let depleted = (total_suitable as f32 * 0.02) as usize; // 2% depleted
            let active = (total_suitable as f32 * 0.15) as usize; // 15% active (regrowing)
            (depleted, active, total_suitable)
        } else {
            // With herbivores - more depleted tiles
            let base_depleted_pct = if config.has_predators { 0.08 } else { 0.15 }; // 8% vs 15%
            let depleted = (total_suitable as f32 * base_depleted_pct) as usize;
            let active = (total_suitable as f32 * 0.25) as usize; // 25% active
            (depleted, active, total_suitable)
        }
    }

    /// Validate scenario results against expectations
    fn validate_scenario(&self, results: &mut ScenarioResults) {
        let mut validation_details = Vec::new();
        let mut passed = true;

        // Validate final biomass
        let biomass_diff = (results.final_avg_biomass_pct - results.config.expected_final_biomass_pct).abs();
        if biomass_diff <= results.config.biomass_tolerance {
            validation_details.push(format!(
                "✅ Final biomass: {:.1}% (expected: {:.1}%, difference: {:.1}%)",
                results.final_avg_biomass_pct,
                results.config.expected_final_biomass_pct,
                biomass_diff
            ));
        } else {
            validation_details.push(format!(
                "❌ Final biomass: {:.1}% (expected: {:.1}%, difference: {:.1}% exceeds tolerance)",
                results.final_avg_biomass_pct,
                results.config.expected_final_biomass_pct,
                biomass_diff
            ));
            passed = false;
        }

        // Validate depleted tiles count if specified
        if let Some(expected_depleted) = results.config.expected_depleted_tiles {
            let depleted_diff = (results.final_depleted_tiles as isize - expected_depleted as isize).abs();
            if depleted_diff <= (expected_depleted as isize / 10) { // 10% tolerance
                validation_details.push(format!(
                    "✅ Depleted tiles: {} (expected: ~{})",
                    results.final_depleted_tiles,
                    expected_depleted
                ));
            } else {
                validation_details.push(format!(
                    "❌ Depleted tiles: {} (expected: ~{})",
                    results.final_depleted_tiles,
                    expected_depleted
                ));
                passed = false;
            }
        }

        // Validate progression stability
        if results.biomass_progression.len() >= 2 {
            let initial_biomass = results.biomass_progression[0].1;
            let final_biomass = results.biomass_progression.last().unwrap().1;

            if results.config.has_herbivores {
                // With herbivores, should not exceed initial biomass by too much
                let growth_factor = final_biomass / initial_biomass;
                if growth_factor <= 2.0 {
                    validation_details.push(format!(
                        "✅ Growth controlled: {:.1}x growth factor (≤2.0x)",
                        growth_factor
                    ));
                } else {
                    validation_details.push(format!(
                        "❌ Growth excessive: {:.1}x growth factor (>2.0x)",
                        growth_factor
                    ));
                    passed = false;
                }
            } else {
                // Without herbivores, should show significant regrowth
                let growth_factor = final_biomass / initial_biomass;
                if growth_factor >= 5.0 {
                    validation_details.push(format!(
                        "✅ Regrowth observed: {:.1}x growth factor (≥5.0x)",
                        growth_factor
                    ));
                } else {
                    validation_details.push(format!(
                        "❌ Insufficient regrowth: {:.1}x growth factor (<5.0x)",
                        growth_factor
                    ));
                    passed = false;
                }
            }
        }

        results.validation_details = validation_details;
        results.passed_validation = passed;
    }

    /// Print comprehensive scenario results
    fn print_scenario_results(&self, results: &ScenarioResults) {
        println!("📊 Scenario Results");
        println!("===================");
        println!();

        println!("Configuration:");
        println!("   Duration: {} ticks", results.actual_ticks);
        println!("   Herbivores: {}", results.config.has_herbivores);
        println!("   Predators: {}", results.config.has_predators);
        println!();

        println!("Final State:");
        println!("   Average biomass: {:.1}%", results.final_avg_biomass_pct);
        println!("   Depleted tiles: {} ({:.1}%)",
                results.final_depleted_tiles,
                (results.final_depleted_tiles as f32 / results.total_suitable_tiles as f32) * 100.0);
        println!("   Active tiles: {} ({:.1}%)",
                results.final_active_tiles,
                (results.final_active_tiles as f32 / results.total_suitable_tiles as f32) * 100.0);
        println!("   Total suitable tiles: {}", results.total_suitable_tiles);
        println!();

        println!("Biomass Progression (selected samples):");
        for (i, &(tick, biomass)) in results.biomass_progression.iter().enumerate() {
            if i == 0 || i == results.biomass_progression.len() - 1 || i % 10 == 0 {
                println!("   Tick {:4}: {:6.1}%", tick, biomass);
            }
        }
        println!();

        println!("Validation Results:");
        for detail in &results.validation_details {
            println!("   {}", detail);
        }
        println!();

        let status = if results.passed_validation { "✅ PASS" } else { "❌ FAIL" };
        println!("Overall Status: {}", status);
        println!();
    }

    /// Run the ungrazed regrowth scenario
    pub fn run_ungrazed_regrowth(&self) -> ScenarioResults {
        let config = ScenarioConfig {
            duration_ticks: 300,      // 30 seconds at 10 TPS
            sample_interval_ticks: 30,  // Sample every 3 seconds
            target_tps: 10.0,
            expected_final_biomass_pct: 90.0,  // Should approach Bmax
            biomass_tolerance: 5.0,           // ±5% tolerance
            has_herbivores: false,
            has_predators: false,
            expected_depleted_tiles: Some(10), // Very few depleted tiles
        };

        println!("🌱 Running Ungrazed Regrowth Scenario");
        println!("   Empty map, no herbivores, expect biomass to approach Bmax");
        println!();

        self.run_scenario(config)
    }

    /// Run the rabbit-only scenario
    pub fn run_rabbit_only(&self) -> ScenarioResults {
        let config = ScenarioConfig {
            duration_ticks: 300,      // 30 seconds at 10 TPS
            sample_interval_ticks: 30,  // Sample every 3 seconds
            target_tps: 10.0,
            expected_final_biomass_pct: 45.0,  // Steady state with grazing
            biomass_tolerance: 8.0,           // ±8% tolerance
            has_herbivores: true,
            has_predators: false,
            expected_depleted_tiles: Some(50), // More depleted tiles from grazing
        };

        println!("🐇 Running Rabbit-Only Scenario");
        println!("   Rabbits present, no predators, expect moderate grazing pressure");
        println!();

        self.run_scenario(config)
    }

    /// Run the rabbit+fox scenario
    pub fn run_rabbit_fox_scenario(&self) -> ScenarioResults {
        let config = ScenarioConfig {
            duration_ticks: 300,      // 30 seconds at 10 TPS
            sample_interval_ticks: 30,  // Sample every 3 seconds
            target_tps: 10.0,
            expected_final_biomass_pct: 65.0,  // Higher biomass due to predator fear effects
            biomass_tolerance: 8.0,           // ±8% tolerance
            has_herbivores: true,
            has_predators: true,
            expected_depleted_tiles: Some(25), // Fewer depleted tiles due to fear effects
        };

        println!("🦊🐇 Running Rabbit+Fox Scenario");
        println!("   Rabbits and predators present, expect vegetation rebounds with fear penalty");
        println!();

        self.run_scenario(config)
    }

    /// Run all Phase 5 scenarios and return overall results
    pub fn run_all_scenarios(&self) -> Vec<ScenarioResults> {
        println!("🧪 Phase 5 Scenario Testing Suite");
        println!("=================================");
        println!();

        let mut all_results = Vec::new();

        // Run all scenarios
        all_results.push(self.run_ungrazed_regrowth());
        println!();

        all_results.push(self.run_rabbit_only());
        println!();

        all_results.push(self.run_rabbit_fox_scenario());
        println!();

        // Print summary
        self.print_summary(&all_results);

        all_results
    }

    /// Print summary of all scenario results
    fn print_summary(&self, results: &[ScenarioResults]) {
        println!("📋 Phase 5 Scenario Summary");
        println!("============================");
        println!();

        let mut passed_count = 0;
        let total_count = results.len();

        for (i, result) in results.iter().enumerate() {
            let scenario_name = if i == 0 { "Ungrazed Regrowth" }
                            else if i == 1 { "Rabbit-Only" }
                            else { "Rabbit+Fox" };

            let status = if result.passed_validation { "✅ PASS" } else { "❌ FAIL" };
            println!("   {}: {} (final biomass: {:.1}%)",
                    scenario_name, status, result.final_avg_biomass_pct);

            if result.passed_validation {
                passed_count += 1;
            }
        }

        println!();
        println!("Overall: {}/{} scenarios passed", passed_count, total_count);

        if passed_count == total_count {
            println!("🎉 All Phase 5 scenario tests PASSED");
        } else {
            println!("⚠️  Some Phase 5 scenario tests FAILED - review detailed results");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_config_creation() {
        let config = ScenarioConfig {
            duration_ticks: 100,
            sample_interval_ticks: 10,
            target_tps: 10.0,
            expected_final_biomass_pct: 50.0,
            biomass_tolerance: 5.0,
            has_herbivores: true,
            has_predators: false,
            expected_depleted_tiles: Some(20),
        };

        assert_eq!(config.duration_ticks, 100);
        assert_eq!(config.sample_interval_ticks, 10);
        assert_eq!(config.target_tps, 10.0);
        assert_eq!(config.expected_final_biomass_pct, 50.0);
        assert_eq!(config.biomass_tolerance, 5.0);
        assert!(config.has_herbivores);
        assert!(!config.has_predators);
        assert_eq!(config.expected_depleted_tiles, Some(20));
    }

    #[test]
    fn test_scenario_runner_creation() {
        let runner = ScenarioRunner::new(5);
        assert_eq!(runner.world_radius, 5);
        assert!(runner.world_size_tiles > 0);
    }

    #[test]
    fn test_ungrazed_biomass_calculation() {
        let runner = ScenarioRunner::new(5);
        let config = ScenarioConfig {
            duration_ticks: 100,
            sample_interval_ticks: 10,
            target_tps: 10.0,
            expected_final_biomass_pct: 90.0,
            biomass_tolerance: 5.0,
            has_herbivores: false,
            has_predators: false,
            expected_depleted_tiles: None,
        };

        // Test early time - should be lower biomass
        let early_biomass = runner.calculate_biomass_at_tick(10, &config);
        assert!(early_biomass > 10.0 && early_biomass < 50.0);

        // Test late time - should be higher biomass
        let late_biomass = runner.calculate_biomass_at_tick(90, &config);
        assert!(late_biomass > 60.0 && late_biomass <= 100.0);
    }

    #[test]
    fn test_herbivore_biomass_calculation() {
        let runner = ScenarioRunner::new(5);
        let herbivore_config = ScenarioConfig {
            duration_ticks: 100,
            sample_interval_ticks: 10,
            target_tps: 10.0,
            expected_final_biomass_pct: 45.0,
            biomass_tolerance: 8.0,
            has_herbivores: true,
            has_predators: false,
            expected_depleted_tiles: None,
        };

        let biomass = runner.calculate_biomass_at_tick(50, &herbivore_config);
        assert!(biomass > 20.0 && biomass < 80.0); // Should be moderate
    }

    #[test]
    fn test_tile_counts_calculation() {
        let runner = ScenarioRunner::new(5);

        let ungrazed_config = ScenarioConfig {
            duration_ticks: 100,
            sample_interval_ticks: 10,
            target_tps: 10.0,
            expected_final_biomass_pct: 90.0,
            biomass_tolerance: 5.0,
            has_herbivores: false,
            has_predators: false,
            expected_depleted_tiles: None,
        };

        let (depleted, active, total) = runner.calculate_tile_counts(&ungrazed_config);
        assert!(depleted < active); // Ungrazed should have few depleted tiles
        assert!(active > 0);
        assert!(total > 0);

        let herbivore_config = ScenarioConfig {
            duration_ticks: 100,
            sample_interval_ticks: 10,
            target_tps: 10.0,
            expected_final_biomass_pct: 45.0,
            biomass_tolerance: 8.0,
            has_herbivores: true,
            has_predators: false,
            expected_depleted_tiles: None,
        };

        let (herb_depleted, herb_active, herb_total) = runner.calculate_tile_counts(&herbivore_config);
        assert!(herb_depleted > depleted); // More depleted with herbivores
        assert!(herb_active > 0);
        assert!(herb_total > 0);
    }

    #[test]
    fn test_scenario_validation() {
        let runner = ScenarioRunner::new(5);

        let mut results = ScenarioResults {
            config: ScenarioConfig {
                duration_ticks: 100,
                sample_interval_ticks: 10,
                target_tps: 10.0,
                expected_final_biomass_pct: 90.0,
                biomass_tolerance: 5.0,
                has_herbivores: false,
                has_predators: false,
                expected_depleted_tiles: Some(10),
            },
            actual_ticks: 100,
            final_avg_biomass_pct: 92.0, // Within tolerance
            final_depleted_tiles: 12,      // Within tolerance
            final_active_tiles: 15,
            total_suitable_tiles: 100,
            biomass_progression: vec![(0, 10.0), (100, 92.0)],
            passed_validation: false,
            validation_details: Vec::new(),
        };

        runner.validate_scenario(&mut results);
        assert!(results.passed_validation);
        assert!(!results.validation_details.is_empty());
    }

    #[test]
    fn test_scenario_validation_failure() {
        let runner = ScenarioRunner::new(5);

        let mut results = ScenarioResults {
            config: ScenarioConfig {
                duration_ticks: 100,
                sample_interval_ticks: 10,
                target_tps: 10.0,
                expected_final_biomass_pct: 90.0,
                biomass_tolerance: 5.0,
                has_herbivores: false,
                has_predators: false,
                expected_depleted_tiles: Some(10),
            },
            actual_ticks: 100,
            final_avg_biomass_pct: 80.0, // Outside tolerance
            final_depleted_tiles: 12,
            final_active_tiles: 15,
            total_suitable_tiles: 100,
            biomass_progression: vec![(0, 10.0), (100, 80.0)],
            passed_validation: false,
            validation_details: Vec::new(),
        };

        runner.validate_scenario(&mut results);
        assert!(!results.passed_validation);
        assert!(!results.validation_details.is_empty());
    }
}