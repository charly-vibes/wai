# Feedback Loop Design: stale wai decision artifacts

## Context

Wai artifacts capture why a change was made, but today they do not know when the code they describe has drifted. The codebase already has adjacent patterns we can reuse:

- `src/commands/doctor.rs` compares generated vs actual content for managed-block staleness.
- `src/commands/doctor.rs` verifies artifact locks with SHA-256 sidecars.
- `src/commands/pipeline.rs` writes sidecar lock files next to artifacts.
- `src/commands/sync.rs` uses mtime as a cheap freshness precheck.
- `src/commands/status.rs` is where wai surfaces “what should I do next?” signals.
- `src/commands/add.rs` already owns artifact frontmatter creation.

The goal here is not spec/runtime drift detection. That belongs to espectacular. This feature covers **decision freshness**: does a wai artifact still describe the code it points at?

## Goals

- Detect when code tracked by a wai artifact has changed since that artifact was last re-evaluated.
- Surface that staleness in a place agents already check during normal work.
- Keep the first implementation small enough to prove the feedback loop end to end.
- Reuse existing wai concepts: frontmatter, sidecars, `wai status`, and `wai doctor`-style comparison logic.

## Non-goals

- Semantic analysis of whether the decision is still correct.
- Replacing espectacular’s spec-to-code drift checks.
- Perfect automatic mapping from prose to code.
- Mandatory git-hook installation in the first version.

## Decisions

### 1. Map artifacts to code with explicit frontmatter paths

Use an explicit frontmatter field on decision artifacts:

```yaml
---
tracks:
  - src/commands/status.rs
  - src/commands/doctor.rs
  - docs/src/concepts/plugins.md
decision_point: status-freshness
---
```

`tracks` entries are repo-relative files or globs. `decision_point` is optional and lets multiple artifacts roll up to one higher-level concern later.

Why this approach:

- **Explicit beats implicit** for “why” artifacts. The author knows what the artifact is about.
- **Convention-only mapping** is too weak. Folder names and tags do not reliably identify affected code.
- **Content hashing alone** can detect that something changed, but not *what* the artifact is responsible for.
- It fits naturally into `wai add`, which already writes artifact frontmatter.

First iteration: only explicit file/glob tracking. Symbol-level tracking can be a future enhancement.

### 2. Detect drift with a freshness sidecar, using mtime first and hash second

For each artifact with `tracks`, store a sidecar snapshot of the tracked paths, similar to artifact locks:

```toml
artifact = "2026-05-12-status-design.md"
verified_at = "2026-05-12T14:00:00Z"
tracked = [
  { path = "src/commands/status.rs", mtime = 1747060000, hash = "sha256:..." },
  { path = "src/commands/doctor.rs", mtime = 1747060100, hash = "sha256:..." },
]
```

Comparison rule:

1. Expand `tracks` to concrete files.
2. Compare mtimes first for a cheap prefilter.
3. If mtime differs, recompute the normalized SHA-256 hash.
4. If the hash changed, mark the artifact stale.

Why this approach:

- Reuses the existing lock/hash pattern already present in pipeline artifacts.
- Avoids relying on git history, which can be absent or misleading during uncommitted work.
- Keeps scans cheap for unchanged files by borrowing the `sync.rs` mtime shortcut.

### 3. Surface staleness in `wai status`, with an optional detailed report command

Primary UX: enrich `wai status`.

Example shape:

- a plugin-info-style summary such as `decision freshness: 3 stale artifacts`
- project suggestions such as `Re-evaluate 2026-05-12-status-design.md (tracked code changed)`

Why `wai status`:

- `wai status` already answers “what should I do next?”
- stale decision artifacts are workflow guidance, not structural corruption
- this matches the earlier choice to surface stale phases in `status`, not `doctor`

For deeper inspection, add a dedicated report command in the implementation phase, e.g. `wai artifacts stale --json`, so CI and scripts can consume the same signal without scraping human output.

### 4. Granularity: detect per artifact, roll up by decision point later

The base unit is the **artifact**.

Why:

- It matches wai storage and frontmatter today.
- It keeps the first tracer bullet simple.
- It avoids inventing a new persistence model before we prove value.

`decision_point` is included now so multiple artifacts can later roll up into one higher-level stale signal, but the first implementation should only require per-artifact freshness.

### 5. Trigger strategy: status/manual first, hooks later

First implementation should trigger checks in these places:

- `wai status`
- dedicated report command (`wai artifacts stale` or similar)
- optionally `wai sync` as a non-blocking note if stale artifacts are found

Git hooks should be a follow-up, not part of the tracer bullet.

Why:

- It proves the feedback loop without forcing repository hook setup.
- It keeps ownership inside wai rather than depending on external hook managers.
- It lets the feature mature before turning it into an always-on interruption.

Later, hook integration can call the report command and cache changed paths for faster scans.

## Data model

### Artifact frontmatter

New optional fields:

- `tracks: [<path-or-glob>, ...]`
- `decision_point: <slug>`

Artifacts without `tracks` are ignored by freshness checks.

### Freshness sidecar

One sidecar per tracked artifact, stored next to the artifact.

Suggested suffix:

- `<artifact>.fresh.lock`

This mirrors artifact locks conceptually while keeping purpose distinct.

## Workflow

1. Author creates or updates a research/design/plan artifact with `tracks`.
2. Wai writes or refreshes the freshness sidecar.
3. Later, tracked code changes.
4. `wai status` or `wai artifacts stale` compares current files to the sidecar.
5. If any tracked file hash differs, the artifact is stale.
6. Human or agent re-evaluates the artifact, updates it or adds a correction artifact, and refreshes the sidecar.

## Tracer-bullet implementation plan

### Phase 1: prove the loop

- Parse `tracks` and `decision_point` from artifact frontmatter.
- Add sidecar read/write helpers.
- Implement a freshness scanner over `.wai/projects/**/{research,design,plans}`.
- Surface a summary in `wai status`.
- Add a machine-readable command for stale-artifact reporting.
- Add tests for tracked file change → stale signal.

### Phase 2: make authoring easier

- Add `wai add research|design|plan --tracks <path>[,<path>...]`.
- Add a refresh command for re-baselining after review.
- Improve messages to suggest the exact artifact to revisit.

### Phase 3: optional automation

- Hook integration.
- Decision-point rollups.
- CI examples.
- Symbol-level mapping.

## Risks and trade-offs

- **Manual mapping burden**: authors must declare `tracks`. Acceptable for v1; silent implicit mapping would be worse.
- **False positives**: refactors may change tracked files without invalidating the decision. Non-blocking status warnings are the right response.
- **Glob expansion drift**: broad globs may become noisy. Document that narrower paths are preferred.
- **Sidecar churn**: freshness state adds files. This is consistent with existing lock-file patterns.

## Open questions

- Should re-evaluation always update the original artifact, or should locked decision artifacts require addenda similar to pipeline artifacts?
- Should freshness scanning include `reviews/` and `handoffs/`, or only long-lived decision artifacts (`research`, `design`, `plan`)?
- Should `wai sync` merely report stale artifacts or fail in a strict mode?

## Recommendation

Build the tracer bullet around:

- explicit `tracks` frontmatter
- per-artifact freshness sidecars
- `wai status` surfacing
- a machine-readable stale-artifact report command

That is the smallest design that proves the feedback loop end to end without overlapping espectacular or overcommitting to automation too early.
