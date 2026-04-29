mod state;
mod ui;

use clap::Parser;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use futures_util::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use state::{App, Message, MessageContent, Popup};
use std::collections::HashMap;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Parser, Debug)]
#[command(name = "mcp-agent-cli")]
#[command(about = "Interactive TUI for AI chat with MCP tools")]
#[command(version)]
struct Args {
    #[arg(short, long, default_value = "coding")]
    skill: String,
    
    #[arg(short, long, default_value = "glm-5:cloud")]
    model: String,
}

enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(MouseEvent),
    Tick,
    LlmStreamContent(String),
    LlmStreamReasoning(String),
    LlmStreamToolCalls { calls: Vec<(String, String)> },
    LlmStreamDone,
    LlmError(String),
}

struct EventLoop {
    receiver: UnboundedReceiver<Event>,
    sender: UnboundedSender<Event>,
}

impl EventLoop {
    fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let event_sender = sender.clone();
        let tick_sender = sender.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                let _ = tick_sender.send(Event::Tick);
            }
        });
        
        tokio::spawn(async move {
            loop {
                if let Ok(true) = event::poll(Duration::from_millis(50)) {
                    if let Ok(crossterm_event) = event::read() {
                        match crossterm_event {
                            CrosstermEvent::Key(key) => {
                                let _ = event_sender.send(Event::Key(key));
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                let _ = event_sender.send(Event::Mouse(mouse));
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
        
        Self { receiver, sender }
    }
    
    fn sender(&self) -> UnboundedSender<Event> {
        self.sender.clone()
    }
    
    async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}

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
    let event_loop = EventLoop::new(Duration::from_millis(100));
    let event_sender = event_loop.sender();
    
    let http_client = reqwest::Client::new();
    let ollama_url = "http://localhost:11434".to_string();
    
    let res = run(&mut terminal, &mut app, event_loop, &event_sender, &http_client, &ollama_url).await;

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
    mut event_loop: EventLoop,
    event_sender: &UnboundedSender<Event>,
    http_client: &reqwest::Client,
    ollama_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Some(event) = event_loop.next().await {
            match event {
                Event::Key(key) if key.kind == event::KeyEventKind::Press => {
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
                                for _ in 0..5 { app.scroll_up(); }
                            }
                            KeyCode::PageDown => {
                                for _ in 0..5 { app.scroll_down(); }
                            }
                            _ => {}
                        }
                    } else {
                        handle_key_event(app, key.code, key.modifiers, event_sender, http_client, ollama_url)?;
                        if app.should_quit {
                            return Ok(());
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(app, mouse);
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
                    // Only auto-scroll if user hasn't scrolled up
                    if app.follow_bottom {
                        app.scroll_to_bottom();
                    }
                }
                Event::LlmStreamReasoning(reasoning) => {
                    if let Some(ref mut r) = app.streaming_reasoning {
                        r.push_str(&reasoning);
                    }
                    // Only auto-scroll if user hasn't scrolled up
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
                        let result = execute_tool(&name, &args);
                        
                        // Update tool result
                        if let Some(Message { content: MessageContent::Tools(tools), .. }) = app.messages.last_mut() {
                            if let Some(tool) = tools.last_mut() {
                                tool.result = Some(result.clone());
                                tool.is_error = result.starts_with("Error:");
                            }
                        }
                        
                        // Continue with tool result
                        start_llm_stream_with_tool_result(app, result, event_sender, http_client, ollama_url);
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

fn handle_key_event(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    event_sender: &UnboundedSender<Event>,
    http_client: &reqwest::Client,
    ollama_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if app.popup != Popup::None {
        return handle_popup_input(app, code, modifiers);
    }
    
    let suggestions = ui::input_bar::get_suggestions(&app.input);
    let has_suggestions = !suggestions.is_empty();
    
    if has_suggestions {
        match code {
            KeyCode::Down => {
                if app.suggestion_index < suggestions.len().saturating_sub(1) {
                    app.suggestion_index += 1;
                    let max_visible = ui::input_bar::max_visible_suggestions();
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
                if let Some(suggestion) = suggestions.get(app.suggestion_index) {
                    app.input = suggestion.to_string();
                    app.suggestion_index = 0;
                    app.suggestion_scroll = 0;
                    if code == KeyCode::Tab {
                        return Ok(());
                    }
                }
                if code == KeyCode::Tab {
                    return Ok(());
                }
                let input = app.input.clone();
                app.input.clear();
                app.suggestion_index = 0;
                app.suggestion_scroll = 0;
                app.browsing_history = false;
                handle_command(app, input.trim(), event_sender, http_client, ollama_url);
                app.input_history.push(input);
                app.history_index = app.input_history.len();
                return Ok(());
            }
            KeyCode::Esc => {
                app.input.clear();
                app.suggestion_index = 0;
                app.suggestion_scroll = 0;
                app.browsing_history = false;
                return Ok(());
            }
            _ => {}
        }
    }
    
    handle_normal_input(app, code, modifiers, event_sender, http_client, ollama_url)
}

fn handle_normal_input(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    event_sender: &UnboundedSender<Event>,
    http_client: &reqwest::Client,
    ollama_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match code {
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.toggle_mode();
        }
        KeyCode::Up if app.input.is_empty() || app.browsing_history => {
            app.input_history_prev();
        }
        KeyCode::Down if app.input.is_empty() || app.browsing_history => {
            app.input_history_next();
        }
        KeyCode::Enter if !app.input.is_empty() => {
            let input = app.input.clone();
            app.input.clear();
            app.suggestion_index = 0;
            app.suggestion_scroll = 0;
            app.browsing_history = false;
            handle_command(app, input.trim(), event_sender, http_client, ollama_url);
            app.input_history.push(input);
            app.history_index = app.input_history.len();
        }
        KeyCode::Enter => {}
        KeyCode::PageUp => {
            app.scroll_up();
        }
        KeyCode::PageDown => {
            app.scroll_down();
        }
        KeyCode::Home => {
            app.scroll_offset = 0;
            app.follow_bottom = false;
        }
        KeyCode::End => {
            app.scroll_to_bottom();
        }
        KeyCode::Backspace => {
            app.input.pop();
            app.suggestion_index = 0;
            app.suggestion_scroll = 0;
            app.browsing_history = false;
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char('m') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.popup = Popup::ModelSelector;
            app.popup_filter.clear();
            app.popup_selection = 0;
        }
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.popup = Popup::SkillSelector;
            app.popup_filter.clear();
            app.popup_selection = 0;
        }
        KeyCode::Char('k') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.popup = Popup::CommandPalette;
            app.popup_filter.clear();
            app.popup_selection = 0;
        }
        KeyCode::Char(c) => {
            app.input.push(c);
            app.suggestion_index = 0;
            app.suggestion_scroll = 0;
            app.browsing_history = false;
        }
        _ => {}
    }
    Ok(())
}

fn handle_popup_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<(), Box<dyn std::error::Error>> {
    match code {
        KeyCode::Esc => {
            app.popup = Popup::None;
            app.popup_filter.clear();
        }
        KeyCode::Up => {
            if app.popup_selection > 0 {
                app.popup_selection -= 1;
            }
        }
        KeyCode::Down => {
            app.popup_selection += 1;
        }
        KeyCode::Enter => {
            apply_popup_selection(app);
            app.popup = Popup::None;
            app.popup_filter.clear();
        }
        KeyCode::Backspace => {
            app.popup_filter.pop();
        }
        KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' => {
            app.popup = Popup::None;
            app.popup_filter.clear();
        }
        KeyCode::Char(c) => {
            app.popup_filter.push(c);
            app.popup_selection = 0;
        }
        _ => {}
    }
    Ok(())
}

fn handle_command(
    app: &mut App,
    cmd: &str,
    event_sender: &UnboundedSender<Event>,
    http_client: &reqwest::Client,
    ollama_url: &str,
) {
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
        
        // Start streaming LLM request
        start_llm_stream(app, cmd.to_string(), event_sender, http_client, ollama_url);
    }
}

fn start_llm_stream(
    app: &mut App,
    _prompt: String,
    event_sender: &UnboundedSender<Event>,
    http_client: &reqwest::Client,
    ollama_url: &str,
) {
    app.streaming = true;
    app.streaming_message = Some(String::new());
    app.streaming_reasoning = Some(String::new());
    app.status = "Thinking...".to_string();
    app.spinner_frame = 0;
    
    let model = app.model.clone();
    let skill_tool_names: Vec<String> = app.skill
        .as_ref()
        .map(|s| s.tools.clone())
        .unwrap_or_default();
    let tools = get_tool_schemas_for_skill(&skill_tool_names);
    
    let messages = app.messages.iter().map(|m| {
        match &m.content {
            MessageContent::Text(content) => serde_json::json!({"role": "user", "content": content}),
            MessageContent::Reasoning(content) => serde_json::json!({"role": "assistant", "content": content}),
            MessageContent::Tools(tools) => {
                let tool_calls: Vec<_> = tools.iter().map(|t| {
                    serde_json::json!({
                        "id": format!("call_{}", t.name),
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "arguments": t.arguments
                        }
                    })
                }).collect();
                serde_json::json!({"role": "assistant", "tool_calls": tool_calls})
            }
        }
    }).collect::<Vec<_>>();
    
    let sender = event_sender.clone();
    let client = http_client.clone();
    let url = ollama_url.to_string();
    let _messages_for_next = messages.clone();
    
    tokio::spawn(async move {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "tools": tools,
            "stream": true
        });
        
        let result = client
            .post(format!("{}/v1/chat/completions", url))
            .json(&request_body)
            .send()
            .await;
        
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();
                    let mut tool_calls: Vec<(String, String)> = Vec::new();
                    
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));
                                
                                while let Some(pos) = buffer.find('\n') {
                                    let line = buffer[..pos].trim().to_string();
                                    buffer = buffer[pos + 1..].to_string();
                                    
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        if data == "[DONE]" {
                                            // Execute tool calls if present
                                            if !tool_calls.is_empty() {
                                                let _ = sender.send(Event::LlmStreamToolCalls { calls: tool_calls });
                                            } else {
                                                let _ = sender.send(Event::LlmStreamDone);
                                            }
                                            return;
                                        }
                                        
                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                            // Handle content
                                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                                if !content.is_empty() {
                                                    let _ = sender.send(Event::LlmStreamContent(content.to_string()));
                                                }
                                            }
                                            
                                            // Handle reasoning
                                            if let Some(reasoning) = json["choices"][0]["delta"]["reasoning"].as_str() {
                                                if !reasoning.is_empty() {
                                                    let _ = sender.send(Event::LlmStreamReasoning(reasoning.to_string()));
                                                }
                                            }
                                            
                                            // Handle tool calls (streaming)
                                            if let Some(tool_call_chunks) = json["choices"][0]["delta"]["tool_calls"].as_array() {
                                                for chunk in tool_call_chunks {
                                                    if let Some(name) = chunk["function"]["name"].as_str() {
                                                        tool_calls.push((name.to_string(), String::new()));
                                                    }
                                                    if let Some(args) = chunk["function"]["arguments"].as_str() {
                                                        if let Some(last) = tool_calls.last_mut() {
                                                            last.1.push_str(args);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = sender.send(Event::LlmError(e.to_string()));
                                return;
                            }
                        }
                    }
                    
                    // Stream ended without [DONE]
                    if !tool_calls.is_empty() {
                        let _ = sender.send(Event::LlmStreamToolCalls { calls: tool_calls });
                    } else {
                        let _ = sender.send(Event::LlmStreamDone);
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let _ = sender.send(Event::LlmError(format!("HTTP {}: {}", status, body)));
                }
            }
            Err(e) => {
                let _ = sender.send(Event::LlmError(e.to_string()));
            }
        }
    });
}

