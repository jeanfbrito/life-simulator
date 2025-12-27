//! Application State Management
//!
//! Manages the state of the TUI application, including data updates and delta tracking.

use crate::api_client::{EntitiesResponse, HealthStatus, SimulatorApiClient, TpsMetrics};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Application state
#[derive(Debug)]
pub struct App {
    /// API client for fetching data
    pub client: SimulatorApiClient,
    /// Current entity counts by species
    pub entity_counts: HashMap<String, i32>,
    /// Previous entity counts for delta calculation
    pub previous_entity_counts: HashMap<String, i32>,
    /// Current health status
    pub health_status: Option<HealthStatus>,
    /// Recent alerts
    pub alerts: Vec<crate::api_client::Alert>,
    /// Current TPS metrics
    pub tps_metrics: Option<TpsMetrics>,
    /// Connection status
    pub is_connected: bool,
    /// Last successful update timestamp
    pub last_update: Option<Instant>,
    /// Update interval
    pub update_interval: Duration,
    /// Should quit
    pub should_quit: bool,
}

impl App {
    /// Create a new application state
    pub fn new(base_url: String, refresh_seconds: u64) -> Self {
        Self {
            client: SimulatorApiClient::new(base_url),
            entity_counts: HashMap::new(),
            previous_entity_counts: HashMap::new(),
            health_status: None,
            alerts: Vec::new(),
            tps_metrics: None,
            is_connected: false,
            last_update: None,
            update_interval: Duration::from_secs(refresh_seconds),
            should_quit: false,
        }
    }

    /// Update all data from API
    pub async fn update(&mut self) -> anyhow::Result<()> {
        // Check connection
        self.is_connected = self.client.is_connected().await;

        if !self.is_connected {
            return Ok(());
        }

        // Fetch entities and calculate counts
        if let Ok(entities_response) = self.client.get_entities().await {
            self.update_entity_counts(entities_response);
        }

        // Fetch health status
        if let Ok(health) = self.client.get_health().await {
            self.health_status = Some(health);
        }

        // Fetch alerts
        if let Ok(alerts_response) = self.client.get_alerts().await {
            self.alerts = alerts_response.alerts.into_iter().take(10).collect();
        }

        // Fetch TPS metrics
        if let Ok(tps) = self.client.get_tps().await {
            self.tps_metrics = Some(tps);
        }

        self.last_update = Some(Instant::now());
        Ok(())
    }

    /// Update entity counts and calculate deltas
    fn update_entity_counts(&mut self, entities: EntitiesResponse) {
        // Store previous counts
        self.previous_entity_counts = self.entity_counts.clone();

        // Calculate new counts
        let mut new_counts: HashMap<String, i32> = HashMap::new();
        for entity in entities.entities {
            *new_counts.entry(entity.species).or_insert(0) += 1;
        }

        self.entity_counts = new_counts;
    }

    /// Get entity count delta for a species
    pub fn get_entity_delta(&self, species: &str) -> i32 {
        let current = self.entity_counts.get(species).copied().unwrap_or(0);
        let previous = self.previous_entity_counts.get(species).copied().unwrap_or(0);
        current - previous
    }

    /// Get total entity count
    pub fn get_total_entities(&self) -> i32 {
        self.entity_counts.values().sum()
    }

    /// Get formatted TPS string
    pub fn get_tps_display(&self) -> String {
        if let Some(ref tps) = self.tps_metrics {
            format!("{:.1}", tps.current_tps)
        } else {
            "N/A".to_string()
        }
    }

    /// Get connection status display
    pub fn get_connection_status(&self) -> &str {
        if self.is_connected {
            "Connected"
        } else {
            "Disconnected"
        }
    }

    /// Mark application for quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Should the application quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_client::{Entity, Position};

