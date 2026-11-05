//! Core sync execution functions for agent config projections.
//!
//! These functions implement the four projection strategies (symlink, inline, reference, copy)
//! without CLI-specific concerns like safe mode checks or user-facing output formatting.
//! They can be used by both the `wai sync` command and `wai doctor --fix`.

use cliclack::log;
use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use std::io::IsTerminal;
use std::path::Path;

use crate::context::current_context;
use crate::error::WaiError;

#[derive(Debug, Deserialize)]
pub(crate) struct Projection {
    pub target: String,
    #[serde(default)]
    pub strategy: String,
    #[serde(default)]
    pub sources: Vec<String>,
}

/// Ensure all parent directories of `path` exist (mkdir -p behaviour).
pub(crate) fn ensure_parent_dirs(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).into_diagnostic()?;
    }
    Ok(())
}

/// Check whether a non-empty directory is safe to remove, prompting if needed.
///
/// Safe to remove silently when:
/// - The directory is empty, or
/// - The directory contains a `.wai/` subdirectory (previously managed by wai).
///
/// Otherwise:
/// - In interactive mode, ask the user for confirmation.
/// - In non-interactive mode (`--no-input` or stdin is not a TTY), return an error.
fn check_unmanaged_dir(target: &Path) -> Result<()> {
    // Collect entries; if we can't read the dir, let the caller handle it.
    let entries: Vec<_> = match std::fs::read_dir(target) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return Ok(()),
    };

    // Empty → always safe.
    if entries.is_empty() {
        return Ok(());
    }

    // Contains a .wai/ marker → managed by wai, safe.
    if target.join(".wai").is_dir() {
        return Ok(());
    }

    // Non-empty, unmanaged directory — need confirmation.
    let ctx = current_context();

    // --yes flag bypasses the prompt.
    if ctx.yes {
        return Ok(());
    }

    if ctx.no_input || !std::io::stdin().is_terminal() {
        return Err(WaiError::NonInteractive {
            message: format!(
                "Directory '{}' contains unmanaged data. \
                 Re-run interactively or pass --yes to confirm removal.",
                target.display()
            ),
        }
        .into());
    }

    // Interactive prompt.
    let confirmed = cliclack::confirm(format!(
        "Directory '{}' contains unmanaged data. Remove it?",
        target.display()
    ))
    .interact()
    .into_diagnostic()?;

    if confirmed {
        Ok(())
    } else {
        Err(WaiError::NonInteractive {
            message: format!("Aborted: directory '{}' was not removed.", target.display()),
        }
        .into())
    }
}

/// Check whether a file→file symlink projection is already up-to-date.
#[cfg(unix)]
fn symlink_file_up_to_date(target: &Path, source: &Path) -> bool {
    if !target.is_symlink() {
        return false;
    }
    let rel_source = make_relative(target, source);
    std::fs::read_link(target)
        .map(|dest| dest == rel_source)
        .unwrap_or(false)
}

/// Check whether a dir→dir symlink projection is already up-to-date.
///
/// Returns `true` when `target` is a real directory whose entries are exactly the
/// symlinks that would be created from `source`, each pointing at the right path.
#[cfg(unix)]
fn symlink_dir_up_to_date(target: &Path, source: &Path) -> bool {
    use std::collections::HashSet;

    if !target.is_dir() || target.is_symlink() {
        return false;
    }

    // Collect expected entry names from source.
    let expected: HashSet<std::ffi::OsString> = match std::fs::read_dir(source) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.file_name()).collect(),
        Err(_) => return false,
    };

    // Collect actual entry names in target.
    let actual: HashSet<std::ffi::OsString> = match std::fs::read_dir(target) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.file_name()).collect(),
        Err(_) => return false,
    };

    if expected != actual {
        return false;
    }

    // Each entry must be a symlink pointing to the correct source path.
    expected.iter().all(|name| {
        let link = target.join(name);
        let expected_dest = source.join(name);
        let rel_dest = make_relative(&link, &expected_dest);

        link.is_symlink()
            && std::fs::read_link(&link)
                .map(|dest| dest == rel_dest)
                .unwrap_or(false)
    })
}

