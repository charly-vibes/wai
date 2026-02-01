---
name: research-review
description: Perform iterative research document review using Rule of 5 methodology. Reviews accuracy, completeness, clarity, actionability, and integration with project context.
---

# Iterative Research Review (Rule of 5)

Perform thorough research document review using the Rule of 5 - iterative refinement until convergence.

## Setup

**If research document path provided:** Read the document completely
**If no path:** Ask for the research document path

## Process

Perform 5 passes, each focusing on different aspects.

### PASS 1 - Accuracy & Sources
Focus: Claims backed by evidence, source credibility, correct code references

### PASS 2 - Completeness & Scope
Focus: Missing topics, unanswered questions, appropriate depth

### PASS 3 - Clarity & Structure
Focus: Logical flow, clear definitions, readability, consistent terminology

### PASS 4 - Actionability & Conclusions
Focus: Clear takeaways, conclusions supported by findings, practical applicability

### PASS 5 - Integration & Context
Focus: Alignment with existing research, connections to specs, contradictions with decisions

## Convergence Check

After each pass (starting with pass 2):
- **CONVERGED**: No new CRITICAL, <10% new issues vs previous pass
- **ITERATE**: Continue to next pass
- **NEEDS_HUMAN**: Found blocking issues requiring human judgment

## Final Report

```
## Research Review Final Report

**Research:** [path/to/research.md]

### Summary
- CRITICAL: [count]
- HIGH: [count]

### Top 3 Most Critical Findings
[With impact and fix]

### Verdict
[READY | NEEDS_REVISION | NEEDS_MORE_RESEARCH]

### Research Quality Assessment
- Accuracy: [Excellent|Good|Fair|Poor]
- Completeness: [Excellent|Good|Fair|Poor]
- Actionability: [Excellent|Good|Fair|Poor]
- Clarity: [Excellent|Good|Fair|Poor]
```
