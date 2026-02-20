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

## Viewing & Navigating

| Command | Description |
|---------|-------------|
| `wai show [name]` | Show PARA overview or item details |
| `wai move <item> <category>` | Move item between PARA categories |
| `wai status` | Show project status with suggestions |

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
| `wai handoff create <project>` | Generate handoff document with plugin context |

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

## Diagnostics

| Command | Description |
|---------|-------------|
| `wai doctor` | Diagnose workspace health |
| `wai doctor --fix` | Auto-repair detected issues |

### Doctor Checks

- Directory structure validation
- Config file integrity
- Plugin tool availability
- Agent config sync status
- Projection validity
- Project state consistency
- Agent instruction files (CLAUDE.md, AGENTS.md)

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

### JSON Output for Automation

```bash
# Get structured data
wai status --json | jq '.projects[] | .name'
wai search "config" --json | jq '.results[].path'
wai plugin list --json
```
