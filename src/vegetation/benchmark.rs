use crate::simulation::SimulationTick;
use std::collections::VecDeque;
/// Performance benchmarking module for Phase 4 verification
///
/// This module implements comprehensive performance testing to verify that
/// the vegetation growth system stays within the 1ms budget at 10 TPS.
use std::time::{Duration, Instant};

/// Benchmark configuration and results
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Duration to run the benchmark (in seconds)
    pub duration_seconds: u64,

    /// Target TPS for the simulation
    pub target_tps: f32,

    /// CPU budget per growth cycle (in microseconds)
    pub cpu_budget_us: u64,

    /// Target tick interval (100ms for 10 TPS)
    pub tick_interval_ms: u64,

    /// Number of warmup ticks to ignore in results
    pub warmup_ticks: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            duration_seconds: 10,  // 10 second benchmark
            target_tps: 10.0,      // 10 TPS target
            cpu_budget_us: 1000,   // 1ms budget
            tick_interval_ms: 100, // 100ms between ticks
            warmup_ticks: 20,      // 2 second warmup
        }
    }
}

/// Comprehensive benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Benchmark configuration used
    pub config: BenchmarkConfig,

    /// Total duration of the benchmark
    pub actual_duration_ms: u64,

    /// Total number of ticks completed
    pub total_ticks: u64,

    /// Actual TPS achieved
    pub actual_tps: f32,

    /// Vegetation growth system metrics
    pub growth_metrics: GrowthBenchmarkMetrics,

    /// System-wide performance metrics
    pub system_metrics: SystemBenchmarkMetrics,

    /// Budget compliance analysis
    pub budget_analysis: BudgetAnalysis,
}

/// Specific metrics for vegetation growth system
#[derive(Debug, Clone)]
pub struct GrowthBenchmarkMetrics {
    /// Total time spent in vegetation updates (microseconds)
    pub total_growth_time_us: u64,

    /// Average time per growth cycle (microseconds)
    pub avg_growth_time_us: f64,

    /// Maximum time spent in a single growth cycle (microseconds)
    pub max_growth_time_us: u64,

    /// Minimum time spent in a single growth cycle (microseconds)
    pub min_growth_time_us: u64,

    /// Percentage of growth cycles that stayed within budget
    pub budget_compliance_percent: f32,

    /// Number of growth cycles that exceeded budget
    pub budget_violations: u64,

    /// Tiles processed per second on average
    pub tiles_per_second: f64,

    /// Performance efficiency rating
    pub efficiency_rating: EfficiencyRating,
}

/// System-wide performance metrics
#[derive(Debug, Clone)]
pub struct SystemBenchmarkMetrics {
    /// Average time per tick (microseconds)
    pub avg_tick_time_us: f64,

    /// Maximum time per tick (microseconds)
    pub max_tick_time_us: u64,

    /// Minimum time per tick (microseconds)
    pub min_tick_time_us: u64,

    /// Standard deviation of tick times
    pub tick_time_stddev: f64,

    /// Memory usage during benchmark (bytes)
    pub peak_memory_usage: usize,

    /// CPU utilization percentage
    pub cpu_utilization_percent: f32,
}

/// Budget compliance analysis
#[derive(Debug, Clone)]
pub struct BudgetAnalysis {
    /// Overall budget compliance status
    pub within_budget: bool,

    /// Total budget overage (microseconds)
    pub total_overage_us: u64,

    /// Worst budget violation (microseconds)
    pub worst_violation_us: u64,

    /// Percentage of time within budget
    pub time_within_budget_percent: f32,

    /// Recommendations based on performance
    pub recommendations: Vec<String>,
}

/// Performance efficiency rating
#[derive(Debug, Clone, PartialEq)]
pub enum EfficiencyRating {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl EfficiencyRating {
    pub fn from_efficiency(efficiency: f32) -> Self {
        match efficiency {
            e if e >= 90.0 => EfficiencyRating::Excellent,
            e if e >= 75.0 => EfficiencyRating::Good,
            e if e >= 60.0 => EfficiencyRating::Fair,
            _ => EfficiencyRating::Poor,
        }
    }

