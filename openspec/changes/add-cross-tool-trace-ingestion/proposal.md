# Change: Add cross-tool trace ingestion and workflow ergonomics

## Why

Wai works best today inside Claude Code, but trace evidence from Claude Code, Codex, Gemini CLI, and AmpCode shows that the workflow degrades outside the best-supported environment. The biggest gaps are:

- reflection depends on manually supplied conversation transcripts even though real traces already exist in local tool stores
- non-Claude tools do not consistently start from `wai status`, `wai search`, `wai prime`, and `wai close`
- long review and remediation sessions devolve into manual loops like `continue`, `fix all`, and repeated confirmations instead of a wai-guided next-step flow
- AmpCode currently exposes only file-change snapshots, leaving wai unable to learn from or reflect on those sessions in the same way as full transcript tools

Without first-class trace support and clearer cross-tool guidance, wai remains strongest as a Claude Code companion rather than a tool-agnostic workflow manager.

## What Changes

- Add a new cross-tool trace ingestion capability for discovering, normalizing, and importing local traces from supported agent tools
- Add `wai trace` commands for trace discovery and import
- Extend `wai reflect` with automatic trace selection so users can reflect on recent work without manually locating transcript files
- Define reduced `diff-only` trace handling for tools like AmpCode that expose file-change events but not full conversations
- Improve generated managed-block and onboarding guidance so non-Claude tools are nudged toward the intended `status` → `search` → `prime` / `close` workflow
- Add context-aware suggestions that detect review-heavy or research-heavy work and recommend pipelines or follow-on capture steps instead of relying on manual `continue` / `fix all` loops

## Impact

- Affected specs: `cli-core`, `project-reflection`, `managed-block`, `onboarding`, `context-suggestions`, new `trace-ingestion`
- Affected code: CLI command definitions, reflection context gathering, managed block generation, workflow suggestion engine, tool-specific trace readers and normalization layer
