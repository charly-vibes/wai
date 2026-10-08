---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-sib1, pipeline-step:plan]
---

ISSUE PLAN: wai-sib1 fix(project) portable project-use output. Outcome: default POSIX-safe export line so eval $(wai project use <name>) works in any shell; fish syntax opt-in via --shell flag. Out of scope: other project commands, pretender debt, suite restructuring. Files: src/commands/project.rs, src/cli.rs, tests/integration.rs, docs help text. Code bug fix, no openspec (not breaking). RED: confirm project_use_valid_prints_export fails with SHELL=fish; GREEN: POSIX-safe default + --shell flag; REFACTOR: align help/docs. Gates: cargo test, cargo clippy -- -D warnings, cargo fmt --check. Follow-up threshold: newly discovered shell needs (e.g. zsh) -> new ticket
