---
tags: [pipeline-run:epic-orchestrator-2026-10-08-wai-qjcz, pipeline-step:spawn]
---

REVIEW wai-qjcz: template convention duplication — duplication is real (8 convention blocks, C1-C8) but TOML format has no include/shared-block mechanism (mod.rs:309-310 render_prompt only substitutes {topic}; definition.rs:32-55 plain toml::from_str, no preprocessing; setup.rs:66 verbatim copy). No code changes. Ratchet green: 8/3/34 passed. Verdict: extraction requires engine preprocessing feature — left to orchestrator follow-up.
