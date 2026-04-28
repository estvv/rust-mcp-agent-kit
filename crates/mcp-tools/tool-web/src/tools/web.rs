use mcp_core::{Tool, ToolEntry, ToolError};
use serde_json::Value;

pub struct HttpGetTool;

impl Tool for HttpGetTool {
    fn name(&self) -> &'static str { "http_get" }
    fn description(&self) -> &'static str { "Make an HTTP GET request to a URL." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to request" }
            },
            "required": ["url"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let url = args["url"].as_str().ok_or_else(|| ToolError::MissingArgument("url".into()))?;
        let response = minreq::get(url)
            .send()
            .map_err(|e| ToolError::ExecutionError(format!("HTTP request failed: {}", e)))?;
        let status = response.status_code;
        let body = response.as_str().map_err(|e| ToolError::ExecutionError(format!("Failed to read response: {}", e)))?;
        Ok(format!("Status: {}\n\n{}", status, body))
    }
}

inventory::submit! { ToolEntry { tool: &HttpGetTool } }

pub struct HttpPostTool;

impl Tool for HttpPostTool {
    fn name(&self) -> &'static str { "http_post" }
    fn description(&self) -> &'static str { "Make an HTTP POST request to a URL with a JSON body." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to request" },
                "body": { "type": "object", "description": "The JSON body to send" }
            },
            "required": ["url", "body"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let url = args["url"].as_str().ok_or_else(|| ToolError::MissingArgument("url".into()))?;
        let body = args.get("body").ok_or_else(|| ToolError::MissingArgument("body".into()))?;
        let body_str = serde_json::to_string(body).map_err(|e| ToolError::ExecutionError(format!("Failed to serialize body: {}", e)))?;
        let response = minreq::post(url)
            .with_header("Content-Type", "application/json")
            .with_body(body_str)
            .send()
            .map_err(|e| ToolError::ExecutionError(format!("HTTP request failed: {}", e)))?;
        let status = response.status_code;
        let response_body = response.as_str().map_err(|e| ToolError::ExecutionError(format!("Failed to read response: {}", e)))?;
        Ok(format!("Status: {}\n\n{}", status, response_body))
    }
}

inventory::submit! { ToolEntry { tool: &HttpPostTool } }