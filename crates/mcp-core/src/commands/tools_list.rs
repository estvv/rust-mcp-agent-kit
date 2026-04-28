// src/commands/tools_list.rs

use serde_json::Value;
use crate::command::{Command, CommandEntry};
use crate::types::{RpcRequest, RpcResponse};
use crate::constants::JSONRPC_VERSION;
use crate::tool::ToolEntry;

pub struct ToolsListCommand;

impl Command for ToolsListCommand {
    fn execute(&self, request: RpcRequest) -> Value {
        let tools: Vec<Value> = inventory::iter::<ToolEntry>
            .into_iter()
            .map(|entry| {
                serde_json::json!({
                    "name": entry.tool.name(),
                    "description": entry.tool.description(),
                    "inputSchema": entry.tool.input_schema()
                })
            })
            .collect();

        let response = RpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: request.id.unwrap_or(serde_json::json!(null)),
            result: serde_json::json!({ "tools": tools }),
        };

        serde_json::to_value(response).unwrap()
    }
}

inventory::submit! {
    CommandEntry {
        method: "tools/list",
        command: &ToolsListCommand,
    }
}
