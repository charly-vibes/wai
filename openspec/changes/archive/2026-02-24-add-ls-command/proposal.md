# Change: Add `wai ls` cross-project global view

## Why

`wai status` is per-workspace. There is no way to see all projects using wai across
repos and their current phase or open issue count at a glance. From user behavior
research: this is a genuine gap (Gap E) with no existing equivalent. `wai ls` fills it
with a simple filesystem scan that produces a one-line summary per project.

## What Changes

- Adds a new `wai ls` top-level command
- Scans for `.wai/config.toml` files under a root directory (default: `$HOME`) up to 3
  levels deep
- For each discovered workspace, reads its project list and per-project phase
- Shows beads open/ready counts as a global column when any workspace has beads detected; the column is omitted entirely when no workspace has beads data
- Accepts `--root <path>` to override the scan root
- Accepts `--depth <n>` to override the scan depth (default: 3)

Expected output:

```
why-command   [review]    3 open, 2 ready
para          [plan]      7 open, 0 ready
rizomas       [implement] 1 open, 1 ready
```

## Design Decisions

**Scan strategy**: walk from a configurable root (default `$HOME`) up to `--depth`
(default 3). No global registry required — keeps `wai ls` zero-setup. Hidden directories
are skipped except `.wai/` itself. Depth 3 covers `~/dev/org/repo` patterns without
runaway traversal.

**Performance**: depth=3 is cheap in practice. Caching is deferred to a follow-on;
the MVP is fast enough for typical layouts (< 100ms on an SSD with ~50 repos at depth 3).

**Counts**: beads counts are fetched per-workspace via `bd stats --json` when `.beads/`
is present. If `bd` is not installed or fails, that workspace's cell is left blank. The
counts column itself is only shown when at least one workspace has beads data; when no
workspace has beads, the column is omitted entirely (cleaner output for non-beads setups).

**Output**: one line per (workspace, project) pair. If a workspace has multiple projects,
each is shown as a separate line. Column widths align to the longest name in the output.
When two projects share the same name (from different workspaces), a short path suffix
disambiguates: `name (~/path/to/repo)`.

**Symlink safety**: the filesystem walker uses `follow_links: false` to prevent infinite
loops from symlinked directories, which are common in monorepo setups.

## Impact

- Affected specs: `cli-core`
- Affected code: `src/cli.rs`, `src/commands/ls.rs` (new), `src/commands/mod.rs`
- Runtime dependency: `dirs` crate (confirmed in `Cargo.toml`)
- No breaking changes
