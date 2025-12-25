use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "sim-trace")]
#[command(about = "Entity behavior analyzer for Life Simulator - tracks movement, actions, and anomalies", long_about = None)]
pub struct Args {
    /// Path to log file
    pub logfile: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Find entities that are stuck in place
    Stuck(StuckOptions),

    /// Analyze specific entity's history and behavior
    #[command(alias = "e")]
    Entity(EntityOptions),

    /// Detect repeated action sequences (action loops)
    #[command(alias = "loops")]
    ActionLoops(ActionLoopsOptions),

    /// Generate entity timeline export
    #[command(alias = "t")]
    Timeline(TimelineOptions),

    /// Summary statistics for all entities
    #[command(alias = "s")]
    Summary(SummaryOptions),
}

#[derive(Parser, Debug)]
pub struct StuckOptions {
    /// Minimum consecutive ticks without movement (default: 50)
    #[arg(long, default_value = "50")]
    pub threshold: usize,

    /// Show last N stuck entities
    #[arg(long)]
    pub tail: Option<usize>,

    /// Export as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct EntityOptions {
    /// Entity ID to analyze
    #[arg(long, short)]
    pub entity_id: u32,

    /// Show last N ticks of history
    #[arg(long)]
    pub history: Option<usize>,

    /// Include action sequence
    #[arg(long, default_value = "true")]
    pub show_actions: bool,

    /// Export as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct ActionLoopsOptions {
    /// Minimum consecutive repetitions to flag as loop (default: 20)
    #[arg(long, default_value = "20")]
    pub threshold: usize,

    /// Show last N action loops
    #[arg(long)]
    pub tail: Option<usize>,

    /// Export as JSON
    #[arg(long)]
    pub json: bool,

    /// Only show this action type
    #[arg(long)]
    pub action: Option<String>,
}

#[derive(Parser, Debug)]
pub struct TimelineOptions {
    /// Entity ID to generate timeline for
    #[arg(long, short = 'e')]
    pub entity_id: u32,

    /// Export file path (JSON format)
    #[arg(long, short = 'o')]
    pub export: Option<String>,

    /// Include full position history
    #[arg(long, default_value = "true")]
    pub positions: bool,

    /// Include action history
    #[arg(long, default_value = "true")]
    pub actions: bool,

    /// Output as JSON (when not exporting to file)
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct SummaryOptions {
    /// Export as JSON
    #[arg(long)]
    pub json: bool,

    /// Export as CSV
    #[arg(long)]
    pub csv: bool,

    /// Show top N entities by lifespan
    #[arg(long)]
    pub top_entities: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stuck_command_parsing() {
        let args = Args::parse_from(&["sim-trace", "logfile.txt", "stuck", "--threshold", "100"]);

        assert_eq!(args.logfile, "logfile.txt");
        match args.command {
            Command::Stuck(opts) => assert_eq!(opts.threshold, 100),
            _ => panic!("Expected Stuck command"),
        }
    }

    #[test]
    fn test_entity_command_parsing() {
        let args = Args::parse_from(&["sim-trace", "logfile.txt", "entity", "--entity-id", "42"]);

        match args.command {
            Command::Entity(opts) => {
                assert_eq!(opts.entity_id, 42);
                assert!(opts.show_actions);
            }
            _ => panic!("Expected Entity command"),
        }
    }

    #[test]
    fn test_action_loops_command_parsing() {
        let args = Args::parse_from(&[
            "sim-trace",
            "logfile.txt",
            "action-loops",
            "--threshold",
            "30",
            "--action",
            "Graze",
        ]);

        match args.command {
            Command::ActionLoops(opts) => {
                assert_eq!(opts.threshold, 30);
                assert_eq!(opts.action, Some("Graze".to_string()));
            }
            _ => panic!("Expected ActionLoops command"),
        }
    }

    #[test]
    fn test_timeline_command_parsing() {
        let args = Args::parse_from(&[
            "sim-trace",
            "logfile.txt",
            "timeline",
            "--entity-id",
            "5",
            "--export",
            "timeline.json",
        ]);

        match args.command {
            Command::Timeline(opts) => {
                assert_eq!(opts.entity_id, 5);
                assert_eq!(opts.export, Some("timeline.json".to_string()));
            }
            _ => panic!("Expected Timeline command"),
        }
    }

    #[test]
    fn test_summary_command_parsing() {
        let args = Args::parse_from(&[
            "sim-trace",
            "logfile.txt",
            "summary",
            "--json",
            "--top-entities",
            "10",
        ]);

        match args.command {
            Command::Summary(opts) => {
                assert!(opts.json);
                assert_eq!(opts.top_entities, Some(10));
            }
            _ => panic!("Expected Summary command"),
        }
    }
}
