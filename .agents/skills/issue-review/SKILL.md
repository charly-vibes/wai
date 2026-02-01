---
name: issue-review
description: Perform iterative issue tracker review using Rule of 5 methodology. Reviews completeness, scope, dependencies, plan alignment, and executability.
---

# Iterative Issue Tracker Review (Rule of 5)

Perform thorough issue review using the Rule of 5 - iterative refinement until convergence.

## Setup

### Gathering Issues to Review

**For Beads:**
```bash
bd list
bd ready
bd graph
bd show <id>
bd dep tree
bd dep cycles
```

**For GitHub Issues:**
```bash
gh issue list --label "needs-review" --json number,title,body,labels
gh issue view <number>
```

## Process

Perform 5 passes, each focusing on different aspects.

### PASS 1 - Completeness & Clarity
Focus: Title, description, file paths, success criteria, acceptance criteria

### PASS 2 - Scope & Atomicity
Focus: Each issue is one logical unit, not too large or too small

### PASS 3 - Dependencies & Ordering
Focus: Dependencies correctly defined, no circular dependencies

### PASS 4 - Plan & Spec Alignment
Focus: Issues trace back to plan phases, TDD approach clear

### PASS 5 - Executability & Handoff
Focus: Can be picked up by any developer/agent without implicit knowledge

## Convergence Check

After each pass (starting with pass 2):
- **CONVERGED**: No new CRITICAL, <10% new issues vs previous pass
- **ITERATE**: Continue to next pass
- **NEEDS_HUMAN**: Found blocking issues requiring human judgment

## Final Report

```
## Issue Tracker Review Final Report

**System:** [Beads/GitHub/Linear/Jira]

### Summary
- Total Issues Reviewed: [count]
- CRITICAL: [count]
- HIGH: [count]

### Top 3 Most Critical Findings
[List with specific commands to fix]

### Verdict
[READY_TO_WORK | NEEDS_UPDATES | NEEDS_REPLANNING]
```
