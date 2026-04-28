# MCP Ecosystem Development Roadmap

## Project Overview

This document outlines the development plan for a **Rust-based MCP (Model Context Protocol) ecosystem** consisting of multiple interconnected projects that work together to provide local-first AI agent capabilities via Ollama.

---

## Repository Structure

### Current State

```
mcp/
├── rust-mcp-ecosystem/     ← Aggregator repo (EMPTY - needs setup)
├── rust-mcp-core/          ← Core library (EMPTY - needs creation)
├── rust-mcp-servers/       ← MCP server implementation (WORKING - needs refactoring)
├── rust-mcp-client/        ← MCP client for Ollama (EMPTY - needs creation)
└── rust-rag-cli/           ← RAG CLI (MATURE - needs MCP integration)
```

### Target Architecture

```
rust-mcp-ecosystem/              (Aggregator with submodules)
├── .gitmodules
├── Cargo.toml                   (workspace for local dev)
├── docs/
│   └── (documentation)
├── core/                        → rust-mcp-core
├── servers/                     → rust-mcp-servers
├── client/                      → rust-mcp-client
└── rag-cli/                     → rust-rag-cli
```

---

## Projects

### 1. rust-mcp-ecosystem (Aggregator)

**Status:** Empty skeleton  
**Purpose:** Meta-repository that aggregates all projects via git submodules

**What Needs to Be Done:**
- [ ] Initialize `.gitmodules` with submodule references
- [ ] Add each project as a submodule
- [ ] Create workspace `Cargo.toml` (optional, for local development)
- [ ] Write comprehensive README
- [ ] Create `/docs` with architecture diagrams

**Submodules to Configure:**
```ini
[submodule "core"]
    path = core
    url = https://github.com/estvv/rust-mcp-core.git

[submodule "servers"]
    path = servers
    url = https://github.com/estvv/rust-mcp-servers.git

[submodule "client"]
    path = client
    url = https://github.com/estvv/rust-mcp-client.git

[submodule "rag-cli"]
    path = rag-cli
    url = https://github.com/estvv/rust-rag-cli.git
```

---

### 2. rust-mcp-core (Shared Library)

**Status:** Empty skeleton  
**Purpose:** Provides shared traits, types, and infrastructure for all MCP components

**What Exists Now:**
- `.git` directory
- `LICENSE` (MIT)
- `README.md` (just project name)

**What Needs to Be Created:**
```
rust-mcp-core/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs              # McpError, ToolError with thiserror
│   ├── protocol/
│   │   ├── mod.rs
│   │   ├── request.rs        # RpcRequest, etc.
│   │   └── response.rs       # RpcResponse, etc.
│   ├── tool.rs               # Tool trait (async)
│   ├── command.rs            # Command trait (async)
│   ├── server/               # Server infrastructure
│   │   ├── mod.rs
│   │   ├── dispatcher.rs
│   │   └── registry.rs
│   └── client/               # Client infrastructure
│       ├── mod.rs
│       ├── connection.rs     # MCP server connection
│       └── transport.rs      # stdio transport
├── tests/
└── README.md
```

**Key Design Decisions:**
1. **Async traits** using `async-trait` crate
2. **Error types** using `thiserror` for structured errors
3. **Protocol types** match MCP/JSON-RPC 2.0 spec exactly
4. **Server infrastructure** extracted from rust-mcp-servers
5. **Client infrastructure** shared by rust-mcp-client and rust-rag-cli

---

### 3. rust-mcp-servers (MCP Servers)

**Status:** WORKING - needs refactoring  
**Current State:** Single monolithic binary `rust-mcp-agent` with 14 tools

**What Exists Now:**
- Complete MCP server implementation
- 14 tools (filesystem, system, web, utilities)
- 6 commands (initialize, ping, shutdown, initialized, tools/list, tools/call)
- 32 tests
- Working binary in `target/release/rust-mcp-agent`

