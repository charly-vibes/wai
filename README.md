# wai  /waɪ/

**wai** — pronounced like *"why"* — is a workflow manager for AI-driven development. It can also be read as *"way"*, and that's intentional: the tool's focus is capturing **why it was built that way**.

Most specs define *what* to build. Wai extends the workflow to also *inform* — preserving the research, reasoning, and decisions that shaped the design. When you revisit a project months later, the spec tells you what exists; wai tells you why.

Organizes artifacts using the PARA method (Projects, Areas, Resources, Archives) with project phase tracking, agent config sync, handoff generation, and plugin integration.

## Design Principles

- **Desire Path Alignment** — Pave the cowpaths, make common workflows shortest
- **Self-Healing Errors** — Errors suggest fixes, not just report problems
- **Progressive Disclosure** — Simple by default, powerful when needed
- **Context-Aware** — Offer next steps based on current state

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize wai in current directory
wai init

# Create a project
wai new project my-app

# Check status and get suggestions
wai status

# Manage project phases
wai phase              # Show current phase
wai phase next         # Advance to next phase

# Add artifacts
wai add research "Initial API analysis"
wai add plan "Implementation approach"
wai add design "Component architecture"

# Generate a handoff document
wai handoff create my-app

# Search across artifacts
wai search "authentication"

# View project timeline
wai timeline my-app

# Sync agent configs to tool-specific locations
wai sync
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

| Command | Description |
|---------|-------------|
| `wai init` | Initialize wai in current directory |
| `wai new project <name>` | Create a new project |
| `wai new area <name>` | Create a new area |
| `wai new resource <name>` | Create a new resource |
| `wai add research <content>` | Add research notes |
| `wai add plan <content>` | Add a plan document |
| `wai add design <content>` | Add a design document |
| `wai show [name]` | Show PARA overview or item details |
| `wai move <item> <category>` | Move item between categories |
| `wai phase` | Show current project phase |
| `wai phase next/back/set` | Change project phase |
| `wai status` | Show project status with suggestions |
| `wai sync` | Sync agent configs to tool locations |
| `wai config add <type> <file>` | Add agent config file |
| `wai config list` | List agent config files |
| `wai handoff create <project>` | Generate handoff document |
| `wai search <query>` | Search across artifacts |
| `wai timeline <project>` | View chronological timeline |
| `wai plugin list` | List detected plugins |
| `wai import <path>` | Import existing tool configs |

## Project Phases

Projects progress through six phases: **research** → **design** → **plan** → **implement** → **review** → **archive**

Transitions are flexible — skip forward or go back as needed. Phase history is tracked with timestamps.

## Plugin System

Wai auto-detects and integrates with external tools:

- **beads** — Issue tracking via `.beads/` directory
- **git** — Version control via `.git/` directory
- **openspec** — Specification management via `openspec/` directory

## Development

```bash
cargo build      # Build
cargo run -- status   # Run
cargo test       # Test
cargo clippy     # Lint
```
