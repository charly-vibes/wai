---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-76dq, pipeline-step:orient-and-claim]
---

ORIENT: selected wai-76dq (P3, R4 of epic wai-ykt1) — last remaining child; self-contained per design.md Decision 4: collision surfacing is heuristic best-effort, never hard-fail. Mechanism: prime-time warning when (a) stash entry message matches agent-provenance pattern (contains 'WIP'/'parked'/'owns' case-insensitive) or (b) unexpired agent lock-file (.lock under .wai/projects, locked_at within 7 days). Extend to wai pipeline next when run active (task 4.3). Files: src/commands/prime.rs (collision scan + render in terminal + JSON), src/commands/pipeline/orchestration.rs (cmd_next warning), src/json.rs (PrimePayload field), tests/suite_prime_collision_warnings.rs.
