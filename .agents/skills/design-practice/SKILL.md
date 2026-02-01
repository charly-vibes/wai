---
name: design-practice
description: Apply the 6-phase design framework (Describe, Diagnose, Delimit, Direction, Design, Develop) to understand a problem before implementing a solution.
---

# Design in Practice - 6 Phase Framework

Apply the 6-phase design framework to understand a problem before implementing a solution.

## Phases

Work through each phase sequentially. Do not skip phases. The goal is understanding, not artifacts.

### PHASE 1: DESCRIBE (Symptoms)

**Objective:** Capture the reality without imposing solutions.

1. **Gather signals:** What symptoms are observed? Where? When? Who is affected?
2. **Write a neutral description:** Facts only, no speculation on causes, no mention of solutions

**Anti-patterns to avoid:**
- "The database is slow" (diagnosis, not description)
- "We need to add caching" (solution, not description)

### PHASE 2: DIAGNOSE (Root Cause)

**Objective:** Identify the mechanism causing the symptoms.

1. **Generate multiple hypotheses:** List at least 3 possible causes
2. **Test hypotheses systematically:** Use logic to rule things out
3. **Write the Problem Statement**

**Quality check:** The solution should feel obvious after writing the right problem statement.

### PHASE 3: DELIMIT (Scope)

**Objective:** Define what's in and what's out.

1. **Set explicit boundaries:** What subset will we solve? What constraints do we accept?
2. **Document non-goals:** Future considerations, related problems not in scope

### PHASE 4: DIRECTION (Strategic Approach)

**Objective:** Select the best approach from viable alternatives.

1. **Generate multiple approaches:** Always include "do nothing" as baseline
2. **Build a Decision Matrix:** Text is FACT, color is JUDGMENT
3. **Select and justify**

### PHASE 5: DESIGN (Tactical Plan)

**Objective:** Create detailed blueprint for implementation.

Only after Direction is selected:
- Data structures and schemas
- API signatures and contracts
- Component responsibilities
- Error handling approach

**Hammock time:** Write the plan, then sleep on it.

### PHASE 6: DEVELOP (Execute)

**Objective:** Translate design into working code.

If Phases 1-5 were rigorous, this phase should feel mechanical.

## When to Return to Earlier Phases

- **Can't write Problem Statement clearly** → Back to Describe
- **Solution doesn't feel obvious** → Back to Diagnose
- **Scope keeps expanding** → Back to Delimit
- **Implementation keeps hitting surprises** → Back to Design
