Phase 7 testing complete. Added 6 new tests:
- Unit: gather_context_populates_artifacts_from_tmpdir (7.1 extended)
- Unit: gather_context_marks_file_query_for_existing_path (7.2 extended)
- Unit: full_pipeline_gather_prompt_parse_and_format_json (7.4 mock LLM integration)
- Integration CLI: why_no_llm_flag_falls_back_to_search (7.5)
- Integration CLI: why_auto_fallback_to_search_when_no_llm_available (7.5)
- Integration CLI: why_file_query_in_non_git_repo_does_not_crash (7.6)
Total test count: 64 unit + 135 integration. All pass.
Mock LLM approach: full pipeline tested in unit tests by calling gather_context → build_prompt → parse_response → format_json with a fixed response string, no real LLM call.
Fallback determinism: force_why_llm() helper sets llm='claude' in config to skip Ollama auto-detection, ensuring fallback tests work on any machine.
