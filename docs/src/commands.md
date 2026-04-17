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

---

## Workspace & Initialization

Commands for managing the wai workspace and checking repo hygiene.

| Command | Description |
|---------|-------------|
| `wai init [--name <name>]` | Initialize wai in current directory |
| `wai tutorial` | Run interactive quickstart tutorial |
| `wai ls [--root <dir>] [--depth <n>] [--timeout <sec>]` | List all wai workspaces under a root directory |
| `wai doctor [--fix]` | Diagnose and repair **wai workspace** health |
| `wai way [--fix <CHECK>]` | Check and scaffold **repository best practices** |
| `wai import <path>` | Import existing tool configs (.claude/, .cursorrules) |

### Choosing the Right Tool

- **Use `wai doctor`** when your `.wai/` directory is missing, a project phase is stuck, or a sync command is failing.
- **Use `wai way`** when you want to improve your overall repository for AI friendliness (e.g., adding `CLAUDE.md`, `.editorconfig`, or `SKILL.md` files).

---

## Projects & Artifacts

Manage PARA items (Projects, Areas, Resources, Archives) and their associated artifacts.

### Items & Phases

| Command | Description |
|---------|-------------|
| `wai new project <name>` | Create a new project (also `new area`, `new resource`) |
| `wai move <item> <category>` | Move an item between PARA categories |
| `wai status [--json]` | Check project status and suggest next steps |
| `wai show [<name>]` | Show PARA overview or details for a specific item |
| `wai phase [show\|next\|back\|set]` | Show or change the current project phase |

**Available phases:** `research`, `design`, `plan`, `implement`, `review`, `archive`

### Adding Artifacts

| Command | Description |
|---------|-------------|
| `wai add research <content>` | Add research notes (also `plan`, `design`) |
| `wai add <type> --file <path>` | Import artifact from file |
| `wai add <type> --tags <tags>` | Add tagged artifact (frontmatter-based) |
| `wai add <type> --bead <id>` | Link artifact to a beads issue ID |
| `wai add <type> --project <name>` | Add to specific project (instead of current) |

#### Choosing an Artifact Type

Wai encourages capturing the right kind of documentation at each stage:

- **Research** (`wai add research`) — Use for gathering information, exploring problem spaces, and evaluating options.
  - *Example:* "Evaluated two DB engines; chose PostgreSQL for its JSONB support."
- **Design** (`wai add design`) — Use for making architectural decisions and defining system structure.
  - *Example:* "Proposed microservices architecture with EventBridge for communication."
- **Plan** (`wai add plan`) — Use for breaking down implementation into specific, actionable steps.
  - *Example:* "Step 1: Scaffold auth service. Step 2: Implement JWT middleware."

### Searching & Timeline

| Command | Description |
|---------|-------------|
| `wai search <query>` | Search across all artifacts |
| `wai search --type <type>` | Filter by type (research/plan/design/handoff) |
| `wai search --in <project>` | Search within specific project |
| `wai search --tag <tag>` | Filter by tag (repeatable) |
| `wai search --regex` | Use regex patterns for query |
| `wai search --latest` | Return only the most recently dated match |
| `wai search -C <n>` | Show N lines of context around each match |
| `wai search --include-memories` | Include `bd memories` in search results |
| `wai timeline <project>` | View chronological project timeline |
| `wai timeline --from <date>` | Filter by date range (YYYY-MM-DD) |

---

## Agent Configuration

Manage how AI agents interact with your project through skills, rules, and context.

| Command | Description |
|---------|-------------|
| `wai sync` | **Overwrite** agent configs to tool-specific locations |
| `wai sync --status` | Check sync status without modifying files |
| `wai sync --from-main` | Sync resources from main git worktree |
| `wai config list` | List all agent config files |
| `wai config add <type> <file>` | Add agent config (skill/rule/context) |
| `wai config edit <path>` | **Safe:** Edit config file in $EDITOR |
| `wai resource list skills` | List all available skills |
| `wai resource add skill <name>` | Scaffold a new skill (also `install`, `export`) |

> **⚠️ WARNING:** `wai sync` is **destructive** to your target files. It will overwrite manual changes in `.cursorrules`, `.claude/config.json`, etc., with the sources from your `.wai/` directory.

---

## AI-Driven Workflows

Advanced reasoning, session management, and automated pipelines.

### Reasoning & Reflection

| Command | Description |
|---------|-------------|
| `wai why <query>` | Ask why a decision was made (LLM-powered) |
| `wai why <file-path>` | Explain a file's history and rationale |
| `wai reflect` | Synthesize session context into a resource file |
| `wai reflect --save-memories` | Save reflection bullets to bd memories |

### Session Management

| Command | Description |
|---------|-------------|
| `wai prime [--project <name>]` | Orient at session start: phase, last handoff, next step |
| `wai close [--project <name>]` | Wrap up session: create handoff and next steps |
| `wai handoff create <project>` | Generate handoff document with plugin context |

### Pipelines

| Command | Description |
|---------|-------------|
| `wai pipeline init <name>` | Scaffold a new TOML pipeline definition |
| `wai pipeline start <name>` | Start a run; writes run ID to `.wai/.pipeline-run` |
| `wai pipeline next` | Advance to the next step in the active run |
| `wai pipeline current` | Show the current step of the active run |
| `wai pipeline status <name>` | Show run status (use `--run <id>` for details) |
| `wai pipeline suggest` | Get a skill suggestion for a topic |
| `wai pipeline lock` | Lock current step's artifacts (SHA-256 hash sidecars) |
| `wai pipeline verify` | Verify integrity of all locked artifacts |

### Plugins

| Command | Description |
|---------|-------------|
| `wai plugin list` | List all plugins (built-in and custom) |
| `wai plugin enable <name>` | Enable a plugin |
| `wai <plugin> <command>` | Pass-through to plugin commands (e.g. `wai beads list`) |

---

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

### JSON Output for Automation

```bash
# Get structured data
wai status --json | jq '.projects[] | .name'
wai search "config" --json | jq '.results[].path'
wai plugin list --json
wai way --json | jq '.summary.pass'
```
