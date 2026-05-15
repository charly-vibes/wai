use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
}

fn create_project(dir: &std::path::Path, name: &str) {
    wai_cmd(dir)
        .args(["new", "project", name])
        .assert()
        .success();
}

// ── wai artifacts stale ──────────────────────────────────────────────────────

#[test]
fn artifacts_stale_clean_workspace_exits_zero_with_no_stale() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "proj");

    // No artifacts with `tracks` → clean
    wai_cmd(tmp.path())
        .args(["artifacts", "stale"])
        .assert()
        .success();
}

#[test]
fn artifacts_stale_json_on_clean_workspace_is_valid_json() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "proj");

    let output = wai_cmd(tmp.path())
        .args(["artifacts", "stale", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&text).expect("artifacts stale --json must emit valid JSON");

    assert!(parsed.get("stale").is_some(), "JSON missing 'stale' key");
    assert!(parsed.get("clean").is_some(), "JSON missing 'clean' key");
    assert!(
        parsed.get("stale_count").is_some(),
        "JSON missing 'stale_count' key"
    );
}

#[test]
fn artifacts_stale_detects_changed_tracked_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "proj");

    // Create a tracked file
    let tracked_file = tmp.path().join("src/commands/target.rs");
    fs::create_dir_all(tracked_file.parent().unwrap()).unwrap();
    fs::write(&tracked_file, "// original content").unwrap();

    // Add a research artifact that tracks the file
    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "design note",
            "--project",
            "proj",
            "--tracks",
            "src/commands/target.rs",
        ])
        .assert()
        .success();

    // Modify the tracked file
    fs::write(&tracked_file, "// changed content").unwrap();

    // wai artifacts stale should report the artifact as stale
    wai_cmd(tmp.path())
        .args(["artifacts", "stale"])
        .assert()
        .success()
        .stdout(predicate::str::contains("stale"));
}

#[test]
fn artifacts_stale_reports_untracked_artifact_separately() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "proj");

    // Create a tracked file
    let tracked_file = tmp.path().join("src/thing.rs");
    fs::create_dir_all(tracked_file.parent().unwrap()).unwrap();
    fs::write(&tracked_file, "// content").unwrap();

    // Add a research artifact with tracks
    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "analysis",
            "--project",
            "proj",
            "--tracks",
            "src/thing.rs",
        ])
        .assert()
        .success();

    // Manually delete the sidecar to simulate "untracked" state
    let research_dir = tmp.path().join(".wai/projects/proj/research");
    let sidecars: Vec<_> = fs::read_dir(&research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".fresh.lock"))
        .collect();

    for sidecar in &sidecars {
        fs::remove_file(sidecar.path()).unwrap();
    }

    // JSON output should have untracked entry
    let output = wai_cmd(tmp.path())
        .args(["artifacts", "stale", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let untracked = parsed["untracked"].as_array().unwrap();
    assert!(!untracked.is_empty(), "expected untracked artifact in JSON");
}

// ── wai add --tracks ─────────────────────────────────────────────────────────

#[test]
fn add_research_with_tracks_writes_frontmatter_and_sidecar() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "proj");

    // Create the tracked file so the sidecar can hash it
    let tracked_file = tmp.path().join("src/commands/status.rs");
    fs::create_dir_all(tracked_file.parent().unwrap()).unwrap();
    fs::write(&tracked_file, "// status implementation").unwrap();

    wai_cmd(tmp.path())
        .args([
            "add",
            "research",
            "status analysis",
            "--project",
            "proj",
            "--tracks",
            "src/commands/status.rs",
        ])
        .assert()
        .success();

    let research_dir = tmp.path().join(".wai/projects/proj/research");
    let artifacts: Vec<_> = fs::read_dir(&research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().into_string().unwrap();
            name.ends_with(".md")
        })
        .collect();
    assert_eq!(artifacts.len(), 1, "expected one artifact");

    let content = fs::read_to_string(artifacts[0].path()).unwrap();
    assert!(
        content.contains("tracks:"),
        "artifact frontmatter must contain 'tracks:'"
    );
    assert!(
        content.contains("src/commands/status.rs"),
        "artifact frontmatter must contain the tracked path"
    );

    // Sidecar must exist
    let sidecars: Vec<_> = fs::read_dir(&research_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".fresh.lock"))
        .collect();
    assert_eq!(sidecars.len(), 1, "expected one freshness sidecar");
}
