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
| `--safe` | Run in read-only safe mode (blocks write operations) |

### Safe mode

`--safe` prevents all write operations. Commands that modify state — `add`, `sync`, `move`, `pipeline start/next/init/approve/lock`, `phase next/back/set`, and `import` — will exit with an error. Read-only commands like `status`, `search`, `doctor`, `pipeline verify`, and `pipeline current` work normally.

---

## Workspace & Initialization

Commands for managing the wai workspace and checking repo hygiene.

| Command | Description |
|---------|-------------|
| `wai init [--name <name>]` | Initialize wai in current directory |
| `wai tutorial` | Run interactive quickstart tutorial |
| `wai ls [--root <dir>] [--depth <n>] [--timeout <sec>]` | List all wai workspaces under a root directory |
| `wai doctor [--fix]` | Diagnose and repair **wai workspace** health (see [checks](#doctor-checks)) |
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
| `wai add review <content>` | Add a review artifact for an existing artifact |
| `wai add <type> --file <path>` | Import artifact from file |
| `wai add <type> --tags <tags>` | Add tagged artifact (frontmatter-based) |
| `wai add <type> --bead <id>` | Link artifact to a beads issue ID |
| `wai add <type> --project <name>` | Add to specific project (instead of current) |
| `wai add <type> --corrects <path>` | Create an addendum correcting a locked artifact |

#### Choosing an Artifact Type

Wai encourages capturing the right kind of documentation at each stage:

- **Research** (`wai add research`) — Use for gathering information, exploring problem spaces, and evaluating options.
  - *Example:* "Evaluated two DB engines; chose PostgreSQL for its JSONB support."
- **Design** (`wai add design`) — Use for making architectural decisions and defining system structure.
  - *Example:* "Proposed microservices architecture with EventBridge for communication."
- **Plan** (`wai add plan`) — Use for breaking down implementation into specific, actionable steps.
  - *Example:* "Step 1: Scaffold auth service. Step 2: Implement JWT middleware."
- **Review** (`wai add review`) — Use for recording validation results against an existing artifact.
  - *Example:* `wai add review --reviews 2026-04-15-findings.md --verdict pass --severity "critical:0,high:1"`
  - Requires `--reviews <filename>` to specify the target artifact.
  - Optional: `--verdict` (pass/fail/needs-work), `--severity` (level:count pairs), `--produced_by` (skill name).

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
| `wai add skill <name> [--template <tpl>]` | Scaffold a new skill (templates: gather, create, tdd, rule-of-5, ubiquitous-language) |
| `wai resource install <skill> [--global\|--from-repo <path>]` | Install a skill globally or from another repo |
| `wai resource export <skills...> --output <file>` | Export skills to a tar.gz archive |
| `wai resource import skills [--from <dir>]` | Import skills from a directory |
| `wai resource import archive <file> [--yes]` | Import skills from a tar.gz archive |

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
| `wai pipeline list` | List all available pipelines |
| `wai pipeline show <name>` | View steps and gates for a pipeline |
| `wai pipeline init <name>` | Scaffold a new TOML pipeline definition |
| `wai pipeline start <name> --topic="..."` | Start a run with topic substitution |
| `wai pipeline next` | Advance to the next step in the active run |
| `wai pipeline current` | Reprint current step prompt (session recovery) |
| `wai pipeline status <name>` | Show run status (use `--run <id>` for details) |
| `wai pipeline suggest [description]` | Rank pipelines by keyword match |
| `wai pipeline gates [name] [--step=<id>]` | Show gate requirements and live status |
| `wai pipeline check [--oracle=<name>]` | Dry-run gate evaluation without advancing |
| `wai pipeline approve` | Record human approval for current step |
| `wai pipeline validate [name]` | Validate pipeline TOML definitions |
| `wai pipeline lock` | Lock current step's artifacts (SHA-256 hash sidecars) |
| `wai pipeline verify` | Verify integrity of all locked artifacts |

### Plugins

| Command | Description |
|---------|-------------|
| `wai plugin list` | List all plugins (built-in and custom) |
| `wai plugin enable <name>` | Enable a plugin |
| `wai <plugin> <command>` | Pass-through to plugin commands (e.g. `wai beads list`) |

---

## Doctor Checks

`wai doctor` runs the following checks. Use `--fix` to auto-repair where possible.

| Check | What it verifies |
|-------|-----------------|
| Directory structure | Required `.wai/` subdirectories exist |
| Configuration | `.wai/config.toml` is parseable and valid |
| Version | Config version is compatible with installed wai |
| Plugin tools | External tools (git, beads, openspec) are installed and reachable |
| Agent config sync | Projections are in sync between source and target files |
| Project state | Phase state files are valid and consistent |
| Custom plugins | TOML plugin definitions parse correctly |
| README badge | Repository README includes a wai badge |
| Claude Code session hook | Claude Code hooks are configured for wai |
| Skills in repo | Skill files are valid and importable |
| Agent tool coverage | Agent instructions reference available skills |
| Agent instructions | CLAUDE.md / AGENTS.md files are present and well-formed |
| Managed block staleness | Auto-generated sections are up to date |
| Pipeline definitions | Pipeline TOML files parse correctly with valid gates |
| WAI_PROJECT env var | Environment variable matches active project |
| Artifact locks | Locked artifacts match their SHA-256 hashes |

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
