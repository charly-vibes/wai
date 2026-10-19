---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-63h9, pipeline-step:fix-review]
---

RO5U-FIXES: all ro5u findings from 2026-10-19-ro5u-wai-63h9.md addressed. CRITICAL 1 (pipe-full deadlock): cmd_wait now takes both pipes BEFORE waiting and drains them on two concurrent threads (read_to_string blocks on EOF — no polling); joined after wait_timeout in the pass arm and after kill+wait in the timeout arm (kill closes pipes, drains terminate). Verified borrow-wise: take() first, move into threads. HIGH 2 (zero-checks order): classify moved into the pure module (src/pipeline_wait.rs) and reordered — 'no checks' detection now runs BEFORE the exit-code check, regardless of code; orchestration.rs bin-side copy deleted. HIGH 3 (0/0 rendering): summary_line omits counts when total==0 ('all checks pass' / 'checks FAILED', never '(0/0)'). MEDIUM 4 (branch PRs): summary_line(pr: impl std::fmt::Display) — branch names render verbatim; 'PR 0' impossible. MEDIUM 5 (gh-auth errors as FAILED): explicitly deferred with reason in the review artifact (needs WaitOutcome::Error variant + coverage; out of minimal footprint) — follow-up fodder. LOWs 6-8 accepted as documented. New tests pinning fixed behaviors: pipeline_wait_summary_handles_branch_names_and_unparsed_counts, pipeline_wait_classify_detects_zero_checks_regardless_of_exit_code.

Verification evidence (commands run):
- cargo test --test suite_pipeline_wait → 5 passed, 0 failed
- cargo fmt → clean
- cargo clippy --all-targets -- -D warnings → Finished, zero warnings
