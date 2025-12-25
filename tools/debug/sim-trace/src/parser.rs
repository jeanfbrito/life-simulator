use regex::Regex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use crate::entities::EntitySnapshot;
use crate::cli::Args;

/// Parses entity data from simulator log files
pub struct EntityParser {
    entity_snapshots: HashMap<u32, Vec<EntitySnapshot>>,
    spawn_regex: Regex,
    position_regex: Regex,
    action_regex: Regex,
}

impl EntityParser {
    pub fn new() -> Self {
        EntityParser {
            entity_snapshots: HashMap::new(),
            // Pattern: ✅ Spawned rabbit #42: Name 🐇 at IVec2(100, 200) or Spawned Deer #5: Fawn at (150, 250)
            spawn_regex: Regex::new(
                r"(?:Spawned|spawned)\s+(\w+)\s+#(\d+):[^(]*(?:IVec2|at)[\(\[](\d+(?:\.\d+)?),\s*(\d+(?:\.\d+)?)[\)\]]"
            )
            .unwrap(),
            // Pattern: Entity 42 at position (100.5, 200.5) or similar
            position_regex: Regex::new(
                r"Entity\s+(\d+).*(?:at|position)\s*[\(\[](\d+(?:\.\d+)?),\s*(\d+(?:\.\d+)?)[\)\]]"
            )
            .unwrap(),
            // Pattern: "current_action": "Graze" or action=Graze
            action_regex: Regex::new(
                r#"(?:"current_action"|action)\s*[:=]\s*"?(\w+)"?"#
            )
            .unwrap(),
        }
    }

