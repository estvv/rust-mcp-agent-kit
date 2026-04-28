// src/server_process.rs

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

pub struct ServerProcess {
    name: String,
    child: Child,
    next_id: i64,
}

impl ServerProcess {
    pub fn spawn(name: &str, cmd: &str) -> Result<Self, String> {
        let child = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn '{}': {}", cmd, e))?;

        Ok(Self {
            name: name.to_string(),
            child,
            next_id: 1,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn initialize(&mut self) -> Result<serde_json::Value, String> {
        self.send_request("initialize", serde_json::json!({}))
    }

    pub fn list_tools(&mut self) -> Result<serde_json::Value, String> {
        self.send_request("tools/list", serde_json::json!({}))
    }

    pub fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> Result<String, String> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let response = self.send_request("tools/call", params)?;

        response["result"]["content"][0]["text"].as_str().map(|s| s.to_string()).ok_or_else(|| "Invalid tool response".to_string())
    }

    fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": method,
            "params": params
        });

        self.next_id += 1;

        let stdin = self.child.stdin.as_mut().ok_or("Failed to get stdin")?;
        let stdout = self.child.stdout.as_mut().ok_or("Failed to get stdout")?;

        writeln!(stdin, "{}", serde_json::to_string(&request).unwrap())
            .map_err(|e| format!("Write failed: {}", e))?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        reader.read_line(&mut line).map_err(|e| format!("Read failed: {}", e))?;

        serde_json::from_str(&line).map_err(|e| format!("Parse failed: {}", e))
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
