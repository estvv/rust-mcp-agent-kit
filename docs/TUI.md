# TUI Specification

Complete specification for the `mcp-agent-cli` terminal user interface.

## Overview

A split-view terminal UI with:
- **Left side**: Chat area with messages and input
- **Right side**: Context sidebar (always visible, fixed 30 columns)

```
┌─────────────────────────────────────────────────────────────┬──────────────────────────────┐
│ MCP Agent | v0.1.0 │ BUILD                                  │ Context                      │
├─────────────────────────────────────────────────────────────┤ v0.1.0                       │
│ Skill: coding │ Tools: 9 │ Ready                          │ Model: glm-5:cloud           │
├─────────────────────────────────────────────────────────────┤──────────────────────────────┤
│                                                             │ Conversation: Untitled       │
│ [User]                                                      │                              │
│ What files are in this directory?                           │ Tokens: 1,234 / 128,000      │
│                                                             │ Usage: 0.96%                 │
│ [Assistant]                                                 │ Cost: $0.00                  │
│ I'll list the directory contents.                           │                              │
│ ┌─────────────────────────────────┐                         │ ───────────Files─────────────│
│ │ 🔧 Tool: list_directory         │                         │ Read Files                   │
| │                                 │▶                       │  docs/FEATURES.md            │
│ │  Result: Cargo.toml, src/...    │                         │                              |
| └─────────────────────────────────┘                         | Modified files               │
│                                                             │  src/main.rs       +179 -27  │
│                                                             │  Cargo.toml        [read]    │
│ The directory contains:                                     │  README.md       [edited]    │
│ - Cargo.toml                                                │                              │
│ - src/                                                      │                              │
│                                                             │                              │
│                                                             │                              │
├─────────────────────────────────────────────────────────────┴──────────────────────────────┤
│ BUILD │ > Type a message or / for commands...                                              │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Layout Structure

### Sections (top to bottom, left to right)

| Section | Height/Width | Description |
|---------|--------------|-------------|
| Title bar | 1 row | App name + mode indicator |
| Status bar | 2 rows | Model, profile, tools, status |
| Main area | Flexible | Split: Chat (flexible) + Sidebar (30 cols) |
| Input bar | ~3 rows | Mode indicator + input field |

---

## Title Bar

**Position**: Top, 1 row, full width

**Content**:
```
MCP Agent │ {MODE}
```

**Mode indicators**:
- `BUILD` (green, bold) - Changes will be applied
- `PLAN` (yellow, bold) - Only suggestions, no changes

---

## Status Bar

**Position**: Below title bar, 2 rows, full width

**Row 1**: Skill name + Tool count + Mode indicator
**Row 2**: Current status message

### Status Bar Fields

| Field | Position | Example |
|-------|----------|---------|
| Model | Left | `Model: glm-5:cloud` |
| Skill | Center-left | `Skill: coding` |
| Tool count | Center | `Tools: 9` |
| Status | Right | `Ready`, `Thinking...`, `Error` |

### Status Colors

| Status | Color |
|--------|-------|
| Ready | Green |
| Thinking | Yellow |
| Streaming | Cyan |
| Error | Red |

---

## Chat Area

**Position**: Left side of main area, flexible width

### Message Format

Each message displays:

```
[Sender]
Message content here...

```

### Sender Labels & Background Colors

| Sender | Label | Background | Text Color |
|--------|-------|------------|------------|
| User | `[User]` | Dark green tint | Green text |
| Assistant | `[Assistant]` | Dark cyan tint | Cyan text |
| System | `[System]` | Dark yellow tint | Yellow text |
| Tool | `[Tool]` | Dark magenta tint | Magenta text |
| Error | `[Error]` | Dark red tint | Red text |

### Message Separators

- Light horizontal line (`─`) between messages
- 1 empty line between sender and content
- 1 empty line after content before separator

### Message Content

- Markdown syntax highlighting (VS Code-style theme)
- Code blocks with language detection
- Inline code with gray background

---

## Tool Cards

Tool calls appear as collapsible cards in the chat.

### Collapsed State (default)

```
┌─────────────────────────────────────────────┐
│ 🔧 Tool: list_directory                   ▶ │
│ ✓ Result: Cargo.toml, src/, README.md      │
└─────────────────────────────────────────────┘
```

**Shows**:
- Tool icon (🔧)
- Tool name
- Status indicator (⏳ running / ✓ done / ✗ error)
- First line of result (truncated to fit)

### Expanded State (click to expand)

```
┌─────────────────────────────────────────────┐
│ 🔧 Tool: list_directory                   ▼ │
│ ─────────────────────────────────────────── │
│ Parameters:                                 │
│   path: "."                                 │
│ ─────────────────────────────────────────── │
│ Result:                                     │
│   Cargo.toml                                │
│   src/                                      │
│   README.md                                 │
│   docs/                                     │
│   profiles/                                 │
└─────────────────────────────────────────────┘
```

**Shows**:
- Tool name
- Parameters (key: value)
- Full result (scrollable if long)

### Collapse Behavior

- Click on card → expand
- Click again → collapse
- Only one tool card expanded at a time

---

## Streaming Response

### Character-by-Character Streaming

When the AI generates text:

```
[Assistant]
The directory contains:█▊

