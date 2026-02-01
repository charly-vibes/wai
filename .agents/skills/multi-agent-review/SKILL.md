---
name: multi-agent-review
description: Perform comprehensive multi-agent parallel code review using Wave/Gate architecture. Simulates security, performance, maintainability, requirements, and operations reviewers.
---

# Multi-Agent Parallel Code Review

Perform a comprehensive multi-agent parallel code review using the Wave/Gate architecture.

CODE TO REVIEW:
[paste code or specify: "Review all files in src/auth/"]

## ARCHITECTURE
Wave 1 (Parallel) → Gate 1 (Sequential) → Wave 2 (Parallel) → Gate 2 (Sequential) → Wave 3 (Sequential)

## WAVE 1: PARALLEL INDEPENDENT ANALYSIS

Launch 5 parallel tasks:

**TASK 1: Security Review** - OWASP Top 10, input validation, auth flaws, secret management
**TASK 2: Performance Review** - Time complexity, N+1 queries, memory allocation
**TASK 3: Maintainability Review** - Readability, patterns, naming, tech debt
**TASK 4: Requirements Validation** - Coverage, edge cases, test gaps
**TASK 5: Operations Review** - Failure modes, logging, observability, resilience

## GATE 1: CONFLICT RESOLUTION

After all Wave 1 tasks complete:
1. Deduplicate issues
2. Resolve severity conflicts (Security CRITICAL always wins)
3. Calculate confidence based on multi-reviewer agreement

## WAVE 2: PARALLEL CROSS-VALIDATION

**TASK 7: Meta-Review** - Coverage gaps, false positives, severity calibration
**TASK 8: Integration Analysis** - System-wide impacts, cascading failures

## GATE 2: FINAL SYNTHESIS

1. Executive summary with blockers
2. Prioritized action list
3. Blocking assessment: BLOCKS_MERGE, BLOCKS_DEPLOY, APPROVED, APPROVED_WITH_NOTES

## WAVE 3: CONVERGENCE CHECK

**CONVERGED** if: new_critical_count == 0 AND new_issue_rate < 0.10 AND false_positive_rate < 0.20
**ESCALATE_TO_HUMAN** if: iteration >= 3 OR conflicting CRITICAL issues
**ITERATE** if: new_critical_count > 0 OR new_issue_rate >= 0.10
