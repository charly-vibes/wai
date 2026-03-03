## Context

The gather → create/implement → review pattern is used repeatedly but encoded
only as CLAUDE.md prose. wai has no concept of ordered skill sequences, what
each stage produces, or run state. This design establishes a minimal pipeline
primitive without over-engineering toward a workflow engine.

## Goals / Non-Goals

- Goals:
  - Machine-readable pipeline definition (stages, artifact types)
  - Run state tracking (which stage is active, what was produced)
  - Artifact tagging to link stages to their run
  - CLI hints that guide the user/agent to the next stage
- Non-Goals:
  - Automatic stage invocation (pipeline does not call skills; it tracks them)
  - Remote execution or distributed pipelines
  - Visual pipeline UI
  - Branching or conditional stages

## Decisions

- Decision: Pipeline is a coordination layer, not an execution engine
  - `wai pipeline run` creates a run ID and prints a hint; it does NOT invoke
    the skill. The agent (human or AI) invokes the skill in a separate step.
  - This keeps the execution model simple and works across all shell environments
  - Alternative: auto-invoke the skill via `wai pipeline run --exec` — deferred;
    the hint model is sufficient for the multi-agent use case

- Decision: Artifact linking via environment variable + tag
  - `WAI_PIPELINE_RUN=<run-id>` is set in the shell when a skill runs inside a
    pipeline context; `wai add` reads this env var and appends the tag automatically
  - Alternative: require explicit `--tag pipeline-run:<id>` in each skill —
    rejected; too fragile, skills would need to be written differently for pipeline
    vs standalone use

- Decision: Run state stored in YAML at `.wai/resources/pipelines/<name>/runs/<id>.yml`
  - Consistent with wai's file-based, offline-first philosophy
  - Simple to inspect and debug
  - Alternative: flat `.state` file per run — YAML is more structured and supports
    per-stage metadata (artifact path, timestamp)

- Decision: Stage artifact type is informational, not enforced
  - The declared artifact type (`research`, `tickets`, `audit`) is metadata for
    status display and search, not a validation gate
  - Alternative: enforce that `wai add <type>` matches the declared type —
    too restrictive; stage output is sometimes external (bd tickets, not wai artifacts)

## Risks / Trade-offs

- `WAI_PIPELINE_RUN` approach requires the user/agent to `export` the env var
  in their shell before invoking the skill. If they forget, the artifact is not
  tagged. Mitigated by the hint output from `wai pipeline run` including the
  `export WAI_PIPELINE_RUN=<id>` command.

## Migration Plan

Existing CLAUDE.md pipeline descriptions remain valid. Users adopt `wai pipeline`
commands when they want machine-readable tracking; prose docs continue to work.

## Open Questions

- Should `wai add` also accept an explicit `--pipeline-run <id>` flag as an alternative
  to the env-var? This would be more robust in environments where `export` is awkward
  (e.g., subshells, fish shell, CI runners with strict env isolation). The env-var approach
  is simpler for the common case but the explicit flag would make the tagging visible in
  skill invocations and auditable in shell history. Recommendation: implement env-var first;
  add explicit flag if users report env-var unreliability.
