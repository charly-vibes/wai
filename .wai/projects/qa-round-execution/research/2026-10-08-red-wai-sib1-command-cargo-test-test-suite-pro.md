---
tags: [pipeline-run:epic-autonomy-tdd-ro5-2026-10-08-wai-sib1, pipeline-step:red-or-analysis]
---

RED: wai-sib1; command=cargo test --test suite_project_use_shell_flag; expected failure=2/3 fail with clap error 'unexpected argument --shell' (fish-override + posix-override), 1/3 passes incidentally (unknown shell rejected by clap); failure is missing behavior not setup breakage
