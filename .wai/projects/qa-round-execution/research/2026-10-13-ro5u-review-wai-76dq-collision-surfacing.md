---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-13-wai-76dq, pipeline-step:ro5-review]
---

# RO5U review: wai-76dq (collision surfacing)

## Critical / High
(none)

## Medium
1. Provenance match uses substring contains('wip'|'parked'|'owns') — 'owns' matches 'towns'/'owners', 'wip' matches 'wiping'. Advisory-only so cost is noise, but tighten to regex word-boundary match \b(wip|parked|owns)\b case-insensitive (regex crate already a dependency).

## Low
1. render_empty_state hardcodes collision_warnings: Vec::new() — the scan could still run project-less; acceptable (empty state has no project context; stash scan needs project root for cwd anyway).
2. cmd_next prints warnings on every advance — by design (task 4.3), advisory.

Checks: git absent → silent (Ok-guard); git failure → silent; lock parse is TOML via toml::from_str matching read_artifact_lock; expiry TTL = 7 days constant; JSON field skip-when-empty; never hard-fails asserted in all 4 tests.

