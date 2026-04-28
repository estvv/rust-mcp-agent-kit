use mcp_client::{Orchestrator, OllamaProvider, Profile};

pub struct App {
    pub input: String,
    pub messages: Vec<(String, String)>,
    pub profile: Option<Profile>,
    pub model: String,
    pub status: String,
    pub should_quit: bool,
    orchestrator: Option<Orchestrator<OllamaProvider>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            profile: None,
            model: "glm-5:cloud".to_string(),
            status: "Ready".to_string(),
            should_quit: false,
            orchestrator: None,
        }
    }

    pub fn handle_input(&mut self) {
        let input = self.input.trim().to_string();
        self.input.clear();

        if input.is_empty() {
            return;
        }

        if input.starts_with('/') {
            self.handle_command(&input);
        } else {
            self.messages.push(("User".to_string(), input.clone()));
            
            if let Some(ref mut orch) = self.orchestrator {
                self.status = "Thinking...".to_string();
                
                match orch.chat(&input) {
                    Ok(response) => {
                        self.messages.push(("Assistant".to_string(), response));
                    }
                    Err(e) => {
                        self.messages.push(("Error".to_string(), e));
                    }
                }
                self.status = "Ready".to_string();
            } else {
                self.messages.push(("Error".to_string(), 
                    "No profile loaded. Use /profile <name>".to_string()));
            }
        }
    }

    fn handle_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        
        match parts.get(0).map(|s| *s) {
            Some("/q") | Some("/quit") => {
                self.should_quit = true;
            }
            Some("/help") | Some("/h") => {
                self.messages.push(("System".to_string(), 
                    "Commands: /help, /profile <name>, /model <name>, /tools, /clear, /quit".to_string()));
            }
            Some("/profile") => {
                if let Some(name) = parts.get(1) {
                    match Profile::load_by_name(name) {
                        Ok(profile) => {
                            let provider = OllamaProvider::new("http://localhost:11434", &self.model);
                            let mut orch = Orchestrator::new(provider);
                            
                            for tool in profile.enabled_tools() {
                                if let Err(e) = orch.spawn_tool(tool.as_str(), tool.as_str()) {
                                    self.messages.push(("Error".to_string(), 
                                        format!("Failed to spawn {}: {}", tool, e)));
                                }
                            }
                            
                            self.status = format!("Profile: {} ({} tools)", name, orch.tools().len());
                            self.messages.push(("System".to_string(), format!("Loaded profile: {}", name)));
                            self.orchestrator = Some(orch);
                            self.profile = Some(profile);
                        }
                        Err(e) => {
                            self.messages.push(("Error".to_string(), format!("Failed to load profile: {}", e)));
                        }
                    }
                } else {
                    self.messages.push(("System".to_string(), 
                        "Usage: /profile <name>".to_string()));
                }
            }
            Some("/model") => {
                if let Some(name) = parts.get(1) {
                    self.model = name.to_string();
                    self.status = format!("Model: {}", name);
                    self.messages.push(("System".to_string(), format!("Model set to: {}", name)));
                } else {
                    self.messages.push(("System".to_string(), 
                        "Usage: /model <name>".to_string()));
                }
            }
            Some("/tools") => {
                if let Some(ref orch) = self.orchestrator {
                    let tools: Vec<_> = orch.tools().iter().map(|t| t.name.clone()).collect();
                    self.messages.push(("System".to_string(), format!("Tools: {:?}", tools)));
                } else {
                    self.messages.push(("System".to_string(), "No profile loaded. Use /profile <name>".to_string()));
                }
            }
            Some("/clear") => {
                self.messages.clear();
            }
            _ => {
                self.messages.push(("System".to_string(), 
                    format!("Unknown command: {}. Type /help", cmd)));
            }
        }
    }

    pub fn tool_count(&self) -> usize {
        self.orchestrator.as_ref().map(|o| o.tools().len()).unwrap_or(0)
    }
}