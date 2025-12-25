mod cli;
mod entities;
mod output;
mod parser;

use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use cli::Args;
use entities::{StuckDetector, ActionLoopDetector, EntityHistory};
use parser::EntityParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = File::open(&args.logfile)?;
    let reader = BufReader::new(file);

    let mut parser = EntityParser::new();
    parser.parse_stream(reader, &args)?;

    match args.command {
        cli::Command::Stuck(opts) => {
            handle_stuck_command(&parser, &opts)?;
        }
        cli::Command::Entity(opts) => {
            handle_entity_command(&parser, &opts)?;
        }
        cli::Command::ActionLoops(opts) => {
            handle_action_loops_command(&parser, &opts)?;
        }
        cli::Command::Timeline(opts) => {
            handle_timeline_command(&parser, &opts)?;
        }
        cli::Command::Summary(opts) => {
            handle_summary_command(&parser, &opts)?;
        }
    }

    Ok(())
}

fn handle_stuck_command(
    parser: &EntityParser,
    opts: &cli::StuckOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut detector = StuckDetector::new(opts.threshold);

    // Build entity histories from parser snapshots
    for entity_id in parser.all_entity_ids() {
        if let Some(snapshots) = parser.get_snapshots(entity_id) {
            for snapshot in snapshots {
                detector.add_snapshot(snapshot.clone());
            }
        }
    }

    let mut stuck_list = detector.find_stuck_entities();

    // Apply tail filtering if specified
    if let Some(tail_count) = opts.tail {
        stuck_list = output::tail_items(&stuck_list, tail_count).to_vec();
    }

    let output = output::format_stuck_entities(&stuck_list, opts.json);
    println!("{}", output);

    Ok(())
}

fn handle_entity_command(
    parser: &EntityParser,
    opts: &cli::EntityOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(snapshots) = parser.get_snapshots(opts.entity_id) {
        let mut history = EntityHistory::new(opts.entity_id);

        for snapshot in snapshots {
            history.add_snapshot(snapshot.clone());
        }

        let output = output::format_entity_details(
            &history,
            opts.history,
            opts.show_actions,
            opts.json,
        );
        println!("{}", output);
    } else {
        eprintln!("Entity {} not found in log file", opts.entity_id);
        std::process::exit(1);
    }

    Ok(())
}

fn handle_action_loops_command(
    parser: &EntityParser,
    opts: &cli::ActionLoopsOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let detector = ActionLoopDetector::new(opts.threshold);

    let mut all_loops = Vec::new();

    for entity_id in parser.all_entity_ids() {
        if let Some(snapshots) = parser.get_snapshots(entity_id) {
            let mut history = EntityHistory::new(entity_id);

            for snapshot in snapshots {
                history.add_snapshot(snapshot.clone());
            }

            let loops = detector.find_loops(&history);
            all_loops.extend(loops);
        }
    }

    // Filter by action if specified
    if let Some(ref action_filter) = opts.action {
        all_loops.retain(|l| l.action == *action_filter);
    }

    // Sort by entity_id then repetitions (descending)
    all_loops.sort_by(|a, b| {
        a.entity_id
            .cmp(&b.entity_id)
            .then_with(|| b.repetitions.cmp(&a.repetitions))
    });

    // Apply tail filtering if specified
    if let Some(tail_count) = opts.tail {
        all_loops = output::tail_items(&all_loops, tail_count).to_vec();
    }

    let output = output::format_action_loops(&all_loops, opts.json);
    println!("{}", output);

    Ok(())
}

fn handle_timeline_command(
    parser: &EntityParser,
    opts: &cli::TimelineOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(snapshots) = parser.get_snapshots(opts.entity_id) {
        let mut history = EntityHistory::new(opts.entity_id);

        for snapshot in snapshots {
            history.add_snapshot(snapshot.clone());
        }

        if let Some(export_path) = &opts.export {
            output::export_timeline_json(export_path, &history, opts.positions, opts.actions)?;
            println!(
                "Timeline exported for entity {} to {}",
                opts.entity_id, export_path
            );
        } else {
            let formatted = output::format_entity_details(&history, None, opts.actions, opts.json);
            println!("{}", formatted);
        }
    } else {
        eprintln!("Entity {} not found in log file", opts.entity_id);
        std::process::exit(1);
    }

    Ok(())
}

fn handle_summary_command(
    parser: &EntityParser,
    opts: &cli::SummaryOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut detector = StuckDetector::new(50); // Default threshold for summary

    // Build detector with all snapshots
    for entity_id in parser.all_entity_ids() {
        if let Some(snapshots) = parser.get_snapshots(entity_id) {
            for snapshot in snapshots {
                detector.add_snapshot(snapshot.clone());
            }
        }
    }

    let output = output::format_summary(&detector, opts.json, opts.csv);
    println!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_main_with_minimal_log() {
        let log_data = r#"TICK=0: Starting
✅ Spawned rabbit #1: Test at IVec2(100, 100)
"#;

        let cursor = Cursor::new(log_data);
        let reader = BufReader::new(cursor);

        let args = Args::parse_from(&["sim-trace", "dummy.txt", "summary"]);
        let mut parser = EntityParser::new();

        let result = parser.parse_stream(reader, &args);
        assert!(result.is_ok());
        assert_eq!(parser.entity_count(), 1);
    }

    #[test]
    fn test_stuck_command_integration() {
        use std::io::Cursor;

        let log_data = r#"TICK=0: Start
✅ Spawned rabbit #1: Test at IVec2(100, 100)
"#;

        for i in 1..15 {
            println!("TICK={}: Processing", i);
            println!("Entity 1 at position (100.0, 100.0)");
        }

        // Parser initialized and basic log processed
        let cursor = Cursor::new(log_data);
        let reader = BufReader::new(cursor);

        let args = Args::parse_from(&["sim-trace", "dummy.txt", "summary"]);
        let mut parser = EntityParser::new();

        let result = parser.parse_stream(reader, &args);
        assert!(result.is_ok());
    }
}
