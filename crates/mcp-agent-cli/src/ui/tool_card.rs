use crate::state::ToolCall;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

#[allow(dead_code)]
pub fn render_tool_card(f: &mut Frame, tool: &ToolCall, area: Rect) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    
    let status_icon = if tool.is_error {
        "[X]"
    } else if tool.result.is_some() {
        "[OK]"
    } else {
        "..."
    };
    
    let expand_icon = if tool.expanded { "[v]" } else { "[>]" };
    
    let status_color = if tool.is_error {
        Color::Red
    } else if tool.result.is_some() {
        Color::Green
    } else {
        Color::Yellow
    };

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("┌─", Style::default().fg(Color::DarkGray)),
        Span::raw(" [T] "),
        Span::styled(format!("Tool: {} ", tool.name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(expand_icon),
        Span::styled("─".repeat(15), Style::default().fg(Color::DarkGray)),
        Span::styled("┐", Style::default().fg(Color::DarkGray)),
    ]));

    if tool.expanded {
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Parameters:", Style::default().fg(Color::Magenta)),
        ]));
        
        for arg_line in tool.arguments.lines() {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("│   ", Style::default().fg(Color::DarkGray)),
                Span::styled(arg_line.to_string(), Style::default().fg(Color::White)),
            ]));
        }
        
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("─".repeat(20), Style::default().fg(Color::DarkGray)),
        ]));
        
        if let Some(ref result) = tool.result {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} Result:", status_icon), Style::default().fg(status_color)),
            ]));
            
            for result_line in result.lines() {
                lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("│   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(result_line.to_string(), Style::default().fg(Color::White)),
                ]));
            }
        }
    } else {
        if let Some(ref result) = tool.result {
            let result_preview: String = result.lines().next().unwrap_or("").chars().take(50).collect();
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
                Span::styled(format!("Result: {}", result_preview), Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled("... Running...", Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("└", Style::default().fg(Color::DarkGray)),
        Span::styled("─".repeat(25), Style::default().fg(Color::DarkGray)),
        Span::styled("┘", Style::default().fg(Color::DarkGray)),
    ]));

    let block = Block::default().borders(Borders::NONE);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}