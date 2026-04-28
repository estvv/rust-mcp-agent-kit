// src/tools/weather.rs

use mcp_core::{Tool, ToolEntry, ToolError};
use serde_json::Value;

pub struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &'static str {
        "get_weather"
    }

    fn description(&self) -> &'static str {
        "Get the current weather for a specified city."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The name of the city (e.g., Lyon, Paris, Tokyo)"
                }
            },
            "required": ["city"]
        })
    }

    fn execute(&self, arguments: Value) -> Result<String, ToolError> {
        let city = arguments["city"].as_str().ok_or_else(|| ToolError::MissingArgument("city".to_string()))?;
        let url = format!("https://wttr.in/{}?format=3", city);
        let response = minreq::get(&url).send().map_err(|e| ToolError::ExecutionError(e.to_string()))?;
        let weather = response.as_str().map(|s| s.to_string()).map_err(|e| ToolError::ExecutionError(e.to_string()))?;

        Ok(weather.trim().to_string())
    }
}

inventory::submit! {
    ToolEntry {
        tool: &WeatherTool,
    }
}
