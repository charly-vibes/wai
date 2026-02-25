# Commands

Complete CLI reference for wai.

Wai provides a comprehensive set of commands for managing projects, artifacts, phases, and agent configurations.

> **Tip:** Run `wai --help` for a quick overview, or `wai <command> --help` for detailed help on any command.

## Global Flags

Available for all commands:

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Increase verbosity (-v, -vv, -vvv) |
| `-q, --quiet` | Suppress non-error output |
| `--json` | Output machine-readable JSON |
| `--no-input` | Disable interactive prompts |
| `--yes` | Auto-confirm actions with defaults |
| `--safe` | Run in read-only safe mode |

## Initialization

| Command | Description |
|---------|-------------|
| `wai init [--name <name>]` | Initialize wai in current directory |
| `wai tutorial` | Run interactive quickstart tutorial |

## Creating Items

| Command | Description |
|---------|-------------|
| `wai new project <name> [--template <tpl>]` | Create a new project |
| `wai new area <name>` | Create a new area |
| `wai new resource <name>` | Create a new resource |

## Adding Artifacts

| Command | Description |
|---------|-------------|
| `wai add research <content>` | Add research notes to current project |
| `wai add research --file <path>` | Import research from file |
| `wai add research --tags <tags>` | Add tagged research notes |
| `wai add research --project <name>` | Add to specific project |
| `wai add plan <content>` | Add a plan document |
| `wai add plan --file <path>` | Import plan from file |
| `wai add design <content>` | Add a design document |
| `wai add design --file <path>` | Import design from file |

## Diagnostics

| Command | Description |
|---------|-------------|
| `wai doctor` | Diagnose **wai workspace** health (requires initialization) |
| `wai doctor --fix` | Auto-repair detected workspace issues |
| `wai way` | Check **repository best practices** for AI development |
| `wai way --json` | Output best practices check as JSON |

### Choosing the Right Tool

- **Use `wai doctor`** when your `.wai/` directory is missing, a project phase is stuck, or a sync command is failing.
- **Use `wai way`** when you want to improve your overall repository for AI friendliness (e.g., adding `CLAUDE.md`, `.editorconfig`, or `SKILL.md` files).

---

## Agent Configuration

> **⚠️ WARNING:** `wai sync` is **destructive** to your target files. It will overwrite any manual changes in `.cursorrules`, `.claude/config.json`, and other tool-specific config files with the sources from your `.wai/` directory.

| Command | Description |
|---------|-------------|
| `wai sync` | **Overwrite** agent configs to tool-specific locations |
| `wai sync --status` | **Recommended:** Check sync status without modifying |
| `wai config add <type> <file>` | Add agent config (skill/rule/context) |
| `wai config list` | List all agent config files |
| `wai config edit <path>` | **Safe:** Edit config file in $EDITOR |
| `wai import <path>` | Import existing tool configs (.claude/, .cursorrules) |

---

## Viewing & Navigating

## Searching & Timeline

| Command | Description |
|---------|-------------|
| `wai search <query>` | Search across all artifacts |
| `wai search --type <type>` | Filter by type (research/plan/design/handoff) |
| `wai search --in <project>` | Search within specific project |
| `wai search --regex` | Use regex patterns |
| `wai search -n <limit>` | Limit number of results |
| `wai timeline <project>` | View chronological project timeline |
| `wai timeline --from <date>` | Show entries from date onward (YYYY-MM-DD) |
| `wai timeline --to <date>` | Show entries up to date (YYYY-MM-DD) |
| `wai timeline --reverse` | Show oldest first |

## Project Phases

| Command | Description |
|---------|-------------|
| `wai phase` | Show current phase with history |
| `wai phase show` | Display current phase |
| `wai phase next` | Advance to next phase |
| `wai phase back` | Return to previous phase |
| `wai phase set <phase>` | Jump to specific phase |

Available phases: `research`, `design`, `plan`, `implement`, `review`, `archive`

## Agent Configuration

| Command | Description |
|---------|-------------|
| `wai sync` | Sync agent configs to tool-specific locations |
| `wai sync --status` | Show sync status without modifying |
| `wai config add <type> <file>` | Add agent config (skill/rule/context) |
| `wai config list` | List all agent config files |
| `wai config edit <path>` | Edit config file in $EDITOR |
| `wai import <path>` | Import existing tool configs (.claude/, .cursorrules) |

## Resources

