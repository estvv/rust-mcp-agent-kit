mod title_bar;
mod status_bar;
mod sidebar;
mod chat_area;
pub mod input_bar;
mod popups;
mod tool_card;

pub use title_bar::render_title_bar;
pub use status_bar::render_status_bar;
pub use sidebar::render_sidebar;
pub use chat_area::render_chat_area;
pub use input_bar::render_input_bar;
pub use popups::{render_popup, PopupType};

use crate::state::{App, Popup};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title_bar(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Length(30),
        ])
        .split(chunks[2]);

    render_chat_area(f, app, main_chunks[0]);
    render_sidebar(f, app, main_chunks[1]);
    render_input_bar(f, app, chunks[3]);

    match app.popup {
        Popup::ModelSelector => render_popup(f, app, PopupType::ModelSelector),
        Popup::SkillSelector => render_popup(f, app, PopupType::SkillSelector),
        Popup::CommandPalette => render_popup(f, app, PopupType::CommandPalette),
        Popup::Help => render_popup(f, app, PopupType::Help),
        Popup::None => {}
    }
}