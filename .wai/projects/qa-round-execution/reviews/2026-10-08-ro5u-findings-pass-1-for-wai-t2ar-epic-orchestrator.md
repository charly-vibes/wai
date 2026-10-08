---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-t2ar, pipeline-step:ro5-review]
---

# RO5U findings pass 1 — wai-t2ar (epic-orchestrator template registration)

Scope reviewed: `src/templates/epic-orchestrator.toml` (new), `src/config.rs`,
`src/commands/pipeline/setup.rs`, `src/cli/subcommands.rs`, `src/cli/mod.rs`,
`src/help.rs`, `tests/suite_pipeline_init_epic_orchestrator_template.rs` (new).

## Stage 1 — DRAFT

Shape sound: 7 steps encode the canon loop (claim → gates → brief → spawn →
verify → ship → stop). Template mirrors the tdd-ro5 TOML shape exactly (gate
syntax, `[pipeline.metadata]`, oracle blocks), so it inherits proven parsing
and gate-resolution behavior.

## Stage 2 — CORRECTNESS

Verified scaffold + validate + gates in a fresh temp workspace with
`target/debug/wai`: template parses, all 7 steps resolve with correct
structural gates; `tests-pass` oracles on verify/ship resolve.

- Finding 2.1 (LOW, NOT A DEFECT): fresh-workspace validate warns
  "Gate oracle 'tests-pass' — command not found". Evidence:
  `wai pipeline validate tdd-ro5` in an equally fresh workspace produces the
  identical warning — the `tests-pass` oracle ships with the workspace suite,
  not bundled per-template (`bundled_oracles_for` only ships
  `release-docs-fresh.sh` for tdd-ro5). Parity with tdd-ro5 confirmed; no fix.

**Convergence: new CRITICAL = 0 → CONVERGED.**

## Stage 3 — CLARITY

Prompts are self-contained and cite canon invariants inline (1, 2, 3, 4, 5,
6, 7). No findings.

**Convergence: new CRITICAL = 0, no new issues → CONVERGED.**

## Stage 4 — EDGE CASES

- Finding 4.1 (LOW, FIXED): claim prompt referenced `bd ready`
  unconditionally; tdd-ro5's orient step guards with "if beads is present".
  Fixed in `src/templates/epic-orchestrator.toml` (also added the openspec
  guard for consistency). Template tests re-run: 5/5 green.
- Finding 4.2 (LOW, DEFERRED): pipeline-help example comment alignment is
  cosmetic; not worth a review round. Deferred, no ticket — it is a one-line
  comment in a help example.

**Convergence: new CRITICAL = 0, issue rate < 10% → CONVERGED.**

## Stage 5 — EXCELLENCE

No further findings. **CONVERGED.**

## Verdict

APPROVED — Critical 0, High 0, Medium 0, Low 1 fixed + 1 deferred (cosmetic).
Satisfies the ro5u-review gate (`max_critical = 0`).
