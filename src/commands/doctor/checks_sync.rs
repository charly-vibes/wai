use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::Deserialize;

use crate::config::agent_config_dir;

use super::{CheckResult, Status};

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

pub(super) fn check_agent_config_sync(project_root: &Path) -> Vec<CheckResult> {
    let config_dir = agent_config_dir(project_root);
    let projections_path = config_dir.join(".projections.yml");
    let mut results = Vec::new();

    if !projections_path.exists() {
        results.push(CheckResult {
            name: "Agent config sync".to_string(),
            status: Status::Warn,
            message: ".projections.yml not found".to_string(),
            fix: Some(
                "Run: wai init (or create .wai/resources/agent-config/.projections.yml)"
                    .to_string(),
            ),
            fix_fn: None,
        });
        return results;
    }

    let content = match std::fs::read_to_string(&projections_path) {
        Ok(c) => c,
        Err(e) => {
            results.push(CheckResult {
                name: "Agent config sync".to_string(),
                status: Status::Fail,
                message: format!("Cannot read .projections.yml: {}", e),
                fix: None,
                fix_fn: None,
            });
            return results;
        }
    };

    match serde_yml::from_str::<ProjectionsConfig>(&content) {
        Ok(config) => {
            if config.projections.is_empty() {
                results.push(CheckResult {
                    name: "Agent config sync".to_string(),
                    status: Status::Pass,
                    message: "No projections configured".to_string(),
                    fix: None,
                    fix_fn: None,
                });
            } else {
                for proj in &config.projections {
                    results.extend(check_projection(project_root, &config_dir, proj));
                }
            }
        }
        Err(e) => {
            results.push(CheckResult {
                name: "Agent config sync".to_string(),
                status: Status::Fail,
                message: format!("Invalid .projections.yml: {}", e),
                fix: Some(
                    "Fix the YAML syntax in .wai/resources/agent-config/.projections.yml"
                        .to_string(),
                ),
                fix_fn: None,
            });
        }
    }

    results
}

pub(super) fn check_projection(
    project_root: &Path,
    config_dir: &Path,
    proj: &ProjectionEntry,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // Check if source directories exist
    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() {
            results.push(CheckResult {
                name: format!("Projection source: {}", source),
                status: Status::Warn,
                message: format!("Source directory '{}' not found", source),
                fix: Some("Check .projections.yml sources".to_string()),
                fix_fn: None,
            });
        }
    }

    let target_path = project_root.join(&proj.target);

    // Check target exists
    if !target_path.exists() {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Target not synced".to_string(),
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                match sync_proj.strategy.as_str() {
                    "symlink" => crate::sync_core::execute_symlink(
                        project_root,
                        &config_dir_clone,
                        &sync_proj,
                    ),
                    "inline" => crate::sync_core::execute_inline(
                        project_root,
                        &config_dir_clone,
                        &sync_proj,
                    ),
                    "reference" => crate::sync_core::execute_reference(
                        project_root,
                        &config_dir_clone,
                        &sync_proj,
                    ),
                    _ => Ok(()),
                }
            })),
        });
        return results;
    }

    // Strategy-specific checks
    match proj.strategy.as_str() {
        "symlink" => {
            results.extend(check_symlink_strategy(
                project_root,
                config_dir,
                proj,
                &target_path,
            ));
        }
        "inline" => {
            results.extend(check_inline_strategy(config_dir, proj, &target_path));
        }
        "reference" => {
            results.extend(check_reference_strategy(config_dir, proj, &target_path));
        }
        _ => {
            // Unknown strategy - just check target exists
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Pass,
                message: format!("Target exists (unknown strategy: {})", proj.strategy),
                fix: None,
                fix_fn: None,
            });
        }
    }

    results
}

fn check_symlink_strategy(
    _project_root: &Path,
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let mut has_issues = false;
    let mut broken_count = 0;

    // For symlink strategy, verify each entry is a symlink pointing to correct source
    for source in &proj.sources {
        let source_path = config_dir.join(source);
        if !source_path.exists() || !source_path.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(&source_path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let entry_name = entry.file_name();
            let link_path = target_path.join(&entry_name);

            if !link_path.exists() {
                // Broken or missing symlink
                broken_count += 1;
                has_issues = true;
            } else {
                #[cfg(unix)]
                {
                    if let Ok(metadata) = std::fs::symlink_metadata(&link_path) {
                        if !metadata.file_type().is_symlink() {
                            has_issues = true;
                        } else if let Ok(target) = std::fs::read_link(&link_path) {
                            let expected = entry.path();
                            if target != expected {
                                has_issues = true;
                            }
                        } else {
                            has_issues = true;
                        }
                    } else {
                        has_issues = true;
                    }
                }
                #[cfg(not(unix))]
                {
                    // On non-Unix, just check file exists (copy strategy)
                    let _ = entry; // Silence unused variable warning
                }
            }
        }
    }

    // Also scan the target directory for any broken symlinks (e.g. source file deleted
    // after sync, leaving a dangling symlink with no corresponding source entry).
    #[cfg(unix)]
    if target_path.exists()
        && let Ok(entries) = std::fs::read_dir(target_path)
    {
        for entry in entries.filter_map(|e| e.ok()) {
            let link_path = entry.path();
            if let Ok(meta) = std::fs::symlink_metadata(&link_path)
                && meta.file_type().is_symlink()
                && !link_path.exists()
            {
                broken_count += 1;
                has_issues = true;
            }
        }
    }

    if broken_count > 0 || has_issues {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        let message = if broken_count > 0 {
            format!("Has {} broken symlinks", broken_count)
        } else {
            "Symlink issues detected".to_string()
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message,
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                crate::sync_core::execute_symlink(project_root, &config_dir_clone, &sync_proj)
            })),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
            fix_fn: None,
        });
    }

    results
}

