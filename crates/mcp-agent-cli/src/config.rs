// src/config.rs

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mcp-agent-cli")]
#[command(about = "Interactive TUI for AI chat with MCP tools")]
#[command(version)]
pub struct Args {
    #[arg(short, long, default_value = "coding")]
    pub skill: String,

    #[arg(short, long, default_value = "glm-5:cloud")]
    pub model: String,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
