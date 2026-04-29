// src/ui/input_bar.rs

use crate::state::{App, Mode};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const MAX_VISIBLE_SUGGESTIONS: usize = 5;

pub fn render_input_bar(f: &mut Frame, app: &App, area: Rect) {
    let (mode_text, mode_color) = match app.mode {
        Mode::Build => ("BUILD", Color::Green),
        Mode::Plan => ("PLAN", Color::Yellow),
    };

    let suggestions = get_suggestions(&app.input);
    let show_dropdown = !suggestions.is_empty() && app.popup == crate::state::Popup::None;

    let input_content = if app.input.is_empty() {
        Line::from(Span::styled(" Type a message or / for commands...", Style::default().fg(Color::DarkGray)))
    } else {
        Line::from(vec![Span::raw(" "), Span::raw(&app.input)])
    };

    let mode_span = Span::styled(format!(" {} ", mode_text), Style::default().fg(mode_color).add_modifier(Modifier::BOLD));

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(mode_span)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(input_content).block(input_block);
    f.render_widget(paragraph, area);

    if show_dropdown {
        let visible_count = suggestions.len().min(MAX_VISIBLE_SUGGESTIONS);
        let dropdown_height = (visible_count + 2) as u16;
        let available_space = area.y;

        if available_space >= dropdown_height {
            let dropdown_area = Rect {
                x: area.x + 1,
                y: area.y.saturating_sub(dropdown_height),
                width: (area.width - 2).min(30),
                height: dropdown_height,
            };
            render_suggestions_dropdown(f, &suggestions, &app.input, app.suggestion_index, app.suggestion_scroll, dropdown_area);
        }
    }

    let cursor_x = area.x + 2 + app.input.len() as u16;
    let cursor_y = area.y + 1;
    f.set_cursor_position((cursor_x.min(area.x + area.width - 2), cursor_y));
}

pub fn get_suggestions(input: &str) -> Vec<&'static str> {
    if !input.starts_with('/') || input.contains(' ') {
        return vec![];
    }

    let cmds = ["/help", "/skill", "/model", "/tools", "/mode", "/clear", "/quit"];
    cmds.iter()
        .filter(|c| c.starts_with(input))
        .copied()
        .collect()
}

pub fn max_visible_suggestions() -> usize {
    MAX_VISIBLE_SUGGESTIONS
}

fn render_suggestions_dropdown(f: &mut Frame, suggestions: &[&str], current_input: &str, selected_idx: usize, scroll: usize, area: Rect) {
    if area.height < 3 || suggestions.is_empty() {
        return;
    }

    let mut lines: Vec<Line<'static>> = Vec::new();

    let visible_suggestions: Vec<_> = suggestions.iter()
        .skip(scroll)
        .take(MAX_VISIBLE_SUGGESTIONS)
        .collect();

    for (idx, suggestion) in visible_suggestions.iter().enumerate() {
        let actual_idx = scroll + idx;
        let is_selected = actual_idx == selected_idx;

        if is_selected {
            lines.push(Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Cyan)),
                Span::styled(suggestion.to_string(), Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            let input_len = current_input.len();

            if suggestion.starts_with(current_input) && input_len > 0 {
                let rest = suggestion[input_len..].to_string();
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(current_input.to_string(), Style::default().fg(Color::White)),
                    Span::styled(rest, Style::default().fg(Color::DarkGray)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(suggestion.to_string(), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Reset)); // Solid background

    let paragraph = Paragraph::new(lines).block(block);

    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(paragraph, area);
}