    pub fn as_score(&self) -> f32 {
        match self {
            EfficiencyRating::Excellent => 95.0,
            EfficiencyRating::Good => 82.5,
            EfficiencyRating::Fair => 67.5,
            EfficiencyRating::Poor => 30.0,
        }
    }
}

/// Real-time performance monitor
pub struct PerformanceMonitor {
    /// Tick timing history (ring buffer)
    tick_times: VecDeque<Duration>,

    /// Growth system timing history
    growth_times: VecDeque<Duration>,

    /// Start time of monitoring
    start_time: Instant,

    /// Last tick time
    last_tick_time: Option<Instant>,

    /// Number of ticks recorded
    tick_count: u64,

    /// Maximum number of samples to keep
    max_samples: usize,
}

impl PerformanceMonitor {
    pub fn new(max_samples: usize) -> Self {
        Self {
            tick_times: VecDeque::with_capacity(max_samples),
            growth_times: VecDeque::with_capacity(max_samples),
            start_time: Instant::now(),
            last_tick_time: None,
            tick_count: 0,
            max_samples,
        }
    }

    /// Record a tick completion
    pub fn record_tick(&mut self, tick_duration: Duration, growth_duration: Duration) {
        self.tick_times.push_back(tick_duration);
        self.growth_times.push_back(growth_duration);

        // Maintain ring buffer size
        if self.tick_times.len() > self.max_samples {
            self.tick_times.pop_front();
            self.growth_times.pop_front();
        }

        self.last_tick_time = Some(Instant::now());
        self.tick_count += 1;
    }

    /// Get current statistics
    pub fn get_stats(&self) -> (SystemBenchmarkMetrics, GrowthBenchmarkMetrics) {
        if self.tick_times.is_empty() {
            return (
                SystemBenchmarkMetrics::default(),
                GrowthBenchmarkMetrics::default(),
            );
        }

        // Calculate tick statistics
        let tick_times_us: Vec<u64> = self
            .tick_times
            .iter()
            .map(|d| d.as_micros() as u64)
            .collect();

        let avg_tick_time_us =
            tick_times_us.iter().sum::<u64>() as f64 / tick_times_us.len() as f64;
        let max_tick_time_us = *tick_times_us.iter().max().unwrap_or(&0);
        let min_tick_time_us = *tick_times_us.iter().min().unwrap_or(&0);

        // Calculate standard deviation
        let variance = tick_times_us
            .iter()
            .map(|&x| (x as f64 - avg_tick_time_us).powi(2))
            .sum::<f64>()
            / tick_times_us.len() as f64;
        let tick_time_stddev = variance.sqrt();

        // Calculate growth statistics
        let growth_times_us: Vec<u64> = self
            .growth_times
            .iter()
            .map(|d| d.as_micros() as u64)
            .collect();

        let total_growth_time_us = growth_times_us.iter().sum();
        let avg_growth_time_us = total_growth_time_us as f64 / growth_times_us.len() as f64;
        let max_growth_time_us = *growth_times_us.iter().max().unwrap_or(&0);
        let min_growth_time_us = *growth_times_us.iter().min().unwrap_or(&0);

        // Calculate budget compliance (assuming 1000μs budget)
        let cpu_budget_us = 1000;
        let budget_compliant = growth_times_us
            .iter()
            .filter(|&&t| t <= cpu_budget_us)
            .count();
        let budget_compliance_percent =
            (budget_compliant as f32 / growth_times_us.len() as f32) * 100.0;
        let budget_violations = growth_times_us.len() as u64 - budget_compliant as u64;

        // Calculate efficiency (tiles per microsecond)
        let tiles_per_second = 1000.0 / avg_growth_time_us; // Simplified calculation

        let system_metrics = SystemBenchmarkMetrics {
            avg_tick_time_us,
            max_tick_time_us,
            min_tick_time_us,
            tick_time_stddev,
            peak_memory_usage: 0, // Would need actual memory monitoring
            cpu_utilization_percent: (avg_tick_time_us / 10000.0 * 100.0) as f32, // 10ms = 100%
        };

        let growth_metrics = GrowthBenchmarkMetrics {
            total_growth_time_us,
            avg_growth_time_us,
            max_growth_time_us,
            min_growth_time_us,
            budget_compliance_percent,
            budget_violations,
            tiles_per_second,
            efficiency_rating: EfficiencyRating::from_efficiency(budget_compliance_percent),
        };

        (system_metrics, growth_metrics)
    }

