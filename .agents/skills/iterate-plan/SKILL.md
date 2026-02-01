---
name: iterate-plan
description: Update an existing implementation plan based on feedback, grounded in codebase reality. Makes surgical edits while preserving completed work.
---

# Iterate Implementation Plan

Update an existing implementation plan based on feedback, grounded in codebase reality.

## When Invoked

1. **No plan file provided**: Ask for the plan path
2. **Plan file provided but NO feedback**: Ask what changes to make
3. **Both provided**: Proceed directly with updates

## Process

### Step 1: Understand Current Plan
Read the existing plan file completely. Note phases, success criteria, and completed work.

### Step 2: Understand Requested Changes
Listen carefully: adding requirements, changing approach, adjusting scope, fixing errors?

### Step 3: Research If Needed
Only if changes require new technical understanding. Skip for straightforward changes.

### Step 4: Confirm Understanding

```
Based on your feedback, I understand you want to:
- [Change 1]
- [Change 2]

I plan to update the plan by:
1. [Specific modification]
2. [Specific modification]

Does this align with your intent?
```

### Step 5: Update the Plan
- Make focused, precise edits
- Maintain existing structure
- Preserve completed work
- Update success criteria if scope changed

### Step 6: Present Changes

```
I've updated the plan at `plans/[filename].md`

**Changes made:**
1. [Specific change]
2. [Specific change]

**Impact:**
- [How this affects implementation]

Would you like any further adjustments?
```

## Guidelines

1. **Be Skeptical**: Question vague feedback, verify feasibility
2. **Be Surgical**: Make precise edits, preserve good content
3. **Respect Completed Work**: Don't modify completed phases without good reason
