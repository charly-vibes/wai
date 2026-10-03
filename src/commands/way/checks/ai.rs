//! Repo-convention checks for `wai way`.

use genesis::doctor::CheckStatus;
use std::path::Path;

use crate::commands::resource::parse_skill_frontmatter;
use crate::commands::way::WayCheckEntry;
use crate::config::{SKILLS_DIR, agent_config_dir};
use std::collections::HashSet;

fn has_reflection_resources(repo_root: &Path) -> bool {
    let refl_dir = crate::config::reflections_dir(repo_root);
    refl_dir
        .exists()
        .then(|| std::fs::read_dir(&refl_dir).ok())
        .flatten()
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

pub(crate) fn check_ai_instructions(repo_root: &Path) -> WayCheckEntry {
    let name = "AI-agent context";
    let intent = Some(
        "Provide persistent \"rules of the road\" and project context for AI collaborators."
            .to_string(),
    );
    let success_criteria = Some(
        "Persistent instructions define coding standards and context for AI assistants."
            .to_string(),
    );

    let claude_md = repo_root.join("CLAUDE.md");
    let agents_md = repo_root.join("AGENTS.md");

    if claude_md.exists() {
        // Check whether wai reflect has been run: a reflection resource file
        // must exist in .wai/resources/reflections/. The old WAI:REFLECT inline
        // block is no longer written — reflect now writes to a resource file and
        // injects a slim WAI:REFLECT:REF reference block instead.
        let suggestion = if !has_reflection_resources(repo_root) {
            Some(
                "No reflection resource found — run `wai reflect` to synthesize project-specific AI guidance into .wai/resources/reflections/".to_string(),
            )
        } else {
            None
        };
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "CLAUDE.md detected (recommended for Claude Code)".to_string(),
            intent,
            success_criteria,
            suggestion,
        }
    } else if agents_md.exists() {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "AGENTS.md detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Consider adding CLAUDE.md for Claude Code compatibility".to_string()),
        }
    } else {
        WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: "No AI instruction files detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Create CLAUDE.md to provide context to AI assistants".to_string()),
        }
    }
}

fn count_context_files(contexts_dir: &Path) -> usize {
    std::fs::read_dir(contexts_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
                .count()
        })
        .unwrap_or(0)
}

fn ubiquitous_language_outcome(root: &Path) -> (CheckStatus, String, Option<String>) {
    let readme = root.join("README.md");
    let shared = root.join("shared.md");
    let context_file_count = count_context_files(&root.join("contexts"));

    let missing_contexts = "Add bounded-context files under .wai/resources/ubiquitous-language/contexts/ to complete the progressive-disclosure layout".to_string();
    let base_suggestion = "Create .wai/resources/ubiquitous-language/ with README.md as the root index and bounded-context files under contexts/".to_string();

    if readme.exists() && context_file_count > 0 {
        return (
            CheckStatus::Pass,
            format!(
                "Ubiquitous-language tree fully configured with {} bounded-context file(s)",
                context_file_count
            ),
            None,
        );
    }

    if readme.exists() && shared.exists() {
        return (
            CheckStatus::Warn,
            "Root index and shared terms exist — valid starting point, but bounded-context files are still missing".to_string(),
            Some(missing_contexts),
        );
    }

    if readme.exists() {
        return (
            CheckStatus::Warn,
            "Root index exists, but bounded-context files are still missing".to_string(),
            Some(missing_contexts),
        );
    }

    if root.exists() && context_file_count > 0 {
        return (
            CheckStatus::Warn,
            "README.md is required as the root index before bounded-context files can be loaded safely".to_string(),
            Some(base_suggestion),
        );
    }

    (
        CheckStatus::Warn,
        "No ubiquitous-language resource tree detected".to_string(),
        Some(base_suggestion),
    )
}

