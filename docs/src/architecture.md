# Architecture

This page describes wai's internal architecture for contributors who want to understand or modify the codebase.

## High-Level Structure

Wai is a modular Rust CLI organized into five core subsystems:

```
src/
├── main.rs                 # Entry point
├── cli.rs                  # Clap argument definitions
├── commands/               # One file per subcommand
│   ├── status.rs           # wai status
│   ├── add.rs              # wai add
│   ├── search.rs           # wai search
│   ├── prime.rs            # wai prime
│   ├── close.rs            # wai close
│   ├── why.rs              # wai why (reasoning oracle)
│   ├── reflect.rs          # wai reflect (synthesis)
│   ├── doctor.rs           # wai doctor
│   ├── way.rs              # wai way (best practices)
│   ├── pipeline.rs         # wai pipeline
│   └── ...
├── workspace.rs            # Registry & PARA directory management
├── config.rs               # Config loading (.wai/config.toml)
├── plugin.rs               # Plugin detection, hooks, passthrough
├── sync_core.rs            # Agent config sync engine
├── llm.rs                  # LLM backend abstraction
├── suggestions.rs          # Context-aware next-step suggestions
└── help.rs                 # Tiered help system
```

## Subsystems

### 1. Registry & Workspace

**Files**: `src/workspace.rs`, `src/config.rs`

Manages the `.wai/` directory structure and the PARA method. Project state is tracked via `.state` files. Artifacts are stored as date-prefixed Markdown files with optional YAML frontmatter.

### 2. Plugin System

**Files**: `src/plugin.rs`

Auto-detects external tools by scanning for workspace markers (`.git/`, `.beads/`, `openspec/`). Plugins contribute context through hooks (`on_status`, `on_handoff_generate`, `on_phase_transition`) and expose commands via passthrough. Custom plugins are loaded from `.wai/plugins/*.toml`.

### 3. Agent Config Sync Engine

**Files**: `src/sync_core.rs`, `src/commands/sync.rs`

Projects agent configurations from `.wai/resources/agent-config/` to tool-specific locations using configurable strategies (symlink, inline, reference, copy). Specialized targets like `claude-code` handle complex tool-specific transformations.

### 4. Reasoning Oracle & Synthesis

**Files**: `src/llm.rs`, `src/commands/why.rs`, `src/commands/reflect.rs`

Interfaces with LLMs (Claude API, Claude CLI, Ollama) to answer questions about the codebase (`wai why`) and synthesize project patterns (`wai reflect`). The `LlmClient` trait abstracts backend differences.

### 5. Pipelines

**Files**: `src/commands/pipeline.rs`

Multi-step, prompt-driven workflows defined in TOML. Tracks run state, supports pipeline gates (structural, procedural, oracle, approval), and auto-tags artifacts with run IDs.

## Design Decisions

Significant trade-offs and architectural choices are captured as wai artifacts. Run `wai search "design"` to browse design decisions, or check `.wai/resources/reflections/` for synthesized project patterns.
