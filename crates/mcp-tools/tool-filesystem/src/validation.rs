use mcp_core::ToolError;
use std::path::{Component, Path, PathBuf};

static ALLOWED_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

fn get_allowed_root() -> &'static PathBuf {
    ALLOWED_ROOT.get_or_init(|| {
        if let Ok(val) = std::env::var("MCP_ALLOWED_ROOT") {
            let p = PathBuf::from(val);
            lexical_normalize(&p)
        } else {
            std::env::current_dir()
                .map(|p| lexical_normalize(&p))
                .unwrap_or_else(|_| PathBuf::from("/"))
        }
    })
}

pub fn lexical_normalize(path: &Path) -> PathBuf {
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

pub fn validate_path_with_root(raw: &str, allowed_root: &Path) -> Result<PathBuf, ToolError> {
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

pub fn validate_path(raw: &str) -> Result<PathBuf, ToolError> {
    validate_path_with_root(raw, get_allowed_root())
}