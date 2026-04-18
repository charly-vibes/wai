## Context

Wai already treats conversation transcripts as the highest-fidelity reflection input, but current workflow assumes the user passes `--conversation <file>` manually. Real traces instead live in tool-specific stores such as `~/.claude/projects/...`, `~/.codex/sessions/...`, `~/.gemini/tmp/...`, and AmpCode file-change directories.

These stores differ significantly:

- Claude Code: rich per-session JSONL with user, assistant, and tool-use events
- Codex: rollout/session JSONL with explicit tool calls and environment context
- Gemini CLI: per-project chat JSON plus summary logs
- AmpCode: file-change snapshots without a full conversational record

The design must improve tool-agnostic workflow support without breaking wai's offline-first, file-based model.

## Goals / Non-Goals

Goals:
- Let wai discover and normalize local traces from supported tools without manual transcript hunting
- Make `wai reflect` able to use recent local traces directly
- Support degraded but explicit handling of diff-only tools like AmpCode
- Strengthen cross-tool workflow nudges so agents converge on the intended wai loop

Non-Goals:
- Real-time streaming ingestion from running tools
- Cloud trace collection or remote telemetry
- Full semantic parity across every tool's trace format
- Implementing all future tool integrations in this change

## Decisions

- Decision: introduce a normalized trace model with source adapters
  - Each adapter converts native local storage into a common shape: source tool, workspace path, session id, timestamps, event list, and fidelity tier
  - Alternatives considered: direct per-command special cases in `wai reflect`
  - Rationale: reflection, suggestions, and future analytics should share one model instead of re-parsing each source differently

- Decision: define fidelity tiers explicitly
  - `full-transcript`: user/assistant/tool events available
  - `partial-transcript`: text-oriented logs without full event structure
  - `diff-only`: file-change evidence with no reliable conversational sequence
  - Alternatives considered: pretending all imports are plain text transcripts
  - Rationale: avoids overstating what a source can support, especially for AmpCode

- Decision: keep trace ingestion local and on-demand
  - `wai trace` inspects local stores only and writes imported artifacts into `.wai/`
  - Alternatives considered: background scanning or persistent indexing daemons
  - Rationale: preserves offline-first and keeps simple commands fast by avoiding mandatory startup scans

- Decision: allow `wai reflect` automatic trace selection, but keep explicit override
  - `wai reflect --conversation <file>` remains authoritative when provided
  - Automatic mode chooses the highest-ranked recent trace matching the current repo
  - Alternatives considered: always requiring a manual file path
  - Rationale: trace evidence shows the manual step is the main source of friction

- Decision: use AmpCode traces as evidence, not transcripts
  - Diff-only traces can feed reflection and suggestions with facts such as touched files, timing clusters, and edit sequences, but not quoted user intent
  - Alternatives considered: excluding AmpCode entirely
  - Rationale: some signal is better than none, but the system must be honest about fidelity

- Decision: strengthen workflow ergonomics through generated guidance and suggestions instead of source-specific prompts alone
  - Managed blocks and onboarding should reinforce `wai status`, `wai search`, `wai prime`, and `wai close`
  - Suggestions should detect “review / fix / continue” loops and propose pipelines or artifact capture
  - Alternatives considered: leaving cross-tool guidance to external prompt repos
  - Rationale: these are core wai adoption issues, not merely prompt-packaging issues

## Risks / Trade-offs

- Risk: local trace formats change upstream
  - Mitigation: source adapters should fail soft, expose source-specific diagnostics, and mark unsupported formats clearly

- Risk: scanning home directories could slow commands down
  - Mitigation: `wai trace` performs targeted discovery; `wai reflect` automatic mode checks known tool roots and filters by current repo path before deeper reads

- Risk: privacy concerns around importing local traces
  - Mitigation: trace import stays local, requires explicit user action or explicit auto-selection in current repo context, and reports source files before import when requested

- Risk: suggestion heuristics become noisy
  - Mitigation: only trigger review/remediation nudges from repeated high-confidence patterns and keep them informational

## Migration Plan

1. Add the trace-ingestion capability and CLI surface.
2. Extend reflection context gathering to consume normalized traces.
3. Update managed blocks and onboarding text.
4. Extend workflow suggestions for review/remediation and pipeline nudges.
5. Add adapter coverage incrementally per supported tool.

## Open Questions

- Should imported traces always be materialized as `.wai/resources/traces/...` files, or can some flows remain ephemeral during reflect?
- Should `wai trace import` support importing multiple sessions at once, or should MVP stay single-session plus latest-auto mode?
- Should AmpCode diff-only imports create a distinct artifact type, or reuse research/reflection-support files under a new trace resource directory?
