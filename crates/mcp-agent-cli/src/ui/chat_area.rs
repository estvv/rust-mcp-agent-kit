// src/ui/chat_area.rs

use crate::state::{App, MessageContent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn render_chat_area(f: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let chat_width = area.width.saturating_sub(2) as usize;

    for msg in &app.messages {
        match &msg.content {
            MessageContent::Text(content) => {
                let (sender_style, bg_color) = get_sender_style(&msg.sender);

                lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(format!("[{}]", msg.sender), sender_style.add_modifier(Modifier::BOLD)),
                ]));

                for line in format_content(&content, bg_color) {
                    lines.push(line);
                }

                lines.push(Line::from(""));
            }
            MessageContent::Reasoning(content) => {
                let (sender_style, bg_color) = get_sender_style(&msg.sender);

                lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(format!("[{}]", msg.sender), sender_style.add_modifier(Modifier::BOLD)),
                    Span::styled(" (reasoning)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ]));

                for line in format_content_italic(&content, bg_color) {
                    lines.push(line);
                }

                lines.push(Line::from(""));
            }
            MessageContent::Tools(tools) => {
                for tool in tools {
                    let card_lines = render_tool_card_inline(&tool, chat_width);
                    for line in card_lines {
                        lines.push(line);
                    }
                    lines.push(Line::from(""));
                }
            }
        }
    }

    if app.streaming {
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("[Assistant]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));


        if let Some(ref reasoning) = app.streaming_reasoning {
            if !reasoning.is_empty() {
                for line in format_content_italic(reasoning, Color::Rgb(0, 40, 40)) {
                    lines.push(line);
                }
            }
        }


        if let Some(ref msg) = app.streaming_message {
            if !msg.is_empty() {
                for line in format_content(msg, Color::Rgb(0, 40, 40)) {
                    lines.push(line);
                }
            }
        }


        let spinner = ['|', '/', '-', '\\'];
        let spinner_char = spinner[app.spinner_frame % spinner.len()];
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(spinner_char.to_string(), Style::default().fg(Color::Yellow)),
        ]));
    }


    let visible_height = area.height.saturating_sub(2) as usize;
    let content_width = area.width.saturating_sub(2) as usize;


    let mut visual_line_count = 0usize;
    for line in &lines {
        let line_len: usize = line.spans.iter().map(|s| s.content.len()).sum();
        if line_len == 0 {
            visual_line_count += 1;
        } else {
            let wrapped = (line_len + content_width.max(1) - 1) / content_width.max(1);
            visual_line_count += wrapped.max(1);
        }
    }

    app.max_scroll = visual_line_count.saturating_sub(visible_height);


    let scroll_y = if app.follow_bottom {
        app.max_scroll as u16
    } else {
        app.scroll_offset.min(app.max_scroll) as u16
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Chat ")
            .border_style(Style::default().fg(Color::DarkGray)))
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0));

    f.render_widget(paragraph, area);
}

fn get_sender_style(sender: &str) -> (Style, Color) {
    match sender {
        "User" => (Style::default().fg(Color::Green), Color::Rgb(0, 40, 0)),
        "Assistant" => (Style::default().fg(Color::Cyan), Color::Rgb(0, 40, 40)),
        "System" => (Style::default().fg(Color::Yellow), Color::Rgb(40, 40, 0)),
        "Tool" => (Style::default().fg(Color::Magenta), Color::Rgb(40, 0, 40)),
        "Error" => (Style::default().fg(Color::Red), Color::Rgb(40, 0, 0)),
        _ => (Style::default().fg(Color::White), Color::Reset),
    }
}

fn format_content(text: &str, bg_color: Color) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in text.lines() {
        let line = line.strip_prefix("  ").unwrap_or(line);

        if line.starts_with("### ") {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    line.strip_prefix("### ").unwrap_or(line).to_string(),
                    Style::default().fg(Color::Yellow).bg(bg_color).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if line.starts_with("## ") {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    line.strip_prefix("## ").unwrap_or(line).to_string(),
                    Style::default().fg(Color::Cyan).bg(bg_color).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if line.starts_with("# ") {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    line.strip_prefix("# ").unwrap_or(line).to_string(),
                    Style::default().fg(Color::Green).bg(bg_color).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled("* ", Style::default().fg(Color::Cyan).bg(bg_color)),
                Span::styled(line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")).unwrap_or(line).to_string(), Style::default().bg(bg_color)),
            ]));
        } else if line.starts_with("```") {
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(line.to_string(), Style::default().fg(Color::DarkGray).bg(bg_color)),
            ]));
        } else if line.starts_with("`") && line.ends_with("`") && line.len() > 2 {
            let code = line[1..line.len()-1].to_string();
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(code, Style::default().fg(Color::Black).bg(Color::Gray)),
            ]));
        } else {
            let mut spans = vec![Span::raw(" ")];
            spans.extend(parse_inline_code(line, bg_color));
            lines.push(Line::from(spans));
        }
    }

    lines
}

fn format_content_italic(text: &str, bg_color: Color) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in text.lines() {
        let line = line.strip_prefix("  ").unwrap_or(line);
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(line.to_string(), Style::default().fg(Color::DarkGray).bg(bg_color).add_modifier(Modifier::ITALIC)),
        ]));
    }

    lines
}

