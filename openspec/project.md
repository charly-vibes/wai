# Project Context

## Purpose

Wai is a workflow manager for AI-driven development. It organizes artifacts using the PARA method (Projects, Areas, Resources, Archives) with project phase tracking, agent config sync, handoff generation, and plugin integration. Beads (`.beads/`) is a separate external tool that wai detects and integrates with via its plugin system.

## Tech Stack

- Rust (clap for CLI, miette for diagnostics, cliclack for interactive prompts)
- TOML for top-level configuration
- YAML for state files, projections, and plugin configs
- File-based storage in `.wai/` directory
- chrono for timestamps, walkdir for directory traversal, slug for filename generation

## Project Conventions

### Code Style

- Idiomatic Rust with clear error handling
- Use `miette` for user-facing errors with diagnostic codes and help text
- Use `cliclack` for interactive prompts
- Use `owo_colors` for terminal styling
- Use `&Path` instead of `&PathBuf` in function signatures

### Architecture Patterns

- Command pattern: `src/commands/<command>.rs` for each command
- CLI definition in `src/cli.rs` using clap derive
- Errors in `src/error.rs` with miette diagnostics
- Config management in `src/config.rs` (PARA directory helpers)
- Project state machine in `src/state.rs`

### Testing Strategy

- Unit tests inline with modules
- Integration tests in `tests/` directory
- Test error messages and help text

### Git Workflow

- Conventional commits (feat:, fix:, docs:, etc.)
- Feature branches off main

## Domain Context

### Core Concepts

- **Project**: An active effort with a defined outcome, tracked through phases
- **Area**: An ongoing area of responsibility without an end date
- **Resource**: Reference material used across projects (agent configs, templates, patterns)
- **Archive**: Completed or inactive items preserved for reference
- **Phase**: Project workflow state (research → plan → design → implement → review → archive)
- **Handoff**: First-class artifact for session context transfer
- **Plugin**: Extensions that detect external tools and integrate their data
- **Agent Config**: Single source of truth for AI assistant configurations, projected to tool-specific formats

### Directory Structure

```
.wai/
├── config.toml
├── projects/            # Active projects with phase state machines
│   └── <name>/
│       ├── .state       # Phase state (YAML)
│       ├── research/    # Date-prefixed research notes
│       ├── plans/       # Date-prefixed plan documents
│       ├── designs/     # Date-prefixed design documents
│       └── handoffs/    # Date-prefixed handoff documents
├── areas/               # Ongoing areas of responsibility
├── resources/           # Reference material
│   ├── agent-config/    # Agent config source of truth
│   │   ├── skills/
│   │   ├── rules/
│   │   ├── context/
│   │   └── .projections.yml
│   ├── templates/
│   └── patterns/
├── archives/            # Completed/inactive items
└── plugins/             # Plugin configurations (YAML)
```

## Important Constraints

- CLI must be fast (<100ms for simple commands)
- Errors must always suggest fixes (self-healing)
- Progressive disclosure: simple by default, powerful when needed
- Works offline-first with file-based storage
- Beads (`.beads/`) is external — wai detects but does not manage it

## External Dependencies

- None required (file-based, no network by default)
- Optional: git integration for status and handoff enrichment
- Optional: beads (`bd`) for issue tracking integration
- Optional: openspec for specification management
