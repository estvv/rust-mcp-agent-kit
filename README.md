# rust-mcp-ecosystem

A Rust-based MCP (Model Context Protocol) ecosystem for local-first AI agents with Ollama.

This is a **monorepo** containing multiple MCP servers, a client library, and a RAG CLI application.

## Architecture

```
┌──────────────────────────────────────┐
│              rag-cli                 │
│      (TUI + RAG + Chat Interface)    │
│                 │                    │
│        ┌────────┴────────┐           │
│        │                 │           │
│        ▼                 ▼           │
│   ┌──────────┐    ┌──────────────┐   │
│   │  Ollama  │    │  mcp-client  │   │
│   │   API    │    │  (library)   │   │
│   └──────────┘    └──────┬───────┘   │
│                          │           │
└──────────────────────────┼───────────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
            ▼              ▼              ▼
     ┌─────────────┐ ┌──────────────┐ ┌─────────────┐
     │ mcp-weather │ │mcp-filesystem│ │ mcp-github  │
     │   server    │ │   server     │ │   server    │
     └─────────────┘ └──────────────┘ └─────────────┘
```

**Key Points:**
- Each MCP server is a **separate process** (not a library)
- `mcp-core` is linked into each server binary (not shown in diagram)
- `mcp-client` spawns servers via `tokio::process::Command` with `Stdio::piped()`
- `rag-cli` uses `mcp-client` internally for tool orchestration

## Project Structure

```
rust-mcp-ecosystem/
├── Cargo.toml                  # Workspace configuration
├── crates/
│   ├── mcp-core/               # Core library (protocol, traits, infrastructure)
│   ├── mcp-client/             # Client library for Ollama + MCP orchestration
│   └── mcp-servers/            # MCP server crates
│       ├── mcp-weather/        # Weather MCP server
│       └── ...                 # Other MCP servers
├── rag-cli/                    # RAG CLI application (git submodule)
├── tests/                      # Integration tests
├── docs/                       # Documentation
└── examples/                   # Example configurations
```

## Crates

| Crate | Description | Status |
|-------|-------------|--------|
| `mcp-core` | Shared library: protocol types, Tool/Command traits, server/client infrastructure | In Progress |
| `mcp-client` | Ollama client library with MCP server orchestration | In Progress |
| `rag-cli` | RAG CLI with MCP integration (git submodule) | Mature |

## MCP Servers

All servers are located under `crates/mcp-servers/`.

| Crate | Description | Status |
|-------|-------------|--------|
| `mcp-weather` | Weather information via wttr.in | In Progress |


## Modules

### mcp-core

| Module | Description |
|--------|-------------|
| `protocol` | JSON-RPC 2.0 types, MCP messages |
| `server` | `Server`, `Dispatcher`, `ToolRegistry`, `CommandRegistry` |
| `tool` | `Tool` trait, `register_tool!` macro, `ToolEntry` |
| `error` | Error types (`ParseError`, `ToolError`, etc.) |

### mcp-client

| Module | Description |
|--------|-------------|
| `ollama` | Ollama API client (`/v1/chat/completions`) |
| `pool` | `ServerPool` for managing MCP server processes |
| `orchestrator` | `Orchestrator` for chat + tool calling loop |

### rag-cli

| Module | Description |
|--------|-------------|
| `tui` | Terminal UI with `ratatui` + `crossterm` |
| `rag` | RAG implementation, embeddings, vector search |
| `mcp` | MCP client integration |

## Key Design Decisions

### Monorepo Architecture

- All code in one repository
- Each crate is independently versioned
- Each crate can be published separately to crates.io
- Single `git clone` to get everything
- Cross-crate refactoring is atomic

### Git Submodule

`rag-cli` is a git submodule (the only one in this repo):
- Maintains separate git history
- Can be developed independently
- Clone with: `git clone --recursive <repo-url>`
- Or after clone: `git submodule update --init --recursive`

### Microservices Architecture

Each MCP server:
- Runs as a **separate process**
- Communicates via stdin/stdout (MCP protocol)
- Has isolated environment (secrets, config)
- Can be deployed independently

### Ollama Integration

The `mcp-client` uses Ollama's **OpenAI-compatible API** (`/v1/chat/completions`):
- Structured tool calls (not text parsing)
- Works with GLM-4, Llama 3.1+, Mistral, and other function-calling models
- Standardized request/response format

### Server Lifecycle

The client spawns MCP servers as child processes:

1. Client spawns `mcp-weather` process
2. Client pipes stdin/stdout
3. Client sends `initialize` request
4. Client discovers tools via `tools/list`
5. Client calls tools via `tools/call`
6. Client manages process lifecycle

**Do NOT run servers manually** (e.g., `mcp-weather &`). The client manages them.

## Quick Start

```bash
# Clone repository (including submodule)
git clone --recursive https://github.com/estvv/rust-mcp-ecosystem.git
cd rust-mcp-ecosystem

# Build all crates
cargo build --release

# Run a specific server (for testing)
cargo run -p mcp-weather

# Run RAG CLI
cargo run -p rag-cli -- chat --path ./your-project
```

