use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{SKILLS_DIR, agent_config_dir, global_skills_dir};
use crate::context::{current_context, require_safe_mode};

use super::metadata::{SkillEntry, SkillListPayload, SkillSource, parse_skill_frontmatter};
use super::require_project;
use super::validation::validate_skill_name;

pub(super) fn add_skill(name: &str, template_name: Option<&str>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("add skill")?;

    // Validate skill name
    validate_skill_name(name)?;

    // Resolve template body
    let body = match template_name {
        Some(t) => skill_template_body(t)?,
        None => "Instructions go here.\n".to_string(),
    };

    // Build path to skills directory
    let skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    // Path::join handles the '/' in hierarchical names correctly on all platforms
    let skill_dir = skills_dir.join(name);

    // Check if skill already exists
    if skill_dir.exists() {
        miette::bail!("Skill '{}' already exists at {}", name, skill_dir.display());
    }

    // Conflict detection for hierarchical names
    if name.contains('/') {
        // Hierarchical: check that the category segment isn't already a flat skill
        let category = name.split('/').next().unwrap();
        let category_dir = skills_dir.join(category);
        if category_dir.join("SKILL.md").exists() {
            miette::bail!(
                "Cannot create '{}': '{}' already exists as a flat skill",
                name,
                category
            );
        }
    } else {
        // Flat: check that the name isn't already used as a category directory
        let candidate = skills_dir.join(name);
        if candidate.is_dir() && !candidate.join("SKILL.md").exists() {
            miette::bail!(
                "Cannot create '{}': it is already used as a category. Use a hierarchical name like '{}/...' instead.",
                name,
                name
            );
        }
    }

    // Create skill directory (create_dir_all handles intermediate category dirs)
    fs::create_dir_all(&skill_dir).into_diagnostic()?;

    // Create SKILL.md with frontmatter + body
    let skill_file = skill_dir.join("SKILL.md");
    let title = kebab_to_title_case(name);
    let content = format!(
        "---\nname: {}\ndescription: \"\"\n---\n\n# {}\n\n{}",
        name, title, body
    );

    fs::write(&skill_file, content).into_diagnostic()?;

    log::success(format!("Created skill '{}'", name)).into_diagnostic()?;
    log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;

    Ok(())
}

const VALID_TEMPLATES: &[&str] = &[
    "gather",
    "create",
    "tdd",
    "rule-of-5",
    "ubiquitous-language",
];

fn skill_template_body(name: &str) -> Result<String> {
    match name {
        "gather" => Ok(r#"## Instructions

Use this skill to research $ARGUMENTS in the $PROJECT project.
Repository root: $REPO_ROOT

1. Search existing artifacts:
   ```
   wai search "$ARGUMENTS"
   ```
2. Explore the codebase at $REPO_ROOT relevant to $ARGUMENTS.
3. Gather findings and record them:
   ```
   wai add research "findings about $ARGUMENTS"
   ```
4. Summary: list key facts, open questions, and recommended next steps.
"#
        .to_string()),

        "create" => Ok(r#"## Instructions

Use this skill to create items for $ARGUMENTS in the $PROJECT project.
Repository root: $REPO_ROOT

1. Retrieve the latest plan:
   ```
   wai search "$ARGUMENTS" --type plan --latest
   ```
2. For each item in the plan:
   - Create a tracking issue:
     ```
     bd create --title="..." --type=task
     ```
   - Wire dependencies with `bd dep add <blocked> <blocker>` as needed.
3. Confirm all items are created and visible with `bd ready`.
"#
        .to_string()),

        "tdd" => Ok(r#"## Instructions

Use this skill to implement $ARGUMENTS in the $PROJECT project using TDD.
Repository root: $REPO_ROOT

### RED
1. Write a failing test for the next behaviour.
2. Run tests — confirm the new test fails:
   ```
   cd $REPO_ROOT && cargo test
   ```

### GREEN
3. Write the minimum code to make the test pass.
4. Run tests — confirm all tests pass.

### REFACTOR
5. Clean up without changing behaviour.
6. Run tests — confirm all tests still pass.

Commit after each GREEN/REFACTOR cycle:
```
git add <files>
git commit -m "..."
```
"#
        .to_string()),

        "rule-of-5" => Ok(r#"## Instructions

Use this skill to review $ARGUMENTS in the $PROJECT project using 5 passes.
Repository root: $REPO_ROOT

Run 5 independent review passes. After each pass, record findings.
Check for convergence: if passes 4 and 5 agree, stop early.

### Pass structure
- What is the purpose of this component?
- Does the implementation match the intent?
- Are there edge cases, error paths, or missing tests?
- Is the interface clear and consistent?

### Verdict

After all passes, output one of:
- **APPROVED** — ready to merge/deploy
- **NEEDS_CHANGES** — list specific required changes
- **NEEDS_HUMAN** — ambiguous or high-risk; escalate to a human reviewer
"#
        .to_string()),

        "ubiquitous-language" => Ok(r#"## Instructions

Use this skill to curate ubiquitous language for $ARGUMENTS in the $PROJECT project.
Repository root: $REPO_ROOT

1. Read `.wai/resources/ubiquitous-language/README.md` first to understand the index and file layout.
2. Search existing artifacts and code for the relevant domain terms:
   ```
   wai search "$ARGUMENTS"
   ```
3. Update only the bounded-context files that match the task under
   `.wai/resources/ubiquitous-language/contexts/`.
4. If a term truly spans contexts, update `shared.md` and link back from the relevant context files.
5. Preserve progressive disclosure: do not collapse all terminology into one giant glossary file.
6. Summarize what changed, open questions, and any terms that still need human confirmation.
"#
        .to_string()),

        other => {
            miette::bail!(
                "Unknown template '{}'. Valid templates: {}",
                other,
                VALID_TEMPLATES.join(", ")
            )
        }
    }
}

