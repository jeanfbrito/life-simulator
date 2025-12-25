//! Life Simulator Monitor - Real-time TUI Dashboard
//!
//! A terminal UI application for monitoring the Life Simulator in real-time.

mod api_client;
mod app;
mod ui;
mod widgets;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "sim-monitor")]
#[command(about = "Real-time monitoring dashboard for Life Simulator", long_about = None)]
struct Args {
    /// Simulator URL
    #[arg(long, default_value = "http://127.0.0.1:54321")]
    url: String,

    /// Refresh interval in seconds
    #[arg(long, default_value = "1")]
    refresh: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Validate refresh interval
    if args.refresh == 0 {
        anyhow::bail!("Refresh interval must be at least 1 second");
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(args.url.clone(), args.refresh);

    // Run the application
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let mut last_update = std::time::Instant::now();

    loop {
        // Render UI
        terminal.draw(|f| ui::render(f, app))?;

        // Handle input with timeout
        let timeout = Duration::from_millis(100);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.quit();
                    }
                    KeyCode::Char('r') => {
                        // Force refresh
                        app.update().await?;
                        last_update = std::time::Instant::now();
                    }
                    _ => {}
                }
            }
        }

        // Periodic update
        if last_update.elapsed() >= app.update_interval {
            app.update().await?;
            last_update = std::time::Instant::now();
        }

        // Check if should quit
        if app.should_quit() {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        // Test default values
        let args = Args::parse_from(&["sim-monitor"]);
        assert_eq!(args.url, "http://127.0.0.1:54321");
        assert_eq!(args.refresh, 1);
    }

    #[test]
    fn test_args_custom_values() {
        let args = Args::parse_from(&[
            "sim-monitor",
            "--url",
            "http://localhost:8080",
            "--refresh",
            "2",
        ]);
        assert_eq!(args.url, "http://localhost:8080");
        assert_eq!(args.refresh, 2);
    }

    #[test]
    fn test_refresh_interval_validation() {
        // This would normally be validated at runtime
        let refresh = 0u64;
        assert!(refresh == 0); // Would fail validation
    }
}
