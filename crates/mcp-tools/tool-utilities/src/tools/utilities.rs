use mcp_core::{Tool, ToolEntry, ToolError};
use serde_json::Value;
use chrono::{DateTime, Local, Utc};

pub struct CalculateTool;

impl Tool for CalculateTool {
    fn name(&self) -> &'static str { "calculate" }
    fn description(&self) -> &'static str { "Evaluate a mathematical expression. Supports basic operations (+, -, *, /, %, parentheses)." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": { "type": "string", "description": "The mathematical expression to evaluate (e.g., '2 + 3 * 4')" }
            },
            "required": ["expression"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let expr = args["expression"].as_str().ok_or_else(|| ToolError::MissingArgument("expression".into()))?;
        let result = evaluate_expression(expr)?;
        Ok(result.to_string())
    }
}

fn evaluate_expression(expr: &str) -> Result<f64, ToolError> {
    let expr = expr.replace(" ", "");
    let tokens: Vec<char> = expr.chars().collect();
    let mut pos = 0;
    parse_expr(&tokens, &mut pos)
}

fn parse_expr(tokens: &[char], pos: &mut usize) -> Result<f64, ToolError> {
    let mut result = parse_term(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            '+' => { *pos += 1; result += parse_term(tokens, pos)?; }
            '-' => { *pos += 1; result -= parse_term(tokens, pos)?; }
            _ => break,
        }
    }
    Ok(result)
}

fn parse_term(tokens: &[char], pos: &mut usize) -> Result<f64, ToolError> {
    let mut result = parse_factor(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            '*' => { *pos += 1; result *= parse_factor(tokens, pos)?; }
            '/' => {
                *pos += 1;
                let divisor = parse_factor(tokens, pos)?;
                if divisor == 0.0 {
                    return Err(ToolError::ExecutionError("Division by zero".into()));
                }
                result /= divisor;
            }
            '%' => { *pos += 1; result %= parse_factor(tokens, pos)?; }
            _ => break,
        }
    }
    Ok(result)
}

fn parse_factor(tokens: &[char], pos: &mut usize) -> Result<f64, ToolError> {
    if *pos >= tokens.len() {
        return Err(ToolError::ExecutionError("Unexpected end of expression".into()));
    }
    if tokens[*pos] == '(' {
        *pos += 1;
        let result = parse_expr(tokens, pos)?;
        if *pos >= tokens.len() || tokens[*pos] != ')' {
            return Err(ToolError::ExecutionError("Missing closing parenthesis".into()));
        }
        *pos += 1;
        return Ok(result);
    }
    if tokens[*pos] == '-' {
        *pos += 1;
        return Ok(-parse_factor(tokens, pos)?);
    }
    let mut num_str = String::new();
    while *pos < tokens.len() && (tokens[*pos].is_ascii_digit() || tokens[*pos] == '.') {
        num_str.push(tokens[*pos]);
        *pos += 1;
    }
    num_str.parse::<f64>().map_err(|_| ToolError::ExecutionError(format!("Invalid number: {}", num_str)))
}

inventory::submit! { ToolEntry { tool: &CalculateTool } }

pub struct FormatJsonTool;

impl Tool for FormatJsonTool {
    fn name(&self) -> &'static str { "format_json" }
    fn description(&self) -> &'static str { "Parse and pretty-print a JSON string." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "json": { "type": "string", "description": "The JSON string to format" }
            },
            "required": ["json"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let json_str = args["json"].as_str().ok_or_else(|| ToolError::MissingArgument("json".into()))?;
        let parsed: Value = serde_json::from_str(json_str).map_err(|e| ToolError::ExecutionError(format!("Invalid JSON: {}", e)))?;
        serde_json::to_string_pretty(&parsed).map_err(|e| ToolError::ExecutionError(format!("Failed to format JSON: {}", e)))
    }
}

inventory::submit! { ToolEntry { tool: &FormatJsonTool } }

pub struct CurrentTimeTool;

impl Tool for CurrentTimeTool {
    fn name(&self) -> &'static str { "current_time" }
    fn description(&self) -> &'static str { "Get the current date and time." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "timezone": { "type": "string", "description": "The timezone to use (e.g., 'UTC', 'local'). Default: local" }
            },
            "required": []
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let timezone = args["timezone"].as_str().unwrap_or("local");
        let time_str = match timezone.to_lowercase().as_str() {
            "utc" => {
                let now: DateTime<Utc> = Utc::now();
                now.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }
            _ => {
                let now: DateTime<Local> = Local::now();
                now.format("%Y-%m-%d %H:%M:%S %Z").to_string()
            }
        };
        Ok(time_str)
    }
}

inventory::submit! { ToolEntry { tool: &CurrentTimeTool } }