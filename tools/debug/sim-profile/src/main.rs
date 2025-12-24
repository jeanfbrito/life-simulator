use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Performance analysis tool for life-simulator TickProfiler output
#[derive(Parser)]
#[command(name = "sim-profile")]
#[command(about = "Analyze TickProfiler performance logs from life-simulator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show top N bottlenecks from a log file
    Top {
        /// Number of systems to show
        #[arg(long, default_value_t = 5)]
        n: usize,

        /// Log file to analyze
        logfile: PathBuf,
    },

    /// Detect regressions against a baseline JSON file
    Regression {
        /// Baseline JSON file with expected timings
        baseline: PathBuf,

        /// Log file to analyze
        logfile: PathBuf,

        /// Threshold percentage for regression detection (default: 10%)
        #[arg(long, default_value_t = 10.0)]
        threshold: f64,
    },

    /// Show performance trends for a specific system over time
    Trend {
        /// System name to track
        #[arg(long)]
        system: Option<String>,

        /// Log file to analyze
        logfile: PathBuf,

        /// Generate ASCII bar chart visualization
        #[arg(long)]
        chart: bool,
    },

    /// Export performance data to JSON format
    Export {
        /// Log file to analyze
        logfile: PathBuf,

        /// Output JSON file
        output: PathBuf,
    },
}

/// Represents a single tick's performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickData {
    pub tick: u64,
    pub systems: HashMap<String, SystemMetrics>,
    pub total_ms: f64,
}

/// Metrics for a single system in a tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub ms: f64,
    pub percentage: f64,
}

/// Represents aggregated performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub system_name: String,
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub median_ms: f64,
    pub stddev_ms: f64,
    pub avg_percentage: f64,
    pub sample_count: usize,
}

/// Parser for TickProfiler output using state machine approach
pub struct TickProfilerParser {
    tick_regex: Regex,
    system_regex: Regex,
    #[allow(dead_code)]
    avg_total_regex: Regex,
}

impl TickProfilerParser {
    pub fn new() -> Self {
        Self {
            // Matches: "🔧 TICK PERFORMANCE - Tick N | Total: X.Xms"
            tick_regex: Regex::new(
                r"TICK PERFORMANCE - Tick (\d+) \| Total: ([\d.]+)ms"
            ).unwrap(),

            // Matches: "├── system_name: X.Xms (Y%)"
            // The system_name is left-aligned in a 15-char field, so there may be trailing spaces
            system_regex: Regex::new(
                r"├──\s+([a-z_]+)\s*:\s+([\d.]+)ms\s+\(\s*(\d+)%\)"
            ).unwrap(),

            // Matches: "└── AVG TOTAL: X.Xms over N systems"
            avg_total_regex: Regex::new(
                r"└── AVG TOTAL: ([\d.]+)ms over (\d+) systems"
            ).unwrap(),
        }
    }

    /// Parse a log file containing TickProfiler output
    pub fn parse_log(&self, content: &str) -> Result<Vec<TickData>, String> {
        let mut ticks = Vec::new();
        let mut current_tick: Option<TickData> = None;

        for line in content.lines() {
            // Check for tick header
            if let Some(caps) = self.tick_regex.captures(line) {
                // Save previous tick if exists
                if let Some(tick) = current_tick.take() {
                    ticks.push(tick);
                }

                let tick = caps.get(1)
                    .ok_or("Missing tick number")?
                    .as_str()
                    .parse::<u64>()
                    .map_err(|e| format!("Failed to parse tick number: {}", e))?;

                let total_ms = caps.get(2)
                    .ok_or("Missing total ms")?
                    .as_str()
                    .parse::<f64>()
                    .map_err(|e| format!("Failed to parse total ms: {}", e))?;

                current_tick = Some(TickData {
                    tick,
                    systems: HashMap::new(),
                    total_ms,
                });
            }
            // Check for system line
            else if let Some(caps) = self.system_regex.captures(line) {
                if let Some(ref mut tick_data) = current_tick {
                    let system_name = caps.get(1)
                        .ok_or("Missing system name")?
                        .as_str()
                        .to_string();

                    let ms = caps.get(2)
                        .ok_or("Missing ms value")?
                        .as_str()
                        .parse::<f64>()
                        .map_err(|e| format!("Failed to parse ms: {}", e))?;

                    let percentage = caps.get(3)
                        .ok_or("Missing percentage")?
                        .as_str()
                        .parse::<f64>()
                        .map_err(|e| format!("Failed to parse percentage: {}", e))?;

                    tick_data.systems.insert(system_name, SystemMetrics { ms, percentage });
                }
            }
        }

        // Don't forget the last tick
        if let Some(tick) = current_tick.take() {
            ticks.push(tick);
        }

        if ticks.is_empty() {
            return Err("No tick performance data found in log file".to_string());
        }

        Ok(ticks)
    }
}

