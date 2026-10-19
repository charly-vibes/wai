# Tasks: add-run-wait-primitive

Sequencing: implement after `add-spawn-preflight` lands (both touch
`src/commands/pipeline/orchestration.rs` and the epic-orchestrator template —
review EXCL-001).

## 1. Regression tests (red-first)

- [x] 1.1 `orchestrator_template_requires_wait_primitive`: template scan asserts
      the spawn step names `wai pipeline wait` and names free-form sleep polling
      as a violation (pattern: existing suite_pipeline_template_blocks tests)
- [x] 1.2 `pipeline_wait_exit_codes_and_summary`: pure-logic test of the
      exit-code mapping + summary-line formatting (pass → 0 + one line naming
      PR/state/count; failure → nonzero; timeout → nonzero + elapsed budget +
      last observed state)
- [x] 1.3 `pipeline_wait_zero_checks_terminal_pass`: zero-checks PR → exit 0,
      summary notes zero checks

## 2. Implementation

- [x] 2.1 `wai pipeline wait <pr> [--timeout <minutes>]` subcommand in
      orchestration.rs: `gh pr checks --watch` core, default 30m budget,
      post-timeout single re-query for last observed state, one summary line,
      no hand-written poll loop
- [x] 2.2 epic-orchestrator template spawn step: wait primitive required for
      status waits; sleep-poll loops named as violation

## 3. Validation

- [ ] 3.1 `wai pipeline wait` against a live PR: one line, correct exit codes
- [ ] 3.2 `just test` green (1.1–1.3 included)
- [ ] 3.3 `just ci` green
- [ ] 3.4 `openspec validate add-run-wait-primitive --strict`
- [ ] 3.5 Follow-up: next weekly `scripts/session-friction.py` scan shows
      sleep_poll_ge60 < 108 (Must gate, wai-63h9) — depends on adoption beyond
      the orchestrator template (monitor sessions too)
