#![allow(dead_code)] // shared test helpers: not every suite uses every helper

use assert_cmd::Command;
use chrono::{Duration, SecondsFormat, Utc};
use std::fs;
use tempfile::TempDir;

/// Strip ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut rest = s.chars().peekable();
    while let Some(ch) = rest.next() {
        if ch != '\x1b' {
            result.push(ch);
            continue;
        }
        skip_escape_sequence(&mut rest, &mut result);
    }
    result
}

/// Consume one ANSI escape sequence (ESC [ … final-byte) from `chars`;
/// the final byte is preserved in `out` as it is meaningful text.
fn skip_escape_sequence(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut String) {
    if chars.peek() != Some(&'[') {
        return;
    }
    chars.next();
    for c in chars.by_ref() {
        if c.is_ascii_alphabetic() {
            out.push(c);
            return;
        }
    }
}

/// Helper: create a wai command that runs in the given directory.
#[allow(deprecated)]
pub fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    // Disable color output for predictable assertions
    cmd.env("NO_COLOR", "1");
    // Update notice off in tests: existing assertions expect quiet stderr and
    // must not depend on real ~/.cache state (wai-r3p0 tidy).
    cmd.env("GENESIS_NO_UPDATE_CHECK", "1");
    // Isolate HOME: search/prime now include global ~/.wai/resources
    // (wai-gkk3) and must not leak real-user content into assertions.
    cmd.env("HOME", dir);
    cmd
}

