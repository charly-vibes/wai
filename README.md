# wai  /waɪ/

**wai** — pronounced like *"why"* — is a command-line workflow manager for AI-driven development. It can also be read as *"way"*, and that's intentional: the tool's focus is capturing **why it was built that way**.

Most specs define *what* to build. Wai extends the workflow to also *inform* — preserving the research, reasoning, and decisions that shaped the design. When you revisit a project months later, the spec tells you what exists; wai tells you why.

Organizes artifacts using the PARA method (Projects, Areas, Resources, Archives) with project phase tracking, agent config sync, handoff generation, and plugin integration.

## Design Principles

- **Desire Path Alignment** — Pave the cowpaths, make common workflows shortest
- **Self-Healing Errors** — Errors suggest fixes, not just report problems
- **Progressive Disclosure** — Simple by default, powerful when needed
- **Context-Aware** — Offer next steps based on current state

## Installation

**Requirements:** Rust 1.70+ and Cargo

```bash
cargo install --path .
```

### Quick Start in 5 Minutes

```bash
# 1. Initialize wai (30 seconds)
wai init

# 2. Create your first project (30 seconds)
wai new project my-feature

# 3. Add research findings (1 minute)
wai add research "Evaluated approach X vs Y - chose X for performance"

# 4. Check status and get suggestions (30 seconds)
wai status

# 5. Move through phases (2 minutes)
wai phase next                              # → design
wai add design "Using microservices pattern"
wai phase next                              # → plan
wai add plan "Phase 1: API, Phase 2: UI"
wai phase next                              # → implement

# 6. Create handoff when done (30 seconds)
wai handoff create my-feature
```

**Result:** You now have a project with documented research, designs, and plans - ready for implementation!

## Quick Start

```bash
# Initialize wai in current directory
wai init

# Or run the interactive tutorial
wai tutorial

# Create a project
wai new project my-app

# Check status and get suggestions
wai status

# Manage project phases
wai phase              # Show current phase with history
wai phase next         # Advance to next phase

# Add artifacts
wai add research "Initial API analysis"
wai add research --file notes.md --tags "api,research"
wai add plan "Implementation approach"
wai add design "Component architecture"

# Search and explore
wai search "authentication"
wai search --type research --regex "api.*auth"
wai timeline my-app --from 2026-02-01

# Generate a handoff document
wai handoff create my-app

# Sync agent configs to tool-specific locations
wai sync --status      # Check sync status
wai sync               # Apply syncs

# Check workspace health
wai doctor             # Diagnose issues
wai doctor --fix       # Auto-repair problems

# Work with plugins
wai plugin list
wai beads list         # Pass-through to beads plugin
```

## Project Structure

```
.wai/
├── config.toml
├── projects/            # Active projects with phases
│   └── my-app/
│       ├── .state       # Phase state machine (YAML)
│       ├── research/    # Date-prefixed research notes
│       ├── plans/       # Date-prefixed plan documents
│       ├── designs/     # Date-prefixed design documents
│       └── handoffs/    # Date-prefixed handoff documents
├── areas/               # Ongoing areas of responsibility
├── resources/           # Reference material
│   ├── agent-config/    # Single source of truth for agent configs
│   │   ├── skills/
│   │   ├── rules/
│   │   ├── context/
│   │   └── .projections.yml
│   ├── templates/
│   └── patterns/
├── archives/            # Completed/inactive items
└── plugins/             # Plugin configurations
```

## Commands

### Core Commands

| Command | Description |
|---------|-------------|
| `wai init [--name <name>]` | Initialize wai in current directory |
| `wai new project <name>` | Create a new project |
| `wai new area <name>` | Create a new area |
| `wai new resource <name>` | Create a new resource |
| `wai show [name]` | Show PARA overview or item details |
| `wai move <item> <category>` | Move item between categories |
| `wai status` | Show project status with suggestions |
| `wai tutorial` | Run interactive quickstart tutorial |
| `wai doctor [--fix]` | Diagnose workspace health (--fix to auto-repair) |