    pub fn parse_stream<R: Read>(
        &mut self,
        reader: BufReader<R>,
        _args: &Args,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut current_tick = 0u64;

        for line in reader.lines() {
            let line = line?;

            // Try to extract tick information from common log patterns
            if let Some(tick) = extract_tick(&line) {
                current_tick = tick;
            }

            // Try to parse entity spawn messages
            if let Some((species, entity_id, x, y)) = self.extract_spawn(&line) {
                let snapshot = EntitySnapshot {
                    tick: current_tick,
                    entity_id,
                    x: x as f64,
                    y: y as f64,
                    action: Some("Spawn".to_string()),
                    species: Some(species),
                };

                self.entity_snapshots
                    .entry(entity_id)
                    .or_insert_with(Vec::new)
                    .push(snapshot);
            }

            // Try to parse entity position messages
            if let Some((entity_id, x, y)) = self.extract_position(&line) {
                let entry = self
                    .entity_snapshots
                    .entry(entity_id)
                    .or_insert_with(Vec::new);

                // Only add if position is different from last known position
                if let Some(last) = entry.last() {
                    if (last.x - x).abs() > 0.001 || (last.y - y).abs() > 0.001 {
                        let snapshot = EntitySnapshot {
                            tick: current_tick,
                            entity_id,
                            x,
                            y,
                            action: None,
                            species: last.species.clone(),
                        };
                        entry.push(snapshot);
                    }
                }
            }

            // Try to parse action messages
            if let Some((entity_id, action)) = self.extract_action(&line) {
                if let Some(entry) = self.entity_snapshots.get_mut(&entity_id) {
                    if let Some(last) = entry.last_mut() {
                        if last.tick == current_tick {
                            last.action = Some(action);
                        } else {
                            let snapshot = EntitySnapshot {
                                tick: current_tick,
                                entity_id,
                                x: last.x,
                                y: last.y,
                                action: Some(action),
                                species: last.species.clone(),
                            };
                            entry.push(snapshot);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_spawn(&self, line: &str) -> Option<(String, u32, f64, f64)> {
        self.spawn_regex
            .captures(line)
            .and_then(|caps| {
                let species = caps.get(1)?.as_str().to_string();
                let entity_id = caps.get(2)?.as_str().parse().ok()?;
                let x = caps.get(3)?.as_str().parse().ok()?;
                let y = caps.get(4)?.as_str().parse().ok()?;
                Some((species, entity_id, x, y))
            })
    }

    fn extract_position(&self, line: &str) -> Option<(u32, f64, f64)> {
        self.position_regex
            .captures(line)
            .and_then(|caps| {
                let entity_id = caps.get(1)?.as_str().parse().ok()?;
                let x = caps.get(2)?.as_str().parse().ok()?;
                let y = caps.get(3)?.as_str().parse().ok()?;
                Some((entity_id, x, y))
            })
    }

    fn extract_action(&self, line: &str) -> Option<(u32, String)> {
        // First try to extract entity_id from the line
        let entity_regex = Regex::new(r"Entity\s+(\d+)|#(\d+)").unwrap();
        let entity_id = entity_regex.captures(line).and_then(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .and_then(|m| m.as_str().parse().ok())
        })?;

        // Then extract action
        let action = self.action_regex.captures(line).and_then(|caps| {
            caps.get(1).map(|m| m.as_str().to_string())
        })?;

        Some((entity_id, action))
    }

    pub fn get_snapshots(&self, entity_id: u32) -> Option<&[EntitySnapshot]> {
        self.entity_snapshots.get(&entity_id).map(|v| v.as_slice())
    }

    pub fn all_entity_ids(&self) -> Vec<u32> {
        let mut ids: Vec<_> = self.entity_snapshots.keys().copied().collect();
        ids.sort();
        ids
    }

    pub fn entity_count(&self) -> usize {
        self.entity_snapshots.len()
    }

    pub fn total_snapshots(&self) -> usize {
        self.entity_snapshots.values().map(|v| v.len()).sum()
    }
}

/// Extract tick number from common log patterns
fn extract_tick(line: &str) -> Option<u64> {
    // Pattern: TICK=12345 or tick: 12345 or tick 12345
    let tick_regex = Regex::new(r"(?i)TICK\s*[:=]?\s*(\d+)").unwrap();
    tick_regex.captures(line).and_then(|caps| {
        caps.get(1)?.as_str().parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_extract_tick() {
        assert_eq!(extract_tick("TICK=12345: Processing"), Some(12345));
        assert_eq!(extract_tick("[TICK 999] update"), Some(999));
        assert_eq!(extract_tick("tick: 500"), Some(500));
        assert_eq!(extract_tick("no tick here"), None);
    }

    #[test]
    fn test_spawn_extraction() {
        let parser = EntityParser::new();

        let result = parser.extract_spawn("✅ Spawned rabbit #42: Name 🐇 at IVec2(100, 200)");
        assert!(result.is_some());

        let (species, entity_id, x, y) = result.unwrap();
        assert_eq!(species, "rabbit");
        assert_eq!(entity_id, 42);
        assert_eq!(x, 100.0);
        assert_eq!(y, 200.0);
    }

    #[test]
    fn test_spawn_extraction_alternative_format() {
        let parser = EntityParser::new();

        // Test with IVec2 format with different case
        let result = parser.extract_spawn("✅ Spawned Deer #5: Bambi at IVec2(150, 250)");
        assert!(result.is_some());

        let (species, entity_id, _x, _y) = result.unwrap();
        assert_eq!(species, "Deer");
        assert_eq!(entity_id, 5);
    }

    #[test]
    fn test_position_extraction() {
        let parser = EntityParser::new();

        let result = parser.extract_position("Entity 42 at position (123.5, 456.7)");
        assert!(result.is_some());

        let (entity_id, x, y) = result.unwrap();
        assert_eq!(entity_id, 42);
        assert_eq!(x, 123.5);
        assert_eq!(y, 456.7);
    }

    #[test]
    fn test_action_extraction() {
        let parser = EntityParser::new();

        let line = r#"Entity 42 status: {"current_action": "Graze", "idle": false}"#;
        let result = parser.extract_action(line);
        assert!(result.is_some());

        let (entity_id, action) = result.unwrap();
        assert_eq!(entity_id, 42);
        assert_eq!(action, "Graze");
    }

    #[test]
    fn test_entity_parser_full_integration() {
        use std::io::Cursor;

        let log_data = r#"TICK=0: Starting simulation
✅ Spawned rabbit #1: Bugs at IVec2(50, 50)
✅ Spawned deer #2: Bambi at IVec2(100, 100)
TICK=10: Processing
Entity 1 at position (51.5, 51.5)
Entity 1 status: {"current_action": "Graze"}
TICK=20: Processing
Entity 1 at position (52.0, 52.0)
Entity 2 at position (105.0, 105.0)
"#;

        let cursor = Cursor::new(log_data);
        let reader = BufReader::new(cursor);

        let args = Args::parse_from(&["sim-trace", "dummy.txt", "summary"]);
        let mut parser = EntityParser::new();

        parser.parse_stream(reader, &args).unwrap();

        assert_eq!(parser.entity_count(), 2);
        assert!(parser.get_snapshots(1).is_some());
        assert!(parser.get_snapshots(2).is_some());
    }

    #[test]
    fn test_position_unchanged_filtering() {
        use std::io::Cursor;

        let log_data = r#"TICK=0: Start
✅ Spawned rabbit #1: Test at IVec2(100, 100)
TICK=1: Processing
Entity 1 at position (100.0, 100.0)
TICK=2: Processing
Entity 1 at position (100.0001, 100.0)
TICK=3: Processing
Entity 1 at position (105.0, 100.0)
"#;

        let cursor = Cursor::new(log_data);
        let reader = BufReader::new(cursor);

        let args = Args::parse_from(&["sim-trace", "dummy.txt", "summary"]);
        let mut parser = EntityParser::new();

        parser.parse_stream(reader, &args).unwrap();

        // Should have significant position changes (spawn + meaningful moves)
        let snapshots = parser.get_snapshots(1).unwrap();
        assert!(snapshots.len() >= 2); // At least spawn and final position
    }
}
