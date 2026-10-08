#![allow(clippy::too_many_lines)]

mod common;
use common::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn pipeline_gates_tdd_ro5_shows_approval_on_ship_close() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "tdd-ro5"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "gates", "tdd-ro5"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();
    let stripped = strip_ansi(&String::from_utf8(output).unwrap());
    let ship_idx = stripped
        .find("ship-close")
        .expect("ship-close step must be shown, got:\n{stripped}");
    assert!(
        stripped[ship_idx..].contains("Approval"),
        "gates output must show the approval tier on ship-close, got:\n{stripped}"
    );

    // The shipped template must still validate.
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "validate", "tdd-ro5"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

#[test]
fn pipeline_lock_creates_lock_files_for_step_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_pipeline_toml(tmp.path(), "basic");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "basic", "--topic=lock-test"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-locked.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:step-one",
        ],
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "lock"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let lock_path = tmp
        .path()
        .join(".wai/projects/my-proj/research/2026-01-01-locked.md")
        .with_file_name(format!("2026-01-01-locked.md.{}.lock", run_id));
    assert!(lock_path.exists(), "Expected lock file at {:?}", lock_path);
}

#[test]
fn pipeline_verify_passes_after_locking_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_pipeline_toml(tmp.path(), "basic");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "basic", "--topic=verify-pass"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-verify.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:step-one",
        ],
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "lock"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "verify"])
        .output()
        .expect("command should run");
    assert!(output.status.success());

    let combined = format!(
        "{}{}",
        strip_ansi(&String::from_utf8(output.stdout).unwrap()),
        strip_ansi(&String::from_utf8(output.stderr).unwrap())
    );
    assert!(
        combined.contains("verified") || combined.contains("All 1 locked artifacts verified"),
        "Verify should pass for intact artifacts, got:\n{combined}"
    );
}

#[test]
fn pipeline_verify_fails_when_locked_artifact_is_tampered() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-proj");
    write_pipeline_toml(tmp.path(), "basic");

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "start", "basic", "--topic=verify-fail"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    let run_id = fs::read_to_string(tmp.path().join(".wai/resources/pipelines/.last-run"))
        .unwrap()
        .trim()
        .to_string();

    let artifact_path = tmp
        .path()
        .join(".wai/projects/my-proj/research/2026-01-01-tampered.md");
    write_artifact_with_tags(
        tmp.path(),
        "my-proj",
        "research",
        "2026-01-01-tampered.md",
        &[
            &format!("pipeline-run:{}", run_id),
            "pipeline-step:step-one",
        ],
    );

    let out = wai_cmd(tmp.path())
        .args(["pipeline", "lock"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    fs::write(&artifact_path, "tampered content\n").unwrap();

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "verify"])
        .output()
        .expect("command should run");
    assert!(!output.status.success());

    let combined = format!(
        "{}{}",
        strip_ansi(&String::from_utf8(output.stdout).unwrap()),
        strip_ansi(&String::from_utf8(output.stderr).unwrap())
    );
    assert!(
        combined.contains("failed verification") || combined.contains("expected sha256:"),
        "Verify should report hash mismatch, got:\n{combined}"
    );
}

