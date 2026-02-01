---
name: rule-of-5-universal
description: Apply Steve Yegge's Rule of 5 iterative review to any artifact - code, plans, research, issues, specs, or documents. Five stages from draft through excellence.
---

# Universal Rule of 5 Review

Review this [CODE/PLAN/RESEARCH/ISSUE/SPEC/DOCUMENT] using Steve Yegge's Rule of 5 - five stages of iterative editorial refinement until convergence.

## Core Philosophy
"Breadth-first exploration, then editorial passes"

## Stage 1: DRAFT - Get the shape right
**Question:** Is the overall approach sound?
Focus: Structure, major issues, solving right problem, scope

## Stage 2: CORRECTNESS - Is the logic sound?
**Question:** Are there errors, bugs, or logical flaws?
Focus: Factual accuracy, logical consistency, internal contradictions

**Convergence Check after Stage 2**

## Stage 3: CLARITY - Can someone else understand this?
**Question:** Is this comprehensible?
Focus: Readability, unclear language, jargon, naming, flow

**Convergence Check after Stage 3**

## Stage 4: EDGE CASES - What could go wrong?
**Question:** Are boundary conditions handled?
Focus: Edge cases, error handling, unusual scenarios, assumptions

**Convergence Check after Stage 4**

## Stage 5: EXCELLENCE - Ready to ship?
**Question:** Would you be proud to ship this?
Focus: Final polish, best practices, performance, completeness

## Convergence Criteria

**CONVERGED** if: No new CRITICAL AND new issue rate < 10% AND false positive rate < 20%
**ESCALATE_TO_HUMAN** if: After 5 stages still finding CRITICAL OR uncertain about severity

## Final Report

```
# Rule of 5 Review - Final Report

**Work Reviewed:** [type] - [path]
**Convergence:** Stage [N]

## Summary
Total Issues by Severity: CRITICAL, HIGH, MEDIUM, LOW

## Top 3 Critical Findings
[With impact and fix]

## Verdict
[READY | NEEDS_REVISION | NEEDS_REWORK | NOT_READY]
```
