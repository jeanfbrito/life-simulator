mod parser;
mod cli;
mod anomaly;
mod output;

use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use cli::Args;
use parser::LogParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = File::open(&args.logfile)?;
    let reader = BufReader::new(file);

    let mut parser = LogParser::new();
    parser.parse_stream(reader, &args)?;

    match args.command {
        cli::Command::Metrics(opts) => {
            parser.output_metrics(&opts)?;
        }
        cli::Command::Anomaly(opts) => {
            parser.detect_anomalies(&opts)?;
        }
        cli::Command::Summary(opts) => {
            parser.output_summary(&opts)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_tps_extraction_from_log_line() {
        let log_line = "[2025-12-24 10:30:45] TICK PERFORMANCE: TPS=59.8 dt=16.67ms entities=1234 chunks=42";
        let metrics = parser::extract_tps_metrics(log_line);

        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.tps, 59.8);
        assert_eq!(m.entities, 1234);
        assert_eq!(m.chunks, 42);
    }

    #[test]
    fn test_tps_extraction_no_match() {
        let log_line = "Some random log line without metrics";
        let metrics = parser::extract_tps_metrics(log_line);
        assert!(metrics.is_none());
    }

    #[test]
    fn test_entity_count_from_spawn_message() {
        let log_line = "Entity spawned: type=Deer count=150 population=1523";
        let count = parser::extract_entity_count(log_line);

        assert!(count.is_some());
        let c = count.unwrap();
        assert_eq!(c.entity_type, "Deer");
        assert_eq!(c.count, 150);
    }

    #[test]
    fn test_log_level_filtering() {
        let lines = vec![
            "[INFO] This is info",
            "[WARN] This is warning",
            "[ERROR] This is error",
            "[DEBUG] This is debug",
        ];

        let filtered: Vec<_> = lines
            .iter()
            .filter(|line| parser::matches_log_level(line, "warn"))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].contains("WARN"));
    }

    #[test]
    fn test_tps_drop_anomaly_detection() {
        let tps_values = vec![60.0, 59.5, 8.3, 9.2, 58.0];
        let anomalies = anomaly::detect_tps_drops(&tps_values, 10.0);

        assert_eq!(anomalies.len(), 2);
        assert_eq!(anomalies[0], 2); // index of first drop
        assert_eq!(anomalies[1], 3); // index of second drop
    }

    #[test]
    fn test_stuck_entity_anomaly() {
        let positions = vec![
            (100.0, 200.0),
            (100.0, 200.0),
            (100.0, 200.0),
            (100.0, 200.0),
            (100.1, 200.1),
        ];

        let is_stuck = anomaly::is_stuck_entity(&positions, 50);
        assert!(is_stuck);
    }

    #[test]
    fn test_not_stuck_entity() {
        let positions = vec![
            (100.0, 200.0),
            (102.0, 202.0),
            (105.0, 205.0),
            (108.0, 208.0),
        ];

        let is_stuck = anomaly::is_stuck_entity(&positions, 5);
        assert!(!is_stuck);
    }

    #[test]
    fn test_stream_parsing_memory_efficient() {
        let data = b"[INFO] Test line 1\n[WARN] Test line 2\n[ERROR] Test line 3\n";
        let cursor = Cursor::new(data);

        let line_count = std::io::BufRead::lines(cursor)
            .filter_map(Result::ok)
            .count();

        assert_eq!(line_count, 3);
    }

    #[test]
    fn test_time_parsing_for_last_duration() {
        let time_str = "5m";
        let duration = output::parse_duration(time_str);

        assert!(duration.is_ok());
        assert_eq!(duration.unwrap(), 5 * 60);
    }

    #[test]
    fn test_json_export_format() {
        let metrics = vec![
            parser::TPSMetric {
                timestamp: "2025-12-24 10:30:45".to_string(),
                tps: 59.8,
                dt_ms: 16.67,
                entities: 1234,
                chunks: 42,
            },
        ];

        let json = output::metrics_to_json(&metrics);
        assert!(json.contains("\"tps\""));
        assert!(json.contains("59.8"));
    }

    #[test]
    fn test_csv_export_format() {
        let metrics = vec![
            parser::TPSMetric {
                timestamp: "2025-12-24 10:30:45".to_string(),
                tps: 59.8,
                dt_ms: 16.67,
                entities: 1234,
                chunks: 42,
            },
        ];

        let csv = output::metrics_to_csv(&metrics);
        assert!(csv.contains("timestamp,tps,dt_ms,entities,chunks"));
        assert!(csv.contains("2025-12-24 10:30:45,59.8"));
    }

    #[test]
    fn test_tail_filtering() {
        let lines = (0..1000).map(|i| format!("[LINE {}]", i)).collect::<Vec<_>>();
        let tailed = parser::tail_lines(&lines, 100);

        assert_eq!(tailed.len(), 100);
        assert!(tailed[0].contains("[LINE 900]"));
        assert!(tailed[99].contains("[LINE 999]"));
    }
}
