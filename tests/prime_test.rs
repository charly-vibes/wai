use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

// Reuses the shared fake-bd helper (wai-xa8i.3) without importing the rest of
// `common` — this file defines its own local fixture helpers.
mod common;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    // Update notice off in tests: existing assertions expect quiet stderr and
    // must not depend on real ~/.cache state (wai-r3p0 tidy).
    cmd.env("GENESIS_NO_UPDATE_CHECK", "1");
    // Isolate HOME: prime now lists global ~/.wai/resources/patterns
    // (wai-gkk3) and must not depend on real-user content.
    cmd.env("HOME", dir);
    cmd
}

fn init_workspace(dir: &std::path::Path) {
    let out = wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .output()
        .expect("command should run");
    assert!(out.status.success());
}

fn create_project(dir: &std::path::Path, name: &str) {
    let out = wai_cmd(dir)
        .args(["new", "project", name])
        .output()
        .expect("command should run");
    assert!(out.status.success());
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

/// Write a pipeline TOML WITHOUT a `[pipeline.metadata]` section. This triggers
/// a `wai doctor` warning ("Missing [pipeline.metadata]") — used to force a
/// non-clean health summary in status/prime tests.
fn write_pipeline_no_metadata(dir: &std::path::Path, name: &str) {
    let pipelines_dir = dir.join(".wai/resources/pipelines");
    fs::create_dir_all(&pipelines_dir).unwrap();
    let toml = format!(
        "[pipeline]\nname = \"{name}\"\ndescription = \"no metadata\"\n[[steps]]\nid = \"one\"\nprompt = \"do {{topic}}\"\n"
    );
    fs::write(pipelines_dir.join(format!("{name}.toml")), toml).unwrap();
}

// ── session orientation ───────────────────────────────────────────────────────

#[test]
fn prime_lists_prior_research_docs() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "demo");
    let research = tmp.path().join(".wai/projects/demo/research");
    fs::create_dir_all(&research).unwrap();
    fs::write(
        research.join("2026-09-17-cache-eviction-investigation.md"),
        "---\ntags: []\n---\n\nline one\nline two\nline three\nline four\nline five\nline six\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["prime"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Prior context"));
    assert!(stdout.contains("2026-09-17-cache-eviction-investigation.md"));
}

#[test]
fn prime_plans_render_body_title_not_frontmatter() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "demo");
    let plans = tmp.path().join(".wai/projects/demo/plans");
    fs::create_dir_all(&plans).unwrap();
    fs::write(
        plans.join("2026-09-17-my-plan.md"),
        "---\ntags: []\n---\n\nMy plan title: evict caches in two phases\n\nmore detail\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .args(["prime"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("My plan title: evict caches in two phases"));
    assert!(!(stdout.contains("• ---")));
}

#[test]
fn prime_single_project_shows_orientation_output() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project: myproject"));
    assert!(stdout.contains("wai prime"));
}

// ── project selection ─────────────────────────────────────────────────────────

#[test]
fn prime_project_flag_selects_named_project() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "alpha");
    create_project(tmp.path(), "beta");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "alpha", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Project: alpha"));
    assert!(!(stdout.contains("Project: beta")));
}

// ── failure: unknown project ──────────────────────────────────────────────────

#[test]
fn prime_unknown_project_fails_with_diagnostic() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "doesnotexist", "--no-input"])
        .output()
        .expect("command should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(stderr.contains("doesnotexist"));
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

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Pipelines"));
    assert!(stdout.contains("research-flow"));
    assert!(stdout.contains("wai pipeline start research-flow --topic=<topic>"));
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

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("PIPELINE RUN ADOPT/RESUME"));
    assert!(stdout.contains("research-flow-test-run"));
    assert!(stdout.contains("research-flow"));
    assert!(stdout.contains("step 1/2"));
    assert!(stdout.contains("wai pipeline next"));
}

#[test]
fn prime_active_run_directives_resume_the_orchestration() {
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

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.to_lowercase().contains("resume the orchestration"));
}

#[test]
fn prime_adopts_active_run_in_json() {
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

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input", "--json"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"run_id\": \"research-flow-test-run\""));
    assert!(stdout.contains("\"pipeline\": \"research-flow\""));
    assert!(stdout.contains("\"next_command\": \"wai pipeline next\""));
}

#[test]
fn prime_no_adopt_block_without_active_run() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_pipeline(
        tmp.path(),
        "research-flow",
        "Use for research investigation",
        &[("gather", "Gather {topic}")],
    );

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("ADOPT/RESUME")));
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

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("deploy-bot"));
    assert!(!(stdout.contains("wai pipeline start")));
}

// ── doctor health summary (wai-j56n) ─────────────────────────────────────────

