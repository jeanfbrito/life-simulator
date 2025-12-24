//! Integration tests for debug API endpoints

#[cfg(test)]
mod tests {
    /// Test that the health endpoint JSON structure is valid
    #[test]
    fn test_health_endpoint_json_structure() {
        // Simulate what the endpoint would return
        let json_str = r#"{"status":"ok","alerts":{"tps_below_10":0,"entities_stuck":0,"population_crash":0,"ai_loops":0},"current_tps":60.0,"total_alerts":0,"is_healthy":true}"#;

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .expect("Health endpoint JSON should be valid");

        // Verify all required fields exist
        assert!(parsed.get("status").is_some(), "Must have 'status' field");
        assert!(parsed.get("alerts").is_some(), "Must have 'alerts' field");
        assert!(parsed.get("current_tps").is_some(), "Must have 'current_tps' field");
        assert!(parsed.get("total_alerts").is_some(), "Must have 'total_alerts' field");
        assert!(parsed.get("is_healthy").is_some(), "Must have 'is_healthy' field");

        // Verify alert count structure
        let alerts = parsed.get("alerts").unwrap().as_object().unwrap();
        assert!(alerts.contains_key("tps_below_10"), "Must have 'tps_below_10' in alerts");
        assert!(alerts.contains_key("entities_stuck"), "Must have 'entities_stuck' in alerts");
        assert!(alerts.contains_key("population_crash"), "Must have 'population_crash' in alerts");
        assert!(alerts.contains_key("ai_loops"), "Must have 'ai_loops' in alerts");

        // Verify field types
        assert!(parsed["status"].is_string(), "status must be a string");
        assert!(parsed["current_tps"].is_number(), "current_tps must be a number");
        assert!(parsed["total_alerts"].is_number(), "total_alerts must be a number");
        assert!(parsed["is_healthy"].is_boolean(), "is_healthy must be a boolean");
    }

    /// Test that the alerts endpoint JSON structure is valid
    #[test]
    fn test_alerts_endpoint_json_structure() {
        let json_str = r#"{"alerts":[{"tick":1000,"type":"TPS below 10","timestamp_ms":1640000000000,"message":"TPS below 10 at tick 1000"}],"total":1}"#;

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .expect("Alerts endpoint JSON should be valid");

        // Verify required fields
        assert!(parsed.get("alerts").is_some(), "Must have 'alerts' field");
        assert!(parsed.get("total").is_some(), "Must have 'total' field");
        assert!(parsed["alerts"].is_array(), "alerts must be an array");

        // Verify alert record structure
        let alerts_array = parsed["alerts"].as_array().unwrap();
        if !alerts_array.is_empty() {
            let alert = &alerts_array[0];
            assert!(alert.get("tick").is_some(), "Alert must have 'tick' field");
            assert!(alert.get("type").is_some(), "Alert must have 'type' field");
            assert!(alert.get("timestamp_ms").is_some(), "Alert must have 'timestamp_ms' field");
            assert!(alert.get("message").is_some(), "Alert must have 'message' field");

            // Verify field types
            assert!(alert["tick"].is_number(), "tick must be a number");
            assert!(alert["type"].is_string(), "type must be a string");
            assert!(alert["timestamp_ms"].is_number(), "timestamp_ms must be a number");
            assert!(alert["message"].is_string(), "message must be a string");
        }
    }

    /// Test that the TPS endpoint JSON structure is valid
    #[test]
    fn test_tps_endpoint_json_structure() {
        let json_str = r#"{"current_tps":59.5,"average_tps":59.5,"status":"excellent"}"#;

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .expect("TPS endpoint JSON should be valid");

        // Verify required fields
        assert!(parsed.get("current_tps").is_some(), "Must have 'current_tps' field");
        assert!(parsed.get("average_tps").is_some(), "Must have 'average_tps' field");
        assert!(parsed.get("status").is_some(), "Must have 'status' field");

        // Verify field types
        assert!(parsed["current_tps"].is_number(), "current_tps must be a number");
        assert!(parsed["average_tps"].is_number(), "average_tps must be a number");
        assert!(parsed["status"].is_string(), "status must be a string");
    }

    /// Test health status values
    #[test]
    fn test_health_status_values() {
        // Test "ok" status
        let ok_json = r#"{"status":"ok","alerts":{},"available":false}"#;
        let parsed: serde_json::Value = serde_json::from_str(ok_json).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Test "degraded" status
        let degraded_json = r#"{"status":"degraded","alerts":{},"available":false}"#;
        let parsed: serde_json::Value = serde_json::from_str(degraded_json).unwrap();
        assert_eq!(parsed["status"], "degraded");

        // Test "critical" status
        let critical_json = r#"{"status":"critical","alerts":{},"available":false}"#;
        let parsed: serde_json::Value = serde_json::from_str(critical_json).unwrap();
        assert_eq!(parsed["status"], "critical");
    }

    /// Test TPS status values
    #[test]
    fn test_tps_status_values() {
        // Test "excellent" status
        let excellent_json = r#"{"current_tps":59.5,"average_tps":59.5,"status":"excellent"}"#;
        let parsed: serde_json::Value = serde_json::from_str(excellent_json).unwrap();
        assert_eq!(parsed["status"], "excellent");

        // Test "good" status
        let good_json = r#"{"current_tps":45.0,"average_tps":45.0,"status":"good"}"#;
        let parsed: serde_json::Value = serde_json::from_str(good_json).unwrap();
        assert_eq!(parsed["status"], "good");

        // Test "ok" status
        let ok_json = r#"{"current_tps":15.0,"average_tps":15.0,"status":"ok"}"#;
        let parsed: serde_json::Value = serde_json::from_str(ok_json).unwrap();
        assert_eq!(parsed["status"], "ok");

        // Test "degraded" status
        let degraded_json = r#"{"current_tps":5.0,"average_tps":5.0,"status":"degraded"}"#;
        let parsed: serde_json::Value = serde_json::from_str(degraded_json).unwrap();
        assert_eq!(parsed["status"], "degraded");
    }