fn get_tool_schemas_for_skill(skill_tool_names: &[String]) -> Vec<serde_json::Value> {
    // Map skill tool names to actual function names
    let tool_mappings: HashMap<&str, Vec<&str>> = [
        ("tool-filesystem", vec!["read_file", "write_file", "list_directory", "search_files"]),
        ("tool-web", vec!["fetch_webpage"]),
        ("tool-utilities", vec!["run_command"]),
        ("tool-weather", vec!["get_weather"]),
    ].iter().cloned().collect();
    
    let all_schemas = get_tool_schemas();
    
    // Collect all function names enabled by this skill
    let enabled_functions: Vec<&str> = skill_tool_names
        .iter()
        .flat_map(|name| {
            tool_mappings.get(name.as_str()).cloned().unwrap_or_default()
        })
        .collect();
    
    all_schemas
        .into_iter()
        .filter(|schema| {
            let name = schema["function"]["name"].as_str().unwrap_or("");
            enabled_functions.contains(&name)
        })
        .collect()
}

fn get_tool_schemas() -> Vec<serde_json::Value> {
    vec![
        // tool-filesystem schemas
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read the contents of a file from the filesystem",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to read"
                        }
                    },
                    "required": ["path"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write content to a file on the filesystem",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "list_directory",
                "description": "List files and directories in a given path",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The directory path to list"
                        }
                    },
                    "required": ["path"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "search_files",
                "description": "Search for files matching a pattern in a directory",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The directory to search in"
                        },
                        "pattern": {
                            "type": "string",
                            "description": "The glob pattern to match files"
                        }
                    },
                    "required": ["path", "pattern"]
                }
            }
        }),
        // tool-web schemas
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fetch_webpage",
                "description": "Fetch content from a URL",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch"
                        }
                    },
                    "required": ["url"]
                }
            }
        }),
        // tool-utilities schemas
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Run a shell command",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to run"
                        }
                    },
                    "required": ["command"]
                }
            }
        }),
        // tool-weather schemas
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather information for a city",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "The city name"
                        }
                    },
                    "required": ["city"]
                }
            }
        }),
    ]
}

