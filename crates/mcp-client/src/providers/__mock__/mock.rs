// src/providers/__mock__/mock.rs

use crate::client::{ChatClient, ChatResponse, Message, ToolCall, ToolDefinition};
use std::cell::Cell;

pub struct MockProvider {
    content: Option<String>,
    tool_calls: Vec<ToolCall>,
    call_count: Cell<usize>,
    max_tool_calls: usize,
}

impl MockProvider {
    pub fn new(response: &str) -> Self {
        Self {
            content: Some(response.to_string()),
            tool_calls: vec![],
            call_count: Cell::new(0),
            max_tool_calls: 0,
        }
    }

    pub fn with_tool_call(name: &str, arguments: &str) -> Self {
        Self {
            content: None,
            tool_calls: vec![ToolCall {
                name: name.to_string(),
                arguments: arguments.to_string(),
            }],
            call_count: Cell::new(0),
            max_tool_calls: 1,
        }
    }

    pub fn with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        let max = tool_calls.len();
        Self {
            content: None,
            tool_calls,
            call_count: Cell::new(0),
            max_tool_calls: max,
        }
    }

    pub fn with_tool_call_then_response(name: &str, arguments: &str, response: &str) -> Self {
        Self {
            content: Some(response.to_string()),
            tool_calls: vec![ToolCall {
                name: name.to_string(),
                arguments: arguments.to_string(),
            }],
            call_count: Cell::new(0),
            max_tool_calls: 1,
        }
    }
}

impl ChatClient for MockProvider {
    fn chat(&self, _messages: Vec<Message>, _tools: Vec<ToolDefinition>) -> Result<ChatResponse, String> {
        let count = self.call_count.get();
        self.call_count.set(count + 1);

        if count < self.max_tool_calls && !self.tool_calls.is_empty() {
            Ok(ChatResponse {
                content: None,
                tool_calls: self.tool_calls.clone(),
            })
        } else {
            Ok(ChatResponse {
                content: self.content.clone(),
                tool_calls: vec![],
            })
        }
    }
}