### Artifact Management

| Command | Description |
|---------|-------------|
| `wai add research <content>` | Add research notes |
| `wai add research --file <path>` | Import research from file |
| `wai add research --tags <tags>` | Add tagged research notes |
| `wai add plan <content>` | Add a plan document |
| `wai add design <content>` | Add a design document |
| `wai search <query>` | Search across all artifacts |
| `wai search --type <type>` | Filter by type (research/plan/design/handoff) |
| `wai search --in <project>` | Search within specific project |
| `wai search --regex` | Use regex patterns |
| `wai search -n <limit>` | Limit number of results |
| `wai timeline <project>` | View chronological timeline |
| `wai timeline --from <date>` | Show entries from date onward |
| `wai timeline --to <date>` | Show entries up to date |
| `wai timeline --reverse` | Show oldest first |

### Phase Management

| Command | Description |
|---------|-------------|
| `wai phase` | Show current phase with history |
| `wai phase show` | Display current phase |
| `wai phase next` | Advance to next phase |
| `wai phase back` | Return to previous phase |
| `wai phase set <phase>` | Jump to specific phase |

### Agent Configuration

| Command | Description |
|---------|-------------|
| `wai sync` | Sync agent configs to tool locations |
| `wai sync --status` | Show sync status without modifying |
| `wai config add <type> <file>` | Add agent config (skill/rule/context) |
| `wai config list` | List all agent config files |
| `wai config edit <path>` | Edit config file in $EDITOR |
| `wai import <path>` | Import existing tool configs |

### Resources

> **Note:** Resource management commands are currently in development. The command structure is defined but full implementation is pending.

| Command | Description |
|---------|-------------|
| `wai resource add skill <name>` | Add a skill resource *(in progress)* |
| `wai resource list skills [--json]` | List all skills *(in progress)* |
| `wai resource import skills [--from <path>]` | Import skills from directory *(in progress)* |

### Session Management

| Command | Description |
|---------|-------------|
| `wai handoff create <project>` | Generate handoff document |

### Plugin Management

| Command | Description |
|---------|-------------|
| `wai plugin list` | List all plugins (built-in and custom) |
| `wai plugin enable <name>` | Enable a plugin |
| `wai plugin disable <name>` | Disable a plugin |
| `wai <plugin> <command>` | Pass-through to plugin commands |

### Global Flags

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Increase verbosity (-v, -vv, -vvv) |
| `-q, --quiet` | Suppress non-error output |
| `--json` | Output machine-readable JSON |
| `--no-input` | Disable interactive prompts |
| `--yes` | Auto-confirm actions with defaults |
| `--safe` | Run in read-only safe mode |

## Project Phases

Projects progress through six phases: **research** → **design** → **plan** → **implement** → **review** → **archive**

### Phase Workflow

- **Research** — Gather information, explore options, understand the problem
- **Design** — Make architectural decisions, choose approaches
- **Plan** — Break work into tasks, define implementation steps
- **Implement** — Write code guided by research and plans
- **Review** — Validate against plans, ensure quality
- **Archive** — Wrap up, document outcomes

### Phase Features

- **Flexible transitions** — Skip forward or go back as needed
- **Phase history tracking** — All transitions recorded with timestamps
- **Context-aware suggestions** — Get relevant next steps based on current phase
- **Plugin integration** — Plugins can react to phase changes via hooks

Use `wai phase` to view current phase with complete history, or `wai phase next/back/set` to transition.

## Plugin System

Wai auto-detects and integrates with external tools through a flexible plugin architecture.

### Built-in Plugins

- **beads** — Issue tracking via `.beads/` directory
  - Commands: `list`, `show`, `ready` (read-only)
  - Hooks: `on_handoff_generate`, `on_status`

- **git** — Version control via `.git/` directory
  - Hooks: `on_handoff_generate` (git status), `on_status` (recent commits)

- **openspec** — Specification management via `openspec/` directory
  - Integrated into status display with change progress

### Custom Plugins