fn execute_tool(name: &str, arguments: &str) -> String {
    // Real tool execution for filesystem operations
    let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or(serde_json::json!({}));
    
    match name {
        "read_file" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
            match std::fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => format!("Error reading file: {}", e),
            }
        }
        "write_file" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
            let content = args.get("content").and_then(|c| c.as_str()).unwrap_or("");
            match std::fs::write(path, content) {
                Ok(()) => format!("File written successfully: {}", path),
                Err(e) => format!("Error writing file: {}", e),
            }
        }
        "list_directory" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            match std::fs::read_dir(path) {
                Ok(entries) => {
                    let items: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    items.join("\n")
                }
                Err(e) => format!("Error listing directory: {}", e),
            }
        }
        "search_files" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            let pattern = args.get("pattern").and_then(|p| p.as_str()).unwrap_or("*");
            let pattern_glob = glob::Pattern::new(pattern);
            match pattern_glob {
                Ok(pat) => {
                    let results: Vec<_> = std::fs::read_dir(path)
                        .ok()
                        .into_iter()
                        .flat_map(|entries| entries.filter_map(|e| e.ok()))
                        .filter(|e| pat.matches(&e.file_name().to_string_lossy()))
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    results.join("\n")
                }
                Err(e) => format!("Invalid pattern: {}", e),
            }
        }
        "fetch_webpage" => {
            let url = args.get("url").and_then(|u| u.as_str()).unwrap_or("");
            format!("Fetch webpage not yet implemented for URL: {}", url)
        }
        "run_command" => {
            let command = args.get("command").and_then(|c| c.as_str()).unwrap_or("");
            format!("Run command not yet implemented for: {}", command)
        }
        "get_weather" => {
            let city = args.get("city").and_then(|c| c.as_str()).unwrap_or("unknown");
            format!("Weather for {}: 18°C, Partly cloudy (mock)", city)
        }
        _ => format!("Unknown tool: {}", name),
    }
}