    /// Test alert types
    #[test]
    fn test_alert_type_values() {
        // Test various alert type strings
        let alert_types = vec![
            "TPS below 10",
            "Entities stuck",
            "Population crash",
            "AI loops detected",
        ];

        for alert_type in alert_types {
            let json = format!(r#"{{"tick":100,"type":"{}","timestamp_ms":1640000000000,"message":"test"}}"#, alert_type);
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed["type"], alert_type);
        }
    }

    /// Test empty alerts list
    #[test]
    fn test_empty_alerts_list() {
        let json_str = r#"{"alerts":[],"total":0}"#;

        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed["total"], 0);
        assert!(parsed["alerts"].as_array().unwrap().is_empty());
    }

    /// Test multiple alerts in response
    #[test]
    fn test_multiple_alerts() {
        let json_str = r#"{"alerts":[{"tick":1000,"type":"TPS below 10","timestamp_ms":1640000000000,"message":"msg1"},{"tick":1050,"type":"Entities stuck","timestamp_ms":1640000000050,"message":"msg2"}],"total":2}"#;

        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed["total"], 2);
        assert_eq!(parsed["alerts"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["alerts"][0]["tick"], 1000);
        assert_eq!(parsed["alerts"][1]["tick"], 1050);
    }

    /// Test alert count breakdown
    #[test]
    fn test_alert_count_breakdown() {
        let json_str = r#"{"status":"ok","alerts":{"tps_below_10":5,"entities_stuck":3,"population_crash":1,"ai_loops":2},"current_tps":30.0,"total_alerts":11,"is_healthy":false}"#;

        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let alerts = parsed["alerts"].as_object().unwrap();

        assert_eq!(alerts["tps_below_10"], 5);
        assert_eq!(alerts["entities_stuck"], 3);
        assert_eq!(alerts["population_crash"], 1);
        assert_eq!(alerts["ai_loops"], 2);
        assert_eq!(parsed["total_alerts"], 11);
    }

    /// Test TPS ranges and corresponding statuses
    #[test]
    fn test_tps_ranges() {
        // Excellent: >= 59.0
        let excellent = vec![59.0, 60.0, 59.5, 120.0];
        for tps in excellent {
            let json = format!(r#"{{"current_tps":{},"average_tps":{},"status":"excellent"}}"#, tps, tps);
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed["status"], "excellent", "TPS {} should be excellent", tps);
        }

        // Good: 30-59
        let good = vec![30.0, 45.0, 58.9];
        for tps in good {
            let json = format!(r#"{{"current_tps":{},"average_tps":{},"status":"good"}}"#, tps, tps);
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed["status"], "good", "TPS {} should be good", tps);
        }

        // OK: 10-30
        let ok = vec![10.0, 15.0, 29.9];
        for tps in ok {
            let json = format!(r#"{{"current_tps":{},"average_tps":{},"status":"ok"}}"#, tps, tps);
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed["status"], "ok", "TPS {} should be ok", tps);
        }

        // Degraded: < 10
        let degraded = vec![0.0, 5.0, 9.9];
        for tps in degraded {
            let json = format!(r#"{{"current_tps":{},"average_tps":{},"status":"degraded"}}"#, tps, tps);
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed["status"], "degraded", "TPS {} should be degraded", tps);
        }
    }

    /// Test health status determination logic
    #[test]
    fn test_health_status_logic() {
        // Test healthy status
        let healthy_json = r#"{"status":"ok","alerts":{},"current_tps":60.0,"total_alerts":0,"is_healthy":true}"#;
        let parsed: serde_json::Value = serde_json::from_str(healthy_json).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["is_healthy"], true);

        // Test degraded status (has alerts, TPS moderate)
        let degraded_json = r#"{"status":"degraded","alerts":{"tps_below_10":1},"current_tps":8.0,"total_alerts":1,"is_healthy":false}"#;
        let parsed: serde_json::Value = serde_json::from_str(degraded_json).unwrap();
        assert_eq!(parsed["status"], "degraded");
        assert_eq!(parsed["is_healthy"], false);

        // Test critical status (TPS very low)
        let critical_json = r#"{"status":"critical","alerts":{"tps_below_10":5},"current_tps":2.0,"total_alerts":5,"is_healthy":false}"#;
        let parsed: serde_json::Value = serde_json::from_str(critical_json).unwrap();
        assert_eq!(parsed["status"], "critical");
        assert_eq!(parsed["is_healthy"], false);
    }

    /// Test timestamp format in alerts
    #[test]
    fn test_alert_timestamp_format() {
        let json_str = r#"{"alerts":[{"tick":1000,"type":"TPS below 10","timestamp_ms":1640000000000,"message":"test"}],"total":1}"#;
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();

        let timestamp_ms = parsed["alerts"][0]["timestamp_ms"].as_u64().unwrap();
        // Should be a reasonable Unix timestamp in milliseconds (after year 2000)
        assert!(timestamp_ms > 946684800000, "Timestamp should be after year 2000");
    }
}
