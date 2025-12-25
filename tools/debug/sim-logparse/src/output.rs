use crate::parser::TPSMetric;
use serde_json::json;

/// Parse duration string like "5m" or "1h" into seconds
pub fn parse_duration(duration_str: &str) -> Result<u64, String> {
    let (number_str, unit) = if let Some(idx) = duration_str.find(|c: char| !c.is_numeric()) {
        duration_str.split_at(idx)
    } else {
        return Err("Invalid duration format".to_string());
    };

    let number: u64 = number_str
        .parse()
        .map_err(|_| "Invalid number in duration".to_string())?;

    let seconds = match unit {
        "s" => number,
        "m" => number * 60,
        "h" => number * 60 * 60,
        "d" => number * 60 * 60 * 24,
        _ => return Err(format!("Unknown time unit: {}", unit)),
    };

    Ok(seconds)
}

/// Convert metrics to JSON format
pub fn metrics_to_json(metrics: &[TPSMetric]) -> String {
    let json_array = metrics
        .iter()
        .map(|m| {
            json!({
                "timestamp": m.timestamp,
                "tps": m.tps,
                "dt_ms": m.dt_ms,
                "entities": m.entities,
                "chunks": m.chunks
            })
        })
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&json_array).unwrap_or_default()
}

/// Convert metrics to CSV format
pub fn metrics_to_csv(metrics: &[TPSMetric]) -> String {
    let mut csv = String::from("timestamp,tps,dt_ms,entities,chunks\n");

    for metric in metrics {
        csv.push_str(&format!(
            "{},{},{},{},{}\n",
            metric.timestamp, metric.tps, metric.dt_ms, metric.entities, metric.chunks
        ));
    }

    csv
}

/// Format metrics as a human-readable table
pub fn metrics_to_table(metrics: &[TPSMetric]) -> String {
    let mut table = String::from("Timestamp            | TPS   | DT(ms) | Entities | Chunks\n");
    table.push_str("---------------------+-------+--------+----------+--------\n");

    for metric in metrics {
        table.push_str(&format!(
            "{:<20} | {:>5.1} | {:>6.2} | {:>8} | {:>6}\n",
            metric.timestamp, metric.tps, metric.dt_ms, metric.entities, metric.chunks
        ));
    }

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("5m").unwrap(), 300);
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("1h").unwrap(), 3600);
    }

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("30s").unwrap(), 30);
    }

    #[test]
    fn test_parse_duration_days() {
        assert_eq!(parse_duration("1d").unwrap(), 86400);
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("xyz").is_err());
    }

    #[test]
    fn test_json_export_format() {
        let metrics = vec![TPSMetric {
            timestamp: "2025-12-24 10:30:45".to_string(),
            tps: 59.8,
            dt_ms: 16.67,
            entities: 1234,
            chunks: 42,
        }];

        let json = metrics_to_json(&metrics);
        assert!(json.contains("\"timestamp\""));
        assert!(json.contains("\"tps\""));
        assert!(json.contains("59.8"));
    }

    #[test]
    fn test_csv_export_format() {
        let metrics = vec![TPSMetric {
            timestamp: "2025-12-24 10:30:45".to_string(),
            tps: 59.8,
            dt_ms: 16.67,
            entities: 1234,
            chunks: 42,
        }];

        let csv = metrics_to_csv(&metrics);
        assert!(csv.contains("timestamp,tps,dt_ms,entities,chunks"));
        assert!(csv.contains("2025-12-24 10:30:45,59.8,16.67,1234,42"));
    }

    #[test]
    fn test_table_format() {
        let metrics = vec![TPSMetric {
            timestamp: "2025-12-24 10:30:45".to_string(),
            tps: 59.8,
            dt_ms: 16.67,
            entities: 1234,
            chunks: 42,
        }];

        let table = metrics_to_table(&metrics);
        assert!(table.contains("Timestamp"));
        assert!(table.contains("TPS"));
        assert!(table.contains("59.8"));
    }
}
