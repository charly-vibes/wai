# Change: Add canonical subagent spawn wrapper (`wai pipeline spawn`)

Tracked as: wai-tdnv (P1, tooling layer). Evidence: 2026-10-10 review of October
pi sessions — 178 blocking `pi -p` orchestration calls totaling ~48h; 18 errored,
several wedged until 3600–7200s bash timeouts (upper bound ~10h lost). The
orchestrator cannot see progress mid-call; the only intervention was killing the
run and re-prompting the full brief (30–60 min of finished work re-executed).

Doc layer (pattern canon invariants 6/8/9) and template layer
(`harden-orchestrator-spawn-budget`) are DONE. This change is the remaining
**tooling layer**: canon invariant 10 ("prose does not enforce") — the spawn
safety invariants must be a wai capability, not prose the agent re-derives.

## Why

Every orchestrator spawn today is an improvised shell incantation: `pi -p -n
<name> "$(cat brief)" > log 2>&1 &`. Each agent re-derives the backgrounding,
the log path, the budget wrapping, and the resume syntax — and gets them wrong
under time pressure (blind foreground calls, unbounded wedges, kill-and-restart
of healthy interrupted sessions). The invariants are canon; the enforcement is
not tooling.

## What Changes

- **`wai pipeline spawn <name> <prompt>`** — canonical spawn: launches `pi -p -n
  <name> <prompt>` detached (wai returns immediately) with output streamed to a
  known log path (`.wai/pipeline-runs/spawns/<name>.log`), wrapped in a
  wall-clock budget (`--budget <minutes>`, default 30) enforced by the `timeout`
  binary, and writes a checkpoint file
  (`.wai/pipeline-runs/spawns/<name>.json`: session name, pid, log path, prompt
  path, budget, started timestamp). The checkpoint is the resume/discoverability
  record — canon Invariant 2.
- **`wai pipeline spawn status [name]`** — lightweight progress probe: reads
  checkpoint(s), classifies each spawn (running / exited / budget exhausted)
  from pid liveness plus a one-line probe of the log tail, and prints exactly
  one line per spawn answering: what is running, where is the log, done or
  stuck — canon Invariant 7. No re-derivation, no session-JSONL digging.
- **`wai pipeline spawn resume <name>`** — resume of an *interrupted-but-healthy*
  spawn (canon Invariant 6 exception): verifies the checkpoint + session record
  exist, then launches the `pi -c -p -n "<name>-resume"` invocation detached
  with a fresh checkpoint and budget. Never re-prompts the full brief.
- **epic-orchestrator template** — the spawn step's improvised background+tail
  incantation is replaced by the wrapper as the required path; manual
  `pi -p ... &` incantations are named as a violation.
- **No gate semantics change** — spawning convenience, not a gate tier.

## Impact

- **Affected specs:** `orchestration-integrity` (ADDED requirement: Orchestrator
  Spawn Wrapper)
- **Affected code:** `src/pipeline_spawn.rs` (new pure module: checkpoint
  record, status classification, resume invocation), `src/commands/pipeline/
  orchestration.rs` (bin-side process supervision), `src/cli/subcommands.rs`,
  `src/commands/pipeline/mod.rs`, `src/templates/epic-orchestrator.toml`
  (co-modification with landed add-run-wait-primitive — rebase onto current
  template state), `tests/suite_pipeline_spawn.rs` (new)
