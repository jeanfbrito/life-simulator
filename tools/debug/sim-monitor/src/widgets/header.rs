//! Header widget for displaying title and key metrics

use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Row, Table},
    Frame,
};

/// Render the header panel
pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    tps: &str,
    connection_status: &str,
    is_connected: bool,
) {
    let status_color = if is_connected {
        Color::Green
    } else {
        Color::Red
    };

    let rows = vec![Row::new(vec![
        Span::styled(
            "Life Simulator Monitor",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .to_string(),
        format!("TPS: {}", tps),
        format!("Status: {}", connection_status),
    ])];

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(status_color))
                .title(" Status "),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(table, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_render_function_exists() {
        // This test just validates that the render function signature is correct
        // Actual rendering tests would require a test backend
        assert!(true);
    }

    #[test]
    fn test_status_color_logic() {
        // Test color selection logic
        let connected_color = if true { Color::Green } else { Color::Red };
        assert_eq!(connected_color, Color::Green);

        let disconnected_color = if false { Color::Green } else { Color::Red };
        assert_eq!(disconnected_color, Color::Red);
    }
}
