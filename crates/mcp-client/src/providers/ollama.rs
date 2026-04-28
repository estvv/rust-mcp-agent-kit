// src/providers/ollama.rs

use crate::client::{ChatClient, ChatResponse, Message, ToolCall, ToolDefinition};

pub struct OllamaProvider {
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            model: model.to_string(),
        }
    }
}

impl ChatClient for OllamaProvider {
    fn chat(&self, messages: Vec<Message>, tools: Vec<ToolDefinition>) -> Result<ChatResponse, String> {
        let request = self.build_request(messages, tools);
        let response = self.send_request(&request)?;

        self.parse_response(&response)
    }
}

impl OllamaProvider {
    fn build_request(&self, messages: Vec<Message>, tools: Vec<ToolDefinition>) -> serde_json::Value {
        let messages: Vec<_> = messages
            .into_iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();

        let tools: Vec<_> = tools
            .into_iter()
            .map(|t| serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                }
            }))
            .collect();

        serde_json::json!({
            "model": self.model,
            "messages": messages,
            "tools": tools
        })
    }

    fn send_request(&self, request: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_string(request).unwrap())
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status_code != 200 {
            return Err(format!("Ollama returned status {}", response.status_code));
        }

        let body = response.as_str().map_err(|e| format!("Failed to read response: {}", e))?;

        serde_json::from_str(body).map_err(|e| format!("Failed to parse response: {}", e))
    }

    fn parse_response(&self, response: &serde_json::Value) -> Result<ChatResponse, String> {
        let message = &response["choices"][0]["message"];
        let content = message["content"].as_str().map(|s| s.to_string());

        let tool_calls = message["tool_calls"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| {
                        let func = &tc["function"];
                        Some(ToolCall {
                            name: func["name"].as_str()?.to_string(),
                            arguments: func["arguments"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(ChatResponse { content, tool_calls })
    }
}