## Examples

### Using mcp-client Programmatically

```rust
use mcp_client::{Orchestrator, OllamaClient, ServerPool, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server pool and start MCP servers
    let mut servers = ServerPool::new();
    servers.start_server("weather", ServerConfig::new("mcp-weather")).await?;
    servers.start_server("filesystem", ServerConfig::new("mcp-filesystem")).await?;

    // Create Ollama client
    let ollama = OllamaClient::new("http://localhost:11434", "glm4");

    // Create orchestrator
    let mut orchestrator = Orchestrator::new(ollama, servers);

    // Chat with automatic tool calling
    let response = orchestrator.chat("What's the weather in Paris?").await?;
    println!("{}", response);

    Ok(())
}
```

### Tool Registration (inventory)

```rust
use mcp_core::{Tool, ToolError, ToolEntry, register_tool};
use serde_json::Value;

pub struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &'static str { "get_weather" }
    fn description(&self) -> &'static str { "Get weather for a city" }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": { "type": "string", "description": "City name" }
            },
            "required": ["city"]
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let city = args["city"].as_str()
            .ok_or_else(|| ToolError::MissingArgument("city".into()))?;
        // Fetch weather from wttr.in
        let url = format!("https://wttr.in/{}?format=3", city);
        let response = reqwest::blocking::get(&url)?;
        Ok(response.text()?)
    }
}

register_tool!(WeatherTool);
```

### Manual Registration (musl/WASM fallback)

```rust
use mcp_core::{ToolRegistry, Tool, Server};

fn main() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(WeatherTool));

    let server = Server::new(registry);
    server.run();
}
```

## Configuration

### RAG CLI with MCP

```toml
# ~/.config/rag-cli/config.toml
[settings]
chat_model = "glm4"
embed_model = "nomic-embed-text"
base_url = "http://localhost:11434"

[mcp]
enabled = true

[mcp.servers.weather]
command = "mcp-weather"

[mcp.servers.filesystem]
command = "mcp-filesystem"

[mcp.servers.github]
command = "mcp-github"
env = { GITHUB_TOKEN = "${GITHUB_TOKEN}" }
```

### Server Configuration

```toml
[servers.weather]
command = "mcp-weather"
args = []
env = {}

[servers.filesystem]
command = "mcp-filesystem"
args = ["--root", "/home/user"]
env = {}

[servers.github]
command = "mcp-github"
env = { GITHUB_TOKEN = "${GITHUB_TOKEN}" }
```

## Dependencies

| Crate | Usage |
|-------|-------|
| `tokio` | Async runtime, process spawning, sync primitives |
| `serde` / `serde_json` | Serialization (JSON-RPC, config files) |
| `thiserror` | Error types |
| `async-trait` | Async trait definitions |
| `inventory` | Compile-time tool registration |
| `tracing` | Structured logging |
| `reqwest` | HTTP client (Ollama API, web tools) |
| `sysinfo` | System information (mcp-system) |
| `ratatui` / `crossterm` | Terminal UI (rag-cli) |

## Model Compatibility

Tested with Ollama models that support function calling:
- **GLM-4** (primary target)
- Llama 3.1+
- Mistral
- Qwen

Uses OpenAI-compatible `/v1/chat/completions` endpoint.

## Chat Flow

```
User
  │
  ▼
Orchestrator.chat("What's the weather in Paris?")
  │
  ├─► Collect tools from all MCP servers
  │
  ├─► POST /v1/chat/completions to Ollama
  │     {
  │       "model": "glm4",
  │       "messages": [...],
  │       "tools": [{ "type": "function", "function": { "name": "get_weather", ... } }]
  │     }
  │
  ├─► Response contains tool_calls
  │     {
  │       "choices": [{
  │         "message": {
  │           "tool_calls": [{ "function": { "name": "get_weather", "arguments": "{\"city\": \"Paris\"}" } }]
  │         }
  │       }]
  │     }
  │
  ├─► Execute tool via MCP server
  │     tools/call { "name": "get_weather", "arguments": { "city": "Paris" } }
  │     → "Paris: 15C, partly cloudy"
  │
  ├─► Append tool result to messages
  │
  ├─► POST /v1/chat/completions again (with tool results)
  │
  └─► Final response: "The weather in Paris is 15C and partly cloudy."
```

## Documentation

- [ROADMAP.md](./ROADMAP.md) - Development roadmap
- [docs/FEATURES.md](./docs/FEATURES.md) - Feature reference by crate
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) - Architecture details (coming soon)
- [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) - Setup guide (coming soon)

## Development

```bash
# Run tests
cargo test --workspace

# Check formatting
cargo fmt --check

# Lint
cargo clippy --workspace -- -D warnings

# Build all
cargo build --workspace
```

## License

MIT License

## References

- [MCP Specification](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Ollama OpenAI Compatibility](https://github.com/ollama/ollama/blob/main/docs/openai.md)
