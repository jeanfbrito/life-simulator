use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A snapshot of an entity's state at a particular tick
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntitySnapshot {
    pub tick: u64,
    pub entity_id: u32,
    pub x: f64,
    pub y: f64,
    pub action: Option<String>,
    pub species: Option<String>,
}

impl EntitySnapshot {
    /// Calculate distance delta from another snapshot
    pub fn position_delta(&self, other: &EntitySnapshot) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Check if position hasn't changed
    pub fn position_unchanged(&self, other: &EntitySnapshot) -> bool {
        self.position_delta(other) < 0.001 // floating point tolerance
    }
}

/// Complete history of an entity's behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityHistory {
    pub entity_id: u32,
    pub species: Option<String>,
    pub snapshots: Vec<EntitySnapshot>,
    pub spawn_tick: Option<u64>,
    pub death_tick: Option<u64>,
}

impl EntityHistory {
    pub fn new(entity_id: u32) -> Self {
        EntityHistory {
            entity_id,
            species: None,
            snapshots: Vec::new(),
            spawn_tick: None,
            death_tick: None,
        }
    }

    /// Add a snapshot maintaining sorted order by tick
    pub fn add_snapshot(&mut self, snapshot: EntitySnapshot) {
        if self.spawn_tick.is_none() {
            self.spawn_tick = Some(snapshot.tick);
        }
        if self.species.is_none() {
            self.species = snapshot.species.clone();
        }
        self.snapshots.push(snapshot);
    }

    /// Get position changes between consecutive snapshots
    pub fn position_deltas(&self) -> Vec<f64> {
        self.snapshots
            .windows(2)
            .map(|w| w[0].position_delta(&w[1]))
            .collect()
    }

    /// Get the number of ticks without position change
    pub fn consecutive_stuck_ticks(&self, threshold: usize) -> Option<usize> {
        let mut consecutive = 0;
        for window in self.snapshots.windows(2) {
            if window[0].position_unchanged(&window[1]) {
                consecutive += 1;
                if consecutive >= threshold {
                    return Some(consecutive);
                }
            } else {
                consecutive = 0;
            }
        }
        None
    }

    /// Check if entity is stuck (no movement for N+ ticks)
    pub fn is_stuck(&self, threshold: usize) -> bool {
        self.consecutive_stuck_ticks(threshold).is_some()
    }

    /// Get current position if snapshots exist
    pub fn current_position(&self) -> Option<(f64, f64)> {
        self.snapshots.last().map(|s| (s.x, s.y))
    }

    /// Get duration in ticks from spawn to latest snapshot
    pub fn lifespan_ticks(&self) -> u64 {
        match (self.spawn_tick, self.snapshots.last()) {
            (Some(start), Some(last)) => last.tick - start,
            _ => 0,
        }
    }
}

/// Detects when entities are stuck
#[derive(Debug)]
pub struct StuckDetector {
    threshold: usize,
    histories: HashMap<u32, EntityHistory>,
}

impl StuckDetector {
    pub fn new(threshold: usize) -> Self {
        StuckDetector {
            threshold,
            histories: HashMap::new(),
        }
    }

    pub fn add_snapshot(&mut self, snapshot: EntitySnapshot) {
        self.histories
            .entry(snapshot.entity_id)
            .or_insert_with(|| EntityHistory::new(snapshot.entity_id))
            .add_snapshot(snapshot);
    }

    pub fn find_stuck_entities(&self) -> Vec<(u32, usize)> {
        self.histories
            .iter()
            .filter_map(|(id, history)| {
                history
                    .consecutive_stuck_ticks(self.threshold)
                    .map(|ticks| (*id, ticks))
            })
            .collect()
    }

    pub fn get_history(&self, entity_id: u32) -> Option<&EntityHistory> {
        self.histories.get(&entity_id)
    }

    pub fn all_histories(&self) -> Vec<&EntityHistory> {
        self.histories.values().collect()
    }
}