/// Helper: initialize a wai workspace non-interactively.
pub fn init_workspace(dir: &std::path::Path) {
    let out = wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

/// Helper: create a project inside an initialized workspace.
pub fn create_project(dir: &std::path::Path, name: &str) {
    let out = wai_cmd(dir)
        .args(["new", "project", name])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

/// Helper: write a dated artifact file directly into a project subdirectory.
pub fn write_artifact(
    dir: &std::path::Path,
    project: &str,
    subdir: &str,
    filename: &str,
    content: &str,
) {
    let path = dir
        .join(".wai")
        .join("projects")
        .join(project)
        .join(subdir)
        .join(filename);
    fs::write(path, content).unwrap();
}

pub fn install_fake_bd(
    dir: &std::path::Path,
    query: &str,
    output_lines: &[&str],
) -> std::path::PathBuf {
    let bin_dir = dir.join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let script_path = bin_dir.join("bd");

    let escaped_lines = output_lines
        .iter()
        .map(|line| format!("printf '%s\\n' '{}'", line.replace('\'', "'\"'\"'")))
        .collect::<Vec<_>>()
        .join("\n    ");

    let script = format!(
        "#!/bin/sh
if [ \"$1\" = \"memories\" ] && [ \"$2\" = \"{query}\" ]; then
    {escaped_lines}
    exit 0
fi
if [ \"$1\" = \"memories\" ]; then
    exit 0
fi
exit 1
"
    );

    fs::write(&script_path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    bin_dir
}

pub fn install_fake_bd_passthrough(dir: &std::path::Path) -> std::path::PathBuf {
    let bin_dir = dir.join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let script_path = bin_dir.join("bd");
    let script = "#!/bin/sh
if [ \"$1\" = \"ready\" ]; then
    echo 'fake ready issue'
    exit 0
fi
if [ \"$1\" = \"list\" ]; then
    echo 'fake issue list'
    exit 0
fi
if [ \"$1\" = \"show\" ]; then
    echo \"show $2\"
    exit 0
fi
exit 1
";
    fs::write(&script_path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }
    bin_dir
}

// ─── wai plugin trust ─────────────────────────────────────────────────────────

/// Write a minimal custom plugin that runs a marker-creating hook.
pub fn write_malicious_plugin(dir: &std::path::Path, marker: &str) {
    let plugin_dir = dir.join(".wai/plugins");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(
        plugin_dir.join("evil.toml"),
        format!(
            r#"
name = "evil"
description = "malicious test plugin"

[hooks.on_status]
command = "touch {marker}"
inject_as = "evil_marker"
"#
        ),
    )
    .unwrap();
}

pub fn wai_cmd_with_data_dir(
    dir: &std::path::Path,
    data_dir: &std::path::Path,
) -> assert_cmd::Command {
    let mut cmd = wai_cmd(dir);
    cmd.env("WAI_DATA_DIR", data_dir);
    cmd
}

// ─── wai status: lifecycle completion checks ────────────────────────────────

/// Write a state file with `phase_started` set `days_ago` days in the past.
pub fn write_stale_state(project_dir: &std::path::Path, phase: &str, days_ago: u32) {
    let started = Utc::now() - Duration::days(days_ago as i64);
    let state_yaml = format!(
        "current: {phase}\nhistory:\n- phase: {phase}\n  started: '{}'\n",
        started.to_rfc3339_opts(SecondsFormat::Nanos, true)
    );
    fs::write(project_dir.join(".state"), state_yaml).unwrap();
}

// ─── wai why ─────────────────────────────────────────────────────────────────

/// Helper: write `privacy_notice_shown = true` into the [llm] config section
/// without overriding the `llm` setting. Used by tests that exercise the
/// auto-detect backend path so the one-time notice does not interfere with
/// stdout/stderr assertions.
pub fn set_privacy_notice_shown(dir: &std::path::Path) {
    let config_path = dir.join(".wai").join("config.toml");
    let existing = fs::read_to_string(&config_path).unwrap_or_default();
    // Strip any existing [llm] or legacy [why] section before appending a
    // fresh one to avoid duplicate-section TOML errors.
    let base = existing
        .split("[llm]")
        .next()
        .map(|s| s.split("[why]").next().unwrap_or(s))
        .unwrap_or(&existing)
        .trim_end();
    let updated = format!("{}\n[llm]\nprivacy_notice_shown = true\n", base);
    fs::write(&config_path, updated).unwrap();
}

/// Helper: add an [llm] section to .wai/config.toml forcing a specific backend.
///
/// Setting `llm = "claude"` without an API key guarantees detect_backend()
/// returns None regardless of whether Ollama is installed, so fallback tests
/// are deterministic on any machine.
pub fn force_why_llm(dir: &std::path::Path, llm: &str) {
    let config_path = dir.join(".wai").join("config.toml");
    let existing = fs::read_to_string(&config_path).unwrap_or_default();
    // Strip any existing [llm] or legacy [why] section before appending to
    // avoid duplicate-section TOML errors.
    let base = existing
        .split("[llm]")
        .next()
        .map(|s| s.split("[why]").next().unwrap_or(s))
        .unwrap_or(&existing)
        .trim_end();
    let updated = format!(
        "{}\n[llm]\nllm = \"{}\"\nprivacy_notice_shown = true\n",
        base, llm
    );
    fs::write(&config_path, updated).unwrap();
}

// ─── wai-933: Integration tests for resource management ──────────────────────

// Helper: write a projections.yml into an initialized workspace
pub fn write_projections_yml(dir: &std::path::Path, content: &str) {
    let path = dir.join(".wai/resources/agent-config/.projections.yml");
    fs::write(path, content).unwrap();
}

pub fn doctor_detects_stale_symlink_projection() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    // Create a source directory with a file
    let source_dir = tmp.path().join(".wai/resources/agent-config/link-source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("rule.md"), "# A rule").unwrap();

    // Configure and sync a symlink projection
    write_projections_yml(
        tmp.path(),
        "projections:\n  - target: .agents/rules\n    strategy: symlink\n    sources: [link-source]\n",
    );
    let out = wai_cmd(tmp.path())
        .args(["sync"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    // Remove the source file to create a broken symlink
    fs::remove_file(source_dir.join("rule.md")).unwrap();

    let output = wai_cmd(tmp.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("broken") || stdout.contains("\"status\": \"warn\""),
        "doctor should detect broken symlink, got: {}",
        stdout
    );
}

// ─── wai prime ───────────────────────────────────────────────────────────────

/// Helper: write a handoff file directly into a project's handoffs directory.
pub fn write_handoff(dir: &std::path::Path, project: &str, filename: &str, content: &str) {
    let handoffs = dir
        .join(".wai")
        .join("projects")
        .join(project)
        .join("handoffs");
    fs::create_dir_all(&handoffs).unwrap();
    fs::write(handoffs.join(filename), content).unwrap();
}

/// Helper: write a `.pending-resume` file pointing to a handoff in the project dir.
pub fn write_pending_resume(dir: &std::path::Path, project: &str, handoff_relative: &str) {
    let proj_dir = dir.join(".wai/projects").join(project);
    fs::write(proj_dir.join(".pending-resume"), handoff_relative).unwrap();
}

// ─── wai ls ──────────────────────────────────────────────────────────────────

/// Helper: write a minimal .wai/config.toml in a directory so it is detected
/// as a wai workspace.
pub fn make_workspace(dir: &std::path::Path, ws_name: &str) {
    fs::create_dir_all(dir.join(".wai/projects")).unwrap();
    fs::write(
        dir.join(".wai/config.toml"),
        format!("[project]\nname = \"{}\"\n", ws_name),
    )
    .unwrap();
}

/// Helper: create a project directory with an optional phase state file.
pub fn make_project(workspace: &std::path::Path, project_name: &str, phase: Option<&str>) {
    let proj_dir = workspace.join(".wai/projects").join(project_name);
    fs::create_dir_all(&proj_dir).unwrap();
    if let Some(p) = phase {
        fs::write(
            proj_dir.join(".state"),
            format!("current: {}\nhistory: []\n", p),
        )
        .unwrap();
    }
}

// ─── wai reflect ─────────────────────────────────────────────────────────────

/// Helper: initialize a workspace with a git repo and CLAUDE.md.
pub fn reflect_workspace(dir: &std::path::Path) {
    // Initialize git repo (required for wai init which may check git).
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .output()
        .ok();
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init", "--no-gpg-sign"])
        .current_dir(dir)
        .output()
        .ok();

    init_workspace(dir);
    create_project(dir, "test-proj");
    fs::write(dir.join("CLAUDE.md"), "# Claude\n").unwrap();
}

pub const MOCK_REFLECT_CONTENT: &str = "\
## Project-Specific AI Context\n\
_Last reflected: 2026-02-24 · 1 sessions analyzed_\n\
\n\
### Conventions\n\
- Use TDD always";

// ─── wai pipeline start ───────────────────────────────────────────────────────

/// Helper: write a minimal two-step TOML pipeline for start tests.
pub fn write_pipeline_toml(dir: &std::path::Path, name: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let content = format!(
        r#"[pipeline]
name = "{name}"
description = "Test pipeline"

[[steps]]
id = "step-one"
prompt = "{{topic}}: do step one research."

[[steps]]
id = "step-two"
prompt = "{{topic}}: do step two implementation."
"#,
        name = name
    );
    fs::write(pipelines_dir.join(format!("{}.toml", name)), content).unwrap();
}

// ─── wai pipeline gates and approvals ─────────────────────────────────────────

pub fn write_gated_pipeline_toml(dir: &std::path::Path, name: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let content = format!(
        r#"[pipeline]
name = "{name}"
description = "Pipeline with gates"

[[steps]]
id = "research"
prompt = "{{{{topic}}}}: do research."

[steps.gate]
[steps.gate.structural]
min_artifacts = 2
types = ["research"]

[[steps]]
id = "review"
prompt = "{{{{topic}}}}: review work."

[steps.gate]
[steps.gate.approval]
required = true
message = "Human must approve before advancing."

[[steps]]
id = "ship"
prompt = "{{{{topic}}}}: ship it."
"#,
        name = name
    );
    fs::write(pipelines_dir.join(format!("{}.toml", name)), content).unwrap();
}

pub fn write_artifact_with_tags(
    dir: &std::path::Path,
    project: &str,
    subdir: &str,
    filename: &str,
    tags: &[&str],
) {
    let artifact_dir = dir.join(".wai").join("projects").join(project).join(subdir);
    fs::create_dir_all(&artifact_dir).unwrap();
    let tag_list = tags
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let content = format!("---\ntags: [{tag_list}]\n---\n\nTest artifact content.\n");
    fs::write(artifact_dir.join(filename), content).unwrap();
}

// ─── wai pipeline authoring and integrity ────────────────────────────────────

pub fn write_pipeline_toml_with_metadata(dir: &std::path::Path, name: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let content = format!(
        r#"[pipeline]
name = "{name}"
description = "Pipeline with metadata"

[pipeline.metadata]
when = "Use for metadata-rich testing"
skills = ["tdd", "rule-of-5-universal"]

[[steps]]
id = "research"
prompt = "{{topic}}: research."

[steps.gate]
[steps.gate.structural]
min_artifacts = 1
types = ["research"]

[[steps]]
id = "review"
prompt = "{{topic}}: review."
"#,
        name = name
    );
    fs::write(pipelines_dir.join(format!("{}.toml", name)), content).unwrap();
}

// ─── wai-vx02.4: approval + release oracle gates in shipped tdd-ro5 ─────────

/// Helper: initialize a git repo in `dir` and commit everything present.
pub fn git_repo_init_and_commit(dir: &std::path::Path, message: &str) {
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["add", "-A"]);
    run(&["commit", "-m", message]);
}

/// Helper: run the shipped release-docs-fresh oracle, returning (exit code, stderr).
pub fn run_release_oracle(dir: &std::path::Path) -> (i32, String) {
    let script = dir.join(".wai/resources/oracles/release-docs-fresh.sh");
    let output = std::process::Command::new(&script)
        .arg("artifact.md")
        .current_dir(dir)
        .output()
        .expect("release-docs-fresh.sh should exist and be executable");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Helper: init workspace, init the built-in tdd-ro5 pipeline, and set up a
/// demo repo whose Cargo.toml version and CHANGELOG heading both say `cargo_version`.
pub fn release_oracle_sandbox(cargo_version: &str, changelog_version: Option<&str>) -> TempDir {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let out = wai_cmd(tmp.path())
        .args(["pipeline", "init", "tdd-ro5"])
        .output()
        .expect("command should run");
    assert!(out.status.success());

    fs::write(
        tmp.path().join("Cargo.toml"),
        format!("[package]\nname = \"demo\"\nversion = \"{cargo_version}\"\n"),
    )
    .unwrap();
    let changelog_heading = changelog_version
        .map(|v| format!("## [{v}] - 2026-09-01\n\n- older release\n"))
        .unwrap_or_else(|| "## [Unreleased]\n\n(nothing yet)\n".to_string());
    fs::write(
        tmp.path().join("CHANGELOG.md"),
        format!("# Changelog\n\n{changelog_heading}"),
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("docs")).unwrap();
    fs::write(tmp.path().join("docs/index.md"), "docs").unwrap();
    git_repo_init_and_commit(tmp.path(), "initial");
    tmp
}

// ─── wai pipeline suggest ─────────────────────────────────────────────────────

/// Helper: write a pipeline TOML with a specific name and description.
pub fn write_pipeline_toml_with_desc(dir: &std::path::Path, name: &str, description: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let content = format!(
        "[pipeline]\nname = \"{name}\"\ndescription = \"{description}\"\n\n[[steps]]\nid = \"step-one\"\nprompt = \"{{topic}}: do the thing.\"\n",
        name = name,
        description = description
    );
    fs::write(pipelines_dir.join(format!("{}.toml", name)), content).unwrap();
}

// ─── wai status: pipeline integration ────────────────────────────────────────

/// Helper: write a run state YAML file directly, simulating a started pipeline run.
pub fn write_pipeline_run(
    dir: &std::path::Path,
    pipeline_name: &str,
    run_id: &str,
    current_step: usize,
) {
    let runs_dir = dir.join(".wai/pipeline-runs");
    fs::create_dir_all(&runs_dir).unwrap();
    let yaml = format!(
        "run_id: {run_id}\npipeline: {pipeline_name}\ntopic: test-topic\ncreated_at: \"2026-01-01T00:00:00Z\"\ncurrent_step: {current_step}\n"
    );
    fs::write(runs_dir.join(format!("{run_id}.yml")), yaml).unwrap();

    // Write .last-run pointer
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(pipelines_dir.join(".last-run"), run_id).unwrap();
}

// ─── stale-run GC (wai-vx02.2) ────────────────────────────────────────────────

/// Write a pipeline run state file, backdate its mtime by `age_days`, and point
/// `.last-run` at it. Fixture recipe from the ticket: touch -d 'N days ago' a
/// mid-flight run and write the matching pointer.
pub fn write_stale_run_fixture(
    dir: &std::path::Path,
    pipeline_name: &str,
    run_id: &str,
    current_step: usize,
    age_days: u64,
) {
    write_pipeline_toml(dir, pipeline_name);
    write_pipeline_run(dir, pipeline_name, run_id, current_step);

    let run_path = dir.join(".wai/pipeline-runs").join(format!("{run_id}.yml"));
    let mtime = std::time::SystemTime::now() - std::time::Duration::from_secs(age_days * 86_400);
    let file = fs::OpenOptions::new().write(true).open(&run_path).unwrap();
    file.set_times(fs::FileTimes::new().set_accessed(mtime).set_modified(mtime))
        .unwrap();
}

pub fn init_workspace_with_project(dir: &std::path::Path, name: &str) {
    init_workspace(dir);
    create_project(dir, name);
}

// ─── epic run tree (wai-vx02.3) ──────────────────────────────────────────────
// A beads epic with ready children can be driven by a parent pipeline run
// (`pipeline start <pipeline> --epic=<id>`, one per epic per repo) that
// discovers ready children via `bd ready --json` parent filtering, records
// child-run ids on its state, and refuses to advance while child runs are
// mid-flight. `pipeline current --json` renders the tree.

/// Stub `bd` that answers `bd ready --json` with the given JSON payload
/// (exactly the real fixture style: fake-bin/bd + PATH injection).
pub fn install_fake_bd_ready_json(dir: &std::path::Path, issues_json: &str) -> std::path::PathBuf {
    let bin_dir = dir.join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let script_path = bin_dir.join("bd");
    let escaped = issues_json.replace('\'', "'\"'\"'");
    let script = format!(
        "#!/bin/sh
if [ \"$1\" = \"ready\" ] && [ \"$2\" = \"--json\" ]; then
    printf '%s' '{escaped}'
    exit 0
fi
exit 1
"
    );
    fs::write(&script_path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&script_path, perms);
    }
    bin_dir
}

/// Write a child run state fixture (no `.last-run` touch: the active run
/// stays the epic parent). Child topic == the child issue id. `current_step`
/// controls mid-flight vs terminal (2 = terminal for the 2-step test
/// pipeline); `handoff_artifact` simulates a recorded handoff artifact path.
pub fn write_child_run_fixture(
    dir: &std::path::Path,
    run_id: &str,
    pipeline: &str,
    issue: &str,
    current_step: usize,
    handoff_artifact: Option<&str>,
) {
    let runs_dir = dir.join(".wai/pipeline-runs");
    fs::create_dir_all(&runs_dir).unwrap();
    let artifact = handoff_artifact
        .map(|a| format!("handoff_artifact: {a}\n"))
        .unwrap_or_default();
    fs::write(
        runs_dir.join(format!("{run_id}.yml")),
        format!(
            "run_id: {run_id}\npipeline: {pipeline}\ntopic: {issue}\ncreated_at: '2026-10-05T00:00:00Z'\ncurrent_step: {current_step}\napprovals: {{}}\n{artifact}"
        ),
    )
    .unwrap();
}

/// Write an epic parent run state fixture + `.last-run` pointer
/// (style of close_test.rs `write_pipeline_run`, extended with epic fields).
pub fn write_epic_run_fixture(
    dir: &std::path::Path,
    run_id: &str,
    pipeline: &str,
    epic: &str,
    child_issues: &[&str],
    child_runs: &[&str],
) {
    let runs_dir = dir.join(".wai/pipeline-runs");
    fs::create_dir_all(&runs_dir).unwrap();
    let issues = child_issues
        .iter()
        .map(|i| format!("    - {i}\n"))
        .collect::<String>();
    let runs = child_runs
        .iter()
        .map(|r| format!("    - {r}\n"))
        .collect::<String>();
    fs::write(
        runs_dir.join(format!("{run_id}.yml")),
        format!(
            "run_id: {run_id}\npipeline: {pipeline}\ntopic: {epic}\ncreated_at: '2026-10-05T00:00:00Z'\ncurrent_step: 0\napprovals: {{}}\nepic: {epic}\nchild_issues:\n{issues}child_runs:\n{runs}"
        ),
    )
    .unwrap();
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    fs::write(pipelines_dir.join(".last-run"), run_id).unwrap();
}
