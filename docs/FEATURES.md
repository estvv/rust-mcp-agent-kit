# FEATURES

## Core (`mcp-core`)

The foundational library shared across all crates.

### Protocol & Messages

- [x] JSON-RPC 2.0 - Full request/response/notification types
- [x] MCP initialize handshake
- [x] tools/list - Discover available tools
- [x] tools/call - Execute tool with arguments
- [x] commands/list - Discover available commands
- [x] commands/execute - Execute command with arguments
- [x] Error Handling - Structured error codes and messages
- [x] Serialization - `serde` for JSON encode/decode

### Server Infrastructure

- [x] `Server` - Main server struct, reads stdin, writes stdout
- [x] `Dispatcher` - Routes methods to handlers
- [x] `ToolRegistry` - Stores and looks up tools by name
- [x] `CommandRegistry` - Stores and looks up commands by name
- [ ] Config file support - Load server config from TOML/JSON
- [ ] Logging - Structured logging with `tracing` crate

### Tool System

- [x] `Tool` trait - Interface for all tools (`name`, `description`, `input_schema`, `execute`)
- [x] `register_tool!` macro - Compile-time registration
- [x] `inventory` - Automatic tool discovery from `ToolEntry` submissions
- [x] Manual registration - Fallback for musl/WASM targets
- [ ] Tool validation - Validate input against JSON schema
- [ ] Tool permissions - Restrict tools by capability level

### Type Definitions

- [x] `RequestId` - JSON-RPC request identifier (number or string)
- [x] `InitializeParams` / `InitializeResult` - Handshake types
- [x] `ToolDefinition` - Tool name, description, input schema
- [x] `ToolCallParams` / `ToolCallResult` - Tool execution types
- [x] `CommandDefinition` - Command name, description
- [x] `CommandExecuteParams` - Command execution types
- [x] Error types - `ParseError`, `MethodNotFoundError`, `InvalidParamsError`, `InternalError`, `ToolError`

### Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `server` | Yes | Server-side infrastructure (`Server`, `Dispatcher`, registries) |
| `client` | Yes | Client-side infrastructure (JSON-RPC client, transport) |
| `inventory` | Yes | Compile-time tool/command registration (excludes musl/WASM) |

---

## Client (`mcp-client`)

Orchestrates Ollama and MCP servers for AI-powered tool calling.

### Ollama Integration

- [x] OpenAI-compatible API - Uses `/v1/chat/completions` endpoint
- [x] Structured tool calling - Returns `tool_calls` array (not text to parse)
- [ ] Streaming responses - Server-sent events for real-time output
- [x] Multi-turn conversations - Maintains message history
- [x] Model support - GLM-4, Llama 3.1+, Mistral, Qwen

### Server Pool

- [x] `ServerPool` - Manages multiple MCP server processes
- [x] Server spawning - `tokio::process::Command` with `Stdio::piped()`
- [x] Lifecycle management - Start, stop, restart servers
- [x] Auto-initialize - Sends `initialize` request on spawn
- [x] Tool discovery - Calls `tools/list` on each server
- [ ] Health monitoring - Detect and restart crashed servers
- [ ] Resource limits - Limit CPU/memory per server process

### Orchestrator

- [x] `Orchestrator` - Main struct combining Ollama client + server pool
- [x] `chat()` - Send message, auto-invoke tools, return response
- [x] Tool aggregation - Collects all tools from all servers
- [ ] Context limit - Warns when tool schemas exceed threshold
- [ ] Dynamic selection - Choose relevant tools (when 10+ tools exist)
- [ ] Tool result caching - Cache repeated tool calls
- [ ] Conversation export - Save/load conversation history

### Configuration

- [x] TOML config file support
- [x] Server definitions with command, args, env
- [ ] Hot reload - Reload config without restart
- [ ] Environment variable interpolation

---

## Servers

All MCP servers are located under `crates/mcp-servers/`. Each server is a standalone binary exposing related tools.

### mcp-weather

- [x] `get_weather` - Current weather for a city via `wttr.in`

### mcp-filesystem

