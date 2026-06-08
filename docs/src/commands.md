# Commands

Complete CLI reference for wai.

## Table of Contents

- [Global Flags](#global-flags)
- [Workspace & Initialization](#workspace--initialization)
- [Projects & Artifacts](#projects--artifacts)
  - [Items & Phases](#items--phases)
  - [Adding Artifacts](#adding-artifacts)
  - [Searching & Timeline](#searching--timeline)
- [Agent Configuration](#agent-configuration)
- [AI-Driven Workflows](#ai-driven-workflows)
  - [Reasoning & Reflection](#reasoning--reflection)
  - [Session Management](#session-management)
  - [Pipelines](#pipelines)
- [Plugin System](#plugin-system)
- [Doctor & Health Checks](#doctor-checks)

---

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

### `wai init`

Creates the `.wai/` directory structure and a default `config.toml` in the current repository. Run once per repo. If you omit `--name`, wai uses the directory name.

```bash
# Initialize in the current repo
wai init

# Initialize with an explicit workspace name
wai init --name "my-api"
```

> **Note:** `wai init` is idempotent — running it again in an initialized repo only fills in missing files; it won't overwrite your existing config or artifacts.

### `wai import`

Pulls in existing tool configuration files from another tool's directory (e.g., `.claude/`, `.cursorrules`) and registers them as wai-managed agent configs. Useful when adopting wai in a repo that already has agent instructions. See [Adopt Wai in an Existing Repo](./how-to/adopt-wai.md) for a step-by-step guide.

```bash
# Import Claude Code config from .claude/
wai import .claude/

# Import from a Cursor rules file
wai import .cursorrules
```

### `wai way`

Audits your repository against AI-friendliness best practices: checks for `CLAUDE.md`, `.editorconfig`, skill files, and similar scaffolding. Unlike `wai doctor`, which focuses on the `.wai/` workspace, `wai way` focuses on the repository as a whole.

```bash
# Check which best practices are missing
wai way

# Scaffold any missing skill files
wai way --fix skills
```

### Choosing the Right Tool

- **Use `wai doctor`** when your `.wai/` directory is missing, a project phase is stuck, or a sync command is failing.
- **Use `wai way`** when you want to improve your overall repository for AI friendliness (e.g., adding `CLAUDE.md`, `.editorconfig`, or `SKILL.md` files).

---

## Projects & Artifacts

### Items & Phases

| Command | Description |
|---------|-------------|
| `wai new project <name>` | Create a new project (also `new area`, `new resource`) |
| `wai move <item> <category>` | Move an item between PARA categories |
| `wai status [--json]` | Check project status and suggest next steps |
| `wai show [<name>]` | Show PARA overview or details for a specific item |
| `wai phase [show\|next\|back\|set]` | Show or change the current project phase |

**Available phases:** `research`, `design`, `plan`, `implement`, `review`, `archive`

See [Project Phases](./concepts/phases.md) and [Organizing with PARA](./concepts/para-method.md) for conceptual background.

#### `wai new`

Creates a new PARA item and scaffolds its directory structure under `.wai/`.

```bash
# Start a new feature project
wai new project user-auth

# Create an area for ongoing work
wai new area platform-reliability

# Add a shared resource
wai new resource api-design-guidelines
```

#### `wai move`

Moves an item between PARA categories. Use this when a project completes and needs archiving, or when scope shifts (e.g., a finished project becomes an area of ongoing maintenance).

```bash
# Archive a finished project
wai move user-auth archives

# Promote a resource to an active project
wai move api-design-guidelines projects
```

#### `wai status`

Shows the active project, current phase, recent activity, and next-step suggestions. Run this at the start of a session to orient yourself.

```bash
wai status
wai status --json | jq '.projects[].phase'
```

#### `wai show`

Without arguments, shows a PARA overview of all items. With a project name, shows that project's phase, artifacts, and recent history.

```bash
# Overview of all projects and areas
wai show

# Detailed view of a specific project
wai show user-auth
```

#### `wai phase`

Reads or changes the current project phase. Phases gate which artifact types are expected, for example, you should add research before advancing to design.

```bash
# Check the current phase
wai phase show

# Advance to the next phase
wai phase next

# Jump to a specific phase
wai phase set implement

# Step back one phase (e.g., to add missing design artifacts)
wai phase back
```

> **Note:** `wai phase next` does not prevent you from advancing even if the current phase has no artifacts — it just changes the phase label. Missing artifacts will show up in `wai status` suggestions.

---

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

Artifacts are the core of wai — they capture the *why* behind your decisions so future agents and contributors can reconstruct your reasoning. Each artifact is a Markdown file timestamped and stored under `.wai/projects/<name>/<type>/`.

#### Choosing an Artifact Type

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

#### Examples

```bash
# Capture a quick research note inline
wai add research "Considered Redis vs Memcached; chose Redis for pub/sub support"

# Import a detailed design from a file
wai add design --file architecture-decision.md

# Link an artifact to a beads issue for traceability
wai add plan "Scaffold the service layer first" --bead wai-abc1

# Add a research note to a specific project (not the active one)
wai add research "Benchmark results" --project payment-service

# Correct a locked artifact without modifying the original
wai add research --corrects .wai/projects/auth/research/2026-01-10.md \
  "Correction: the Redis pub/sub approach has 50ms latency at scale"
```

> **Note on `--corrects`:** You can only correct a *locked* artifact (one with a SHA-256 sidecar). To lock an artifact, run `wai pipeline lock` at the relevant pipeline step. The correction is stored as a linked addendum — the original is not modified.

---

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

#### `wai search`

Searches artifact content across your wai workspace. Results include file paths and matching lines. Use `--latest` to find the most recent decision on a topic rather than all mentions.

```bash
# Find all artifacts mentioning a topic
wai search "authentication"

# Find only the most recent research on a topic
wai search "database" --type research --latest

# Regex search with context lines
wai search "api.*error" --regex -C 3

# Search within a specific project
wai search "caching" --in payment-service

# Include bd memories in the search sweep
wai search "retry policy" --include-memories
```

#### `wai timeline`

Shows all artifacts for a project in chronological order, making it easy to trace how a decision evolved across sessions.

```bash
wai timeline user-auth
wai timeline user-auth --from 2026-01-01 --to 2026-03-01
```

---

## Agent Configuration

See [Agent Config Sync](./concepts/agent-config-sync.md) for how projections work.

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

> **⚠️ WARNING:** `wai sync` is **destructive** to your target files. Target locations are defined in `.wai/resources/agent-config/.projections.yml` — the built-in `claude-code` target writes to `.claude/commands/`. Always edit the `.wai/` source files; changes to the projected copies will be overwritten on the next sync.

### `wai sync`

Writes your `.wai/` agent configs out to their tool-specific target locations. Run after editing any config in `.wai/` to push changes out to the agent tools.

```bash
# Write all configs to their targets
wai sync

# Preview what would change without writing
wai sync --status

# Sync from main branch when working in a git worktree
# (use when .wai/ lives on main but your branch doesn't have it)
wai sync --from-main
```

### `wai config`

Manages the list of agent config files tracked by wai. Use `wai config add` to register a new file and `wai config edit` to safely modify a tracked config in your editor (edits the source in `.wai/`, not the projected copy).

```bash
# See what config files are registered
wai config list

# Register a new skill file
wai config add skill .wai/resources/skills/my-skill.md

# Edit a tracked config (opens $EDITOR on the source file)
wai config edit .wai/resources/skills/my-skill.md
```

### `wai resource` — Skill Lifecycle

Skills are reusable agent instruction files. Use `wai resource` to list, install, scaffold, or share them.

```bash
# See all available skills in this workspace
wai resource list skills

# Scaffold a new TDD skill from a template
wai add skill my-tdd-workflow --template tdd

# Install a skill from another repo (e.g., a shared team template)
wai resource install code-review --from-repo ../shared-skills

# Install globally so all your repos can use it
wai resource install deploy-checklist --global
```

> **`wai add skill` vs `wai config add`:** Use `wai add skill` to scaffold a new skill from a template (creates the file and registers it). Use `wai config add skill <file>` to register an existing file you created manually.

### `wai resource` — Import/Export

Share skill bundles between repositories or teams using tar.gz archives.

```bash
# Export a set of skills to a shareable archive
wai resource export code-review deploy-checklist --output team-skills.tar.gz

# Import skills from a directory
wai resource import skills --from ../shared-skills/

# Import from an archive (prompts for confirmation per skill)
wai resource import archive team-skills.tar.gz

# Import from an archive without prompts
wai resource import archive team-skills.tar.gz --yes
```

---

## AI-Driven Workflows

These commands require an LLM backend. Run `wai why --no-llm` or configure a provider before using them in automated contexts. See [LLM Configuration](#backend-selection-and-fallback-behavior) under `wai why`.

### Reasoning & Reflection

See [Reasoning](./concepts/reasoning.md) for conceptual background on how `wai why` and `wai reflect` work.

| Command | Description |
|---------|-------------|
| `wai why <query>` | Ask why a decision was made (LLM-powered) |
| `wai why <file-path>` | Explain a file's history and rationale |
| `wai reflect` | Synthesize session context into a resource file |
| `wai reflect --save-memories` | Save reflection bullets to bd memories |

#### `wai why`

Answers natural-language questions about your project by combining `.wai/` artifacts with git history and querying an LLM. Results include an answer, a decision chain, and references to the artifacts that informed it.

```bash
# Ask a decision question
wai why "Why did we choose TOML over YAML for config?"

# Explain a specific file's history
wai why src/plugin.rs

# Run without LLM — falls back to wai search (useful offline)
wai why --no-llm "plugin architecture"
```

**Backend selection and fallback behavior:**

Wai auto-detects the available LLM backend in this order: Agent (if inside a Claude Code session), Claude API (`ANTHROPIC_API_KEY`), Claude CLI binary, Ollama. If no backend is available:

- By default, wai falls back to `wai search` on the same query (equivalent to `--no-llm`).
- Set `fallback = "error"` in `[llm]` config to get an explicit error instead.

```toml
# .wai/config.toml
[llm]
fallback = "error"   # "search" (default) or "error"
```

**Agent mode (zero-cost inside Claude Code):**

When running inside a Claude Code session, set `llm = "agent"` to route the query through your agent instead of calling the API directly:

```toml
[llm]
llm = "agent"
```

> **Privacy:** The first time you use an external backend (Claude API or Claude CLI), wai displays a privacy notice explaining that your artifact content will be sent to an external service. Set `privacy_notice_shown = true` in `[llm]` to suppress it after you've acknowledged it.

#### `wai reflect`

Synthesizes accumulated session context (handoffs, research, conversation history) into a versioned reflection file in `.wai/resources/reflections/`. Run approximately every 5 sessions.

```bash
# Synthesize and write a reflection file
wai reflect

# Also save key bullets to bd memories for cross-session retrieval
wai reflect --save-memories
```

The reflection is automatically woven into managed blocks in `CLAUDE.md` / `AGENTS.md` so the next agent session starts with the patterns, conventions, and gotchas extracted from your session history. See [Reasoning](./concepts/reasoning.md) for the full synthesis cycle.

---

### Session Management

See [Sessions](./concepts/sessions.md) for the session lifecycle and how handoffs preserve context.

| Command | Description |
|---------|-------------|
| `wai prime [--project <name>]` | Orient at session start: phase, last handoff, next step |
| `wai close [--project <name>]` | Wrap up session: create handoff and next steps |
| `wai handoff create <project>` | Generate handoff document with plugin context |

#### `wai prime`

Runs at the start of a session to orient you: reads the last handoff, reports the current phase, and surfaces the next suggested step. Most Claude Code setups call this automatically via a startup hook.

```bash
# Orient for the active project
wai prime

# Orient for a specific project
wai prime --project payment-service
```

#### `wai close`

Wraps up the current session by creating a handoff document summarizing what was done and what comes next. Always run this before ending a session — it ensures the next session can resume cleanly. A startup hook in Claude Code detects a pending handoff and recovers context automatically.

```bash
# Close the active project session
wai close

# Close a specific project
wai close --project user-auth
```

> **Workflow:** `wai close` calls `wai handoff create` internally. You rarely need `wai handoff create` directly unless you want a handoff mid-session without closing.

#### `wai handoff`

Generates a handoff document capturing the current session state, plugin context (beads status, openspec state), and next steps. Called automatically by `wai close`.

```bash
# Generate a handoff explicitly (mid-session)
wai handoff create user-auth
```

---

### Pipelines

See [Pipelines](./concepts/pipelines.md) for conceptual background on pipeline structure and gates.

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

```bash
# Start a research pipeline
wai pipeline start scientific-research --topic="Effect of caching on P99 latency"

# Check what step you're on after a session break
wai pipeline current

# See what gates must pass before advancing
wai pipeline gates scientific-research --step=generate

# Advance once gates are satisfied
wai pipeline next

# Lock artifacts at a milestone step
wai pipeline lock
```

---

## Plugin System

| Command | Description |
|---------|-------------|
| `wai plugin list` | List all plugins (built-in and custom) |
| `wai plugin enable <name>` | Enable a plugin |
| `wai <plugin> <command>` | Pass-through to plugin commands (e.g. `wai beads list`) |

See [Plugin System](./concepts/plugins.md) for how auto-detection and pass-through work.

```bash
# See which plugins are active
wai plugin list

# Pass a command through to the beads plugin
wai beads ready

# Pass a command through to the openspec plugin
wai openspec list
```

---

## Doctor Checks

`wai doctor` runs the following checks. Use `--fix` to auto-repair where possible.

```bash
# Run all checks
wai doctor

# Auto-fix any repairable issues
wai doctor --fix
```

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

# Orient at session start
wai prime

# Capture research before designing
wai add research "Evaluated options A and B, chose A for performance"
wai phase next

# Add design decisions
wai add design "Architecture uses microservices pattern"
wai phase next

# Close session with a handoff
wai close
```

### Search and Timeline

```bash
# Search with filters
wai search "authentication" --type research
wai search "api.*error" --regex -C 3

# Find the most recent decision on a topic
wai search "caching strategy" --latest

# View project history
wai timeline my-feature
wai timeline my-feature --from 2026-02-01 --to 2026-02-15
```

### Reasoning

```bash
# Ask a question grounded in your artifacts
wai why "Why did we choose event sourcing over CQRS?"

# Explain a file's purpose from its git history
wai why src/event_store.rs

# Force offline/search mode
wai why --no-llm "event store design"

# Synthesize accumulated context into a reflection
wai reflect
```

### JSON Output for Automation

```bash
# Get structured data
wai status --json | jq '.projects[] | .name'
wai search "config" --json | jq '.results[].path'
wai plugin list --json
wai way --json | jq '.summary.pass'
```
