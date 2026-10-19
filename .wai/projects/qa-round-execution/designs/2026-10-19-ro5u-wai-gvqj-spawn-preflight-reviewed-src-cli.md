---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-gvqj, pipeline-step:fix-review]
---

RO5U wai-gvqj — spawn preflight, reviewed: src/cli/subcommands.rs (Preflight subcommand), src/commands/pipeline/mod.rs (dispatch), src/commands/pipeline/orchestration.rs (preflight_check + cmd_preflight), src/templates/epic-orchestrator.toml + repo-local copy (spawn-step rule), tests/fixtures baseline (+6 lines), tests/suite_pipeline_preflight.rs (3 tests).

F1 MEDIUM (FIXED): spec orchestration-integrity requires testaruda STORE INITIALIZATION; implementation initially checked `--version` only. Fixed → `testaruda init` (idempotent; spec permits mutation "no files outside the tool stores"). Evidence: spec.md:7,37; tests unaffected (fake stub exits 0, creates nothing); builtin_templates_render green (args not rendered into baseline).

F2 LOW (accepted, tracked): `bd list` check does not pre-verify .beads presence; in non-beads dirs bd exits nonzero and the failure names bd + `bd doctor` fix — naming contract satisfied.

F3 LOW (accepted): version echo takes first stdout line (may be a long bd list header) — cosmetic.


Verification evidence (commands run):
- `cargo test --test suite_pipeline_preflight` — 3 passed; 0 failed (after F1 fix)
- `cargo test --bin wai builtin_templates_render` — 1 passed (baseline stable)
- `just test` — 1261 results ingested, exit 0
- `just lint` / `just fmt` — clean
