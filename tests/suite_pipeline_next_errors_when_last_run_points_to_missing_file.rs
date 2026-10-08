#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn pipeline_next_errors_when_last_run_points_to_missing_file() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Write a .last-run that points to a non-existent run file
    let pipelines_dir = tmp.path().join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(
        pipelines_dir.join(".last-run"),
        "ghost-run-2026-01-01-does-not-exist",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .env_remove("WAI_PIPELINE_RUN")
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(
        (((stderr.contains("stale") || stderr.contains("not found"))
            || stderr.contains("deleted"))
            || stderr.contains("No such file"))
    );
}

#[test]
fn pipeline_current_treats_stale_last_run_as_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Point .last-run to a non-existent run file
    let pipelines_dir = tmp.path().join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(
        pipelines_dir.join(".last-run"),
        "ghost-run-2026-01-01-does-not-exist",
    )
    .unwrap();

    // current uses a gentle resolution path — stale .last-run is treated as "no active run"
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
fn pipeline_run_yaml_has_all_required_fields_after_start() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=yaml-check"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path)
        .unwrap()
        .trim()
        .to_string();
    let run_file = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", run_id));
    let content = fs::read_to_string(&run_file).unwrap();

    assert!(
        content.contains("run_id:"),
        "YAML must contain run_id field"
    );
    assert!(
        content.contains("pipeline: my-pipe"),
        "YAML must contain pipeline name"
    );
    assert!(content.contains("yaml-check"), "YAML must contain topic");
    assert!(
        content.contains("created_at:"),
        "YAML must contain created_at timestamp"
    );
    assert!(
        content.contains("current_step: 0"),
        "YAML must start at step 0"
    );
    assert!(
        content.contains("approvals:"),
        "YAML must contain approvals map"
    );
}

#[test]
fn pipeline_run_yaml_updates_correctly_through_lifecycle() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=lifecycle"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let run_id = fs::read_to_string(&last_run_path)
        .unwrap()
        .trim()
        .to_string();
    let run_file = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", run_id));

    // After start: step 0
    let content = fs::read_to_string(&run_file).unwrap();
    assert!(content.contains("current_step: 0"), "Step 0 after start");

    // After first next: step 1
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let content = fs::read_to_string(&run_file).unwrap();
    assert!(
        content.contains("current_step: 1"),
        "Step 1 after first next"
    );

    // After second next: step 2 (complete for a 2-step pipeline)
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let content = fs::read_to_string(&run_file).unwrap();
    assert!(
        content.contains("current_step: 2"),
        "Step 2 (complete) after second next"
    );
}

#[test]
fn pipeline_start_overwrites_last_run_when_starting_new_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Start first run
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=first-run"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let last_run_path = tmp.path().join(".wai/resources/pipelines/.last-run");
    let first_id = fs::read_to_string(&last_run_path)
        .unwrap()
        .trim()
        .to_string();

    // Start second run
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "my-pipe", "--topic=second-run"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let second_id = fs::read_to_string(&last_run_path)
        .unwrap()
        .trim()
        .to_string();

    assert_ne!(first_id, second_id, "Second run should have a different ID");
    assert!(
        second_id.contains("second-run"),
        ".last-run should point to the second run, got: {second_id}"
    );

    // Both run files should exist
    let first_file = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", first_id));
    let second_file = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", second_id));
    assert!(
        first_file.exists(),
        "First run state file should still exist"
    );
    assert!(second_file.exists(), "Second run state file should exist");
}

#[test]
fn pipeline_gates_shows_definitions_for_named_pipeline() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_gated_pipeline_toml(tmp.path(), "gated");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "gates", "gated"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("research"),
        "Should show step 'research', got:\n{stripped}"
    );
    assert!(
        stripped.contains("review"),
        "Should show step 'review', got:\n{stripped}"
    );
    assert!(
        stripped.contains("ship"),
        "Should show step 'ship', got:\n{stripped}"
    );
    assert!(
        stripped.contains("min 2"),
        "Should show structural gate requirement 'min 2', got:\n{stripped}"
    );
}

#[test]
fn pipeline_gates_filters_by_step() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_gated_pipeline_toml(tmp.path(), "gated");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "gates", "gated", "--step=review"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("review"),
        "Should show step 'review', got:\n{stripped}"
    );
    assert!(
        stripped.contains("approv"),
        "Should mention approval gate, got:\n{stripped}"
    );
    assert!(
        !stripped.contains("Step: research"),
        "Should NOT show 'research' step when filtering to 'review', got:\n{stripped}"
    );
}

