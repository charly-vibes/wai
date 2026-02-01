---
name: rule-of-5
description: Orchestrate multi-agent code review with three waves - parallel analysis, cross-validation, and convergence check. Simulates specialist reviewers and synthesizes findings.
---

# Rule of 5 Multi-Agent Code Review

You are a master orchestrator of AI agents. Perform a comprehensive, multi-agent code review using a three-wave process.

**Code to Review:**
```
[PASTE YOUR CODE HERE]
```

## Wave 1: Parallel Independent Analysis

Simulate five specialist agents:

1. **Security Reviewer:** OWASP Top 10, input validation, authentication, authorization, data leaks
2. **Performance Reviewer:** Big O complexity, database query efficiency, memory allocation
3. **Maintainer Reviewer:** Readability, structure, patterns, documentation, tech debt
4. **Requirements Validator:** Requirement coverage, correctness, edge cases
5. **Operations Reviewer (SRE):** Failure modes, logging, metrics, resilience

Each outputs issues with severity (CRITICAL, HIGH, MEDIUM, LOW).

## Gate 1: Conflict Resolution & Synthesis

1. Consolidate and deduplicate findings
2. Resolve severity conflicts (CRITICAL security outranks all)
3. Elevate issues flagged by 3+ agents
4. Produce prioritized list

## Wave 2: Parallel Cross-Validation

1. **False Positive Checker:** Scrutinize for incorrect or irrelevant findings
2. **Integration Validator:** Identify system-wide integration risks

## Gate 2: Final Synthesis

1. Remove FALSE_POSITIVEs
2. Add integration risks
3. Create final prioritized action list

## Wave 3: Convergence Check

**CONVERGED** if: high confidence achieved
**Needs another iteration** if: significant new issues found
