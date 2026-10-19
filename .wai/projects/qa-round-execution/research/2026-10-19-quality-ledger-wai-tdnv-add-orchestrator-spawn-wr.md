---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-tdnv, pipeline-step:quality-ledger]
---

QUALITY LEDGER wai-tdnv (add-orchestrator-spawn-wrapper)

Changed:
- src/pipeline_spawn.rs (new): pure module — SpawnCheckpoint serde record (session/pid/log_path/prompt/budget/started_at), classify (dead-pid > deadline > running; unparseable timestamp degrades to running), last_nonempty_line, status_line one-line contract
- src/commands/pipeline/orchestration.rs: cmd_spawn (detached timeout-wrapped pi -p -n, checkpoint before return, budget>=1 guard, refuse duplicate name), cmd_spawn_status (kill-0 liveness + log-tail probe, one line per spawn), cmd_spawn_resume (pi -c -p -n <name>-resume, fresh checkpoint with parent budget+prompt, refuse missing/duplicate), launch_detached (stdout+stderr to log file — no pipes, pid reparenting), pid_alive (/proc on Linux, kill -0 fallback), log_tail (bytes + lossy), spawns_dir, validate_spawn_name
- src/cli/subcommands.rs + src/commands/pipeline/mod.rs: Spawn/SpawnStatus/SpawnResume wiring
- src/lib.rs + src/main.rs: module declarations (lib+bin pattern)
- src/templates/epic-orchestrator.toml: spawn step rewritten around the wrapper (spawn/spawn-status/spawn-resume required; manual pi -p incantations named violation; budget default 30m)
- tests/fixtures/pipeline-rendered-baseline-vsn6/epic-orchestrator.txt: baseline regen (UPDATE_BASELINE=1)
- tests/suite_pipeline_init_epic_orchestrator_template.rs: resume assertion rebased to wrapper shape (co-mod rebase)
- tests/suite_pipeline_spawn.rs (new): 6 tests — template scan, checkpoint lifecycle, status classification + pure classify, resume canon shape, budget-0 rejection, stderr-to-log (128KB)
- .wai/.gitignore: pipeline-runs/spawns/ local-only

Verified:
- cargo test --test suite_pipeline_spawn → 6 passed; 0 failed (stable across 6+ runs)
- cargo test --bin wai → 524+ passed incl. vsn6 baseline ratchet (post-regen) and rebase-rebased template suite 9/9
- just test (testaruda exec) → exit 0, CALIBRATED, ingested 1270 results
- cargo clippy --all-targets → 0 warnings; cargo fmt → clean
- openspec validate add-orchestrator-spawn-wrapper --strict → valid (re-validate at ship)

Review:
- RO5U pass-with-fixes: 1 CRITICAL (stderr pipe deadlock) + 2 HIGH (which_timeout false negative; spawns dir unignored) + 2 MEDIUM (budget 0; procps kill dependency) FIXED this step; 2 LOW drive-by fixed (double-sort, UTF-8 boundary); 2 LOW noted-not-fixed (L3 non-atomic checkpoint write, L4 orphaned test stubs)

Risks:
- L3: crash mid-checkpoint-write leaves corrupt JSON (~50µs window; status hard-errors until manual delete)
- resume uses pi -c per canon; if pi -c targets the wrong session in multi-session cwd, pi-side semantics apply (canon owns the shape)
- task 3.5 (pattern-doc projection, chezmoi-managed) deferred to follow-up ticket

Next:
- mark openspec tasks 1.1-3.4 complete; 3.4 live validation of a real spawn; ship step
