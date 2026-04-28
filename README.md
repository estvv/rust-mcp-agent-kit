# rust-mcp-agent-kit

A Rust-based MCP (Model Context Protocol) implementation for building AI coding agents.

This project provides the infrastructure to build your own AI agent (like opencode, Claude Code, or Aider), with full control over tools, LLM backend, and data privacy. It's a **learning project** to understand how AI agents work under the hood.

## What this project IS

- An **MCP implementation in Rust** following the [open MCP standard](https://modelcontextprotocol.io/)
- A **toolkit for building AI agents** that can read/write files, run commands, query APIs
- A **client library** (`mcp-client`) that orchestrates LLMs and MCP tool servers
- A **CLI** (`mcp-agent-cli`) for interactive AI chat with tools
- A **collection of tools** (`mcp-tools/`) organized into profiles
- **Modular profiles** (`profiles/`) that define which tools an agent can use
- **Local-first**: Works with Ollama, but supports any LLM (OpenAI, Anthropic, etc.)

## What this project is NOT

- NOT a fork or copy of opencode, Claude Code, or Aider (built from scratch in Rust)
- NOT tied to any specific LLM provider (use Ollama, OpenAI, Anthropic, or others)
- NOT a closed system (open standard, extensible, you own the code)
- NOT a production-ready product (it's a learning/hobby project)

## Why build this?

| Reason | Benefit |
|--------|---------|
| **Full control** | Add any tool you want, no limitations |
| **Privacy** | Everything can run locally (Ollama) |
| **Learning** | Understand how AI agents work under the hood |
| **Custom LLM** | Switch between local (free) and cloud (powerful) |
| **Open standard** | MCP ecosystem is growing, interoperable |

## Architecture

```
     ┌────────────────────────────────────────────────────────────────────┐
     │                             USER INPUT                             │
     └─────────────────────────────────┬──────────────────────────────────┘
                                       │
                                       ▼
     ┌────────────────────────────────────────────────────────────────────┐
     │                                                                    │
     │                       mcp-agent-cli (TUI)                          │
     │                                                                    │
     │    ┌──────────────────────────────────────────────────────────┐    │
     │    │  STATUS BAR: Model: glm-5:cloud | Profile: coding | 3t   │    │
     │    └──────────────────────────────────────────────────────────┘    │
     │    ┌──────────────────────────────────────────────────────────┐    │
     │    │                                                          │    │
     │    │  [User] What's the weather in Paris?                     │    │
     │    │                                                          │    │
     │    │  [Assistant] I'll check the weather for you.             │    │
     │    │  [Tool: get_weather] {"city": "Paris"}                   │    │
     │    │  [Tool Result] Sunny, 18°C                               │    │
     │    │  [Assistant] It's sunny and 18°C in Paris right now!     │    │
     │    │                                                          │    │
     │    └──────────────────────────────────────────────────────────┘    │
     │    ┌──────────────────────────────────────────────────────────┐    │
     │    │  > _                                                     │    │
     │    └──────────────────────────────────────────────────────────┘    │
     │                                                                    │
     │    Commands: /help  /profile <name>  /model <name>  /tools  /quit  │
     │                                                                    │
     └─────────────────────────────────┬──────────────────────────────────┘
                                       │
                                       ▼
    ┌───────────────────────────────────────────────────────────────────┐
    │                                                                   │
    │                         mcp-client                                │
    │                    (Orchestrator Library)                         │
    │                                                                   │
    │    ┌─────────────────────────────────────────────────────────┐    │
    │    │                      LLM Providers                      │    │
    │    │                                                         │    │
    │    │    OllamaProvider    OpenAIProvider    MockProvider     │    │
    │    │    (localhost:11434) (api.openai.com)  (for testing)    │    │
    │    │                                                         │    │
    │    └─────────────────────────────────────────────────────────┘    │
    │                                                                   │
    │    ┌─────────────────────────────────────────────────────────┐    │
    │    │                    Tool Management                      │    │
    │    │                                                         │    │
    │    │  1. Load profile → spawn tool servers                   │    │
    │    │  2. Collect tool definitions                            │    │
    │    │  3. Send tools to LLM                                   │    │
    │    │  4. Execute tool calls → return results                 │    │
    │    │                                                         │    │
    │    └─────────────────────────────────────────────────────────┘    │
    │                                                                   │
    └──────────────────────────────────┬────────────────────────────────┘
                                       │
         ┌───────────────────┼───────────────────┬───────────────────┐
         │                   │                   │                   │
         ▼                   ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│                 │ │                 │ │                 │ │                 │
│  tool-weather   │ │ tool-filesystem │ │   tool-system   │ │    tool-web     │
│                 │ │                 │ │                 │ │                 │
│  get_weather    │ │  read_file      │ │  get_ram_usage  │ │    http_get     │
│                 │ │  write_file     │ │  get_cpu_usage  │ │    http_post    │
│                 │ │  list_directory │ │  get_disk_usage │ │                 │
│                 │ │  search_files   │ │  get_processes  │ │                 │
│                 │ │                 │ │                 │ │                 │
│  (stdin/stdout) │ │  (stdin/stdout) │ │  (stdin/stdout) │ │   (stdin/out)   │
│                 │ │                 │ │                 │ │                 │
└─────────────────┘ └─────────────────┘ └─────────────────┘ └─────────────────┘
```

## Project Structure

```
rust-mcp-agent-kit/
├── Cargo.toml                          # Workspace configuration
├── crates/
│   ├── mcp-core/                        # Protocol library (JSON-RPC, Tool trait, Server)
│   ├── mcp-client/                      # Client library (LLM providers, orchestration)
│   ├── mcp-agent-cli/                   # Interactive TUI for AI chat
│   └── mcp-tools/                       # Tool implementations
│       ├── tool-weather/                # get_weather
│       ├── tool-filesystem/             # read_file, write_file, list_directory, search_files
│       ├── tool-system/                 # get_ram_usage, get_cpu_usage, get_disk_usage, get_processes
│       ├── tool-web/                    # http_get, http_post
│       └── tool-utilities/              # calculate, format_json, current_time
└── profiles/                            # Tool profiles (config files)
    ├── coding.toml                      # Tools for coding assistance
    ├── personal.toml                    # Personal assistant tools
    ├── devops.toml                      # DevOps tools
    └── data.toml                        # Data processing tools
```

## Crates

| Crate | Description | Status |
|-------|-------------|--------|
| `mcp-core` | Protocol library: JSON-RPC types, Tool/Command traits, Server infrastructure | Done |
| `mcp-client` | Client library: LLM providers (Ollama, OpenAI, Mock), tool orchestration | Done |
| `mcp-agent-cli` | Interactive TUI for AI chat with tools | Done |

## Tools

All tools are located under `crates/mcp-tools/`. Each tool crate is a standalone binary.

### tool-weather

| Tool | Description |
|------|-------------|
| `get_weather` | Get current weather for a city via wttr.in |

### tool-filesystem

| Tool | Description |
|------|-------------|
| `read_file` | Read file contents |
| `write_file` | Write content to file |
| `list_directory` | List directory contents |
| `search_files` | Search for files matching a regex pattern |

### tool-system

| Tool | Description |
|------|-------------|
| `get_ram_usage` | Get RAM usage statistics |
| `get_cpu_usage` | Get CPU usage per core |
| `get_disk_usage` | Get disk space usage |
| `get_processes` | List top processes by CPU usage |

### tool-web

| Tool | Description |
|------|-------------|
| `http_get` | Make HTTP GET request |
| `http_post` | Make HTTP POST request with JSON body |

### tool-utilities

| Tool | Description |
|------|-------------|
| `calculate` | Evaluate mathematical expressions (+, -, *, /, %, parentheses) |
| `format_json` | Parse and pretty-print JSON |
| `current_time` | Get current date and time |

## Profiles

Profiles define which tools are available to an agent. Each profile is a TOML config file.

| Profile | Description | Tools |
|---------|-------------|-------|
| `coding` | Coding assistance | filesystem, web, utilities |
| `personal` | Personal assistant | weather, utilities |
| `devops` | DevOps operations | system, web, utilities |
| `data` | Data processing | utilities, filesystem |

### Profile Structure

```toml
# profiles/coding.toml
[profile]
name = "coding"
description = "Tools for coding assistance"

[tools]
tool-filesystem = { enabled = true }
tool-web = { enabled = true }
tool-utilities = { enabled = true }
```

### How Profiles Work

```
User: /profile coding

mcp-client:
  1. Read profiles/coding.toml
  2. Identify enabled tools
  3. Spawn tool-filesystem, tool-web, tool-utilities processes
  4. Collect tool definitions from each
  5. Send tools to LLM
  6. LLM uses only available tools
```

### Behavior Change

| Profile | Tools Available | LLM Behavior |
|---------|-----------------|--------------|
| `coding` | read/write files, web | "I can edit code, fetch docs" |
| `personal` | weather, calculate, time | "I can answer questions, check weather" |
| `devops` | system stats, web | "I can monitor infrastructure" |

## mcp-agent-cli

Interactive terminal UI for AI chat with MCP tools.

### Features

- **Status bar**: Shows model, profile, tool count
- **Chat panel**: Messages with color-coded senders
- **Input bar**: Type messages or commands
- **Tool call display**: Shows tool calls and results inline

### Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/profile <name>` | Load profile (coding, personal, devops, data) |
| `/model <name>` | Switch LLM model |
| `/tools` | List loaded tools |
| `/clear` | Clear chat history |
| `/quit` | Exit CLI |

### Usage

```bash
# Build all
cargo build --release

# Run CLI
./target/release/mcp-agent-cli

# Inside CLI:
/profile coding
What files are in this directory?
> [Tool: list_directory] → [Result: Cargo.toml, src/, ...]

/model glm-4
/profile personal
What's the weather in Tokyo?
> [Tool: get_weather] → [Result: Rainy, 15°C]
```

## Key Design Decisions

### Tools in `mcp-tools/`, Profiles as Config

- **Tools** = Rust code implementing the `Tool` trait
- **Profiles** = TOML config files defining which tools to use
- **Separation** = Logic lives in tools, profiles are just configuration

### Profile-based Tool Selection

- Each profile enables specific tools
- LLM adapts behavior based on available tools
- User explicitly chooses profile: `/profile coding`
- Different profiles = different agent "modes"

### Microservices Architecture

Each tool crate:
- Is a **separate binary** (separate process)
- Communicates via stdin/stdout (MCP protocol)
- Has isolated environment
- Can be deployed independently

### LLM Provider Abstraction

```rust
pub trait ChatClient {
    fn chat(&self, messages: Vec<Message>, tools: Vec<ToolDefinition>) -> Result<ChatResponse, String>;
}

OllamaProvider::new("http://localhost:11434", "glm-5:cloud")
OpenAIProvider::new("sk-...", "gpt-4o")
MockProvider::with_tool_call("get_weather", r#"{"city": "Paris"}"#)
```

## Quick Start

```bash
# Clone repository
git clone https://github.com/estvv/rust-mcp-agent-kit.git
cd rust-mcp-agent-kit

# Build all crates
cargo build --release

# Run CLI
./target/release/mcp-agent-cli

# Inside CLI:
/profile coding
Hello, what can you do?
```

## Testing

```bash
# Test tools directly
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | target/release/tool-weather

# Call a tool
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_weather","arguments":{"city":"Paris"}}}' | target/release/tool-weather

# Run mock test (no LLM needed)
cargo run --example test_mock

# Run with Ollama (requires ollama serve + model)
cargo run --example test_multi_turn_real
```

## Using mcp-client Programmatically

```rust
use mcp_client::{Orchestrator, OllamaProvider, Profile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load profile
    let profile = Profile::load_by_name("coding")?;

    // Create provider and orchestrator
    let provider = OllamaProvider::new("http://localhost:11434", "glm-5:cloud");
    let mut orch = Orchestrator::new(provider);

    // Spawn tools from profile
    for tool in profile.enabled_tools() {
        orch.spawn_tool(tool, tool)?;
    }

    // Chat with automatic tool execution
    let response = orch.chat("What's the weather in Paris?")?;
    println!("{}", response);

    Ok(())
}
```

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

## Dependencies

| Crate | Usage |
|-------|-------|
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `inventory` | Compile-time tool registration |
| `minreq` | HTTP client (lightweight) |
| `ratatui` | TUI framework (mcp-agent-cli) |
| `crossterm` | Terminal control (mcp-agent-cli) |
| `sysinfo` | System information (tool-system) |
| `regex` | Pattern matching (tool-filesystem) |
| `chrono` | Date/time (tool-utilities) |

## Model Compatibility

Works with any LLM that supports function calling:

| Provider | Models |
|----------|--------|
| **Ollama** | GLM-4, GLM-5:cloud, Llama 3.1+, Mistral, Qwen |
| **OpenAI** | GPT-4, GPT-4o, GPT-3.5-turbo |
| **Anthropic** | Claude 3+ (via OpenAI-compatible endpoint) |
| **Mock** | For testing without LLM |

Uses OpenAI-compatible `/v1/chat/completions` endpoint.

## Development

```bash
# Build all
cargo build --workspace

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --check

# Lint
cargo clippy --workspace -- -D warnings
```

## Documentation

- [docs/ROADMAP.md](./docs/ROADMAP.md) - Development roadmap
- [docs/FEATURES.md](./docs/FEATURES.md) - Feature reference
- [profiles/README.md](./profiles/README.md) - Profile documentation

## License

MIT License

## References

- [MCP Specification](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Ollama OpenAI Compatibility](https://github.com/ollama/ollama/blob/main/docs/openai.md)