#[test]
fn prime_surfaces_doctor_warning_when_not_clean() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    // A pipeline without [pipeline.metadata] triggers a doctor Warn.
    write_pipeline_no_metadata(tmp.path(), "orphan");

    let out = wai_cmd(tmp.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("warning"));
    assert!(stdout.contains("wai doctor"));
}

#[test]
fn prime_silent_on_health_when_doctor_is_clean() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    // Stub HOME so the claude-session-hook check passes (it needs
    // ~/.claude/settings.json with a wai status SessionStart hook).
    let home = tmp.path().join("fake-home");
    let claude_dir = home.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_dir.join("settings.json"),
        r#"{
  "hooks": {
    "SessionStart": [
      {"matcher": "", "hooks": [{"type": "command", "command": "wai status 2>/dev/null || true"}]}
    ]
  }
}"#,
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .env("HOME", &home)
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!(stdout.contains("wai doctor")));
}

// ── global patterns observability (wai-gkk3) ──────────────────────────────────

/// A global pattern (~/.wai/resources/patterns/*.md) is listed by prime with
/// its name and first heading, so repo-session agents discover the canon.
#[test]
fn prime_lists_global_patterns_when_present() {
    let tmp = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    let patterns = home.path().join(".wai/resources/patterns");
    fs::create_dir_all(&patterns).unwrap();
    fs::write(
        patterns.join("orchestrator-subagents.md"),
        "# Orchestrator Subagents\n\nCanon body.\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .env("HOME", home.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("prime should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("Global Patterns"));
    assert!(stdout.contains("orchestrator-subagents"));
    assert!(stdout.contains("Orchestrator Subagents"));
}

/// No global patterns dir → no Global Patterns section at all.
#[test]
fn prime_omits_global_patterns_section_when_empty() {
    let tmp = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");

    let out = wai_cmd(tmp.path())
        .env("HOME", home.path())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("prime should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(!stdout.contains("Global Patterns"));
}

/// When a mid-flight pipeline run is active, the epics-flow suggestion from
/// `bd ready --json` must be suppressed (terminal AND JSON next_steps): the
/// pipeline-run resume directive is authoritative and a competing
/// "Suggested next: bd show <id>" confuses agents about which system to
/// follow (wai-xa8i.3, openspec resume-pipeline-adoption).
#[test]
fn prime_active_run_suppresses_suggested_next() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    write_pipeline(
        tmp.path(),
        "research-flow",
        "Use for research investigation",
        &[("gather", "Gather {topic}"), ("synth", "Synth {topic}")],
    );
    // Mid-flight: step 0 of 2.
    write_active_run(tmp.path(), "research-flow", 0);
    // Fake `bd ready --json` emits one ready issue so the epics-flow
    // suggestion would fire on the ungated implementation.
    let fake_bin = common::install_fake_bd_ready_json(tmp.path(), r#"[{"id":"wai-1234"}]"#);
    let path = format!(
        "{}:{}",
        fake_bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = wai_cmd(tmp.path())
        .env("PATH", path.clone())
        .args(["prime", "--project", "myproject", "--no-input"])
        .output()
        .expect("command should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    // Fixture sanity: the run IS adopted as active and mid-flight.
    assert!(stdout.contains("PIPELINE RUN ADOPT/RESUME"));
    assert!(stdout.contains("wai pipeline next"));
    // The epics-flow suggestion must NOT appear alongside the run directive.
    assert!(
        !stdout.contains("Suggested next:"),
        "prime must not suggest `bd show` when a mid-flight pipeline run is active"
    );

    // The JSON payload agrees: no next_steps fallback from bd ready.
    let out = wai_cmd(tmp.path())
        .env("PATH", path)
        .args(["prime", "--project", "myproject", "--no-input", "--json"])
        .output()
        .expect("prime --json should run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(
        !stdout.contains("bd show wai-1234"),
        "JSON next_steps must not carry the epics-flow suggestion under an active run"
    );
}

/// The JSON payload carries the same list as an additive field.
#[test]
fn prime_json_includes_global_patterns() {
    let tmp = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "myproject");
    let patterns = home.path().join(".wai/resources/patterns");
    fs::create_dir_all(&patterns).unwrap();
    fs::write(
        patterns.join("orchestrator-subagents.md"),
        "# Orchestrator Subagents\n\nCanon body.\n",
    )
    .unwrap();

    let out = wai_cmd(tmp.path())
        .env("HOME", home.path())
        .args(["prime", "--project", "myproject", "--no-input", "--json"])
        .output()
        .expect("prime --json should succeed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"global_patterns\""));
    assert!(stdout.contains("orchestrator-subagents"));
}
