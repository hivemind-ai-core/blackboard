# Blackboard (`bb`)

Local coordination and knowledge surface for AI coding agents.

## What Is It?

Blackboard (`bb`) is a **local, SQLite-backed coordination tool** for multiple AI coding agents and humans working in the same project. It provides:

- **Presence awareness**: Who is active, what they're doing, whether blocked
- **Structured messaging**: Tagged, prioritized, threadable messages
- **Artifact registry**: Track who produced what file and why
- **Cross-tool references**: Link to external tools (e.g., task trackers)

## Quick Start

```bash
# Initialize in your project
bb init

# Post a message
bb post "Starting work on authentication"

# Check agent status
bb status

# List recent messages
bb log
```

## Building

```bash
# Build the binary
cargo build --release

# The binary will be at target/release/bb
```

## Installing

### CLI

```bash
cargo install --path .
```

### MCP Server (for AI agents)

```bash
# Auto-install for Claude, Kimi, and Kilo
bb install

# Or install for a specific tool
bb install --tool claude
bb install --tool kilo
```

This creates the necessary config files (`.mcp.json`, `.kilocode/mcp.json`) and adds them to `.gitignore`.

## Agent Configuration (AGENTS.md)

Add an `AGENTS.md` file to your project with these rules:

```markdown
# Agent Rules

## Coordination

- Use `bb` (Blackboard) for all coordination and communication
- Check `bb status` before starting work to see what other agents are doing
- Post a message with `bb post` when starting, completing, or blocking
- Use tags: `#todo`, `#done`, #blocked`, `#info`
- Set status with `bb status set --task "..." --status working` when actively working

## Artifacts

- Register significant files with `bb artifact-add <path> --description "..."`
- Artifacts should explain *why* the file exists, not just what it is

## References

- Use `bb refs` to find references to tasks, files, or other entities
- Format: `tt:task:123` (tool:what:id)
```

## MCP Tools

When connected via MCP, these tools are available:

| Tool | Description |
|------|-------------|
| `identify` | Establish agent identity |
| `set_status` | Update your status (task, progress, blockers) |
| `get_status` | Get agent status(es) |
| `post_message` | Post a message to the blackboard |
| `read_messages` | Read messages with filters |
| `register_artifact` | Register a file as an artifact |
| `list_artifacts` | List artifacts with filters |
| `find_refs` | Find references to external entities |
| `summary` | Get overview of all activity |

### MCP Parameters

Parameters match CLI flags:

| CLI Flag | MCP Parameter | Description |
|----------|---------------|-------------|
| `--tag` | `tags` | Tags (array) |
| `--by` | `by` | Filter by producer |
| `--reply-to` | `reply_to` | Reply to message ID |
| `--since` | `since` | Duration (e.g., "30m", "1h") |

### Example MCP Usage

```json
{
  "name": "post_message",
  "arguments": {
    "content": "Starting authentication work",
    "tags": ["todo"],
    "priority": "high"
  }
}
```

```json
{
  "name": "set_status",
  "arguments": {
    "current_task": "Implementing OAuth",
    "progress": 50,
    "status": "working"
  }
}
```

## Environment Variables

- `BB_AGENT_ID`: Default agent identity
- `BB_DIR`: Project directory (defaults to current directory)

## Files

- `.bb/blackboard.db`: SQLite database (auto-created by `bb init`)
- `.bb/`: Added to `.gitignore` by default
