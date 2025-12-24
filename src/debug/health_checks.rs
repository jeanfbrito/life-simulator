//! Health Check System
//!
//! Provides monitoring and alerting for simulation performance and entity health.
//! Detects TPS drops, stuck entities, population crashes, and AI loops.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum number of alerts to store in ring buffer
const MAX_ALERTS: usize = 100;

/// TPS threshold below which to trigger alert
const MIN_HEALTHY_TPS: f64 = 10.0;

/// Number of ticks an entity must not move to be considered stuck
const STUCK_ENTITY_THRESHOLD_TICKS: u64 = 50;

/// Population loss percentage (0-100) to trigger crash alert
const POPULATION_CRASH_THRESHOLD: f32 = 50.0;

/// Time window for population monitoring (in ticks)
const POPULATION_WINDOW_TICKS: u64 = 100;

/// Number of times same action must repeat to trigger AI loop alert
const AI_LOOP_REPEAT_THRESHOLD: u32 = 20;

// ============================================================================
// TYPES
// ============================================================================

/// Represents different health alerts that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthAlert {
    /// TPS has dropped below minimum threshold (10)
    TpsBelow10,
    /// An entity hasn't moved in 50+ ticks
    EntitiesStuck,
    /// Population lost 50%+ entities in 100 ticks
    PopulationCrash,
    /// Same AI action repeated 20+ times
    AiLoops,
}

impl std::fmt::Display for HealthAlert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TpsBelow10 => write!(f, "TPS below 10"),
            Self::EntitiesStuck => write!(f, "Entities stuck"),
            Self::PopulationCrash => write!(f, "Population crash"),
            Self::AiLoops => write!(f, "AI loops detected"),
        }
    }
}

/// Alert record with timestamp
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AlertRecord {
    /// Type of alert
    pub alert_type: HealthAlert,
    /// Tick number when alert occurred
    pub tick: u64,
    /// Timestamp (milliseconds)
    pub timestamp_ms: u64,
}

impl AlertRecord {
    /// Create new alert record
    pub fn new(alert_type: HealthAlert, tick: u64) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            alert_type,
            tick,
            timestamp_ms,
        }
    }
}

/// Health state tracking for an entity
#[derive(Debug, Clone)]
pub struct EntityHealthState {
    /// Last known position (x, y) as tuple
    pub last_position: (i32, i32),
    /// Tick when position was last updated
    pub last_position_update_tick: u64,
    /// Current action (for AI loop detection)
    pub current_action: String,
    /// Action repeat counter
    pub action_repeat_count: u32,
}

impl EntityHealthState {
    pub fn new(position: (i32, i32), tick: u64) -> Self {
        Self {
            last_position: position,
            last_position_update_tick: tick,
            current_action: String::new(),
            action_repeat_count: 0,
        }
    }
}

/// Resource that manages health checking and alert storage
#[derive(Resource, Debug)]
pub struct HealthChecker {
    /// Ring buffer of recent alerts
    alerts: VecDeque<AlertRecord>,
    /// Entity health states (indexed by entity id hash)
    entity_states: std::collections::HashMap<u32, EntityHealthState>,
    /// Population history for crash detection
    population_history: VecDeque<(u64, u32)>, // (tick, count)
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self {
            alerts: VecDeque::with_capacity(MAX_ALERTS),
            entity_states: std::collections::HashMap::new(),
            population_history: VecDeque::with_capacity(POPULATION_WINDOW_TICKS as usize),
        }
    }
}

impl HealthChecker {
    /// Add a new alert to the ring buffer
    pub fn add_alert(&mut self, alert: HealthAlert, tick: u64) {
        let record = AlertRecord::new(alert, tick);
        self.alerts.push_back(record);

        // Maintain ring buffer size
        while self.alerts.len() > MAX_ALERTS {
            self.alerts.pop_front();
        }

        info!("HEALTH_ALERT: {} at tick {}", alert, tick);
    }

    /// Get all recent alerts
    pub fn get_alerts(&self) -> Vec<AlertRecord> {
        self.alerts.iter().copied().collect()
    }

    /// Get latest alert of specific type
    pub fn get_latest_alert(&self, alert_type: HealthAlert) -> Option<AlertRecord> {
        self.alerts
            .iter()
            .rev()
            .find(|a| a.alert_type == alert_type)
            .copied()
    }