**What Needs to Be Done:**
1. **Extract core to rust-mcp-core:**
   - Move `Tool` trait
   - Move `Command` trait
   - Move protocol types
   - Move dispatcher logic
   - Move error types

2. **Split into microservices:**
   ```
   rust-mcp-servers/           (workspace)
   ├── Cargo.toml             (workspace root)
   ├── crates/
   │   ├── mcp-filesystem/    (read_file, write_file, list_directory, search_files)
   │   ├── mcp-github/        (create_issue, get_issue, create_pr, search_code, etc.)
   │   ├── mcp-system/        (get_ram_usage, get_cpu_usage, get_disk_usage, get_processes)
   │   ├── mcp-web/           (http_get, http_post, http_put, http_delete)
   │   ├── mcp-docker/        (containers, images, compose)
   │   └── mcp-database/      (query, schema inspection)
   └── tests/
   ```

3. **Each server will:**
   - Depend on `mcp-core`
   - Be its own binary
   - Run as separate process
   - Communicate via MCP protocol (stdin/stdout)

**Current Files (to be refactored):**
```
src/
├── main.rs                  # Entry point
├── lib.rs                    # Module exports
├── server.rs                 # Event loop, request handling
├── dispatcher.rs             # Method routing
├── types.rs                  # Protocol types
├── constants.rs              # Version, name, etc.
├── commands/
│   ├── command.rs            # Command trait + inventory
│   ├── initialize.rs
│   ├── initialized.rs
│   ├── ping.rs
│   ├── shutdown.rs
│   ├── tools_list.rs
│   └── tools_call.rs
└── tools/
    ├── tool.rs               # Tool trait + inventory
    ├── filesystem.rs         # 4 tools
    ├── system.rs             # 4 tools
    ├── http.rs               # 2 tools
    ├── weather.rs            # 1 tool
    └── utilities.rs          # 3 tools
```

---

### 4. rust-mcp-client (MCP Client for Ollama)

**Status:** Empty skeleton  
**Purpose:** Bridges Ollama models to MCP servers, orchestrates tool calls

**What Exists Now:**
- `.git` directory
- `LICENSE` (MIT)
- `README.md` (just project name)

**What Needs to Be Created:**
```
rust-mcp-client/
├── Cargo.toml                # depends on mcp-core, reqwest
├── src/
│   ├── main.rs               # CLI entry point
│   ├── cli.rs                # CLI arguments (clap)
│   ├── config.rs              # Configuration (TOML, env)
│   ├── ollama/
│   │   ├── mod.rs
│   │   ├── client.rs         # Ollama HTTP client
│   │   └── models.rs         # Model types
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── orchestrator.rs   # Plan → Act → Observe loop
│   │   └── tool_parser.rs    # Parse tool calls from model response
│   └── mcp/
│       ├── mod.rs
│       ├── server_pool.rs    # Manage multiple MCP server connections
│       └── transport.rs      # stdio transport for servers
├── tests/
└── README.md
```

**Key Responsibilities:**
1. **Ollama API client:**
   - Connect to `localhost:11434`
   - Send chat requests with streaming
   - Handle embedding requests

2. **MCP connection manager:**
   - Spawn MCP server processes
   - Communicate via stdin/stdout
   - Tool discovery (`tools/list` on each server)
   - Tool execution (`tools/call`)

3. **Agent loop:**
   - Send user prompt + tool definitions to Ollama
   - Parse model response for tool calls
   - Execute tool calls on appropriate servers
   - Feed results back to model
   - Continue until model produces final response

4. **Exported as library:**
   - `McpClient` struct for use by `rust-rag-cli`
   - Configuration types
   - Error types

---

### 5. rust-rag-cli (RAG CLI)

**Status:** MATURE - working application  
**Purpose:** Local-first semantic code search and chat with Ollama

**What Exists Now:**
- Full TUI with ratatui
- Semantic search using embeddings
- RAG chat with context retrieval
- File references (`@path` syntax)
- Streaming responses
- File watching
- Index persistence
- Extensive documentation (487 line README)

