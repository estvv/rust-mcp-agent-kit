use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Skill {
    skill: SkillInfo,
    tools: HashMap<String, ToolConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolConfig {
    pub enabled: bool,
}

impl Skill {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read skill: {}", e))?;
        
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse skill: {}", e))
    }

    pub fn load_by_name(name: &str) -> Result<Self, String> {
        let path = format!("skills/{}.toml", name);
        Self::load(&path)
    }

    pub fn name(&self) -> &str {
        &self.skill.name
    }

    pub fn description(&self) -> &str {
        &self.skill.description
    }

    pub fn enabled_tools(&self) -> Vec<String> {
        self.tools
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }
}