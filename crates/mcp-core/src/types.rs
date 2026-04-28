// src/types.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Debug)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: Value,
    pub result: T,
}

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
pub struct InitializeResult {
    pub protocolVersion: String,
    pub capabilities: Capabilities,
    pub serverInfo: ServerInfo,
}

#[derive(Serialize, Debug)]
pub struct Capabilities {
    pub tools: Value,
}

#[derive(Serialize, Debug)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}