    #[test]
    fn test_app_creation() {
        let app = App::new("http://localhost:54321".to_string(), 1);
        assert_eq!(app.entity_counts.len(), 0);
        assert!(!app.is_connected);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_entity_counts_update() {
        let mut app = App::new("http://localhost:54321".to_string(), 1);

        let entities = EntitiesResponse {
            entities: vec![
                Entity { id: 1, species: "Rabbit".to_string(), position: Position { x: 0, y: 0 }, health: Some(100.0) },
                Entity { id: 2, species: "Rabbit".to_string(), position: Position { x: 1, y: 1 }, health: Some(100.0) },
                Entity { id: 3, species: "Fox".to_string(), position: Position { x: 2, y: 2 }, health: Some(100.0) },
            ],
        };

        app.update_entity_counts(entities);

        assert_eq!(app.entity_counts.get("Rabbit"), Some(&2));
        assert_eq!(app.entity_counts.get("Fox"), Some(&1));
        assert_eq!(app.get_total_entities(), 3);
    }

    #[test]
    fn test_entity_delta_calculation() {
        let mut app = App::new("http://localhost:54321".to_string(), 1);

        // Initial state
        let entities1 = EntitiesResponse {
            entities: vec![
                Entity { id: 1, species: "Rabbit".to_string(), position: Position { x: 0, y: 0 }, health: Some(100.0) },
                Entity { id: 2, species: "Rabbit".to_string(), position: Position { x: 1, y: 1 }, health: Some(100.0) },
            ],
        };
        app.update_entity_counts(entities1);

        // Updated state - one rabbit died
        let entities2 = EntitiesResponse {
            entities: vec![
                Entity { id: 1, species: "Rabbit".to_string(), position: Position { x: 0, y: 0 }, health: Some(100.0) },
            ],
        };
        app.update_entity_counts(entities2);

        assert_eq!(app.get_entity_delta("Rabbit"), -1);
        assert_eq!(app.entity_counts.get("Rabbit"), Some(&1));
    }

    #[test]
    fn test_entity_delta_new_species() {
        let mut app = App::new("http://localhost:54321".to_string(), 1);

        let entities = EntitiesResponse {
            entities: vec![
                Entity { id: 1, species: "Wolf".to_string(), position: Position { x: 0, y: 0 }, health: Some(100.0) },
            ],
        };
        app.update_entity_counts(entities);

        assert_eq!(app.get_entity_delta("Wolf"), 1);
    }

    #[test]
    fn test_tps_display() {
        let mut app = App::new("http://localhost:54321".to_string(), 1);

        // No TPS data
        assert_eq!(app.get_tps_display(), "N/A");

        // With TPS data
        app.tps_metrics = Some(TpsMetrics {
            current_tps: 59.8,
            average_tps: 59.8,
            status: "excellent".to_string(),
        });
        assert_eq!(app.get_tps_display(), "59.8");
    }

    #[test]
    fn test_connection_status() {
        let mut app = App::new("http://localhost:54321".to_string(), 1);

        assert_eq!(app.get_connection_status(), "Disconnected");

        app.is_connected = true;
        assert_eq!(app.get_connection_status(), "Connected");
    }

    #[test]
    fn test_quit_flag() {
        let mut app = App::new("http://localhost:54321".to_string(), 1);

        assert!(!app.should_quit());

        app.quit();
        assert!(app.should_quit());
    }

    #[test]
    fn test_total_entities() {
        let mut app = App::new("http://localhost:54321".to_string(), 1);

        let entities = EntitiesResponse {
            entities: vec![
                Entity { id: 1, species: "Rabbit".to_string(), position: Position { x: 0, y: 0 }, health: Some(100.0) },
                Entity { id: 2, species: "Fox".to_string(), position: Position { x: 1, y: 1 }, health: Some(100.0) },
                Entity { id: 3, species: "Wolf".to_string(), position: Position { x: 2, y: 2 }, health: Some(100.0) },
            ],
        };
        app.update_entity_counts(entities);

        assert_eq!(app.get_total_entities(), 3);
    }
}
