# Engram

AI agent memory system. Gives any AI agent structured, token-efficient project context on demand.

## The Problem

Every time you start a new AI session, switch agents, or hit a context limit, you lose everything. The agent doesn't know what decisions were made, what's in progress, what was tried and failed, or where the last session left off. You waste tokens re-explaining the project every time.

## The Solution

Engram stores project memory in a local SQLite database (`.engram/memory.db`). Any AI agent can query it to get oriented fast. Agents record decisions, update tasks, summarize files, and create session handoffs as they work.

It works two ways:
- **CLI** (`engram context`, `engram decision add`, etc.) - synchronous, fast startup
- **MCP Server** (`engram serve`) - 7 tools over stdio, usable by Claude Code, Cursor, and any MCP-compatible client

## Quick Start

```bash
# Build and install
cargo install --path .

# Initialize in your project
cd your-project
engram init --name "MyProject" --description "What it does"

# Get context (as an agent or human)
engram context --role build --level standard

# Record a decision
engram decision add "Use PostgreSQL" \
  --context "Need a production database" \
  --decision "PostgreSQL for JSONB support and reliability" \
  --alternatives "MySQL,SQLite" \
  --tags "architecture,database"

# Track tasks
engram task add "Implement auth layer" --priority high --phase "Phase 1"
engram task update <id-prefix> --status in_progress

# Start a work session
engram session start --agent claude "Implement Phase 1 storage"

# ... do work ...

# End with handoff notes for the next agent
engram session end "Completed CRUD layer. Next: add FTS5 search indexing. Note: content is a reserved FTS5 column name."
```

## Token-Efficient Tiered Context

The core feature. `engram context` returns role-filtered, level-appropriate markdown:

| Level | Tokens | What's Included |
|-------|--------|-----------------|
| **minimal** | ~50-200 | Project name, stack, next task, last handoff (resume only) |
| **standard** | ~200-1000 | Above + recent decisions (max 5), active tasks (max 10), last session |
| **full** | varies | Above + all decisions with rationale, file summaries, session history |

Roles filter what's relevant:

| Role | Focus |
|------|-------|
| **build** | Task lists, architecture decisions, file summaries |
| **review** | Conventions, file summaries, decisions |
| **debug** | Recent changes, recently completed tasks, known issues |
| **resume** | Last session's full handoff notes |

```bash
# Quick orientation for a build agent
engram context --role build --level standard

# Full context for a code reviewer
engram context --role review --level full

# Minimal handoff for resuming work
engram context --role resume --level minimal
```

## Four Memory Types

| Type | What it stores | CLI |
|------|---------------|-----|
| **Decisions** | Why things were built a certain way, alternatives considered | `engram decision add/list/show` |
| **Tasks** | What's done, in progress, blocked, next | `engram task add/update/list` |
| **File summaries** | What each file does, key types, dependencies | `engram file summarize/list/check` |
| **Sessions** | Where you left off, handoff notes for next agent | `engram session start/end/list` |

## MCP Server

Engram exposes 7 tools over the Model Context Protocol for use by AI agents:

| Tool | Description |
|------|-------------|
| `get_context` | Get token-efficient project context (role + level filtering) |
| `add_decision` | Record an architectural decision with rationale |
| `update_task` | Update a task's status by ID prefix |
| `summarize_file` | Add or update a file summary with hash tracking |
| `start_session` | Start a work session (enforces single active) |
| `end_session` | End session with handoff notes |
| `search` | Full-text search across all memory |

### Configure in Claude Code