/// Converts a skill name to Title Case for use in templates.
/// Handles both flat ("my-cool-skill" -> "My Cool Skill") and
/// hierarchical ("issue/gather" -> "Issue / Gather") names.
pub(super) fn kebab_to_title_case(s: &str) -> String {
    let title_segment = |seg: &str| -> String {
        seg.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    };

    s.splitn(2, '/')
        .map(title_segment)
        .collect::<Vec<_>>()
        .join(" / ")
}

pub(super) fn list_skills(json: bool) -> Result<()> {
    // Merge local --json with global --json so both
    // `wai resource list skills --json` and `wai --json resource list skills` work.
    let json = json || current_context().json;
    let project_root = require_project()?;
    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let global_dir = global_skills_dir();

    // Collect local skills (keyed by canonical name for deduplication)
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut entries = Vec::new();

    if local_skills_dir.exists() {
        scan_skills_dir(
            &local_skills_dir,
            &local_skills_dir,
            SkillSource::Local,
            &mut entries,
        );
    }
    // Track names from local skills to suppress global duplicates
    for e in &entries {
        seen_names.insert(e.name.clone());
    }

    // Also collect global skills that aren't shadowed by local ones
    if global_dir.exists() {
        let mut global_entries = Vec::new();
        scan_skills_dir(
            &global_dir,
            &global_dir,
            SkillSource::Global,
            &mut global_entries,
        );
        for e in global_entries {
            if !seen_names.contains(&e.name) {
                entries.push(e);
            }
        }
    }

    // Sort alphabetically by name
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    // Output
    if json {
        let payload = SkillListPayload { skills: entries };
        crate::output::print_json(&payload)?;
    } else if entries.is_empty() {
        println!();
        println!("  {} No skills found", "○".dimmed());
        println!(
            "  {} Run 'wai resource add skill <name>' to create one",
            "→".cyan()
        );
        println!();
    } else {
        println!();
        println!("  {} Skills", "◆".cyan());
        println!();
        for entry in entries {
            let desc = if entry.description.len() > 60 {
                format!("{}...", &entry.description[..57])
            } else {
                entry.description.clone()
            };
            let global_tag = if entry.source == SkillSource::Global {
                " [global]".dimmed().to_string()
            } else {
                String::new()
            };

            if entry.description == "(no metadata)" {
                println!(
                    "    {} {}{}  {}",
                    "•".dimmed(),
                    entry.name,
                    global_tag,
                    desc.dimmed()
                );
            } else {
                println!(
                    "    {} {}{}  {}",
                    "•".dimmed(),
                    entry.name.bold(),
                    global_tag,
                    desc.dimmed()
                );
            }
        }
        println!();
    }

    Ok(())
}