#[test]
fn pipeline_suggest_lists_all_pipelines() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "auth", "Authentication workflow");
    write_pipeline_toml_with_desc(tmp.path(), "database", "Database migration workflow");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("auth"),
        "Output should contain 'auth' pipeline, got:\n{stripped}"
    );
    assert!(
        stripped.contains("database"),
        "Output should contain 'database' pipeline, got:\n{stripped}"
    );

    // Both pipelines listed, should show start hint
    assert!(
        stripped.contains("wai pipeline start"),
        "Output should contain a start hint, got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_with_description_ranks_keyword_match_first() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "auth", "Authentication and login workflow");
    write_pipeline_toml_with_desc(
        tmp.path(),
        "database",
        "Database migration and schema changes",
    );

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest", "auth login"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // "auth" should appear before "database" in the output
    let auth_pos = stripped.find("auth").expect("'auth' should be in output");
    let db_pos = stripped
        .find("database")
        .expect("'database' should be in output");
    assert!(
        auth_pos < db_pos,
        "'auth' should appear before 'database' when query matches 'auth login', got:\n{stripped}"
    );

    // The start hint should point to the top result (auth)
    assert!(
        stripped.contains("wai pipeline start auth"),
        "Start hint should point to 'auth', got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_no_match_still_shows_all_alphabetically() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "zebra", "Zebra workflow");
    write_pipeline_toml_with_desc(tmp.path(), "apple", "Apple workflow");

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest", "xyzzy-nomatch-token"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Both should appear
    assert!(
        stripped.contains("apple"),
        "Output should contain 'apple', got:\n{stripped}"
    );
    assert!(
        stripped.contains("zebra"),
        "Output should contain 'zebra', got:\n{stripped}"
    );

    // Alphabetically "apple" comes before "zebra" when scores are tied at 0
    let apple_pos = stripped.find("apple").unwrap();
    let zebra_pos = stripped.find("zebra").unwrap();
    assert!(
        apple_pos < zebra_pos,
        "'apple' should appear before 'zebra' (alphabetical tie-break), got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_no_pipelines_prints_hint() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    // Ensure the pipelines dir exists but is empty (no TOML files)
    fs::create_dir_all(tmp.path().join(".wai/resources/pipelines")).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("No pipelines"),
        "Output should say 'No pipelines', got:\n{stripped}"
    );
    assert!(
        stripped.contains("wai pipeline init"),
        "Output should hint 'wai pipeline init', got:\n{stripped}"
    );
}

#[test]
fn pipeline_suggest_empty_string_treated_as_no_description() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    write_pipeline_toml_with_desc(tmp.path(), "beta", "Beta workflow");
    write_pipeline_toml_with_desc(tmp.path(), "alpha", "Alpha workflow");

    // With empty string — should behave same as no description: alphabetical order
    let output = wai_cmd(tmp.path())
        .args(["pipeline", "suggest", ""])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    // Should show both pipelines
    assert!(
        stripped.contains("alpha"),
        "Output should contain 'alpha', got:\n{stripped}"
    );
    assert!(
        stripped.contains("beta"),
        "Output should contain 'beta', got:\n{stripped}"
    );

    // Alphabetically: "alpha" before "beta"
    let alpha_pos = stripped.find("alpha").unwrap();
    let beta_pos = stripped.find("beta").unwrap();
    assert!(
        alpha_pos < beta_pos,
        "'alpha' should appear before 'beta' (alphabetical, empty string = no scoring), got:\n{stripped}"
    );
}

#[test]
fn status_shows_pipeline_active_when_run_exists() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create the pipeline TOML definition
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Simulate a started run at step 0 (step 1/2)
    write_pipeline_run(tmp.path(), "my-pipe", "my-pipe-2026-01-01-test-topic", 0);

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("PIPELINE ACTIVE"),
        "Output should contain 'PIPELINE ACTIVE', got:\n{stripped}"
    );
    assert!(
        stripped.contains("my-pipe"),
        "Output should contain pipeline name 'my-pipe', got:\n{stripped}"
    );
    assert!(
        stripped.contains("step 1/"),
        "Output should contain 'step 1/', got:\n{stripped}"
    );
}

#[test]
fn status_shows_pipeline_current_suggestion_when_active() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    write_pipeline_toml(tmp.path(), "my-pipe");
    write_pipeline_run(tmp.path(), "my-pipe", "my-pipe-2026-01-01-test-topic", 0);

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("wai pipeline current"),
        "Output should suggest 'wai pipeline current', got:\n{stripped}"
    );
}

#[test]
fn status_shows_available_pipelines_when_no_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create a pipeline TOML but no active run
    write_pipeline_toml(tmp.path(), "my-pipe");

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("Available pipelines"),
        "Output should contain 'Available pipelines', got:\n{stripped}"
    );
    assert!(
        stripped.contains("my-pipe"),
        "Output should contain pipeline name 'my-pipe', got:\n{stripped}"
    );
}

#[test]
fn status_shows_pipeline_suggest_when_pipelines_exist() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create a pipeline TOML but no active run
    write_pipeline_toml(tmp.path(), "my-pipe");

    let output = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        stripped.contains("wai pipeline suggest"),
        "Output should suggest 'wai pipeline suggest', got:\n{stripped}"
    );
}

#[test]
fn status_ignores_stale_last_run_pointer() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    // Create a pipeline TOML
    write_pipeline_toml(tmp.path(), "my-pipe");

    // Write .last-run pointing to a nonexistent run file (stale pointer)
    let pipelines_dir = tmp.path().join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(pipelines_dir.join(".last-run"), "nonexistent-run-id").unwrap();

    // Should NOT crash and should show "Available pipelines" instead
    let output = wai_cmd(tmp.path())
        .args(["status"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let output = output.stdout.clone();

    let stdout = String::from_utf8(output).unwrap();
    let stripped = strip_ansi(&stdout);

    assert!(
        !stripped.contains("PIPELINE ACTIVE"),
        "Should not show PIPELINE ACTIVE for stale pointer, got:\n{stripped}"
    );
    assert!(
        stripped.contains("Available pipelines"),
        "Should show Available pipelines when pointer is stale, got:\n{stripped}"
    );
}

// ─── wai way verbose / json agnostic fields ──────────────────────────────────

#[test]
fn way_verbose_shows_intent() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Intent:"));
}

