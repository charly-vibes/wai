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

/// Write a minimal pipeline TOML (with `[pipeline.metadata]`) into the
/// workspace's pipelines dir so prime can discover it.
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

// ── session orientation ───────────────────────────────────────────────────────

#[test]
fn prime_single_project_shows_orientation_output() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project: myproject"))
        .stdout(predicate::str::contains("wai prime"));
}

// ── project selection ─────────────────────────────────────────────────────────

#[test]
fn prime_project_flag_selects_named_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "alpha", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project: alpha"))
        .stdout(predicate::str::contains("Project: beta").not());
}

// ── failure: unknown project ──────────────────────────────────────────────────

#[test]
fn prime_unknown_project_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    wai_cmd(tmp.path())
        .args(["prime", "--project", "doesnotexist", "--no-input"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("doesnotexist"));
}

// ── pipelines section (wai-knbl) ──────────────────────────────────────────────

#[test]
fn prime_shows_available_pipeline_and_start_suggestion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject"); // default phase: research
    write_pipeline(
        tmp.path(),
        "research-flow",
        "Use for research investigation",
        &[("gather", "Gather {topic}")],
    );

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pipelines"))
        .stdout(predicate::str::contains("research-flow"))
        .stdout(predicate::str::contains(
            "wai pipeline start research-flow --topic=<topic>",
        ));
}

#[test]
fn prime_shows_active_pipeline_run_step() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_pipeline(
        tmp.path(),
        "research-flow",
        "Use for research investigation",
        &[("gather", "Gather {topic}"), ("synth", "Synth {topic}")],
    );
    write_active_run(tmp.path(), "research-flow", 0);

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PIPELINE ACTIVE"))
        .stdout(predicate::str::contains("research-flow"))
        .stdout(predicate::str::contains("step 1/2"));
}

#[test]
fn prime_omits_start_suggestion_when_no_pipeline_matches_phase() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject"); // phase: research
    write_pipeline(
        tmp.path(),
        "deploy-bot",
        "Frontier-level computation requiring systematic validation",
        &[("run", "Run {topic}")],
    );

    wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .assert()
        .success()
        // Pipeline is still listed (available pipelines are shown) ...
        .stdout(predicate::str::contains("deploy-bot"))
        // ... but no start suggestion because `when` doesn't match the phase.
        .stdout(predicate::str::contains("wai pipeline start").not());
}
