# Tasks: harden-orchestration-gates

## 1. Step artifact manifest gates (R1)

- [ ] 1.1 RED: test that epic-orchestrator implement/ship steps declare structural gates and block without artifacts
- [ ] 1.2 GREEN: add per-step `[steps.<step>.gate.structural]` blocks to `src/templates/epic-orchestrator.toml` (and tdd-ro5 where applicable)
- [ ] 1.3 Tidy: extract shared gate-block boilerplate (coordinate with wai-opk8 shared-blocks adoption)
- [ ] 1.4 Update rendered baseline fixtures (wai-vsn6 ratchet: run UPDATE_BASELINE gate)

## 2. Gate bypass telemetry (R2)

- [ ] 2.1 RED: test that a recorded bypass during an active run appends a ledger entry and surfaces in `wai pipeline status`
- [ ] 2.2 GREEN: run-state ledger bypass entry + status rendering (count, last reason)
- [ ] 2.3 Document the record path in pipelines.md (discipline: agents record reason at bypass time)

## 3. Verifier evidence contract (R3)

- [ ] 3.1 RED: test that verify review artifacts lacking test-run output resolve to REVIEW_REQUIRED
- [ ] 3.2 GREEN: evidence check in verify-step oracle gate; brief template requires artifact cites + fresh test-run output
- [ ] 3.3 GREEN: single-model degradation records REVIEW_REQUIRED with note (lead-side path)

## 4. Cross-agent collision surfacing (R4)

- [ ] 4.1 RED: test prime collision warning on provenance-matching stash entry
- [ ] 4.2 GREEN: stash-provenance scan + lock-file check in prime (advisory, silent-when-green)
- [ ] 4.3 Extend to `wai pipeline next` when a run is active

## 5. Validation

- [ ] 5.1 Full cargo test + clippy + fmt green
- [ ] 5.2 openspec validate harden-orchestration-gates --strict
- [ ] 5.3 Dogfood: one orchestrated run with manifest gates active (RED violation class regression test)