/// Scan a skills directory and collect SkillEntry records.
///
/// Handles both flat skills (skills_dir/<name>/SKILL.md) and hierarchical
/// skills (skills_dir/<category>/<name>/SKILL.md).
pub(super) fn scan_skills_dir(
    skills_dir: &Path,
    display_root: &Path,
    source: SkillSource,
    entries: &mut Vec<SkillEntry>,
) {
    let Ok(read_dir) = fs::read_dir(skills_dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let skill_file = path.join("SKILL.md");

        if skill_file.exists() {
            // Flat skill
            let relative_path = path
                .strip_prefix(display_root)
                .unwrap_or(&path)
                .display()
                .to_string();
            if let Some(metadata) = parse_skill_frontmatter(&skill_file) {
                entries.push(SkillEntry {
                    name: metadata.name,
                    category: None,
                    description: metadata.description,
                    path: relative_path,
                    source: source.clone(),
                });
            } else {
                entries.push(SkillEntry {
                    name: dir_name,
                    category: None,
                    description: "(no metadata)".to_string(),
                    path: relative_path,
                    source: source.clone(),
                });
            }
        } else {
            // Category directory: recurse one level
            let Ok(sub_read) = fs::read_dir(&path) else {
                continue;
            };
            for sub_entry in sub_read.flatten() {
                let sub_path = sub_entry.path();
                if !sub_path.is_dir() {
                    continue;
                }

                let sub_name = sub_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let sub_skill_file = sub_path.join("SKILL.md");
                let hierarchical_name = format!("{}/{}", dir_name, sub_name);
                let relative_path = sub_path
                    .strip_prefix(display_root)
                    .unwrap_or(&sub_path)
                    .display()
                    .to_string();

                if let Some(metadata) = parse_skill_frontmatter(&sub_skill_file) {
                    entries.push(SkillEntry {
                        name: metadata.name,
                        category: Some(dir_name.clone()),
                        description: metadata.description,
                        path: relative_path,
                        source: source.clone(),
                    });
                } else {
                    entries.push(SkillEntry {
                        name: hierarchical_name,
                        category: Some(dir_name.clone()),
                        description: "(no metadata)".to_string(),
                        path: relative_path,
                        source: source.clone(),
                    });
                }
            }
        }
    }
}

pub(super) fn import_skills(from: Option<String>) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("import skills")?;

    // Determine source path
    let source_path = if let Some(path) = from {
        Path::new(&path).to_path_buf()
    } else {
        project_root.join(".agents").join("skills")
    };

    // Check if source exists
    if !source_path.exists() {
        miette::bail!(
            "Source path not found: {}\n  Specify a different path with --from <path>",
            source_path.display()
        );
    }

    if !source_path.is_dir() {
        miette::bail!("Source path is not a directory: {}", source_path.display());
    }

    let target_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    fs::create_dir_all(&target_dir).into_diagnostic()?;

    let mut imported = 0;
    let mut skipped = 0;

    // Scan source directory
    for entry in fs::read_dir(&source_path).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let source_skill_dir = entry.path();

        // Only process directories
        if !source_skill_dir.is_dir() {
            continue;
        }

        // Check if SKILL.md exists
        let skill_md = source_skill_dir.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let skill_name = source_skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| miette::miette!("Invalid skill directory name"))?;

        let target_skill_dir = target_dir.join(skill_name);

        // Skip if target already exists
        if target_skill_dir.exists() {
            log::warning(format!("Skipped '{}' (already exists)", skill_name)).into_diagnostic()?;
            skipped += 1;
            continue;
        }

        // Copy entire directory tree
        copy_dir_all(&source_skill_dir, &target_skill_dir).into_diagnostic()?;
        imported += 1;
    }

    // Report results
    if imported == 0 && skipped == 0 {
        println!();
        println!(
            "  {} No skills found in {}",
            "○".dimmed(),
            source_path.display()
        );
        println!();
    } else {
        println!();
        if imported > 0 {
            log::success(format!(
                "Imported {} skill{}",
                imported,
                if imported == 1 { "" } else { "s" }
            ))
            .into_diagnostic()?;
        }
        if skipped > 0 {
            log::info(format!(
                "Skipped {} skill{} (already exist{})",
                skipped,
                if skipped == 1 { "" } else { "s" },
                if skipped == 1 { "s" } else { "" }
            ))
            .into_diagnostic()?;
        }
        log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;
        println!();
    }

    Ok(())
}