Create custom plugins by adding YAML files to `.wai/plugins/`:

```yaml
name: my-tool
description: Custom tool integration
detector:
  type: directory
  path: .mytool
commands:
  - name: list
    description: List items
    command: mytool list
    read_only: true
hooks:
  - type: on_status
    command: mytool stats
    inject_as: mytool_stats
```

### Plugin Hooks

Plugins can inject context into various workflows:
- `on_status` — Add information to status output
- `on_handoff_generate` — Include context in handoffs
- `on_phase_transition` — React to phase changes

## Agent Config Sync

Wai maintains a single source of truth for agent configurations in `.wai/resources/agent-config/` and syncs them to tool-specific locations using three strategies:

### Choosing a Sync Strategy

| Strategy | Use When | Example Tools | Pros | Cons |
|----------|----------|---------------|------|------|
| **Symlink** | Tool expects directory with individual files | Claude Code, Aider | Live updates, no duplication | Requires symlink support |
| **Inline** | Tool expects single concatenated file | Cursor, Windsurf | Works everywhere, simple | No per-file editing |
| **Reference** | Tool can follow file paths | Custom scripts | Flexible, explicit | Requires tool support |

### Sync Strategies

Configure in `.wai/resources/agent-config/.projections.yml`:

**1. Symlink Strategy** — Create symlinks to source files
```yaml
- strategy: symlink
  sources:
    - skills/
  target: .claude/skills/
```

**2. Inline Strategy** — Concatenate files into single target
```yaml
- strategy: inline
  sources:
    - rules/base.md
    - rules/security.md
  target: .cursorrules
```

**3. Reference Strategy** — Create reference file with source paths
```yaml
- strategy: reference
  sources:
    - context/
  target: .agents/context-refs.md
```

Run `wai sync` to apply projections, or `wai sync --status` to check sync status without modifying files.

⚠️ **Important:** Wai always overwrites target files during sync. Do not manually edit synced files (e.g., `.cursorrules`) - edit the source files in `.wai/resources/agent-config/` instead.

## JSON Output

All major commands support `--json` flag for machine-readable output:

```bash
wai status --json           # Project status, plugins, suggestions
wai search "query" --json   # Search results with context
wai timeline proj --json    # Timeline entries
wai plugin list --json      # Plugin information
```

JSON output enables integration with other tools, scripts, and automation workflows.

## Advanced Features

### Workflow Detection

Wai automatically detects project patterns and provides context-aware suggestions:

- **NewProject** — *Triggers:* Project exists but has no artifacts, in research phase
  - *Suggests:* Add initial research, import existing notes, run tutorial

- **ResearchPhaseMinimal** — *Triggers:* In research phase with ≤1 research artifacts
  - *Suggests:* Continue gathering information, add more research artifacts

- **ReadyToImplement** — *Triggers:* In plan/design phase with design documents created
  - *Suggests:* Move to implement phase, review designs, create handoff

- **ImplementPhaseActive** — *Triggers:* Currently in implement phase
  - *Suggests:* Run tests, document decisions, advance when ready

### Tutorial Mode

Run `wai tutorial` for an interactive quickstart guide:

```bash
wai tutorial
```

The tutorial is a 5-step guided walkthrough perfect for first-time users:

1. **Introduction** - What is wai and the "why" philosophy
2. **PARA Method** - Understanding Projects, Areas, Resources, Archives
3. **Phase Workflow** - Six phases from research to archive
4. **Core Commands** - Hands-on practice with key features
5. **Session Handoffs** - Knowledge transfer between sessions

**First-run detection:** Wai automatically prompts new users to run the tutorial. You can run it anytime to refresh your knowledge or show others how to use wai.

### Safe Mode

Use `--safe` flag for read-only operations:
```bash
wai status --safe           # View status without any modifications
wai search "query" --safe   # Search without side effects
```

Prevents accidental modifications during exploration or automated analysis.

### Artifact Tags

Add tags to research artifacts for better organization:
```bash
wai add research "API findings" --tags "api,security,authentication"
```

