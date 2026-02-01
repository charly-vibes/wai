---
name: design-review
description: Review design artifacts (problem statements, decision matrices, scope documents) for rigor, completeness, and adherence to Design in Practice principles.
---

# Design Artifact Review

Review design artifacts for rigor, completeness, and adherence to Design in Practice principles.

## Setup

**If artifact path provided:** Read the artifact completely
**If no path:** Ask which artifact to review

## Review Framework

Perform targeted review based on artifact type.

---

## ARTIFACT TYPE: Problem Statement

### Check 1: Solution Contamination
Does the problem statement contain or imply solutions?

### Check 2: Root Cause vs. Symptom
Does this describe the mechanism, or just the observable effect?

### Check 3: Specificity
Is the problem precise enough to guide solution selection?

### Check 4: Evidence Quality
Is the diagnosis based on verified facts or assumptions?

### Check 5: The Obvious Solution Test
Does the solution feel obvious after reading this statement?

---

## ARTIFACT TYPE: Decision Matrix

### Check 1: Status Quo Baseline
Is "do nothing" the first column?

### Check 2: Fact vs. Judgment Separation
Is cell text factual and neutral, with judgment shown separately?

### Check 3: Criteria Completeness
Are all relevant trade-off dimensions represented?

### Check 4: Approach Diversity
Are the approaches fundamentally different, or variations of the same idea?

### Check 5: Cell Verification
Are the facts in the matrix verifiable and accurate?

---

## Final Report Template

```
## Design Artifact Review Report

**Artifact(s) Reviewed:** [List]

### Summary

| Artifact | Verdict | Key Issues |
|----------|---------|------------|
| Problem Statement | [READY/REVISION/BACK] | [Top issue] |
| Decision Matrix | [READY/REVISION/MORE] | [Top issue] |

### Critical Findings

1. **[ARTIFACT-CHECK]** [Severity]
   - Issue: [What's wrong]
   - Impact: [Why it matters]
   - Fix: [Specific action]

### Overall Verdict

[PROCEED_TO_IMPLEMENTATION | REVISE_ARTIFACTS | RETURN_TO_EARLIER_PHASE]
```
