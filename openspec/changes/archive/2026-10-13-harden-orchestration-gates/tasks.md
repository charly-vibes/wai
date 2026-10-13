# Tasks: harden-orchestration-gates

## 1. Step artifact manifest gates (R1)

- [x] 1.1 RED: test that epic-orchestrator implement/ship steps declare structural gates and block without artifacts
- [x] 1.2 GREEN: add per-step `[steps.<step>.gate.structural]` blocks to `src/templates/epic-orchestrator.toml` (and tdd-ro5 where applicable)
- [ ] 1.3 Tidy: extract shared gate-block boilerplate (coordinate with wai-opk8 shared-blocks adoption) — deferred: `{{block:}}` expansion is prompt-only today; needs gate-table mechanism
- [x] 1.4 Update rendered baseline fixtures (wai-vsn6 ratchet: run UPDATE_BASELINE gate) — drift = ship-close Record line only (RO5U critical fix)

## 2. Gate bypass telemetry (R2)

- [x] 2.1 RED: test that a recorded bypass during an active run appends a ledger entry and surfaces in `wai pipeline status`
- [x] 2.2 GREEN: run-state ledger bypass entry + status rendering (count, last reason)
- [x] 2.3 Document the record path in pipelines.md (discipline: agents record reason at bypass time)

## 3. Verifier evidence contract (R3)

- [x] 3.1 RED: test that verify review artifacts lacking test-run output resolve to REVIEW_REQUIRED
- [x] 3.2 GREEN: evidence check in verify-step oracle gate; brief template requires artifact cites + fresh test-run output
- [x] 3.3 GREEN: single-model degradation records REVIEW_REQUIRED with note (lead-side path)

## 4. Cross-agent collision surfacing (R4)

- [x] 4.1 RED: test prime collision warning on provenance-matching stash entry
- [x] 4.2 GREEN: stash-provenance scan + lock-file check in prime (advisory, silent-when-green)
- [x] 4.3 Extend to `wai pipeline next` when a run is active

## 5. Validation

- [x] 5.1 Full cargo test + clippy + fmt green (1312 passed, 0 failed)
- [x] 5.2 openspec validate harden-orchestration-gates --strict
- [x] 5.3 Dogfood: one orchestrated run with manifest gates active (RED violation class regression test) — the three wai-giqg/px4g/76dq runs hit live gate refusals (plan-gate block, oracle evidence blocks) proving the RED-violation class is caught