fn start_llm_stream_with_tool_result(
    app: &mut App,
    _tool_result: String,
    event_sender: &UnboundedSender<Event>,
    http_client: &reqwest::Client,
    ollama_url: &str,
) {
    app.streaming = true;
    app.streaming_message = Some(String::new());
    app.streaming_reasoning = Some(String::new());
    app.status = "Processing tool result...".to_string();
    
    // Build messages with proper roles
    let mut messages: Vec<serde_json::Value> = vec![];
    
    for m in app.messages.iter() {
        match &m.content {
            MessageContent::Text(content) => {
                // User messages
                messages.push(serde_json::json!({"role": "user", "content": content}));
            }
            MessageContent::Reasoning(content) => {
                // Assistant reasoning - include as assistant content
                messages.push(serde_json::json!({"role": "assistant", "content": content}));
            }
            MessageContent::Tools(tools) => {
                // First, find if previous message was user asking for tool call
                // Convert to assistant message with tool_calls
                let tool_calls: Vec<_> = tools.iter().map(|t| {
                    serde_json::json!({
                        "id": format!("call_{}", t.name),
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "arguments": t.arguments
                        }
                    })
                }).collect();
                messages.push(serde_json::json!({"role": "assistant", "tool_calls": tool_calls}));
                
                // Then add tool response for each tool
                for tool in tools {
                    messages.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": format!("call_{}", tool.name),
                        "content": tool.result.as_ref().unwrap_or(&String::new())
                    }));
                }
            }
        }
    }
    
    let model = app.model.clone();
    let skill_tool_names: Vec<String> = app.skill
        .as_ref()
        .map(|s| s.tools.clone())
        .unwrap_or_default();
    let tools = get_tool_schemas_for_skill(&skill_tool_names);
    
    let sender = event_sender.clone();
    let client = http_client.clone();
    let url = ollama_url.to_string();
    
    tokio::spawn(async move {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "tools": tools,
            "stream": true
        });
        
        let result = client
            .post(format!("{}/v1/chat/completions", url))
            .json(&request_body)
            .send()
            .await;
        
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();
                    let mut tool_calls: Vec<(String, String)> = Vec::new();
                    
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));
                                
                                while let Some(pos) = buffer.find('\n') {
                                    let line = buffer[..pos].trim().to_string();
                                    buffer = buffer[pos + 1..].to_string();
                                    
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        if data == "[DONE]" {
                                            if !tool_calls.is_empty() {
                                                let _ = sender.send(Event::LlmStreamToolCalls { calls: tool_calls });
                                            } else {
                                                let _ = sender.send(Event::LlmStreamDone);
                                            }
                                            return;
                                        }
                                        
                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                                if !content.is_empty() {
                                                    let _ = sender.send(Event::LlmStreamContent(content.to_string()));
                                                }
                                            }
                                            
                                            if let Some(reasoning) = json["choices"][0]["delta"]["reasoning"].as_str() {
                                                if !reasoning.is_empty() {
                                                    let _ = sender.send(Event::LlmStreamReasoning(reasoning.to_string()));
                                                }
                                            }
                                            
                                            if let Some(tool_call_chunks) = json["choices"][0]["delta"]["tool_calls"].as_array() {
                                                for chunk in tool_call_chunks {
                                                    if let Some(name) = chunk["function"]["name"].as_str() {
                                                        tool_calls.push((name.to_string(), String::new()));
                                                    }
                                                    if let Some(args) = chunk["function"]["arguments"].as_str() {
                                                        if let Some(last) = tool_calls.last_mut() {
                                                            last.1.push_str(args);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = sender.send(Event::LlmError(e.to_string()));
                                return;
                            }
                        }
                    }
                    
                    if !tool_calls.is_empty() {
                        let _ = sender.send(Event::LlmStreamToolCalls { calls: tool_calls });
                    } else {
                        let _ = sender.send(Event::LlmStreamDone);
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let _ = sender.send(Event::LlmError(format!("HTTP {}: {}", status, body)));
                }
            }
            Err(e) => {
                let _ = sender.send(Event::LlmError(e.to_string()));
            }
        }
    });
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
                app.popup = Popup::None;
                app.popup_filter.clear();
                
                app.input = cmd.to_string();
            }
        }
        _ => {}
    }
}

fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            app.scroll_up();
        }
        MouseEventKind::ScrollDown => {
            app.scroll_down();
        }
        _ => {}
    }
}