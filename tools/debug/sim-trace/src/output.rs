use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use crate::entities::{EntityHistory, ActionLoopDetection, StuckDetector};

/// Export entity history to JSON file
pub fn export_timeline_json(
    filename: &str,
    history: &EntityHistory,
    include_positions: bool,
    include_actions: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut timeline = json!({
        "entity_id": history.entity_id,
        "species": history.species,
        "spawn_tick": history.spawn_tick,
        "death_tick": history.death_tick,
        "lifespan_ticks": history.lifespan_ticks(),
        "total_snapshots": history.snapshots.len(),
    });

    if include_positions || include_actions {
        let mut snapshots = Vec::new();

        for snapshot in &history.snapshots {
            let mut snap = json!({
                "tick": snapshot.tick,
            });

            if include_positions {
                snap["position"] = json!({
                    "x": snapshot.x,
                    "y": snapshot.y,
                });
            }

            if include_actions {
                snap["action"] = serde_json::to_value(&snapshot.action)?;
            }

            snapshots.push(snap);
        }

        timeline["snapshots"] = Value::Array(snapshots);
    }

    let mut file = File::create(filename)?;
    file.write_all(serde_json::to_string_pretty(&timeline)?.as_bytes())?;

    Ok(())
}

/// Format stuck entities for display
pub fn format_stuck_entities(stuck_list: &[(u32, usize)], json_output: bool) -> String {
    if json_output {
        let data: Vec<_> = stuck_list
            .iter()
            .map(|(id, ticks)| json!({ "entity_id": id, "stuck_ticks": ticks }))
            .collect();

        serde_json::to_string_pretty(&data).unwrap_or_default()
    } else {
        let mut output = String::new();
        output.push_str("Entity ID | Stuck Ticks\n");
        output.push_str("-----------|-----------\n");

        for (entity_id, ticks) in stuck_list {
            output.push_str(&format!("{:<9} | {:<9}\n", entity_id, ticks));
        }

        output
    }
}

/// Format entity history details
pub fn format_entity_details(
    history: &EntityHistory,
    history_limit: Option<usize>,
    show_actions: bool,
    json_output: bool,
) -> String {
    if json_output {
        let detail = json!({
            "entity_id": history.entity_id,
            "species": history.species,
            "spawn_tick": history.spawn_tick,
            "death_tick": history.death_tick,
            "lifespan_ticks": history.lifespan_ticks(),
            "total_snapshots": history.snapshots.len(),
        });

        serde_json::to_string_pretty(&detail).unwrap_or_default()
    } else {
        let mut output = String::new();

        output.push_str(&format!("Entity #{}\n", history.entity_id));
        output.push_str(&format!("Species: {}\n", history.species.as_deref().unwrap_or("Unknown")));
        output.push_str(&format!(
            "Lifespan: {} ticks (spawned at tick {})\n",
            history.lifespan_ticks(),
            history.spawn_tick.unwrap_or(0)
        ));
        output.push_str(&format!(
            "Total snapshots: {}\n\n",
            history.snapshots.len()
        ));

        let snapshots = if let Some(limit) = history_limit {
            if history.snapshots.len() > limit {
                &history.snapshots[history.snapshots.len() - limit..]
            } else {
                &history.snapshots
            }
        } else {
            &history.snapshots
        };

        output.push_str("Tick    | Position              | Action\n");
        output.push_str("--------|----------------------|--------------------\n");

        for snapshot in snapshots {
            let action_str = snapshot
                .action
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("(unknown)");

            if show_actions {
                output.push_str(&format!(
                    "{:<7} | ({:7.1}, {:7.1}) | {}\n",
                    snapshot.tick, snapshot.x, snapshot.y, action_str
                ));
            } else {
                output.push_str(&format!(
                    "{:<7} | ({:7.1}, {:7.1})\n",
                    snapshot.tick, snapshot.x, snapshot.y
                ));
            }
        }

        output
    }
}

/// Format action loops for display
pub fn format_action_loops(
    loops: &[ActionLoopDetection],
    json_output: bool,
) -> String {
    if json_output {
        serde_json::to_string_pretty(&loops).unwrap_or_default()
    } else {
        let mut output = String::new();
        output.push_str("Entity | Action   | Repetitions | Duration (ticks)\n");
        output.push_str("-------|----------|-------------|------------------\n");

        for action_loop in loops {
            let duration = action_loop.end_tick - action_loop.start_tick;
            output.push_str(&format!(
                "{:<6} | {:<8} | {:<11} | {}\n",
                action_loop.entity_id,
                action_loop.action,
                action_loop.repetitions,
                duration
            ));
        }

        output
    }
}

