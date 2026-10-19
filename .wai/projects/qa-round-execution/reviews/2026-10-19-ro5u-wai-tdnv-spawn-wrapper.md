---
reviews: 2026-10-19-green-implemented-minimum-change-src-pipeline-s.md
verdict: pass-with-fixes
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-tdnv, pipeline-step:ro5u-review]
---

RO5U wai-tdnv spawn wrapper — findings from full-diff review (GREEN state,
before fixes):

## Critical

- **C1 — stderr pipe deadlock** (`src/commands/pipeline/orchestration.rs`,
  `launch_detached`): stderr is `Stdio::piped()` but the pipe is never read. A
  subagent writing >64KB to stderr fills the OS pipe buffer and blocks the
  spawn forever while `spawn-status` still reports `running` — the exact
  capture-then-drain-after-wait class ro5u caught in wai-63h9. Fix: stream
  stderr to the log file like stdout.

## High

- **H1 — `which_timeout` false negative** (orchestration.rs): the pre-check
  probes `timeout --version`; non-GNU timeouts (busybox, mac gtimeout) may not
  support `--version` and exit nonzero → every spawn falsely blocked with "not
  found" though the binary works. Fix: drop the pre-check; map launch spawn
  error `kind == NotFound` to the actionable message.
- **H2 — spawns dir not gitignored** (verified `git check-ignore` exit 1 for
  `.wai/pipeline-runs/spawns/x.json`): pid/log/checkpoint churn pollutes
  `git status`; stale checkpoints (dead pids) would get committed. Fix: add
  `pipeline-runs/spawns/` to `.wai/.gitignore` local-only section.

## Medium

- **M1 — `--budget 0` accepted** (cmd_spawn): `timeout 0s` kills the spawn
  instantly; the agent gets a silent no-op spawn. Fix: validate budget >= 1.
- **M2 — `pid_alive` depends on /usr/bin/kill** (procps): absent on minimal
  images → always-false liveness → live spawns misreported as exited. Fix:
  `/proc/<pid>` existence check on Linux (the supported platform, zero forks),
  keep the `kill -0` fallback for non-Linux.

## Low

- **L1** (cmd_spawn_status): paths sorted twice, leftover `let _ = &mut paths`,
  needless `lines` buffering — sort once, print in-loop. Fixed as drive-by.
- **L2** (log_tail): a 4096-byte seek boundary can split a UTF-8 char →
  `read_to_string` error swallowed → empty probe. Fix: read bytes +
  `from_utf8_lossy`. Fixed as drive-by.
- **L3**: checkpoint save is non-atomic `fs::write` — crash mid-write leaves
  corrupt JSON that hard-errors status. NOT fixed this step (window is ~50µs
  between spawn and save); tracked as follow-up fodder.
- **L4**: orphaned 30s stub processes outlive test runs — harmless (unique
  tmp dirs per test), noted only.