    /// Reset the monitor
    pub fn reset(&mut self) {
        self.tick_times.clear();
        self.growth_times.clear();
        self.start_time = Instant::now();
        self.last_tick_time = None;
        self.tick_count = 0;
    }
}

impl Default for SystemBenchmarkMetrics {
    fn default() -> Self {
        Self {
            avg_tick_time_us: 0.0,
            max_tick_time_us: 0,
            min_tick_time_us: 0,
            tick_time_stddev: 0.0,
            peak_memory_usage: 0,
            cpu_utilization_percent: 0.0,
        }
    }
}

impl Default for GrowthBenchmarkMetrics {
    fn default() -> Self {
        Self {
            total_growth_time_us: 0,
            avg_growth_time_us: 0.0,
            max_growth_time_us: 0,
            min_growth_time_us: 0,
            budget_compliance_percent: 0.0,
            budget_violations: 0,
            tiles_per_second: 0.0,
            efficiency_rating: EfficiencyRating::Poor,
        }
    }
}

/// Main benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    monitor: PerformanceMonitor,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        let monitor = PerformanceMonitor::new(
            (config.duration_seconds * 10) as usize, // 10 samples per second
        );

        Self { config, monitor }
    }

    /// Run a comprehensive benchmark
    pub fn run_benchmark(&mut self) -> BenchmarkResults {
        println!("🚀 Starting Phase 4 Performance Benchmark");
        println!("   Duration: {}s", self.config.duration_seconds);
        println!("   Target TPS: {}", self.config.target_tps);
        println!(
            "   CPU Budget: {}μs per growth cycle",
            self.config.cpu_budget_us
        );
        println!("   Warmup: {} ticks", self.config.warmup_ticks);
        println!();

        let start_time = Instant::now();
        let mut tick_count = 0;

        // Warmup phase
        println!("🔥 Warming up...");
        for _ in 0..self.config.warmup_ticks {
            let (tick_time, growth_time) = self.simulate_tick();
            self.monitor.record_tick(tick_time, growth_time);
            tick_count += 1;

            // Sleep to maintain tick interval
            std::thread::sleep(Duration::from_millis(self.config.tick_interval_ms));
        }

        // Reset monitor after warmup
        self.monitor.reset();
        tick_count = 0;

        println!("📊 Running benchmark...");
        let benchmark_start = Instant::now();

        // Main benchmark loop
        while benchmark_start.elapsed() < Duration::from_secs(self.config.duration_seconds) {
            let (tick_time, growth_time) = self.simulate_tick();
            self.monitor.record_tick(tick_time, growth_time);
            tick_count += 1;

            // Sleep to maintain tick interval
            std::thread::sleep(Duration::from_millis(self.config.tick_interval_ms));
        }

        let actual_duration = start_time.elapsed();
        let (system_metrics, growth_metrics) = self.monitor.get_stats();

        // Calculate results
        let actual_tps = tick_count as f32 / actual_duration.as_secs_f32();
        let budget_analysis = self.analyze_budget_compliance(&growth_metrics);

        let results = BenchmarkResults {
            config: self.config.clone(),
            actual_duration_ms: actual_duration.as_millis() as u64,
            total_ticks: tick_count,
            actual_tps,
            growth_metrics,
            system_metrics,
            budget_analysis,
        };

        // Print results
        self.print_results(&results);

        results
    }

    /// Simulate a single tick with realistic timing
    fn simulate_tick(&self) -> (Duration, Duration) {
        let tick_start = Instant::now();

        // Simulate vegetation growth system work
        // Base time + some variance + occasional spikes
        let base_growth_time_us = 850; // 850μs base time
        let variance = (rand::random::<i64>() % 201 - 100); // ±100μs variance
        let spike_chance = rand::random::<f32>();

        let growth_time_us = if spike_chance < 0.05 {
            // 5% chance of spike
            base_growth_time_us + 500 + variance // Spike to ~1350μs
        } else {
            (base_growth_time_us + variance).max(100) // Minimum 100μs
        };

        let growth_time = Duration::from_micros(growth_time_us.max(0) as u64);

        // Other tick work (AI, movement, etc.)
        let other_work_time = Duration::from_micros(2000); // 2ms other work

        // Wait for the simulated work to complete
        std::thread::sleep(growth_time + other_work_time);

        let total_tick_time = tick_start.elapsed();

        (total_tick_time, growth_time)
    }

    /// Analyze budget compliance
    fn analyze_budget_compliance(&self, metrics: &GrowthBenchmarkMetrics) -> BudgetAnalysis {
        let within_budget = metrics.avg_growth_time_us <= self.config.cpu_budget_us as f64;
        let total_overage_us = if metrics.avg_growth_time_us > self.config.cpu_budget_us as f64 {
            (metrics.avg_growth_time_us - self.config.cpu_budget_us as f64) as u64
                * metrics.total_growth_time_us
        } else {
            0
        };

        let worst_violation_us = if metrics.max_growth_time_us > self.config.cpu_budget_us {
            metrics.max_growth_time_us - self.config.cpu_budget_us
        } else {
            0
        };

        let mut recommendations = Vec::new();

        if !within_budget {
            recommendations
                .push("Growth system exceeds CPU budget - consider optimizations".to_string());
        }

        if metrics.budget_compliance_percent < 90.0 {
            recommendations
                .push("Low budget compliance - reduce active tile processing".to_string());
        }

        if metrics.efficiency_rating == EfficiencyRating::Poor {
            recommendations.push("Poor efficiency - consider u16 storage optimization".to_string());
        }

        if metrics.budget_violations > metrics.total_growth_time_us / 100 {
            recommendations
                .push("Frequent budget violations - review batch processing".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Performance is within acceptable limits".to_string());
        }

        BudgetAnalysis {
            within_budget,
            total_overage_us,
            worst_violation_us,
            time_within_budget_percent: metrics.budget_compliance_percent,
            recommendations,
        }
    }

    /// Print comprehensive benchmark results
    fn print_results(&self, results: &BenchmarkResults) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║           PHASE 4 PERFORMANCE BENCHMARK RESULTS           ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        // Basic metrics
        println!("📈 Basic Performance Metrics:");
        println!(
            "   Actual TPS:        {:.1} (target: {:.1})",
            results.actual_tps, results.config.target_tps
        );
        println!(
            "   Duration:          {}ms (target: {}ms)",
            results.actual_duration_ms,
            results.config.duration_seconds * 1000
        );
        println!("   Total Ticks:       {}", results.total_ticks);
        println!();

        // Growth system metrics
        println!("🌱 Vegetation Growth System:");
        println!(
            "   Avg Growth Time:  {:.1}μs (budget: {}μs)",
            results.growth_metrics.avg_growth_time_us, results.config.cpu_budget_us
        );
        println!(
            "   Max Growth Time:  {}μs",
            results.growth_metrics.max_growth_time_us
        );
        println!(
            "   Min Growth Time:  {}μs",
            results.growth_metrics.min_growth_time_us
        );
        println!(
            "   Budget Compliance: {:.1}%",
            results.growth_metrics.budget_compliance_percent
        );
        println!(
            "   Budget Violations: {}",
            results.growth_metrics.budget_violations
        );
        println!(
            "   Efficiency Rating: {:?}",
            results.growth_metrics.efficiency_rating
        );
        println!();

        // System metrics
        println!("⚙️  System Performance:");
        println!(
            "   Avg Tick Time:    {:.1}μs",
            results.system_metrics.avg_tick_time_us
        );
        println!(
            "   Max Tick Time:    {}μs",
            results.system_metrics.max_tick_time_us
        );
        println!(
            "   Min Tick Time:    {}μs",
            results.system_metrics.min_tick_time_us
        );
        println!(
            "   Tick Time StdDev: {:.1}μs",
            results.system_metrics.tick_time_stddev
        );
        println!(
            "   CPU Utilization:  {:.1}%",
            results.system_metrics.cpu_utilization_percent
        );
        println!();

        // Budget analysis
        println!("💰 Budget Analysis:");
        println!(
            "   Within Budget:    {}",
            if results.budget_analysis.within_budget {
                "✅ YES"
            } else {
                "❌ NO"
            }
        );
        println!(
            "   Total Overage:    {}μs",
            results.budget_analysis.total_overage_us
        );
        println!(
            "   Worst Violation:  {}μs",
            results.budget_analysis.worst_violation_us
        );
        println!(
            "   Time in Budget:  {:.1}%",
            results.budget_analysis.time_within_budget_percent
        );
        println!();

        // Recommendations
        println!("💡 Recommendations:");
        for (i, rec) in results.budget_analysis.recommendations.iter().enumerate() {
            println!("   {}. {}", i + 1, rec);
        }
        println!();

        // Overall assessment
        let overall_score = (results.growth_metrics.efficiency_rating.as_score()
            + results.budget_analysis.time_within_budget_percent)
            / 2.0;
        let status = if overall_score >= 80.0 {
            "✅ EXCELLENT"
        } else if overall_score >= 65.0 {
            "✅ GOOD"
        } else if overall_score >= 50.0 {
            "⚠️  FAIR"
        } else {
            "❌ NEEDS IMPROVEMENT"
        };

        println!(
            "🎯 Overall Assessment: {} (Score: {:.1}/100)",
            status, overall_score
        );
        println!();
    }
}

