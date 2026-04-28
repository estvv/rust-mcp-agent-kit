# Profiles

Profiles define which tools are available to an agent. Each profile is a TOML file listing the tools to load.

## Available Profiles

| Profile | Description | Tools |
|---------|-------------|-------|
| `coding` | Coding assistance | filesystem, web, utilities |
| `personal` | Personal assistant | weather, utilities |
| `devops` | DevOps operations | system, web, utilities |
| `data` | Data processing | utilities, filesystem |

## Structure

```toml
[profile]
name = "coding"
description = "Tools for coding assistance"

[tools]
tool-filesystem = { enabled = true }
tool-web = { enabled = true }
tool-utilities = { enabled = true }
```

## Usage

In `mcp-agent-cli`:

```
/profile coding
/model glm-5:cloud
Hello, what can you help me with?
```

## Creating Custom Profiles

1. Create new TOML file in `profiles/`:

```toml
# profiles/my-profile.toml
[profile]
name = "my-profile"
description = "Custom tool set"

[tools]
tool-weather = { enabled = true }
tool-utilities = { enabled = true }
tool-filesystem = { enabled = true }
```

2. Load in CLI:

```
/profile my-profile
```

## Tool Reference

| Tool Crate | Tools |
|------------|-------|
| `tool-weather` | get_weather |
| `tool-filesystem` | read_file, write_file, list_directory, search_files |
| `tool-system` | get_ram_usage, get_cpu_usage, get_disk_usage, get_processes |
| `tool-web` | http_get, http_post |
| `tool-utilities` | calculate, format_json, current_time |

## Behavior Changes per Profile

| Profile | Tools Available | LLM Behavior |
|---------|-----------------|--------------|
| `coding` | read/write files, web, calculate | "I can edit code, fetch docs, do math" |
| `personal` | weather, calculate, time | "I can answer questions, check weather" |
| `devops` | system stats, web, calculate | "I can monitor infrastructure, fetch APIs" |
| `data` | filesystem, calculate, format_json | "I can process files and data" |