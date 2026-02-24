## Context

`wai prime` aggregates output from several existing subsystems into a single view. The
key implementation question is how prime.rs gets plugin status summaries without
duplicating or re-implementing the logic already in `src/commands/status.rs`.

## Goals / Non-Goals

- Goals: single source of truth for plugin status output; prime.rs calls existing functions, not subprocess wai
- Non-Goals: change wai status output format; add new plugin hooks; introduce a new plugin API

## Decisions

### Plugin status hook reuse

**Decision**: Call `plugin::run_hooks(project_root, "on_status")` and
`openspec::read_status(project_root)` directly from `prime.rs`, same as `status.rs` does
at lines 89 and 92. Do not shell out to `wai status` and parse its output.

**Why**: Both functions are in-process library calls. Shelling out to `wai status` would
couple prime to status's text format, add subprocess overhead, and break if status output
changes. The direct call approach is zero-copy, works in tests, and keeps prime's output
format independent.

**Alternatives considered**:
- Shell out to `wai status --json`: not yet implemented; adds an interface contract we'd
  have to maintain.
- Extract a `collect_plugin_status(root) -> Vec<StatusLine>` helper shared by both: valid
  future refactor, but premature for an MVP where prime is the only new caller.

### bd ready integration

**Decision**: Invoke `bd ready --json` as a subprocess, parse the first element's `id`
field, and surface it as `→ Suggested next: bd show <id>`. Skip silently on any error.

**Why**: `bd ready --json` is a stable interface (confirmed in bd help output). JSON
parsing is robust against whitespace/color changes. The first element is the
highest-priority ready issue per bd's own sort order — prime does not re-rank.

**Alternatives considered**:
- Parse `bd ready` plain text: fragile against formatting changes.
- Call a beads library directly: bd is an external binary; no Rust API available.

### Handoff snippet extraction

**Decision**: Return `(NaiveDate, String)` from `read_handoff_summary`. If YAML
frontmatter is missing/invalid, return today's date and an empty snippet (omit the
handoff line). If frontmatter parses but no paragraph is found (all-heading handoff),
return the date with snippet `"no summary yet"`.

**Why**: Graceful degradation is more useful than a hard failure for a display-only
field. The handoff line is informational; failing the whole prime command on a bad
handoff file would frustrate users.

## Risks / Trade-offs

- If `plugin::run_hooks` signature changes, both status.rs and prime.rs need updating.
  Mitigation: acceptable coupling given they live in the same crate.
- If `bd ready --json` schema changes (e.g., `id` renamed), the suggested-next line
  silently disappears. Mitigation: the silent-skip behavior is the correct fallback.

## Open Questions

- None blocking implementation.