pub(crate) fn check_ubiquitous_language(repo_root: &Path) -> WayCheckEntry {
    let name = "Ubiquitous language context";
    let intent = Some(
        "Provide a canonical, machine-readable source of domain terminology so humans and agents use the same language."
            .to_string(),
    );
    let success_criteria = Some(
        "A progressively disclosed ubiquitous-language resource tree exists with a lightweight index and bounded-context term files."
            .to_string(),
    );
    let mk = |status: CheckStatus, message: String, suggestion: Option<String>| -> WayCheckEntry {
        WayCheckEntry {
            name: name.to_string(),
            status,
            message,
            intent,
            success_criteria,
            suggestion,
        }
    };

    let root = repo_root.join(".wai/resources/ubiquitous-language");
    let (status, message, suggestion) = ubiquitous_language_outcome(&root);
    mk(status, message, suggestion)
}
fn collect_skill_ids(skills_dir: &Path) -> (HashSet<String>, usize) {
    let mut skill_ids: HashSet<String> = HashSet::new();
    let mut skill_count = 0usize;
    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return (skill_ids, skill_count);
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let skill_file = entry.path().join("SKILL.md");
        if !skill_file.exists() {
            continue;
        }
        skill_count += 1;
        if let Some(dir_name) = entry.file_name().to_str() {
            skill_ids.insert(dir_name.to_string());
        }
        if let Some(meta) = parse_skill_frontmatter(&skill_file) {
            for alias in meta.aliases {
                skill_ids.insert(alias);
            }
        }
    }
    (skill_ids, skill_count)
}

fn missing_recommended(skill_ids: &HashSet<String>) -> Vec<&'static str> {
    let has_ro5 = skill_ids.contains("rule-of-5-universal") || skill_ids.contains("ro5");
    let has_commit = skill_ids.contains("commit");
    [
        (!has_ro5).then_some("rule-of-5-universal (ro5)"),
        (!has_commit).then_some("commit"),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn agent_skills_entry(
    status: CheckStatus,
    message: String,
    suggestion: Option<String>,
) -> WayCheckEntry {
    WayCheckEntry {
        name: "Extended agent capabilities".to_string(),
        status,
        message,
        intent: Some(
            "Enhance agent functionality with specialized iterative review and commit workflows."
                .to_string(),
        ),
        success_criteria: Some(
            "Specialized agent workflows (Rule of 5, Deliberate Commits) are active.".to_string(),
        ),
        suggestion,
    }
}

pub(crate) fn check_agent_skills(repo_root: &Path) -> WayCheckEntry {
    const FIX_SKILLS: &str =
        "Run 'wai way --fix skills' to scaffold rule-of-5-universal (ro5) and commit";

    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);

    if !skills_dir.exists() {
        return agent_skills_entry(
            CheckStatus::Warn,
            "No skills configured".to_string(),
            Some(FIX_SKILLS.to_string()),
        );
    }

    // Collect skill dir names and aliases from frontmatter
    let (skill_ids, skill_count) = collect_skill_ids(&skills_dir);

    if skill_count == 0 {
        return agent_skills_entry(
            CheckStatus::Warn,
            "Skills directory present but empty".to_string(),
            Some(FIX_SKILLS.to_string()),
        );
    }

    let missing = missing_recommended(&skill_ids);

    if missing.is_empty() {
        agent_skills_entry(
            CheckStatus::Pass,
            format!(
                "{} skill(s) configured — includes rule-of-5-universal (ro5) and commit",
                skill_count
            ),
            None,
        )
    } else {
        agent_skills_entry(
            CheckStatus::Warn,
            format!(
                "{} skill(s) configured — missing recommended: {}",
                skill_count,
                missing.join(", ")
            ),
            Some(format!(
                "Run 'wai way --fix skills' to scaffold missing: {}",
                missing.join(", ")
            )),
        )
    }
}
pub(crate) fn check_agent_config_sync(repo_root: &Path) -> WayCheckEntry {
    let name = "Agent context sync";
    let intent = Some(
        "Project agent-specific configurations (skills, rules) to tool-specific locations."
            .to_string(),
    );
    let success_criteria = Some(
        "Agent configurations are automatically synced to tool-specific directories (e.g. .agents/, .claude/commands/)."
            .to_string(),
    );

    let agent_config = agent_config_dir(repo_root);
    let projections_path = agent_config.join(".projections.yml");

    if !projections_path.exists() {
        return WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: "No agent config projections found".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Run `wai init` to set up .projections.yml".to_string()),
        };
    }

    // Check if any projections are defined
    let content = std::fs::read_to_string(&projections_path).unwrap_or_default();
    if content.contains("projections: []") || !content.contains("target:") {
        return WayCheckEntry {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: "Projections file exists but no projections are configured".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Edit .wai/resources/agent-config/.projections.yml to add targets like .agents or claude-code"
                    .to_string(),
            ),
        };
    }

    WayCheckEntry {
        name: name.to_string(),
        status: CheckStatus::Pass,
        message: "Agent config projections configured".to_string(),
        intent,
        success_criteria,
        suggestion: Some("Run `wai sync` to apply projections".to_string()),
    }
}
