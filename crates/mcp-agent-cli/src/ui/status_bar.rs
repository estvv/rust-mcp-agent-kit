// src/ui/status_bar.rs

use crate::state::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let skill_name = app.skill_name();

    let status_color = if app.status.contains("Thinking") || app.streaming {
        Color::Yellow
    } else if app.status.contains("Error") || app.status.contains("Failed") {
        Color::Red
    } else {
        Color::Green
    };

    let text = Line::from(vec![
        Span::styled("Skill: ", Style::default().fg(Color::DarkGray)),
        Span::styled(skill_name, Style::default().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled("Tools: ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.tool_count().to_string(), Style::default().fg(Color::Blue)),
        Span::raw(" | "),
        Span::styled(app.status.clone(), Style::default().fg(status_color)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}
