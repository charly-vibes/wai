---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-sib1, pipeline-step:fix-review-findings]
---

RO5U-FIXES: wai-sib1; fixed=M1 stale NOTE comment rewritten to point at detect_shell doc comment; M2 added project_use_default_is_posix_export_line integration test (no-flag SHELL=bash -> export line), suite header contract updated; M3 dangling wai-pr9m design path replaced with real artifact pointer (qa-round-execution designs dir); L4 alias documentation: error message and help text now say posix (also bash, zsh, sh), fish; L5 new default test is hermetic via env_remove(FISH_VERSION). deferred=none. Commands run = cargo test --test suite_project_use_shell_flag --test suite_project_use_fish_shell_syntax (24+4 passed / 0 failed), cargo clippy --all-targets -- -D warnings (0 warnings), cargo fmt (clean), cargo test --lib project:: (3 passed)
