# Development Roadmap

## Project Status (April 2026)

| Component | Status | Description |
|-----------|--------|-------------|
| `mcp-core` | Done | Protocol library with Tool/Command traits |
| `mcp-client` | Done | LLM providers, orchestrator, profile loading |
| `mcp-agent-cli` | Done | Interactive TUI for AI chat |
| `mcp-tools` | Done | 5 tool crates with 13 tools |
| `profiles` | Done | 4 profile configs |

## Current Focus

- Streaming responses in CLI
- OpenAI provider
- More tools (github, docker, database)

## Completed Milestones

### Phase 1: Core Library (Done)

- [x] JSON-RPC 2.0 types
- [x] Tool and Command traits
- [x] Server infrastructure (stdin/stdout)
- [x] Error types
- [x] `inventory`-based tool registration

### Phase 2: Client Library (Done)

- [x] OllamaProvider (OpenAI-compatible endpoint)
- [x] MockProvider (for testing)
- [x] ServerProcess (spawn and communicate with tool servers)
- [x] Orchestrator (chat with automatic tool execution)
- [x] Profile loading (TOML configs)

### Phase 3: CLI (Done)

- [x] TUI with ratatui + crossterm
- [x] Status bar (model, profile, tool count)
- [x] Chat panel with color-coded messages
- [x] Input bar
- [x] Commands (`/help`, `/profile`, `/model`, `/tools`, `/clear`, `/quit`)
- [x] Tool execution via orchestrator

### Phase 4: Tools (Done)

- [x] tool-weather (1 tool)
- [x] tool-filesystem (4 tools)
- [x] tool-system (4 tools)
- [x] tool-web (2 tools)
- [x] tool-utilities (3 tools)

## Next Milestones

### Phase 5: Polish CLI

- [ ] Streaming responses
- [ ] Scroll chat history
- [ ] Syntax highlighting
- [ ] Tool call panel (optional)

### Phase 6: Additional Providers

- [ ] OpenAIProvider
- [ ] AnthropicProvider (via OpenAI-compatible)
- [ ] Custom endpoints

### Phase 7: New Tools

- [ ] tool-github
- [ ] tool-docker
- [ ] tool-database

### Phase 8: Advanced Features

- [ ] Context limit warnings
- [ ] Tool result caching
- [ ] Conversation export
- [ ] Hot profile reload

## Technical Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (edition 2021) |
| Async Runtime | Tokio |
| HTTP Client | minreq (lightweight) |
| Serialization | serde, serde_json |
| Errors | Custom error types |
| TUI | ratatui, crossterm |
| System Info | sysinfo |

## Architecture

```
┌───────────────────────────────────────────────────┐
│                                                   │
│                 mcp-agent-cli                     │
│                 (Interactive TUI)                 │
│                                                   │
│  Commands: /help /profile /model /tools /clear    │
│                                                   │
└──────────────────────┬────────────────────────────┘
                       │
                       ▼
┌───────────────────────────────────────────────────┐
│                                                   │
│                   mcp-client                      │
│                 (Orchestrator)                    │
│                                                   │
│   Profile → Spawn tools → Chat → Execute tools    │
│                                                   │
└──────────┬─────────────────────┬──────────────────┘
           │                     │
           ▼                     ▼
┌──────────────────┐  ┌──────────────────────────────┐
│                  │  │                              │
│     Ollama       │  │        Tool Servers          │
│   (LLM API)      │  │   (stdin/stdout MCP proto)   │
│                  │  │                              │
│  localhost:11434 │  │  tool-weather                │
│                  │  │  tool-filesystem             │
│                  │  │  tool-system                 │
│                  │  │  tool-web                    │
│                  │  │  tool-utilities              │
│                  │  │                              │
└──────────────────┘  └──────────────────────────────┘
```

## Quick Start

```bash
# Build
cargo build --release

# Run CLI
./target/release/mcp-agent-cli

# Use
/profile coding
What files are in this directory?
```

## References

- [MCP Specification](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Ollama OpenAI Compatibility](https://github.com/ollama/ollama/blob/main/docs/openai.md)