/// Detects action loops (repeated actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLoopDetection {
    pub entity_id: u32,
    pub action: String,
    pub repetitions: usize,
    pub start_tick: u64,
    pub end_tick: u64,
}

/// Analyzes action sequences for loops
#[derive(Debug)]
pub struct ActionLoopDetector {
    threshold: usize,
}

impl ActionLoopDetector {
    pub fn new(threshold: usize) -> Self {
        ActionLoopDetector { threshold }
    }

    /// Find all action loops in a history
    pub fn find_loops(&self, history: &EntityHistory) -> Vec<ActionLoopDetection> {
        let mut loops = Vec::new();

        if history.snapshots.is_empty() {
            return loops;
        }

        let mut i = 0;
        while i < history.snapshots.len() {
            let current = &history.snapshots[i];
            if let Some(action) = &current.action {
                let mut j = i + 1;
                while j < history.snapshots.len()
                    && history.snapshots[j].action.as_ref() == Some(action)
                {
                    j += 1;
                }

                let repetitions = j - i;
                if repetitions >= self.threshold {
                    loops.push(ActionLoopDetection {
                        entity_id: history.entity_id,
                        action: action.clone(),
                        repetitions,
                        start_tick: current.tick,
                        end_tick: history.snapshots[j - 1].tick,
                    });
                }

                i = j;
            } else {
                i += 1;
            }
        }

        loops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_snapshot_position_delta() {
        let snap1 = EntitySnapshot {
            tick: 0,
            entity_id: 1,
            x: 0.0,
            y: 0.0,
            action: None,
            species: None,
        };

        let snap2 = EntitySnapshot {
            tick: 1,
            entity_id: 1,
            x: 3.0,
            y: 4.0,
            action: None,
            species: None,
        };

        let delta = snap1.position_delta(&snap2);
        assert!((delta - 5.0).abs() < 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_entity_snapshot_position_unchanged() {
        let snap1 = EntitySnapshot {
            tick: 0,
            entity_id: 1,
            x: 10.5,
            y: 20.5,
            action: None,
            species: None,
        };

        let snap2 = EntitySnapshot {
            tick: 1,
            entity_id: 1,
            x: 10.5,
            y: 20.5,
            action: None,
            species: None,
        };

        assert!(snap1.position_unchanged(&snap2));
    }

    #[test]
    fn test_entity_history_add_snapshot() {
        let mut history = EntityHistory::new(42);

        history.add_snapshot(EntitySnapshot {
            tick: 0,
            entity_id: 42,
            x: 0.0,
            y: 0.0,
            action: Some("Graze".to_string()),
            species: Some("Rabbit".to_string()),
        });

        assert_eq!(history.entity_id, 42);
        assert_eq!(history.snapshots.len(), 1);
        assert_eq!(history.spawn_tick, Some(0));
        assert_eq!(history.species, Some("Rabbit".to_string()));
    }

    #[test]
    fn test_entity_history_lifespan() {
        let mut history = EntityHistory::new(1);

        history.add_snapshot(EntitySnapshot {
            tick: 10,
            entity_id: 1,
            x: 0.0,
            y: 0.0,
            action: None,
            species: None,
        });

        history.add_snapshot(EntitySnapshot {
            tick: 50,
            entity_id: 1,
            x: 10.0,
            y: 10.0,
            action: None,
            species: None,
        });

        assert_eq!(history.lifespan_ticks(), 40);
    }

    #[test]
    fn test_entity_history_is_stuck() {
        let mut history = EntityHistory::new(1);

        // Simulate entity stuck at same position for 10 ticks
        for tick in 0..15 {
            history.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: 100.0,
                y: 200.0,
                action: Some("Idle".to_string()),
                species: None,
            });
        }

        assert!(history.is_stuck(5)); // Stuck for more than 5 ticks
        assert!(!history.is_stuck(20)); // Not stuck for 20+ ticks
    }

    #[test]
    fn test_entity_history_not_stuck_moving() {
        let mut history = EntityHistory::new(1);

        // Entity moving each tick
        for tick in 0..10 {
            history.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: 100.0 + tick as f64,
                y: 200.0 + tick as f64,
                action: Some("Walk".to_string()),
                species: None,
            });
        }

