use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::Deserialize;

use crate::config::agent_config_dir;
use crate::sync_core::Projection;

use super::WaiCheckEntry;
use genesis::doctor::CheckStatus;

type ExecuteFn = fn(&Path, &Path, &Projection) -> miette::Result<()>;

#[derive(Deserialize)]
pub(super) struct ProjectionsConfig {
    #[serde(default)]
    pub(super) projections: Vec<ProjectionEntry>,
}

#[derive(Deserialize)]
pub(super) struct ProjectionEntry {
    pub(super) target: String,
    pub(super) strategy: String,
    #[serde(default)]
    pub(super) sources: Vec<String>,
}

pub(super) fn check_agent_config_sync(project_root: &Path) -> Vec<WaiCheckEntry> {
    let config_dir = agent_config_dir(project_root);
    let projections_path = config_dir.join(".projections.yml");
    let mut results = Vec::new();

    if !projections_path.exists() {
        results.push(warn_sync_entry(
            "Agent config sync",
            ".projections.yml not found",
            "Run: wai init (or create .wai/resources/agent-config/.projections.yml)",
        ));
        return results;
    }

    let content = match std::fs::read_to_string(&projections_path) {
        Ok(c) => c,
        Err(e) => {
            results.push(fail_sync_entry(
                "Agent config sync",
                format!("Cannot read .projections.yml: {}", e),
            ));
            return results;
        }
    };

    match serde_yml::from_str::<ProjectionsConfig>(&content) {
        Ok(config) => {
            if config.projections.is_empty() {
                results.push(pass_sync_entry(
                    "Agent config sync",
                    "No projections configured",
                ));
            } else {
                for proj in &config.projections {
                    results.extend(check_projection(project_root, &config_dir, proj));
                }
            }
        }
        Err(e) => {
            results.push(warn_sync_entry(
                "Agent config sync",
                format!("Invalid .projections.yml: {}", e),
                "Fix the YAML syntax in .wai/resources/agent-config/.projections.yml",
            ));
        }
    }

    results
}

pub(super) fn check_projection(
    project_root: &Path,
    config_dir: &Path,
    proj: &ProjectionEntry,
) -> Vec<WaiCheckEntry> {
    let mut results = Vec::new();

    // Check if source directories exist
    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() {
            results.push(WaiCheckEntry {
                name: format!("Projection source: {}", source),
                status: CheckStatus::Warn,
                message: format!("Source directory '{}' not found", source),
                fix: Some("Check .projections.yml sources".to_string()),
                fix_fn: None,
            });
        }
    }

    let target_path = project_root.join(&proj.target);

    // Check target exists
    if !target_path.exists() {
        results.push(unsynced_entry(config_dir, proj));
        return results;
    }

    // Strategy-specific checks
    match proj.strategy.as_str() {
        "symlink" => results.extend(check_symlink_strategy(config_dir, proj, &target_path)),
        "inline" => results.extend(check_inline_strategy(config_dir, proj, &target_path)),
        "reference" => results.extend(check_reference_strategy(config_dir, proj, &target_path)),
        _ => {
            // Unknown strategy - just check target exists
            results.push(pass_sync_entry(
                &format!("Projection → {}", proj.target),
                &format!("Target exists (unknown strategy: {})", proj.strategy),
            ));
        }
    }

    results
}

fn to_sync_projection(proj: &ProjectionEntry) -> Projection {
    Projection {
        target: proj.target.clone(),
        strategy: proj.strategy.clone(),
        sources: proj.sources.clone(),
    }
}

fn pass_sync_entry(name: &str, message: &str) -> WaiCheckEntry {
    WaiCheckEntry {
        name: name.to_string(),
        status: CheckStatus::Pass,
        message: message.to_string(),
        fix: None,
        fix_fn: None,
    }
}

fn warn_sync_entry(name: &str, message: impl Into<String>, fix: &str) -> WaiCheckEntry {
    WaiCheckEntry {
        name: name.to_string(),
        status: CheckStatus::Warn,
        message: message.into(),
        fix: Some(fix.to_string()),
        fix_fn: None,
    }
}

fn fail_sync_entry(name: &str, message: String) -> WaiCheckEntry {
    WaiCheckEntry {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message,
        fix: None,
        fix_fn: None,
    }
}

/// Warning entry with a "Run: wai sync" fix that re-executes `exec` for this projection.
fn unsynced_with_exec(config_dir: &Path, proj: &ProjectionEntry, exec: ExecuteFn) -> WaiCheckEntry {
    let config_dir = config_dir.to_path_buf();
    let sync_proj = to_sync_projection(proj);
    let name = format!("Projection → {}", proj.target);
    WaiCheckEntry {
        name,
        status: CheckStatus::Warn,
        message: "Target not synced".to_string(),
        fix: Some("Run: wai sync".to_string()),
        fix_fn: Some(Box::new(move |project_root| {
            exec(project_root, &config_dir, &sync_proj)
        })),
    }
}

fn unsynced_entry(config_dir: &Path, proj: &ProjectionEntry) -> WaiCheckEntry {
    let exec: ExecuteFn = match proj.strategy.as_str() {
        "symlink" => crate::sync_core::execute_symlink,
        "inline" => crate::sync_core::execute_inline,
        "reference" => crate::sync_core::execute_reference,
        _ => |_project_root, _config_dir, _proj| Ok(()),
    };
    unsynced_with_exec(config_dir, proj, exec)
}

