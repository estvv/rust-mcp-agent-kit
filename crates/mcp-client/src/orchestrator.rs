use crate::{ChatClient, Message, ServerProcess, ToolDefinition};
use std::collections::HashMap;

pub struct Orchestrator<C: ChatClient> {
    client: C,
    servers: HashMap<String, ServerProcess>,
    tools: Vec<ToolDefinition>,
}

impl<C: ChatClient> Orchestrator<C> {
    pub fn new(client: C) -> Self {
        Orchestrator {
            client,
            servers: HashMap::new(),
            tools: Vec::new(),
        }
    }

    pub fn spawn_tool(&mut self, name: &str, binary: &str) -> Result<(), String> {
        let mut server = ServerProcess::spawn(name, binary)?;
        server.initialize()?;
        
        let tools_response = server.list_tools()?;
        let tools_array = tools_response["result"]["tools"]
            .as_array()
            .ok_or("Invalid tools response")?;
        
        for tool_json in tools_array {
            let tool = ToolDefinition {
                name: tool_json["name"].as_str().ok_or("Missing tool name")?.to_string(),
                description: tool_json["description"].as_str().unwrap_or("").to_string(),
                parameters: tool_json["inputSchema"].clone(),
            };
            self.tools.push(tool);
        }
        
        self.servers.insert(name.to_string(), server);
        Ok(())
    }

    pub fn tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    pub fn chat(&mut self, message: &str) -> Result<String, String> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: message.to_string(),
        }];
        
        self.chat_loop(messages)
    }

    pub fn chat_with_history(&mut self, messages: Vec<Message>) -> Result<String, String> {
        self.chat_loop(messages)
    }

    fn chat_loop(&mut self, mut messages: Vec<Message>) -> Result<String, String> {
        let mut iterations = 0;
        let max_iterations = 50;

        loop {
            if iterations >= max_iterations {
                return Ok("Agent reached maximum tool call iterations.".to_string());
            }
            iterations += 1;

            let response = self.client.chat(messages.clone(), self.tools.clone())?;
            
            if response.tool_calls.is_empty() {
                return Ok(response.content.unwrap_or_else(|| "No response".to_string()));
            }

            if let Some(ref content) = response.content {
                if !content.is_empty() {
                    eprintln!("[orchestrator] assistant: {}", content);
                }
            }
            
            for tool_call in &response.tool_calls {
                eprintln!("[orchestrator] calling {}({})", tool_call.name, tool_call.arguments);
                
                let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
                    .map_err(|e| format!("Failed to parse tool arguments: {}", e))?;
                
                let tool_result = self.execute_tool(&tool_call.name, args)?;
                
                eprintln!("[orchestrator] result: {}", if tool_result.len() > 200 { &tool_result[..200] } else { &tool_result });
                
                messages.push(Message {
                    role: "assistant".to_string(),
                    content: format!("{}({})", tool_call.name, tool_call.arguments),
                });
                
                messages.push(Message {
                    role: "user".to_string(),
                    content: format!("Tool result: {}", tool_result),
                });
            }
        }
    }

    fn execute_tool(&mut self, name: &str, args: serde_json::Value) -> Result<String, String> {
        for (_, server) in &mut self.servers {
            if let Ok(result) = server.call_tool(name, args.clone()) {
                return Ok(result);
            }
        }
        Err(format!("Tool '{}' not found in any server", name))
    }
}