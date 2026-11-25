# Change: Approval-first gate order + enforced oracle timeouts

## Why

sunset-mcw diagnosed two pipeline defects that re-wedge every future
`wai pipeline next` on a tdd-ro5 ship-close step (repro x3, 2026-11-25,
exit 124 at the caller's 90s wrapper):

1. `execute_oracle` never enforced the TOML `timeout` — the
   `timeout_secs > 0` branch called the same plain `wait_with_output()` as
   the no-timeout branch, so slow oracles (tests-pass 300s, spec-corpus
   ~600s) hung until an outer wrapper killed wai mid-oracle.
2. `evaluate_gates` ran the oracle tier **before** the approval tier, so an
   unapproved ship-close burned minutes of oracle wall-time before reporting
   the missing human approval — guaranteeing the wedge above.

## What Changes

- **Approval-first evaluation**: the approval tier SHALL be evaluated
  immediately after the cheap tiers (structural, procedural, coverage) and
  before the oracle tier. An unsatisfied human checkpoint blocks advancement
  without executing any oracle; once approved, `wai pipeline next` runs the
  oracles and advances when they pass.
- **Enforced oracle timeout**: an oracle that does not exit within its
  configured `timeout` is killed and reaped, and is reported as a **gate
  failure** (message: `Oracle '<name>' timed out after <n>s (killed)`),
  not a CLI abort. `timeout = 0` remains unbounded; the implicit default
  (30s) is now actually enforced. Pipe drain threads prevent a chatty
  oracle from deadlocking on full pipe buffers while the deadline is
  polled.
- **Known limitation, tracked**: `kill` targets the direct oracle process
  only; grandchildren spawned by the oracle may outlive a timeout until
  they finish naturally (beads wai-wicn, process-group kill follow-up).

## Impact

- Affected specs: `pipeline-resource` (Pipeline Gate Protocol: tier order;
  Oracle timeout scenario)
- Affected code:
  - `src/commands/pipeline/gates.rs` (`evaluate_gates` tier order;
    `execute_oracle` deadline enforcement + `OracleOutcome`;
    `run_oracle` failure mapping)
- Regression tests:
  - `commands::pipeline::tests_coverage_oracle::execute_oracle_enforces_timeout`
  - `commands::pipeline::tests_coverage_oracle::approval_gate_blocks_oracle_execution`
- Cross-references: sunset beads `sunset-mcw`, wai beads `wai-a6uy`
