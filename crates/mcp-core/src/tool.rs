// src/tools/tool.rs

use serde_json::Value;

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    fn execute(&self, arguments: Value) -> Result<String, ToolError>;
}

pub struct ToolEntry {
    pub tool: &'static (dyn Tool + Send + Sync),
}

#[derive(Debug)]
pub enum ToolError {
    MissingArgument(String),
    ExecutionError(String),
}

inventory::collect!(ToolEntry);