fn check_inline_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let expected_content = build_inline_content(config_dir, &proj.sources);
    let expected_hash = hash_string(&expected_content);

    let actual_content = match std::fs::read_to_string(target_path) {
        Ok(c) => c,
        Err(_) => {
            let config_dir_clone = config_dir.to_path_buf();
            let sync_proj = crate::sync_core::Projection {
                target: proj.target.clone(),
                strategy: proj.strategy.clone(),
                sources: proj.sources.clone(),
            };
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Warn,
                message: "Cannot read target file".to_string(),
                fix: Some("Run: wai sync".to_string()),
                fix_fn: Some(Box::new(move |project_root| {
                    crate::sync_core::execute_inline(project_root, &config_dir_clone, &sync_proj)
                })),
            });
            return results;
        }
    };
    let actual_hash = hash_string(&actual_content);

    if expected_hash != actual_hash {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Stale (content changed)".to_string(),
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                crate::sync_core::execute_inline(project_root, &config_dir_clone, &sync_proj)
            })),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
            fix_fn: None,
        });
    }

    results
}

fn check_reference_strategy(
    config_dir: &Path,
    proj: &ProjectionEntry,
    target_path: &Path,
) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let expected_content = build_reference_content(config_dir, &proj.sources);
    let expected_hash = hash_string(&expected_content);

    let actual_content = match std::fs::read_to_string(target_path) {
        Ok(c) => c,
        Err(_) => {
            let config_dir_clone = config_dir.to_path_buf();
            let sync_proj = crate::sync_core::Projection {
                target: proj.target.clone(),
                strategy: proj.strategy.clone(),
                sources: proj.sources.clone(),
            };
            results.push(CheckResult {
                name: format!("Projection → {}", proj.target),
                status: Status::Warn,
                message: "Cannot read target file".to_string(),
                fix: Some("Run: wai sync".to_string()),
                fix_fn: Some(Box::new(move |project_root| {
                    crate::sync_core::execute_reference(project_root, &config_dir_clone, &sync_proj)
                })),
            });
            return results;
        }
    };
    let actual_hash = hash_string(&actual_content);

    if expected_hash != actual_hash {
        let config_dir_clone = config_dir.to_path_buf();
        let sync_proj = crate::sync_core::Projection {
            target: proj.target.clone(),
            strategy: proj.strategy.clone(),
            sources: proj.sources.clone(),
        };
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Warn,
            message: "Stale (content changed)".to_string(),
            fix: Some("Run: wai sync".to_string()),
            fix_fn: Some(Box::new(move |project_root| {
                crate::sync_core::execute_reference(project_root, &config_dir_clone, &sync_proj)
            })),
        });
    } else {
        results.push(CheckResult {
            name: format!("Projection → {}", proj.target),
            status: Status::Pass,
            message: "In sync".to_string(),
            fix: None,
            fix_fn: None,
        });
    }

    results
}

fn build_inline_content(config_dir: &Path, sources: &[String]) -> String {
    let mut content = String::from("# Auto-generated by wai — do not edit directly\n\n");

    for source in sources {
        let source_path = config_dir.join(source);
        if source_path.exists() {
            if source_path.is_dir() {
                let mut entries: Vec<_> = std::fs::read_dir(&source_path)
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
            } else if let Ok(file_content) = std::fs::read_to_string(&source_path) {
                content.push_str(&format!("# Source: {}\n", source));
                content.push_str(&file_content);
                content.push_str("\n\n");
            }
        }
    }

    content
}

fn build_reference_content(_config_dir: &Path, sources: &[String]) -> String {
    let mut content = String::from("# Auto-generated by wai — do not edit directly\n");
    content.push_str("# References to agent config sources:\n\n");

    for source in sources {
        // The config_dir is .wai/resources/agent-config, so we need to construct
        // paths relative to that
        let source_path = _config_dir.join(source);
        if source_path.exists() && source_path.is_dir() {
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
    }

    content
}

fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
