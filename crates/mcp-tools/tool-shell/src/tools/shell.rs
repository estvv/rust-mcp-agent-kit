use mcp_core::{Tool, ToolEntry, ToolError};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};
use tokio::io::AsyncReadExt;

const MAX_OUTPUT_CHARS: usize = 2000;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const MIN_TIMEOUT_SECS: u64 = 1;
const MAX_TIMEOUT_SECS: u64 = 600;

static ALLOWED_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

fn get_allowed_root() -> &'static PathBuf {
    ALLOWED_ROOT.get_or_init(|| {
        if let Ok(val) = std::env::var("MCP_ALLOWED_ROOT") {
            lexical_normalize(Path::new(&val))
        } else {
            std::env::current_dir()
                .map(|p| lexical_normalize(&p))
                .unwrap_or_else(|_| PathBuf::from("/"))
        }
    })
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut stack: Vec<std::ffi::OsString> = Vec::new();
    let mut has_root = false;

    for comp in path.components() {
        match comp {
            Component::Prefix(p) => {
                stack.clear();
                stack.push(p.as_os_str().to_os_string());
                has_root = false;
            }
            Component::RootDir => {
                stack.clear();
                stack.push(std::ffi::OsString::from("/"));
                has_root = true;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if stack.last().map_or(false, |s| s != "/") && !stack.is_empty() {
                    stack.pop();
                }
            }
            Component::Normal(s) => {
                stack.push(s.to_os_string());
            }
        }
    }

    if stack.is_empty() {
        if has_root {
            return PathBuf::from("/");
        }
        return PathBuf::from(".");
    }

    let mut result = PathBuf::new();
    for (i, part) in stack.iter().enumerate() {
        if i == 0 && part == "/" {
            result.push("/");
        } else {
            result.push(part);
        }
    }
    result
}

fn validate_path(raw: &str) -> Result<PathBuf, ToolError> {
    let allowed_root = get_allowed_root();
    let raw_path = Path::new(raw);

    let resolved = if raw_path.is_absolute() {
        lexical_normalize(raw_path)
    } else {
        lexical_normalize(&allowed_root.join(raw_path))
    };

    let normalized_root = lexical_normalize(allowed_root);

    if resolved.starts_with(&normalized_root) {
        Ok(resolved)
    } else {
        Err(ToolError::ExecutionError(format!(
            "Path '{}' escapes allowed root '{}'",
            raw,
            normalized_root.display()
        )))
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let truncated = &s[..max.saturating_sub(14)];
        format!("{}...[TRUNCATED]", truncated)
    }
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
}

#[cfg(not(unix))]
fn kill_process_group(_pid: u32) {}

pub struct RunCommandTool;

impl Tool for RunCommandTool {
    fn name(&self) -> &'static str { "run_command" }
    fn description(&self) -> &'static str {
        "Execute a shell command with timeout and output capture. Commands run in a sandboxed environment with defensive env vars injected."
    }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Maximum execution time in seconds (1-600, default 30)"
                },
                "working_directory": {
                    "type": "string",
                    "description": "The directory to run the command in (must be within allowed root, default: allowed root)"
                }
            },
            "required": ["command"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let command = args["command"].as_str()
            .ok_or_else(|| ToolError::MissingArgument("command".into()))?;

        let timeout_secs = args["timeout_secs"].as_u64()
            .map(|t| t.clamp(MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS))
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        let working_directory = if let Some(wd) = args["working_directory"].as_str() {
            validate_path(wd)?
        } else {
            get_allowed_root().clone()
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| ToolError::ExecutionError(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let mut child = tokio::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(command)
                .current_dir(&working_directory)
                .env("CI", "true")
                .env("DEBIAN_FRONTEND", "noninteractive")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .process_group(0)
                .spawn()
                .map_err(|e| ToolError::ExecutionError(format!("Failed to spawn command: {}", e)))?;

            let child_pid = child.id();

            let mut stdout_handle = child.stdout.take();
            let mut stderr_handle = child.stderr.take();

            let timeout = tokio::time::Duration::from_secs(timeout_secs);

            let status_result = tokio::time::timeout(timeout, child.wait()).await;

            match status_result {
                Ok(Ok(status)) => {
                    let exit_code = status.code().unwrap_or(-1);

                    let mut stdout_buf = Vec::new();
                    let mut stderr_buf = Vec::new();

                    if let Some(ref mut handle) = stdout_handle {
                        let _ = handle.read_to_end(&mut stdout_buf).await;
                    }
                    if let Some(ref mut handle) = stderr_handle {
                        let _ = handle.read_to_end(&mut stderr_buf).await;
                    }

                    let stdout = String::from_utf8_lossy(&stdout_buf);
                    let stderr = String::from_utf8_lossy(&stderr_buf);

                    let mut result = String::new();
                    if !stdout.is_empty() {
                        result.push_str("STDOUT:\n");
                        result.push_str(&truncate(&stdout, MAX_OUTPUT_CHARS));
                        result.push('\n');
                    }
                    if !stderr.is_empty() {
                        result.push_str("STDERR:\n");
                        result.push_str(&truncate(&stderr, MAX_OUTPUT_CHARS));
                        result.push('\n');
                    }
                    result.push_str(&format!("EXIT CODE: {}", exit_code));
                    Ok(result)
                }
                Ok(Err(e)) => {
                    Err(ToolError::ExecutionError(format!("Failed to wait for command: {}", e)))
                }
                Err(_) => {
                    if let Some(pid) = child_pid {
                        kill_process_group(pid);
                    }
                    let _ = child.kill().await;
                    Err(ToolError::ExecutionError(format!(
                        "Command timed out after {}s (process group killed)",
                        timeout_secs
                    )))
                }
            }
        })
    }
}

inventory::submit! { ToolEntry { tool: &RunCommandTool } }