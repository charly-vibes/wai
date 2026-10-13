---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-px4g, pipeline-step:ro5-review]
---

# RO5U review: wai-px4g (verifier evidence contract)

## Critical
(none)

## High
(none)

## Medium
1. Oracle verdict detection uses grep -Eqi 'CONFIRM' — substring match also hits "confirmation"/"confirmed" in prose, misclassifying an in-progress note as a CONFIRM verdict subject to evidence checks (false fail). Fix: word-boundary match \bCONFIRM\b / \bREVIEW_REQUIRED\b.
2. Oracle cannot distinguish a pasted passing test result from a failing one ('cargo test' mention alone passes). Accepted limitation per design.md Decision 3 (evidence contract lives in the artifact; the verifier is read-only and cross-checks claims separately) — document, do not over-engineer.

## Low
(none — script mirrors tests-pass.sh house style; hermetic_wai_cmd promoted to shared common helper benefits future suites)

Checks performed: scaffold path (write_executable_if_absent keeps exec bit); old epic-orchestrator workspaces get the oracle only on re-init (write-if-absent semantics — acceptable, fresh init covered by test); REVIEW_REQUIRED path always passes (spec: degradation non-blocking); unknown-verdict artifacts fail loudly rather than silently passing; baseline drift reviewed line-by-line (13 lines, all evidence-contract content).

