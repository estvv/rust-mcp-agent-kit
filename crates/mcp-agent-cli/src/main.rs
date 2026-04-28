mod app;
mod ui;

use app::App;
use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen
    )?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    event::KeyCode::Char('q') => return Ok(()),
                    event::KeyCode::Enter => {
                        app.handle_input();
                        if app.should_quit {
                            return Ok(());
                        }
                    }
                    event::KeyCode::Backspace => {
                        app.input.pop();
                    }
                    event::KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    _ => {}
                }
            }
        }
    }
}