/// Format summary statistics
pub fn format_summary(
    detector: &StuckDetector,
    json_output: bool,
    csv_output: bool,
) -> String {
    if json_output {
        let histories = detector.all_histories();
        let stats: Vec<_> = histories
            .iter()
            .map(|h| {
                json!({
                    "entity_id": h.entity_id,
                    "species": h.species,
                    "lifespan_ticks": h.lifespan_ticks(),
                    "snapshot_count": h.snapshots.len(),
                    "spawn_tick": h.spawn_tick,
                    "current_position": h.current_position(),
                })
            })
            .collect();

        serde_json::to_string_pretty(&stats).unwrap_or_default()
    } else if csv_output {
        let mut output = String::from("entity_id,species,lifespan_ticks,snapshots,spawn_tick,x,y\n");

        for history in detector.all_histories() {
            let (x, y) = history.current_position().unwrap_or((0.0, 0.0));
            output.push_str(&format!(
                "{},{},{},{},{},{:.1},{:.1}\n",
                history.entity_id,
                history.species.as_deref().unwrap_or("Unknown"),
                history.lifespan_ticks(),
                history.snapshots.len(),
                history.spawn_tick.unwrap_or(0),
                x,
                y
            ));
        }

        output
    } else {
        let histories = detector.all_histories();
        let mut output = String::new();

        output.push_str(&format!("Entity Summary: {} total entities\n\n", histories.len()));
        output.push_str("Entity | Species    | Lifespan | Snapshots | Current Pos\n");
        output.push_str("-------|------------|----------|-----------|---------------------\n");

        for history in histories {
            let (x, y) = history.current_position().unwrap_or((0.0, 0.0));
            let species = history.species.as_deref().unwrap_or("Unknown");

            output.push_str(&format!(
                "{:<6} | {:<10} | {:<8} | {:<9} | ({:7.1}, {:7.1})\n",
                history.entity_id,
                species,
                history.lifespan_ticks(),
                history.snapshots.len(),
                x,
                y
            ));
        }

        output
    }
}

/// Tail a list of items, keeping only the last N
pub fn tail_items<T>(items: &[T], count: usize) -> &[T] {
    if items.len() > count {
        &items[items.len() - count..]
    } else {
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntitySnapshot;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_stuck_entities_text() {
        let stuck = vec![(1, 50), (2, 75), (3, 100)];
        let output = format_stuck_entities(&stuck, false);

        assert!(output.contains("Entity ID"));
        assert!(output.contains("1"));
        assert!(output.contains("50"));
    }

    #[test]
    fn test_format_stuck_entities_json() {
        let stuck = vec![(42, 100)];
        let output = format_stuck_entities(&stuck, true);

        assert!(output.contains("42"));
        assert!(output.contains("100"));
        assert!(output.contains("entity_id"));
    }

    #[test]
    fn test_format_entity_details() {
        let mut history = EntityHistory::new(42);
        history.species = Some("Rabbit".to_string());
        history.spawn_tick = Some(0);

        history.add_snapshot(EntitySnapshot {
            tick: 0,
            entity_id: 42,
            x: 100.0,
            y: 200.0,
            action: Some("Spawn".to_string()),
            species: Some("Rabbit".to_string()),
        });

        history.add_snapshot(EntitySnapshot {
            tick: 10,
            entity_id: 42,
            x: 110.0,
            y: 210.0,
            action: Some("Graze".to_string()),
            species: Some("Rabbit".to_string()),
        });

        let output = format_entity_details(&history, None, true, false);

        assert!(output.contains("Entity #42"));
        assert!(output.contains("Rabbit"));
        assert!(output.contains("Graze"));
    }

    #[test]
    fn test_format_entity_details_history_limit() {
        let mut history = EntityHistory::new(1);

        for tick in 0..100 {
            history.add_snapshot(EntitySnapshot {
                tick: tick as u64,
                entity_id: 1,
                x: tick as f64,
                y: 0.0,
                action: None,
                species: None,
            });
        }

        let output = format_entity_details(&history, Some(10), false, false);

        // Should only show last 10 ticks
        assert!(output.contains("99"));
        assert!(!output.contains("0:") || output.contains("10:"));
    }

    #[test]
    fn test_format_action_loops() {
        let loops = vec![
            ActionLoopDetection {
                entity_id: 1,
                action: "Graze".to_string(),
                repetitions: 50,
                start_tick: 0,
                end_tick: 50,
            },
        ];

        let output = format_action_loops(&loops, false);

        assert!(output.contains("Graze"));
        assert!(output.contains("50"));
    }

    #[test]
    fn test_export_timeline_json() {
        let mut history = EntityHistory::new(42);
        history.species = Some("Deer".to_string());
        history.spawn_tick = Some(0);

        history.add_snapshot(EntitySnapshot {
            tick: 0,
            entity_id: 42,
            x: 100.0,
            y: 200.0,
            action: Some("Spawn".to_string()),
            species: Some("Deer".to_string()),
        });

        history.add_snapshot(EntitySnapshot {
            tick: 10,
            entity_id: 42,
            x: 110.0,
            y: 210.0,
            action: Some("Graze".to_string()),
            species: Some("Deer".to_string()),
        });

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let result = export_timeline_json(path, &history, true, true);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("42"));
        assert!(content.contains("Deer"));
        assert!(content.contains("snapshots"));
    }

    #[test]
    fn test_tail_items() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let tailed = tail_items(&items, 3);
        assert_eq!(tailed.len(), 3);
        assert_eq!(tailed[0], 8);
        assert_eq!(tailed[2], 10);
    }

    #[test]
    fn test_tail_items_less_than_limit() {
        let items = vec![1, 2, 3];

        let tailed = tail_items(&items, 10);
        assert_eq!(tailed.len(), 3);
    }

    #[test]
    fn test_format_summary_csv() {
        let detector = StuckDetector::new(50);

        let output = format_summary(&detector, false, true);
        assert!(output.contains("entity_id,species"));
        assert!(output.contains("lifespan_ticks"));
    }
}
