# Tasks: add-spawn-preflight

## 1. Regression tests (red-first)

- [ ] 1.1 `preflight_reports_all_failures`: simulated worktree missing lefthook
      with broken bd state → exit nonzero, both tools named in output
- [ ] 1.2 `preflight_passes_clean_env`: healthy env → exit 0, no mutation
- [ ] 1.3 `orchestrator_template_requires_preflight`: template scan asserts the
      spawn step requires `wai pipeline preflight` before spawning

## 2. Implementation

- [ ] 2.1 `wai pipeline preflight` subcommand in orchestration.rs: check set
      (lefthook, bd list, testaruda store), all-failures-one-pass reporting,
      actionable fix message per tool, no install / no network
- [ ] 2.2 epic-orchestrator template spawn step: preflight required before spawn

## 3. Validation

- [ ] 3.1 `just test` green (1.1–1.3 included)
- [ ] 3.2 `just ci` green
- [ ] 3.3 `openspec validate add-spawn-preflight --strict`
- [ ] 3.4 Follow-up: next weekly `scripts/session-friction.py` scan shows
      lefthook-exit-2-no-output class = 0 (Must gate, wai-gvqj)
