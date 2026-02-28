# Rule of 5 Review: Agnostic Way Capabilities

**Work Reviewed:** OpenSpec Proposal (`openspec/changes/refactor-way-agnostic/`)
**Stage 1: DRAFT**

Assessment: The overall approach is sound and aligns perfectly with the goal of making `wai way` agnostic and machine-readable. It correctly identifies the need for "Intent" and "Success Criteria" to move beyond tool-specific checks.

Major Issues:

[DRAFT-001] [MEDIUM] - `design.md` Mapping
Description: The mapping table in the design document is excellent, but it doesn't specify if the `message` field (the tool-specific part) will be updated to be more agnostic or if it will continue to report the specific tool found.
Recommendation: Clarify that `message` remains the "Discovery" result (e.g., "justfile detected"), while `name`, `intent`, and `success_criteria` provide the agnostic context.

[DRAFT-002] [LOW] - `tasks.md` granularity
Description: The tasks are well-ordered, but Task 14 ("Update repository-best-practices specification") is a very large task compared to the others.
Recommendation: Break down Task 14 into smaller items (e.g., update each requirement section) if possible, or ensure it's understood as a significant documentation effort.

Shape Quality: EXCELLENT
