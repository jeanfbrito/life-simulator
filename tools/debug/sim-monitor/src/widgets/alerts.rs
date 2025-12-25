//! Recent alerts widget

use crate::api_client::Alert;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Render the alerts panel
pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, alerts: &[Alert]) {
    let items: Vec<ListItem> = if alerts.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "No recent alerts",
            Style::default().fg(Color::Gray),
        )]))]
    } else {
        alerts.iter().map(|alert| {
            let color = match alert.alert_type.as_str() {
                "TpsBelow10" => Color::Red,
                "EntitiesStuck" => Color::Yellow,
                "PopulationCrash" => Color::Red,
                "AiLoops" => Color::Yellow,
                _ => Color::White,
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", alert.tick),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&alert.message, Style::default().fg(color)),
            ]))
        }).collect()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Recent Alerts "),
    );

    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_color_mapping() {
        assert_eq!(get_alert_color("TpsBelow10"), Color::Red);
        assert_eq!(get_alert_color("EntitiesStuck"), Color::Yellow);
        assert_eq!(get_alert_color("PopulationCrash"), Color::Red);
        assert_eq!(get_alert_color("AiLoops"), Color::Yellow);
        assert_eq!(get_alert_color("Unknown"), Color::White);
    }

    fn get_alert_color(alert_type: &str) -> Color {
        match alert_type {
            "TpsBelow10" => Color::Red,
            "EntitiesStuck" => Color::Yellow,
            "PopulationCrash" => Color::Red,
            "AiLoops" => Color::Yellow,
            _ => Color::White,
        }
    }
}
