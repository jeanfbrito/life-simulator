//! Entity table widget

use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::collections::HashMap;

/// Render the entities panel
pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    entity_counts: &HashMap<String, i32>,
    get_delta: impl Fn(&str) -> i32,
    total: i32,
) {
    let mut rows = Vec::new();

    // Sort species alphabetically
    let mut species: Vec<_> = entity_counts.keys().collect();
    species.sort();

    for species_name in species {
        let count = entity_counts.get(species_name.as_str()).copied().unwrap_or(0);
        let delta = get_delta(species_name);

        let delta_text = if delta > 0 {
            format!("(+{})", delta)
        } else if delta < 0 {
            format!("({})", delta)
        } else {
            String::new()
        };

        let delta_color = if delta > 0 {
            Color::Green
        } else if delta < 0 {
            Color::Red
        } else {
            Color::Gray
        };

        rows.push(Row::new(vec![
            Cell::from(species_name.as_str()),
            Cell::from(count.to_string()),
            Cell::from(Span::styled(delta_text, Style::default().fg(delta_color))),
        ]));
    }

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Species", "Count", "Delta"])
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Entities ({}) ", total)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(table, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_formatting() {
        assert_eq!(format_delta(5), "(+5)");
        assert_eq!(format_delta(-3), "(-3)");
        assert_eq!(format_delta(0), "");
    }

    #[test]
    fn test_delta_color_selection() {
        assert_eq!(get_delta_color(5), Color::Green);
        assert_eq!(get_delta_color(-3), Color::Red);
        assert_eq!(get_delta_color(0), Color::Gray);
    }

    fn format_delta(delta: i32) -> String {
        if delta > 0 {
            format!("(+{})", delta)
        } else if delta < 0 {
            format!("({})", delta)
        } else {
            String::new()
        }
    }

    fn get_delta_color(delta: i32) -> Color {
        if delta > 0 {
            Color::Green
        } else if delta < 0 {
            Color::Red
        } else {
            Color::Gray
        }
    }
}
