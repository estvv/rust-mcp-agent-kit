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

### Skill System

- [x] TOML config file support
- [x] `Skill::load(path)` - Load from file path
- [x] `Skill::load_by_name(name)` - Load from skills/{name}.toml
- [x] `enabled_tools()` - Get list of enabled tools
- [ ] Hot reload - Reload skill without restart

---

## CLI (`mcp-agent-cli`)

Interactive terminal UI for AI chat with MCP tools. See [TUI.md](./TUI.md) for complete UI specification.

### Layout

| Section | Status | Description |
|---------|--------|-------------|
| Title bar | Done | App name + mode indicator (BUILD/PLAN) |
| Status bar | Done | Model, profile, tool count, status |
| Chat area | Done | Messages with role-based backgrounds |
| Context sidebar | Planned | Files context, model info, tokens |
| Input bar | Done | Mode indicator + input field |

### Split View Layout

| Feature | Status | Description |
|---------|--------|-------------|
| Right sidebar (30 cols) | Planned | Always visible context panel |
| Chat area (flexible) | Done | Main conversation area |
| Responsive split | Planned | Handle terminal resize |

### Title Bar

| Feature | Status | Description |
|---------|--------|-------------|
| App name | Done | "MCP Agent" |
| Mode indicator | Done | BUILD (green) / PLAN (yellow) |

### Status Bar

| Field | Status | Description |
|-------|--------|-------------|
| Model name | Done | Current LLM model |
| Skill name | Done | Current skill |
| Tool count | Done | Number of loaded tools |
| Mode indicator | Planned | Build/Plan mode display |
| Status message | Done | Ready, Thinking..., Error |

### Chat Area

| Feature | Status | Description |
|---------|--------|-------------|
| Message display | Done | User/Assistant/System/Tool messages |
| Role-based backgrounds | Planned | Dark tint per sender role |
| Role-based text colors | Done | Green/Cyan/Yellow/Magenta |
| Message separators | Planned | Light lines between messages |
| Markdown rendering | Done | Headers, bullets, inline code |
| Syntax highlighting | Planned | VS Code-style theme for code blocks |
| Scroll (mouse) | Planned | Mouse wheel to scroll chat |

### Tool Cards

| Feature | Status | Description |
|---------|--------|-------------|
| Inline tool display | Done | [Tool: name] → [Result: ...] |
| Collapsed cards | Planned | Name + 1-line preview |
| Expanded cards | Planned | Full parameters + result |
| Click to expand | Planned | Mouse interaction |
| Status indicator | Planned | ⏳ running / ✓ done / ✗ error |

### Streaming Response

| Feature | Status | Description |
|---------|--------|-------------|
| Character-by-character | Planned | Text streams as generated |
| Cursor indicator | Planned | Block cursor at position |
| Spinner animation | Planned | ⠋⠙⠹⠸⠴⠦⠧⠇ while streaming |
| Streaming complete | Planned | Add to history, clear animation |

### Context Sidebar

| Feature | Status | Description |
|---------|--------|-------------|
| Version display | Planned | CLI version number |
| Model name | Planned | Current model name |
| Conversation name | Planned | "Untitled" (placeholder) |
| Token count | Planned | Used / max tokens |
| Usage percentage | Planned | Context window usage |
| Cost estimate | Planned | "$0.00" (placeholder) |
| Files context | Planned | Files read/edited/mentioned |
| File status indicators | Planned | [read], [edited], [added], [mentioned] |

### Input Bar

| Feature | Status | Description |
|---------|--------|-------------|
| Mode indicator | Done | BUILD/PLAN label |
| Input field | Done | Text input area |
| Placeholder text | Done | "Type a message or / for commands..." |
| Cursor positioning | Done | Text cursor in input |
| Command suggestions | Done | Dropdown when typing `/` |
| Input history (Up/Down) | Planned | Browse previous messages |
| Multi-line input (Ctrl+Enter) | Planned | Newlines in input |
| Tab completion | Planned | Complete commands/paths |
| Text selection (mouse) | Planned | Auto-copy on release |

### Popups

| Popup | Status | Trigger |
|-------|--------|---------|
| Model selector | Planned | `/model` or `Ctrl+M` |
| Skill selector | Planned | `/skill` or `Ctrl+S` |
| Command palette | Planned | `Ctrl+K` |
| Help overlay | Planned | `/help` or `Ctrl+?` |

### Popup Features

| Feature | Status | Description |
|---------|--------|-------------|
| Centered overlay | Planned | Popup above chat area |
| Current selection | Planned | Bullet (●) marks current item |
| Type to filter | Planned | Filter list by typing |
| Enter to select | Planned | Keyboard selection |
| Escape to close | Planned | Close popup |

### Mouse Support

| Feature | Status | Description |
|---------|--------|-------------|
| Scroll chat | Planned | Mouse wheel navigation |
| Scroll sidebar | Planned | Independent scroll |
| Select text | Planned | Click and drag to select |
| Auto-copy | Planned | Release mouse to copy |
| Click tool cards | Planned | Expand/collapse tool cards |

### Keyboard Shortcuts

| Key | Status | Action |
|-----|--------|--------|
| `Up`/`Down` (in input) | Planned | Input history navigation |
| `Ctrl+M` | Planned | Open model selector |
| `Ctrl+P` | Planned | Open profile selector |
| `Ctrl+K` | Planned | Open command palette |
| `Ctrl+Enter` | Planned | Add newline to input |
| `Tab` | Done | Toggle Build/Plan mode |
| `Escape` | Planned | Close popup |
| `Page Up`/`Page Down` | Planned | Scroll chat |

### Commands

| Command | Status | Description | Popup |
|---------|--------|-------------|-------|
| `/help` | Done | Show help overlay | Yes |
| `/skill [name]` | Done | Load skill |
| `/model [name]` | Done | Switch model |
| `/tools` | Done | List loaded tools | No |
| `/mode` | Done | Toggle Build/Plan |
| `/clear` | Done | Clear chat history | No |
| `/quit` | Done | Exit CLI | No |

### Syntax Highlighting

| Feature | Status | Description |
|---------|--------|-------------|
| VS Code theme | Planned | One Dark / Monokai inspired |
| Keywords | Planned | Purple (#C678DD) |
| Strings | Planned | Green (#98C379) |
| Numbers | Planned | Orange (#D19A66) |
| Comments | Planned | Gray (#5C6370) |
| Functions | Planned | Blue (#61AFEF) |
| Types | Planned | Yellow (#E5C07B) |

### Error Handling

| Feature | Status | Description |
|---------|--------|-------------|
| Error messages | Done | `[Error]` label with red tint |
| Connection errors | Planned | Actionable error message |
| Tool failures | Planned | Error indicator on tool card |

### Performance

| Feature | Status | Description |
|---------|--------|-------------|
| Virtual scrolling | Planned | Render only visible messages |
| Lazy highlighting | Planned | Cache syntax highlighting |
| Throttled redraws | Planned | Max 60 FPS during streaming |

### Terminal Requirements

| Requirement | Status | Description |
|-------------|--------|-------------|
| 256-color support | Planned | Minimum color support |
| True color | Planned | Preferred for gradients |
| Unicode | Planned | Box-drawing, spinner |
| Minimum size | Planned | 80 cols × 24 rows |

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
