---
date: 2026-05-13
project: qa-round-execution
phase: research
---

# Session Handoff

## What Was Done

- Closed `wai-fvhv.119` with focused `wai why` provider-selection and no-LLM tests.
- Closed `wai-fvhv.118` with focused `wai why` history/fallback and no-artifact tests.
- Closed `wai-fvhv.115` with focused `wai resource` export/import archive tests.

## Key Decisions

- Added targeted tests next to existing `why`/`resource` coverage instead of refactoring command code, because the tickets asked for localized regression coverage.
- Used fake `claude` binaries and temporary git repos to make provider and history tests deterministic.

## Gotchas & Surprises

- `wai resource import archive` reports success text on stderr via the logging helper, not stdout.
- `tar` rejects `..` archive paths before `wai` sees them, so malformed-archive coverage should use a structurally valid tar entry with an invalid `wai` skill path like `evil.txt`.

## What Took Longer Than Expected

- Initial archive-malformation test used `../evil/SKILL.md`, but the tar library rejected it before command execution.
- A first cargo test invocation passed multiple test names incorrectly; reran with one filter per command.

## Open Questions

- Whether to continue with `wai-fvhv.114` next or switch to the remaining `wai plugin` coverage ticket.

## Next Steps

1. Pick up `wai-fvhv.114` for `wai resource` skill-lifecycle coverage or explicitly re-prioritize.
2. Commit and push the new coverage plus tracker/handoff updates.

## Context

### verification

- `cargo test why_ -- --nocapture`
- `cargo test resource_ -- --nocapture`
- targeted new tests for `claude_cli`, `gather_git_file_context`, and archive import/export flows


### git_status

```
 M .beads/backup/backup_state.json
 M .beads/interactions.jsonl
M  .beads/issues.jsonl
 M .wai/pipeline-runs/epic-autonomy-tdd-ro5-2026-05-13-work-one-ready-child-issue-from-epic-wai-fvhv.yml
 M src/commands/why.rs
 M src/llm.rs
 M tests/integration.rs
```

### open_issues

```
○ wai-fvhv ● P1 [epic] QA round: 100+ wai CLI findings across usability, docs, scope, code quality, and test coverage
├── ○ wai-fvhv.101 ● P1 Scope: Restructure documentation IA around beginner, contributor, and integrator audiences
├── ○ wai-fvhv.102 ● P1 Scope: Add an ADR or product map for command taxonomy
├── ○ wai-fvhv.103 ● P1 Scope: Define release criteria that require docs/tests/spec sync before surface-area growth
├── ○ wai-fvhv.111 ● P1 Tests: add focused coverage for `wai plugin` management and passthrough behavior
├── ○ wai-fvhv.25 ● P1 Tests: add focused coverage for `wai add`
├── ○ wai-fvhv.28 ● P1 Tests: add focused coverage for `wai doctor`
├── ○ wai-fvhv.39 ● P1 Tests: add focused coverage for `wai reflect`
├── ○ wai-fvhv.47 ● P1 Tests: add focused coverage for `wai way`
├── ○ wai-fvhv.65 ● P1 Usability review: Clarify `status` vs `prime` entry-point guidance
├── ○ wai-fvhv.66 ● P1 Usability review: Clarify `doctor` vs `way` mental model
├── ○ wai-fvhv.69 ● P1 Usability review: Audit consistency of global flags across commands
├── ○ wai-fvhv.70 ● P1 Usability review: Review non-interactive behavior for CI/agent use
├── ○ wai-fvhv.72 ● P1 Usability review: Audit typo and wrong-order recovery UX
├── ○ wai-fvhv.73 ● P1 Usability review: Review tutorial exit and next-step messaging
├── ○ wai-fvhv.74 ● P1 Usability review: Improve empty-workspace status UX
├── ○ wai-fvhv.77 ● P1 Usability review: Audit pipeline command usability for first-time users
├── ○ wai-fvhv.79 ● P1 Usability review: Audit `import` preview and safety UX
├── ○ wai-fvhv.81 ● P1 Usability review: Audit search filter ergonomics
├── ○ wai-fvhv.83 ● P1 Usability review: Audit JSON output discoverability
├── ○ wai-fvhv.84 ● P1 Scope: Define product boundary between workflow management and repository-hygiene auditing
├── ○ wai-fvhv.86 ● P1 Scope: Clarify command-role naming for wai/way/why to reduce brand and scope ambiguity
├── ○ wai-fvhv.88 ● P1 Scope: Define support policy for LLM-backed `why` providers and fallback modes
├── ○ wai-fvhv.89 ● P1 Scope: Establish admission criteria for adding new top-level commands
├── ○ wai-fvhv.91 ● P1 Scope: Clarify ownership boundaries among `doctor`, `way`, `sync`, and `import`
├── ○ wai-fvhv.92 ● P1 Scope: Decide whether pipelines are core workflow or optional advanced feature
├── ○ wai-fvhv.93 ● P1 Scope: Evaluate plugin-system scope against maintenance budget
├── ○ wai-fvhv.94 ● P1 Scope: Clarify tutorial target audience and success criteria
├── ○ wai-fvhv.96 ● P1 Scope: Define JSON output stability policy for automation users
├── ○ wai-fvhv.97 ● P1 Scope: Clarify what `--safe` guarantees across every command family
├── ○ wai-fvhv.30 ● P2 Tests: add focused coverage for `wai import`
├── ○ wai-fvhv.31 ● P2 Tests: add focused coverage for `wai init`
├── ○ wai-fvhv.32 ● P2 Tests: add focused coverage for `wai ls`
├── ○ wai-fvhv.33 ● P2 Tests: add focused coverage for `wai move`
├── ○ wai-fvhv.34 ● P2 Tests: add focused coverage for `wai new`
├── ○ wai-fvhv.35 ● P2 Tests: add focused coverage for `wai phase`
├── ○ wai-fvhv.38 ● P2 Tests: add focused coverage for `wai prime`
├── ○ wai-fvhv.41 ● P2 Tests: add focused coverage for `wai search`
├── ○ wai-fvhv.42 ● P2 Tests: add focused coverage for `wai show`
├── ○ wai-fvhv.43 ● P2 Tests: add focused coverage for `wai status`
├── ○ wai-fvhv.44 ● P2 Tests: add focused coverage for `wai sync`
├── ○ wai-fvhv.45 ● P2 Tests: add focused coverage for `wai timeline`
├── ○ wai-fvhv.46 ● P2 Tests: add focused coverage for `wai tutorial`
├── ○ wai-fvhv.49 ● P2 Code quality: modularize src/commands/pipeline.rs
├── ○ wai-fvhv.50 ● P2 Code quality: modularize src/commands/way.rs
├── ○ wai-fvhv.51 ● P2 Code quality: modularize src/commands/doctor.rs
├── ○ wai-fvhv.52 ● P2 Code quality: modularize src/commands/why.rs
├── ○ wai-fvhv.53 ● P2 Code quality: modularize src/commands/reflect.rs
├── ○ wai-fvhv.54 ● P2 Code quality: modularize src/commands/resource.rs
└── ○ wai-fvhv.59 ● P2 Code quality: modularize src/commands/add.rs

--------------------------------------------------------------------------------
Total: 50 issues (50 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
```