- [x] `read_file` - Read file contents
- [x] `write_file` - Write content to file
- [x] `list_directory` - List directory contents
- [x] `search_files` - Regex search for files
- [ ] `create_directory` - Create directory recursively
- [ ] `delete_file` - Delete a file
- [ ] `move_file` - Move/rename a file
- [ ] `copy_file` - Copy a file
- [ ] `get_file_info` - Get file metadata (size, modified time)

### mcp-system

- [x] `get_ram_usage` - RAM usage stats via `sysinfo`
- [x] `get_cpu_usage` - CPU usage per core
- [x] `get_disk_usage` - Disk space usage
- [x] `get_processes` - Top processes by CPU
- [ ] `get_network_stats` - Network I/O statistics
- [ ] `kill_process` - Terminate a process by PID
- [ ] `get_environment` - Environment variables

### mcp-web

- [x] `http_get` - GET request
- [x] `http_post` - POST request with JSON body
- [ ] `http_put` - PUT request
- [ ] `http_delete` - DELETE request
- [ ] `http_request` - Generic HTTP request with custom headers
- [ ] Rate limiting - Prevent abuse
- [ ] Timeout configuration

### mcp-github

- [ ] `list_repos` - List user repositories
- [ ] `get_repo` - Get repository info
- [ ] `list_issues` - List repository issues
- [ ] `create_issue` - Create new issue
- [ ] `search_code` - Search code in repo
- [ ] `list_pull_requests` - List PRs
- [ ] `create_pull_request` - Create new PR
- [ ] `get_file_contents` - Get file from repo

**Auth**: Requires `GITHUB_TOKEN` environment variable

### mcp-docker

- [ ] `list_containers` - List all containers
- [ ] `get_container` - Get container details
- [ ] `container_logs` - Get container logs
- [ ] `exec_container` - Execute command in container
- [ ] `start_container` - Start a stopped container
- [ ] `stop_container` - Stop a running container
- [ ] `list_images` - List Docker images
- [ ] `build_image` - Build image from Dockerfile

### mcp-database

- [ ] `query` - Execute SQL query
- [ ] `list_tables` - List all tables
- [ ] `describe_table` - Get table schema
- [ ] Connection pooling - Reuse database connections
- [ ] Transaction support - BEGIN/COMMIT/ROLLBACK
- [ ] Multiple backends - SQLite, PostgreSQL, MySQL

---

## rag-cli

Terminal-based RAG application with MCP integration.

> **Note**: `rag-cli` is a git submodule. Clone with `--recursive` or run `git submodule update --init --recursive` after cloning.

### Core Features

- [x] TUI - Terminal UI with `ratatui` + `crossterm`
- [x] Semantic search - Vector similarity search over codebase
- [x] Streaming chat - Real-time AI responses
- [x] MCP integration - Tool calling via `mcp-client`
- [x] Embeddings - Local embeddings via Ollama
- [x] Session history - Persist conversation across runs

### Commands

| Command | Description |
|---------|-------------|
| `rag-cli chat --path ./project` | Start interactive chat session |
| `rag-cli search --path ./project "query"` | Semantic search |
| `rag-cli index --path ./project` | Build/update embeddings index |

### Planned Features

- [ ] Multi-project support - Index multiple codebases
- [ ] File watching - Auto-reindex on file changes
- [ ] Export conversations - Markdown/JSON export
- [ ] Custom prompts - User-defined system prompts
- [ ] Keyboard shortcuts - Configurable keybindings
- [ ] Theme support - Custom color schemes

---

## Comparison: inventory vs Manual Registration

| Aspect | `inventory` | Manual |
|--------|-------------|--------|
| Registration | Automatic (`register_tool!` macro) | Explicit (`registry.register()`) |
| Boilerplate | Minimal | More code |
| Compile-time check | Yes | No |
| musl/WASM support | No | Yes |
| Debugging | Harder (linker magic) | Easier (normal Rust) |

**Recommendation**: Use `inventory` by default. Fall back to manual registration when targeting Alpine Linux, musl, or WebAssembly.