**Tag Format:** Comma-separated alphanumeric strings. Avoid spaces around commas.
- ✅ Good: `"api,security,v2"`
- ❌ Bad: `"api, security, v2"` (extra spaces)
- ❌ Bad: `"api security"` (use comma separator)

Tags are stored in YAML frontmatter and searchable via `wai search`.

### Doctor Command

The `wai doctor` command performs comprehensive workspace health checks:

**Checks performed:**
- Directory structure validation
- Config file integrity
- Plugin tool availability
- Agent config sync status
- Projection validity (symlink, inline, reference)
- Project state consistency
- Agent instruction files (CLAUDE.md, AGENTS.md)

**Auto-fix mode:**
```bash
wai doctor --fix       # Automatically repair detected issues
```

Doctor provides actionable error messages with suggestions for manual fixes when auto-repair isn't possible.

## Real-World Workflows

### Starting a New Project

```bash
# Initialize and create project
wai init
wai new project feature-auth

# Enter research phase
wai add research "Evaluated OAuth vs JWT - JWT chosen for stateless API"
wai add research --file research/auth-comparison.md

# Check status
wai status
# Output:
# Project: feature-auth
# Phase: research (2 artifacts)
# Suggestion: Move to design when ready → wai phase next

# Move to design
wai phase next
wai add design "JWT stored in httpOnly cookies, refresh token rotation"

# Create implementation plan
wai phase next
wai add plan "1. User model, 2. Auth middleware, 3. Token service, 4. Tests"

# Ready to implement
wai phase next
```

### Mid-Project Session Handoff

```bash
# Check what's been done
wai status
wai timeline feature-auth

# Search for previous decisions
wai search "database" --in feature-auth

# Create handoff for next session
wai handoff create feature-auth

# Handoff includes: phase, recent artifacts, plugin context (git status, open issues)
```

### Working with Multiple Projects

```bash
# Check all projects
wai show

# Add to specific project
wai add research "Performance findings" --project optimization

# Search across all projects
wai search "performance"

# Move completed project to archives
wai move feature-auth archives
```

### Agent Configuration Workflow

```bash
# Add a new skill
wai config add skill my-skill.md

# Check all configs
wai config list

# Edit existing config
wai config edit skills/my-skill.md

# Sync to tool locations
wai sync --status                    # Preview changes
wai sync                             # Apply sync

# Verify sync worked
wai doctor
```

### Troubleshooting

```bash
# Check workspace health
wai doctor

# Auto-fix common issues
wai doctor --fix

# Check sync status without modifying
wai sync --status

# Safe mode for read-only exploration
wai show --safe
wai search "config" --safe

# JSON output for debugging
wai status --json | jq .
```

## Documentation

- **[Quick Start](docs/src/quick-start.md)** - Get started in 5 minutes
- **[Commands Reference](docs/src/commands.md)** - Complete command documentation
- **[Troubleshooting](docs/src/troubleshooting.md)** - Common issues and solutions
- **[FAQ](docs/src/faq.md)** - Frequently asked questions
- **[CHANGELOG](CHANGELOG.md)** - Version history and upgrade guide

### Concepts
- **[PARA Method](docs/src/concepts/para-method.md)** - Project organization
- **[Project Phases](docs/src/concepts/phases.md)** - Workflow stages
- **[Plugin System](docs/src/concepts/plugins.md)** - Extensibility and integration
- **[Agent Config Sync](docs/src/concepts/agent-config-sync.md)** - Configuration management

### Advanced
- **[JSON Output](docs/src/advanced/json-output.md)** - Automation and scripting
- **[Workflow Detection](docs/src/advanced/workflow-detection.md)** - Context-aware suggestions

## Development

```bash
cargo build                  # Build
cargo run -- status          # Run
cargo test                   # Test
cargo clippy                 # Lint
cargo run -- --help -v       # View advanced help
```

## Contributing

See [Development](docs/src/development.md) for guidelines on contributing to wai.

## License

MIT
