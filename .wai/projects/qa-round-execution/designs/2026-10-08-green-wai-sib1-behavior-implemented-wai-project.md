---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-sib1, pipeline-step:execute]
---

GREEN: wai-sib1; behavior implemented: wai project use gains --shell <posix|fish> override (#[arg(long)], src/cli/mod.rs) beating SHELL detection; POSIX-safe default preserved (fish detection kept: FISH_VERSION env first, then SHELL ends-with-fish — heuristic documented in detect_shell doc comment); ShellSyntax enum + resolve_shell (rejects unknown values) + detect_shell + export_line added to src/commands/project.rs with 3 unit tests; commands run: cargo test --test suite_project_use_shell_flag (3 pass), --test suite_project_use_fish_shell_syntax (24 pass), cargo test full (all green), cargo clippy --all-targets -- -D warnings clean, cargo fmt --check clean; note: plugin::tests::execute_hook_no_deadlock_on_fast_command is the known timing-flaky wai-z25x, passes in isolation
