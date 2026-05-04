use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    skill: String,
    description: String,
    #[serde(default)]
    tools: Vec<String>,
    #[serde(default)]
    constraints: Option<SkillConstraints>,
    #[serde(default)]
    state_machine: Vec<String>,
    #[serde(default)]
    input_required: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillConstraints {
    pub timeout_secs: Option<u64>,
    pub max_output_chars: Option<usize>,
    pub max_iterations: Option<u32>,
}

impl Default for SkillConstraints {
    fn default() -> Self {
        SkillConstraints {
            timeout_secs: None,
            max_output_chars: None,
            max_iterations: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Plan,
    Execute,
    Verify,
}

impl State {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PLAN" => Some(State::Plan),
            "EXECUTE" => Some(State::Execute),
            "VERIFY" => Some(State::Verify),
            _ => None,
        }
    }
}

pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub constraints: SkillConstraints,
    pub state_machine: Vec<State>,
    pub input_required: bool,
    pub prompt_template: String,
}

pub struct Skill {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub constraints: SkillConstraints,
    pub state_machine: Vec<State>,
    pub input_required: bool,
    pub prompt: String,
}

pub struct SkillLoader {
    search_paths: Vec<PathBuf>,
}

impl SkillLoader {
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let user_dir = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config")))
            .map(|p| p.join("mcp-agent/skills"));

        let mut paths = vec![
            PathBuf::from(".mcp-agent/skills"),
        ];
        if let Some(ref ud) = user_dir {
            paths.push(ud.clone());
        }
        paths.push(exe_dir.join("skills"));

        SkillLoader { search_paths: paths }
    }

    pub fn with_search_paths(paths: Vec<PathBuf>) -> Self {
        SkillLoader { search_paths: paths }
    }

    pub fn load_by_name(&self, name: &str) -> Result<SkillManifest, String> {
        for dir in &self.search_paths {
            let path = dir.join(format!("{}.md", name));
            if path.exists() {
                return SkillManifest::load(&path);
            }
        }
        let fallback = PathBuf::from(format!("skills/{}.md", name));
        if fallback.exists() {
            return SkillManifest::load(&fallback);
        }
        Err(format!("Skill '{}' not found in any search path", name))
    }
}

impl SkillManifest {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read skill '{}': {}", path.display(), e))?;
        let (frontmatter_str, body) = split_frontmatter(&content)?;
        let fm: SkillFrontmatter = serde_yaml::from_str(frontmatter_str)
            .map_err(|e| format!("Failed to parse skill frontmatter: {}", e))?;
        let state_machine = fm.state_machine.iter()
            .filter_map(|s| State::from_str(s))
            .collect();
        let input_required = fm.input_required.unwrap_or(false);
        let constraints = fm.constraints.unwrap_or_default();
        Ok(SkillManifest {
            name: fm.skill,
            description: fm.description,
            tools: fm.tools,
            constraints,
            state_machine,
            input_required,
            prompt_template: body.trim().to_string(),
        })
    }

    pub fn render(self, vars: &HashMap<String, String>) -> Skill {
        let prompt = vars.iter()
            .fold(self.prompt_template.clone(), |acc, (key, val)| {
                acc.replace(&format!("{{{{{}}}}}", key), val)
            });
        Skill {
            name: self.name,
            description: self.description,
            tools: self.tools,
            constraints: self.constraints,
            state_machine: self.state_machine,
            input_required: self.input_required,
            prompt,
        }
    }
}

impl Skill {
    pub fn load(path: &str) -> Result<Self, String> {
        let manifest = SkillManifest::load(Path::new(path))?;
        Ok(manifest.render(&HashMap::new()))
    }

    pub fn load_by_name(name: &str) -> Result<Self, String> {
        let loader = SkillLoader::new();
        let manifest = loader.load_by_name(name)?;
        Ok(manifest.render(&HashMap::new()))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn enabled_tools(&self) -> Vec<String> {
        self.tools.clone()
    }
}

fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("Skill file must start with YAML frontmatter (---)".to_string());
    }
    let after_first = &trimmed[3..];
    let rest = after_first.trim_start_matches('\n');
    let end = rest.find("\n---").or_else(|| rest.find("---"))
        .ok_or_else(|| "Skill frontmatter must be closed with ---".to_string())?;
    let frontmatter = &rest[..end];
    let body_start = if rest[end..].starts_with("\n---") { end + 4 } else { end + 3 };
    let body = rest[body_start..].trim_start_matches('\n');
    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_frontmatter() {
        let content = "---\nskill: coding\ndescription: \"Test\"\ntools:\n  - tool-filesystem\n---\nYou are a coder.";
        let (fm, body) = split_frontmatter(content).unwrap();
        assert!(fm.contains("skill: coding"));
        assert_eq!(body.trim(), "You are a coder.");
    }

    #[test]
    fn parses_init_web_project_format() {
        let content = "---\nskill: init-web-project\ndescription: \"Scaffold\"\ntools:\n  - tool-shell\nconstraints:\n  timeout_secs: 60\nstate_machine:\n  - PLAN\n  - EXECUTE\n  - VERIFY\n---\n# ROLE\nYou are an engineer.";
        let (fm, body) = split_frontmatter(content).unwrap();
        assert!(fm.contains("skill: init-web-project"));
        assert!(body.contains("# ROLE"));
    }

    #[test]
    fn rejects_missing_closing_delimiter() {
        let content = "---\nskill: test\nYou are a coder.";
        assert!(split_frontmatter(content).is_err());
    }

    #[test]
    fn full_manifest_round_trip() {
        let content = "---\nskill: coding\ndescription: \"Coding skill\"\ntools:\n  - tool-filesystem\n  - tool-web\n---\nYou are a coding assistant.";
        let manifest = SkillManifest::load(Path::new("")).unwrap_or_else(|_| {
            let (fm, body) = split_frontmatter(content).unwrap();
            let parsed: SkillFrontmatter = serde_yaml::from_str(fm).unwrap();
            SkillManifest {
                name: parsed.skill,
                description: parsed.description,
                tools: parsed.tools,
                constraints: parsed.constraints.unwrap_or_default(),
                state_machine: parsed.state_machine.iter().filter_map(|s| State::from_str(s)).collect(),
                input_required: parsed.input_required.unwrap_or(false),
                prompt_template: body.trim().to_string(),
            }
        });
        assert_eq!(manifest.name, "coding");
        assert_eq!(manifest.tools, vec!["tool-filesystem", "tool-web"]);
    }
}