/// Execute a symlink projection.
///
/// - File source → file symlink at target pointing to source.
/// - Directory source → create target directory and symlink each entry inside it.
///
/// Idempotent: skips work when the target already matches the desired state.
/// Uses relative symlinks for better portability.
pub(crate) fn execute_symlink(
    project_root: &Path,
    config_dir: &Path,
    proj: &Projection,
) -> Result<()> {
    let target = project_root.join(&proj.target);

    ensure_parent_dirs(&target)?;

    // Check if the projection is already up-to-date.
    #[cfg(unix)]
    if symlink_up_to_date(&target, config_dir, proj) {
        return Ok(());
    }

    remove_existing_target(&target)?;

    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() {
            continue;
        }

        if source_path.is_file() {
            // file→file: create a single symlink at target pointing to source.
            #[cfg(unix)]
            {
                let rel_source = make_relative(&target, &source_path);
                std::os::unix::fs::symlink(rel_source, &target).into_diagnostic()?;
            }
            #[cfg(not(unix))]
            std::fs::copy(&source_path, &target).into_diagnostic()?;
        } else if source_path.is_dir() {
            // dir→dir: create target directory, symlink each entry inside.
            symlink_dir_entries(&target, &source_path)?;
        }
    }

    log::info(format!("Symlinked → {}", proj.target)).into_diagnostic()?;
    Ok(())
}

/// True when every projected source is already correctly linked at the target.
#[cfg(unix)]
fn symlink_up_to_date(target: &Path, config_dir: &Path, proj: &Projection) -> bool {
    proj.sources.iter().all(|source| {
        let source_path = config_dir.join(source);
        if !source_path.exists() {
            return !target.exists(); // stale target → not up-to-date
        }
        if source_path.is_file() {
            symlink_file_up_to_date(target, &source_path)
        } else if source_path.is_dir() {
            symlink_dir_up_to_date(target, &source_path)
        } else {
            true
        }
    })
}

/// Remove an existing target before recreating; refuses unmanaged dirs.
fn remove_existing_target(target: &Path) -> Result<()> {
    if let Ok(meta) = target.symlink_metadata() {
        if meta.is_dir() && !meta.file_type().is_symlink() {
            check_unmanaged_dir(target)?;
            std::fs::remove_dir_all(target).into_diagnostic()?;
        } else {
            std::fs::remove_file(target).into_diagnostic()?;
        }
    }
    Ok(())
}

/// dir→dir: create target directory, replace-and-link each entry inside.
fn symlink_dir_entries(target: &Path, source_path: &Path) -> Result<()> {
    std::fs::create_dir_all(target).into_diagnostic()?;
    for entry in std::fs::read_dir(source_path).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let entry_path = entry.path();
        let link_path = target.join(entry.file_name());

        remove_existing_target(&link_path)?;

        #[cfg(unix)]
        {
            let rel_source = make_relative(&link_path, &entry_path);
            std::os::unix::fs::symlink(rel_source, &link_path).into_diagnostic()?;
        }
        #[cfg(not(unix))]
        std::fs::copy(entry_path, &link_path).into_diagnostic()?;
    }
    Ok(())
}

/// Calculate a relative path from `from` (the symlink location) to `to` (the target file).
fn make_relative(from: &Path, to: &Path) -> std::path::PathBuf {
    if let Some(parent) = from.parent() {
        pathdiff::diff_paths(to, parent).unwrap_or_else(|| to.to_path_buf())
    } else {
        to.to_path_buf()
    }
}

/// Execute an inline projection: concatenate source files into a single target file.
pub(crate) fn execute_inline(
    project_root: &Path,
    config_dir: &Path,
    proj: &Projection,
) -> Result<()> {
    let target = project_root.join(&proj.target);
    let mut content = String::from("# Auto-generated by wai — do not edit directly\n\n");

    for source in &proj.sources {
        inline_source(&mut content, config_dir, source);
    }

    ensure_parent_dirs(&target)?;
    std::fs::write(&target, content).into_diagnostic()?;

    log::info(format!("Inlined → {}", proj.target)).into_diagnostic()?;
    Ok(())
}

/// Execute a reference projection: create a file listing references to source files.
///
/// Preserves user-authored files: if the target already exists and its content is not
/// a wai-generated reference stub, the file is left untouched. Only wai-managed files
/// (those starting with the "Auto-generated by wai" header) are overwritten.
pub(crate) fn execute_reference(
    project_root: &Path,
    config_dir: &Path,
    proj: &Projection,
) -> Result<()> {
    let target = project_root.join(&proj.target);

    // If the target exists and has user-authored content, preserve it.
    if target.exists()
        && let Ok(existing) = std::fs::read_to_string(&target)
        && !existing.starts_with("# Auto-generated by wai")
    {
        log::info(format!(
            "Skipping reference → {} (preserving existing content)",
            proj.target
        ))
        .into_diagnostic()?;
        return Ok(());
    }

    let mut content = String::from("# Auto-generated by wai — do not edit directly\n");
    content.push_str("# References to agent config sources:\n\n");

    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if source_path.exists() && source_path.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&source_path)
                .into_diagnostic()?
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by_key(|e| e.file_name());

            for entry in entries {
                if let Some(name) = entry.file_name().to_str() {
                    content.push_str(&format!(
                        "- .wai/resources/agent-config/{}/{}\n",
                        source, name
                    ));
                }
            }
        }
    }

    ensure_parent_dirs(&target)?;
    std::fs::write(&target, content).into_diagnostic()?;

    log::info(format!("Referenced → {}", proj.target)).into_diagnostic()?;
    Ok(())
}

