// src/main.rs

mod config;
mod event;
mod input;
mod llm;
mod state;
mod tools;
mod ui;

use config::Args;
use crossterm::event::KeyCode;
use event::Event;
use ratatui::{backend::CrosstermBackend, Terminal};
use state::{App, Message, MessageContent};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let mut app = App::with_skill_and_model(&args.skill, &args.model);
    let event_loop = event::EventLoop::new(Duration::from_millis(100));
    let event_sender = event_loop.sender();

    let res = run(&mut terminal, &mut app, event_loop, &event_sender).await;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    res
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut event_loop: event::EventLoop,
    event_sender: &UnboundedSender<Event>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Some(event) = event_loop.next().await {
            match event {
                Event::Key(key) if key.kind == crossterm::event::KeyEventKind::Press => {
                    if app.streaming {
                        match key.code {
                            KeyCode::Esc => {
                                app.streaming = false;
                                app.streaming_message = None;
                                app.streaming_reasoning = None;
                                app.status = "Cancelled".to_string();
                            }
                            KeyCode::Up => {
                                app.follow_bottom = false;
                                app.scroll_up();
                            }
                            KeyCode::Down => {
                                app.scroll_down();
                            }
                            KeyCode::PageUp => {
                                app.follow_bottom = false;
                                for _ in 0..5 {
                                    app.scroll_up();
                                }
                            }
                            KeyCode::PageDown => {
                                for _ in 0..5 {
                                    app.scroll_down();
                                }
                            }
                            _ => {}
                        }
                    } else {
                        input::handle_key_event(app, key.code, key.modifiers)?;
                        if app.should_quit {
                            return Ok(());
                        }

                        // Check if we should start LLM stream after handling input
                        if app.streaming && app.streaming_message.is_some() {
                            llm::start_llm_stream(
                                app.skill.as_ref(),
                                &app.messages,
                                event_sender.clone(),
                            );
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    input::handle_mouse_event(app, mouse);
                }
                Event::Tick => {
                    if app.streaming {
                        app.spinner_frame = (app.spinner_frame + 1) % 8;
                    }
                }
                Event::LlmStreamContent(content) => {
                    if let Some(ref mut msg) = app.streaming_message {
                        msg.push_str(&content);
                    }
                    if app.follow_bottom {
                        app.scroll_to_bottom();
                    }
                }
                Event::LlmStreamReasoning(reasoning) => {
                    if let Some(ref mut r) = app.streaming_reasoning {
                        r.push_str(&reasoning);
                    }
                    if app.follow_bottom {
                        app.scroll_to_bottom();
                    }
                }
                Event::LlmStreamToolCalls { calls } => {
                    // Finalize any pending content/reasoning first
                    if let Some(reasoning) = app.streaming_reasoning.take() {
                        if !reasoning.is_empty() {
                            app.messages.push(Message {
                                sender: "Assistant".to_string(),
                                content: MessageContent::Reasoning(reasoning),
                            });
                        }
                    }
                    if let Some(content) = app.streaming_message.take() {
                        if !content.is_empty() {
                            app.messages.push(Message {
                                sender: "Assistant".to_string(),
                                content: MessageContent::Text(content),
                            });
                        }
                    }

                    app.streaming = false;
                    app.status = "Executing tools...".to_string();

                    // Process each tool call
                    for (name, args) in calls {
                        app.messages.push(Message {
                            sender: "Tool".to_string(),
                            content: MessageContent::Tools(vec![state::ToolCall {
                                name: name.clone(),
                                arguments: args.clone(),
                                result: None,
                                expanded: false,
                                is_error: false,
                            }]),
                        });
                        app.scroll_to_bottom();

                        // Execute tool
                        let result = tools::execute_tool(&name, &args);

                        // Update tool result
                        if let Some(Message {
                            content: MessageContent::Tools(tools),
                            ..
                        }) = app.messages.last_mut()
                        {
                            if let Some(tool) = tools.last_mut() {
                                tool.result = Some(result.clone());
                                tool.is_error = result.starts_with("Error:");
                            }
                        }

                        // Continue with tool result
                        llm::start_llm_stream_with_tool_result(
                            &app.model,
                            app.skill.as_ref(),
                            &app.messages,
                            event_sender.clone(),
                        );
                    }
                }
                Event::LlmStreamDone => {
                    // Finalize reasoning if present
                    if let Some(reasoning) = app.streaming_reasoning.take() {
                        if !reasoning.is_empty() {
                            app.messages.push(Message {
                                sender: "Assistant".to_string(),
                                content: MessageContent::Reasoning(reasoning),
                            });
                        }
                    }
                    // Finalize content if present
                    if let Some(msg) = app.streaming_message.take() {
                        if !msg.is_empty() {
                            app.messages.push(Message {
                                sender: "Assistant".to_string(),
                                content: MessageContent::Text(msg),
                            });
                        }
                    }
                    app.streaming = false;
                    app.streaming_reasoning = None;
                    app.streaming_message = None;
                    app.status = "Ready".to_string();
                    app.scroll_to_bottom();
                }
                Event::LlmError(e) => {
                    app.messages.push(Message {
                        sender: "Error".to_string(),
                        content: MessageContent::Text(format!("LLM error: {}", e)),
                    });
                    app.streaming = false;
                    app.streaming_message = None;
                    app.streaming_reasoning = None;
                    app.status = "Error".to_string();
                }
                _ => {}
            }
        }
    }
}