fn parse_inline_code(line: &str, bg_color: Color) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut in_code = false;
    let mut current = String::new();

    for ch in line.chars() {
        if ch == '`' {
            if !current.is_empty() {
                if in_code {
                    spans.push(Span::styled(current.clone(), Style::default().fg(Color::Black).bg(Color::Gray)));
                } else {
                    spans.push(Span::styled(current.clone(), Style::default().bg(bg_color)));
                }
                current.clear();
            }
            in_code = !in_code;
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        if in_code {
            spans.push(Span::styled(current, Style::default().fg(Color::Black).bg(Color::Gray)));
        } else {
            spans.push(Span::styled(current, Style::default().bg(bg_color)));
        }
    }

    if spans.is_empty() {
        vec![Span::styled(line.to_string(), Style::default().bg(bg_color))]
    } else {
        spans
    }
}

fn render_tool_card_inline(tool: &crate::state::ToolCall, chat_width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let status_icon = if tool.is_error {
        "[X]"
    } else if tool.result.is_some() {
        "[OK]"
    } else {
        "[...]"
    };

    let expand_icon = if tool.expanded { "[-]" } else { "[+]" };

    let status_color = if tool.is_error {
        Color::Red
    } else if tool.result.is_some() {
        Color::Green
    } else {
        Color::Yellow
    };

    let box_color = Color::Cyan;
    let name = tool.name.clone();
    let name_width = name.width();
    let line_width = (chat_width.saturating_sub(4)).clamp(20, 60);
    let args = tool.arguments.clone();


    lines.push(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("┌", Style::default().fg(box_color).add_modifier(Modifier::BOLD)),
        Span::styled("─".repeat(line_width), Style::default().fg(box_color)),
        Span::styled("┐", Style::default().fg(box_color).add_modifier(Modifier::BOLD)),
    ]));


    let content_width = 8 + name_width + expand_icon.width() + 1;
    let padding = line_width.saturating_sub(content_width);

    lines.push(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("│", Style::default().fg(box_color).add_modifier(Modifier::BOLD)),
        Span::styled(" [Tool] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}", expand_icon), Style::default().fg(Color::DarkGray)),
        Span::styled(" ".repeat(padding), Style::default()),
        Span::styled("│", Style::default().fg(box_color).add_modifier(Modifier::BOLD)),
    ]));

    if tool.expanded {

        lines.push(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("├", Style::default().fg(box_color)),
            Span::styled("─".repeat(line_width), Style::default().fg(box_color)),
            Span::styled("┤", Style::default().fg(box_color)),
        ]));


        let args_display: String = args.chars().take(line_width.saturating_sub(10)).collect();
        let args_pad = line_width.saturating_sub(args_display.width() + 7);
        lines.push(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("│", Style::default().fg(box_color)),
            Span::styled(" Args: ", Style::default().fg(Color::Magenta)),
            Span::styled(args_display, Style::default().fg(Color::White)),
            Span::styled(" ".repeat(args_pad), Style::default()),
            Span::styled("│", Style::default().fg(box_color)),
        ]));


        if let Some(ref result) = tool.result {
            lines.push(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled("├", Style::default().fg(box_color)),
                Span::styled("─".repeat(line_width), Style::default().fg(box_color)),
                Span::styled("┤", Style::default().fg(box_color)),
            ]));

            let result_pad = line_width.saturating_sub(status_icon.width() + 9);
            lines.push(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled("│", Style::default().fg(box_color)),
                Span::styled(" ", Style::default()),
                Span::styled(status_icon, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(" Result:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(" ".repeat(result_pad), Style::default()),
                Span::styled("│", Style::default().fg(box_color)),
            ]));

            for result_line in result.lines().take(10) {
                let trimmed: String = result_line.chars().take(line_width.saturating_sub(4)).collect();
                let pad = line_width.saturating_sub(trimmed.width() + 4);
                lines.push(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled("│", Style::default().fg(box_color)),
                    Span::styled("   ", Style::default()),
                    Span::styled(trimmed, Style::default().fg(Color::White)),
                    Span::styled(" ".repeat(pad), Style::default()),
                    Span::styled("│", Style::default().fg(box_color)),
                ]));
            }
        }
    } else {
        if let Some(ref result) = tool.result {
            let result_preview: String = result.lines().next().unwrap_or("").chars().take(line_width.saturating_sub(status_icon.width() + 2)).collect();
            let pad = line_width.saturating_sub(result_preview.width() + status_icon.width() + 2);
            lines.push(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled("│", Style::default().fg(box_color)),
                Span::styled(" ", Style::default()),
                Span::styled(status_icon, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default()),
                Span::styled(result_preview, Style::default().fg(Color::DarkGray)),
                Span::styled(" ".repeat(pad), Style::default()),
                Span::styled("│", Style::default().fg(box_color)),
            ]));
        } else {
            let pad = line_width.saturating_sub(12);
            lines.push(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled("│", Style::default().fg(box_color)),
                Span::styled(" Running...", Style::default().fg(Color::Yellow)),
                Span::styled(" ".repeat(pad), Style::default()),
                Span::styled("│", Style::default().fg(box_color)),
            ]));
        }
    }


    lines.push(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled("└", Style::default().fg(box_color).add_modifier(Modifier::BOLD)),
        Span::styled("─".repeat(line_width), Style::default().fg(box_color)),
        Span::styled("┘", Style::default().fg(box_color).add_modifier(Modifier::BOLD)),
    ]));

    lines
}
