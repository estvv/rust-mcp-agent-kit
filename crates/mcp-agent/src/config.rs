use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub model: ModelConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub paths: PathsConfig,
}

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_base_url() -> String {
    "http://localhost:11434".to_string()
}

#[derive(Debug, Deserialize, Default)]
pub struct ToolsConfig {
    #[serde(default)]
    pub allowed_root: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PathsConfig {
    #[serde(flatten)]
    pub tool_paths: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let home = std::env::var("HOME").unwrap_or_default();
        let config_paths = [
            ".mcp-agent/config.toml",
            &format!("{}/.mcp-agent/config.toml", home),
        ];

        for path in &config_paths {
            let p = PathBuf::from(path);
            if p.exists() {
                let content = std::fs::read_to_string(&p)
                    .map_err(|e| format!("Failed to read config '{}': {}", path, e))?;
                return toml::from_str(&content)
                    .map_err(|e| format!("Failed to parse config '{}': {}", path, e));
            }
        }

        Ok(Config::default_config())
    }

    pub fn default_config() -> Self {
        Config {
            model: ModelConfig {
                name: "glm-5:cloud".to_string(),
                base_url: default_base_url(),
            },
            tools: ToolsConfig::default(),
            paths: PathsConfig::default(),
        }
    }

    pub fn tool_binary_path(&self, tool_name: &str) -> String {
        if let Some(path) = self.paths.tool_paths.get(tool_name) {
            return path.clone();
        }
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.to_path_buf()));
        match exe_dir {
            Some(dir) => dir.join(tool_name).to_string_lossy().to_string(),
            None => format!("target/release/{}", tool_name),
        }
    }

    pub fn allowed_root(&self) -> PathBuf {
        match &self.tools.allowed_root {
            Some(root) => {
                let p = PathBuf::from(root);
                if p.is_absolute() {
                    p
                } else {
                    std::env::current_dir().unwrap_or_default().join(p)
                }
            }
            None => std::env::current_dir().unwrap_or_default(),
        }
    }
}