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

Orchestrates LLMs and MCP servers for AI-powered tool calling.

### LLM Integration

- [x] OpenAI-compatible API - Uses `/v1/chat/completions` endpoint
- [x] Structured tool calling - Returns `tool_calls` array (not text to parse)
- [ ] Streaming responses - Server-sent events for real-time output
- [x] Multi-turn conversations - Maintains message history
- [x] Model support - GLM-4, GLM-5:cloud, Llama 3.1+, Mistral, Qwen

### Providers

| Provider | Status | Description |
|----------|--------|-------------|
| `OllamaProvider` | Done | Local LLM via Ollama (localhost:11434) |
| `MockProvider` | Done | For testing without LLM |
| `OpenAIProvider` | Planned | OpenAI API (api.openai.com) |

### Orchestrator

- [x] `Orchestrator` - Main struct combining LLM client + tool servers
- [x] `chat()` - Send message, auto-invoke tools, return response
- [x] Tool aggregation - Collects all tools from all servers
- [x] Tool execution loop - Automatic tool call → result → continue
- [ ] Context limit - Warns when tool schemas exceed threshold
- [ ] Dynamic selection - Choose relevant tools (when 10+ tools exist)
- [ ] Tool result caching - Cache repeated tool calls
- [ ] Conversation export - Save/load conversation history

### Profile System

- [x] TOML config file support
- [x] `Profile::load(path)` - Load from file path
- [x] `Profile::load_by_name(name)` - Load from profiles/{name}.toml
- [x] `enabled_tools()` - Get list of enabled tools
- [ ] Hot reload - Reload profile without restart

---

## CLI (`mcp-agent-cli`)

Interactive terminal UI for AI chat with MCP tools.

### TUI Features

- [x] Status bar - Model, profile, tool count
- [x] Chat panel - Messages with color-coded senders
- [x] Input bar - Type messages or commands
- [x] Keyboard input - Enter, Backspace, Char
- [ ] Mouse support - Click to focus panels
- [ ] Scroll - Navigate chat history
- [ ] Resize - Handle terminal resize

### Commands

| Command | Status | Description |
|---------|--------|-------------|
| `/help` | Done | Show available commands |
| `/profile <name>` | Done | Load profile (coding, personal, devops, data) |
| `/model <name>` | Done | Switch LLM model |
| `/tools` | Done | List loaded tools |
| `/clear` | Done | Clear chat history |
| `/quit` | Done | Exit CLI |

### Future Features

- [ ] Streaming responses - Real-time AI output
- [ ] Tool call panel - Dedicated panel for tool execution
- [ ] Syntax highlighting - Code blocks in messages
- [ ] Conversation export - Save chat to file
- [ ] Theme support - Custom color schemes

---

## Tools

All MCP tools are located under `crates/mcp-tools/`. Each tool crate is a standalone binary.

### tool-weather

| Tool | Status | Description |
|------|--------|-------------|
| `get_weather` | Done | Get current weather for a city via `wttr.in` |

### tool-filesystem

| Tool | Status | Description |
|------|--------|-------------|
| `read_file` | Done | Read file contents |
| `write_file` | Done | Write content to file |
| `list_directory` | Done | List directory contents |
| `search_files` | Done | Search for files matching a regex pattern |
| `create_directory` | Planned | Create directory recursively |
| `delete_file` | Planned | Delete a file |
| `move_file` | Planned | Move/rename a file |
| `copy_file` | Planned | Copy a file |
| `get_file_info` | Planned | Get file metadata (size, modified time) |

### tool-system

| Tool | Status | Description |
|------|--------|-------------|
| `get_ram_usage` | Done | RAM usage stats via `sysinfo` |
| `get_cpu_usage` | Done | CPU usage per core |
| `get_disk_usage` | Done | Disk space usage |
| `get_processes` | Done | Top processes by CPU |
| `get_network_stats` | Planned | Network I/O statistics |
| `kill_process` | Planned | Terminate a process by PID |
| `get_environment` | Planned | Environment variables |

### tool-web

| Tool | Status | Description |
|------|--------|-------------|
| `http_get` | Done | GET request |
| `http_post` | Done | POST request with JSON body |
| `http_put` | Planned | PUT request |
| `http_delete` | Planned | DELETE request |
| `http_request` | Planned | Generic HTTP request with custom headers |
| Rate limiting | Planned | Prevent abuse |
| Timeout configuration | Planned | Configurable timeouts |

### tool-utilities

| Tool | Status | Description |
|------|--------|-------------|
| `calculate` | Done | Evaluate mathematical expressions |
| `format_json` | Done | Parse and pretty-print JSON |
| `current_time` | Done | Get current date and time |

### Planned Tools

| Crate | Tools |
|-------|-------|
| `tool-github` | list_repos, get_repo, list_issues, create_issue, search_code, list_pull_requests, create_pull_request, get_file_contents |
| `tool-docker` | list_containers, get_container, container_logs, exec_container, start_container, stop_container, list_images, build_image |
| `tool-database` | query, list_tables, describe_table |

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
