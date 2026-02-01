---
name: resume-handoff
description: Resume work from a handoff document. Reads the handoff, verifies current state against documented state, and presents analysis before continuing work.
---

# Resume from Handoff

Resume work from a handoff document through analysis and verification.

## Step 1: Read Handoff Completely

```bash
cat handoffs/YYYY-MM-DD_HH-MM-SS_description.md
```

## Step 2: Extract Key Information

- Task Status (completed, in progress, planned)
- Critical Files with line ranges
- Key Learnings
- Open Questions
- Next Steps

## Step 3: Verify Current State

```bash
git log --oneline [handoff_commit]..HEAD
git branch --show-current
git status
git diff [handoff_commit]
```

## Step 4: Read Referenced Files

Read "Critical Files" to understand current implementation and verify learnings still apply.

## Step 5: Present Analysis

```
I've analyzed the handoff from [date]. Here's the current situation:

## Original Context
[Summary]

## Task Status Review
**Completed:** [verified list]
**In Progress:** [current state]
**Planned:** [next steps]

## Changes Since Handoff
[List any commits]

## Key Learnings Still Applicable
[Verified learnings]

## Questions Needing Resolution
[From handoff]

## Recommended Next Action
**Priority 1:** [Action]
- Reason: [Why]
- Files: [What]
- Approach: [How]

Shall I proceed?
```

## Step 6: Get Confirmation and Begin

Wait for user confirmation, then:
- Create todo list from next steps
- Start with highest priority action
- Apply learnings from handoff

## Guidelines

1. **Always verify before acting** - Don't assume handoff matches reality
2. **Apply learnings** - Use documented patterns and avoid noted pitfalls
3. **Adapt to changes** - If codebase changed, acknowledge and adapt
