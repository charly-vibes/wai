---
name: tdd
description: Implement features following Test-Driven Development methodology. Red-Green-Refactor cycle with phased approach and verification at each step.
---

# Implement with TDD Workflow

Implement the requested feature following Test-Driven Development and a phased approach.

## Core Principle: Red, Green, Refactor

**Red:** Write a failing test that describes desired behavior
**Green:** Write minimal code to make the test pass
**Refactor:** Clean up code while keeping tests green

## Process

### Phase 0: Plan (If needed)

For complex features, create a plan first with:
- Current State → Desired End State
- Out of Scope
- Phases with Changes Required, Tests, and Success Criteria

**Get plan approved before proceeding.**

### For Each Phase:

#### Step 1: Write Failing Test (RED)

```typescript
describe('UserAuthentication', () => {
  it('should validate JWT tokens', async () => {
    const result = await validateToken('valid.jwt.token')
    expect(result).toEqual({ userId: '123', valid: true })
  })
})
```

Run test: **Confirm it fails**

#### Step 2: Implement Minimal Code (GREEN)

Write just enough code to pass the test. Run test: **Confirm it passes**

#### Step 3: Refactor If Needed

Clean up while tests stay green.

#### Step 4: Verify Phase Completion

```bash
npm test
npm run type-check
npm run lint
```

#### Step 5: Inform User

```
Phase 1 Complete - Ready for Verification

Automated verification:
- [x] Tests pass
- [x] Type checking passes

Manual verification needed:
- [ ] [Manual step]

Let me know when verified so I can proceed to Phase 2.
```

**Wait for user confirmation before proceeding.**

## Key Guidelines

1. **Tests first, always**
2. **Minimal implementation** - just enough to pass
3. **One phase at a time**
4. **Keep tests green** - never commit with failing tests
5. **Update plan** - check off completed items
