use crate::state::{App, FileStatus};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Context", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(format!("v{}", app.version), Style::default().fg(Color::DarkGray)),
    ]));

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Model: ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.model_name(), Style::default().fg(Color::Cyan)),
    ]));

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Conversation: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.conversation_name, Style::default().fg(Color::White)),
    ]));

    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format_tokens(app.tokens_used, app.tokens_max), Style::default().fg(Color::Blue)),
    ]));

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Usage: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:.2}%", app.usage_percent()), Style::default().fg(Color::Green)),
    ]));

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Cost: ", Style::default().fg(Color::DarkGray)),
        Span::styled("$0.00", Style::default().fg(Color::White)),
    ]));

    lines.push(Line::from(""));

    let separator_width = area.width.saturating_sub(4) as usize;
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("─".repeat(separator_width), Style::default().fg(Color::DarkGray)),
    ]));

    let collapse_icon = if app.files_collapsed { "▶" } else { "▼" };
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::raw(collapse_icon),
        Span::styled(" Files", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));

    if !app.files_collapsed {
        lines.push(Line::from(""));
        
        for file in &app.files {
            let status_span = match file.status {
                FileStatus::Read => Span::styled(" [read]", Style::default().fg(Color::Blue)),
                FileStatus::Edited => Span::styled(" [edited]", Style::default().fg(Color::Yellow)),
                FileStatus::Added => Span::styled(" [added]", Style::default().fg(Color::Green)),
                FileStatus::Mentioned => Span::styled(" [mentioned]", Style::default().fg(Color::DarkGray)),
            };

            let path = if file.path.len() > 18 {
                format!("...{}", &file.path[file.path.len().saturating_sub(15)..])
            } else {
                file.path.clone()
            };

            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(path, Style::default().fg(Color::White)),
                status_span,
            ]));
        }

        if app.files.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled("No files yet", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("", Style::default()))
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn format_tokens(used: u64, max: u64) -> String {
    fn format_num(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }
    format!("{}/{}", format_num(used), format_num(max))
}