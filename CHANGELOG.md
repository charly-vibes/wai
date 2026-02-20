# Changelog

All notable changes to wai will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Calendar Versioning](https://calver.org/) (YYYY.M.MICRO).

## [2026.2.0] - 2026-02-20

### Added

#### Core Features
- **Tutorial Mode** - Interactive quickstart guide with `wai tutorial`
- **Doctor Command** - Comprehensive workspace health checks with auto-fix capability
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