#[test]
fn pipeline_gates_errors_for_unknown_step() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_gated_pipeline_toml(tmp.path(), "gated");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "gates", "gated", "--step=nonexistent"])
        .output()
        .expect("command should run");
    assert!(!output.status.success());
    let output = output.stderr.clone();

    let stderr = String::from_utf8(output).unwrap();
    assert!(
        stderr.contains("nonexistent") && stderr.contains("not found"),
        "Should report unknown step, got:\n{stderr}"
    );
}

#[test]
fn pipeline_gates_errors_for_unknown_pipeline() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "gates", "no-such-pipe"])
        .output()
        .expect("command should run");
    assert!(!out.status.success());
}

#[test]
fn pipeline_approve_records_timestamp_in_run_state() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_gated_pipeline_toml(tmp.path(), "gated");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "gated", "--topic=test-approve"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Advance to step 2 (review) which has the approval gate — need to bypass step 1 gates first
    // Step 1 has structural gate requiring 2 research artifacts, so we create them
    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-artifact-one.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:research",
        ],
    );
    write_step_artifacts(tmp.path(), "my-proj", &run_id);

    // Now advance past step 1
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Now on step 2 (review) which has approval gate. Run approve.
    let cmd_output = wai_cmd(tmp.path())
        .args(["pipeline", "approve"])
        .output()
        .expect("command should run");
    assert!(cmd_output.status.success());
    assert_approval_confirmed(&cmd_output);

    // Verify the YAML state has an approval entry
    let run_path = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", run_id));
    let yaml = fs::read_to_string(&run_path).unwrap();
    assert!(
        yaml.contains("approvals"),
        "Run state YAML should contain approvals section, got:\n{yaml}"
    );
    assert!(
        yaml.contains("review:") && yaml.contains("2026-"),
        "Run state YAML should contain approval timestamp for 'review' step, got:\n{yaml}"
    );
}

#[test]
fn pipeline_approve_errors_when_run_is_complete() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_gated_pipeline_toml(tmp.path(), "gated");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "gated", "--topic=test-complete"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    // Set current_step past the end to simulate completion
    let run_path = tmp
        .path()
        .join(".wai/pipeline-runs")
        .join(format!("{}.yml", run_id));
    let yaml = format!(
        "run_id: {run_id}\npipeline: gated\ntopic: test-complete\ncreated_at: \"2026-01-01T00:00:00Z\"\ncurrent_step: 3\napprovals: {{}}\n"
    );
    fs::write(&run_path, yaml).unwrap();

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "approve"])
        .output()
        .expect("command should run");
    assert!(!out.status.success());
}

#[test]
fn pipeline_check_passes_when_structural_gate_satisfied() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_gated_pipeline_toml(tmp.path(), "gated");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "gated", "--topic=test-check"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    // Create 2 research artifacts tagged for this run/step
    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-res-a.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:research",
        ],
    );
    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-res-b.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:research",
        ],
    );

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "check"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);
    assert!(
        stripped.contains("PASS"),
        "Gate check should PASS with 2 research artifacts, got:\n{stripped}"
    );
}

#[test]
fn pipeline_check_fails_when_structural_gate_unsatisfied() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_gated_pipeline_toml(tmp.path(), "gated");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "gated", "--topic=test-check-fail"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Only create 1 artifact (need 2)
    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-res-only.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:research",
        ],
    );

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "check"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);
    assert!(
        stripped.contains("requires at least 2"),
        "Gate check should report insufficient artifacts, got:\n{stripped}"
    );
}

#[test]
fn pipeline_next_blocked_by_unsatisfied_gate() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_gated_pipeline_toml(tmp.path(), "gated");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "gated", "--topic=test-blocked"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Try to advance without any artifacts — structural gate should block
    // cmd_next returns Ok(()) but prints gate failure to stdout
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);
    assert!(
        stripped.contains("Gate check failed") || stripped.contains("requires at least"),
        "Should report gate failure, got:\n{stripped}"
    );

    // Verify step was NOT advanced (still at step 0)
    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();
    let run_yaml = fs::read_to_string(
        tmp.path()
            .join(".wai/pipeline-runs")
            .join(format!("{}.yml", run_id)),
    )
    .unwrap();
    assert!(
        run_yaml.contains("current_step: 0"),
        "Step should NOT have advanced, got:\n{run_yaml}"
    );
}

