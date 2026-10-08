#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn project_use_fish_shell_syntax() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("SHELL", "/usr/bin/fish")
        .args(["project", "use", "myproj"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("set -gx WAI_PROJECT 'myproj'"),
        "expected fish syntax in: {}",
        stdout
    );
}

#[test]
fn pipeline_init_creates_toml_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "my-workflow"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("my-workflow"));

    let toml_path = tmp.path().join(".wai/resources/pipelines/my-workflow.toml");
    assert!(toml_path.exists(), "Expected TOML file at {:?}", toml_path);

    let content = fs::read_to_string(&toml_path).unwrap();
    // Check the pipeline name is set correctly
    assert!(
        content.contains("name = \"my-workflow\""),
        "Expected pipeline name in TOML: {}",
        content
    );
    // Check both steps are present
    assert!(
        content.contains("id = \"step-one\""),
        "Expected step-one in TOML: {}",
        content
    );
    assert!(
        content.contains("id = \"step-two\""),
        "Expected step-two in TOML: {}",
        content
    );
    // Check the {topic} placeholder is present (not substituted)
    assert!(
        content.contains("{topic}"),
        "Expected {{topic}} placeholder in TOML: {}",
        content
    );
    // Check the convention comment is present
    assert!(
        content.contains("navigation hints"),
        "Expected convention comment in TOML: {}",
        content
    );
}

#[test]
fn pipeline_init_creates_pipelines_dir_if_absent() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let pipelines_dir = tmp.path().join(".wai/resources/pipelines");
    assert!(
        !pipelines_dir.exists(),
        "Pipelines dir should not exist yet"
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "new-pipe"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    assert!(
        pipelines_dir.exists(),
        "Pipelines dir should have been created"
    );
}

#[test]
fn pipeline_init_fails_if_file_already_exists() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create the pipeline the first time — should succeed
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "duplicate"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Try again — should fail with a clear error
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "duplicate"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("already exists"));
}

#[test]
fn pipeline_init_rejects_invalid_name() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "Bad Name!"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("Invalid character"));
}

#[test]
fn pipeline_init_template_is_valid_toml() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "valid-check"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp.path().join(".wai/resources/pipelines/valid-check.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Verify the generated file parses as valid TOML
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Generated TOML should be valid, but got error: {:?}",
        parsed.err()
    );
}

#[test]
fn pipeline_init_tdd_ro5_uses_autonomous_ro5u_template() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "tdd-ro5"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let toml_path = tmp.path().join(".wai/resources/pipelines/tdd-ro5.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    for expected in [
        "id = \"orient\"",
        "id = \"ro5u-review\"",
        "id = \"quality-ledger\"",
        "id = \"ship-close\"",
        "QUALITY LEDGER",
    ] {
        assert!(
            content.contains(expected),
            "Expected {expected} in TOML: {content}"
        );
    }
}

#[test]
fn pipeline_start_creates_run_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=auth-refactor"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // A run state file should exist under .wai/pipeline-runs/
    let runs_dir = tmp.path().join(".wai/pipeline-runs");
    assert!(runs_dir.exists(), ".wai/pipeline-runs/ should be created");

    let entries: Vec<_> = fs::read_dir(&runs_dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yml"))
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "Exactly one run state file should be written"
    );

    // The run state file name should contain the pipeline name and topic slug
    let run_file = &entries[0].path();
    let stem = run_file.file_stem().unwrap().to_string_lossy().to_string();
    assert!(
        stem.contains("my-pipe"),
        "Run ID should contain pipeline name, got: {stem}"
    );
    assert!(
        stem.contains("auth-refactor"),
        "Run ID should contain topic slug, got: {stem}"
    );

    // The run state YAML should parse and have correct fields
    let content = fs::read_to_string(run_file).unwrap();
    assert!(
        content.contains("pipeline: my-pipe"),
        "Run state should record pipeline name"
    );
    assert!(
        content.contains("auth-refactor"),
        "Run state should record topic"
    );
    assert!(
        content.contains("current_step: 0"),
        "New run should start at step 0"
    );
}

