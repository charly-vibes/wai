# OpenSpec Tasks: Agnostic Way Capabilities

**Change ID:** `refactor-way-agnostic`
**Status:** `draft`
**Author:** `Gemini CLI`

## Tasks

- [x] **[agnostic-capabilities-1]** Update `CheckResult` struct in `src/commands/way.rs` to include `intent` and `success_criteria` fields.
- [x] **[agnostic-capabilities-2]** Update `render_human` in `src/commands/way.rs` to optionally show `intent` and `success_criteria` when the `--verbose` flag is used.
- [x] **[agnostic-capabilities-3]** Refactor `check_task_runner` to use "Command standardization" and include its intent/success criteria.
- [x] **[agnostic-capabilities-4]** Refactor `check_git_hooks` to use "Pre-commit quality gates" and include its intent/success criteria.
- [x] **[agnostic-capabilities-5]** Refactor `check_editorconfig` to use "Consistent formatting" and include its intent/success criteria.
- [x] **[agnostic-capabilities-6]** Refactor `check_documentation` to use "Project documentation" and include its intent/success criteria.
- [x] **[agnostic-capabilities-7]** Refactor `check_ai_instructions` to use "AI-agent context" and include its intent/success criteria.
- [x] **[agnostic-capabilities-8]** Refactor `check_ci_cd` to use "Automated verification" and include its intent/success criteria.
- [x] **[agnostic-capabilities-9]** Refactor `check_devcontainer` to use "Reproducible environments" and include its intent/success criteria.
- [x] **[agnostic-capabilities-10]** Refactor `check_llm_txt` to use "LLM-friendly context" and include its intent/success criteria.
- [x] **[agnostic-capabilities-11]** Refactor `check_agent_skills` to use "Extended agent capabilities" and include its intent/success criteria.
- [x] **[agnostic-capabilities-12]** Refactor `check_release_pipeline` to use "Automated delivery" and include its intent/success criteria.
- [x] **[agnostic-capabilities-13]** Refactor `check_gh_cli` to use "Integration & automation" and include its intent/success criteria.
- [x] **[agnostic-capabilities-14]** Add `intent` and `success_criteria` fields to `PluginDef` in `src/plugin.rs` with `#[serde(default)]`.
- [x] **[agnostic-capabilities-15]** Update `repository-best-practices` specification in `openspec/specs/repository-best-practices/spec.md` to reflect the new agnostic naming, intent, and success criteria for all requirements.
- [ ] **[agnostic-capabilities-16]** Migrate plugin loader in `src/plugin.rs` from YAML (`*.yml`/`*.yaml`) to TOML (`*.toml`): change extension filter and swap `serde_yml::from_str` for `toml::from_str`.
- [ ] **[agnostic-capabilities-17]** Add integration test `way_verbose_shows_intent`: run `wai way -v` and assert output contains `"Intent:"`.
- [ ] **[agnostic-capabilities-18]** Add integration test `way_verbose_shows_success_criteria`: run `wai way -v` and assert output contains `"Success:"`.
- [ ] **[agnostic-capabilities-19]** Add integration test `way_json_includes_intent`: run `wai way --json` and assert payload contains `"intent"` key.
- [ ] **[agnostic-capabilities-20]** Add integration test `way_plugin_toml_parsed`: place a `.toml` plugin file with `intent`/`success_criteria` in `.wai/plugins/` and assert it is loaded correctly.
- [ ] **[agnostic-capabilities-21]** Update `Plugin Agnostic Context Support` requirement in `openspec/specs/repository-best-practices/spec.md` to use TOML examples and add a scenario for TOML plugin parsing.
- [ ] **[agnostic-capabilities-22]** Run `openspec validate refactor-way-agnostic --strict` and confirm clean.
