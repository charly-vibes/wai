---
date: 2026-02-04
project: para-restructure
phase: implement
agent: claude-opus-4.5
---

# Session Handoff — PARA Restructure (Phases 1–5)

## What Was Done

### Phase 1: OpenSpec Specifications (complete)

- Archived `bead-lifecycle` spec → `spec.md.archived`
- Rewrote `cli-core` spec — removed bead verbs, added PARA commands (`phase`, `sync`, `config`, `handoff`, `search`, `timeline`)
- Created 5 new specs: `para-structure`, `project-state-machine`, `agent-config-sync`, `handoff-system`, `timeline-search`
- Updated `plugin-system` — YAML configs, hook points, command pass-through, built-in plugins
- Updated `research-management` — research now lives within project subdirectories
- Updated `error-recovery` — replaced `BeadNotFound` with `ProjectNotFound`, `AreaNotFound`, `ResourceNotFound`, added `ConfigSyncError`, `HandoffError`
- Updated `context-suggestions` — phase-based suggestions replacing bead-based
- Updated `onboarding` — welcome screen references projects/phases instead of beads
- Updated `openspec/project.md` — new domain concepts and directory structure

### Phase 2: Core Architecture (complete, builds clean)

- `Cargo.toml` — added `serde_yaml`, `chrono`, `walkdir`, `slug`, `tempfile`
- `src/state.rs` — NEW: 6-phase state machine (research→plan→design→implement→review→archive) with YAML persistence and history tracking
- `src/error.rs` — replaced `BeadNotFound` with `ProjectNotFound`/`AreaNotFound`/`ResourceNotFound`, added `NoProjectContext`, `ConfigSyncError`, `HandoffError`, `Yaml`
- `src/config.rs` — PARA directory constants + helper functions (`projects_dir`, `areas_dir`, `resources_dir`, etc.), `&Path` signatures
- `src/cli.rs` — full restructure: `New{Project,Area,Resource}`, `Add{Research,Plan,Design}`, `Show{name}`, `Move{item,target}`, `Phase{Next,Set,Back,Show}`, `Sync`, `Config{Add,List}`, `Handoff{Create}`, `Search`, `Timeline`, `Plugin{List,Enable,Disable}`, `Import`
- `src/commands/mod.rs` — new dispatch routing, `require_project()` returns `PathBuf`
- 12 new command files: `new.rs`, `add.rs`, `show.rs`, `move_cmd.rs`, `phase.rs`, `sync.rs`, `config_cmd.rs`, `handoff.rs`, `search.rs`, `timeline.rs`, `plugin.rs`, `import.rs`
- Updated `init.rs` — creates full PARA structure with agent-config dirs and `.projections.yml`
- Updated `status.rs` — shows projects with phases, plugin detection, phase-based suggestions
- Updated `README.md` — new architecture, command table, phase documentation

### Phase 3: Agent Config Projections (complete — code existed, verified)

- `src/commands/sync.rs` — parses `.projections.yml`, supports symlink/inline/reference strategies, `--status` flag
- `src/commands/config_cmd.rs` — `wai config add skill|rule|context <file>`, `wai config list`
- `src/commands/import.rs` — imports directories or files, auto-categorizes by filename patterns

### Phase 4–5: Plugin Architecture + Handoff Wiring (complete)

- `src/plugin.rs` — NEW: centralized plugin abstraction layer
  - `PluginDef`, `HookDef`, `PluginCommand`, `ActivePlugin` structs with serde YAML deserialization
  - `builtin_plugins()` — defines git, beads, openspec with detectors, hooks, and commands
  - `detect_plugins()` — auto-detects built-in plugins + loads custom YAML plugins from `.wai/plugins/`
  - `run_hooks()` — executes all hooks for a given event across detected plugins
  - `execute_hook()` — runs a single hook command and captures output
  - `find_plugin_command()` — looks up a specific plugin command by name
  - `execute_passthrough()` — runs plugin pass-through commands with extra args
- `src/commands/handoff.rs` — replaced hardcoded `git status --short` and `bd list --status=open` calls with `plugin::run_hooks("on_handoff_generate")`
- `src/commands/status.rs` — replaced hardcoded `.beads`/`.git`/`openspec` existence checks with `plugin::detect_plugins()`, added `on_status` hook output display (shows recent commits, beads stats)
- `src/commands/plugin.rs` — replaced hardcoded plugin array with `plugin::detect_plugins()`, now shows commands and hooks per plugin
- `src/main.rs` — added `pub mod plugin`

## Verification

- `cargo build` — compiles clean
- `cargo test` — passes (0 tests, no regressions)
- `cargo clippy` — zero warnings

## Key Decisions

- `Phase` is a required subcommand (not optional) — `wai phase` requires `wai phase show` explicitly, because clap doesn't support `Option<Subcommand>`. The plan's `Phase(Option<PhaseCommands>)` was changed to `Phase(PhaseCommands)`.
- Plugin hooks use `std::process::Command` under the hood but are now routed through the centralized `plugin::run_hooks()` system rather than hardcoded per-command.
- Built-in plugins have a `detector` field (e.g. `.git` directory); custom plugins without a detector default to `detected: true`.
- `resolve_project()` in `add.rs` picks the first project alphabetically when `--project` isn't specified. Multi-project workflows may need a "default project" concept.
- The `PROJECTIONS_FILE` constant exists but is unused (the string is hardcoded where needed). Left for future use.
- Plugin enable/disable remains stubbed — no persistence mechanism yet for disabling a detected plugin.

## What Remains (Phases 6–8)

### Phase 6: Timeline + Search polish

Both commands are implemented and functional. Potential improvements:
- Date range filtering for timeline (`--from`, `--to`)
- `--reverse` flag for timeline
- Regex support for search
- Result count limiting for search

### Phase 7: Update existing code

- Change proposals in `openspec/changes/` (self-healing-errors, progressive-disclosure, context-suggestions, first-run-experience) haven't been revised to match the new PARA architecture
- Command pass-through routing (e.g., `wai beads list` → `bd list`) is not yet wired into the CLI dispatch — the `find_plugin_command()` and `execute_passthrough()` functions exist in `plugin.rs` but aren't called from `commands/mod.rs`

### Phase 8: Integration tests

No integration tests exist yet. Key scenarios:
- `wai init` creates correct PARA structure
- `wai new project` creates project with `.state`
- `wai phase next/back/set` transitions work
- `wai add research` creates date-prefixed files
- `wai move` relocates between categories
- `wai search` finds content
- `wai timeline` shows chronological entries
- `wai handoff create` generates handoff with plugin enrichment
- `wai plugin list` shows built-in and custom plugins

## Commits

- `ed52596` — "refactor: restructure wai from bead-centric to PARA-based architecture" (Phases 1–2)
- `5d7431d` — "docs: add handoff and plan for PARA restructure project"
- `4ab7e05` — "feat(plugin): Add plugin abstraction layer and wire through commands" (Phases 4–5)