impl Default for TickProfilerParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzer for performance data
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// Get top N bottleneck systems by average time
    pub fn top_bottlenecks(ticks: &[TickData], n: usize) -> Vec<PerformanceStats> {
        let mut system_data: HashMap<String, Vec<f64>> = HashMap::new();

        for tick in ticks {
            for (system_name, metrics) in &tick.systems {
                system_data.entry(system_name.clone())
                    .or_insert_with(Vec::new)
                    .push(metrics.ms);
            }
        }

        let mut stats = Vec::new();

        for (system_name, measurements) in system_data {
            if measurements.is_empty() {
                continue;
            }

            let avg_ms = measurements.iter().sum::<f64>() / measurements.len() as f64;
            let min_ms = measurements.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_ms = measurements.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            let mut sorted = measurements.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median_ms = if sorted.len() % 2 == 0 {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            };

            let variance = measurements.iter()
                .map(|x| (x - avg_ms).powi(2))
                .sum::<f64>() / measurements.len() as f64;
            let stddev_ms = variance.sqrt();

            let avg_percentage = if let Some(first_tick) = ticks.first() {
                if let Some(sys_metrics) = first_tick.systems.get(&system_name) {
                    sys_metrics.percentage
                } else {
                    0.0
                }
            } else {
                0.0
            };

            stats.push(PerformanceStats {
                system_name,
                avg_ms,
                min_ms,
                max_ms,
                median_ms,
                stddev_ms,
                avg_percentage,
                sample_count: measurements.len(),
            });
        }

        stats.sort_by(|a, b| b.avg_ms.partial_cmp(&a.avg_ms).unwrap());
        stats.truncate(n);
        stats
    }

    /// Detect regressions against baseline
    pub fn detect_regressions(
        baseline: &HashMap<String, f64>,
        current_stats: &[PerformanceStats],
        threshold: f64,
    ) -> Vec<(String, f64, f64)> {
        let mut regressions = Vec::new();

        for stat in current_stats {
            if let Some(&baseline_ms) = baseline.get(&stat.system_name) {
                let change_percent = ((stat.avg_ms - baseline_ms) / baseline_ms) * 100.0;
                if change_percent > threshold {
                    regressions.push((
                        stat.system_name.clone(),
                        baseline_ms,
                        stat.avg_ms,
                    ));
                }
            }
        }

        regressions.sort_by(|a, b| {
            // Sort by absolute timing (worst first)
            b.2.partial_cmp(&a.2).unwrap()
        });

        regressions
    }

    /// Extract trend data for a specific system
    pub fn system_trend(ticks: &[TickData], system_name: &str) -> Vec<(u64, f64)> {
        ticks.iter()
            .filter_map(|tick| {
                tick.systems.get(system_name)
                    .map(|metrics| (tick.tick, metrics.ms))
            })
            .collect()
    }

    /// Generate ASCII bar chart for trend data
    pub fn generate_bar_chart(trend: &[(u64, f64)], max_width: usize) -> String {
        if trend.is_empty() {
            return "No data to display".to_string();
        }

        let max_value = trend.iter()
            .map(|(_, value)| *value)
            .fold(0.0, f64::max);

        let mut chart = String::new();
        chart.push_str(&format!("Performance Trend (max: {:.1}ms)\n", max_value));
        chart.push_str(&"─".repeat(max_width + 20));
        chart.push('\n');

        for (tick, value) in trend {
            let bar_width = if max_value > 0.0 {
                ((value / max_value) * max_width as f64) as usize
            } else {
                0
            };

            chart.push_str(&format!("Tick {:>6} │ {:<width$} {:.1}ms\n",
                tick,
                "█".repeat(bar_width),
                value,
                width = max_width
            ));
        }

        chart.push_str(&"─".repeat(max_width + 20));
        chart.push('\n');
        chart
    }
}

