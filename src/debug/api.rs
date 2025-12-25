//! Debug API endpoints and data structures
//!
//! Provides thread-safe access to health check data for web server endpoints.

use crate::debug::HealthChecker;
use bevy::prelude::*;
use serde_json::{json, Value};
use std::sync::{Arc, RwLock, OnceLock};
use std::collections::HashMap;

/// Global instance of HealthCheckApi
static HEALTH_CHECK_API_INSTANCE: OnceLock<Arc<HealthCheckApi>> = OnceLock::new();

/// Snapshot of health check data
#[derive(Clone)]
pub struct HealthCheckSnapshot {
    pub alerts: Vec<crate::debug::AlertRecord>,
    pub alert_counts: HashMap<&'static str, usize>,
    pub is_healthy: bool,
    pub current_tps: f64,
    pub current_tick: u64,
}

impl HealthCheckSnapshot {
    fn get_health_status_json(&self) -> String {
        let status = if self.is_healthy {
            "ok"
        } else if self.current_tps < 5.0 {
            "critical"
        } else {
            "degraded"
        };

        let mut alerts_obj = serde_json::Map::new();
        alerts_obj.insert("tps_below_10".to_string(), Value::Number(
            self.alert_counts.get("tps_below_10").copied().unwrap_or(0).into()
        ));
        alerts_obj.insert("entities_stuck".to_string(), Value::Number(
            self.alert_counts.get("entities_stuck").copied().unwrap_or(0).into()
        ));
        alerts_obj.insert("population_crash".to_string(), Value::Number(
            self.alert_counts.get("population_crash").copied().unwrap_or(0).into()
        ));
        alerts_obj.insert("ai_loops".to_string(), Value::Number(
            self.alert_counts.get("ai_loops").copied().unwrap_or(0).into()
        ));

        json!({
            "status": status,
            "alerts": alerts_obj,
            "current_tps": self.current_tps,
            "total_alerts": self.alerts.len(),
            "is_healthy": self.is_healthy
        }).to_string()
    }

    fn get_alerts_json(&self) -> String {
        let alert_list: Vec<Value> = self.alerts.iter()
            .rev()
            .take(100)
            .map(|record| {
                json!({
                    "tick": record.tick,
                    "type": record.alert_type.to_string(),
                    "timestamp_ms": record.timestamp_ms,
                    "message": format!("{} at tick {}", record.alert_type, record.tick)
                })
            })
            .collect();

        json!({
            "alerts": alert_list,
            "total": self.alerts.len()
        }).to_string()
    }

    fn get_tps_json(&self) -> String {
        let status = if self.current_tps >= 59.0 {
            "excellent"
        } else if self.current_tps >= 30.0 {
            "good"
        } else if self.current_tps >= 10.0 {
            "ok"
        } else {
            "degraded"
        };

        json!({
            "current_tps": self.current_tps,
            "average_tps": self.current_tps,
            "status": status
        }).to_string()
    }
}

/// Thread-safe wrapper for exposing health check data to web server
#[derive(Clone, Resource)]
pub struct HealthCheckApi {
    inner: Arc<RwLock<Option<HealthCheckSnapshot>>>,
}

impl HealthCheckApi {
    /// Create a new API wrapper
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    /// Get or initialize the global instance
    pub fn global() -> Arc<Self> {
        HEALTH_CHECK_API_INSTANCE
            .get_or_init(|| Arc::new(Self::new()))
            .clone()
    }

    /// Store a snapshot of the health checker data
    pub fn update(&self, health_checker: &HealthChecker, tps: f64, current_tick: u64) {
        let snapshot = HealthCheckSnapshot {
            alerts: health_checker.get_alerts(),
            alert_counts: health_checker.get_alert_counts(),
            is_healthy: health_checker.is_healthy(current_tick),
            current_tps: tps,
            current_tick,
        };

        if let Ok(mut inner) = self.inner.write() {
            *inner = Some(snapshot);
        }
    }

    /// Get health status JSON
    pub fn get_health_status_json(&self) -> String {
        if let Ok(inner) = self.inner.read() {
            if let Some(ref snapshot) = *inner {
                return snapshot.get_health_status_json();
            }
        }

        // Return default if no data available
        json!({
            "status": "unknown",
            "alerts": {},
            "available": false
        }).to_string()
    }

    /// Get alerts list JSON
    pub fn get_alerts_json(&self) -> String {
        if let Ok(inner) = self.inner.read() {
            if let Some(ref snapshot) = *inner {
                return snapshot.get_alerts_json();
            }
        }

        // Return empty alerts if no data available
        json!({
            "alerts": [],
            "total": 0
        }).to_string()
    }

    /// Get TPS JSON
    pub fn get_tps_json(&self) -> String {
        if let Ok(inner) = self.inner.read() {
            if let Some(ref snapshot) = *inner {
                return snapshot.get_tps_json();
            }
        }

        // Return default if no data available
        json!({
            "current_tps": 0.0,
            "average_tps": 0.0,
            "status": "unknown"
        }).to_string()
    }
}

impl Default for HealthCheckApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SYSTEM FOR UPDATING API DATA
// ============================================================================

