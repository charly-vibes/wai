# OpenSpec Tasks: Agnostic Way Capabilities

**Change ID:** `refactor-way-agnostic`
**Status:** `draft`
**Author:** `Gemini CLI`

## Tasks

1. **[agnostic-capabilities-1]** Update `CheckResult` struct in `src/commands/way.rs` to include `intent` and `success_criteria` fields.
2. **[agnostic-capabilities-2]** Update `render_human` in `src/commands/way.rs` to optionally show `intent` and `success_criteria` when the `--verbose` flag is used.
3. **[agnostic-capabilities-3]** Refactor `check_task_runner` to use "Command standardization" and include its intent/success criteria.
4. **[agnostic-capabilities-4]** Refactor `check_git_hooks` to use "Pre-commit quality gates" and include its intent/success criteria.
5. **[agnostic-capabilities-5]** Refactor `check_editorconfig` to use "Consistent formatting" and include its intent/success criteria.
6. **[agnostic-capabilities-6]** Refactor `check_documentation` to use "Project documentation" and include its intent/success criteria.
7. **[agnostic-capabilities-7]** Refactor `check_ai_instructions` to use "AI-agent context" and include its intent/success criteria.
8. **[agnostic-capabilities-8]** Refactor `check_ci_cd` to use "Automated verification" and include its intent/success criteria.
9. **[agnostic-capabilities-9]** Refactor `check_devcontainer` to use "Reproducible environments" and include its intent/success criteria.
10. **[agnostic-capabilities-10]** Refactor `check_llm_txt` to use "LLM-friendly context" and include its intent/success criteria.
11. **[agnostic-capabilities-11]** Refactor `check_agent_skills` to use "Extended agent capabilities" and include its intent/success criteria.
12. **[agnostic-capabilities-12]** Refactor `check_release_pipeline` to use "Automated delivery" and include its intent/success criteria.
13. **[agnostic-capabilities-13]** Refactor `check_gh_cli` to use "Integration & automation" and include its intent/success criteria.
14. **[agnostic-capabilities-14]** Update YAML plugin parser in `src/plugin.rs` (or similar) to support `intent` and `success_criteria` fields.
15. **[agnostic-capabilities-15]** Update `repository-best-practices` specification in `openspec/specs/repository-best-practices/spec.md` to reflect the new agnostic naming, intent, and success criteria for all requirements.
16. **[agnostic-capabilities-16]** Validate the entire change with `openspec validate refactor-way-agnostic --strict`.
