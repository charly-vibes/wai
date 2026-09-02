# Cartography — micro trace: `wai sync`

**Entry zoom level:** micro (functionality trace)
**Scope covered:** `src/commands/sync.rs` (344) + `src/sync_core.rs` (1,102)

## Entry point

`pub fn run(status_only: bool, dry_run: bool, from_main: bool)` — `src/commands/sync.rs:94`

## Call chain

```
 1. sync::run(...)                   commands/sync.rs:94
 2. require_project()                config                     [I/O: fs]
 3. ── from_main? ── run_from_main() commands/sync.rs:235       [I/O: git checkout main first]
 4. read .projections.yml            commands/sync.rs:101-110   [I/O: fs; serde_yml parse]
 5. empty-config early exit          commands/sync.rs:116-127
 6. ── status_only? ── claude_code_needs_sync()  commands/sync.rs:25  [I/O: fs mtime compare]
 7. dispatch per projection target:
      claude-code     → sync_core::execute_claude_code()        [I/O: fs symlink/write]
      agents-projection → sync_core::execute_agents_projection() [I/O: fs symlink/write]
      strategy match  → sync_core::execute_symlink / inline /
                        reference / copy   sync.rs:212-215      [I/O: fs symlink/write]
```

## Data flow

```
.wai/resources/agent-config/.projections.yml (declarative: target, strategy, sources)
  → ProjectionsConfig (parsed in sync.rs itself)
  → per-projection: source files under agent-config dir
  → sync_core strategies create/refresh symlinks or write inlined/referenced copies
  → projected artifacts land in .claude/commands/ (claude-code) or AGENTS.md targets
```

## Structural observations

- `sync.rs` defines its own `ProjectionsConfig` struct (`sync.rs:14`) while `doctor/checks_sync.rs` defines a second, parallel one (`checks_sync.rs:13`) — the projection schema has 2 parsers + 3 modeling sites (with `sync_core::Projection`)
- `claude_code_needs_sync` (`sync.rs:21` doc) deliberately mirrors `sync_core::execute_claude_code` traversal logic — a documented, hand-maintained parallel implementation
- `doctor/checks_sync.rs` verifies health by re-issuing `sync_core::execute_*` calls as dry-run probes (`checks_sync.rs:130-140`) — sync core doubles as the check oracle
- Strategy set is closed: symlink | inline | reference | copy, matched in both `sync.rs` and `checks_sync.rs`
- Core helpers are private (`check_unmanaged_dir`, `symlink_file_up_to_date`, `sync_core.rs:42-113`); public surface is the `execute_*` family

## Adjacent levels offered

Micro trace of a single strategy (`execute_symlink` up-to-date checks) or the doctor check that dry-runs it.