**Key Modules:**
```
src/
├── main.rs                  # Event loop (769 lines)
├── cli.rs                    # CLI arguments
├── scrapper.rs               # File scanning, chunking
├── app/
│   ├── state.rs              # Application state
│   ├── command.rs           # /commands parsing
│   └── action.rs             # State reduction
├── service/
│   └── chat.rs               # RAG service (embed, retrieve, chat)
├── clients/
│   └── ollama.rs             # Ollama HTTP client
├── db/
│   ├── store.rs              # Semantic index
│   └── metadata.rs           # File metadata
├── ui/
│   ├── render.rs             # TUI rendering
│   └── syntax.rs             # Syntax highlighting
└── ... (other modules)
```

**What Needs to Be Added:**
1. **MCP client integration:**
   ```rust
   // In Cargo.toml
   [dependencies]
   mcp-client = { path = "../rust-mcp-client" }
   
   // In src/service/chat.rs
   pub struct ChatService {
       ollama_client: OllamaClient,
       index: SemanticIndex,
       mcp_client: Option<McpClient>,  // NEW
   }
   ```

2. **Tool calling during chat:**
   - RAG retrieves context
   - MCP client enables tool calls
   - Model can call `read_file`, `create_issue`, etc.
   - Results integrated into response

3. **Configuration:**
   ```toml
   # ~/.config/rag-cli/config.toml
   [mcp]
   enabled = true
   
   [mcp.servers.filesystem]
   command = "mcp-filesystem"
   
   [mcp.servers.github]
   command = "mcp-github"
   env = { GITHUB_TOKEN = "ghp_xxx" }
   ```

---

## Architecture Overview

### Data Flow

```
User
  │
  ▼
┌──────────────────┐
│  rust-rag-cli    │  (TUI, RAG, Chat interface)
│                  │
│ ┌──────────────┐ │
│ │ RAG Engine   │ │  Embeddings + Retrieval
│ └──────────────┘ │
│                  │
│ ┌──────────────┐ │
│ │ MCP Client   │ │  Tool orchestration
│ └──────────────┘ │
└────────┬─────────┘
         │
    ┌────┴────┬────────────┬────────────┐
    │         │            │            │
    ▼         ▼            ▼            ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│Ollama  │ │mcp-fs  │ │mcp-gh  │ │mcp-sys │
│API     │ │server  │ │server  │ │server  │
└────────┘ └────────┘ └────────┘ └────────┘
```

### Query Flow Example

```
User: "What's in my README? Also create a GitHub issue about the auth bug."

1. RAG CLI receives input
2. RAG retrieves context (semantic search)
3. Loads @file references if any
4. Builds augmented prompt
   
5. MCP Client sends to Ollama:
   {
     "model": "llama3",
     "messages": [...],
     "tools": [all tools from all servers]
   }
   
6. Ollama responds:
   "I'll read the README and then create an issue."
   <tool_call name="read_file" args={path: "README.md"}>
   
7. MCP Client parses tool call
8. Routes to mcp-filesystem server
9. Server executes read_file
10. Returns README content to MCP Client
11. MCP Client sends back to Ollama:
    {
      "role": "tool",
      "content": "README content..."
    }
    
12. Ollama responds:
    <tool_call name="create_issue" args={title: "...", body: "..."}>
    
13. MCP Client routes to mcp-github server
14. GitHub server creates issue
15. Returns issue URL
16. MCP Client sends to Ollama
17. Ollama generates final response
18. RAG CLI displays in TUI
```

---

## Implementation Phases

### Phase 1: Core Library (Week 1-2)

**Goal:** Create `rust-mcp-core` with shared infrastructure

**Tasks:**
- [ ] Create `Cargo.toml` with dependencies (tokio, serde, thiserror, async-trait)
- [ ] Implement error types (`McpError`, `ToolError`)
- [ ] Implement protocol types (extract from `rust-mcp-servers/src/types.rs`)
- [ ] Create async `Tool` trait
- [ ] Create async `Command` trait
- [ ] Implement server dispatcher (extract from `rust-mcp-servers/src/dispatcher.rs`)
- [ ] Write unit tests
- [ ] Publish documentation

