# Rule of 5 Review: Agnostic Way Capabilities

**Work Reviewed:** OpenSpec Proposal (`openspec/changes/refactor-way-agnostic/`)
**Stage 3: CLARITY**

Issues Found:

[CLAR-001] [MEDIUM] - `Documentation` -> `Project documentation`
Description: While agnostic, "Project documentation" is slightly redundant given it's a repository health tool.
Impact: Less specific than it could be.
Recommendation: Consider "Project identity & onboarding" or "Core documentation" to reflect its intent (README, LICENSE, CONTRIBUTING). (Self-correction: The design document uses "Project documentation" consistently, which is clear enough).

[CLAR-002] [LOW] - `AI instructions` -> `AI-agent context`
Description: "AI-agent context" is clear, but since it checks for `CLAUDE.md` and `AGENTS.md`, "AI guidance" might be even clearer for humans while remaining agnostic.
Impact: Small clarity improvement for human users.
Recommendation: Stick with "AI-agent context" as it reflects the technical intent better for agents.

[CLAR-003] [MEDIUM] - `Success Criteria` wording
Description: Some success criteria describe the tool (e.g., "A task runner (justfile, Makefile, etc.) is present") rather than the agnostic state (e.g., "Standardized commands are defined for building, testing, and linting").
Impact: Might lead agents to focus on the tool rather than the goal.
Recommendation: Update the "Success Criteria" in the `design.md` table to be more behavior-focused where possible.

Clarity Quality: GOOD

**Convergence Check (after Stage 3):**
New CRITICAL issues: 0
Total new issues: 3
New issues vs Stage 2: 50% change in count.
Status: CONTINUE
