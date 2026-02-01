---
name: create-issues
description: Convert an implementation plan into tracked issues with proper dependencies. Generates commands for issue trackers like Beads or GitHub Issues.
---

# Iterative Issue Creation from Plan

You will act as a project manager. Your task is to take the provided plan and create a set of issues in the specified issue tracking system. You will generate the precise, runnable commands to do so.

## Setup

### Input Plan

The plan to be implemented will be provided here. Your task is to parse it and create issues accordingly.

## Process

For each phase or logical unit of work in the plan, create a corresponding issue. After creating all issues, define their dependencies.

### Issue Template

Each issue you create MUST use the following template for its title and description.

**Title:** A short, clear, action-oriented title (e.g., "Create Login Endpoint").

```
**Context:** [Brief explanation of what this issue is about, referencing the plan]
Ref: [Link to plan document and section]

**Files:**
- [List of files to be modified]

**Acceptance Criteria:**
- [ ] A checklist of what "done" means for this issue.

---
**CRITICAL: Follow Test Driven Development and Tidy First workflows.**
- Write tests *before* writing implementation code.
- Clean up related code *before* adding new functionality.
```

### Creating Issues and Dependencies

Generate the full, runnable commands to create the issues and then wire up their dependencies.

#### Strategy for Robust Execution

1. **Create Issues:** Run the creation command for each issue.
2. **Capture IDs:** From the output of each command, capture the newly created issue ID.
3. **Connect Dependencies:** Use the variables from the previous step to run the dependency commands.

**Example for Beads:**
```bash
issue_1_id=$(bd create --title="DB Schema: Add auth fields" --description="...")
issue_2_id=$(bd create --title="API: Create Login Endpoint" --description="...")
bd dep add "$issue_2_id" "$issue_1_id"
```

**Example for GitHub Issues:**
```bash
issue_1_url=$(gh issue create --title "DB Schema: Add auth fields" --body "...")
issue_2_url=$(gh issue create --title "API: Create Login Endpoint" --body "...")
```

## Final Report

```
## Issue Creation Summary

**System:** [Beads/GitHub/Linear/Jira]
**Plan:** [path/to/plan.md]

### Summary

- Total Issues Created: [count]
- Dependencies Defined: [count]

### Verdict

[ISSUES_CREATED | FAILED_TO_CREATE]
```
