# Tasks: Decision Matrix

## 1. Core Scaffold

- [ ] 1.1 `wai matrix init`: prompt for/accept the problem statement, create
      `problem.md`, `criteria/`, `approaches/01-status-quo/` (with
      `_description.md`), and empty `decision.md`
- [ ] 1.2 Enforce `01-status-quo` as first column on init
- [ ] 1.3 `wai matrix criterion add <name>`: create `criteria/NN-name.md` and
      cell dirs in every existing approach (enforce rectangularity on creation)
- [ ] 1.4 `wai matrix approach add <name>`: create `approaches/NN-name/` with
      `_description.md` and empty cell dirs for all existing criteria

## 2. Cell Model (file-first)

- [ ] 2.1 Define and document the cell encoding: `fact.md` + exactly one
      marker of {`neutral`, `green`, `yellow`, `red`}
- [ ] 2.2 Define incompleteness as missing/empty `fact.md` (never a color);
      document that direct file I/O is the primary editing path
- [ ] 2.3 `?`-question convention documented (no code — `wai search` handles it)

## 3. Deciding

- [ ] 3.1 `wai matrix decide <approach> <rationale>`: validate the approach
      directory exists, write `decision.md` (approach, rationale, date,
      pointer to design doc)
- [ ] 3.2 Auto-scaffold `designs/<date>-<slug>.md`: standard frontmatter
      (tags, `tracks:`), rationale, trade-offs, matrix link, decision-time
      snapshot of the winning column's aspect texts
- [ ] 3.3 `wai matrix decide` verifies the selected approach exists; error
      messages suggest fixes (Self-Healing Errors)

## 4. Renderer

- [ ] 4.1 Deterministic renderer: `problem.md` banner → criteria rows →
      approach columns → cells (`fact.md` text + judgment chip)
- [ ] 4.2 Empty cells render as gray "not yet assessed" placeholder — never a
      judgment color
- [ ] 4.3 `neutral` renders as a filled cell with a clear/neutral chip
- [ ] 4.4 Self-contained output (inline CSS, no JS); assessment key baked in
- [ ] 4.5 Snapshot tests: same directory state → byte-identical output
- [ ] 4.6 Output written on demand only; documented as never-committed view

## 5. Validation & Lints

Structural — errors (non-zero exit):

- [ ] 5.1 Rectangularity check (criterion subdirs identical across approaches)
- [ ] 5.2 Marker checks: exactly one per filled cell, empty, ∈ the four colors
- [ ] 5.3 Empty/missing `fact.md` detection
- [ ] 5.4 Status-quo-first check

Methodology — warnings (never block):

- [ ] 5.5 All-green column detection ("are you rationalizing?")
- [ ] 5.6 Undistinguished columns (identical cell texts across approaches)
- [ ] 5.7 Judgment-in-text heuristic (stoplist: "good", "bad", "better", ...)
- [ ] 5.8 Link-only cell detection
- [ ] 5.9 Criteria-phrased-as-questions detection
- [ ] 5.10 Empty `problem.md` warning
- [ ] 5.11 Stale-decision detection (matrix files newer than `decision.md`)

## 6. Workflow Integration

- [ ] 6.1 Phase gate `design → plan`: requires `decision.md` newer than every
      file under `approaches/` and `criteria/`; self-healing failure message
      naming the offending files and `wai matrix decide`
- [ ] 6.2 `wai status` matrix awareness during design phase: current problem
      statement, cell completion count, next unfilled cell suggestion
- [ ] 6.3 `wai close` handoff includes problem statement, decision + rationale,
      links to matrix directory and design doc

## 7. Documentation

- [ ] 7.1 New Concepts page: "Decision Matrix" — grounded in the
      Design in Practice methodology (cite the talks), covering A1, the
      fact/judgment split, and the question (`?`) convention
- [ ] 7.2 Commands Reference entries for the `matrix` command group
- [ ] 7.3 Quick Start + tutorial update with a matrix example
- [ ] 7.4 Adoption note: opt-in via `wai matrix init`; no-matrix projects
      unchanged

## 8. Testing & Release

- [ ] 8.1 Integration tests: agent-style direct file I/O (no CLI) producing a
      valid lint pass and render
- [ ] 8.2 Integration tests: phase gate pass/fail incl. scaffolded (undecided)
      `decision.md`, edited-after-decision, and fresh-clone ambiguous-timestamp
      warning cases
- [ ] 8.3 Integration tests: `decide` scaffolds design doc with frontmatter
- [ ] 8.4 Snapshot tests for byte-identical renderer output
- [ ] 8.5 Snapshot tests for byte-identical renderer output
- [ ] 8.6 Lint coverage: status-quo-without-red and decided-with-unfilled-cells
      warnings
- [ ] 8.7 CHANGELOG entry + release notes
