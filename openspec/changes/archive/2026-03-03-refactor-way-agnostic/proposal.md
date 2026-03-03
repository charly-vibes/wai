# OpenSpec Proposal: Agnostic Way Capabilities

**Change ID:** `refactor-way-agnostic`
**Status:** `draft`
**Author:** `Gemini CLI`

## Summary

Refactor the `wai way` command to use tool-agnostic "Capabilities" instead of hardcoded tool checks. This shift provides clearer "Intent" and "Success Criteria" for both humans and AI agents, allowing them to make informed decisions based on the project context.

## Motivation

The current `wai way` implementation is tightly coupled to specific tools (justfile, prek, goreleaser). While these are good defaults, forcing them or reporting specifically on them can be confusing in ecosystems where other tools are standard (e.g., `Makefile` in C projects, `husky` in Node projects, `nox` in Python). 

By reporting on **Capabilities** (e.g., "Command Standardization") rather than **Tools** (e.g., "Task Runner"), we:
1. Provide a "North Star" for repository health that applies to any language or framework.
2. Enable AI agents to understand *why* a check exists and *what* it aims to achieve.
3. Allow agents to autonomously choose and implement the best tool for the specific project context.

## Proposed Changes

1. **Rename Checks:** Update existing checks to use agnostic capability names (e.g., "Task runner" -> "Command standardization").
2. **Expand Data Model:** Add `intent` and `success_criteria` fields to the `CheckResult` structure.
3. **Update JSON Output:** Include these new fields in the `--json` payload to provide rich context for automated tools.
4. **Verbose Human Output:** Show intent and success criteria in the human-readable output when the `-v` (verbose) flag is used.
5. **Agnostic Suggestions:** Update suggestions to focus on the desired outcome rather than just "installing tool X".

## Impact

- **Users:** Will see more meaningful categories and can understand the rationale behind recommendations via verbose output.
- **AI Agents:** Will receive structured data defining the *purpose* and *criteria* for repository standards, enabling better autonomous decision-making.
- **Backward Compatibility:** Existing `wai way` command and basic output format remain functional. JSON consumers will see new fields but existing fields remain.
