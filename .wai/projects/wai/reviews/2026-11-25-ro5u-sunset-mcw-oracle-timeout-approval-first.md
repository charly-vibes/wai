# RO5U Review — sunset-mcw / wai-a6uy: oracle timeout enforcement + approval-first gate order

Scope: `git diff src/commands/pipeline/gates.rs` (+ tests in `tests_coverage_oracle.rs`) on run `tdd-ro5-2026-11-25-sunset-mcw-enforce-oracle-timeouts-and-approval-first-gate-order`.

> BANNER: Findings NOT verified by TypeSafe (API key absent). Self-reported validation in effect; findings validated by direct code reading and executed tests.

## Stage 1 — DRAFT

Shape Quality: GOOD — two focused changes matching ticket scope: (1) approval tier moved before oracles in `evaluate_gates`, (2) real deadline enforcement in `execute_oracle`. No scope creep; dead `wait_with_output` path removed.

## Stage 2 — CORRECTNESS

[CORR-001] MEDIUM — gates.rs:574-580 (timeout kill path)
Description: `child.kill()` terminates only the direct oracle process; grandchildren spawned by the oracle (e.g. `just spec-check` → python/pytest) survive the timeout and keep running detached.
Evidence: std `Command` has no process-group API; kill(pid) does not signal the process group. Prior behavior was strictly worse (no kill at all; wrapper timeout killed only wai itself, exit 124, leaving the whole tree running).
Recommendation: follow-up — spawn oracles in their own process group (`pre_exec` + `setsid`/`setpgid` via libc/nix, or platform-gated) and `kill(-pgid)` on deadline. Deferred from this change: adds an unsafe/dependency surface beyond the minimal fix; the ticket's defect class (unbounded hang) is resolved. Filed as wai follow-up.

[CORR-002] LOW — gates.rs:571-584 (post-kill exit race)
Description: if the oracle exits successfully in the ~100ms poll window straddling the deadline, `kill()` may land on an already-exited child and the run is still reported as a timeout.
Evidence: kill-then-`wait()` result is discarded; only `Ok(Some(status))` from `try_wait` carries a verdict.
Recommendation: acceptable — poll granularity bounds the window; after `kill()`, `wait()` could return the real status and a retry could distinguish it. Not worth the branch today.

[CORR-003] LOW — gates.rs:583 (try_wait error path detaches reader threads)
Description: on `try_wait` Err the function returns without joining the drain threads; they stay blocked in `read_to_end` until the child exits or the process dies.
Evidence: early `return Err(...)` skips both `join()` calls.
Recommendation: harmless in practice (wai exits → pipes close → threads end); could kill+join in the error arm for symmetry.

Correctness Quality: GOOD

## Stage 3 — CLARITY

No new findings. Doc comment now states actual semantics ("Errors once timeout_secs elapses… killed; 0 = wait indefinitely") replacing the previous false "Wait with timeout" branch. Comments explain the pipe-drain rationale. Clarity Quality: EXCELLENT

Convergence after Stage 3: New CRITICAL: 0; new issues: 0; Status: CONVERGED.

## Stage 4 — EDGE CASES

[EDGE-001] LOW — run_oracle `?` propagation of timeout Err
Description: a timed-out oracle aborts `wai pipeline next` with `Error:` (miette) instead of a gate-failure line in the gate report.
Evidence: `execute_oracle(...)?` inside `run_oracle`; failures vs Err distinction.
Impact: none for callers — process exits non-zero with a precise message instead of hanging; agent-visible and retryable.
Recommendation: keep; an oracle that never returned a verdict is infra failure, not gate data.

Pre-existing behaviors checked, no regression: `timeout` absent → default 30s enforced (previously ignored entirely); `timeout_secs = 0` stays unbounded; approval-postdates-artifacts semantics unchanged; `pipeline status` also becomes fast on unapproved steps (desirable). Edge Case Coverage: GOOD

Convergence after Stage 4: New CRITICAL: 0; Status: CONVERGED.

## Stage 5 — EXCELLENCE

Excellence Assessment:
- Structure: GOOD
- Correctness: GOOD (CORR-001 deferred with reason + follow-up ticket)
- Clarity: EXCELLENT
- Edge Cases: GOOD
- Overall: GOOD

Production Ready: YES

## Severity counts

- CRITICAL: 0
- HIGH: 0
- MEDIUM: 1 (CORR-001, deferred with reason → follow-up ticket)
- LOW: 3 (CORR-002, CORR-003, EDGE-001 — accepted as-is)

## Verdict

READY — both ticket defects (unbounded oracle hang; approval-last wedge) are fixed and regression-locked by tests; the one MEDIUM is a strictly-better-than-status-quo limitation deferred to a tracked follow-up.
