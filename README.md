# rust-mcp-ecosystem

A Rust-based MCP (Model Context Protocol) ecosystem for local-first AI agents with Ollama.

## Overview

This repository aggregates multiple Rust projects that work together to provide:
- **MCP Servers** - Tools for file operations, system monitoring, GitHub, Docker, databases
- **MCP Client** - Bridges Ollama models to MCP servers
- **RAG CLI** - Semantic code search and chat with tool calling

## Projects

| Repository | Description | Status |
|------------|-------------|--------|
| [rust-mcp-core](./core/) | Shared library for MCP infrastructure | 🔄 In Progress |
| [rust-mcp-servers](./servers/) | Collection of MCP servers (filesystem, github, system, etc.) | ✅ Working |
| [rust-mcp-client](./client/) | MCP client for Ollama integration | 🔄 In Progress |
| [rust-rag-cli](./rag-cli/) | RAG-based semantic code search CLI | ✅ Mature |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    rust-rag-cli                         │
│              (TUI + RAG + Chat Interface)               │
│                        │                                │
│               ┌────────┴────────┐                       │
│               │                 │                       │
│               ▼                 ▼                       │
│        ┌──────────┐      ┌──────────────┐              │
│        │  Ollama  │      │  MCP Client  │              │
│        │   API    │      │              │              │
│        └──────────┘      └──────┬───────┘              │
│                                 │                       │
└─────────────────────────────────┼───────────────────────┘
                                  │
           ┌──────────────────────┼──────────────────────┐
           │                      │                      │
           ▼                      ▼                      ▼
    ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
    │mcp-filesystem│       │  mcp-github │       │ mcp-system  │
    │   server     │       │   server    │       │   server    │
    └─────────────┘       └─────────────┘       └─────────────┘
```

## Quick Start

```bash
# Clone with submodules
git clone --recursive https://github.com/estvv/rust-mcp-ecosystem.git
cd rust-mcp-ecosystem

# Build all components
cargo build --release

# Run RAG CLI
./rag-cli/target/release/rust-rag-cli chat --path ./your-project
```

## Documentation

- [ROADMAP.md](./ROADMAP.md) - Complete development roadmap and plan
- [Architecture](./docs/ARCHITECTURE.md) - System design (coming soon)
- [Getting Started](./docs/GETTING_STARTED.md) - Setup guide (coming soon)

## Status

This ecosystem is under active development. See [ROADMAP.md](./ROADMAP.md) for implementation progress.

## License

MIT License - See individual project LICENSE files for details.
