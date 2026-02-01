---
name: implement-plan
description: Execute an approved implementation plan following TDD methodology. Works through phases with Red-Green-Refactor cycles, verifying completion at each step.
---

# Implement Plan

Implement an approved plan following Test-Driven Development methodology.

## Getting Started

When given a plan path:
1. Read the plan completely (no limit/offset)
2. Check for existing checkmarks indicating completed work
3. Read related specs/documentation referenced in the plan
4. Create a todo list to track progress through phases
5. Start implementing from the first unchecked phase

## Implementation Philosophy

Follow the **Red, Green, Refactor** cycle:

1. **Red**: Write a failing test for the desired behavior
2. **Green**: Write minimal code to make the test pass
3. **Refactor**: Clean up code while keeping tests green

**Never skip the Red phase** - watching tests fail confirms they actually test something.

## Workflow

### For Each Phase:

#### Step 1: Read Phase Requirements
#### Step 2: Write Tests First (TDD - RED)
#### Step 3: Implement Minimal Code (GREEN)
#### Step 4: Refactor (if needed)
#### Step 5: Run Success Criteria Checks

```bash
npm test
npm run type-check
npm run lint
npm run build
```

#### Step 6: Mark Phase Complete
#### Step 7: Inform User and Wait for Confirmation

```
Phase [N] Complete - Ready for Verification

Automated verification:
- [x] All tests pass
- [x] Type checking passes

Manual verification needed:
- [ ] [Manual step from plan]

Let me know when verified so I can proceed to Phase [N+1].
```

## When Things Don't Match Reality

```
Issue in Phase [N]:

Expected (from plan): [What the plan says]
Found (in codebase): [Actual situation]

Options:
1. Adapt implementation to current reality
2. Update the plan
3. Ask user for guidance
```

## Key Reminders

1. **Tests first, always**
2. **Watch it fail** - Confirm test fails before implementing
3. **Minimal code** - Write just enough to pass
4. **One phase at a time**
5. **Verify thoroughly**