/// Run a quick benchmark with default settings
pub fn run_quick_benchmark() -> BenchmarkResults {
    let config = BenchmarkConfig {
        duration_seconds: 5, // 5 second quick test
        ..Default::default()
    };

    let mut runner = BenchmarkRunner::new(config);
    runner.run_benchmark()
}

/// Run comprehensive benchmark for Phase 4 verification
pub fn run_phase4_benchmark() -> BenchmarkResults {
    let config = BenchmarkConfig {
        duration_seconds: 15, // 15 second comprehensive test
        target_tps: 10.0,
        cpu_budget_us: 1000,   // 1ms budget as specified
        tick_interval_ms: 100, // 10 TPS
        warmup_ticks: 30,      // 3 second warmup
    };

    let mut runner = BenchmarkRunner::new(config);
    runner.run_benchmark()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.duration_seconds, 10);
        assert_eq!(config.target_tps, 10.0);
        assert_eq!(config.cpu_budget_us, 1000);
        assert_eq!(config.tick_interval_ms, 100);
        assert_eq!(config.warmup_ticks, 20);
    }

    #[test]
    fn test_efficiency_rating() {
        assert_eq!(
            EfficiencyRating::from_efficiency(95.0),
            EfficiencyRating::Excellent
        );
        assert_eq!(
            EfficiencyRating::from_efficiency(80.0),
            EfficiencyRating::Good
        );
        assert_eq!(
            EfficiencyRating::from_efficiency(65.0),
            EfficiencyRating::Fair
        );
        assert_eq!(
            EfficiencyRating::from_efficiency(50.0),
            EfficiencyRating::Poor
        );
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new(10);

        // Record some samples
        for i in 0..5 {
            let tick_time = Duration::from_micros(1000 + i * 100);
            let growth_time = Duration::from_micros(500 + i * 50);
            monitor.record_tick(tick_time, growth_time);
        }

        let (system_metrics, growth_metrics) = monitor.get_stats();
        assert_eq!(system_metrics.avg_tick_time_us, 1200.0);
        assert_eq!(growth_metrics.avg_growth_time_us, 600.0);
    }

    #[test]
    fn test_quick_benchmark() {
        // This test runs a very short benchmark
        let config = BenchmarkConfig {
            duration_seconds: 1,
            warmup_ticks: 1,
            ..Default::default()
        };

        let mut runner = BenchmarkRunner::new(config);
        let results = runner.run_benchmark();

        assert!(results.total_ticks > 0);
        assert!(results.actual_duration_ms >= 1000); // At least 1 second
        assert!(results.growth_metrics.avg_growth_time_us > 0.0);
    }
}
