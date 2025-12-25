//! API Client for Life Simulator Debug API
//!
//! Provides async HTTP client for polling simulator endpoints.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API client for communicating with the simulator
#[derive(Clone, Debug)]
pub struct SimulatorApiClient {
    base_url: String,
    client: reqwest::Client,
}

/// Position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Entity data from /api/entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: u32,
    #[serde(rename = "entity_type")]
    pub species: String,
    pub position: Position,
    pub health: Option<f32>,
}

/// Response from /api/entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesResponse {
    pub entities: Vec<Entity>,
}

/// Health status from /api/debug/health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub alerts: HashMap<String, u64>,
    pub current_tps: f64,
    pub total_alerts: Option<u64>,
    pub is_healthy: Option<bool>,
}

/// Alert record from /api/debug/alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub tick: u64,
    #[serde(rename = "type")]
    pub alert_type: String,
    pub timestamp_ms: u64,
    pub message: String,
}

/// Response from /api/debug/alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub alerts: Vec<Alert>,
    pub total: u64,
}

/// TPS metrics from /api/debug/tps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpsMetrics {
    pub current_tps: f64,
    pub average_tps: f64,
    pub status: String,
}

impl SimulatorApiClient {
    /// Create a new API client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .unwrap(),
        }
    }

    /// Get all entities
    pub async fn get_entities(&self) -> Result<EntitiesResponse> {
        let url = format!("{}/api/entities", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch entities")?;

        response
            .json::<EntitiesResponse>()
            .await
            .context("Failed to parse entities response")
    }

    /// Get health status
    pub async fn get_health(&self) -> Result<HealthStatus> {
        let url = format!("{}/api/debug/health", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch health status")?;

        response
            .json::<HealthStatus>()
            .await
            .context("Failed to parse health response")
    }

    /// Get recent alerts
    pub async fn get_alerts(&self) -> Result<AlertsResponse> {
        let url = format!("{}/api/debug/alerts", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch alerts")?;

        response
            .json::<AlertsResponse>()
            .await
            .context("Failed to parse alerts response")
    }

    /// Get TPS metrics
    pub async fn get_tps(&self) -> Result<TpsMetrics> {
        let url = format!("{}/api/debug/tps", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch TPS metrics")?;

        response
            .json::<TpsMetrics>()
            .await
            .context("Failed to parse TPS response")
    }

    /// Check if simulator is reachable
    pub async fn is_connected(&self) -> bool {
        self.get_health().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_client_creation() {
        let client = SimulatorApiClient::new("http://localhost:54321".to_string());
        assert_eq!(client.base_url, "http://localhost:54321");
    }

    #[tokio::test]
    async fn test_get_entities_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/api/entities")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"entities": [{"id": 1, "entity_type": "Rabbit", "position": {"x": 10, "y": 20}, "health": 100.0}]}"#)
            .create_async()
            .await;

        let client = SimulatorApiClient::new(server.url());
        let result = client.get_entities().await;

        assert!(result.is_ok());
        let entities = result.unwrap();
        assert_eq!(entities.entities.len(), 1);
        assert_eq!(entities.entities[0].species, "Rabbit");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_health_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/api/debug/health")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "ok", "alerts": {"tps_below_10": 0}, "current_tps": 60.0, "is_healthy": true}"#)
            .create_async()
            .await;

        let client = SimulatorApiClient::new(server.url());
        let result = client.get_health().await;

        assert!(result.is_ok());
        let health = result.unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.current_tps, 60.0);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_alerts_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/api/debug/alerts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"alerts": [{"tick": 1234, "type": "TpsBelow10", "timestamp_ms": 1640000000000, "message": "TPS dropped"}], "total": 1}"#)
            .create_async()
            .await;

        let client = SimulatorApiClient::new(server.url());
        let result = client.get_alerts().await;

        assert!(result.is_ok());
        let alerts = result.unwrap();
        assert_eq!(alerts.total, 1);
        assert_eq!(alerts.alerts[0].alert_type, "TpsBelow10");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_tps_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/api/debug/tps")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"current_tps": 59.8, "average_tps": 59.8, "status": "excellent"}"#)
            .create_async()
            .await;

        let client = SimulatorApiClient::new(server.url());
        let result = client.get_tps().await;

        assert!(result.is_ok());
        let tps = result.unwrap();
        assert_eq!(tps.status, "excellent");
        assert!((tps.current_tps - 59.8).abs() < 0.1);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_connection_check_when_available() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server.mock("GET", "/api/debug/health")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "ok", "alerts": {}, "current_tps": 60.0}"#)
            .create_async()
            .await;

        let client = SimulatorApiClient::new(server.url());
        assert!(client.is_connected().await);
    }

    #[tokio::test]
    async fn test_connection_check_when_unavailable() {
        let client = SimulatorApiClient::new("http://localhost:99999".to_string());
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_network_error_handling() {
        let client = SimulatorApiClient::new("http://invalid-host-that-does-not-exist:54321".to_string());
        let result = client.get_entities().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_malformed_json_handling() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server.mock("GET", "/api/entities")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": "json", "missing": "entities_field"}"#)
            .create_async()
            .await;

        let client = SimulatorApiClient::new(server.url());
        let result = client.get_entities().await;
        assert!(result.is_err());
    }
}
