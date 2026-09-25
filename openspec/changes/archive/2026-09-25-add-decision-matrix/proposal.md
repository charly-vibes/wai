# Change: Add decision matrix to the design phase

## Why

Wai's value proposition is capturing the **why** behind design decisions. Today,
`wai add design "…"` records only the conclusion — the alternatives considered,
the criteria that mattered, and the trade-offs accepted are lost unless someone
documents them by hand.

The decision matrix from the Design in Practice methodology (Rich Hickey,
"Design in Practice"; Alex Miller, "Design in Practice in Practice") is the
artifact that fixes this: approaches as columns, criteria as rows, facts in
cells, subjective judgment in color. It is a **live deliberation surface** —
the human/agent pair works in it during the design session, contrast between
approaches drives thinking, new approaches are *born* in it, and it grows over
time as methods, criteria, and solutions appear.

Wai currently has no first-class representation of this artifact, and a markdown
table is the worst possible container for it: one misaligned pipe corrupts the
grid, diffs are unreadable, merges conflict, and programmatic access requires
fragile scraping.

## What Changes

Introduce the **decision matrix**: a plain directory tree under
`.wai/projects/<project>/designs/matrix/` where approaches are directories,
criteria are subdirectories, each cell is a `fact.md` file plus one judgment
marker file, and `problem.md` anchors the matrix to the decision being made.
The filesystem is the source of truth; an HTML render is a generated,
on-demand view for human comprehension.

New `wai matrix` command group (`init`, `criterion add`, `approach add`,
`decide`, `lint`, `render`), integrated into the design phase, `wai status`,
and the `design → plan` phase gate.

Key properties:

- **One matrix per project.** It is a standing surface that is re-anchored
  (via `problem.md`) as new decisions arise, and it grows over the project's
  lifetime.
- **Text is FACT, color is JUDGMENT.** Cell text is factual description;
  subjectivity lives only in one of four explicit marker files:
  `neutral`, `green`, `yellow`, `red`.
- **The CLI is sugar.** Every operation is achievable with plain file I/O;
  direct file editing is always equivalent and always supported.
- **Complement, not replace.** `wai matrix decide` auto-scaffolds a design doc
  in `designs/` — the existing artifact taxonomy (search, freshness sidecars,
  handoffs) remains the system of record for narrative decisions.

Full schema, commands, lints, and workflow integration are in
[`design.md`](./design.md); the implementation checklist is in
[`tasks.md`](./tasks.md).

## Non-Goals (v1)

- No numeric weighting or scoring — the matrix stays qualitative
  (facts + judgment color), per the methodology.
- No custom judgment vocabularies — `neutral`/`green`/`yellow`/`red` only.
- No custom HTML templates — the renderer is fixed and deterministic.
- No committed rendered output — `matrix.html` is a generated view, rendered
  on demand, never a source of truth.
- No multi-matrix support per project. The directory is self-contained, so
  per-decision matrices can be added later if the standing model proves wrong.
- No breaking changes: projects without a matrix behave exactly as before;
  adoption is opt-in via `wai matrix init`.

## Impact

**Affected specs:**

- New capability: `decision-matrix`
- Modified: project phases (staleness-aware gate on `design → plan`),
  `status` command (matrix awareness), `close`/handoff (matrix summary)

**Affected code:**

- `src/commands/` — new `matrix` command group
- `src/` phase/status/close modules — matrix detection + phase gate
- `docs/` — new concept page + command reference updates

**Compatibility:**

- Purely additive and opt-in per project. Projects that never run
  `wai matrix init` behave exactly as before; the phase gate only activates
  for projects with a matrix.

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Structured markdown table | Fragile under LLM edits; unreadable diffs; merge conflicts; programmatic access needs regex scraping. The methodology itself abandoned prose docs: "docs are linear, and do not support contrast" |
| SQLite/JSON blob | Binary or nested-schema files hurt git diffing and agent file I/O; the filesystem is the natural source of truth |
| External tool (Google Sheets, per the source talk) | The methodology's medium of choice, but it violates Context-Aware: fragments the workspace, loses git history, and is not agent-native. Wai reproduces the sheet's *properties* — at-a-glance view, per-cell editing, live contrast — on the filesystem |
| Plugin/extension only | The matrix is tied to phase gates and the design phase; core ensures consistency |

## References

- Design practice framework (decision matrix, Phase 4 — Direction):
  https://charly-vibes.github.io/incitaciones/content/distilled/design-practice.md
- Rich Hickey, "Design in Practice" (transcript: matthiasn/talk-transcripts,
  `Hickey_Rich/DesignInPractice.md`)
- Alex Miller, "Design in Practice in Practice":
  https://www.youtube.com/watch?v=VBnGhQOyTM4 (ASR notes archived at
  `tv/downloads/miller-design-in-practice-in-practice-notes.md`)
- Wai value proposition: capture the *why*, recover full context in <2 min
