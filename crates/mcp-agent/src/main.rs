mod config;

use clap::Parser;
use mcp_client::{Message, Orchestrator, OllamaProvider, SkillLoader};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "mcp-agent")]
#[command(about = "Headless AI agent — runs a skill with a prompt, no TUI")]
#[command(version)]
struct Args {
    skill: String,
    #[clap(trailing_var_arg = true)]
    prompt: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let prompt = args.prompt.join(" ");
    if prompt.is_empty() {
        eprintln!("Usage: mcp-agent <skill> \"<prompt>\"");
        std::process::exit(1);
    }

    let config = config::Config::load().unwrap_or_else(|e| {
        eprintln!("[mcp-agent] config: {} (using defaults)", e);
        config::Config::default_config()
    });

    if let Err(e) = run(&args.skill, &prompt, &config) {
        eprintln!("[mcp-agent] error: {}", e);
        std::process::exit(1);
    }
}

fn run(skill_name: &str, prompt: &str, config: &config::Config) -> Result<(), String> {
    std::env::set_var("MCP_ALLOWED_ROOT", config.allowed_root().to_string_lossy().to_string());

    let loader = SkillLoader::new();
    let manifest = loader.load_by_name(skill_name)?;
    let skill = manifest.render(&HashMap::new());

    eprintln!("[mcp-agent] skill: {} — {}", skill.name, skill.description);
    eprintln!("[mcp-agent] model: {}", config.model.name);
    eprintln!("[mcp-agent] tools: {:?}", skill.tools);

    let provider = OllamaProvider::new(&config.model.base_url, &config.model.name);
    let mut orchestrator = Orchestrator::new(provider);

    for tool_name in &skill.tools {
        let binary = config.tool_binary_path(tool_name);
        match orchestrator.spawn_tool(tool_name, &binary) {
            Ok(_) => eprintln!("[mcp-agent]   {} ✓", tool_name),
            Err(e) => {
                eprintln!("[mcp-agent]   {} ✗ ({})", tool_name, e);
                return Err(format!("Failed to spawn tool '{}': {}", tool_name, e));
            }
        }
    }

    let mut system_content = skill.prompt.clone();
    if !system_content.is_empty() {
        let mut constraint_hints = Vec::new();
        if let Some(timeout) = skill.constraints.timeout_secs {
            constraint_hints.push(format!("Each shell command must complete within {} seconds. Use timeout_secs parameter on run_command when needed.", timeout));
        }
        if let Some(max_iter) = skill.constraints.max_iterations {
            constraint_hints.push(format!("You have a maximum of {} tool-call iterations. Plan your steps accordingly.", max_iter));
        }
        if !constraint_hints.is_empty() {
            system_content.push_str("\n\n# CONSTRAINTS\n");
            system_content.push_str(&constraint_hints.join("\n"));
        }
    }

    let mut messages = Vec::new();
    if !system_content.is_empty() {
        messages.push(Message {
            role: "system".to_string(),
            content: system_content,
        });
    }
    messages.push(Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    });

    eprintln!("[mcp-agent] running...");

    let response = orchestrator.chat_with_history(messages)?;

    println!("{}", response);
    Ok(())
}