// src/input.rs

use crate::state::{App, Message, MessageContent, Popup};
use crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};

pub fn handle_key_event(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<(), Box<dyn std::error::Error>> {
    if app.popup != Popup::None {
        return handle_popup_input(app, code, modifiers);
    }

    let suggestions = crate::ui::input_bar::get_suggestions(&app.input);
    let has_suggestions = !suggestions.is_empty();

    if has_suggestions {
        match code {
            KeyCode::Down => {
                if app.suggestion_index < suggestions.len().saturating_sub(1) {
                    app.suggestion_index += 1;
                    let max_visible = crate::ui::input_bar::max_visible_suggestions();
                    if app.suggestion_index >= app.suggestion_scroll + max_visible {
                        app.suggestion_scroll = app.suggestion_index - max_visible + 1;
                    }
                }
                return Ok(());
            }
            KeyCode::Up => {
                if app.suggestion_index > 0 {
                    app.suggestion_index -= 1;
                    if app.suggestion_index < app.suggestion_scroll {
                        app.suggestion_scroll = app.suggestion_index;
                    }
                }
                return Ok(());
            }
            KeyCode::Tab | KeyCode::Enter => {
                if let Some(&suggestion) = suggestions.get(app.suggestion_index) {
                    app.input = suggestion.to_string();
                    app.show_suggestions = false;
                }
                if code == KeyCode::Enter {
                    return handle_enter(app);
                }
                return Ok(());
            }
            KeyCode::Esc => {
                app.show_suggestions = false;
                return Ok(());
            }
            _ => {}
        }
    }

    match code {
        KeyCode::Enter => handle_enter(app),
        KeyCode::Up => {
            if app.input.is_empty() {
                app.input_history_prev();
            } else {
                app.scroll_up();
            }
            Ok(())
        }
        KeyCode::Down => {
            if app.input.is_empty() {
                app.input_history_next();
            } else {
                app.scroll_down();
            }
            Ok(())
        }
        KeyCode::PageUp => {
            for _ in 0..5 { app.scroll_up(); }
            Ok(())
        }
        KeyCode::PageDown => {
            for _ in 0..5 { app.scroll_down(); }
            Ok(())
        }
        KeyCode::Esc => {
            app.should_quit = true;
            Ok(())
        }
        KeyCode::Char(c) => {
            if modifiers == KeyModifiers::CONTROL {
                match c {
                    'm' => { app.popup = Popup::ModelSelector; app.popup_filter.clear(); app.popup_selection = 0; }
                    's' => { app.popup = Popup::SkillSelector; app.popup_filter.clear(); app.popup_selection = 0; }
                    'k' => { app.popup = Popup::CommandPalette; app.popup_filter.clear(); app.popup_selection = 0; }
                    _ => {}
                }
            } else {
                app.input.push(c);
                app.show_suggestions = crate::ui::input_bar::get_suggestions(&app.input).len() > 0;
                app.suggestion_index = 0;
                app.suggestion_scroll = 0;
            }
            Ok(())
        }
        KeyCode::Backspace => {
            app.input.pop();
            app.show_suggestions = crate::ui::input_bar::get_suggestions(&app.input).len() > 0;
            app.suggestion_index = 0;
            Ok(())
        }
        KeyCode::Tab => {
            app.toggle_mode();
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_enter(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if app.input.trim().is_empty() {
        return Ok(());
    }

    let cmd = app.input.trim().to_string();
    app.input_history.push(cmd.clone());
    app.input.clear();
    app.show_suggestions = false;

    handle_command(app, &cmd);

    Ok(())
}

fn handle_command(app: &mut App, cmd: &str) {
    let cmd = cmd.trim();

    if cmd.starts_with('/') {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];

        match command {
            "/help" => {
                app.popup = Popup::Help;
                app.popup_filter.clear();
                app.popup_selection = 0;
            }
            "/model" => {
                app.popup = Popup::ModelSelector;
                app.popup_filter.clear();
                app.popup_selection = 0;
            }
            "/skill" => {
                app.popup = Popup::SkillSelector;
                app.popup_filter.clear();
                app.popup_selection = 0;
            }
            "/clear" => {
                app.messages.clear();
                app.scroll_offset = 0;
                app.max_scroll = 0;
            }
            "/quit" => {
                app.should_quit = true;
            }
            "/mode" => {
                app.toggle_mode();
            }
            _ => {
                app.messages.push(Message {
                    sender: "System".to_string(),
                    content: MessageContent::Text(format!("Unknown command: {}", command)),
                });
            }
        }
    } else if !cmd.is_empty() {
        app.messages.push(Message {
            sender: "User".to_string(),
            content: MessageContent::Text(cmd.to_string()),
        });
        app.scroll_to_bottom();

        // Signal to start LLM stream
        app.streaming = true;
        app.streaming_message = Some(String::new());
        app.streaming_reasoning = Some(String::new());
        app.status = "Thinking...".to_string();
        app.spinner_frame = 0;
    }
}

pub fn handle_popup_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<(), Box<dyn std::error::Error>> {
    match code {
        KeyCode::Esc => {
            app.popup = Popup::None;
            Ok(())
        }
        KeyCode::Up => {
            if app.popup_selection > 0 {
                app.popup_selection -= 1;
            }
            Ok(())
        }
        KeyCode::Down => {
            let max_items = match app.popup {
                Popup::ModelSelector => app.available_models.len(),
                Popup::SkillSelector => app.available_skills.len(),
                Popup::CommandPalette => 7,
                Popup::Help => 0,
                Popup::None => 0,
            };
            if app.popup_selection < max_items.saturating_sub(1) {
                app.popup_selection += 1;
            }
            Ok(())
        }
        KeyCode::Enter => {
            apply_popup_selection(app);
            Ok(())
        }
        KeyCode::Char(c) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
            app.popup_filter.push(c);
            app.popup_selection = 0;
            Ok(())
        }
        KeyCode::Backspace => {
            app.popup_filter.pop();
            Ok(())
        }
        _ => Ok(()),
    }
}

fn apply_popup_selection(app: &mut App) {
    match app.popup {
        Popup::ModelSelector => {
            let filtered: Vec<_> = app.available_models
                .iter()
                .filter(|m| app.popup_filter.is_empty() || m.to_lowercase().contains(&app.popup_filter.to_lowercase()))
                .collect();

            if let Some(model) = filtered.get(app.popup_selection.min(filtered.len().saturating_sub(1))) {
                app.model = model.to_string();
            }
        }
        Popup::SkillSelector => {
            let filtered: Vec<_> = app.available_skills
                .iter()
                .filter(|s| app.popup_filter.is_empty() || s.name.to_lowercase().contains(&app.popup_filter.to_lowercase()))
                .collect();

            if let Some(skill) = filtered.get(app.popup_selection.min(filtered.len().saturating_sub(1))) {
                let skill_name = skill.name.clone();
                if let Err(e) = app.load_skill(&skill_name) {
                    app.messages.push(Message {
                        sender: "Error".to_string(),
                        content: MessageContent::Text(format!("Failed to load skill: {}", e)),
                    });
                }
            }
        }
        Popup::CommandPalette => {
            let commands = [
                ("/help", "Show available commands"),
                ("/skill", "Load tool skill"),
                ("/model", "Switch LLM model"),
                ("/tools", "List loaded tools"),
                ("/mode", "Toggle Plan/Build mode"),
                ("/clear", "Clear chat history"),
                ("/quit", "Exit CLI"),
            ];

            let filtered: Vec<_> = commands
                .iter()
                .filter(|(cmd, _)| app.popup_filter.is_empty() || cmd.starts_with(&app.popup_filter) || cmd.contains(&app.popup_filter))
                .collect();

            if let Some((cmd, _)) = filtered.get(app.popup_selection.min(filtered.len().saturating_sub(1))) {
                app.input = cmd.to_string();
                app.show_suggestions = true;
            }
        }
        _ => {}
    }

    app.popup = Popup::None;
    app.popup_filter.clear();
}

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            app.scroll_down();
        }
        MouseEventKind::ScrollUp => {
            app.scroll_up();
        }
        _ => {}
    }
}
