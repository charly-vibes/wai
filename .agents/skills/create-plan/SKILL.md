---
name: create-plan
description: Create a detailed implementation plan for a feature or task. Researches codebase, presents design options, and produces a phased plan with TDD approach and success criteria.
---

# Create Implementation Plan

Create a detailed implementation plan for the requested feature.

## When Invoked

1. **If a spec file is provided**: Read it fully and begin planning
2. **If no parameters**: Ask for the feature/task description

## Process

### Step 1: Understand the Requirement

1. Read any mentioned spec files completely
2. Check existing research/documentation for related work
3. Understand the scope and constraints
4. Ask clarifying questions if anything is unclear

### Step 2: Research the Codebase

**Create a todo list to track research tasks.**

1. Find relevant existing patterns and code
2. Identify integration points
3. Note conventions to follow
4. Identify files that will need changes

### Step 3: Design Options (if applicable)

If multiple approaches are viable:

1. Present 2-3 design options
2. Include pros/cons for each
3. Recommend an approach (explain why)
4. Get user alignment before detailed planning

### Step 4: Write the Plan

Save to `plans/YYYY-MM-DD-description.md`:

```markdown
# [Feature Name] Implementation Plan

**Date**: YYYY-MM-DD

## Overview

[Brief description of what we're implementing]

## Current State

[What exists now, what's missing, what needs to change]

## Desired End State

[What will exist after implementation]

**How to verify:**
- [Specific verification steps]

## Out of Scope

[What we're explicitly NOT doing]

## Risks & Mitigations

[Identified risks and how we'll handle them]

## Phase 1: [Name]

### Changes Required

**File: `path/to/file.ext`**
- Changes: [Specific modifications needed]
- Tests: [What tests to write first (TDD)]

### Success Criteria

#### Automated:
- [ ] Tests pass
- [ ] Type checking passes
- [ ] Build succeeds

#### Manual:
- [ ] [Specific manual verification step]

---

## Testing Strategy

**Following TDD:**
1. Write tests first for each behavior
2. Watch tests fail (Red)
3. Implement minimal code to pass (Green)
4. Refactor while keeping tests green
```

### Step 5: Review and Iterate

1. Present the plan to the user
2. Highlight key decisions made
3. Iterate based on feedback until approved

## Guidelines

1. **Be specific**: Include actual file paths and concrete changes
2. **Follow TDD**: Plan tests before implementation
3. **Break into phases**: Each phase should be independently verifiable
4. **No open questions**: Resolve all questions before finalizing