#[test]
fn pipeline_start_writes_last_run_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=feature-x"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    assert!(
        last_run_path.exists(),
        ".last-run pointer file should be written"
    );

    let run_id = fs::read_to_string(&last_run_path).unwrap();
    let run_id = run_id.trim();
    assert!(
        !run_id.is_empty(),
        ".last-run should contain a non-empty run ID"
    );
    assert!(
        run_id.starts_with("my-pipe"),
        "Run ID should start with pipeline name, got: {run_id}"
    );
    assert!(
        run_id.contains("feature-x"),
        "Run ID should contain topic slug, got: {run_id}"
    );
}

#[test]
fn pipeline_start_prints_first_step_prompt() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should contain env export line
    assert!(
        stripped.contains("WAI_PIPELINE_RUN="),
        "Output should contain WAI_PIPELINE_RUN export line, got:\n{stripped}"
    );

    // Should contain step 1/N header
    assert!(
        stripped.contains("step 1/2"),
        "Output should contain 'step 1/2' header, got:\n{stripped}"
    );

    // Should contain the rendered prompt with topic substituted
    assert!(
        stripped.contains("my-topic"),
        "Output should contain topic in rendered prompt, got:\n{stripped}"
    );

    // Should NOT contain the literal {topic} placeholder
    assert!(
        !stripped.contains("{topic}"),
        "Output should not contain literal {{topic}} placeholder, got:\n{stripped}"
    );
}

#[test]
fn pipeline_start_fails_for_unknown_pipeline() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "nonexistent", "--topic=foo"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("nonexistent") || stderr.contains("not found")));
}

// ─── wai pipeline next ────────────────────────────────────────────────────────

#[test]
fn pipeline_next_advances_to_second_step() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run first
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Read .last-run to get run ID
    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path).unwrap();
    let run_id = run_id.trim().to_string();

    // Advance to next step
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should show step 2/2
    assert!(
        stripped.contains("step 2/2"),
        "Output should contain 'step 2/2', got:\n{stripped}"
    );

    // The run state file should have current_step incremented to 1
    let run_file = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", run_id));
    let content = fs::read_to_string(&run_file).unwrap();
    assert!(
        content.contains("current_step: 1"),
        "Run state should show current_step: 1 after advancing, got:\n{content}"
    );
}

#[test]
fn pipeline_next_on_last_step_shows_completion() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Advance once (step 1 → step 2)
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Advance again (step 2 → completion)
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should show completion block
    assert!(
        stripped.contains("complete"),
        "Output should contain 'complete', got:\n{stripped}"
    );

    // Should contain wai close suggestion
    assert!(
        stripped.contains("wai close"),
        "Output should contain 'wai close' suggestion, got:\n{stripped}"
    );
}

#[test]
fn pipeline_next_errors_when_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No pipeline started, no .last-run file, no WAI_PIPELINE_RUN env
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("No active pipeline run") || stderr.contains("pipeline start")));
}

#[test]
fn pipeline_next_errors_when_run_already_complete() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Advance through all steps (2-step pipeline)
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Third call should fail — run is already complete
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("already complete") || stderr.contains("complete")));
}

// ─── wai pipeline current ─────────────────────────────────────────────────────

#[test]
fn pipeline_current_reprints_step_prompt() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a pipeline run
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Call pipeline current — should reprint step 1 without advancing
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should contain step 1/2 header
    assert!(
        stripped.contains("step 1/2"),
        "Output should contain 'step 1/2', got:\n{stripped}"
    );

    // Should contain the rendered prompt with topic substituted
    assert!(
        stripped.contains("my-topic"),
        "Output should contain topic in rendered prompt, got:\n{stripped}"
    );

    // Should NOT contain the literal {topic} placeholder
    assert!(
        !stripped.contains("{topic}"),
        "Output should not contain literal {{topic}} placeholder, got:\n{stripped}"
    );

    // Verify state was NOT advanced — current_step should still be 0
    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path).unwrap();
    let run_id = run_id.trim().to_string();
    let run_file = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", run_id));
    let content = fs::read_to_string(&run_file).unwrap();
    assert!(
        content.contains("current_step: 0"),
        "Run state must NOT have advanced after 'current'; got:\n{content}"
    );
}

