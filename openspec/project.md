# Project Context

## Purpose

Wai is a workflow manager for AI-driven development. It helps developers manage work units (beads), research notes, and project phases with a focus on exceptional CLI UX.

## Tech Stack

- Rust (clap for CLI, miette for diagnostics, cliclack for interactive prompts)
- TOML for configuration
- File-based storage in `.wai/` directory

## Project Conventions

### Code Style

- Idiomatic Rust with clear error handling
- Use `miette` for user-facing errors with diagnostic codes and help text
- Use `cliclack` for interactive prompts
- Use `owo_colors` for terminal styling

### Architecture Patterns

- Command pattern: `src/commands/<command>.rs` for each command
- CLI definition in `src/cli.rs` using clap derive
- Errors in `src/error.rs` with miette diagnostics
- Config management in `src/config.rs`

### Testing Strategy

- Unit tests inline with modules
- Integration tests in `tests/` directory
- Test error messages and help text

### Git Workflow

- Conventional commits (feat:, fix:, docs:, etc.)
- Feature branches off main

## Domain Context

### Core Concepts

- **Bead**: A work unit (feature, fix, chore) that moves through phases
- **Phase**: Workflow state (draft → ready → in-progress → done)
- **Research**: Notes and findings attached to beads or standalone
- **Plugin**: Extensions that hook into wai commands

### Directory Structure

```
.wai/
├── config.toml      # Project configuration
├── beads/           # Work unit files
├── research/        # Research notes
└── plugins/         # Installed plugins
```

## Important Constraints

- CLI must be fast (<100ms for simple commands)
- Errors must always suggest fixes (self-healing)
- Progressive disclosure: simple by default, powerful when needed
- Works offline-first with file-based storage

## External Dependencies

- None required (file-based, no network by default)
- Optional: git integration for sync
