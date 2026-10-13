# Proposal: harden-orchestration-gates

## Why

A Rule-of-5-reviewed evaluation of 32 wai pi sessions (Oct 3–13, 4,668 tool
calls) found four orchestration-integrity gaps that the existing gate system
does not close:

1. **False-evidence advancement.** The 2026-10-05 catastrophic RED violation
   (wai-vx02.3): the run advanced steps 4→5 with zero underlying work — no
   GREEN design artifact, RED tests written against an imaginary fixture API,
   and a misleading gate summary. The structural-gate mechanism exists
   (`pipeline-resource`: "Structural gate blocks on missing artifacts") but
   the epic-orchestrator / epic-autonomy-tdd-ro5 templates never declared
   per-step artifact requirements, so nothing blocked advancement.
2. **Invisible gate bypasses.** `--no-verify` appeared 36× in bash commands
   across the window — always with narrative disclosure, but never recorded
   anywhere machine-readable. Bypasses are invisible to later sessions and
   to `wai pipeline status`.
3. **Rubber-stamp verification.** Verify subagent sessions ran 5–7 tools in
   under a minute. Nothing requires a verify session to produce evidence
   (artifact reads, a fresh test-run output) before its CONFIRM counts. In
   single-model environments the verify step degrades to the lead side
   (observed Oct 9), so CONFIRM may not be independent at all.
4. **Cross-project agent collisions are invisible.** The evallerina-00n
   incident: a parallel agent in another project parked a WIP fix in
   `stash@{1}` ("evallerina-00n WIP (parked — parallel agent owns wai)") that
   had to be discovered manually during the release train. wai's orchestration
   loop has no surfacing for foreign-agent provenance in workspace state.

## What Changes

- **Step artifact manifests.** Orchestrator templates declare, per step, the
  artifacts that must exist before the step may advance (structural gates
  with per-step `types`/`min_artifacts`). A run advancing past a step whose
  declared artifacts are missing is blocked — closing the RED-violation class.
- **Bypass telemetry.** When a pre-push/staged gate is bypassed (`--no-verify`
  or equivalent) during an active run, the agent records it via
  `wai pipeline bypass <reason>` (hooks can't self-record a bypass); the
  run-state ledger entry is surfaced by `wai pipeline status`.
- **Verifier evidence contract.** Verify briefs must require evidence: a
  minimum set of artifact reads plus a fresh test-run output echoed into the
  review artifact before CONFIRM is valid; degraded (single-model) verify
  records REVIEW_REQUIRED instead of CONFIRM.
- **Cross-agent collision surfacing.** `wai prime` (and `wai pipeline next`)
  warn when workspace state indicates a parallel agent: a stash entry with
  agent-provenance naming, or an unexpired agent lock-file under `.wai/`.

## Impact

- **Affected specs:** orchestration-integrity (new capability)
- **Affected code:** pipeline templates (`src/templates/epic-orchestrator.toml`,
  `src/templates/tdd-ro5.toml`), run-state ledger + status rendering,
  verify brief template, prime collision warning
- **Relationships:** complements wai-tdnv (spawn wrapper/heartbeat — different
  layer: spawn mechanics vs run integrity) and wai-t3qa (exit codes).
  Supersedes nothing.

## Success Criteria

- [ ] A run cannot advance past a manifest step while its declared artifacts are missing
- [ ] Every `--no-verify` bypass during an active run is recorded and surfaced in status
- [ ] A verify session without test-run evidence cannot yield CONFIRM
- [ ] A parallel-agent stash/lock is surfaced at prime time (evallerina-00n class)
- [ ] Test coverage for each change (red→green→refactor)
