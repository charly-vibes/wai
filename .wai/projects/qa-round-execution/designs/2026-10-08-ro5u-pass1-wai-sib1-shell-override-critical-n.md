---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-sib1, pipeline-step:fix-review-findings]
---

RO5U-PASS1: wai-sib1 (shell override). CRITICAL: none. HIGH: none. MEDIUM: (1) src/commands/project.rs:57 stale comment '(see the NOTE below...)' — NOTE was replaced by detect_shell doc comment, reference dangling; (2) no integration test covers the no-flag POSIX default path (SHELL=bash -> export line); suite header claims fish-syntax file covers it but it only tests fish; (3) tests/suite_project_use_shell_flag.rs header references .wai/projects/wai-pr9m/designs/2026-10-06-design.md which does not exist. LOW: (4) resolve_shell accepts bash|zsh|sh aliases but --shell help and error message advertise only posix/fish; (5) FISH_VERSION env leaks through test env inheritance on fish hosts and could flip no-flag assertions — make no-flag tests hermetic with env_remove.

Verification: review-only pass, no code changes in this step. Commands run = git diff src/cli/mod.rs src/commands/project.rs tests/suite_project_use_shell_flag.rs (inspected), cargo test --test suite_project_use_shell_flag --test suite_project_use_fish_shell_syntax (24+3 passed / 0 failed at review time).