| Command | Description |
|---------|-------------|
| `wai resource add skill <name>` | Add a skill resource |
| `wai resource list skills [--json]` | List all skills |
| `wai resource import skills [--from <path>]` | Import skills from directory |

## Session Management

| Command | Description |
|---------|-------------|
| `wai prime [--project <name>]` | Orient at session start: phase, last handoff, suggested next step |
| `wai close [--project <name>]` | Wrap up session: create handoff and show next steps |
| `wai handoff create <project>` | Generate handoff document with plugin context |

`wai prime` detects an in-progress session (via a `.pending-resume` signal) and shows a "RESUMING" banner with the exact next steps from the previous handoff. `wai close` creates the handoff and prints the resume checklist — run it before every `/clear` or end of session.

## Plugins

| Command | Description |
|---------|-------------|
| `wai plugin list` | List all plugins (built-in and custom) |
| `wai plugin enable <name>` | Enable a plugin |
| `wai plugin disable <name>` | Disable a plugin |
| `wai <plugin> <command> [args...]` | Pass-through to plugin commands |

### Built-in Plugins

- **beads** — Commands: `list`, `show`, `ready`
- **git** — Provides context via hooks
- **openspec** — Integrated into status display

## AI-Powered Features

### Why — Reasoning Oracle

| Command | Description |
|---------|-------------|
| `wai why <query>` | Ask why a decision was made (LLM-powered) |
| `wai why <file-path>` | Explain a file's history and rationale |
| `wai why --no-llm <query>` | Force fallback to `wai search` (offline/testing) |
| `wai why --json <query>` | Output machine-readable JSON |

`wai why` queries your accumulated artifacts using an LLM to synthesize a coherent narrative explaining why decisions were made. Falls back to `wai search` when no LLM is configured.

**Configuration** (`.wai/config.toml`):
```toml
[why]
llm     = "claude"        # Backend: "claude" or "ollama" (auto-detected if omitted)
model   = "haiku"         # Claude: "haiku"/"sonnet"; Ollama: "llama3.1:8b"
api_key = "sk-ant-..."    # Claude API key (or ANTHROPIC_API_KEY env var)
fallback = "search"       # On LLM unavailable: "search" (default) or "error"
```

**LLM Backends:**
- **Claude** — set `ANTHROPIC_API_KEY` or add `api_key` to `[why]` config
- **Ollama** — install from https://ollama.com and run a local model

### Reflect — CLAUDE.md Synthesis

| Command | Description |
|---------|-------------|
| `wai reflect` | Synthesize session context into AI guidance block |
| `wai reflect --conversation <file>` | Include conversation transcript as richest input |
| `wai reflect --output <target>` | Target: `claude.md`, `agents.md`, or `both` |
| `wai reflect --dry-run` | Preview changes without writing |
| `wai reflect --yes` | Skip confirmation prompt |

`wai reflect` reads accumulated handoffs, research, and optional conversation transcript, then asks an LLM to extract project-specific conventions and gotchas. Injects the result into `CLAUDE.md`/`AGENTS.md` as a persistent `WAI:REFLECT` block.

**Context sources (ranked by richness):**
1. Conversation transcript (`--conversation <file>`) — raw session detail
2. Handoff artifacts — session summaries and next steps
3. Research/design/plan artifacts — curated decisions

Reuses the `[why]` LLM config — no separate setup required.

## Examples

### Basic Workflow

```bash
# Initialize and create project
wai init
wai new project my-feature

# Add artifacts
wai add research "Evaluated options A and B, chose A for performance"
wai phase next
wai add design "Architecture uses microservices pattern"
```

### Search and Timeline

```bash
# Search with filters
wai search "authentication" --type research
wai search "api.*error" --regex -n 10

# View project history
wai timeline my-feature
wai timeline my-feature --from 2026-02-01 --to 2026-02-15
```

### Configuration Management

```bash
# Add and sync configs
wai config add skill my-skill.md
wai sync --status
wai sync

# Verify with doctor
wai doctor
```

### Repository Best Practices

```bash
# Check repository setup
wai way

# Get JSON output for CI integration
wai way --json | jq '.summary'

# Track best practice adoption
wai way --json | jq '.checks[] | select(.status == "info") | .name'
```

### JSON Output for Automation

```bash
# Get structured data
wai status --json | jq '.projects[] | .name'
wai search "config" --json | jq '.results[].path'
wai plugin list --json
wai way --json | jq '.summary.pass'
```
