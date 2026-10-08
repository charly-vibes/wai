---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-sib1, pipeline-step:refactor-or-tidy]
---

REFACTOR: wai-sib1; what changed structurally: Use doc comment now mentions --shell posix|fish override so help text stays truthful; no behavior change. Commands run = cargo test --test suite_project_use_shell_flag --test suite_project_use_fish_shell_syntax (24+3 passed / 0 failed), cargo clippy --all-targets -- -D warnings (0 warnings), cargo fmt --check (clean).
