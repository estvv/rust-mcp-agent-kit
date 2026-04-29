

use mcp_client::Skill;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Build,
    Plan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Popup {
    ModelSelector,
    SkillSelector,
    CommandPalette,
    Help,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum FileStatus {
    Read,
    Edited,
    Added,
    Mentioned,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub status: FileStatus,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
    pub expanded: bool,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MessageContent {
    Text(String),
    Reasoning(String),
    Tools(Vec<ToolCall>),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub tool_count: usize,
    pub tools: Vec<String>,
}

#[allow(dead_code)]
pub struct App {
    pub input: String,
    pub input_history: Vec<String>,
    pub history_index: usize,
    pub messages: Vec<Message>,
    pub skill: Option<SkillInfo>,
    pub model: String,
    pub status: String,
    pub should_quit: bool,
    pub mode: Mode,
    pub popup: Popup,
    pub popup_filter: String,
    pub popup_selection: usize,
    #[allow(dead_code)]
    pub sidebar_scroll: usize,
    pub files: Vec<FileInfo>,
    pub files_collapsed: bool,
    pub tokens_used: u64,
    pub tokens_max: u64,
    pub scroll_offset: usize,
    pub max_scroll: usize,
    pub follow_bottom: bool,
    pub streaming: bool,
    pub streaming_message: Option<String>,
    pub streaming_reasoning: Option<String>,
    pub spinner_frame: usize,
    pub conversation_history: Vec<serde_json::Value>,
    pub available_models: Vec<String>,
    pub available_skills: Vec<SkillInfo>,
    pub version: String,
    pub conversation_name: String,
    #[allow(dead_code)]
    pub has_focus: bool,
    pub suggestion_index: usize,
    pub suggestion_scroll: usize,
    pub show_suggestions: bool,
    pub browsing_history: bool,
}

#[allow(dead_code)]
impl App {
    pub fn new() -> Self {
        Self::with_skill_and_model("coding", "glm-5:cloud")
    }

    pub fn with_skill(name: &str) -> Self {
        Self::with_skill_and_model(name, "glm-5:cloud")
    }

    pub fn with_skill_and_model(skill_name: &str, model: &str) -> Self {
        let mut app = Self {
            input: String::new(),
            input_history: Vec::new(),
            history_index: 0,
            messages: Vec::new(),
            skill: None,
            model: model.to_string(),
            status: "Ready".to_string(),
            should_quit: false,
            mode: Mode::Build,
            popup: Popup::None,
            popup_filter: String::new(),
            popup_selection: 0,
            sidebar_scroll: 0,
            files: Vec::new(),
            files_collapsed: false,
            tokens_used: 0,
            tokens_max: 128_000,
            scroll_offset: 0,
            max_scroll: 0,
            follow_bottom: true,
            streaming: false,
            streaming_message: None,
            streaming_reasoning: None,
            spinner_frame: 0,
            conversation_history: vec![],
            available_models: vec![
                "glm-5:cloud".to_string(),
                "llama3:latest".to_string(),
                "mistral:latest".to_string(),
                "gemma4:latest".to_string(),
                "phi3:3.8b".to_string(),
                "qwen3.5:latest".to_string(),
                "qwen2.5-coder:latest".to_string(),
            ],
            available_skills: Self::load_available_skills(),
            version: "0.1.0".to_string(),
            conversation_name: "Untitled".to_string(),
            has_focus: true,
            suggestion_index: 0,
            suggestion_scroll: 0,
            show_suggestions: false,
            browsing_history: false,
        };
        
        if let Err(e) = app.load_skill(skill_name) {
            app.messages.push(Message {
                sender: "Error".to_string(),
                content: MessageContent::Text(format!("Failed to load skill '{}': {}", skill_name, e)),
            });
        }
        
        app
    }

    pub fn version(&self) -> &str {
        &self.version
    }
    
    fn load_available_skills() -> Vec<SkillInfo> {
        let skill_names = ["coding", "personal", "devops", "data"];
        skill_names
            .iter()
            .filter_map(|name| {
                Skill::load_by_name(name).ok().map(|skill| {
                    let tools = skill.enabled_tools();
                    SkillInfo {
                        name: name.to_string(),
                        tool_count: tools.len(),
                        tools,
                    }
                })
            })
            .collect()
    }

    pub fn load_skill(&mut self, name: &str) -> Result<(), String> {
        let skill = Skill::load_by_name(name)?;
        let tools: Vec<String> = skill.enabled_tools();
        let tool_count = tools.len();
        
        self.skill = Some(SkillInfo {
            name: name.to_string(),
            tool_count,
            tools: tools.clone(),
        });
        self.status = "Ready".to_string();
        self.files.clear();
        
        self.messages.push(Message {
            sender: "System".to_string(),
            content: MessageContent::Text(format!("Loaded skill '{}' with {} tools", name, tool_count)),
        });
        
        Ok(())
    }

    pub fn set_skill(&mut self, name: &str) -> Result<(), String> {
        self.load_skill(name)
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Build => Mode::Plan,
            Mode::Plan => Mode::Build,
        };
        self.messages.push(Message {
            sender: "System".to_string(),
            content: MessageContent::Text(format!(
                "Switched to {} mode",
                match self.mode {
                    Mode::Build => "BUILD (will make changes)",
                    Mode::Plan => "PLAN (will only suggest)",
                }
            )),
        });
    }

    pub fn scroll_up(&mut self) {
        // When user scrolls up, disable auto-follow
        self.follow_bottom = false;
        if self.scroll_offset > 3 {
            self.scroll_offset -= 3;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(3);
        // Re-enable follow_bottom if scrolled to or past the bottom
        if self.scroll_offset >= self.max_scroll {
            self.scroll_offset = self.max_scroll;
            self.follow_bottom = true;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.follow_bottom = true;
        // Set scroll to a large value - render will clamp it to actual max
        self.scroll_offset = usize::MAX;
    }

    pub fn count_message_lines(&self) -> usize {
        self.messages.iter().map(|m| match &m.content {
            MessageContent::Text(t) => t.lines().count() + 2,
            MessageContent::Reasoning(t) => t.lines().count() + 2,
            MessageContent::Tools(tools) => {
                let mut lines = 2;
                for tool in tools {
                    if tool.expanded {
                        lines += 5 + tool.arguments.lines().count();
                        if let Some(ref result) = tool.result {
                            lines += 1 + result.lines().count();
                        }
                    } else {
                        lines += 2;
                    }
                }
                lines
            }
        }).sum()
    }

    pub fn input_history_prev(&mut self) {
        if !self.input_history.is_empty() {
            if !self.browsing_history {
                self.browsing_history = true;
                self.history_index = self.input_history.len();
            }
            if self.history_index > 0 {
                self.history_index -= 1;
                self.input = self.input_history[self.history_index].clone();
            }
        }
    }

    pub fn input_history_next(&mut self) {
        if !self.input_history.is_empty() && self.browsing_history {
            self.history_index += 1;
            if self.history_index >= self.input_history.len() {
                self.history_index = self.input_history.len();
                self.input.clear();
                self.browsing_history = false;
            } else {
                self.input = self.input_history[self.history_index].clone();
            }
        }
    }

    pub fn toggle_tool_expansion(&mut self, message_idx: usize, tool_idx: usize) {
        if let Some(Message { content: MessageContent::Tools(tools), .. }) = self.messages.get_mut(message_idx) {
            if let Some(tool) = tools.get_mut(tool_idx) {
                tool.expanded = !tool.expanded;
            }
        }
    }

    pub fn add_file(&mut self, path: String, status: FileStatus) {
        if let Some(file) = self.files.iter_mut().find(|f| f.path == path) {
            file.status = status;
        } else {
            self.files.push(FileInfo {
                path,
                status,
                additions: 0,
                deletions: 0,
            });
        }
    }

    pub fn tool_count(&self) -> usize {
        self.skill.as_ref().map(|s| s.tool_count).unwrap_or(0)
    }

    pub fn model_name(&self) -> &str {
        &self.model
    }

    pub fn skill_name(&self) -> &str {
        self.skill.as_ref().map(|s| s.name.as_str()).unwrap_or("none")
    }

    pub fn usage_percent(&self) -> f64 {
        if self.tokens_max > 0 {
            (self.tokens_used as f64 / self.tokens_max as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn toggle_files_collapsed(&mut self) {
        self.files_collapsed = !self.files_collapsed;
    }
}