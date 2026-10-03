//! Repo-convention checks for `wai way`.

use genesis::doctor::CheckStatus;
use std::path::Path;

use crate::commands::way::WayCheckEntry;

/// Shared outcome for "tool configured with/without enforced threshold".
fn threshold_outcome(
    enforced: bool,
    configured_msg: &str,
    threshold_hint: &str,
) -> (CheckStatus, String, Option<String>) {
    if enforced {
        (
            CheckStatus::Pass,
            format!("{configured_msg} (threshold enforced)"),
            None,
        )
    } else {
        (
            CheckStatus::Pass,
            configured_msg.to_string(),
            Some(threshold_hint.to_string()),
        )
    }
}

/// Rust coverage: tarpaulin (config or Cargo metadata) or cargo-llvm-cov.
fn detect_rust_coverage(repo_root: &Path) -> Option<(CheckStatus, String, Option<String>)> {
    const TARP_HINT: &str = "Add `fail-under` to tarpaulin config to enforce a minimum — https://github.com/xd009642/tarpaulin";

    let tarpaulin_toml = repo_root.join("tarpaulin.toml");
    if tarpaulin_toml.exists() {
        let has_threshold = std::fs::read_to_string(&tarpaulin_toml)
            .ok()
            .is_some_and(|c| c.contains("fail-under"));
        return Some(threshold_outcome(
            has_threshold,
            "tarpaulin configured",
            TARP_HINT,
        ));
    }

    let cargo_content = std::fs::read_to_string(repo_root.join("Cargo.toml")).ok()?;
    if cargo_content.contains("[package.metadata.tarpaulin]")
        || cargo_content.contains("[workspace.metadata.tarpaulin]")
    {
        let has_threshold = cargo_content.contains("fail-under");
        return Some(threshold_outcome(
            has_threshold,
            "tarpaulin configured",
            TARP_HINT,
        ));
    }
    // Rust — cargo-llvm-cov via dev-dependencies
    if cargo_content.contains("cargo-llvm-cov") {
        return Some((
            CheckStatus::Pass,
            "cargo-llvm-cov detected".to_string(),
            Some(
                "Configure a threshold via CI flags or llvm-cov config — https://github.com/taiki-e/cargo-llvm-cov"
                    .to_string(),
            ),
        ));
    }
    None
}

/// Python coverage: coverage.py (.coveragerc or pyproject.toml).
fn detect_python_coverage(repo_root: &Path) -> Option<(CheckStatus, String, Option<String>)> {
    const PY_HINT: &str = "Add `fail_under` to [tool.coverage.report] in pyproject.toml — https://coverage.readthedocs.io";

    let coveragerc = repo_root.join(".coveragerc");
    if coveragerc.exists() {
        let has_threshold = std::fs::read_to_string(&coveragerc)
            .ok()
            .is_some_and(|c| c.contains("fail_under"));
        return Some(threshold_outcome(
            has_threshold,
            "coverage.py configured",
            PY_HINT,
        ));
    }
    let pyproject = std::fs::read_to_string(repo_root.join("pyproject.toml")).ok()?;
    if !pyproject.contains("[tool.coverage.report]") {
        return None;
    }
    let has_threshold = pyproject.contains("fail_under");
    Some(threshold_outcome(
        has_threshold,
        "coverage.py configured",
        PY_HINT,
    ))
}

/// JavaScript/TypeScript coverage: vitest, nyc, or c8.
fn detect_js_coverage(repo_root: &Path) -> Option<(CheckStatus, String, Option<String>)> {
    const NYC_HINT: &str = "Add branch/line thresholds to nyc or c8 config — https://github.com/istanbuljs/nyc · https://github.com/bcoe/c8";
    const VITEST_HINT: &str = "Add `thresholds` to the coverage block in vitest.config — https://vitest.dev/config/#coverage";

    // JavaScript / TypeScript — vitest
    for config_name in ["vitest.config.ts", "vitest.config.js", "vitest.config.mts"] {
        let content = std::fs::read_to_string(repo_root.join(config_name)).unwrap_or_default();
        if content.contains("coverage") {
            let has_threshold = content.contains("thresholds");
            return Some(threshold_outcome(
                has_threshold,
                "vitest coverage configured",
                VITEST_HINT,
            ));
        }
    }

    // JavaScript / TypeScript — nyc or c8
    let nycrc = repo_root.join(".nycrc");
    let nycrc_json = repo_root.join(".nycrc.json");
    let c8_config = repo_root.join("c8.config.js");
    let nyc_content = nycrc
        .exists()
        .then(|| std::fs::read_to_string(&nycrc).ok())
        .flatten()
        .or_else(|| {
            nycrc_json
                .exists()
                .then(|| std::fs::read_to_string(&nycrc_json).ok())
                .flatten()
        })
        .or_else(|| {
            c8_config
                .exists()
                .then(|| std::fs::read_to_string(&c8_config).ok())
                .flatten()
        });
    if let Some(content) = nyc_content {
        let has_threshold = content.contains("branches") || content.contains("lines");
        return Some(threshold_outcome(
            has_threshold,
            "nyc/c8 configured",
            NYC_HINT,
        ));
    }
    let pkg = std::fs::read_to_string(repo_root.join("package.json")).ok()?;
    if !(pkg.contains("\"nyc\"") || pkg.contains("\"c8\"")) {
        return None;
    }
    let has_threshold = pkg.contains("branches") || pkg.contains("lines");
    Some(threshold_outcome(
        has_threshold,
        "nyc/c8 configured",
        NYC_HINT,
    ))
}

/// Any language — codecov / coveralls reporting service.
fn detect_reporting_service(repo_root: &Path) -> Option<(CheckStatus, String, Option<String>)> {
    let has_service = repo_root.join(".codecov.yml").exists()
        || repo_root.join("codecov.yml").exists()
        || repo_root.join(".coveralls.yml").exists();
    if !has_service {
        return None;
    }
    Some((
        CheckStatus::Pass,
        "Coverage reporting service detected (codecov/coveralls)".to_string(),
        Some("Add a coverage threshold in the config to enforce minimums".to_string()),
    ))
}

pub(crate) fn check_test_coverage(repo_root: &Path) -> WayCheckEntry {
    let name = "Test coverage";
    let intent = Some(
        "Enforced coverage thresholds catch regressions automatically and keep quality high."
            .to_string(),
    );
    let success_criteria =
        Some("A coverage tool is configured with a minimum threshold.".to_string());
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

    let detected = detect_rust_coverage(repo_root)
        .or_else(|| detect_python_coverage(repo_root))
        .or_else(|| detect_js_coverage(repo_root))
        .or_else(|| detect_reporting_service(repo_root));

    match detected {
        Some((status, message, suggestion)) => mk(status, message, suggestion),
        None => mk(
            CheckStatus::Warn,
            "No coverage tool configured".to_string(),
            Some(
                "Configure a coverage tool with an enforced threshold — Rust: https://github.com/xd009642/tarpaulin · Python: https://coverage.readthedocs.io · JS: https://github.com/istanbuljs/nyc"
                    .to_string(),
            ),
        ),
    }
}
