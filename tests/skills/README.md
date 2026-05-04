# Skills

Skills define an agent's tools, constraints, and behavior. Each skill is a Markdown file with YAML frontmatter.

## Available Skills

| Skill | Description | Tools | State Machine |
|-------|-------------|-------|---------------|
| `coding` | Coding assistance | filesystem, web, utilities | — |
| `personal` | Personal assistant | weather, utilities | — |
| `devops` | DevOps operations | system, web, utilities | — |
| `data` | Data processing | utilities, filesystem | — |
| `init-web-project` | Scaffold a web project | shell, filesystem | PLAN → EXECUTE → VERIFY |

## Structure

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
## PLAN state
...
## EXECUTE state
...
## VERIFY state
...
```

**Frontmatter fields:**

| Field | Required | Description |
|-------|----------|-------------|
| `skill` | yes | Skill name (used to load it) |
| `description` | yes | One-line description |
| `tools` | yes | List of tool crate names |
| `constraints` | no | `timeout_secs`, `max_output_chars`, `max_iterations` |
| `state_machine` | no | Ordered list of states: `PLAN`, `EXECUTE`, `VERIFY` |
| `input_required` | no | Whether the skill needs user input (default: false) |

The Markdown body after the frontmatter is the **prompt** sent to the LLM. Template variables like `{{max_iterations}}` are resolved at render time.

## Resolution Order

Skills are loaded from three tiers (first match wins):

1. **Project-level**: `.mcp-agent/skills/` (per-repo overrides)
2. **User-level**: `~/.config/mcp-agent/skills/` (personal defaults)
3. **Base-level**: `<executable_dir>/skills/` (bundled defaults)

## Usage

```
/skill coding
/model glm-5:cloud
Hello, what can you help me with?
```

## Tool Reference

| Tool Crate | Tools |
|------------|-------|
| `tool-weather` | get_weather |
| `tool-filesystem` | read_file, write_file, list_directory, search_files |
| `tool-system` | get_ram_usage, get_cpu_usage, get_disk_usage, get_processes |
| `tool-web` | http_get, http_post |
| `tool-utilities` | calculate, format_json, current_time |
| `tool-shell` | run_command |