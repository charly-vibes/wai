# Change: Fix stale pipeline-run tagging in `wai add`

## Why

`.last-run` records the last *started* pipeline run, and only `wai close`
clears it. A run that completes without `wai close` keeps its pointer, and
`wai add` resolved the pointer unconditionally — stamping
`pipeline-run:<completed-run-id>` provenance onto unrelated artifacts.
Reproduced 2026-10-14 (beads wild-00r): a finished
`epic-orchestrator-2026-10-13-wild-7e9` run tagged a new unikernel-cd
research artifact; caught by rule-of-5 review as CORR-001.

## What Changes

- **Pointer resolution gated**: `wai add` stamps the `pipeline-run:` tag from
  the `.last-run` pointer only when the run state file exists, parses, and
  shows the run still in progress (`current_step < step count`) — fail-closed.
- **Stale pointer self-heal**: when the pointer names a proven-complete run,
  `wai add` removes the stale pointer (the same clearing `wai close`
  performs), so later adds and `wai status` stop advertising it.
- **Env var unchanged**: `WAI_PIPELINE_RUN` is explicit intent — it stamps
  unconditionally, as before.
- **Unverifiable runs**: a pointer naming a missing/unreadable run state
  suppresses the tag but keeps the pointer (removal is reserved for runs
  proven complete).

## Impact

- Affected specs: `pipeline-resource`
- Affected code:
  - `src/commands/add.rs` (`build_tags` pointer path; new
    `active_pointer_run` helper)
- Regression tests: `commands::add::tests::{stale_pointer_to_completed_run_not_stamped, stale_pointer_cleared_when_run_complete, pointer_to_missing_run_state_not_stamped_and_pointer_kept}`