#[test]
fn pipeline_current_after_next_shows_step_2() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Advance once
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Now pipeline current should show step 2
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("step 2/2"),
        "Output should contain 'step 2/2' after one advance, got:\n{stripped}"
    );
}

#[test]
fn pipeline_current_errors_when_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // No pipeline started, no .last-run file, no WAI_PIPELINE_RUN env
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!((stderr.contains("No active pipeline run") || stderr.contains("pipeline start")));
}

#[test]
fn pipeline_current_json_reports_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current", "--json"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let payload: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(payload["data"]["active"], true);
    assert_eq!(payload["data"]["pipeline"], "my-pipe");
    assert_eq!(payload["data"]["topic"], "my-topic");
    assert_eq!(payload["data"]["step"]["index"], 1);
    assert_eq!(payload["data"]["step"]["id"], "step-one");
    assert!(
        payload["data"]["step"]["prompt"]
            .as_str()
            .unwrap()
            .contains("my-topic")
    );
    assert_eq!(payload["data"]["next_command"], "wai pipeline next");
    assert!(
        payload["data"]["run_id"]
            .as_str()
            .unwrap()
            .contains("my-pipe")
    );
    assert!(payload["data"].get("gate_summary").is_some());
}

#[test]
fn pipeline_current_json_reports_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current", "--json"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let payload: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(payload["data"]["active"], false);
    assert!(
        payload["data"]["message"]
            .as_str()
            .unwrap()
            .contains("No active pipeline run")
    );
    assert_eq!(
        payload["data"]["next_command"],
        "wai pipeline start <name> --topic=<topic>"
    );
}

#[test]
fn pipeline_status_json_reports_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=my-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let output = wai_cmd(tmp.path())
        .args(["--json", "pipeline", "status"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let payload: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(payload["data"]["active"], true);
    assert_eq!(payload["data"]["pipeline"], "my-pipe");
    assert_eq!(payload["data"]["step"]["id"], "step-one");
}

#[test]
fn pipeline_current_on_complete_run_prints_done() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=test-topic"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Advance through all steps (2-step pipeline)
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // pipeline current on a completed run should indicate done (not error)
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "current"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("complete")
            || stripped.contains("done")
            || stripped.contains("wai close"),
        "Output should indicate the pipeline is complete, got:\n{stripped}"
    );
}

// ─── wai pipeline run lifecycle (focused coverage) ───────────────────────────

#[test]
fn pipeline_next_resolves_run_via_env_var() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start a run and capture the run ID
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=env-test"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path)
        .unwrap()
        .trim()
        .to_string();

    // Delete .last-run to force env var resolution
    fs::remove_file(&last_run_path).unwrap();

    // Advance using WAI_PIPELINE_RUN env var
    let output = wai_cmd(tmp.path())
        .env("WAI_PIPELINE_RUN", &run_id)
        .args(["pipeline", "next"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8(output).unwrap());
    assert!(
        stripped.contains("step 2/2"),
        "Should advance to step 2/2 via env var, got:\n{stripped}"
    );
}

#[test]
fn pipeline_current_resolves_run_via_env_var() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=env-test"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path)
        .unwrap()
        .trim()
        .to_string();

    // Delete .last-run to force env var resolution
    fs::remove_file(&last_run_path).unwrap();

    let output = wai_cmd(tmp.path())
        .env("WAI_PIPELINE_RUN", &run_id)
        .args(["pipeline", "current"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stripped = strip_ansi(&String::from_utf8(output).unwrap());
    assert!(
        stripped.contains("step 1/2"),
        "Should show step 1/2 via env var, got:\n{stripped}"
    );
}
