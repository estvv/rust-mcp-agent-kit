// src/command.rs

use crate::types::RpcRequest;
use serde_json::Value;

pub trait Command: Send + Sync {
    fn execute(&self, request: RpcRequest) -> Value;
}

pub struct CommandEntry {
    pub method: &'static str,
    pub command: &'static (dyn Command + Send + Sync),
}

inventory::collect!(CommandEntry);