        assert!(!history.is_stuck(3));
    }

    #[test]
    fn test_stuck_detector_find_stuck_entities() {
        let mut detector = StuckDetector::new(5);

        // Entity 1: stuck
        for tick in 0..10 {
            detector.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: 50.0,
                y: 50.0,
                action: None,
                species: None,
            });
        }

        // Entity 2: moving
        for tick in 0..10 {
            detector.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 2,
                x: 100.0 + tick as f64,
                y: 100.0,
                action: None,
                species: None,
            });
        }

        let stuck = detector.find_stuck_entities();
        assert_eq!(stuck.len(), 1);
        assert_eq!(stuck[0].0, 1); // entity_id
        assert!(stuck[0].1 >= 5); // repetitions >= threshold
    }

    #[test]
    fn test_action_loop_detector_single_loop() {
        let mut history = EntityHistory::new(1);

        // Simulate entity with repeated "Graze" action
        for tick in 0..25 {
            let action = if tick < 20 {
                Some("Graze".to_string())
            } else {
                Some("Drink".to_string())
            };

            history.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: 100.0 + (tick as f64),
                y: 200.0,
                action,
                species: Some("Deer".to_string()),
            });
        }

        let detector = ActionLoopDetector::new(10);
        let loops = detector.find_loops(&history);

        assert!(!loops.is_empty());
        assert_eq!(loops[0].action, "Graze");
        assert_eq!(loops[0].repetitions, 20);
    }

    #[test]
    fn test_action_loop_detector_no_loops() {
        let mut history = EntityHistory::new(1);

        // Simulate entity with changing actions
        let actions = vec!["Graze", "Walk", "Drink", "Sleep", "Walk", "Graze"];
        for (tick, action) in actions.iter().cycle().take(30).enumerate() {
            history.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: 100.0 + tick as f64,
                y: 200.0,
                action: Some(action.to_string()),
                species: None,
            });
        }

        let detector = ActionLoopDetector::new(10);
        let loops = detector.find_loops(&history);

        assert!(loops.is_empty()); // No action repeats 10+ times
    }

    #[test]
    fn test_action_loop_detector_threshold() {
        let mut history = EntityHistory::new(1);

        // Simulate entity with repeated action 5 times
        for tick in 0..10 {
            history.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: 100.0,
                y: 200.0,
                action: Some("Idle".to_string()),
                species: None,
            });
        }

        let detector_loose = ActionLoopDetector::new(3);
        let detector_strict = ActionLoopDetector::new(15);

        let loops_loose = detector_loose.find_loops(&history);
        let loops_strict = detector_strict.find_loops(&history);

        assert!(!loops_loose.is_empty()); // Threshold met
        assert!(loops_strict.is_empty()); // Threshold not met
    }

    #[test]
    fn test_position_deltas() {
        let mut history = EntityHistory::new(1);

        for tick in 0..4 {
            history.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: (tick * 10) as f64,
                y: 0.0,
                action: None,
                species: None,
            });
        }

        let deltas = history.position_deltas();
        assert_eq!(deltas.len(), 3);
        assert!(deltas.iter().all(|d| (d - 10.0).abs() < 0.001));
    }

    #[test]
    fn test_current_position() {
        let mut history = EntityHistory::new(1);

        history.add_snapshot(EntitySnapshot {
            tick: 0,
            entity_id: 1,
            x: 10.0,
            y: 20.0,
            action: None,
            species: None,
        });

        history.add_snapshot(EntitySnapshot {
            tick: 1,
            entity_id: 1,
            x: 30.0,
            y: 40.0,
            action: None,
            species: None,
        });

        let (x, y) = history.current_position().unwrap();
        assert_eq!(x, 30.0);
        assert_eq!(y, 40.0);
    }
}
