## Design: Unifying `wai way` and `wai doctor`

### Context

`wai way` and `wai doctor` both run checks and produce fix recommendations, creating confusion about which to reach for. The partial fix (73286cd) added cross-references in help text but the UX split remains.

### Current Domains (After 73286cd)

| Command | Domain | Requires .wai/ | Checks |
|---------|---------|----------------|--------|
| `wai way` | Repo hygiene & agent conventions | No | task runner, git hooks, editorconfig, docs, AI instructions, llm.txt, skills, gh CLI, CI/CD, devcontainer, releases, test coverage, beads/openspec present |
| `wai doctor` | wai workspace health | Yes | PARA dirs, config.toml, schema version, plugin tools, agent-config projections, project state, custom plugins, CLAUDE.md WAI blocks, README badge, session hook |

The domains are **genuinely distinct** — `way` answers 'is this a well-run repo?' while `doctor` answers 'is the wai workspace correctly configured?'. There is minor logical overlap (both check skills exist vs projected, both check AI instructions exist vs have correct blocks) but not functional duplication.

### Option Analysis

#### Option A: `wai check` umbrella (recommended)

A new `wai check` command runs `way` then `doctor` in sequence, with clear section headers.

```
wai check
  ┌─ Repo hygiene (wai way) ─────────────────────────────
  ✓ Task runner          Justfile found
  ✓ Git hooks            pre-commit present
  ⚠ Test coverage        No coverage config found
  ...
  ┌─ Workspace health (wai doctor) ──────────────────────
  ✓ Directories          PARA structure complete
  ✓ Config               config.toml valid
  ✗ Agent config sync    3 projections out of date
  ...
```

- If no `.wai/` exists: shows way checks, then prints '(workspace checks skipped — run wai init first)'.
- `--way-only` / `--doctor-only` flags for targeted runs.
- `wai way` and `wai doctor` are **preserved** (no breaking change).
- `wai check` becomes the recommended entry point in docs and `wai init` next-steps.

**Pros:** Eliminates user confusion. Single command. Degrades gracefully on no-workspace repos.  
**Cons:** Two-pass output is longer. Exit code logic needs to combine both summaries.

#### Option B: Strict domain partitioning

Audit overlap and move checks to their definitive home:
- Move 'beads/openspec present' from `way` to `doctor` (they're wai ecosystem tools)
- Move 'CLAUDE.md WAI blocks' from `doctor` to `way` (it's about AI instructions in the repo)
- Improve help text to make the boundary unambiguous

**Pros:** Clean separation, no new commands.  
**Cons:** Still two commands. Users still need to know which to run for which concern.

#### Option C: Deprecate one

- Merge `doctor` into `way`: Breaks no-workspace use case — `doctor` checks require .wai/ state.
- Merge `way` into `doctor`: Requires `wai init` before getting repo hygiene feedback; hostile to new users.

**Verdict:** Not viable. The workspace dependency asymmetry prevents clean merger.

### Recommendation

**Implement Option A (`wai check`)**, with Option B's partitioning cleanup bundled in.

Implementation steps:
1. Create `src/commands/check.rs` — calls `way::run_checks()` and `doctor::run_checks()` (refactored to return Vec<CheckResult> without printing)
2. Refactor `way.rs` and `doctor.rs` to expose `run_checks()` functions (presentation stays in their own `run()`)
3. Unify `CheckResult` structs under a shared type (or make `check` re-use the superset: Pass/Info/Warn/Fail)
4. Move beads/openspec checks from `way` to `doctor` (Option B cleanup)
5. Add `wai check` to CLAUDE.md managed block, `wai init` next-steps, and way/doctor help cross-refs

### OpenSpec Decision

This requires a new CLI subcommand and refactoring of two existing command modules. An openspec change proposal is warranted. The task split is:

- openspec change: defines the `wai check` CLI contract, flag semantics, and output format
- beads tickets: one per implementation step above
