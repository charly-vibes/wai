---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-05-wai-vx02-5-inter-child-handoff, pipeline-step:refactor-or-tidy]
---

REFACTOR: wai-vx02.5; commands run: cargo test --test integration add_ (31 passed), cargo fmt clean, cargo clippy --all-targets -D warnings clean, cargo test --test integration (391 passed/0 failed). Structural change: extracted the env-var-first active-run resolution into config::resolve_active_pipeline_run and switched src/commands/add.rs build_tags to use it (behavior-neutral — same env first, .last-run fallback order), removing the duplication with the new handoff.rs call site. No new source files created, so the ticket's new-file header requirement is N/A. Remaining overlap: build_epic_tree_from_parent's child-run scan and find_epic_parent_run_by_topic both iterate the runs dir but with different match predicates (topic==issue ownership vs started-child lookup) — left as-is, extraction would obscure intent for no real gain.
