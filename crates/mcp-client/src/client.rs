// src/client.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

pub trait ChatClient {
    fn chat(&self, messages: Vec<Message>, tools: Vec<ToolDefinition>) -> Result<ChatResponse, String>;
}