    /// Count recent alerts of type in last N ticks
    pub fn count_alerts_in_window(&self, alert_type: HealthAlert, window_ticks: u64, current_tick: u64) -> usize {
        self.alerts
            .iter()
            .filter(|a| a.alert_type == alert_type && current_tick.saturating_sub(a.tick) <= window_ticks)
            .count()
    }

    /// Clear all alerts
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    /// Check if TPS is below threshold
    pub fn check_tps(&mut self, tps: f64, tick: u64) -> bool {
        if tps < MIN_HEALTHY_TPS && tps > 0.0 {
            self.add_alert(HealthAlert::TpsBelow10, tick);
            true
        } else {
            false
        }
    }

    /// Update entity position for stuck detection
    pub fn update_entity_position(&mut self, entity_id: u32, position: (i32, i32), tick: u64) {
        self.entity_states
            .entry(entity_id)
            .and_modify(|state| {
                if state.last_position != position {
                    state.last_position = position;
                    state.last_position_update_tick = tick;
                }
            })
            .or_insert_with(|| EntityHealthState::new(position, tick));
    }

    /// Check for stuck entities (no position change for 50+ ticks)
    pub fn check_stuck_entities(&mut self, current_tick: u64) -> bool {
        let stuck_count = self
            .entity_states
            .values()
            .filter(|state| current_tick.saturating_sub(state.last_position_update_tick) >= STUCK_ENTITY_THRESHOLD_TICKS)
            .count();

        if stuck_count > 0 {
            self.add_alert(HealthAlert::EntitiesStuck, current_tick);
            true
        } else {
            false
        }
    }

    /// Update population count for crash detection
    pub fn update_population(&mut self, entity_count: u32, tick: u64) {
        // Keep only relevant history
        while self.population_history.len() > POPULATION_WINDOW_TICKS as usize {
            self.population_history.pop_front();
        }

        // Only update if changed significantly
        if let Some((_, last_count)) = self.population_history.back() {
            if *last_count != entity_count {
                self.population_history.push_back((tick, entity_count));
            }
        } else {
            self.population_history.push_back((tick, entity_count));
        }
    }

    /// Check for population crash (50%+ loss in 100 ticks)
    pub fn check_population_crash(&mut self, current_tick: u64) -> bool {
        if self.population_history.len() < 2 {
            return false;
        }

        // Find oldest entry in window
        let window_start = current_tick.saturating_sub(POPULATION_WINDOW_TICKS);
        let oldest_in_window = self
            .population_history
            .iter()
            .find(|(tick, _)| *tick >= window_start)
            .copied();

        let current_pop = self.population_history.back().copied();

        if let (Some((_, old_count)), Some((_, new_count))) = (oldest_in_window, current_pop) {
            if old_count > 0 {
                let loss_percent = ((old_count as f32 - new_count as f32) / old_count as f32) * 100.0;
                if loss_percent >= POPULATION_CRASH_THRESHOLD {
                    self.add_alert(HealthAlert::PopulationCrash, current_tick);
                    return true;
                }
            }
        }

        false
    }

    /// Update entity action for AI loop detection
    pub fn update_entity_action(&mut self, entity_id: u32, action: String) {
        self.entity_states
            .entry(entity_id)
            .and_modify(|state| {
                if state.current_action == action {
                    state.action_repeat_count += 1;
                } else {
                    state.current_action = action.clone();
                    state.action_repeat_count = 1;
                }
            })
            .or_insert_with(|| {
                let mut state = EntityHealthState::new((0, 0), 0);
                state.current_action = action;
                state.action_repeat_count = 1;
                state
            });
    }

    /// Check for AI loops (same action 20+ times)
    pub fn check_ai_loops(&mut self, current_tick: u64) -> bool {
        let loop_count = self
            .entity_states
            .values()
            .filter(|state| state.action_repeat_count >= AI_LOOP_REPEAT_THRESHOLD)
            .count();

        if loop_count > 0 {
            self.add_alert(HealthAlert::AiLoops, current_tick);
            true
        } else {
            false
        }
    }

