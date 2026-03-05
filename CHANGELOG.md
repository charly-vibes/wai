# Changelog

All notable changes to wai will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Calendar Versioning](https://calver.org/) (YYYY.M.MICRO).

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
