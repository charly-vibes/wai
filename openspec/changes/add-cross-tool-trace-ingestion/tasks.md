## 1. Specification
- [ ] 1.1 Add new `trace-ingestion` capability spec covering trace discovery, normalization, import, and diff-only support
- [ ] 1.2 Modify `cli-core` spec to add `wai trace` commands and `wai reflect` automatic trace selection behavior
- [ ] 1.3 Modify `project-reflection` spec to define auto-detected trace context and trace-priority rules
- [ ] 1.4 Modify `managed-block` and `onboarding` specs to strengthen session-start/session-close guidance for non-Claude tools
- [ ] 1.5 Modify `context-suggestions` spec to add review/remediation and research pipeline suggestions based on observed workflow patterns

## 2. Design
- [ ] 2.1 Define the normalized trace model and source-specific adapters for Claude Code, Codex, Gemini CLI, and AmpCode
- [ ] 2.2 Define trace ranking, privacy boundaries, and fallback behavior when multiple traces match the same repo
- [ ] 2.3 Define how diff-only traces contribute to reflection and suggestion generation without pretending to be full transcripts

## 3. Validation
- [ ] 3.1 Run `openspec validate add-cross-tool-trace-ingestion --strict`
- [ ] 3.2 Resolve all validation issues before requesting approval
