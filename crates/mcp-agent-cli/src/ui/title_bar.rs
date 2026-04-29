// src/ui/title_bar.rs

use crate::state::{App, Mode};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_title_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode_style = match app.mode {
        Mode::Build => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        Mode::Plan => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    };

    let mode_text = match app.mode {
        Mode::Build => "BUILD",
        Mode::Plan => "PLAN",
    };

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled("MCP Agent", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("v{}", app.version), Style::default().fg(Color::White)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(mode_text, mode_style),
        Span::raw(" "),
    ]);

    let paragraph = Paragraph::new(title)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(paragraph, area);
}