/// Recursively copy a directory and all its contents
pub(super) fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Warn if a skill file contains content that looks like hardcoded project names
/// or absolute paths (should use $PROJECT, $REPO_ROOT, $ARGUMENTS instead).
fn warn_hardcoded_content(skill_path: &Path, project_name: &str, project_root: &Path) {
    let Ok(content) = fs::read_to_string(skill_path) else {
        return;
    };
    let root_str = project_root.to_string_lossy();
    let mut warnings: Vec<String> = Vec::new();

    // Check for the literal project name (if non-empty)
    if !project_name.is_empty() && content.contains(project_name) {
        warnings.push(format!("contains project name '{}'", project_name));
    }
    // Check for absolute paths starting with the repo root
    if root_str.len() > 1 && content.contains(root_str.as_ref()) {
        warnings.push(format!("contains absolute path '{}'", root_str));
    }

    if !warnings.is_empty() {
        let _ = log::warning(format!(
            "Skill may have hardcoded content ({}). Consider using $PROJECT, $REPO_ROOT, $ARGUMENTS instead.",
            warnings.join("; ")
        ));
    }
}

/// Install a skill from the current project into the global skills library
/// at `~/.wai/resources/skills/`.
pub(super) fn install_skill_global(name: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("install skill globally")?;

    validate_skill_name(name)?;

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let src_skill_dir = local_skills_dir.join(name);
    let src_skill_md = src_skill_dir.join("SKILL.md");

    if !src_skill_md.exists() {
        miette::bail!(
            "Skill '{}' not found in current project (looked in {})",
            name,
            src_skill_dir.display()
        );
    }

    let global_dir = global_skills_dir();
    let dst_skill_dir = global_dir.join(name);

    if dst_skill_dir.exists() {
        miette::bail!(
            "Skill '{}' already exists in global library at {}",
            name,
            dst_skill_dir.display()
        );
    }

    // Warn about hardcoded content
    let project_name = crate::config::ProjectConfig::load(&project_root)
        .map(|c| c.project.name)
        .unwrap_or_default();
    warn_hardcoded_content(&src_skill_md, &project_name, &project_root);

    fs::create_dir_all(&dst_skill_dir).into_diagnostic()?;
    copy_dir_all(&src_skill_dir, &dst_skill_dir).into_diagnostic()?;

    log::success(format!(
        "Installed '{}' globally to {}",
        name,
        dst_skill_dir.display()
    ))
    .into_diagnostic()?;

    Ok(())
}

