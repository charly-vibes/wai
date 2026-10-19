---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-tdnv, pipeline-step:fix-review]
---

RO5U-FIXES: C1 stderr pipe deadlock — launch_detached now dup's the log handle for stderr (both streams to the log file, no pipes; structural elimination) + regression test spawn_streams_stderr_to_log (128KB stderr stub must reach the log). H1 which_timeout false negative — pre-check dropped, launch error kind NotFound maps to the actionable budget message. H2 .wai/.gitignore gains pipeline-runs/spawns/ (verified: git check-ignore now ignores). M1 budget 0 → hard error + regression test spawn_rejects_zero_budget (no checkpoint written). M2 pid_alive → /proc/<pid> on Linux (zero forks, no procps dependency), kill -0 fallback for non-Linux. L1 double-sort + lines buffer + dead 'let _' removed. L2 log_tail reads bytes + from_utf8_lossy (no swallowed UTF-8 errors). L3 (non-atomic checkpoint write) + L4 (orphaned test stubs) noted, not fixed.

## Verification evidence (commands run)

- cargo test --test suite_pipeline_spawn → 6 passed; 0 failed (2 new regression tests)
- cargo clippy --all-targets → 0 warnings; cargo fmt → clean
