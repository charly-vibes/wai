# Tasks: fix-gate-approval-first-and-oracle-timeout

## 1. Regression tests (red-first)

- [x] 1.1 `execute_oracle_enforces_timeout`: `sleep 5` with `timeout = 1` →
      timed-out outcome, killed near the deadline (RED before fix: returned
      `Ok` after waiting the full 5s)
- [x] 1.2 `approval_gate_blocks_oracle_execution`: required-approval gate
      unmet + `touch` oracle → approval failure, oracle never runs (artifact
      mtime unchanged)

## 2. Implementation

- [x] 2.1 `evaluate_gates`: evaluate the approval tier before the oracle tier
- [x] 2.2 `execute_oracle`: enforce `timeout_secs` — drain threads, 100ms
      `try_wait` poll, kill + reap at deadline; cleanup on wait error
- [x] 2.3 `run_oracle`: map timeout to a gate failure
      `Oracle '<name>' timed out after <n>s (killed)` instead of a CLI abort
- [x] 2.4 Tracked limitation: grandchildren of a killed oracle outlive the
      timeout (beads wai-wicn; process-group kill follow-up)

## 3. Validation

- [x] 3.1 `cargo test --bin wai pipeline::` green (82 tests)
- [x] 3.2 `cargo clippy --bin wai` clean
- [x] 3.3 `openspec validate fix-gate-approval-first-and-oracle-timeout --strict`
- [x] 3.4 `cargo test` full suite green
