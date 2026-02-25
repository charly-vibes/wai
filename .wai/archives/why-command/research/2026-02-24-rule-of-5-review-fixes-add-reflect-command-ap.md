# Rule-of-5 Review Fixes — add-reflect-command

Applied all fixes from a Rule-of-5-Universal review of the initial add-reflect-command
proposal, plus three explicit user requirements.

## User Requirements Added
1. AGENTS.md support alongside CLAUDE.md
2. Claude conversation transcripts as first-class input source
3. Handoff quality improvement (nuance capture sections)

## 19 Issues Found and Fixed

### CRITICAL (both user-driven)
- [DRAFT-001] AGENTS.md not supported → added --output flag, both-target default
- [DRAFT-002] Conversation input discarded → added --conversation <file> flag

### HIGH
- [DRAFT-003] No handoff quality improvement → added Gotchas & Surprises and
  'What Took Longer Than Expected' sections to handoff template; spec delta added
- [CORR-001] Nudge heuristic via CLAUDE.md mtime fragile → .reflect-meta TOML file
- [CORR-002] Fixed 50/50 context split wrong → dynamic three-tier budget:
  conversation (~30K) > handoffs (~40K) > artifacts (~30K), greedy fill
- [CLAR-001] --output flag missing from spec/tasks → added to all files
- [CLAR-002] Conversation format vague → plain text, any format, LLM extracts
- [EDGE-001] Multi-project behavior unspecified → default: aggregate all projects
- [EDGE-002] Silent REFLECT overwrite footgun → unified diff shown before confirm

### MEDIUM
- [CORR-004] Circular fallback 'suggest wai why' when no LLM → suggest LLM setup
- [CLAR-003] AGENTS.md block placement unspecified → append at end of file
- [CLAR-004] session vs handoff file count ambiguity → .reflect-meta makes it explicit
- [EDGE-003] No target file exists → clear diagnostic error
- [EDGE-004] Conversation file size limit unspecified → 30K chars, truncate from top
- [EDGE-005] Old handoffs describing stale patterns → LLM instructed to flag >6mo artifacts
- [EDGE-006] REFLECT block placement in AGENTS.md → at end of file

### LOW
- [CLAR-005] Hardcoded confirm prompt string → dynamic (lists actual target files)
- [EXCL-003] Cost estimate too precise → range /usr/bin/zsh.005-/usr/bin/zsh.03 depending on transcript size
- [EXCL-004] wai doctor stale reflect check → added to tasks Phase 8

## Key Design Insight Elevated
Three-tier input hierarchy (proposal.md Why section now explains):
1. Conversation transcripts — richest: failed commands, surprises, trial-and-error
2. Handoffs — distilled: intent, next steps, explicit gotchas
3. Research/design/plan — curated: explicit decisions, domain knowledge
The reflect command is only as good as its inputs, so handoff quality improvement
is not a nice-to-have — it is load-bearing.

