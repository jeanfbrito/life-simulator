use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "sim-logparse")]
#[command(about = "Log parser for Life Simulator performance analysis", long_about = None)]
pub struct Args {
    /// Path to log file
    pub logfile: String,

    /// Log level filter (all, info, warn, error)
    #[arg(short, long, default_value = "all")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Extract and display metrics
    #[command(alias = "m")]
    Metrics(MetricsOptions),

    /// Detect anomalies in logs
    #[command(alias = "a")]
    Anomaly(AnomalyOptions),

    /// Display log summary
    #[command(alias = "s")]
    Summary(SummaryOptions),
}

#[derive(Parser, Debug)]
pub struct MetricsOptions {
    /// Show only last N lines
    #[arg(long)]
    pub tail: Option<usize>,

    /// Metric to display (tps, entities, all)
    #[arg(long)]
    pub metric: Option<String>,
}

#[derive(Parser, Debug)]
pub struct AnomalyOptions {
    /// Type of anomaly to detect (stuck-entity, tps-drop)
    #[arg(long)]
    pub anomaly_type: String,

    /// Threshold for anomaly detection
    #[arg(long, default_value = "10")]
    pub threshold: u32,
}

#[derive(Parser, Debug)]
pub struct SummaryOptions {
    /// Export as JSON
    #[arg(long)]
    pub export_json: bool,

    /// Export as CSV
    #[arg(long)]
    pub export_csv: bool,

    /// Last N minutes (format: 5m, 1h)
    #[arg(long)]
    pub last: Option<String>,
}
