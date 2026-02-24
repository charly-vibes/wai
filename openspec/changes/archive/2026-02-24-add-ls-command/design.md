## Context

`wai ls` needs to efficiently scan a potentially large filesystem subtree for wai
workspaces, read per-project state, and optionally invoke beads per workspace. The
key design questions are: traversal strategy, symlink safety, counts column model,
and duplicate name disambiguation.

## Goals / Non-Goals

- Goals: zero-setup discovery, correct output, safe traversal, fast enough for typical layouts
- Non-Goals: caching (deferred); global registry; real-time watching; recursive workspace nesting

## Decisions

### Traversal library and symlink safety

**Decision**: Use the `walkdir` crate (already in use in the wai codebase) with
`follow_links(false)`. For each visited directory, check if `.wai/config.toml` exists
within it. Do not descend into `.wai/` itself. Skip all other hidden directories
(starting with `.`) during traversal.

**Why**: `walkdir` with `follow_links(false)` prevents infinite loops from symlinked
directories, which are common in monorepo setups (e.g., `~/dev/current -> ~/dev/proj`).
Depth 3 covers `~/dev/org/repo` and similar two-tier layouts without runaway traversal.

**The skip rule precisely**: during traversal, if the current entry is a directory and
its name starts with `.` AND it is not being checked as a potential workspace marker,
skip it. The check for `.wai/config.toml` is a file existence test, not a descent.

**Alternatives considered**:
- Walk home directory without depth limit: too slow; could scan thousands of directories.
- Global registry in `~/.wai/config.toml`: requires users to register repos manually;
  zero-setup goal violated.

### Counts column model

**Decision**: Global toggle. The counts column is either shown for ALL rows (when any
workspace has beads data) or absent entirely (when no workspace has beads). Rows without
beads data show a blank cell.

**Why**: A per-row toggle produces ragged columns that are harder to read and mentally
parse. A global toggle produces a consistent table. The blank-cell approach for non-beads
rows still communicates absence clearly.

**Alternatives considered**:
- Per-row toggle (each line independently shows or omits counts): inconsistent column
  widths; rejected.
- Always show counts, `—` for absent: implies a failure rather than absence of beads.

### Duplicate name disambiguation

**Decision**: When two projects from different workspaces share the same name, append a
short path suffix to each: `name (~/path/to/repo)`. The suffix uses `~` for home
directory prefix when applicable.

**Why**: Name collisions are probable for common project names like `api`, `web`, `main`.
Without disambiguation, the output is ambiguous and unusable. The path suffix is the
minimum information needed to disambiguate.

**Detection**: collect all (name, workspace_path) pairs; find names that appear more
than once; apply suffix to affected rows only. Unambiguous names are not modified.

### beads count fetching

**Decision**: Invoke `bd stats --json` per workspace when `.beads/` is present. Parse
`open` and `ready` from the JSON. Skip gracefully if `bd` exits non-zero or is not in
PATH.

**Why**: `bd stats --json` is the stable machine-readable interface. Parsing `.beads/`
files directly would couple wai to beads' internal format, which is not a public API.

## Risks / Trade-offs

- Performance on large home directories: depth=3 with hidden-dir skipping is fast in
  practice but untested on extreme layouts (e.g., home dirs with thousands of top-level
  entries). Mitigation: add a `--depth 1` escape hatch for users with slow scans.
- `bd stats --json` schema stability: if the `open`/`ready` field names change, counts
  silently disappear. Mitigation: silent-skip is the correct fallback.

## Open Questions

- None blocking implementation.
