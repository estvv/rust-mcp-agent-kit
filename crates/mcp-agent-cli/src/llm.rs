// src/llm.rs

use crate::event::Event;
use crate::state::{Message, MessageContent};
use crate::tools::get_tool_schemas_for_skill;
use futures_util::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

const OLLAMA_URL: &str = "http://localhost:11434";

pub fn start_llm_stream(skill: Option<&crate::state::SkillInfo>, messages: &[Message], event_sender: UnboundedSender<Event>) {
    let skill_tool_names: Vec<String> = skill.map(|s| s.tools.clone()).unwrap_or_default();
    let tools = get_tool_schemas_for_skill(&skill_tool_names);
    let messages = build_messages(messages);

    tokio::spawn(async move {
        let request_body = serde_json::json!({
            "model": "glm-5:cloud", // TODO: Make this configurable
            "messages": messages,
            "tools": tools,
            "stream": true
        });

        let client = reqwest::Client::new();
        let result = client
            .post(format!("{}/v1/chat/completions", OLLAMA_URL))
            .json(&request_body)
            .send()
            .await;

        handle_stream_response(result, event_sender).await;
    });
}

pub fn start_llm_stream_with_tool_result(
    model: &str,
    skill: Option<&crate::state::SkillInfo>,
    messages: &[Message],
    event_sender: UnboundedSender<Event>,
) {
    let skill_tool_names: Vec<String> = skill
        .map(|s| s.tools.clone())
        .unwrap_or_default();
    let tools = get_tool_schemas_for_skill(&skill_tool_names);
    let messages = build_messages(messages);
    let model = model.to_string();

    tokio::spawn(async move {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "tools": tools,
            "stream": true
        });

        let client = reqwest::Client::new();
        let result = client
            .post(format!("{}/v1/chat/completions", OLLAMA_URL))
            .json(&request_body)
            .send()
            .await;

        handle_stream_response(result, event_sender).await;
    });
}

fn build_messages(app_messages: &[Message]) -> Vec<serde_json::Value> {
    let mut messages: Vec<serde_json::Value> = vec![];

    for m in app_messages {
        match &m.content {
            MessageContent::Text(content) => {
                let role = if m.sender == "Assistant" { "assistant" } else { "user" };
                messages.push(serde_json::json!({"role": role, "content": content}));
            }
            MessageContent::Reasoning(content) => {
                messages.push(serde_json::json!({"role": "assistant", "content": content}));
            }
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
                messages.push(serde_json::json!({"role": "assistant", "tool_calls": tool_calls}));

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

    messages
}

async fn handle_stream_response(
    result: Result<reqwest::Response, reqwest::Error>,
    sender: UnboundedSender<Event>,
) {
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

                                        // Handle tool_calls
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
}
