// src/commands/initialize.rs

use serde_json::Value;
use crate::command::{Command, CommandEntry};
use crate::types::{Capabilities, InitializeResult, RpcRequest, RpcResponse, ServerInfo};
use crate::constants::{DATE, NAME, VERSION, JSONRPC_VERSION};

pub struct InitializeCommand;

impl Command for InitializeCommand {
    fn execute(&self, request: RpcRequest) -> Value {
        let result = InitializeResult {
            protocolVersion: DATE.to_string(),
            capabilities: Capabilities {
                tools: serde_json::json!({}),
            },
            serverInfo: ServerInfo {
                name: NAME.to_string(),
                version: VERSION.to_string(),
            },
        };

        let response = RpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: request.id.unwrap_or(serde_json::json!(null)),
            result,
        };

        serde_json::to_value(response).unwrap()
    }
}

inventory::submit! {
    CommandEntry {
        method: "initialize",
        command: &InitializeCommand,
    }
}