#[test]
fn pipeline_check_reports_missing_approval() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_gated_pipeline_toml(tmp.path(), "gated");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "gated", "--topic=test-approval-check"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    // Satisfy structural gate on step 1 and advance to step 2
    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-r1.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:research",
        ],
    );
    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-r2.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:research",
        ],
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Now on step 2 (review) with approval gate. Check without approving.
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "check"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);
    assert!(
        stripped.contains("Human must approve") || stripped.contains("approval"),
        "Should report missing approval, got:\n{stripped}"
    );
}

#[test]
fn pipeline_gates_live_status_shows_current_step() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_gated_pipeline_toml(tmp.path(), "gated");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "gated", "--topic=test-live-gates"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Run gates with no name (should show live status for current step)
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "gates"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("research"),
        "Live gates should show current step 'research', got:\n{stripped}"
    );
}

#[test]
fn pipeline_check_no_gates_reports_pass() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    // Use the basic pipeline TOML (no gates)
    write_pipeline_toml(tmp.path(), "basic");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "basic", "--topic=no-gates"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "check"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);
    assert!(
        stripped.contains("PASS") || stripped.contains("No gates configured"),
        "Should report pass or no gates, got:\n{stripped}"
    );
}

#[test]
fn pipeline_list_shows_defined_pipelines() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml(tmp.path(), "beta");
    write_pipeline_toml(tmp.path(), "alpha");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "list"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("Pipelines"),
        "Should show pipelines header, got:\n{stripped}"
    );
    assert!(
        stripped.contains("alpha") && stripped.contains("beta"),
        "Should list both pipelines, got:\n{stripped}"
    );

    let alpha_pos = stripped.find("alpha").unwrap();
    let beta_pos = stripped.find("beta").unwrap();
    assert!(
        alpha_pos < beta_pos,
        "Pipelines should be listed alphabetically, got:\n{stripped}"
    );
}

#[test]
fn pipeline_show_displays_metadata_and_steps() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_metadata(tmp.path(), "meta-pipe");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "show", "meta-pipe"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    for expected in [
        "meta-pipe",
        "Pipeline with metadata",
        "Use for metadata-rich testing",
        "tdd, rule-of-5-universal",
        "Steps (2)",
        "research",
        "review",
        "structural",
        "Oracles:",
    ] {
        assert!(
            stripped.contains(expected),
            "Expected '{expected}' in pipeline show output, got:\n{stripped}"
        );
    }
}

#[test]
fn pipeline_validate_succeeds_for_valid_definition() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_metadata(tmp.path(), "valid-pipe");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "validate", "valid-pipe"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let combined = format!(
        "{}{}",
        strip_ansi(&String::from_utf8(output.stdout).unwrap()),
        strip_ansi(&String::from_utf8(output.stderr).unwrap())
    );
    assert!(
        combined.contains("valid-pipe") && combined.contains("steps"),
        "Validate should report success for valid pipeline, got:\n{combined}"
    );
}

#[test]
fn pipeline_validate_fails_for_invalid_toml() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let pipelines_dir = tmp.path().join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(
        pipelines_dir.join("broken.toml"),
        "[pipeline]\nname = \"broken\"\n[[steps]]\nid = ",
    )
    .unwrap();

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "validate", "broken"])
        .output()
        .expect("command should run");
    assert!(!output.status.success());

    let combined = format!(
        "{}{}",
        strip_ansi(&String::from_utf8(output.stdout).unwrap()),
        strip_ansi(&String::from_utf8(output.stderr).unwrap())
    );
    assert!(
        combined.contains("broken") && (combined.contains("parse") || combined.contains("TOML")),
        "Validate should report invalid TOML, got:\n{combined}"
    );
}

/// Write the research-step artifacts tagged to `run_id`.
fn write_step_artifacts(tmp: &std::path::Path, project: &str, run_id: &str) {
    write_artifact_with_tags(
        tmp,
        project,
        "research",
        "2026-01-01-artifact-two.md",
        &[&format!("pipeline-run:{run_id}"), "pipeline-step:research"],
    );
}

/// Assert the approve command confirmed the review step.
fn assert_approval_confirmed(cmd_output: &std::process::Output) {
    let stdout = String::from_utf8(cmd_output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(cmd_output.stderr.clone()).unwrap();
    let all_output = format!("{}{}", strip_ansi(&stdout), strip_ansi(&stderr));
    assert!(
        all_output.contains("Approved step") || all_output.contains("review"),
        "Should confirm approval, got:\n{all_output}"
    );
}