**Deliverable:** `mcp-core` crate that can be used by all other projects

---

### Phase 2: Refactor Servers (Week 3-4)

**Goal:** Split `rust-mcp-agent` into microservices using `mcp-core`

**Tasks:**
- [ ] Create workspace `Cargo.toml` for multiple servers
- [ ] Create `mcp-filesystem` crate (extract filesystem tools)
- [ ] Create `mcp-system` crate (extract system tools)
- [ ] Create `mcp-web` crate (extract HTTP tools)
- [ ] Refactor to use `mcp-core`
- [ ] Update all servers to async
- [ ] Port tests to each crate
- [ ] Test servers individually

**Deliverable:** 3 working MCP servers using `mcp-core`

---

### Phase 3: MCP Client (Week 5-6)

**Goal:** Create `rust-mcp-client` that bridges Ollama to MCP

**Tasks:**
- [ ] Create `Cargo.toml` (depends on `mcp-core`, `reqwest`, `tokio`)
- [ ] Implement Ollama HTTP client
- [ ] Implement MCP server connector (spawn processes, stdio communication)
- [ ] Implement tool discovery (call `tools/list` on all servers)
- [ ] Implement tool execution (call `tools/call`)
- [ ] Implement agent loop (detect tool calls, execute, return)
- [ ] Create CLI interface
- [ ] Test with mock servers
- [ ] Test with real servers

**Deliverable:** `mcp-client` binary + library crate

---

### Phase 4: RAG Integration (Week 7-8)

**Goal:** Integrate `rust-rag-cli` with `mcp-client`

**Tasks:**
- [ ] Add `mcp-client` dependency to `rust-rag-cli`
- [ ] Add MCP configuration support
- [ ] Integrate MCP client into `ChatService`
- [ ] Update chat flow to support tool calling
- [ ] Add configuration examples
- [ ] Test end-to-end
- [ ] Update documentation

**Deliverable:** RAG CLI with MCP tool support

---

### Phase 5: New Servers (Week 9-10)

**Goal:** Add GitHub, Docker, Database servers

**Tasks:**
- [ ] Create `mcp-github` crate
  - [ ] GitHub HTTP client (reqwest)
  - [ ] Tools: create_issue, get_issue, create_pr, list_repos, search_code
  - [ ] Environment variable for GITHUB_TOKEN
- [ ] Create `mcp-docker` crate
  - [ ] Docker socket client (bollard)
  - [ ] Tools: containers, images, compose
- [ ] Create `mcp-database` crate
  - [ ] SQL client (sqlx)
  - [ ] Tools: query, schema inspection
- [ ] Tests for all new servers

**Deliverable:** 3 new specialized servers

---

### Phase 6: Ecosystem Setup (Week 11-12)

**Goal:** Setup aggregator repo and polish

**Tasks:**
- [ ] Initialize `.gitmodules` in aggregator
- [ ] Add all projects as submodules
- [ ] Create workspace `Cargo.toml` for local dev
- [ ] Create comprehensive README for ecosystem
- [ ] Create architecture documentation
- [ ] Create getting started guide
- [ ] Create usage examples
- [ ] Set up CI/CD (GitHub Actions)

**Deliverable:** Complete ecosystem ready for use

---

## Technical Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (edition 2024) |
| Async Runtime | Tokio |
| HTTP Client | reqwest |
| Serialization | serde, serde_json |
| Errors | thiserror |
| Async Traits | async-trait |
| CLI | clap |
| Configuration | toml |
| Logging | tracing, tracing-subscriber |
| TUI (RAG CLI) | ratatui, crossterm |
| System Info | sysinfo |
| Docker | bollard |
| Database | sqlx |
| Regex | regex |
| Date/Time | chrono |

---

## Key Design Patterns

### 1. Inventory-based Registration

