# Changelog

All notable changes to wai will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Calendar Versioning](https://calver.org/) (YYYY.M.MICRO).

## [2026.4.5] - 2026-04-19

### Added

- **Prime plan context** — `wai prime` now shows recent plans even when the latest handoff is empty, so resume context still includes intended next work
- **Initialization workflow improvements** — `wai init` better repairs and refreshes existing workspaces during re-init
- **Built-in `.agents` projection** — initialization now ships built-in agent configuration projection support out of the box
- **Implementation status page check** — `wai way` now checks for a docs status page to improve implementation-status discoverability
- **Agent projection sync check** — `wai way` now reports whether projected agent config appears out of sync and points to the relevant fix path

### Changed

- **Local artifact ignores** — `.gitignore` now excludes local agent/tool artifacts such as `.claude/worktrees/` and `.codex`
- **Repo maintenance cleanup** — archived completed wai project artifacts and finished associated ticket housekeeping

---

## [2026.4.4] - 2026-04-17

### Added

#### Artifact Locking & Integrity Verification
- **SHA-256 artifact hashing** — tamper-evident hashing for pipeline artifacts with LF-normalized content (`sha2` crate)
- **`wai pipeline lock`** — lock current step's artifacts with SHA-256 hashes stored in `.lock` sidecar TOML files
- **`wai pipeline verify`** — recompute hashes and report integrity mismatches (exits non-zero on failure)
- **Auto-lock on advancement** — `wai pipeline next` automatically locks artifacts when step has `lock = true`
- **`--corrects=<path>` flag** — create addenda referencing locked artifacts via `wai add`; auto-tags with `pipeline-addendum:<step-id>`
- **Unlock warning** — warns when `--corrects` targets an unlocked artifact
- **Doctor lock checks** — `wai doctor` walks `.lock` sidecars and verifies SHA-256 integrity

#### Coverage Gate
- **CoverageGate tier** — new gate tier requiring input coverage manifests at pipeline step boundaries
- **Coverage gate blocking** — `wai pipeline next` blocks when coverage manifest is missing
- **Lock-without-gate warning** — `wai pipeline validate` warns when step locks artifacts without a gate
- **Addenda display** — `wai pipeline current` shows addenda for the active step

#### Other
- **Context threshold warning** — managed block warns the user when context nears 40% instead of silently closing

### Fixed
- **`--project` flag position** — `wai phase next --project foo` now accepted (marked as global arg)
- **Embedded dolt mode** — switched from server mode to prevent zombie `dolt sql-server` processes causing lock errors
- **Stale quick reference** — updated `pipeline run`→`pipeline start` and `pipeline advance`→`pipeline next`
- **Broken doc links** — corrected anchor refs and dead relative links across docs
- **Clippy lints** — resolved `unnecessary_sort_by`, `question_mark`, `manual_checked_ops` from Rust 1.95
- **Test assertion** — narrowed `doctor_warns_wai_project_empty` to avoid false matches on CI

### Changed
- **Documentation overhaul** — new concept pages (sessions, reasoning, toolchain synergy, glossary, architecture, adopt-wai guide), rewritten introduction with problem/solution framing
- **Comprehensive docs audit** — documented review artifacts, resource commands, safe mode, all 16 doctor checks, and 5 missing pipeline subcommands

---

## [2026.4.2] - 2026-04-07

### Fixed
- **Beads CLI discoverability** — managed block now prominently shows `bd` as the CLI command throughout (detected tools, "When to Use What" table, key distinction, quick reference header) so agents stop looking for a `beads` binary that doesn't exist

---

## [2026.4.1] - 2026-04-02

### Added

#### Pipeline Gates
- **Gate protocol engine** — 4-tier validation system (artifact, review, oracle, custom) for enforcing quality checks at pipeline step boundaries
- **`wai pipeline show`** — display pipeline definition with steps and gate requirements
- **`wai pipeline gates`** — list all gates for a pipeline with validation tier details
- **`wai pipeline check`** — check gate satisfaction status for the current run
- **`wai pipeline validate`** — run full gate validation before advancing to next step
- **Oracle system** — LLM-powered validation with workspace scaffolding and structured prompts
- **Built-in gate templates** — reusable gate definitions for common validation patterns
- **Step-level artifact tagging** — pipeline steps can declare required artifact types
- **Metadata parsing** — structured gate metadata with managed block integration
- **Pipeline doctor checks** — `wai doctor` validates pipeline gate configurations

#### Unified Project Resolution
- **`wai project use <name>`** — session-scoped project binding via `WAI_PROJECT` env var
- **`WAI_PROJECT` environment variable** — set active project without `--project` flag on every command
- **Resolution source indicator** — `wai phase show` displays how the project was resolved (flag, env, default)
- **Unified `resolve_project()`** — all commands now use consistent project resolution logic
- **Doctor checks** — `wai doctor` validates `WAI_PROJECT` configuration

#### Review Artifacts
- **Review artifact type** — `wai add review` with structured frontmatter for capturing review findings

### Fixed
- **Path traversal in reviews** — `--reviews` target now rejects path traversal attempts
- **Ro5 review findings** — addressed issues found during Rule of 5 quality pass

### Changed
- **Refactored project resolution** — migrated all commands from ad-hoc resolution to unified `resolve_project()`

---

## [2026.3.5] - 2026-03-24

### Added
- **Shell linting check in `wai way`** — detects shell scripts (`.sh`/`.bash` files in root, `scripts/`, `bin/`) and GitHub Actions workflows, then checks for `actionlint` and `shellcheck` availability. Recommends the appropriate tool based on what's present — actionlint for workflow YAML with embedded `run:` blocks, shellcheck for standalone scripts.

---

## [2026.3.4] - 2026-03-20

### Added
- **Interactive topic guides** — `wai way <topic>` prints an LLM facilitation guide that instructs an AI assistant to walk the user through setting up a specific aspect of their repository interactively, one decision at a time. 9 topics available: `ai`, `ci`, `coverage`, `devxp`, `docs`, `gh`, `hooks`, `issues`, `specs`. Each guide detects current repo state, provides a TL;DR for fast LLM orientation, and includes structured discussion topics with trade-offs and guidelines.

---

## [2026.3.3] - 2026-03-14

### Added
- **Typos and vale checks in `wai way`** — spell checking (`typos`) and prose linting (`vale`) are now included in the repo hygiene check matrix
- **Lefthook git hooks** — added `lefthook.yml` for local CI checks (format, lint)
- **LLM discoverability** — added `llms.txt` for machine-readable project context
- **LLM authorship disclaimer** — added AI attribution notice to README and docs

### Fixed
- **Test TTY hang** — set `no_input` context in unmanaged directory test to avoid blocking on TTY
- **Reflect test** — updated to use `ReflectArgs` struct
- **Formatting** — applied `cargo fmt` to resolve style drift

---

## [2026.3.2] - 2026-03-06

### Added
- **Safety check for symlink directory removal** — `wai sync` now checks if a target directory contains unmanaged data before removing it; prompts in interactive mode, fails in non-interactive
- **Plugin execution timeouts** — external plugin commands are now terminated after 30 seconds to prevent indefinite hangs

### Fixed
- **Atomic state file writes** — `state.yml` is now written via temp file + rename to prevent corruption on concurrent access
- **LLM context character budget** — `wai why` enforces a 100,000-character limit on artifact content added to prompts, truncating with a clear marker

### Changed
- **Deprecated `[why]` config section** — `[why]` in `.wai/config.toml` is deprecated in favour of `[llm]`; a warning is printed at runtime and the legacy write path has been removed
- **Refactored `search` and `reflect` commands** — internal argument structs replace bare parameter lists (fixes `too_many_arguments` Clippy warning)
- **Clippy cleanup** — resolved 22 warnings across 10 files (collapsible `if`, redundant closures, doc comment indentation)

---

## [2026.3.1] - 2026-03-05

### Added

#### Issue Linkage
- **`--bead` flag** — `wai add research/plan/design --bead <id>` links an artifact to a beads issue ID via YAML frontmatter

#### Init
- **Git auto-commit** — `wai init` automatically stages and commits `.wai/` when inside a git repo (best-effort; silent on failure)

#### Pipeline Refactor
- **`wai pipeline init <name>`** — scaffold a new TOML pipeline definition
- **`wai pipeline start <name> --topic=<slug>`** — start a run; writes run ID to `.wai/.pipeline-run` so `wai add` picks it up without `WAI_PIPELINE_RUN`
- **`wai pipeline next`** — advance to the next step in the active run
- **`wai pipeline current`** — show the current step of the active run
- **`wai pipeline suggest "<query>"`** — get a skill suggestion for a topic
- Removed deprecated `pipeline create/run/advance` commands

#### Beads Memories Integration
- `wai reflect --save-memories` — extract top-level bullets and persist each as a bd memory
- `bd memories` context surfaced in `wai status`, `wai prime`, and `wai handoff`

#### Workspace
- Plugin definitions migrated from YAML to TOML (`.wai/plugins/*.toml`)
- `valid_patterns` derived from the CLI struct — no manual maintenance required

### Fixed
- `wai close` and `wai reflect` are now idempotent
- `wai move` falls back to copy+delete on cross-device rename
- `wai add` uses `create_dir_all` before writing artifacts
- `wai way` prek hook detection handles `core.hooksPath` and tool conflicts
- Non-TTY multi-project resolve now errors cleanly
- `wai doctor` suppresses projection warnings when projections are deliberately empty
- `tool_commit` in config only updated during `wai init`, not every invocation
- Error messages no longer have whimsical prefixes
- `wai ls` adds timeout, progress indicator, and parallel call cap
- `wai reflect` handles AgentBackend sentinel correctly

### Documentation
- Pipeline section updated to reflect new `init/start/next/current/suggest` API
- `--bead` flag documented on all `wai add` subcommands

---

## [2026.2.1] - 2026-02-25

### Added

#### Pipeline Workflows
- **Pipeline Resource** — `wai pipeline create/run/advance/status/list` for ordered multi-skill workflows
- **Auto-tagging** — `wai add` commands auto-inject `pipeline-run:<id>` tag when `WAI_PIPELINE_RUN` env var is set
- Pipeline state persisted as YAML in `.wai/resources/pipelines/`

#### Skills & Resources
- **Hierarchical Skill Names** — `category/action` format (e.g. `issue/gather`) with full path support
- **Skill Templates** — `wai resource add skill --template=gather|create|tdd|rule-of-5` built-in starters
- **Global Skill Library** — `wai resource install --global` / `--from-repo` for cross-project sharing
- **Skill Export/Import** — `wai resource export` and `wai resource import archive` for tar.gz archives

#### Sync & Agent Config
- **Claude Code Projection** — `target: claude-code` built-in translates wai skills to Claude Code slash commands
- **Copy Sync Strategy** — new `copy` strategy alongside symlink/inline/reference
- **Sync Dry-run** — `wai sync --dry-run` previews changes without writing

#### Search & Artifacts
- **Tag Filtering** — `wai search --tag <tag>` filters by YAML frontmatter tags
- **Latest Filter** — `wai search --latest` returns only the most recent match
- **Tags on Plans/Designs** — `--tags` flag added to `wai add plan` and `wai add design`

#### Session Management
- **OpenSpec in Checklist** — `wai close` session-close checklist now includes openspec tasks.md step
- **Cross-tool Tracking** — managed block tracks beads + openspec state across sessions

### Fixed
- 32 test failures after way-agnostic rename and workspace changes
- Symlink sync strategy handling
- Non-TTY multi-project resolve now errors cleanly

### Documentation
- `pipeline` added to `wai --help` COMMANDS, per-command help, and `wai -vv` env vars
- `WAI_PIPELINE_RUN` documented in `wai add --help` and `wai pipeline run --help`
- `wai way` checks refactored to tool-agnostic capability names

---

## [2026.2.0] - 2026-02-20

### Added

#### Core Features
- **Tutorial Mode** - Interactive quickstart guide with `wai tutorial`
- **Doctor Command** - Comprehensive workspace health checks with auto-fix capability
- **Way Command** - Repository best practices checker with AI-friendly development recommendations
- **Resource Management** - Command structure for skills/rules/context (implementation in progress)
- **External Command Pass-through** - Direct plugin command execution (e.g., `wai beads list`)

#### Search & Discovery
- **Advanced Search** - Regex support, type filtering, project scoping, result limiting
- **Timeline Filtering** - Date range filtering with `--from` and `--to` flags
- **Timeline Ordering** - Reverse chronological order with `--reverse`

#### Agent Configuration
- **Config Edit** - Edit agent configs in `$EDITOR` with `wai config edit`
- **Sync Preview** - Check sync status with `wai sync --status` before applying
- **Three Sync Strategies** - Symlink, inline, and reference projection strategies

#### Plugin System
- **Plugin Lifecycle** - Enable/disable plugins with `wai plugin enable/disable`
- **Plugin Hooks** - Three hook types (on_status, on_handoff_generate, on_phase_transition)
- **Custom Plugins** - YAML-based plugin definitions in `.wai/plugins/`

#### Workflow Features
- **Workflow Detection** - Four pattern types with context-aware suggestions
- **Phase History** - Complete transition tracking with timestamps
- **Artifact Tags** - YAML frontmatter tagging for research artifacts
- **Safe Mode** - Read-only operation mode with `--safe` flag

#### Output & Integration
- **JSON Output** - Machine-readable output for all major commands
- **Global Flags** - `--json`, `--no-input`, `--yes`, `--safe` for automation
- **Error Recovery** - Self-healing error messages with actionable suggestions

### Enhanced
- **Status Command** - Now includes plugin context, OpenSpec integration, and suggestions
- **Handoff Generation** - Includes plugin context via hooks (git status, open issues, etc.)
- **Phase Management** - Added phase history tracking and visualization
- **Plugin Integration** - OpenSpec progress tracking in status output

### Documentation
- Complete documentation overhaul with 7 major guides
- Real-world workflow examples
- JSON output integration examples
- Troubleshooting guides
- Advanced features documentation

### Technical
- Miette-based error handling with diagnostics
- Managed block injection for AGENTS.md and CLAUDE.md
- Comprehensive doctor validation checks
- Plugin hook system architecture

## [Previous Versions]

Earlier versions focused on core PARA structure, basic artifact management, and phase tracking. See git history for details.

---

## Upgrade Guide

### From Earlier Versions

If you're upgrading from an earlier version of wai:

1. **Backup your workspace:**
   ```bash
   cp -r .wai .wai.backup
   ```

2. **Run doctor to check for issues:**
   ```bash
   wai doctor
   ```

3. **Auto-fix common issues:**
   ```bash
   wai doctor --fix
   ```

4. **Review new features:**
   ```bash
   wai tutorial  # Run interactive tutorial
   wai --help -v # View all commands
   ```

### Breaking Changes

**None in 2026.2.0** - All changes are backwards compatible.

### Deprecations

**None** - No features have been deprecated in this release.

### New Recommended Practices

1. **Use `wai doctor` regularly** - Catches sync issues and workspace problems early
2. **Leverage JSON output** - Enables powerful automation and integration
3. **Try workflow detection** - `wai status` now provides context-aware suggestions
4. **Use safe mode for exploration** - `--safe` prevents accidental modifications

---

## Future Plans

### Planned Features
- Project templates (--template flag reserved)
- Full resource management implementation
- Workflow customization in config
- Enhanced plugin discovery
- Performance optimizations

### Under Consideration
- Multi-workspace support
- Remote handoff sync
- AI-assisted artifact summarization
- Integration with more dev tools

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on contributing to wai.

## License

MIT License - See [LICENSE](LICENSE) for details.