/// Execute a copy projection: copy source files to the target location.
///
/// - File source → copy source file to target path (overwrites if present).
/// - Directory source → create target directory and copy each entry inside.
pub(crate) fn execute_copy(
    project_root: &Path,
    config_dir: &Path,
    proj: &Projection,
) -> Result<()> {
    let target = project_root.join(&proj.target);

    ensure_parent_dirs(&target)?;

    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() {
            continue;
        }

        if source_path.is_file() {
            std::fs::copy(&source_path, &target).into_diagnostic()?;
        } else if source_path.is_dir() {
            std::fs::create_dir_all(&target).into_diagnostic()?;
            for entry in std::fs::read_dir(&source_path).into_diagnostic()? {
                let entry = entry.into_diagnostic()?;
                std::fs::copy(entry.path(), target.join(entry.file_name())).into_diagnostic()?;
            }
        }
    }

    log::info(format!("Copied → {}", proj.target)).into_diagnostic()?;
    Ok(())
}

/// Translate a category and action segment (kebab-case) to a Claude Code skill name.
///
/// Capitalises the first letter of each hyphen-separated word and joins the two
/// segments with `": "`.  For example:
/// - `"issue"`, `"gather"` → `"Issue: Gather"`
/// - `"my-cat"`, `"my-action"` → `"My Cat: My Action"`
fn translate_skill_name(category: &str, action: &str) -> String {
    let capitalise = |seg: &str| -> String {
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
    format!("{}: {}", capitalise(category), capitalise(action))
}

/// Parse a SKILL.md and return `(name, description, body)`.
///
/// `body` is everything after the closing `---` frontmatter delimiter (may be empty).
/// Returns `None` if the file cannot be read or the frontmatter is missing/invalid.
fn parse_skill_for_projection(path: &Path) -> Option<(String, String, String)> {
    let contents = std::fs::read_to_string(path).ok()?;
    let mut lines = contents.lines();

    // Expect opening ---
    let first = lines.next()?.trim();
    if first != "---" {
        return None;
    }

    // Collect frontmatter lines until closing ---
    let mut fm_lines = Vec::new();
    for line in lines.by_ref() {
        if line.trim() == "---" {
            break;
        }
        fm_lines.push(line);
    }
    if fm_lines.is_empty() {
        return None;
    }

    // Parse only the fields we need
    #[derive(Deserialize)]
    struct Fm {
        name: String,
        description: String,
    }
    let fm: Fm = serde_yml::from_str(&fm_lines.join("\n")).ok()?;
    if fm.name.trim().is_empty() || fm.description.trim().is_empty() {
        return None;
    }

    // Body is whatever remains after the frontmatter
    let body: Vec<&str> = lines.collect();
    // Trim a single leading newline that separates frontmatter from body
    let body_str = body.join("\n");
    let body_str = body_str.trim_start_matches('\n').to_string();

    Some((fm.name, fm.description, body_str))
}

/// Execute a claude-code projection: translate wai hierarchical skills to Claude Code commands.
///
/// Scans `config_dir/skills/<category>/<action>/SKILL.md` for all hierarchical skills
/// and writes `.claude/commands/<category>/<action>.md` with translated frontmatter.
/// Flat skills (where `SKILL.md` lives directly under `skills/<name>/`) are skipped.
pub(crate) fn execute_claude_code(project_root: &Path, config_dir: &Path) -> Result<()> {
    let skills_dir = config_dir.join("skills");
    if !skills_dir.exists() {
        log::info("No skills directory found; nothing to sync").into_diagnostic()?;
        return Ok(());
    }

    let mut count = 0;
    for (category, action) in hierarchical_skills(&skills_dir) {
        if sync_skill_to_claude_code(project_root, &category, &action)? {
            count += 1;
        }
    }

    if count == 0 {
        log::info("No hierarchical skills found to sync").into_diagnostic()?;
    } else {
        log::success(format!(
            "Synced {} skill{} to .claude/commands/",
            count,
            if count == 1 { "" } else { "s" }
        ))
        .into_diagnostic()?;
    }

    Ok(())
}

/// All hierarchical (category/action/SKILL.md) skills, sorted.
fn hierarchical_skills(skills_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let mut cat_entries: Vec<_> = std::fs::read_dir(skills_dir)
        .into_diagnostic()
        .ok()
        .map(|rd| rd.filter_map(|e| e.ok()).collect::<Vec<_>>())
        .unwrap_or_default();
    cat_entries.sort_by_key(|e| e.file_name());

    let mut skills = Vec::new();
    for cat_entry in cat_entries {
        let cat_path = cat_entry.path();
        if !cat_path.is_dir() {
            continue;
        }
        // Skip flat skills (SKILL.md directly in category dir)
        if cat_path.join("SKILL.md").exists() {
            continue;
        }
        let category = cat_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let mut action_entries: Vec<_> = std::fs::read_dir(&cat_path)
            .into_diagnostic()
            .ok()
            .map(|rd| rd.filter_map(|e| e.ok()).collect::<Vec<_>>())
            .unwrap_or_default();
        action_entries.sort_by_key(|e| e.file_name());

        for action_entry in action_entries {
            let action_path = action_entry.path();
            if !action_path.is_dir() {
                continue;
            }
            if !action_path.join("SKILL.md").exists() {
                continue;
            }
            skills.push((category.clone(), action_path));
        }
    }
    skills
}

/// Project one skill into `.claude/commands/<category>/<action>.md`.
/// Returns true when a skill was written.
fn sync_skill_to_claude_code(
    project_root: &Path,
    category: &str,
    action_path: &Path,
) -> Result<bool> {
    let action = action_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let skill_file = action_path.join("SKILL.md");
    let Some((_name, description, body)) = parse_skill_for_projection(&skill_file) else {
        log::warning(format!(
            "Skipping {category}/{action}: missing or invalid frontmatter"
        ))
        .into_diagnostic()?;
        return Ok(false);
    };

    let cc_name = translate_skill_name(category, &action);
    let output = format!(
        "---\n\
         # Auto-generated by wai — do not edit directly\n\
         name: \"{}\"\n\
         description: \"{}\"\n\
         category: \"{}\"\n\
         ---\n\
         \n\
         {}",
        cc_name, description, category, body
    );

    let target = project_root
        .join(".claude")
        .join("commands")
        .join(category)
        .join(format!("{}.md", action));
    ensure_parent_dirs(&target)?;
    std::fs::write(&target, output).into_diagnostic()?;

    log::info(format!(
        "claude-code → .claude/commands/{category}/{action}.md"
    ))
    .into_diagnostic()?;
    Ok(true)
}

/// Execute an .agents projection: symlink all skills to the .agents/ directory.
///
/// This is a built-in projection for general agents (like Pi) that look for
/// instructions in the `.agents/` directory. It symlinks all entries from
/// `config_dir/skills/` into `project_root/.agents/`.
pub(crate) fn execute_agents_projection(project_root: &Path, config_dir: &Path) -> Result<()> {
    let skills_dir = config_dir.join("skills");
    if !skills_dir.exists() {
        log::info("No skills directory found; nothing to sync").into_diagnostic()?;
        return Ok(());
    }

    let target = project_root.join(".agents");
    ensure_parent_dirs(&target)?;

    // Use execute_symlink logic but hardcoded for skills -> .agents
    let proj = Projection {
        target: ".agents".to_string(),
        strategy: "symlink".to_string(),
        sources: vec!["skills".to_string()],
    };

    execute_symlink(project_root, config_dir, &proj)
}

#[cfg(test)]
mod tests;

/// Append one inline source (dir entries sorted, or single file) to `content`.
fn inline_source(content: &mut String, config_dir: &Path, source: &str) {
    let source_path = config_dir.join(source);
    if !source_path.exists() {
        return;
    }
    if source_path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(&source_path)
            .into_diagnostic()
            .ok()
            .map(|rd| rd.filter_map(|e| e.ok()).collect::<Vec<_>>())
            .unwrap_or_default();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            if let Ok(file_content) = std::fs::read_to_string(entry.path()) {
                content.push_str(&format!(
                    "# Source: {}/{}\n",
                    source,
                    entry.file_name().to_str().unwrap_or("?")
                ));
                content.push_str(&file_content);
                content.push_str("\n\n");
            }
        }
    } else if let Ok(file_content) = std::fs::read_to_string(&source_path) {
        content.push_str(&format!("# Source: {}\n", source));
        content.push_str(&file_content);
        content.push_str("\n\n");
    }
}