Add to your Claude Code MCP settings (`~/.claude/settings.json` or project `.claude/settings.local.json`):

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["serve"]
    }
  }
}
```

The server communicates over stdio using JSON-RPC. Any MCP-compatible client can use it.

### Auto-Start Sessions

When `get_context` is called and no session is active, Engram automatically starts one. This means an agent's first call to `get_context` seamlessly begins tracking the session.

## Full CLI Reference

```
engram init [--name NAME] [--description DESC]   Initialize engram in current project
engram context [--role ROLE] [--level LEVEL]      Get token-efficient project context
engram decision add TITLE --context CTX --decision DEC [--alternatives A,B] [--tags X,Y]
engram decision list [--status active|superseded|reverted]
engram decision show ID_PREFIX
engram task add TITLE [--description DESC] [--priority low|medium|high|critical] [--phase PHASE] [--tags X,Y]
engram task update ID_PREFIX --status todo|in_progress|done|blocked
engram task list [--status STATUS]
engram file summarize PATH --summary TEXT [--key-types A,B] [--deps X,Y] [--tags T]
engram file list
engram file check PATH                            Check if summary is stale (SHA256)
engram session start GOAL [--agent NAME] [--tags X,Y]
engram session end HANDOFF
engram session list [--limit N]
engram search QUERY [-t decision|task|file|session]
engram status                                     Compact project overview
engram export [PATH]                              Export to JSON (stdout if no path)
engram import PATH                                Import from JSON
engram completions SHELL                          Generate shell completions (bash/zsh/fish)
engram serve                                      Start MCP server on stdio
```

## Storage

SQLite with FTS5 full-text search. All data lives in `.engram/memory.db` (add `.engram/` to `.gitignore`). Tags are stored as JSON arrays. Search spans all memory types via a unified FTS5 index.

```bash
# Search across everything
engram search "authentication"

# Filter by type
engram search "PostgreSQL" -t decision
```

## Export / Import

Migrate engram data between projects or back it up:

```bash
# Export everything to JSON
engram export backup.json

# Import into another project
cd other-project
engram init
engram import backup.json
```

## Shell Completions

```bash
# Bash
engram completions bash > ~/.local/share/bash-completion/completions/engram

# Zsh
engram completions zsh > ~/.zfunc/_engram

# Fish
engram completions fish > ~/.config/fish/completions/engram.fish
```

## Architecture

Single Rust crate with `[lib]` + `[[bin]]`. CLI commands are synchronous for fast startup; only `engram serve` uses tokio for the async MCP server.

```
src/
  main.rs              CLI entry point
  lib.rs               Library root
  cli/                 Clap definitions + command handlers
  mcp/                 MCP server (rmcp) + tool definitions
  storage/             SQLite Store struct, schema, CRUD
  models/              All data types (Decision, Task, Session, etc.)
  engine/              Context generation + markdown rendering
```

## Building

Requires Rust 1.70+. SQLite is bundled (no system dependency needed).

```bash
cargo build --release
cargo test              # 23 tests
```

## Releases (GitHub + Homebrew)

Engram ships prebuilt binaries via GitHub Releases for:
- `aarch64-apple-darwin` (Apple Silicon)
- `x86_64-apple-darwin` (Intel macOS)
- `x86_64-unknown-linux-gnu` (Linux)

### 1. Publish a GitHub release from a tag

The workflow at `.github/workflows/release.yml` runs when you push a tag matching `v*`:

```bash
git tag v0.1.1
git push origin v0.1.1
```

It uploads tarballs plus `checksums.txt` to the release.

### 2. Homebrew tap automation

Create a tap repository named `homebrew-tap` under your GitHub account/org (for example `gasmanc/homebrew-tap`) with a `Formula/` folder.

Set these in the main `Engram` repo:
- `secrets.HOMEBREW_TAP_TOKEN`: PAT with write access to the tap repo
- Optional `vars.HOMEBREW_TAP_REPO`: override repo name (defaults to `<owner>/homebrew-tap`)

On each published release, `.github/workflows/homebrew-tap.yml` regenerates and pushes:
- `Formula/engram.rb`

Install with:

```bash
brew tap <yourname>/tap
brew install engram
```

## Publish to crates.io (Optional)

Standard Rust publish flow:

```bash
# 1) bump version in Cargo.toml
# 2) verify package contents
cargo package

# 3) publish (requires CARGO_REGISTRY_TOKEN or prior cargo login)
cargo publish
```

## License

MIT
