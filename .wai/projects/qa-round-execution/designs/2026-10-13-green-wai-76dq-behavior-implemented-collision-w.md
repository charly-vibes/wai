---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-76dq, pipeline-step:execute]
---

GREEN: wai-76dq; behavior implemented: collision_warnings(project_root) in prime.rs = stash_provenance_warnings (git stash list, cwd=project root, case-insensitive WIP/parked/owns match, silent when git absent/fails) + lock_file_warnings (.lock sidecars under .wai/projects parsed as TOML via toml::from_str, unexpired = locked_at within 7 days); rendered as ⚠ lines in prime closing sections + collision_warnings JSON payload field (skip-when-empty, also in empty-state path); cmd_next prints warnings before gate checks (task 4.3). Verification evidence (commands run): cargo test --test suite_prime_collision_warnings → 4 passed; pipeline+collision suites → 115 passed 0 failed; clippy clean; fmt clean.