fn check_symlink_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<WaiCheckEntry> {
    let mut results = Vec::new();

    // For symlink strategy, verify each entry is a symlink pointing to correct source
    let links = collect_links(config_dir, &proj.sources, target_path);
    let broken = links.iter().filter(|(l, _)| !l.exists()).count();
    let mismatched = links
        .iter()
        .filter(|(l, e)| l.exists() && link_missing_or_mismatched(l, e))
        .count();

    // Also scan the target directory for any broken symlinks (e.g. source file deleted
    // after sync, leaving a dangling symlink with no corresponding source entry).
    let dangling = count_dangling_links(target_path);

    if broken + mismatched + dangling > 0 {
        let message = if broken + dangling > 0 {
            format!("Has {} broken symlinks", broken + dangling)
        } else {
            "Symlink issues detected".to_string()
        };
        let config_dir = config_dir.to_path_buf();
        let sync_proj = to_sync_projection(proj);
        results.push(WaiCheckEntry {
            name: format!("Projection → {}", proj.target),
            status: CheckStatus::Warn,
            message,
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                crate::sync_core::execute_symlink(project_root, &config_dir, &sync_proj)
            })),
        });
    } else {
        results.push(pass_sync_entry(
            &format!("Projection → {}", proj.target),
            "In sync",
        ));
    }

    results
}

/// (link_path, expected_source_path) pairs for every synced entry of `sources`.
fn collect_links(
    config_dir: &Path,
    sources: &[String],
    target_path: &Path,
) -> Vec<(std::path::PathBuf, std::path::PathBuf)> {
    sources
        .iter()
        .flat_map(|source| {
            let source_path = config_dir.join(source);
            if !source_path.exists() || !source_path.is_dir() {
                return Vec::new();
            }
            std::fs::read_dir(&source_path)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .map(|entry| (target_path.join(entry.file_name()), entry.path()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Dangling symlinks in `target_path` (e.g. source file deleted after sync).
#[cfg(unix)]
fn count_dangling_links(target_path: &Path) -> usize {
    if !target_path.exists() {
        return 0;
    }
    std::fs::read_dir(target_path)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            std::fs::symlink_metadata(p)
                .map(|meta| meta.file_type().is_symlink() && !p.exists())
                .unwrap_or(false)
        })
        .count()
}

#[cfg(not(unix))]
fn count_dangling_links(_target_path: &Path) -> usize {
    0
}

/// True when the link is missing, not a symlink, or points somewhere unexpected.
#[cfg(unix)]
fn link_missing_or_mismatched(link_path: &Path, expected: &Path) -> bool {
    if !link_path.exists() {
        return true;
    }
    let Ok(metadata) = std::fs::symlink_metadata(link_path) else {
        return true;
    };
    if !metadata.file_type().is_symlink() {
        return true;
    }
    std::fs::read_link(link_path)
        .map(|target| target != expected)
        .unwrap_or(true)
}

/// On non-Unix, just check the link target exists (copy strategy).
#[cfg(not(unix))]
fn link_missing_or_mismatched(link_path: &Path, _expected: &Path) -> bool {
    !link_path.exists()
}

/// Shared comparison flow for content-hash strategies (inline / reference).
fn check_hashed_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
    expected_content: String,
    exec: ExecuteFn,
) -> Vec<WaiCheckEntry> {
    let mut results = Vec::new();
    let expected_hash = hash_string(&expected_content);

    let actual_hash = match std::fs::read_to_string(target_path) {
        Ok(c) => hash_string(&c),
        Err(_) => {
            results.push(unsynced_with_exec(config_dir, proj, exec));
            return results;
        }
    };

    if expected_hash != actual_hash {
        let config_dir = config_dir.to_path_buf();
        let sync_proj = to_sync_projection(proj);
        results.push(WaiCheckEntry {
            name: format!("Projection → {}", proj.target),
            status: CheckStatus::Warn,
            message: "Stale (content changed)".to_string(),
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                exec(project_root, &config_dir, &sync_proj)
            })),
        });
    } else {
        results.push(pass_sync_entry(
            &format!("Projection → {}", proj.target),
            "In sync",
        ));
    }

    results
}

fn check_inline_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<WaiCheckEntry> {
    let expected_content = build_inline_content(config_dir, &proj.sources);
    check_hashed_strategy(
        config_dir,
        proj,
        target_path,
        expected_content,
        crate::sync_core::execute_inline,
    )
}

fn check_reference_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<WaiCheckEntry> {
    let expected_content = build_reference_content(config_dir, &proj.sources);
    check_hashed_strategy(
        config_dir,
        proj,
        target_path,
        expected_content,
        crate::sync_core::execute_reference,
    )
}

fn build_inline_content(config_dir: &Path, sources: &[String]) -> String {
    let mut content = String::from("# Auto-generated by wai — do not edit directly\n\n");

    for source in sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() {
            continue;
        }
        if source_path.is_dir() {
            append_inline_dir(&source_path, source, &mut content);
        } else if let Ok(file_content) = std::fs::read_to_string(&source_path) {
            content.push_str(&format!("# Source: {}\n", source));
            content.push_str(&file_content);
            content.push_str("\n\n");
        }
    }

    content
}

fn append_inline_dir(source_path: &Path, source: &str, content: &mut String) {
    let mut entries: Vec<_> = std::fs::read_dir(source_path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .collect();
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
}

fn build_reference_content(config_dir: &Path, sources: &[String]) -> String {
    let mut content = String::from("# Auto-generated by wai — do not edit directly\n");
    content.push_str("# References to agent config sources:\n\n");

    for source in sources {
        // The config_dir is .wai/resources/agent-config, so we need to construct
        // paths relative to that
        let source_path = config_dir.join(source);
        if !source_path.exists() || !source_path.is_dir() {
            continue;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&source_path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            if let Some(name) = entry.file_name().to_str() {
                // Format: .wai/resources/agent-config/{source}/{name}
                content.push_str(&format!(
                    "- .wai/resources/agent-config/{}/{}\n",
                    source, name
                ));
            }
        }
    }

    content
}

fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
