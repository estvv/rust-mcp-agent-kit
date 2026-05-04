// examples/test_mock.rs

use mcp_client::{ChatClient, Message, MockProvider, ServerProcess, ToolDefinition};

fn main() {
    let mut server = match ServerProcess::spawn("weather", "target/debug/tool-weather") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to spawn server: {}", e);
            std::process::exit(1);
        }
    };

    println!("Spawning MCP server: {}", server.name());

    let init_result = server.initialize().expect("Failed to initialize");
    println!("Initialize response: {}", serde_json::to_string_pretty(&init_result).unwrap());

    let tools_response = server.list_tools().expect("Failed to list tools");
    let tools_json: Vec<serde_json::Value> = tools_response["result"]["tools"]
        .as_array()
        .unwrap()
        .clone();

    println!("Available tools: {}", serde_json::to_string_pretty(&tools_json).unwrap());

    let tools: Vec<ToolDefinition> = tools_json
        .iter()
        .map(|t| ToolDefinition {
            name: t["name"].as_str().unwrap().to_string(),
            description: t["description"].as_str().unwrap().to_string(),
            parameters: t["inputSchema"].clone(),
        })
        .collect();

    let messages = vec![Message {
        role: "user".to_string(),
        content: "What's the weather like in Paris?".to_string(),
    }];

    let client = MockProvider::with_tool_call("get_weather", r#"{"city": "Paris"}"#);

    println!("\nUsing MockProvider (no LLM required)");
    println!("Simulated tool call request.\n");

    let response = client.chat(messages, tools).expect("Mock should not fail");

    if !response.tool_calls.is_empty() {
        println!("Mock requested {} tool call(s):", response.tool_calls.len());

        for tool_call in &response.tool_calls {
            println!("  Tool: {}", tool_call.name);
            println!("  Arguments: {}", tool_call.arguments);

            let args: serde_json::Value =
                serde_json::from_str(&tool_call.arguments).expect("Failed to parse arguments");

            let tool_result = server.call_tool(&tool_call.name, args).expect("Tool call failed");
            println!("  Result: {}", tool_result);
        }
    } else if let Some(content) = &response.content {
        println!("Response: {}", content);
    }

    println!("\nDone.");
}
