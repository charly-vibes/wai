use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    // Update notice off in tests: existing assertions expect quiet stderr and
    // must not depend on real ~/.cache state (wai-r3p0 tidy).
    cmd.env("GENESIS_NO_UPDATE_CHECK", "1");
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    let out = wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

fn write_projections_yml(dir: &std::path::Path, content: &str) {
    let path = dir.join(".wai/resources/agent-config/.projections.yml");
    fs::write(path, content).unwrap();
}

// ── successful sync ───────────────────────────────────────────────────────────

#[test]
fn sync_projects_inline_source_to_target_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let source_dir = tmp.path().join(".wai/resources/agent-config/docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("guide.md"), "# Guide").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: GUIDE.md\n    strategy: inline\n    sources: [docs]\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        tmp.path().join("GUIDE.md").exists(),
        "sync should create the projected target file"
    );
}

// ── resume breadcrumb for active pipeline runs (wai-xa8i.2) ─────────────────

/// Write a minimal pipeline TOML (with `[pipeline.metadata]`) into the
/// workspace's pipelines dir so pipeline status discovery can load it.
/// (Test-local copy of prime_test's helper — intentionally not shared.)
fn write_pipeline(dir: &std::path::Path, name: &str, when: &str, steps: &[(&str, &str)]) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let mut toml = String::new();
    toml.push_str("[pipeline]\n");
    toml.push_str(&format!("name = \"{}\"\n", name));
    toml.push_str("description = \"pipeline for tests\"\n");
    toml.push_str("[pipeline.metadata]\n");
    toml.push_str(&format!("when = \"{}\"\n", when));
    for (id, prompt) in steps {
        toml.push_str("[[steps]]\n");
        toml.push_str(&format!("id = \"{}\"\n", id));
        toml.push_str(&format!("prompt = \"{}\"\n", prompt));
    }
    fs::write(pipelines_dir.join(format!("{name}.toml")), toml).unwrap();
}

/// Write an active pipeline run state and point `.last-run` at it.
/// (Test-local copy of prime_test's helper — intentionally not shared.)
fn write_active_run(dir: &std::path::Path, pipeline: &str, current_step: usize) {
    let run_id = format!("{pipeline}-test-run");
    let runs_dir = dir.join(".wai/pipeline-runs");
    fs::create_dir_all(&runs_dir).unwrap();
    let yml = format!(
        "run_id: {run_id}\npipeline: {pipeline}\ntopic: test topic\ncreated_at: '2026-07-29T00:00:00Z'\ncurrent_step: {current_step}\napprovals: {{}}\n"
    );
    fs::write(runs_dir.join(format!("{run_id}.yml")), yml).unwrap();
    fs::write(dir.join(".wai/resources/pipelines/.last-run"), &run_id).unwrap();
}

#[test]
fn sync_active_run_includes_resume_the_orchestration() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let source_dir = tmp.path().join(".wai/resources/agent-config/docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("guide.md"), "# Guide").unwrap();
    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: GUIDE.md\n    strategy: inline\n    sources: [docs]\n",
    );

    // Active mid-flight pipeline run: step 0 of 2.
    write_pipeline(
        tmp.path(),
        "research-flow",
        "Use for research investigation",
        &[("gather", "Gather {topic}"), ("synth", "Synth {topic}")],
    );
    write_active_run(tmp.path(), "research-flow", 0);

    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.to_lowercase().contains("resume the orchestration"),
        "sync output should surface the resume-the-orchestration breadcrumb; got: {stdout}"
    );
}

// ── dry-run: no files written ─────────────────────────────────────────────────

#[test]
fn sync_dry_run_does_not_create_files() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let source_dir = tmp.path().join(".wai/resources/agent-config/docs");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("guide.md"), "# Guide").unwrap();

    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: GUIDE.md\n    strategy: inline\n    sources: [docs]\n",
    );

    let out = wai_cmd(tmp.path())
        .args(["sync", "--dry-run"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        !tmp.path().join("GUIDE.md").exists(),
        "dry-run must not create any files"
    );
}

// ── empty workspace: sync is a no-op ─────────────────────────────────────────

#[test]
fn sync_empty_workspace_succeeds_without_projections() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}