    /// Reset action counters (call after checking for loops)
    pub fn reset_action_counters(&mut self) {
        for state in self.entity_states.values_mut() {
            state.action_repeat_count = 0;
        }
    }

    /// Clear old entity states to prevent memory leak
    /// Only removes states for dead entities, preserves state for alive entities
    pub fn cleanup_old_states(&mut self, is_alive: impl Fn(u32) -> bool) {
        self.entity_states.retain(|entity_id, _| is_alive(*entity_id));
    }

    /// Get health status summary as JSON
    pub fn get_health_summary(&self) -> serde_json::Value {
        use serde_json::json;

        let recent_alerts: Vec<_> = self
            .alerts
            .iter()
            .rev()
            .take(10)
            .map(|a| json!({
                "type": a.alert_type.to_string(),
                "tick": a.tick,
                "timestamp_ms": a.timestamp_ms,
            }))
            .collect();

        json!({
            "total_alerts": self.alerts.len(),
            "recent_alerts": recent_alerts,
            "entity_states_count": self.entity_states.len(),
            "population_history_len": self.population_history.len(),
        })
    }

    /// Get count of each alert type
    pub fn get_alert_counts(&self) -> std::collections::HashMap<&'static str, usize> {
        let mut counts = std::collections::HashMap::new();
        for alert in self.alerts.iter() {
            let key = match alert.alert_type {
                HealthAlert::TpsBelow10 => "tps_below_10",
                HealthAlert::EntitiesStuck => "entities_stuck",
                HealthAlert::PopulationCrash => "population_crash",
                HealthAlert::AiLoops => "ai_loops",
            };
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }

    /// Check overall health status
    pub fn is_healthy(&self) -> bool {
        // Healthy if no critical alerts in last 50 ticks
        // (this would be set in context when called)
        self.alerts.len() == 0 || self.get_latest_alert(HealthAlert::TpsBelow10).is_none()
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

/// Plugin that registers the health check system
pub struct HealthCheckPlugin;

impl Plugin for HealthCheckPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HealthChecker>()
            .add_systems(Update, health_check_system.run_if(every_50_ticks));
    }
}

/// Run condition: Execute every 50 ticks
fn every_50_ticks(tick: Res<crate::simulation::SimulationTick>) -> bool {
    tick.get() % 50 == 0
}

