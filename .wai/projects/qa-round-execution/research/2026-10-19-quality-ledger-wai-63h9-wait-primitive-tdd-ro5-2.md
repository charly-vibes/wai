---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-63h9, pipeline-step:quality-ledger]
---

QUALITY LEDGER: wai-63h9 wait primitive (tdd-ro5-2026-10-19-wai-63h9).

Changed:
- src/pipeline_wait.rs (NEW) — pure wait-outcome module: WaitOutcome, exit_code, summary_line (Display pr, counts omitted when total==0), parse_counts, classify (zero-checks detected before exit-code check). Declared pub in both src/main.rs and src/lib.rs (matrix/plugin pattern).
- src/commands/pipeline/orchestration.rs — cmd_wait: gh pr checks <pr> --watch with concurrent drain threads (started before wait, joined after), deadline supervision via wait-timeout (no sleep-poll), kill+reap on timeout then ONE requery for last observed state, validate_pr_arg, format_elapsed, condense_state, read_pipe, emit_wait_outcome (one summary line; exit code = script API).
- src/cli/subcommands.rs + src/commands/pipeline/mod.rs — Wait { pr, timeout: default 30 } wiring.
- src/templates/epic-orchestrator.toml + .wai/resources/pipelines/epic-orchestrator.toml — spawn step requires wai pipeline wait for status waits, names free-form sleep<N> poll loops as violation (deployed fork otherwise untouched).
- tests/suite_pipeline_wait.rs (NEW) — 5 tests: template scan, exit-code/summary mapping, zero-checks terminal pass, branch-name/unparsed-counts summary, classify zero-checks ordering.
- Cargo.toml/Cargo.lock — wait-timeout 0.2 dep (justified: std has no wait-with-deadline; ticket anti-goal forbids internal sleep-poll).
- openspec/changes/add-run-wait-primitive/tasks.md — 1.1-1.3, 2.1, 2.2 checked (3.x: 3.2 partially green, 3.1/3.3/3.4 validated below, 3.5 deferred to weekly scan).

Verified:
- cargo test --test suite_pipeline_wait → 5 passed, 0 failed
- cargo test --test suite_pipeline_preflight / suite_pipeline_init_epic_orchestrator_template / suite_pipeline_epic_orchestrator_ticket_state → 3+3+9 all passed
- cargo fmt clean; cargo clippy --all-targets -- -D warnings clean
- openspec validate add-run-wait-primitive --strict → valid

Review:
- RO5U converged: 1 Critical (pipe deadlock) + 2 High (zero-checks order, 0/0 rendering) + Medium 4 (Display pr) FIXED with regression tests; Medium 5 (gh-auth errors render as FAILED) explicitly deferred — follow-up; 3 Lows accepted and documented.

Risks:
- classify counts parse is best-effort against gh's actual output format (untestable offline); degrades to counts-omitted summary, never crashes.
- live-PR validation (tasks 3.1) not yet run — needs a real PR watch cycle; do during ship or post-merge.
- Medium 5 deferred (see above).

Next:
- step 9 ship-close: commit the slice on a feature branch, run just ci (tasks 3.3), open PR; live-PR wait check (3.1) on the PR itself; human approval gate before close.