/// Install a skill from another repository's `.wai` directory into the current project.
pub(super) fn install_skill_from_repo(name: &str, repo_path: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("install skill from repo")?;

    validate_skill_name(name)?;

    let repo = PathBuf::from(repo_path);
    let src_skills_dir = crate::config::agent_config_dir(&repo).join(SKILLS_DIR);
    let src_skill_dir = src_skills_dir.join(name);
    let src_skill_md = src_skill_dir.join("SKILL.md");

    if !src_skill_md.exists() {
        miette::bail!(
            "Skill '{}' not found in '{}' (looked in {})",
            name,
            repo_path,
            src_skill_dir.display()
        );
    }

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let dst_skill_dir = local_skills_dir.join(name);

    if dst_skill_dir.exists() {
        miette::bail!("Skill '{}' already exists in current project", name);
    }

    // Warn about hardcoded content (use current project's name as the suspect)
    let project_name = crate::config::ProjectConfig::load(&project_root)
        .map(|c| c.project.name)
        .unwrap_or_default();
    warn_hardcoded_content(&src_skill_md, &project_name, &project_root);

    fs::create_dir_all(&dst_skill_dir).into_diagnostic()?;
    copy_dir_all(&src_skill_dir, &dst_skill_dir).into_diagnostic()?;

    log::success(format!(
        "Installed '{}' from '{}' into current project ({})",
        name,
        repo_path,
        dst_skill_dir.display()
    ))
    .into_diagnostic()?;
    log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill_dir(dir: &Path, name: &str, content: &str) {
        // name may be hierarchical (cat/action)
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    // Title case conversion tests
    #[test]
    fn test_kebab_to_title_case() {
        assert_eq!(kebab_to_title_case("my-skill"), "My Skill");
        assert_eq!(kebab_to_title_case("single"), "Single");
        assert_eq!(kebab_to_title_case("a-b-c"), "A B C");
        assert_eq!(kebab_to_title_case("my-cool-skill-2"), "My Cool Skill 2");
        assert_eq!(kebab_to_title_case(""), "");
        // Hierarchical names
        assert_eq!(kebab_to_title_case("issue/gather"), "Issue / Gather");
        assert_eq!(
            kebab_to_title_case("my-cat/my-action"),
            "My Cat / My Action"
        );
    }

    // ── local overrides global priority ─────────────────────────────────────

    #[test]
    fn test_local_skill_overrides_global_in_list() {
        let local_dir = tempfile::tempdir().unwrap();
        let global_dir = tempfile::tempdir().unwrap();

        // Same skill name in both
        let local_content = "---\nname: my-skill\ndescription: Local version\n---\n\nLocal.\n";
        let global_content = "---\nname: my-skill\ndescription: Global version\n---\n\nGlobal.\n";
        make_skill_dir(local_dir.path(), "my-skill", local_content);
        make_skill_dir(global_dir.path(), "my-skill", global_content);

        // Scan local
        let mut entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            local_dir.path(),
            local_dir.path(),
            SkillSource::Local,
            &mut entries,
        );
        let mut seen: std::collections::HashSet<String> =
            entries.iter().map(|e| e.name.clone()).collect();

        // Scan global, deduplicate
        let mut global_entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            global_dir.path(),
            global_dir.path(),
            SkillSource::Global,
            &mut global_entries,
        );
        for e in global_entries {
            if !seen.contains(&e.name) {
                let name = e.name.clone();
                entries.push(e);
                seen.insert(name);
            }
        }

        // Should only have one entry, and it's the local one
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].source, SkillSource::Local);
        assert_eq!(entries[0].description, "Local version");
    }

    // ── install: copy between directories ───────────────────────────────────

    #[test]
    fn test_install_global_copies_skill() {
        let local_skills = tempfile::tempdir().unwrap();
        let global_skills = tempfile::tempdir().unwrap();

        let content = "---\nname: gather\ndescription: Gather\n---\n\nInstructions.\n";
        make_skill_dir(local_skills.path(), "gather", content);

        // Simulate the copy that install --global does
        let src = local_skills.path().join("gather");
        let dst = global_skills.path().join("gather");
        copy_dir_all(&src, &dst).unwrap();

        let installed = fs::read_to_string(dst.join("SKILL.md")).unwrap();
        assert_eq!(installed, content);
    }

    #[test]
    fn test_install_from_repo_copies_skill() {
        let repo_skills = tempfile::tempdir().unwrap();
        let local_skills = tempfile::tempdir().unwrap();

        let content =
            "---\nname: impl/run\ndescription: Implementation run\n---\n\nInstructions.\n";
        make_skill_dir(repo_skills.path(), "impl/run", content);

        // Simulate the copy that install --from-repo does
        let src = repo_skills.path().join("impl").join("run");
        let dst = local_skills.path().join("impl").join("run");
        copy_dir_all(&src, &dst).unwrap();

        let installed = fs::read_to_string(dst.join("SKILL.md")).unwrap();
        assert_eq!(installed, content);
    }

    // ── global skill visible when no local ──────────────────────────────────

    #[test]
    fn test_global_skill_visible_when_no_local() {
        let local_dir = tempfile::tempdir().unwrap();
        let global_dir = tempfile::tempdir().unwrap();

        // Only in global
        let global_content =
            "---\nname: shared-skill\ndescription: Shared globally\n---\n\nGlobal.\n";
        make_skill_dir(global_dir.path(), "shared-skill", global_content);

        let mut entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            local_dir.path(),
            local_dir.path(),
            SkillSource::Local,
            &mut entries,
        );
        let seen: std::collections::HashSet<String> =
            entries.iter().map(|e| e.name.clone()).collect();

        let mut global_entries: Vec<SkillEntry> = Vec::new();
        scan_skills_dir(
            global_dir.path(),
            global_dir.path(),
            SkillSource::Global,
            &mut global_entries,
        );
        for e in global_entries {
            if !seen.contains(&e.name) {
                entries.push(e);
            }
        }

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].source, SkillSource::Global);
    }
}
