//! Repo-convention checks for `wai way`.

mod ai;
mod core;
mod coverage;
mod docs;

pub(crate) use ai::{
    check_agent_config_sync, check_agent_skills, check_ai_instructions, check_ubiquitous_language,
};
pub(crate) use core::{
    check_beads, check_ci_cd, check_devcontainer, check_editorconfig, check_gh_cli, check_llm_txt,
    check_openspec, check_pretender, check_task_runner, detect_doc_tool,
};
pub(crate) use coverage::check_test_coverage;
pub(crate) use docs::{
    check_artifact_stubs, check_docs_openspec_inclusion, check_docs_status_page,
    check_documentation,
};