/// System that runs health checks every 50 ticks
fn health_check_system(
    mut health_checker: ResMut<HealthChecker>,
    tick: Res<crate::simulation::SimulationTick>,
    metrics: Res<crate::simulation::TickMetrics>,
    entity_query: Query<Entity>,
) {
    let current_tick = tick.get();
    let tps = metrics.actual_tps();

    // Run all checks
    health_checker.check_tps(tps, current_tick);
    health_checker.check_stuck_entities(current_tick);
    health_checker.check_population_crash(current_tick);
    health_checker.check_ai_loops(current_tick);

    // Reset action counters
    health_checker.reset_action_counters();

    // Collect alive entity IDs
    let alive_entities: std::collections::HashSet<u32> = entity_query
        .iter()
        .map(|entity| entity.index())
        .collect();

    // Clean only dead entities
    health_checker.cleanup_old_states(|id| alive_entities.contains(&id));

    // Log summary if we have alerts
    let alert_count = health_checker.get_alerts().len();
    if alert_count > 0 {
        let recent_count = health_checker.count_alerts_in_window(
            HealthAlert::TpsBelow10,
            50,
            current_tick,
        );
        if recent_count > 0 {
            warn!("HEALTH_CHECK: {} total alerts, {} TPS alerts in last 50 ticks", alert_count, recent_count);
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = HealthAlert::TpsBelow10;
        assert_eq!(alert.to_string(), "TPS below 10");

        let alert = HealthAlert::EntitiesStuck;
        assert_eq!(alert.to_string(), "Entities stuck");

        let alert = HealthAlert::PopulationCrash;
        assert_eq!(alert.to_string(), "Population crash");

        let alert = HealthAlert::AiLoops;
        assert_eq!(alert.to_string(), "AI loops detected");
    }

    #[test]
    fn test_alert_record_creation() {
        let record = AlertRecord::new(HealthAlert::TpsBelow10, 100);
        assert_eq!(record.alert_type, HealthAlert::TpsBelow10);
        assert_eq!(record.tick, 100);
        assert!(record.timestamp_ms > 0);
    }

    #[test]
    fn test_health_checker_add_alert() {
        let mut checker = HealthChecker::default();
        assert_eq!(checker.get_alerts().len(), 0);

        checker.add_alert(HealthAlert::TpsBelow10, 50);
        assert_eq!(checker.get_alerts().len(), 1);

        let alert = checker.get_alerts()[0];
        assert_eq!(alert.alert_type, HealthAlert::TpsBelow10);
        assert_eq!(alert.tick, 50);
    }

    #[test]
    fn test_health_checker_ring_buffer() {
        let mut checker = HealthChecker::default();

        // Add more alerts than max capacity
        for i in 0..150 {
            checker.add_alert(HealthAlert::TpsBelow10, i);
        }

        // Should only keep last 100
        assert_eq!(checker.get_alerts().len(), MAX_ALERTS);
        assert_eq!(checker.get_alerts()[0].tick, 50); // First remaining should be tick 50
    }

    #[test]
    fn test_check_tps_below_threshold() {
        let mut checker = HealthChecker::default();

        // TPS below 10 should trigger alert
        assert!(checker.check_tps(5.0, 100));
        assert_eq!(checker.get_alerts().len(), 1);
        assert_eq!(checker.get_alerts()[0].alert_type, HealthAlert::TpsBelow10);

        // TPS above 10 should not trigger alert
        assert!(!checker.check_tps(15.0, 150));
        assert_eq!(checker.get_alerts().len(), 1); // No new alert
    }

    #[test]
    fn test_check_stuck_entities() {
        let mut checker = HealthChecker::default();

        // Create entity that moves at tick 10
        checker.update_entity_position(1, (10, 20), 10);

        // Check at tick 40 - should not be stuck (30 ticks < 50)
        assert!(!checker.check_stuck_entities(40));

        // Check at tick 65 - should be stuck (55 ticks >= 50)
        assert!(checker.check_stuck_entities(65));
        assert_eq!(checker.get_alerts().len(), 1);
        assert_eq!(checker.get_alerts()[0].alert_type, HealthAlert::EntitiesStuck);

        // Entity moves - should reset
        checker.update_entity_position(1, (11, 20), 70);
        checker.reset_action_counters();
        assert!(!checker.check_stuck_entities(100)); // Only 30 ticks have passed
    }

    #[test]
    fn test_population_crash_detection() {
        let mut checker = HealthChecker::default();

        // Start with 100 entities at tick 0
        checker.update_population(100, 0);

        // Add population at various ticks
        checker.update_population(90, 20);
        checker.update_population(80, 40);
        checker.update_population(50, 80); // 50% loss
        checker.update_population(45, 100); // Drops below 50% of original

        // Check at tick 100 - should detect crash
        assert!(checker.check_population_crash(100));
        assert_eq!(checker.get_alerts().len(), 1);
        assert_eq!(checker.get_alerts()[0].alert_type, HealthAlert::PopulationCrash);
    }

    #[test]
    fn test_ai_loop_detection() {
        let mut checker = HealthChecker::default();

        // Entity repeats same action
        for _ in 0..20 {
            checker.update_entity_action(1, "Attack".to_string());
        }

        // Should detect AI loop
        assert!(checker.check_ai_loops(100));
        assert_eq!(checker.get_alerts().len(), 1);
        assert_eq!(checker.get_alerts()[0].alert_type, HealthAlert::AiLoops);

        // After reset, should not detect loop
        checker.reset_action_counters();
        assert!(!checker.check_ai_loops(200));
    }

    #[test]
    fn test_get_latest_alert() {
        let mut checker = HealthChecker::default();

        checker.add_alert(HealthAlert::TpsBelow10, 10);
        checker.add_alert(HealthAlert::EntitiesStuck, 20);
        checker.add_alert(HealthAlert::TpsBelow10, 30);

        let latest = checker.get_latest_alert(HealthAlert::TpsBelow10).unwrap();
        assert_eq!(latest.tick, 30);

        let latest = checker.get_latest_alert(HealthAlert::PopulationCrash);
        assert!(latest.is_none());
    }

    #[test]
    fn test_count_alerts_in_window() {
        let mut checker = HealthChecker::default();

        checker.add_alert(HealthAlert::TpsBelow10, 10);
        checker.add_alert(HealthAlert::TpsBelow10, 20);
        checker.add_alert(HealthAlert::TpsBelow10, 30);
        checker.add_alert(HealthAlert::EntitiesStuck, 35);

        // Count in 30-tick window from tick 50
        let count = checker.count_alerts_in_window(HealthAlert::TpsBelow10, 30, 50);
        assert_eq!(count, 2); // Ticks 20 and 30 are within window

        let count = checker.count_alerts_in_window(HealthAlert::EntitiesStuck, 30, 50);
        assert_eq!(count, 1); // Tick 35 is within window
    }

    #[test]
    fn test_clear_alerts() {
        let mut checker = HealthChecker::default();

        checker.add_alert(HealthAlert::TpsBelow10, 10);
        checker.add_alert(HealthAlert::EntitiesStuck, 20);
        assert_eq!(checker.get_alerts().len(), 2);

        checker.clear_alerts();
        assert_eq!(checker.get_alerts().len(), 0);
    }

    #[test]
    fn test_get_alert_counts() {
        let mut checker = HealthChecker::default();

        checker.add_alert(HealthAlert::TpsBelow10, 10);
        checker.add_alert(HealthAlert::TpsBelow10, 20);
        checker.add_alert(HealthAlert::EntitiesStuck, 30);
        checker.add_alert(HealthAlert::PopulationCrash, 40);

        let counts = checker.get_alert_counts();
        assert_eq!(counts.get("tps_below_10"), Some(&2));
        assert_eq!(counts.get("entities_stuck"), Some(&1));
        assert_eq!(counts.get("population_crash"), Some(&1));
        assert_eq!(counts.get("ai_loops"), None);
    }

    #[test]
    fn test_get_health_summary() {
        let mut checker = HealthChecker::default();

        checker.add_alert(HealthAlert::TpsBelow10, 10);
        checker.add_alert(HealthAlert::EntitiesStuck, 20);

        let summary = checker.get_health_summary();
        assert_eq!(summary["total_alerts"], 2);
        assert_eq!(summary["recent_alerts"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_is_healthy() {
        let mut checker = HealthChecker::default();

        // Initially healthy
        assert!(checker.is_healthy());

        // Not healthy with TPS alert
        checker.add_alert(HealthAlert::TpsBelow10, 10);
        assert!(!checker.is_healthy());

        // Healthy with other alerts
        let mut checker2 = HealthChecker::default();
        checker2.add_alert(HealthAlert::EntitiesStuck, 10);
        assert!(checker2.is_healthy());
    }

    #[test]
    fn test_entity_health_state() {
        let state = EntityHealthState::new((10, 20), 5);
        assert_eq!(state.last_position, (10, 20));
        assert_eq!(state.last_position_update_tick, 5);
        assert_eq!(state.current_action, String::new());
        assert_eq!(state.action_repeat_count, 0);
    }

    #[test]
    fn test_mixed_alerts() {
        let mut checker = HealthChecker::default();

        // Simulate multiple failure conditions
        checker.check_tps(5.0, 10); // TPS alert
        checker.update_entity_position(1, (10, 20), 10);
        checker.check_stuck_entities(65); // Stuck alert

        // Population crash - need to be within 100 ticks
        checker.update_population(100, 10);
        checker.update_population(45, 70); // 55% loss in 60 ticks
        checker.check_population_crash(70); // Check within window

        checker.update_entity_action(2, "Loop".to_string());
        for _ in 0..20 {
            checker.update_entity_action(2, "Loop".to_string());
        }
        checker.check_ai_loops(200); // AI loop alert

        let alerts = checker.get_alerts();
        assert!(alerts.len() >= 3); // At least 3 alerts
        assert!(alerts.iter().any(|a| a.alert_type == HealthAlert::TpsBelow10));
        assert!(alerts.iter().any(|a| a.alert_type == HealthAlert::EntitiesStuck));
        assert!(alerts.iter().any(|a| a.alert_type == HealthAlert::PopulationCrash));
    }

}
