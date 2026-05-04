# rust-mcp-agent-kit

A Rust-based MCP (Model Context Protocol) implementation for building AI coding agents.

This project provides the infrastructure to build your own AI agent (like opencode, Claude Code, or Aider), with full control over tools, LLM backend, and data privacy. It's a **learning project** to understand how AI agents work under the hood.

## What this project IS

- An **MCP implementation in Rust** following the [open MCP standard](https://modelcontextprotocol.io/)
- A **toolkit for building AI agents** that can read/write files, run commands, query APIs
- A **client library** (`mcp-client`) that orchestrates LLMs and MCP tool servers
- A **headless binary** (`mcp-agent`) for running skills from the command line
- A **TUI** (`mcp-agent-cli`) for interactive AI chat with tools
- A **collection of tools** (`mcp-tools/`) organized into skills
- **Modular skills** (`skills/`) in Markdown + YAML frontmatter format
- **Local-first**: Works with Ollama, but supports any LLM (OpenAI, Anthropic, etc.)

## What this project is NOT

- NOT a fork or copy of opencode, Claude Code, or Aider (built from scratch in Rust)
- NOT tied to any specific LLM provider (use Ollama, OpenAI, Anthropic, or others)
- NOT a closed system (open standard, extensible, you own the code)
- NOT a production-ready product (it's a learning/hobby project)

## Quick Start

```bash
# Clone and build
git clone https://github.com/estvv/rust-mcp-agent-kit.git
cd rust-mcp-agent-kit
cargo build --release

# Run a skill headlessly
./target/release/mcp-agent coding "list the files in this directory"

# Run with a state-machine skill
./target/release/mcp-agent init-web-project "scaffold the project"

# Or use the TUI
./target/release/mcp-agent-cli
```

## Architecture

```
  ┌────────────────────────────────────────────────────────────────┐
  │                         USER INPUT                             │
  └───────────────────────┬────────────────────────────────────────┘
                          │
            ┌─────────────┴─────────────┐
            ▼                           ▼
  ┌──────────────────┐        ┌──────────────────┐
  │   mcp-agent      │        │ mcp-agent-cli     │
  │  (headless CLI)  │        │   (TUI)           │
  │                  │        │                    │
  │  mcp-agent \     │        │  /skill coding    │
  │    coding "fix"  │        │  > fix the bug    │
  └────────┬─────────┘        └────────┬──────────┘
           │                           │
           └───────────┬───────────────┘
                       ▼
          ┌────────────────────────┐
          │      mcp-client        │
          │   (Orchestrator +      │
          │    SkillLoader +       │
          │    LLM Providers)     │
          └───────────┬────────────┘
                      │
      ┌───────┬───────┼───────┬───────────┐
      ▼       ▼       ▼       ▼           ▼
  tool-    tool-    tool-  tool-      tool-
  weather  filesystem system  web        shell
  (bin)    (bin)    (bin)   (bin)      (bin)
```

## Usage

### Headless (`mcp-agent`)

```bash
# Build
cargo build --release

# Run a skill with a prompt
mcp-agent <skill> "<prompt>"

# Examples
mcp-agent coding "fix the typo in README.md"
mcp-agent init-web-project "scaffold a react project"
mcp-agent personal "what's the weather in Tokyo?"
mcp-agent devops "show me the top processes by CPU"
mcp-agent data "format the JSON in package.json"
```

All config comes from `.mcp-agent/config.toml`:

```toml
[model]
name = "glm-5:cloud"
base_url = "http://localhost:11434"

[tools]
allowed_root = "."

[paths]
# tool-shell = "/usr/local/bin/tool-shell"
```

### TUI (`mcp-agent-cli`)

```bash
# Run with defaults (coding skill, glm-5:cloud model)
./target/release/mcp-agent-cli

# Specific skill and model
./target/release/mcp-agent-cli --skill personal --model glm-4

# Inside the TUI:
/skill coding
What files are in this directory?
/model glm-4
/skill personal
What's the weather in Tokyo?
```

## Project Structure

```
rust-mcp-agent-kit/
├── .mcp-agent/
│   ├── config.example.toml      # Project config template (model, allowed_root, tool paths)
│   └── config.toml              # Local config (gitignored)
├── crates/
│   ├── mcp-core/                # Protocol library (JSON-RPC, Tool trait, Server)
│   ├── mcp-client/              # Client library (Orchestrator, SkillLoader, LLM providers)
│   ├── mcp-agent/               # Headless binary — mcp-agent <skill> "<prompt>"
│   ├── mcp-agent-cli/           # Interactive TUI
│   └── mcp-tools/
│       ├── tool-filesystem/     # read_file, write_file, list_directory, search_files
│       ├── tool-shell/          # run_command (process group isolation, timeout, output truncation)
│       ├── tool-system/         # get_ram_usage, get_cpu_usage, get_disk_usage, get_processes
│       ├── tool-web/            # http_get, http_post
│       ├── tool-weather/        # get_weather
│       └── tool-utilities/      # calculate, format_json, current_time
└── skills/                      # Markdown-based skill definitions
    ├── coding.md
    ├── personal.md
    ├── devops.md
    ├── data.md
    └── init-web-project.md      # State-machine skill (PLAN → EXECUTE → VERIFY)
```

## Crates

| Crate | Description | Type |
|-------|-------------|------|
| `mcp-core` | Protocol: JSON-RPC types, Tool/Command traits, Server | Library |
| `mcp-client` | Orchestrator, SkillLoader, LLM providers (Ollama, Mock) | Library |
| `mcp-agent` | Headless binary: `mcp-agent <skill> "<prompt>"` | Binary |
| `mcp-agent-cli` | Interactive TUI for AI chat | Binary |

## Tools

All tools are standalone binaries under `crates/mcp-tools/`. They communicate via stdin/stdout JSON-RPC 2.0.

| Tool Crate | Functions | Security |
|------------|-----------|----------|
| `tool-filesystem` | read_file, write_file, list_directory, search_files | Lexical path guard (MCP_ALLOWED_ROOT) |
| `tool-shell` | run_command | Process group kill, timeout, env injection, output truncation, working dir validation |
| `tool-system` | get_ram_usage, get_cpu_usage, get_disk_usage, get_processes | Read-only |
| `tool-web` | http_get, http_post | — |
| `tool-weather` | get_weather | — |
| `tool-utilities` | calculate, format_json, current_time | — |

## Skills

Skills are Markdown files with YAML frontmatter. They define which tools an agent can use, behavioral constraints, and the prompt sent to the LLM.

| Skill | Description | Tools | State Machine |
|-------|-------------|-------|---------------|
| `coding` | Coding assistance | filesystem, web, utilities | — |
| `personal` | Personal assistant | weather, utilities | — |
| `devops` | DevOps operations | system, web, utilities | — |
| `data` | Data processing | utilities, filesystem | — |
| `init-web-project` | Scaffold a web project | shell, filesystem | PLAN → EXECUTE → VERIFY |

### Skill Format

```markdown
---
skill: my-skill
description: "What this skill does"
tools:
  - tool-filesystem
  - tool-shell
constraints:
  timeout_secs: 60
  max_output_chars: 3000
  max_iterations: 3
state_machine:
  - PLAN
  - EXECUTE
  - VERIFY
input_required: false
---

# ROLE
You are a senior engineer...

# PROCESS
Instructions for the LLM, with {{max_iterations}} template variables.
```

### Skill Resolution

Skills load from three tiers (first match wins):

1. **Project**: `.mcp-agent/skills/`
2. **User**: `~/.config/mcp-agent/skills/`
3. **Base**: `<executable_dir>/skills/`

### Creating a Custom Skill

1. Create `skills/my-skill.md` (or `.mcp-agent/skills/my-skill.md` for project-level)
2. Run: `mcp-agent my-skill "do the thing"`

## Key Design Decisions

### Tools as Separate Binaries

- Each tool is a standalone process communicating via stdin/stdout JSON-RPC
- `tool-filesystem` validates paths lexically (no `canonicalize()`) against `MCP_ALLOWED_ROOT`
- `tool-shell` runs commands in isolated process groups with `libc::kill(-pgid, SIGKILL)` on timeout

### Skills as Markdown

- YAML frontmatter for config (tools, constraints, state machine)
- Markdown body for the LLM prompt (role, process, rules)
- Template variables (`{{max_iterations}}`) resolved at render time
- `SkillManifest` (parsed) → `Skill` (rendered) separation

### Config Layering

| Layer | Source | Controls |
|-------|--------|----------|
| Tool defaults | Hardcoded in binary | timeout=30, max_output=2000 |
| Skill overrides | `.md` frontmatter `constraints` | timeout=60 for init-web-project |
| Deployment | `.mcp-agent/config.toml` | model, allowed_root, binary paths |

## Using mcp-client Programmatically

```rust
use mcp_client::{Orchestrator, OllamaProvider, SkillLoader};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = SkillLoader::new();
    let manifest = loader.load_by_name("coding")?;
    let skill = manifest.render(&HashMap::new());

    let provider = OllamaProvider::new("http://localhost:11434", "glm-5:cloud");
    let mut orch = Orchestrator::new(provider);

    for tool in &skill.tools {
        orch.spawn_tool(tool, &format!("target/release/{}", tool))?;
    }

    let response = orch.chat("list the files in this directory")?;
    println!("{}", response);
    Ok(())
}
```

## Model Compatibility

Works with any LLM that supports function calling via OpenAI-compatible `/v1/chat/completions`:

| Provider | Models |
|----------|--------|
| **Ollama** | GLM-4, GLM-5:cloud, Llama 3.1+, Mistral, Qwen |
| **OpenAI** | GPT-4, GPT-4o, GPT-3.5-turbo |
| **Anthropic** | Claude 3+ (via OpenAI-compatible endpoint) |
| **Mock** | For testing without LLM |

## Development

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

## Documentation

- [docs/ROADMAP.md](./docs/ROADMAP.md) - Development roadmap
- [docs/FEATURES.md](./docs/FEATURES.md) - Feature reference
- [skills/README.md](./skills/README.md) - Skill format documentation

## License

MIT License

## References

- [MCP Specification](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Ollama OpenAI Compatibility](https://github.com/ollama/ollama/blob/main/docs/openai.md)

## Creating a New Tool

```rust
// crates/mcp-tools/tool-mytool/src/tools/mytool.rs
use mcp_core::{Tool, ToolEntry, ToolError};
use serde_json::Value;

pub struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &'static str { "my_tool" }
    fn description(&self) -> &'static str { "Description of what it does" }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string", "description": "Input parameter" }
            },
            "required": ["input"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let input = args["input"].as_str().ok_or_else(|| ToolError::MissingArgument("input".into()))?;
        Ok(format!("Processed: {}", input))
    }
}

inventory::submit! { ToolEntry { tool: &MyTool } }
```