#[test]
fn way_verbose_shows_success_criteria() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "-v"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Success:"));
}

#[test]
fn way_json_includes_intent() {
    let tmp = TempDir::new().unwrap();

    let out = wai_cmd(tmp.path())
        .args(["way", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"intent\""));
}

#[test]
fn way_plugin_toml_parsed() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Place a TOML plugin file with intent and success_criteria
    let plugins_dir = tmp.path().join(".wai/plugins");
    fs::create_dir_all(&plugins_dir).unwrap();
    fs::write(
        plugins_dir.join("my-check.toml"),
        r#"name = "my-check"
description = "A custom check"
intent = "Ensure the repo has a CODEOWNERS file."
success_criteria = "CODEOWNERS file exists in the repo root or .github/ directory."
"#,
    )
    .unwrap();

    // Plugin list is the command that exercises detect_plugins
    let out = wai_cmd(tmp.path())
        .args(["plugin", "list", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"my-check\""));
}

// ─── Project Resolution (WAI_PROJECT env var) ────────────────────────────────

#[test]
fn phase_show_single_project_auto_detects() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "only-project");

    let out = wai_cmd(tmp.path())
        .args(["phase", "show"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("only-project"));
}

#[test]
fn phase_show_multiple_projects_no_context_errors_non_interactive() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    // Without WAI_PROJECT or --project, non-interactive should error
    let out = wai_cmd(tmp.path())
        .args(["--no-input", "phase", "show"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("alpha"));
    assert!(stderr.contains("beta"));
}

#[test]
fn phase_show_wai_project_env_selects_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .env("WAI_PROJECT", "beta")
        .args(["phase", "show"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("beta"));
}

#[test]
fn phase_show_wai_project_invalid_errors() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");

    let out = wai_cmd(tmp.path())
        .env("WAI_PROJECT", "nonexistent")
        .args(["phase", "show"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("nonexistent"));
}

#[test]
fn phase_show_wai_project_empty_treated_as_unset() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "only-project");

    // Empty WAI_PROJECT should fall through to auto-detect
    let out = wai_cmd(tmp.path())
        .env("WAI_PROJECT", "")
        .args(["phase", "show"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("only-project"));
}

#[test]
fn phase_show_project_flag_overrides_wai_project_env() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    // --project flag should win over WAI_PROJECT env
    let out = wai_cmd(tmp.path())
        .env("WAI_PROJECT", "alpha")
        .args(["phase", "--project", "beta", "show"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("beta"));
}

#[test]
fn phase_show_project_flag_invalid_errors() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");

    let out = wai_cmd(tmp.path())
        .args(["phase", "--project", "nonexistent", "show"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("nonexistent"));
}

#[test]
fn phase_next_project_flag_only_mutates_target() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    // Advance alpha to design
    let out = wai_cmd(tmp.path())
        .args(["phase", "--project", "alpha", "next"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Alpha should now be in design
    let out = wai_cmd(tmp.path())
        .args(["phase", "--project", "alpha", "show"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    assert!(
        stdout.contains("design"),
        "expected alpha in design phase: {}",
        stdout
    );

    // Beta should still be in research (unaffected)
    let out = wai_cmd(tmp.path())
        .args(["phase", "--project", "beta", "show"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
    let out = out.stdout.clone();
    let stdout = strip_ansi(&String::from_utf8_lossy(&out));
    assert!(
        stdout.contains("research"),
        "expected beta still in research phase: {}",
        stdout
    );
}

// ─── wai project use ─────────────────────────────────────────────────────────

#[test]
fn project_use_valid_prints_export() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .env("SHELL", "/bin/bash")
        .args(["project", "use", "myproj"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("WAI_PROJECT='myproj'"),
        "expected WAI_PROJECT=myproj in: {}",
        stdout
    );
}

#[test]
fn project_use_invalid_errors() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproj");

    let out = wai_cmd(tmp.path())
        .args(["project", "use", "nonexistent"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("nonexistent"));
}

#[test]
fn project_use_no_args_lists_projects() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["project", "use"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("alpha"));
    assert!(stdout.contains("beta"));
}
