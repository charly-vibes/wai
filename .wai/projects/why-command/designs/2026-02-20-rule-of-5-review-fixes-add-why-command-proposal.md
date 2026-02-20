Rule of 5 Review Fixes - add-why-command Proposal

## Summary

Applied all 29 issues from Rule of 5 review to the OpenSpec proposal for  command.

## Critical Fixes (2)

### [CORR-002] Auto-detection order conflicts with explicit config
**Fixed**: Changed priority to: 1. Explicit config, 2. Auto-detect (ANTHROPIC_API_KEY, ollama)
- **Location**: specs/reasoning-oracle/spec.md, design.md
- **Impact**: Users can now override auto-detection with explicit config

### [EDGE-001] No handling for empty artifact sets
**Fixed**: Added scenario for zero artifacts with warning and disclaimer
- **Location**: specs/reasoning-oracle/spec.md
- **Change**: Shows warning, proceeds with git-only context, includes disclaimer in LLM response

## High Priority Fixes (8)

### [EDGE-003] Prompt size exceeds LLM context window
**Fixed**: Added context size management requirement
- Truncates to 50 most recent artifacts when >100K tokens
- Shows info message to user

### [CORR-001] File path detection too naive
**Fixed**: Robust heuristic with multiple checks
- File exists, starts with ./ or src/, contains / without spaces
- Handles spaces in paths, version numbers with dots

### [EDGE-002] Git commands fail in non-git repository
**Fixed**: Added git failure handling scenario
- Checks if git repo exists, file is tracked
- Shows helpful message, proceeds with artifacts only

### [CORR-003] Privacy warning message confusing
**Fixed**: Clarified privacy notice wording
- Better acknowledgment flow
- Separated API key from privacy concerns

### [EXCL-002] Missing concrete example queries
**Fixed**: Added 7 example query types to spec
- Natural language, file paths, design decisions, rejections
- Helps implementers understand use cases

### [EXCL-003] No error diagnostic codes
**Fixed**: Added miette error codes for all failure modes
- wai::llm::invalid_api_key, rate_limit, network_error, model_not_found
- Consistent with project patterns

### [EXCL-006] Privacy notice tracking not specified
**Fixed**: Added  tracking
- Stored in .wai/config.toml
- One-time notice on first external API use

### [CORR-005] Prompt injection risk
**Fixed**: Added artifact escaping requirement
- Escape markdown, prevent injection
- Use code blocks for artifact content

## Medium Priority Fixes (12)

1. **[CLAR-001]** - Defined "oracle" term in spec purpose section
2. **[CORR-004]** - Clarified cost varies with project size
3. **[CLAR-003]** - Defined "cost-effective" as <$0.01/query
4. **[CLAR-002]** - Clarified "available" means API responds or model can load
5. **[CLAR-004]** - Documented model alias mapping (haiku → claude-haiku-3-5)
6. **[CLAR-005]** - Fixed relevance scores (parse from LLM High/Medium/Low)
7. **[EDGE-006]** - Added API key in config support (api_key field)
8. **[EDGE-005]** - Added malformed LLM response handling
9. **[EDGE-008]** - Added rate limit handling (no auto-retry)
10. **[DRAFT-001]** - Added cross-reference to timeline-search spec
11. **[EXCL-001]** - Added CLAUDE.md to affected files
12. **[EXCL-007]** - Moved open questions to "Future Enhancements"

## Low Priority Fixes (7)

1. **[DRAFT-002]** - Restructured tasks.md with MVP/Production/Polish sections
2. **[CORR-006]** - Clarified "prioritizing" means metadata for LLM
3. **[CLAR-006]** - Clarified "clickable" means file:line format
4. **[EDGE-004]** - Updated detection for paths with spaces
5. **[EDGE-007]** - Added Ollama model download detection
6. **[EXCL-004]** - Clarified success metrics measurement approach
7. **[EXCL-005]** - Added emoji fallback note

## Files Updated

- 
  - Added 8 new scenarios
  - Enhanced 5 existing scenarios
  - Added Purpose clarification and cross-reference

- 
  - Fixed backend selection priority
  - Clarified cost analysis
  - Moved open questions to Future Enhancements section

- 
  - Added affected docs section
  - Added error.rs to affected code

- 
  - Restructured into MVP (1-5), Production (6-8), Polish (9)
  - Added 12 new task items for fixes
  - Enhanced 8 existing tasks

## Validation

All changes validated with:
```
openspec validate add-why-command --strict
✓ Change 'add-why-command' is valid
```

## Impact

- **Completeness**: Spec now covers all edge cases and failure modes
- **Clarity**: Terminology defined, examples provided, ambiguities resolved
- **Correctness**: Logic bugs fixed (config priority, file detection, context limits)
- **Excellence**: Error codes, privacy tracking, accessibility considerations

The proposal is now ready for implementation with high confidence.

