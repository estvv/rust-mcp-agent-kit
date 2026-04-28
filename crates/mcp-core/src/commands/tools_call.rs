// src/commands/tools_call.rs

use crate::command::{Command, CommandEntry};
use crate::constants::JSONRPC_VERSION;
use crate::tool::{ToolEntry, ToolError};
use crate::types::{RpcRequest, RpcResponse};
use serde_json::Value;

pub struct ToolsCallCommand;

fn error_response(id: Value, code: i32, message: String) -> Value {
    serde_json::json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn success_response(id: Value, text: String) -> Value {
    let response = RpcResponse {
        jsonrpc: JSONRPC_VERSION.to_string(),
        id,
        result: serde_json::json!({
            "content": [{ "type": "text", "text": text }]
        }),
    };

    serde_json::to_value(response).unwrap()
}

impl Command for ToolsCallCommand {
    fn execute(&self, request: RpcRequest) -> Value {
        let id = request.id.clone().unwrap_or(Value::Null);

        let params = match &request.params {
            Some(p) => p,
            None => return error_response(id, -32602, "Invalid params".into()),
        };

        let tool_name = match params["name"].as_str() {
            Some(name) => name,
            None => return error_response(id, -32602, "Missing 'name' in params".into()),
        };

        let arguments = params.get("arguments").cloned().unwrap_or(Value::Object(serde_json::Map::new()));

        let tool = inventory::iter::<ToolEntry>.into_iter().find(|entry| entry.tool.name() == tool_name).map(|entry| entry.tool);

        match tool {
            Some(tool) => match tool.execute(arguments) {
                Ok(result) => success_response(id, result),
                Err(ToolError::MissingArgument(arg)) => {
                    error_response(id, -32602, format!("Missing required argument: {}", arg))
                }
                Err(ToolError::ExecutionError(msg)) => {
                    error_response(id, -32603, format!("Internal error: {}", msg))
                }
            },
            None => error_response(id, -32601, format!("Tool not found: {}", tool_name)),
        }
    }
}

inventory::submit! {
    CommandEntry {
        method: "tools/call",
        command: &ToolsCallCommand,
    }
}
