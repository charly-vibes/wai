---
tags: [pipeline-run:tdd-ro5-2026-10-19-wai-tdnv, pipeline-step:orient]
---

ORIENT: wai-tdnv tooling layer — wai pipeline spawn (detached pi -p under timeout budget + checkpoint JSON), spawn status (pid-liveness + log probe, one line), spawn resume (pi -c named session, canon inv 6). Doc+template layers done; pure module src/pipeline_spawn.rs mirroring pipeline_wait pattern; hermetic tests via PATH-stubbed pi. Openspec change: add-orchestrator-spawn-wrapper
