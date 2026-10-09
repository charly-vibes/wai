# Proposal: resume-pipeline-adoption

## Why

Pi session evaluation (Oct 7–8, sessions 15:33 and 18:02) found that the
epic-orchestrator adoption lagged behind `wai status` "Next step" guidance.
The agent had to be hand-nudged to continue orchestration — three times in
the 18:02 session ("continue" ×2, then "remember to always resume
orchestration without asking") and twice in the 20:07 session ("wai close"
×2, "you may continue with the next issue").

Root cause: neither `wai prime` nor `bd sync` output ever surfaced "resume
the orchestration". When the agent resumed (pi `-c` or a fresh session),
the "Next step: auto-execute epic-orchestrator" suggestion conflicted with
the older epics/`bd ready` flow, and the agent couldn't tell which system
was authoritative — so it asked instead of continuing.

## What Changes

- `wai prime` must surface an explicit **resume-the-orchestration** line
  when a pipeline run is active for the current project.
- `bd sync` output should include the same breadcrumb so resume detection
  (wai close → resume detection) picks it up.
- The suggestion must be authoritative over the older epics/`bd ready`
  flow when both are active — no ambiguity about which to follow.

## Impact

- **Affected specs:** pipeline-resume
- **Affected code:** src/commands/prime.rs (suggestion), src/commands/sync.rs (breadcrumb)
- **Test shape:** red→green→refactor per TDD; each change gets a failing
  test first, then implementation, then tidy commit.

## Success Criteria

- [ ] `wai prime` shows resume-the-orchestration when a pipeline run is active
- [ ] `bd sync` shows the same breadcrumb
- [ ] A resumed session continues the orchestration without user nudging
- [ ] Test coverage for each change (unit test for prime suggestion, unit test for sync breadcrumb)
