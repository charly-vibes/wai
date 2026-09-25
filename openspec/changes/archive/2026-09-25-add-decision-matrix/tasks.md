# Tasks: Decision Matrix

## 1. Core Scaffold

- [x] 1.1 `wai matrix init`: prompt for/accept the problem statement, create
      `problem.md`, `criteria/`, `approaches/01-status-quo/` (with
      `_description.md`), and empty `decision.md`
- [x] 1.2 Enforce `01-status-quo` as first column on init
- [x] 1.3 `wai matrix criterion add <name>`: create `criteria/NN-name.md` and
      cell dirs in every existing approach (enforce rectangularity on creation)
- [x] 1.4 `wai matrix approach add <name>`: create `approaches/NN-name/` with
      `_description.md` and empty cell dirs for all existing criteria

## 2. Cell Model (file-first)

- [x] 2.1 Define and document the cell encoding: `fact.md` + exactly one
      marker of {`neutral`, `green`, `yellow`, `red`}
- [x] 2.2 Define incompleteness as missing/empty `fact.md` (never a color);
      document that direct file I/O is the primary editing path
- [x] 2.3 `?`-question convention documented (no code — `wai search` handles it)

## 3. Deciding

- [x] 3.1 `wai matrix decide <approach> <rationale>`: validate the approach
      directory exists, write `decision.md` (approach, rationale, date,
      pointer to design doc)
- [x] 3.2 Auto-scaffold `designs/<date>-<slug>.md`: standard frontmatter
      (tags, `tracks:`), rationale, trade-offs, matrix link, decision-time
      snapshot of the winning column's aspect texts
- [x] 3.3 `wai matrix decide` verifies the selected approach exists; error
      messages suggest fixes (Self-Healing Errors)

## 4. Renderer

- [x] 4.1 Deterministic renderer: `problem.md` banner → criteria rows →
      approach columns → cells (`fact.md` text + judgment chip)
- [x] 4.2 Empty cells render as gray "not yet assessed" placeholder — never a
      judgment color
- [x] 4.3 `neutral` renders as a filled cell with a clear/neutral chip
- [x] 4.4 Self-contained output (inline CSS, no JS); assessment key baked in
- [x] 4.5 Snapshot tests: same directory state → byte-identical output
- [x] 4.6 Output written on demand only; documented as never-committed view

## 5. Validation & Lints

Structural — errors (non-zero exit):

- [x] 5.1 Rectangularity check (criterion subdirs identical across approaches)
- [x] 5.2 Marker checks: exactly one per filled cell, empty, ∈ the four colors
- [x] 5.3 Empty/missing `fact.md` detection
- [x] 5.4 Status-quo-first check

Methodology — warnings (never block):

- [x] 5.5 All-green column detection ("are you rationalizing?")
- [x] 5.6 Undistinguished columns (identical cell texts across approaches)
- [x] 5.7 Judgment-in-text heuristic (stoplist: "good", "bad", "better", ...)
- [x] 5.8 Link-only cell detection
- [x] 5.9 Criteria-phrased-as-questions detection
- [x] 5.10 Empty `problem.md` warning
- [x] 5.11 Stale-decision detection (matrix files newer than `decision.md`)

## 6. Workflow Integration

- [x] 6.1 Phase gate `design → plan`: requires `decision.md` newer than every
      file under `approaches/` and `criteria/`; self-healing failure message
      naming the offending files and `wai matrix decide`
- [x] 6.2 `wai status` matrix awareness during design phase: current problem
      statement, cell completion count, next unfilled cell suggestion
- [x] 6.3 `wai close` handoff includes problem statement, decision + rationale,
      links to matrix directory and design doc

## 7. Documentation

- [x] 7.1 New Concepts page: "Decision Matrix" — grounded in the
      Design in Practice methodology (cite the talks), covering A1, the
      fact/judgment split, and the question (`?`) convention
- [x] 7.2 Commands Reference entries for the `matrix` command group
- [x] 7.3 Quick Start + tutorial update with a matrix example
- [x] 7.4 Adoption note: opt-in via `wai matrix init`; no-matrix projects
      unchanged

## 8. Testing & Release

- [x] 8.1 Integration tests: agent-style direct file I/O (no CLI) producing a
      valid lint pass and render
- [x] 8.2 Integration tests: phase gate pass/fail incl. scaffolded (undecided)
      `decision.md`, edited-after-decision, and fresh-clone ambiguous-timestamp
      warning cases
- [x] 8.3 Integration tests: `decide` scaffolds design doc with frontmatter
- [x] 8.4 Snapshot tests for byte-identical renderer output
- [x] 8.5 Snapshot tests for byte-identical renderer output
- [x] 8.6 Lint coverage: status-quo-without-red and decided-with-unfilled-cells
      warnings
- [x] 8.7 CHANGELOG entry + release notes