```rust
// Tools register themselves at compile time
pub struct ToolEntry {
    pub tool: &'static (dyn Tool + Send + Sync),
}

inventory::collect!(ToolEntry);

// In tool implementation:
inventory::submit! {
    ToolEntry { tool: &ReadFileTool }
}
```

### 2. Async Trait Pattern

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, arguments: Value) -> Result<String, ToolError>;
}
```

### 3. Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 4. Server Process Architecture

Each MCP server:
- Runs as separate process
- Communicates via stdin/stdout (MCP protocol)
- Has isolated environment (secrets, config)
- Can be deployed independently
- Can fail without affecting others

---

## Configuration

### MCP Client Config

```toml
# ~/.config/mcp-client/config.toml
[ollama]
base_url = "http://localhost:11434"
model = "llama3"
embed_model = "nomic-embed-text"

[servers.filesystem]
command = "mcp-filesystem"
args = []

[servers.github]
command = "mcp-github"
env = { GITHUB_TOKEN = "ghp_xxx" }

[servers.system]
command = "mcp-system"

[servers.docker]
command = "mcp-docker"
```

### RAG CLI Config (enhanced)

```toml
# ~/.config/rag-cli/config.toml
[settings]
chat_model = "llama3"
embed_model = "nomic-embed-text"
base_url = "http://localhost:11434"

[mcp]
enabled = true

[mcp.servers.filesystem]
command = "mcp-filesystem"

[mcp.servers.github]
command = "mcp-github"
env = { GITHUB_TOKEN = "${GITHUB_TOKEN}" }
```

---

## Security Considerations

1. **Secrets isolation:** Each server process has its own environment
2. **File system bounds:** Filesystem server can be restricted to certain paths
3. **Token management:** API tokens stored in environment, not logged
4. **Input validation:** All tool inputs validated against schema
5. **Error handling:** Errors don't leak sensitive information

---

## Testing Strategy

1. **Unit tests:** For core library and each server
2. **Integration tests:** Client → server communication
3. **End-to-end tests:** RAG CLI → MCP client → servers → Ollama
4. **Mock servers:** For testing client logic without full server stack

---

## Success Metrics

| Goal | Metric |
|------|--------|
| CV Value | Complete ecosystem showing Rust + async + architecture |
| Personal Use | Daily use with Ollama for development tasks |
| Startup Value | Production-grade code, extensible, documented |
| Open Source | Contributable to MCP ecosystem (Rust gap) |

---

## Quick Start (Target State)

```bash
# Clone ecosystem
git clone --recursive https://github.com/estvv/rust-mcp-ecosystem.git
cd rust-mcp-ecosystem

# Build all components
cargo build --release

# Start servers (in background)
mcp-filesystem &
mcp-github &
mcp-system &

# Start RAG CLI
rust-rag-cli chat --path ./my-project

# Or use MCP client directly
mcp-client chat --model llama3
```

---

## Current Blocking Issues

1. **rust-mcp-core:** Needs to be created from scratch
2. **rust-mcp-client:** Needs to be created from scratch
3. **rust-mcp-servers:** Needs to be split into microservices
4. **Aggregator:** Needs submodules and workspace setup
5. **Ollama tool calling:** Need to research format (function calling vs. MCP format)

---

## Next Steps (Immediate)

1. **Phase 1: Create rust-mcp-core**
   - Extract traits and types from `rust-mcp-servers`
   - Set up Cargo.toml
   - Write basic tests

2. **Verify Ollama tool calling format**
   - Test with llama3, glm-4, kimi
   - Document parsing requirements

3. **Refactor rust-mcp-servers**
   - Create workspace structure
   - Split into separate crates

---

## References

- [MCP Specification](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [rust-rag-cli README](../rust-rag-cli/README.md)
- [rust-mcp-servers Implementation](../rust-mcp-servers/)

---

## Document History

- **Created:** April 28, 2026
- **Author:** estv
- **Version:** 1.0

---

This document serves as the master roadmap for the entire ecosystem. As each phase completes, update the checkboxes and document any deviations from the plan.
