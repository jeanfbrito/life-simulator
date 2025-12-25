use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use crate::cli::{Args, MetricsOptions, AnomalyOptions, SummaryOptions};
use crate::anomaly;
use crate::output;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TPSMetric {
    pub timestamp: String,
    pub tps: f64,
    pub dt_ms: f64,
    pub entities: u32,
    pub chunks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCount {
    pub entity_type: String,
    pub count: u32,
}

pub struct LogParser {
    metrics: Vec<TPSMetric>,
    entity_counts: Vec<EntityCount>,
    log_lines: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            metrics: Vec::new(),
            entity_counts: Vec::new(),
            log_lines: Vec::new(),
        }
    }

    pub fn parse_stream<R: Read>(
        &mut self,
        reader: BufReader<R>,
        args: &Args,
    ) -> Result<(), Box<dyn std::error::Error>> {

        for line in reader.lines() {
            let line = line?;

            // Filter by log level if specified
            if !matches_log_level(&line, &args.log_level) {
                continue;
            }

            self.log_lines.push(line.clone());

            // Extract TPS metrics
            if let Some(metric) = extract_tps_metrics(&line) {
                self.metrics.push(metric);
            }

            // Extract entity counts
            if let Some(entity) = extract_entity_count(&line) {
                self.entity_counts.push(entity);
            }
        }

        Ok(())
    }

    pub fn output_metrics(&self, opts: &MetricsOptions) -> Result<(), Box<dyn std::error::Error>> {
        let mut metrics = self.metrics.clone();

        // Apply tail filtering if specified
        if let Some(tail) = opts.tail {
            metrics = tail_lines(
                &metrics
                    .iter()
                    .map(|m| format!("{}", m.tps))
                    .collect::<Vec<_>>(),
                tail,
            )
            .iter()
            .zip(&metrics)
            .map(|(_, m)| m.clone())
            .collect();
        }

        match &opts.metric {
            Some(metric_type) if metric_type == "tps" => {
                for m in &metrics {
                    println!("{}: {} TPS (entities: {}, chunks: {})",
                        m.timestamp, m.tps, m.entities, m.chunks);
                }
            }
            Some(metric_type) if metric_type == "entities" => {
                for m in &metrics {
                    println!("{}: {} entities", m.timestamp, m.entities);
                }
            }
            _ => {
                for m in &metrics {
                    println!("{}", serde_json::to_string(&m)?);
                }
            }
        }

        Ok(())
    }

    pub fn detect_anomalies(&self, opts: &AnomalyOptions) -> Result<(), Box<dyn std::error::Error>> {
        match opts.anomaly_type.as_str() {
            "stuck-entity" => {
                println!("Detecting stuck entities with threshold: {}", opts.threshold);
                // This would need entity position tracking in logs
                println!("No stuck entities detected in current log format");
            }
            "tps-drop" => {
                let tps_values: Vec<f64> = self.metrics.iter().map(|m| m.tps).collect();
                let anomalies = anomaly::detect_tps_drops(&tps_values, opts.threshold as f64);

                if anomalies.is_empty() {
                    println!("No TPS drops detected below threshold: {}", opts.threshold);
                } else {
                    println!("Found {} TPS drop anomalies:", anomalies.len());
                    for idx in anomalies {
                        if idx < self.metrics.len() {
                            let m = &self.metrics[idx];
                            println!("  [{}] {} TPS at {}", idx, m.tps, m.timestamp);
                        }
                    }
                }
            }
            _ => {
                println!("Unknown anomaly type: {}", opts.anomaly_type);
            }
        }

        Ok(())
    }

    pub fn output_summary(&self, opts: &SummaryOptions) -> Result<(), Box<dyn std::error::Error>> {
        let metrics = &self.metrics;

        if metrics.is_empty() {
            println!("No metrics found in log");
            return Ok(());
        }

        let tps_values: Vec<f64> = metrics.iter().map(|m| m.tps).collect();
        let avg_tps = tps_values.iter().sum::<f64>() / tps_values.len() as f64;
        let min_tps = tps_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_tps = tps_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let entity_values: Vec<u32> = metrics.iter().map(|m| m.entities).collect();
        let avg_entities = entity_values.iter().sum::<u32>() as f64 / entity_values.len() as f64;

        println!("=== Log Summary ===");
        println!("Total metrics: {}", metrics.len());
        println!("Time range: {} to {}", metrics[0].timestamp, metrics[metrics.len() - 1].timestamp);
        println!();
        println!("TPS Statistics:");
        println!("  Average: {:.2}", avg_tps);
        println!("  Min: {:.2}", min_tps);
        println!("  Max: {:.2}", max_tps);
        println!();
        println!("Entity Statistics:");
        println!("  Average count: {:.0}", avg_entities);
        println!("  Total recorded: {}", self.entity_counts.len());
        println!();

        if opts.export_json {
            let json = output::metrics_to_json(&metrics);
            println!("JSON Export:\n{}", json);
        }

        if opts.export_csv {
            let csv = output::metrics_to_csv(&metrics);
            println!("CSV Export:\n{}", csv);
        }

        Ok(())
    }
}

/// Extract TPS metrics from a log line
pub fn extract_tps_metrics(line: &str) -> Option<TPSMetric> {
    // Pattern: TICK PERFORMANCE: TPS=59.8 dt=16.67ms entities=1234 chunks=42
    let re = Regex::new(
        r"TICK PERFORMANCE:.*TPS=([0-9.]+).*dt=([0-9.]+)ms.*entities=(\d+).*chunks=(\d+)",
    )
    .ok()?;

    if let Some(caps) = re.captures(line) {
        let tps: f64 = caps.get(1)?.as_str().parse().ok()?;
        let dt_ms: f64 = caps.get(2)?.as_str().parse().ok()?;
        let entities: u32 = caps.get(3)?.as_str().parse().ok()?;
        let chunks: u32 = caps.get(4)?.as_str().parse().ok()?;

        // Extract timestamp if available
        let timestamp_re = Regex::new(r"\[([^\]]+)\]").ok()?;
        let timestamp = timestamp_re
            .captures(line)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        return Some(TPSMetric {
            timestamp,
            tps,
            dt_ms,
            entities,
            chunks,
        });
    }

    None
}

/// Extract entity count from spawn messages
pub fn extract_entity_count(line: &str) -> Option<EntityCount> {
    // Pattern: Entity spawned: type=Deer count=150
    let re = Regex::new(r"Entity spawned:.*type=(\w+).*count=(\d+)").ok()?;

    if let Some(caps) = re.captures(line) {
        let entity_type = caps.get(1)?.as_str().to_string();
        let count: u32 = caps.get(2)?.as_str().parse().ok()?;

        return Some(EntityCount { entity_type, count });
    }

    None
}

/// Filter log line by level
pub fn matches_log_level(line: &str, level: &str) -> bool {
    if level == "all" {
        return true;
    }

    match level {
        "error" => line.contains("[ERROR]") || line.contains("ERROR"),
        "warn" => {
            line.contains("[WARN]")
                || line.contains("WARN")
                || line.contains("[WARNING]")
                || line.contains("WARNING")
        }
        "info" => {
            line.contains("[INFO]")
                || line.contains("INFO")
                || line.contains("[debug]")
                || line.contains("DEBUG")
        }
        _ => true,
    }
}

/// Get last N lines from a vector
pub fn tail_lines<T: Clone>(lines: &[T], n: usize) -> Vec<T> {
    if lines.len() <= n {
        lines.to_vec()
    } else {
        lines[lines.len() - n..].to_vec()
    }
}
