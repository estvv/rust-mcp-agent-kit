use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_status_bar(f, app, chunks[0]);
    render_messages(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let profile_name = app.profile.as_ref()
        .map(|p| p.name())
        .unwrap_or("none");

    let text = Line::from(vec![
        ratatui::text::Span::styled(
            format!("Model: {} | Profile: {} | Tools: {}", 
                app.model, profile_name, app.tool_count()),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Status"));

    f.render_widget(paragraph, area);
}

fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app.messages
        .iter()
        .flat_map(|(sender, content)| {
            let sender_style = match sender.as_str() {
                "User" => Style::default().fg(Color::Green),
                "System" => Style::default().fg(Color::Yellow),
                "Error" => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::White),
            };

            vec![
                Line::styled(format!("[{}]", sender), sender_style),
                Line::styled(content.clone(), Style::default().fg(Color::White)),
                Line::from(""),
            ]
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Chat"));

    f.render_widget(paragraph, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let text = Line::from(format!("> {}", app.input));

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Input (q to quit)"));

    f.render_widget(paragraph, area);
}