[Spinner: ⠋⠙⠹⠸⠴⠦⠧⠇]
```

- Text appears character-by-character
- Cursor block (`▊`) shows current position
- Spinner animation (`⠋⠙⠹⠸⠴⠦⠧⠇`) below the response while generating

### Streaming Complete

When streaming finishes:

```
[Assistant]
The directory contains:
- Cargo.toml
- src/
- README.md
```

- Spinner disappears
- Full message is added to chat history
- Cursor returns to input bar

---

## Context Sidebar

**Position**: Right side of main area, fixed 30 columns width, always visible

### Sidebar Sections

#### Header Section

```
Context
v0.1.0
Model: glm-5:cloud
Conversation: Untitled
Tokens: 1,234 / 128,000
Usage: 0.96%
Cost: $0.00
```

**Fields**:
| Field | Description |
|-------|-------------|
| Version | CLI version (placeholder for now) |
| Model | Current model name |
| Conversation | Conversation name (placeholder: "Untitled") |
| Tokens | Used tokens / max tokens |
| Usage | Percentage of context used |
| Cost | Estimated cost (placeholder: "$0.00") |

#### Files Section

```
─────────────────────────────
Files

 src/main.rs       [read]
 Cargo.toml        [read]
 README.md       [edited]
 src/lib.rs       [added]
```

**File Entry Format**: `{path} [{status}]`

**Status Indicators**:
| Status | Description |
|--------|-------------|
| `[read]` | File was read by a tool |
| `[edited]` | File was modified |
| `[added]` | File was created |
| `[mentioned]` | User mentioned this file |

---

## Input Bar

**Position**: Bottom of screen, full width, below main area

### Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│ BUILD │ > _                                                              │
└─────────────────────────────────────────────────────────────────────────┘
```

**Left side**: Mode indicator (`BUILD` or `PLAN`)
**Right side**: Input field

### Mode Indicator (Left)

| Mode | Color | Position |
|------|-------|----------|
| BUILD | Green | Left of input, in brackets |
| PLAN | Yellow | Left of input, in brackets |

### Input Field

- Placeholder text (gray): `Type a message or / for commands...`
- Cursor position indicator (text cursor `│` or block `█`)

### Input Features

#### Command Suggestions Dropdown

When user types `/`:

```
┌─────────────────────────────────────────────┐
│ BUILD │ > /pro                              │
│ ┌─────────────────────────────┐             │
│ │ /profile                    │             │
│ │ /projects                   │             │
│ └─────────────────────────────┘             │
└─────────────────────────────────────────────┘
```

- Shows matching commands
- Current input highlighted (cyan)
- Remaining letters in dark gray

#### Input History

- **Up arrow**: Previous message (when input is empty or matches history)
- **Down arrow**: Next message (more recent)
- History persists within session only

#### Tab Completion

- **Tab**: Complete command or file path
- Cycles through matches

#### Multi-line Input

- **Ctrl+Enter**: Add newline to input
- Input area expands to show multiple lines (max 5 lines visible)

---

## Popups

Popups appear as centered overlays above the chat area.

### Opening Popups

| Popup | Trigger |
|-------|---------|
| Model selector | `/model` or `Ctrl+M` |
| Skill selector | `/skill` or `Ctrl+S` |
| Command palette | `Ctrl+K` |
| Help overlay | `/help` or `Ctrl+?` |

