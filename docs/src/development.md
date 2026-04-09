# Development

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [just](https://github.com/casey/just) (command runner)

## Architecture Overview

`wai` is built as a modular CLI tool in Rust, focusing on local-first storage and tool-agnostic integration.

For a deeper dive into the codebase structure and ADRs, see [Architecture](./architecture.md).

### Core Systems

#### 1. Registry & Workspace (`src/workspace.rs`, `src/config.rs`)
Manages the `.wai/` directory structure and the PARA method (Projects, Areas, Resources, Archives).
- **Project state**: Tracked via `.state` files in each project.
- **Artifacts**: Stored as date-prefixed markdown files with optional frontmatter.

#### 2. Plugin System (`src/plugin.rs`)
A flexible architecture for detecting and interacting with external tools.
- **Built-in plugins**: `git`, `beads`, `openspec`.
- **Custom plugins**: Loaded from `.wai/plugins/*.toml`.
- **Hooks**: Allows tools to inject context into `wai status` (`on_status`) and `wai handoff create` (`on_handoff_generate`).
- **Passthrough**: Direct execution of tool commands (e.g., `wai beads list`).

#### 3. Agent-Config Sync Engine (`src/sync_core.rs`, `src/commands/sync.rs`)
Syncs a single source of truth for agent configurations to tool-specific locations.
- **Projections**: Defined in `.projections.yml`.
- **Strategies**: `symlink`, `inline` (concatenation), `reference` (path listing), and `copy`.
- **Specialized Targets**: Built-in logic for tools like `claude-code`.

#### 4. Reasoning Oracle (`src/llm.rs`, `src/commands/why.rs`)
Interfaces with LLMs (Claude, Ollama) to answer natural language questions about the codebase using project artifacts as context.

#### 5. Pipelines (`src/commands/pipeline.rs`)
Multi-step, prompt-driven workflows that track run IDs and automatically tag artifacts.

## Building

```bash
just build           # Debug build
just build-release   # Release build
```

## Testing

```bash
just test            # Run all tests
just test-verbose    # Run tests with output
just test-one <name> # Run a specific test
```

## Linting & formatting

```bash
just lint            # Run clippy
just fmt             # Format code
just fmt-check       # Check formatting without changes
```

## Full CI pipeline

Run the same checks that CI runs:

```bash
just ci
```

## Documentation

Build and preview the docs locally:

```bash
just docs            # Build docs
just docs-serve      # Live preview at localhost:3000
```

Requires [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html):

```bash
cargo install mdbook
```
