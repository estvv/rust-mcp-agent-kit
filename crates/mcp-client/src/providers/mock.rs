// src/providers/mock.rs

use crate::client::{ChatClient, ChatResponse, Message, ToolCall, ToolDefinition};

pub struct MockProvider {
    content: Option<String>,
    tool_calls: Vec<ToolCall>,
}

impl MockProvider {
    pub fn new(response: &str) -> Self {
        Self {
            content: Some(response.to_string()),
            tool_calls: vec![],
        }
    }

    pub fn with_tool_call(name: &str, arguments: &str) -> Self {
        Self {
            content: None,
            tool_calls: vec![ToolCall {
                name: name.to_string(),
                arguments: arguments.to_string(),
            }],
        }
    }

    pub fn with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: None,
            tool_calls,
        }
    }
}

impl ChatClient for MockProvider {
    fn chat(&self, _messages: Vec<Message>, _tools: Vec<ToolDefinition>) -> Result<ChatResponse, String> {
        Ok(ChatResponse {
            content: self.content.clone(),
            tool_calls: self.tool_calls.clone(),
        })
    }
}