### Model Selector Popup

```
┌───────────────────────────────────────┐
│ Select Model                       ✕ │
├───────────────────────────────────────┤
│ ● glm-5:cloud                        │
│   glm-4                              │
│   llama3.1                           │
│   mistral                            │
│   qwen2.5                            │
│   Other...                           │
└───────────────────────────────────────┘
```

- Current model marked with `●`
- Type to filter
- Enter to select
- Escape to close

### Skill Selector Popup

```
┌───────────────────────────────────────┐
│ Select Skill                        ✕ │
├───────────────────────────────────────┤
│ ● coding                              │
│   personal                            │
│   devops                              │
│   data                                │
│   Other...                            │
└───────────────────────────────────────┘
```

- Current skill marked with `●`
- Shows tool count for each skill
- Type to filter
- Enter to select

### Command Palette Popup

```
┌───────────────────────────────────────┐
│ Command Palette                     ✕ │
├───────────────────────────────────────┤
│ /help        Show available commands   │
│ /skill       Load tool skill            │
│ /model       Switch LLM model          │
│ /tools       List loaded tools         │
│ /mode        Toggle Plan/Build mode     │
│ /clear       Clear chat history        │
│ /quit        Exit CLI                  │
└───────────────────────────────────────┘
```

- `Ctrl+K` opens
- Type to filter commands
- Enter to execute

### Help Overlay Popup

```
┌───────────────────────────────────────────────────┐
│ Help                                            ✕ │
├───────────────────────────────────────────────────┤
│ Commands:                                         │
│   /help          Show this help                   │
│   /skill <n>     Load skill                     │
│   /model <n>     Switch model                     │
│   /tools         List loaded tools                │
│   /mode          Toggle Plan/Build mode           │
│   /clear         Clear chat                       │
│   /quit          Exit CLI                         │
│                                                   │
│ Keyboard:                                         │
│   Up/Down        Navigate input history           │
│   Ctrl+M         Open model selector              │
│   Ctrl+S         Open skill selector             │
│   Ctrl+K         Open command palette             │
│   Tab            Toggle Build/Plan mode           │
│   Enter          Send message                     │
│   Escape         Close popup / Cancel             │
│                                                   │
│ Mouse:                                            │
│   Scroll         Scroll chat                      │
│   Click tool card  Expand/collapse                │
│   Select text    Auto-copy to clipboard          │
└───────────────────────────────────────────────────┘
```

---

## Mouse Interaction

### Scroll

- **Mouse scroll**: Scroll chat area up/down
- Sidebar scrolls independently when hovered

### Text Selection & Copy

- **Click and drag**: Select text in chat or sidebar
- **Release mouse**: Selected text auto-copied to clipboard
- Visual highlight shows selected region

### Click Actions

| Click Target | Action |
|--------------|--------|
| Tool card | Expand/collapse |
| Popup item | Select item |
| Sidebar file | Show file details |
| Input field | Focus input |

---

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `Up` | Previous input history (when in input) |
| `Down` | Next input history (when in input) |
| `Page Up` | Scroll chat up |
| `Page Down` | Scroll chat down |
| `Home` | Scroll to top of chat |
| `End` | Scroll to bottom of chat |

### Popups

| Key | Action |
|-----|--------|
| `Ctrl+M` | Open model selector |
| `Ctrl+S` | Open skill selector |
| `Ctrl+K` | Open command palette |
| `Ctrl+?` | Open help overlay |
| `Escape` | Close popup |

