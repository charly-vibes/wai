# Design: harden-orchestration-gates

## Decision 1: manifest gates via existing structural tier (no new gate tier)

The `pipeline-resource` spec already defines a structural gate tier that
blocks on missing step-tagged artifacts. The RED violation happened because
templates never used it, not because the mechanism is missing.

**Choice:** declare per-step `[steps.<step>.gate.structural]` blocks in the
orchestrator templates (e.g., implement steps require ≥1 design artifact;
ship steps require the run-state head record). No new gate code unless the
structural tier lacks a needed predicate — in that case extend the tier, not
a new mechanism.

**Rejected:** a new "manifest" gate tier — duplicates the structural tier and
adds validation surface.

## Decision 2: bypass telemetry is ledger-only, not enforcement

`--no-verify` bypasses are sometimes legitimate (pre-existing debt blocks the
staged gate; wai-abiy/wai-80x5 debt class). Enforcement would push agents to
hide bypasses.

**Choice:** record-only. A `wai` wrapper/hook path (or documented discipline
entry appended to run state) records: command, timestamp, disclosed reason.
`wai pipeline status` shows `bypasses: N (last: <reason>)`. Surfacing, not
blocking.

**Rejected:** blocking `--no-verify` outright — makes the gate class worse
(agents hide it); post-hoc git shims — too fragile.

## Decision 3: verifier evidence is recorded in the review artifact

The read-only verifier convention (wai-979g) produces a review artifact.
The evidence contract lives there: the review artifact must cite ≥N artifact
paths read and include fresh test-run output; a verify lacking them records
verdict REVIEW_REQUIRED (not CONFIRM). In single-model degradation, verdict
is REVIEW_REQUIRED by definition — never CONFIRM.

**Choice:** contract enforced by the verify step's oracle gate (the template's
tests-pass oracle gains an evidence check on the review artifact).

## Decision 4: collision surfacing is heuristic, best-effort

Detecting parallel agents reliably is impossible; the evallerina-00n case
worked because the stash message named the agent/ticket.

**Choice:** prime-time warning when (a) a stash entry message matches a
known agent-provenance pattern (`<agent> WIP`, "parked", "owns"), or (b) an
agent lock-file under `.wai/` is present and unexpired. Never hard-fail.

**Rejected:** lock-file enforcement (agents ignore locks today; adoption
cost too high), git-owned mechanisms (would require coordinating other tools).

## Sequencing

Manifest gates (R1) are the highest-value, lowest-risk item and land first.
Telemetry (R2) and verifier contract (R3) are independent and parallelizable.
Collision surfacing (R4) is self-contained and last.
