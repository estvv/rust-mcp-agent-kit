use crate::state::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupType {
    ModelSelector,
    SkillSelector,
    CommandPalette,
    Help,
}

pub fn render_popup(f: &mut Frame, app: &App, popup_type: PopupType) {
    match popup_type {
        PopupType::ModelSelector => render_model_selector(f, app),
        PopupType::SkillSelector => render_skill_selector(f, app),
        PopupType::CommandPalette => render_command_palette(f, app),
        PopupType::Help => render_help(f, app),
    }
}

fn render_model_selector(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(40, 60, f.area());
    
    let filtered_models: Vec<_> = app.available_models
        .iter()
        .filter(|m| app.popup_filter.is_empty() || m.to_lowercase().contains(&app.popup_filter.to_lowercase()))
        .collect();

    let mut lines: Vec<Line<'static>> = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled(" Select Model", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    for (idx, model) in filtered_models.iter().enumerate() {
        let is_current = *model == &app.model;
        let is_selected = idx == app.popup_selection.min(filtered_models.len().saturating_sub(1));
        
        let prefix = if is_current { "* " } else { "  " };
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        lines.push(Line::from(vec![
            Span::styled(format!("{}{}", prefix, model), style),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Reset)); // Solid background

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area); // Clear the area first
    f.render_widget(paragraph, popup_area);
}

fn render_skill_selector(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(50, 70, f.area());
    
    let filtered_skills: Vec<_> = app.available_skills
        .iter()
        .filter(|s| app.popup_filter.is_empty() || s.name.to_lowercase().contains(&app.popup_filter.to_lowercase()))
        .collect();

    let selected_idx = app.popup_selection.min(filtered_skills.len().saturating_sub(1));
    let selected_skill = filtered_skills.get(selected_idx);

    let mut lines: Vec<Line<'static>> = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled(" Select Skill", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    
    for (idx, skill) in filtered_skills.iter().enumerate() {
        let is_current = app.skill.as_ref().map(|s| s.name.as_str()) == Some(skill.name.as_str());
        let is_selected = idx == selected_idx;
        
        let prefix = if is_current { "* " } else { "  " };
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        
        lines.push(Line::from(vec![
            Span::styled(format!("{}{} ({} tools)", prefix, skill.name, skill.tool_count), style),
        ]));
    }
    
    // Show tools for selected skill
    if let Some(skill) = selected_skill {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(format!(" Tools in {}:", skill.name), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        for tool in &skill.tools {
            lines.push(Line::from(vec![
                Span::styled(format!("  - {}", tool), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Reset)); // Solid background

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn render_command_palette(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(50, 70, f.area());
    
    let commands = [
        ("/help", "Show available commands"),
        ("/skill", "Load tool skill"),
        ("/model", "Switch LLM model"),
        ("/tools", "List loaded tools"),
        ("/mode", "Toggle Plan/Build mode"),
        ("/clear", "Clear chat history"),
        ("/quit", "Exit CLI"),
    ];

    let filtered_commands: Vec<_> = commands
        .iter()
        .filter(|(cmd, _)| app.popup_filter.is_empty() || cmd.starts_with(&app.popup_filter) || cmd.contains(&app.popup_filter))
        .collect();

    let mut lines: Vec<Line<'static>> = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled(" Command Palette", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));
    
    for (idx, (cmd, desc)) in filtered_commands.iter().enumerate() {
        let is_selected = idx == app.popup_selection.min(filtered_commands.len().saturating_sub(1));
        
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        lines.push(Line::from(vec![
            Span::styled(format!("{:<12} {}", cmd, desc), style),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Reset)); // Solid background

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn render_help(f: &mut Frame, _app: &App) {
    let popup_area = centered_rect(60, 80, f.area());
    
    let lines: Vec<Line<'static>> = vec![
        Line::from(vec![
            Span::styled(" Help", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Commands:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  /help          Show this help"),
        Line::from("  /skill <n>     Load skill"),
        Line::from("  /model <n>     Switch model"),
        Line::from("  /tools         List loaded tools"),
        Line::from("  /mode          Toggle Plan/Build mode"),
        Line::from("  /clear         Clear chat"),
        Line::from("  /quit          Exit CLI"),
        Line::from(""),
        Line::from(vec![Span::styled("Keyboard:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Up/Down        Input history (when input empty)"),
        Line::from("  PageUp/PageDown  Scroll chat"),
        Line::from("  Ctrl+M         Open model selector"),
        Line::from("  Ctrl+S         Open skill selector"),
        Line::from("  Ctrl+K         Open command palette"),
        Line::from("  Tab            Toggle Build/Plan mode"),
        Line::from("  Enter          Send message"),
        Line::from("  Escape         Close popup"),
        Line::from(""),
        Line::from(vec![Span::styled("Mouse:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Scroll         Scroll chat"),
        Line::from("  Click tool card  Expand/collapse"),
        Line::from("  Select text    Auto-copy to clipboard"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Reset)); // Solid background

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}