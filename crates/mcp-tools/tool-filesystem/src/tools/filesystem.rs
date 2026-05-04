use mcp_core::{Tool, ToolEntry, ToolError};
use serde_json::Value;
use std::fs;
use regex::Regex;

use tool_filesystem::validation::validate_path;

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &'static str { "read_file" }
    fn description(&self) -> &'static str { "Read the contents of a file." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "The path to the file to read" }
            },
            "required": ["path"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let raw_path = args["path"].as_str().ok_or_else(|| ToolError::MissingArgument("path".into()))?;
        let path = validate_path(raw_path)?;
        fs::read_to_string(&path).map_err(|e| ToolError::ExecutionError(format!("Failed to read file: {}", e)))
    }
}

inventory::submit! { ToolEntry { tool: &ReadFileTool } }

pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> &'static str { "write_file" }
    fn description(&self) -> &'static str { "Write content to a file. Creates the file if it doesn't exist, overwrites if it does." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "The path to the file to write" },
                "content": { "type": "string", "description": "The content to write to the file" }
            },
            "required": ["path", "content"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let raw_path = args["path"].as_str().ok_or_else(|| ToolError::MissingArgument("path".into()))?;
        let content = args["content"].as_str().ok_or_else(|| ToolError::MissingArgument("content".into()))?;
        let path = validate_path(raw_path)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ToolError::ExecutionError(format!("Failed to create parent directories: {}", e)))?;
        }
        fs::write(&path, content).map_err(|e| ToolError::ExecutionError(format!("Failed to write file: {}", e)))?;
        Ok(format!("Successfully wrote to {}", path.display()))
    }
}

inventory::submit! { ToolEntry { tool: &WriteFileTool } }

pub struct ListDirectoryTool;

impl Tool for ListDirectoryTool {
    fn name(&self) -> &'static str { "list_directory" }
    fn description(&self) -> &'static str { "List the contents of a directory." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "The path to the directory to list (default: allowed root)" }
            },
            "required": []
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let raw_path = args["path"].as_str().unwrap_or(".");
        let path = validate_path(raw_path)?;
        let entries: Vec<String> = fs::read_dir(&path)
            .map_err(|e| ToolError::ExecutionError(format!("Failed to read directory: {}", e)))?
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect();
        Ok(entries.join("\n"))
    }
}

inventory::submit! { ToolEntry { tool: &ListDirectoryTool } }

pub struct SearchFilesTool;

impl Tool for SearchFilesTool {
    fn name(&self) -> &'static str { "search_files" }
    fn description(&self) -> &'static str { "Search for files matching a pattern in a directory." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "The regex pattern to search for in file names" },
                "directory": { "type": "string", "description": "The directory to search in (default: allowed root)" }
            },
            "required": ["pattern"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let pattern = args["pattern"].as_str().ok_or_else(|| ToolError::MissingArgument("pattern".into()))?;
        let raw_directory = args["directory"].as_str().unwrap_or(".");
        let directory = validate_path(raw_directory)?;
        let re = Regex::new(pattern).map_err(|e| ToolError::ExecutionError(format!("Invalid regex pattern: {}", e)))?;
        let matches: Vec<String> = fs::read_dir(&directory)
            .map_err(|e| ToolError::ExecutionError(format!("Failed to read directory: {}", e)))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                re.is_match(&name)
            })
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        if matches.is_empty() {
            Ok("No files found matching the pattern".to_string())
        } else {
            Ok(matches.join("\n"))
        }
    }
}

inventory::submit! { ToolEntry { tool: &SearchFilesTool } } 