### Input

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Ctrl+Enter` | Add newline (multi-line input) |
| `Tab` | Toggle Build/Plan mode |
| `Backspace` | Delete character |
| `Ctrl+U` | Clear input line |
| `Ctrl+C` | Copy selected text |

### Mode Toggle

| Key | Action |
|-----|--------|
| `Tab` | Toggle between BUILD and PLAN mode |

### Quit

| Key | Action |
|-----|--------|
| `Ctrl+C` (twice) | Exit CLI |
| `Escape` (from popup) | Close popup |
| `Escape` (from input) | Clear input or exit |

---

## Commands

All commands start with `/` and trigger actions:

| Command | Description | Popup? |
|---------|-------------|--------|
| `/help` | Show help overlay | Yes |
| `/skill [name]` | Load skill (opens popup if no name) | Yes |
| `/model [name]` | Switch model (opens popup if no name) | Yes |
| `/tools` | List loaded tools in chat | No |
| `/mode` | Toggle Build/Plan mode | No |
| `/clear` | Clear chat history | No |
| `/quit` or `/exit` | Exit CLI | No |

### Command Behavior

- `/skill` (no args) → Opens skill selector popup
- `/skill coding` → Loads skill directly (no popup)
- `/model` (no args) → Opens model selector popup
- `/model glm-4` → Switches model directly (no popup)

---

## Syntax Highlighting

### Theme: VS Code Style (One Dark / Monokai inspired)

| Element | Color |
|---------|-------|
| Keywords (if, for, fn, let) | Purple (#C678DD) |
| Strings | Green (#98C379) |
| Numbers | Orange (#D19A66) |
| Comments | Gray (#5C6370) |
| Functions | Blue (#61AFEF) |
| Types | Yellow (#E5C07B) |
| Variables | White (#ABB2BF) |

### Markdown Elements

| Element | Style |
|---------|-------|
| H1 (`#`) | Green, bold |
| H2 (`##`) | Cyan, bold |
| H3 (`###`) | Yellow, bold |
| Bullet (`-`, `*`) | Cyan bullet |
| Code block (` ``` `) | Dark gray background |
| Inline code (`` ` ``) | Gray background |

---

## Visual Elements

### Borders

- Single line borders (`─`, `│`, `┌`, `┐`, `└`, `┘`)
- Corners: round for popups, square for sections

### Spacing

- 1 row margin around main layout
- 1 column padding inside boxes
- Empty line between messages

### Animations

| Animation | Style |
|-----------|-------|
| Streaming spinner | `⠋⠙⠹⠸⠴⠦⠧⠇` (dots) |
| Loading | `...` or spinner |
| Transition | Instant (no fade) |

---

## Error Handling

### Error Messages

- Displayed with `[Error]` label
- Red background tint
- Show actionable info (e.g., "Use /profile to load a profile")

### Connection Errors

```
[Error]
Failed to connect to Ollama at localhost:11434
Make sure Ollama is running: ollama serve
```

### Tool Failures

```
┌─────────────────────────────────────────────┐
│ 🔧 Tool: read_file                         │
│ ✗ Error: File not found: missing.txt        │
└─────────────────────────────────────────────┘
```

---

## Session State

### Persistent During Session

- Chat history
- Input history
- Files context (touched by tools)

### Reset on Skill Change

- Tools loaded
- Files context (cleared)

### Reset on Model Change

- Nothing (chat and tools preserved)

---

## Files Context Tracking

### When Files Appear

| Trigger | Status |
|---------|--------|
| Tool `read_file` | `[read]` |
| Tool `write_file` | `[edited]` |
| Tool `create_file` | `[added]` |
| User mentions in message | `[mentioned]` |

### File Entry Display

```
src/main.rs       [read]
Cargo.toml        [read]
README.md       [edited]
```

- Path: relative to working directory
- Status: right-aligned in brackets
- Truncate long paths: `.../long/path/file.rs`

---

## Implementation Notes

### Rendering Order

1. Clear terminal
2. Draw title bar
3. Draw status bar
4. Draw sidebar (right)
5. Draw chat area (left, respecting sidebar)
6. Draw input bar
7. Draw popups (if open)
8. Restore cursor position to input field

### Performance Considerations

- Virtual scrolling: only render visible messages
- Lazy syntax highlighting: highlight on first render, cache result
- Throttle redraws: max 60 FPS during streaming

### Terminal Requirements

- 256-color support (minimum)
- True color support (preferred)
- Unicode support (for box-drawing, spinner)
- Minimum size: 80 cols × 24 rows

---

## Summary

| Feature | Implementation |
|---------|---------------|
| Layout | Split view (chat + sidebar) |
| Sidebar | Fixed 30 cols, always visible |
| Status bar | Skill, tools, mode |
| Messages | Background per role, light separators |
| Tool cards | Collapsed by default, click to expand |
| Streaming | Char-by-char + spinner |
| Popups | Command palette, model/skill selectors |
| Input | History (↑/↓), Tab completion, multi-line |
| Mouse | Scroll, select text (auto-copy) |
| Theme | VS Code style syntax highlighting |
