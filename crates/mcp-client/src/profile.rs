use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Profile {
    profile: ProfileInfo,
    tools: HashMap<String, ToolConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProfileInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolConfig {
    pub enabled: bool,
}

impl Profile {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read profile: {}", e))?;
        
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse profile: {}", e))
    }

    pub fn load_by_name(name: &str) -> Result<Self, String> {
        let path = format!("profiles/{}.toml", name);
        Self::load(&path)
    }

    pub fn name(&self) -> &str {
        &self.profile.name
    }

    pub fn description(&self) -> &str {
        &self.profile.description
    }

    pub fn enabled_tools(&self) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }
}