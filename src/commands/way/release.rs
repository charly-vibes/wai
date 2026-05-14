use std::path::Path;

use super::{CheckResult, Status};

pub(super) fn has_binary_target(repo_root: &Path) -> bool {
    // Rust: src/main.rs or [[bin]] in Cargo.toml
    if repo_root.join("src/main.rs").exists() {
        return true;
    }
    if let Ok(content) = std::fs::read_to_string(repo_root.join("Cargo.toml"))
        && content.contains("[[bin]]")
    {
        return true;
    }
    // Go: any top-level .go file with "package main"
    if let Ok(entries) = std::fs::read_dir(repo_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("go")
                && let Ok(content) = std::fs::read_to_string(&path)
                && content.contains("package main")
            {
                return true;
            }
        }
    }
    // Python: [project.scripts] or [tool.poetry.scripts] in pyproject.toml
    if let Ok(content) = std::fs::read_to_string(repo_root.join("pyproject.toml"))
        && (content.contains("[project.scripts]") || content.contains("[tool.poetry.scripts]"))
    {
        return true;
    }
    false
}

pub(super) fn has_cargo_dist_in_toml(repo_root: &Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(repo_root.join("Cargo.toml")) {
        return content.contains("[workspace.metadata.dist]")
            || content.contains("[package.metadata.dist]");
    }
    false
}

pub(super) fn has_release_workflow(repo_root: &Path) -> bool {
    let workflows_dir = repo_root.join(".github/workflows");
    if !workflows_dir.exists() {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let is_yaml = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "yml" || e == "yaml");
            if !is_yaml {
                continue;
            }
            // Filename contains "release"
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("release"))
            {
                return true;
            }
            // Content has tag trigger matching "v*"
            if let Ok(content) = std::fs::read_to_string(&path)
                && content.contains("tags:")
                && content.contains("v*")
            {
                return true;
            }
        }
    }
    false
}

pub(super) fn check_release_pipeline(repo_root: &Path) -> CheckResult {
    let name = "Automated delivery";
    let intent = Some(
        "Automate the process of building, packaging, and publishing software releases."
            .to_string(),
    );
    let success_criteria = Some(
        "Software releases and distribution (packages, binaries) are fully automated.".to_string(),
    );

    if !has_binary_target(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Library project — release pipeline not required".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if repo_root.join(".goreleaser.yml").exists() || repo_root.join(".goreleaser.yaml").exists() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "goreleaser detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if repo_root.join("dist.toml").exists() || has_cargo_dist_in_toml(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "cargo-dist detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if has_release_workflow(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "GitHub Actions release workflow detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    CheckResult {
        name: name.to_string(),
        status: Status::Info,
        message: "No release pipeline found".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Consider goreleaser (Go/Rust/any) or cargo-dist (Rust) to automate GitHub releases and publish to Homebrew/Scoop".to_string(),
        ),
    }
}
