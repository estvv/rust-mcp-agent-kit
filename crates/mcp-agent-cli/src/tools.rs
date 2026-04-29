// src/tools.rs

use serde_json::Value;
use std::collections::HashMap;

pub fn get_tool_schemas_for_skill(skill_tool_names: &[String]) -> Vec<Value> {
    let tool_mappings: HashMap<&str, Vec<&str>> = [
        ("tool-filesystem", vec!["read_file", "write_file", "list_directory", "search_files"]),
        ("tool-web", vec!["fetch_webpage"]),
        ("tool-utilities", vec!["run_command"]),
        ("tool-weather", vec!["get_weather"]),
    ].iter().cloned().collect();

    let all_schemas = get_tool_schemas();

    let enabled_functions: Vec<&str> = skill_tool_names
        .iter()
        .flat_map(|name| {
            tool_mappings.get(name.as_str()).cloned().unwrap_or_default()
        })
        .collect();

    all_schemas
        .into_iter()
        .filter(|schema| {
            let name = schema["function"]["name"].as_str().unwrap_or("");
            enabled_functions.contains(&name)
        })
        .collect()
}

fn get_tool_schemas() -> Vec<Value> {
    vec![
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read the contents of a file from the filesystem",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to read"
                        }
                    },
                    "required": ["path"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write content to a file on the filesystem",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "list_directory",
                "description": "List files and directories in a given path",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The directory path to list"
                        }
                    },
                    "required": ["path"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "search_files",
                "description": "Search for files matching a pattern in a directory",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The directory to search in"
                        },
                        "pattern": {
                            "type": "string",
                            "description": "The glob pattern to match files"
                        }
                    },
                    "required": ["path", "pattern"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fetch_webpage",
                "description": "Fetch content from a URL",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch"
                        }
                    },
                    "required": ["url"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Run a shell command",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to run"
                        }
                    },
                    "required": ["command"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather information for a city",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "The city name"
                        }
                    },
                    "required": ["city"]
                }
            }
        }),
    ]
}

pub fn execute_tool(name: &str, arguments: &str) -> String {
    let args: Value = serde_json::from_str(arguments).unwrap_or(serde_json::json!({}));

    match name {
        "read_file" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
            match std::fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => format!("Error reading file: {}", e),
            }
        }
        "write_file" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
            let content = args.get("content").and_then(|c| c.as_str()).unwrap_or("");
            match std::fs::write(path, content) {
                Ok(()) => format!("File written successfully: {}", path),
                Err(e) => format!("Error writing file: {}", e),
            }
        }
        "list_directory" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            match std::fs::read_dir(path) {
                Ok(entries) => {
                    let items: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    items.join("\n")
                }
                Err(e) => format!("Error listing directory: {}", e),
            }
        }
        "search_files" => {
            let path = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            let pattern = args.get("pattern").and_then(|p| p.as_str()).unwrap_or("*");
            match glob::Pattern::new(pattern) {
                Ok(pat) => {
                    let results: Vec<_> = std::fs::read_dir(path)
                        .ok()
                        .into_iter()
                        .flat_map(|entries| entries.filter_map(|e| e.ok()))
                        .filter(|e| pat.matches(&e.file_name().to_string_lossy()))
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    results.join("\n")
                }
                Err(e) => format!("Invalid pattern: {}", e),
            }
        }
        "fetch_webpage" => {
            let url = args.get("url").and_then(|u| u.as_str()).unwrap_or("");
            format!("Fetch webpage not yet implemented for URL: {}", url)
        }
        "run_command" => {
            let command = args.get("command").and_then(|c| c.as_str()).unwrap_or("");
            format!("Run command not yet implemented for: {}", command)
        }
        "get_weather" => {
            let city = args.get("city").and_then(|c| c.as_str()).unwrap_or("unknown");
            format!("Weather for {}: 18C, Partly cloudy (mock)", city)
        }
        _ => format!("Unknown tool: {}", name),
    }
}
