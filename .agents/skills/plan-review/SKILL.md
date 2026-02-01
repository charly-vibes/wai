---
name: plan-review
description: Perform iterative implementation plan review using Rule of 5 methodology. Reviews feasibility, completeness, TDD alignment, ordering, and executability.
---

# Iterative Plan Review (Rule of 5)

Perform thorough implementation plan review using the Rule of 5 - iterative refinement until convergence.

## Setup

**If plan path provided:** Read the plan completely
**If no plan path:** Ask for the plan path or list available plans

## Process

Perform 5 passes, each focusing on different aspects.

### PASS 1 - Feasibility & Risk
Focus: Technical feasibility, dependencies, assumptions, blockers, resource requirements

### PASS 2 - Completeness & Scope
Focus: Missing phases, success criteria, "Out of Scope" defined, testing strategy

### PASS 3 - Spec & TDD Alignment
Focus: Tests planned before implementation, success criteria testable, spec coverage

### PASS 4 - Ordering & Dependencies
Focus: Phases in correct order, parallelizable work identified, incremental verification

### PASS 5 - Clarity & Executability
Focus: Specific enough for someone else to implement, no ambiguous instructions

## Convergence Check

After each pass (starting with pass 2):
- **CONVERGED**: No new CRITICAL, <10% new issues vs previous pass
- **ITERATE**: Continue to next pass
- **NEEDS_HUMAN**: Found blocking issues requiring human judgment

## Final Report

```
## Plan Review Final Report

**Plan:** plans/[filename].md

### Summary
- CRITICAL: [count]
- HIGH: [count]
- MEDIUM: [count]
- LOW: [count]

### Top 3 Most Critical Findings
[With impact and fix]

### Verdict
[READY_TO_IMPLEMENT | NEEDS_REVISION | NEEDS_MORE_RESEARCH]
```