/// Print formatted performance statistics
fn print_stats(stats: &[PerformanceStats]) {
    if stats.is_empty() {
        println!("No performance data found");
        return;
    }

    println!("\n{:<20} {:>10} {:>10} {:>10} {:>10} {:>8}",
        "System", "Avg (ms)", "Min (ms)", "Max (ms)", "Median (ms)", "Stddev");
    println!("{}", "─".repeat(90));

    for stat in stats {
        println!("{:<20} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>8.2}",
            stat.system_name,
            stat.avg_ms,
            stat.min_ms,
            stat.max_ms,
            stat.median_ms,
            stat.stddev_ms
        );
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Top { n, logfile } => {
            let content = fs::read_to_string(&logfile)?;
            let parser = TickProfilerParser::new();
            let ticks = parser.parse_log(&content)?;

            let stats = PerformanceAnalyzer::top_bottlenecks(&ticks, n);
            println!("\n=== Top {} Performance Bottlenecks ===", n);
            print_stats(&stats);
        }

        Commands::Regression { baseline, logfile, threshold } => {
            let baseline_json = fs::read_to_string(&baseline)?;
            let baseline_data: HashMap<String, f64> = serde_json::from_str(&baseline_json)?;

            let log_content = fs::read_to_string(&logfile)?;
            let parser = TickProfilerParser::new();
            let ticks = parser.parse_log(&log_content)?;

            let stats = PerformanceAnalyzer::top_bottlenecks(&ticks, 50);
            let regressions = PerformanceAnalyzer::detect_regressions(&baseline_data, &stats, threshold);

            if regressions.is_empty() {
                println!("\nNo regressions detected (threshold: {}%)", threshold);
            } else {
                println!("\n=== Performance Regressions (threshold: {}%) ===", threshold);
                println!("{:<20} {:>12} {:>12} {:>12}",
                    "System", "Baseline (ms)", "Current (ms)", "Change (%)");
                println!("{}", "─".repeat(60));

                for (name, baseline_ms, current_ms) in &regressions {
                    let change_percent = ((current_ms - baseline_ms) / baseline_ms) * 100.0;
                    println!("{:<20} {:>12.2} {:>12.2} {:>11.1}%",
                        name, baseline_ms, current_ms, change_percent);
                }
            }
            println!();
        }

        Commands::Trend { system, logfile, chart } => {
            let content = fs::read_to_string(&logfile)?;
            let parser = TickProfilerParser::new();
            let ticks = parser.parse_log(&content)?;

            if let Some(system_name) = system {
                let trend = PerformanceAnalyzer::system_trend(&ticks, &system_name);

                if trend.is_empty() {
                    println!("No data found for system: {}", system_name);
                } else {
                    if chart {
                        println!("\n=== Performance Trend for {} ===", system_name);
                        println!("{}", PerformanceAnalyzer::generate_bar_chart(&trend, 40));
                    } else {
                        println!("\n=== Performance Trend for {} ===", system_name);
                        println!("{:<10} {:<15}", "Tick", "Time (ms)");
                        println!("{}", "─".repeat(25));
                        for (tick, ms) in &trend {
                            println!("{:<10} {:<15.2}", tick, ms);
                        }
                    }
                }
            } else {
                // Show all systems if none specified
                let stats = PerformanceAnalyzer::top_bottlenecks(&ticks, 10);
                println!("\n=== Top 10 Systems by Complexity ===");
                print_stats(&stats);
            }
            println!();
        }

        Commands::Export { logfile, output } => {
            let content = fs::read_to_string(&logfile)?;
            let parser = TickProfilerParser::new();
            let ticks = parser.parse_log(&content)?;

            let json = serde_json::to_string_pretty(&ticks)?;
            fs::write(&output, json)?;

            println!("Exported {} ticks to {}", ticks.len(), output.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tick_header() {
        let parser = TickProfilerParser::new();
        let log_content = r#"
🔧 TICK PERFORMANCE - Tick 50 | Total: 5.2ms
├── ai_planner:      2.1ms ( 40%)
├── movement:        1.5ms ( 28%)
├── vegetation:      1.2ms ( 23%)
└── AVG TOTAL: 4.8ms over 3 systems
"#;

        let result = parser.parse_log(log_content);
        assert!(result.is_ok());

        let ticks = result.unwrap();
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].tick, 50);
        assert_eq!(ticks[0].total_ms, 5.2);
    }

    #[test]
    fn test_parse_system_metrics() {
        let parser = TickProfilerParser::new();
        let log_content = r#"
🔧 TICK PERFORMANCE - Tick 50 | Total: 5.2ms
├── ai_planner:      2.1ms ( 40%)
├── movement:        1.5ms ( 28%)
├── vegetation:      1.2ms ( 23%)
└── AVG TOTAL: 4.8ms over 3 systems
"#;

        let ticks = parser.parse_log(log_content).unwrap();
        assert_eq!(ticks[0].systems.len(), 3);

        assert!(ticks[0].systems.contains_key("ai_planner"));
        let ai = &ticks[0].systems["ai_planner"];
        assert_eq!(ai.ms, 2.1);
        assert_eq!(ai.percentage, 40.0);

        assert!(ticks[0].systems.contains_key("movement"));
        let mov = &ticks[0].systems["movement"];
        assert_eq!(mov.ms, 1.5);
        assert_eq!(mov.percentage, 28.0);
    }

    #[test]
    fn test_parse_multiple_ticks() {
        let parser = TickProfilerParser::new();
        let log_content = r#"
🔧 TICK PERFORMANCE - Tick 50 | Total: 5.2ms
├── ai_planner:      2.1ms ( 40%)
└── AVG TOTAL: 2.1ms over 1 systems

🔧 TICK PERFORMANCE - Tick 100 | Total: 6.1ms
├── ai_planner:      3.0ms ( 49%)
└── AVG TOTAL: 3.0ms over 1 systems

🔧 TICK PERFORMANCE - Tick 150 | Total: 4.8ms
├── ai_planner:      2.0ms ( 41%)
└── AVG TOTAL: 2.0ms over 1 systems
"#;

        let ticks = parser.parse_log(log_content).unwrap();
        assert_eq!(ticks.len(), 3);
        assert_eq!(ticks[0].tick, 50);
        assert_eq!(ticks[1].tick, 100);
        assert_eq!(ticks[2].tick, 150);

        assert_eq!(ticks[0].systems["ai_planner"].ms, 2.1);
        assert_eq!(ticks[1].systems["ai_planner"].ms, 3.0);
        assert_eq!(ticks[2].systems["ai_planner"].ms, 2.0);
    }

    #[test]
    fn test_top_bottlenecks() {
        let parser = TickProfilerParser::new();
        let log_content = r#"
🔧 TICK PERFORMANCE - Tick 50 | Total: 10.0ms
├── vegetation:      5.0ms ( 50%)
├── ai_planner:      3.0ms ( 30%)
├── movement:        2.0ms ( 20%)
└── AVG TOTAL: 10.0ms over 3 systems

🔧 TICK PERFORMANCE - Tick 100 | Total: 11.0ms
├── vegetation:      5.5ms ( 50%)
├── ai_planner:      3.3ms ( 30%)
├── movement:        2.2ms ( 20%)
└── AVG TOTAL: 11.0ms over 3 systems
"#;

        let ticks = parser.parse_log(log_content).unwrap();
        let stats = PerformanceAnalyzer::top_bottlenecks(&ticks, 2);

        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].system_name, "vegetation");
        assert!(stats[0].avg_ms > stats[1].avg_ms);
    }

    #[test]
    fn test_regression_detection() {
        let mut baseline = HashMap::new();
        baseline.insert("ai_planner".to_string(), 2.0);
        baseline.insert("vegetation".to_string(), 3.0);

        let stats = vec![
            PerformanceStats {
                system_name: "ai_planner".to_string(),
                avg_ms: 2.5,
                min_ms: 2.0,
                max_ms: 3.0,
                median_ms: 2.5,
                stddev_ms: 0.3,
                avg_percentage: 45.0,
                sample_count: 10,
            },
            PerformanceStats {
                system_name: "vegetation".to_string(),
                avg_ms: 3.0,
                min_ms: 2.8,
                max_ms: 3.2,
                median_ms: 3.0,
                stddev_ms: 0.1,
                avg_percentage: 50.0,
                sample_count: 10,
            },
        ];

        let regressions = PerformanceAnalyzer::detect_regressions(&baseline, &stats, 10.0);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].0, "ai_planner");
    }

    #[test]
    fn test_system_trend() {
        let parser = TickProfilerParser::new();
        let log_content = r#"
🔧 TICK PERFORMANCE - Tick 50 | Total: 5.0ms
├── ai_planner:      2.0ms ( 40%)
└── AVG TOTAL: 2.0ms over 1 systems

🔧 TICK PERFORMANCE - Tick 100 | Total: 6.0ms
├── ai_planner:      3.0ms ( 50%)
└── AVG TOTAL: 3.0ms over 1 systems

🔧 TICK PERFORMANCE - Tick 150 | Total: 5.5ms
├── ai_planner:      2.5ms ( 45%)
└── AVG TOTAL: 2.5ms over 1 systems
"#;

        let ticks = parser.parse_log(log_content).unwrap();
        let trend = PerformanceAnalyzer::system_trend(&ticks, "ai_planner");

        assert_eq!(trend.len(), 3);
        assert_eq!(trend[0], (50, 2.0));
        assert_eq!(trend[1], (100, 3.0));
        assert_eq!(trend[2], (150, 2.5));
    }

    #[test]
    fn test_generate_bar_chart() {
        let trend = vec![(50, 2.0), (100, 4.0), (150, 3.0)];
        let chart = PerformanceAnalyzer::generate_bar_chart(&trend, 20);

        assert!(chart.contains("Tick"));
        assert!(chart.contains("4.0ms"));
        assert!(chart.contains("█"));
    }

    #[test]
    fn test_empty_log() {
        let parser = TickProfilerParser::new();
        let result = parser.parse_log("");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No tick performance data found"));
    }

    #[test]
    fn test_statistical_calculations() {
        let stats = vec![
            PerformanceStats {
                system_name: "test".to_string(),
                avg_ms: 5.0,
                min_ms: 2.0,
                max_ms: 8.0,
                median_ms: 5.0,
                stddev_ms: 1.5,
                avg_percentage: 50.0,
                sample_count: 10,
            },
        ];

        assert_eq!(stats[0].avg_ms, 5.0);
        assert_eq!(stats[0].min_ms, 2.0);
        assert_eq!(stats[0].max_ms, 8.0);
    }
}
