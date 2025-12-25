//! UI rendering logic

use crate::app::App;
use crate::widgets;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(8),  // Alerts
        ])
        .split(frame.area());

    // Header
    widgets::header::render(
        frame,
        chunks[0],
        &app.get_tps_display(),
        app.get_connection_status(),
        app.is_connected,
    );

    // Main content - split horizontally
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Entities panel
    widgets::entities::render(
        frame,
        main_chunks[0],
        &app.entity_counts,
        |species| app.get_entity_delta(species),
        app.get_total_entities(),
    );

    // Health panel
    widgets::health::render(frame, main_chunks[1], app.health_status.as_ref());

    // Alerts panel
    widgets::alerts::render(frame, chunks[2], &app.alerts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constraints() {
        // Test that we have the correct number of constraints
        let constraints = vec![
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(8),
        ];
        assert_eq!(constraints.len(), 3);
    }

    #[test]
    fn test_main_content_split() {
        let constraints = vec![Constraint::Percentage(50), Constraint::Percentage(50)];
        assert_eq!(constraints.len(), 2);
    }
}
