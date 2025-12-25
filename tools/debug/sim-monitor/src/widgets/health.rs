//! Health status widget

use crate::api_client::HealthStatus;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Render the health status panel
pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, health: Option<&HealthStatus>) {
    let items = if let Some(health_status) = health {
        get_health_items(health_status)
    } else {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "No health data",
            Style::default().fg(Color::Gray),
        )]))]
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Health Status "),
    );

    frame.render_widget(list, area);
}

fn get_health_items(health: &HealthStatus) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();

    // Overall status
    let (status_icon, status_color) = match health.status.as_str() {
        "ok" => ("✓", Color::Green),
        "degraded" => ("⚠", Color::Yellow),
        "critical" => ("✗", Color::Red),
        _ => ("?", Color::Gray),
    };

    items.push(ListItem::new(Line::from(vec![
        Span::styled(status_icon, Style::default().fg(status_color)),
        Span::raw(" Overall: "),
        Span::styled(
            health.status.clone(),
            Style::default().fg(status_color),
        ),
    ])));

    // TPS status
    let tps_status = if health.current_tps >= 59.0 {
        ("✓", Color::Green, "Excellent")
    } else if health.current_tps >= 30.0 {
        ("✓", Color::Green, "Good")
    } else if health.current_tps >= 10.0 {
        ("⚠", Color::Yellow, "OK")
    } else {
        ("✗", Color::Red, "Poor")
    };

    items.push(ListItem::new(Line::from(vec![
        Span::styled(tps_status.0, Style::default().fg(tps_status.1)),
        Span::raw(format!(" TPS: {:.1} ({})", health.current_tps, tps_status.2)),
    ])));

    // Alert counts
    for (alert_type, count) in &health.alerts {
        if *count > 0 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("⚠", Style::default().fg(Color::Yellow)),
                Span::raw(format!(" {}: {}", alert_type, count)),
            ])));
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_health_status_icons() {
        let mut alerts = HashMap::new();
        alerts.insert("tps_below_10".to_string(), 0);

        let health = HealthStatus {
            status: "ok".to_string(),
            alerts,
            current_tps: 60.0,
            total_alerts: Some(0),
            is_healthy: Some(true),
        };

        let items = get_health_items(&health);
        assert!(!items.is_empty());
    }

    #[test]
    fn test_tps_status_classification() {
        assert_eq!(classify_tps(60.0), ("✓", Color::Green, "Excellent"));
        assert_eq!(classify_tps(45.0), ("✓", Color::Green, "Good"));
        assert_eq!(classify_tps(15.0), ("⚠", Color::Yellow, "OK"));
        assert_eq!(classify_tps(5.0), ("✗", Color::Red, "Poor"));
    }

    fn classify_tps(tps: f64) -> (&'static str, Color, &'static str) {
        if tps >= 59.0 {
            ("✓", Color::Green, "Excellent")
        } else if tps >= 30.0 {
            ("✓", Color::Green, "Good")
        } else if tps >= 10.0 {
            ("⚠", Color::Yellow, "OK")
        } else {
            ("✗", Color::Red, "Poor")
        }
    }

    #[test]
    fn test_status_color_mapping() {
        assert_eq!(get_status_color("ok"), Color::Green);
        assert_eq!(get_status_color("degraded"), Color::Yellow);
        assert_eq!(get_status_color("critical"), Color::Red);
        assert_eq!(get_status_color("unknown"), Color::Gray);
    }

    fn get_status_color(status: &str) -> Color {
        match status {
            "ok" => Color::Green,
            "degraded" => Color::Yellow,
            "critical" => Color::Red,
            _ => Color::Gray,
        }
    }
}
