# Tasks: add-run-wait-primitive

## 1. Regression test (red-first)

- [ ] 1.1 `orchestrator_template_requires_wait_primitive`: template scan asserts
      the spawn step names `just ci-wait` and names free-form sleep polling as a
      violation (pattern: existing suite_pipeline_template_blocks tests)

## 2. Implementation

- [ ] 2.1 `just ci-wait <pr>` recipe: `gh pr checks --watch` core, optional
      timeout (default 30m), single summary line, nonzero exit on check failure
      or timeout
- [ ] 2.2 epic-orchestrator template spawn step: wait primitive required for
      status waits; sleep-poll loops named as violation

## 3. Validation

- [ ] 3.1 `just ci-wait` against a live PR: one line, correct exit codes
- [ ] 3.2 `just test` green (template test included)
- [ ] 3.3 `openspec validate add-run-wait-primitive --strict`
- [ ] 3.4 Follow-up: next weekly `scripts/session-friction.py` scan shows
      sleep_poll_ge60 < 108 (Must gate, wai-63h9)