/// System that updates the health check API with current data
pub fn update_health_check_api(
    health_checker: Res<HealthChecker>,
    metrics: Res<crate::simulation::TickMetrics>,
    tick: Res<crate::simulation::SimulationTick>,
    api: Res<HealthCheckApi>,
) {
    let tps = metrics.actual_tps();
    let current_tick = tick.get();
    api.update(&health_checker, tps, current_tick);
}

// ============================================================================
// PLUGIN
// ============================================================================

/// Plugin that registers the health check API
pub struct HealthCheckApiPlugin;

impl Plugin for HealthCheckApiPlugin {
    fn build(&self, app: &mut App) {
        // Initialize global instance and get it
        let api = HealthCheckApi::global();
        // Deref the Arc to get the inner HealthCheckApi
        let api_clone = (*api).clone();
        app.insert_resource(api_clone)
            .add_systems(Update, update_health_check_api.run_if(every_50_ticks));
    }
}

/// Run condition: Execute every 50 ticks
fn every_50_ticks(tick: Res<crate::simulation::SimulationTick>) -> bool {
    tick.get() % 50 == 0
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_api_new() {
        let api = HealthCheckApi::new();
        let json = api.get_health_status_json();
        assert!(json.contains("unknown"));
    }

    #[test]
    fn test_health_status_json_format() {
        let api = HealthCheckApi::new();
        let json = api.get_health_status_json();

        // Parse and verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("status").is_some());
        assert!(parsed.get("alerts").is_some());
        assert!(parsed.get("available").is_some());
    }

    #[test]
    fn test_alerts_json_format() {
        let api = HealthCheckApi::new();
        let json = api.get_alerts_json();

        // Parse and verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("alerts").is_some());
        assert!(parsed.get("total").is_some());
        assert!(parsed["alerts"].is_array());
    }

    #[test]
    fn test_tps_json_format() {
        let api = HealthCheckApi::new();
        let json = api.get_tps_json();

        // Parse and verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("current_tps").is_some());
        assert!(parsed.get("average_tps").is_some());
        assert!(parsed.get("status").is_some());
    }

    #[test]
    fn test_snapshot_health_status() {
        let snapshot = HealthCheckSnapshot {
            alerts: vec![],
            alert_counts: HashMap::new(),
            is_healthy: true,
            current_tps: 60.0,
            current_tick: 100,
        };

        let json = snapshot.get_health_status_json();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"current_tps\":60"));
    }

    #[test]
    fn test_snapshot_degraded_status() {
        let snapshot = HealthCheckSnapshot {
            alerts: vec![],
            alert_counts: HashMap::new(),
            is_healthy: false,
            current_tps: 8.5,
            current_tick: 100,
        };

        let json = snapshot.get_health_status_json();
        assert!(json.contains("\"status\":\"degraded\""));
    }

    #[test]
    fn test_snapshot_critical_status() {
        let snapshot = HealthCheckSnapshot {
            alerts: vec![],
            alert_counts: HashMap::new(),
            is_healthy: false,
            current_tps: 3.0,
            current_tick: 100,
        };

        let json = snapshot.get_health_status_json();
        assert!(json.contains("\"status\":\"critical\""));
    }

    #[test]
    fn test_tps_status_excellent() {
        let snapshot = HealthCheckSnapshot {
            alerts: vec![],
            alert_counts: HashMap::new(),
            is_healthy: true,
            current_tps: 59.5,
            current_tick: 100,
        };

        let json = snapshot.get_tps_json();
        assert!(json.contains("\"status\":\"excellent\""));
    }

    #[test]
    fn test_tps_status_good() {
        let snapshot = HealthCheckSnapshot {
            alerts: vec![],
            alert_counts: HashMap::new(),
            is_healthy: true,
            current_tps: 45.0,
            current_tick: 100,
        };

        let json = snapshot.get_tps_json();
        assert!(json.contains("\"status\":\"good\""));
    }

    #[test]
    fn test_tps_status_ok() {
        let snapshot = HealthCheckSnapshot {
            alerts: vec![],
            alert_counts: HashMap::new(),
            is_healthy: true,
            current_tps: 15.0,
            current_tick: 100,
        };

        let json = snapshot.get_tps_json();
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_tps_status_degraded() {
        let snapshot = HealthCheckSnapshot {
            alerts: vec![],
            alert_counts: HashMap::new(),
            is_healthy: false,
            current_tps: 5.0,
            current_tick: 100,
        };

        let json = snapshot.get_tps_json();
        assert!(json.contains("\"status\":\"degraded\""));
    }

    #[test]
    fn test_snapshot_alerts_json() {
        let mut alerts = vec![];
        for i in 0..5 {
            alerts.push(crate::debug::AlertRecord::new(
                crate::debug::HealthAlert::TpsBelow10,
                100 + i,
            ));
        }

        let snapshot = HealthCheckSnapshot {
            alerts,
            alert_counts: HashMap::new(),
            is_healthy: false,
            current_tps: 10.0,
            current_tick: 105,
        };

        let json = snapshot.get_alerts_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["total"], 5);
        assert_eq!(parsed["alerts"].as_array().unwrap().len(), 5);
    }
}
