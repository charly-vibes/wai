//! Integration tests for the `wai matrix` command group (decision matrix).
//!
//! Covers openspec change `add-decision-matrix`: core scaffold (tasks 1.x),
//! deciding (3.x), renderer (4.x), lints (5.x), and phase gate (6.1).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd
}

fn init_workspace(dir: &Path) {
    wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
}

fn create_project(dir: &Path, name: &str) {
    wai_cmd(dir)
        .args(["new", "project", name])
        .assert()
        .success();
}

fn matrix_dir(dir: &Path, project: &str) -> std::path::PathBuf {
    dir.join(".wai/projects")
        .join(project)
        .join("designs/matrix")
}

// ── 1.1 matrix init ───────────────────────────────────────────────────────────

#[test]
fn matrix_init_scaffolds_expected_layout() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["matrix", "init", "Which storage engine should we adopt?"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    assert!(m.join("problem.md").exists(), "problem.md must exist");
    assert!(m.join("criteria").is_dir(), "criteria/ must exist");
    assert!(
        m.join("approaches/01-status-quo").is_dir(),
        "approaches/01-status-quo/ must exist"
    );
    assert!(
        m.join("approaches/01-status-quo/_description.md").exists(),
        "status quo needs _description.md"
    );
    assert!(m.join("decision.md").exists(), "decision.md must exist");

    let problem = fs::read_to_string(m.join("problem.md")).unwrap();
    assert!(
        problem.contains("Which storage engine should we adopt?"),
        "problem statement must be written into problem.md"
    );
}

// 1.2 — status quo is the first column on init (verified via _description.md)

#[test]
fn matrix_init_twice_fails_naming_existing_matrix() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");

    wai_cmd(tmp.path())
        .args(["matrix", "init", "first"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "init", "second"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("matrix"));
}

#[test]
fn matrix_init_requires_workspace() {
    let tmp = TempDir::new().unwrap();
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .failure();
}

// ── 1.3 criterion add ─────────────────────────────────────────────────────────

#[test]
fn matrix_criterion_add_touches_every_approach() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "operational-cost"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    let crit = m.join("criteria/01-operational-cost.md");
    assert!(crit.exists(), "criterion definition file must exist");
    assert!(
        m.join("approaches/01-status-quo/01-operational-cost")
            .is_dir(),
        "cell dir must be created in status quo"
    );
    assert!(
        m.join("approaches/02-event-sourcing/01-operational-cost")
            .is_dir(),
        "cell dir must be created in every approach"
    );
}

// ── 1.4 approach add ──────────────────────────────────────────────────────────

#[test]
fn matrix_approach_add_creates_empty_cells_for_all_criteria() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "impact"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "risk"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    let approach = m.join("approaches/02-event-sourcing");
    assert!(
        approach.join("_description.md").exists(),
        "new approach needs _description.md"
    );
    assert!(approach.join("01-impact").is_dir());
    assert!(approach.join("02-risk").is_dir());
}

// ── 3.1/3.2 decide ─────────────────────────────────────────────────────────

#[test]
fn matrix_decide_writes_decision_and_design_doc() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args([
            "matrix",
            "decide",
            "02-event-sourcing",
            "Best audit story at acceptable ops cost.",
        ])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    let decision = fs::read_to_string(m.join("decision.md")).unwrap();
    assert!(
        decision.contains("02-event-sourcing"),
        "decision must name the approach"
    );
    assert!(decision.contains("Best audit story at acceptable ops cost."));
    assert!(
        decision.contains("Decided at: 2"),
        "must record a UTC timestamp"
    );

    let designs = tmp.path().join(".wai/projects/my-app/designs");
    let entries: Vec<_> = fs::read_dir(&designs).unwrap().collect();
    let doc = entries.iter().find_map(|e| {
        let name = e.as_ref().unwrap().file_name().into_string().unwrap();
        name.starts_with("2026-")
            .then(|| e.as_ref().unwrap().path())
    });
    let doc = doc.expect("design doc must be scaffolded in designs/");
    let body = fs::read_to_string(&doc).unwrap();
    assert!(
        body.contains("tracks:"),
        "design doc needs frontmatter tracks"
    );
    assert!(
        body.contains("Best audit story at acceptable ops cost."),
        "rationale in doc"
    );
}

#[test]
fn matrix_decide_rejects_unknown_approach_listing_valid() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "decide", "02-nope", "because"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("01-status-quo"));
}

#[test]
fn matrix_decide_snapshot_includes_winning_column_facts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "impact"])
        .assert()
        .success();

    let cell = matrix_dir(tmp.path(), "my-app").join("approaches/02-event-sourcing/01-impact");
    fs::write(cell.join("fact.md"), "Full audit trail by construction.").unwrap();

    wai_cmd(tmp.path())
        .args(["matrix", "decide", "02-event-sourcing", "because"])
        .assert()
        .success();

    let designs = tmp.path().join(".wai/projects/my-app/designs");
    let entries: Vec<_> = fs::read_dir(&designs).unwrap().collect();
    let body = entries
        .iter()
        .map(|e| fs::read_to_string(e.as_ref().unwrap().path()).unwrap())
        .find(|s| s.contains("# Design"))
        .unwrap();
    assert!(
        body.contains("Full audit trail by construction."),
        "design doc must snapshot the winning column's facts"
    );
}

// ── 5.1–5.11 lint ─────────────────────────────────────────────────────────

fn setup_matrix() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    let m = matrix_dir(tmp.path(), "my-app");
    (tmp, m)
}

fn lint_cmd(dir: &Path) -> Command {
    let mut cmd = wai_cmd(dir);
    cmd.args(["matrix", "lint"]);
    cmd
}

#[test]
fn lint_valid_matrix_passes() {
    let (tmp, m) = setup_matrix();
    fill_cell(&m, "01-status-quo", "x", "fact", Some("red"));
    lint_cmd(tmp.path()).assert().success();
}

// 5.1 rectangularity

#[test]
fn lint_missing_cell_fails_naming_path() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "impact"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "es"])
        .assert()
        .success();
    // remove the cell dir from one approach → non-rectangular
    fs::remove_dir_all(m.join("approaches/02-es/01-impact")).unwrap();

    lint_cmd(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("02-es/01-impact"));
}

// 5.2 markers

#[test]
fn lint_multiple_markers_fails_naming_cell() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    let cell = m.join("approaches/01-status-quo/01-c");
    fs::write(cell.join("fact.md"), "fact").unwrap();
    fs::write(cell.join("green"), "").unwrap();
    fs::write(cell.join("red"), "").unwrap();

    lint_cmd(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("01-c"));
}

#[test]
fn lint_unknown_marker_fails() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    let cell = m.join("approaches/01-status-quo/01-c");
    fs::write(cell.join("fact.md"), "fact").unwrap();
    fs::write(cell.join("chartreuse"), "").unwrap();

    lint_cmd(tmp.path()).assert().failure();
}

// 5.3 empty/missing fact

#[test]
fn lint_empty_fact_md_fails() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fs::write(m.join("approaches/01-status-quo/01-c/fact.md"), "   \n").unwrap();

    lint_cmd(tmp.path()).assert().failure();
}

#[test]
fn lint_marker_without_fact_reports_incomplete() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fs::write(m.join("approaches/01-status-quo/01-c/red"), "").unwrap();

    lint_cmd(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing or empty fact.md"));
}

// 5.4 status quo first

#[test]
fn lint_missing_status_quo_fails() {
    let (tmp, m) = setup_matrix();
    fs::remove_dir_all(m.join("approaches/01-status-quo")).unwrap();
    lint_cmd(tmp.path()).assert().failure();
}

// 5.5 all-green column

#[test]
fn lint_all_green_column_warns_but_passes() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "es"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fill_cell(&m, "02-es", "01-c", "Perfect in every way.", Some("green"));

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("02-es"));
}

// 5.6 undistinguished columns

#[test]
fn lint_undistinguished_columns_warn() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "es"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fill_cell(&m, "01-status-quo", "01-c", "Same text.", Some("neutral"));
    fill_cell(&m, "02-es", "01-c", "Same text.", Some("neutral"));

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("indistinguish"));
}

// 5.7/5.8/5.9 text lints

#[test]
fn lint_judgment_words_in_fact_warn() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fill_cell(
        &m,
        "01-status-quo",
        "01-c",
        "This is good and better than before.",
        Some("neutral"),
    );

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("judgment"));
}

#[test]
fn lint_link_only_cell_warns() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fill_cell(
        &m,
        "01-status-quo",
        "01-c",
        "https://example.com/docs/very-long-path",
        Some("neutral"),
    );

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("link"));
}

#[test]
fn lint_criterion_phrased_as_question_warns() {
    let (tmp, _m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fs::write(
        matrix_dir(tmp.path(), "my-app").join("criteria/01-c.md"),
        "# c\n\nHow cheap is it?\n",
    )
    .unwrap();

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("question"));
}

// status-quo without red

#[test]
fn lint_status_quo_without_red_warns() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fill_cell(
        &m,
        "01-status-quo",
        "01-c",
        "Works fine, nothing wrong.",
        Some("green"),
    );

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("status quo"));
}

// decided with unfilled cells

#[test]
fn lint_decided_with_unfilled_cells_warns() {
    let (tmp, _m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "es"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "decide", "02-es", "because"])
        .assert()
        .success();

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("unfilled"));
}

// 5.10 empty problem

#[test]
fn lint_empty_problem_md_warns() {
    let (tmp, m) = setup_matrix();
    fs::write(m.join("problem.md"), "# Problem\n\n").unwrap();
    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("problem"));
}

// 5.11 stale decision

#[test]
fn lint_stale_decision_warns() {
    let (tmp, m) = setup_matrix();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    fill_cell(&m, "01-status-quo", "01-c", "fact", Some("red"));
    wai_cmd(tmp.path())
        .args(["matrix", "decide", "01-status-quo", "keep"])
        .assert()
        .success();
    // simulate an edit after the decision, with a decision timestamp in the past
    fill_cell(&m, "01-status-quo", "01-c", "edited fact", Some("red"));
    let decision = fs::read_to_string(m.join("decision.md")).unwrap();
    let rewritten: String = decision
        .lines()
        .map(|l| {
            if l.starts_with("Decided at:") {
                "Decided at: 2020-01-01T00:00:00Z".to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(m.join("decision.md"), rewritten).unwrap();

    lint_cmd(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("newer than the decision"));
}

// ── 4.1–4.6 render ──────────────────────────────────────────────────────────

fn fill_cell(m: &Path, approach: &str, criterion: &str, fact: &str, marker: Option<&str>) {
    let cell = m.join("approaches").join(approach).join(criterion);
    fs::create_dir_all(&cell).unwrap();
    fs::write(cell.join("fact.md"), fact).unwrap();
    if let Some(marker) = marker {
        fs::write(cell.join(marker), "").unwrap();
    }
}

#[test]
fn matrix_render_produces_banner_rows_columns_and_key() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "Which storage engine?"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "impact"])
        .assert()
        .success();
    let m = matrix_dir(tmp.path(), "my-app");
    fill_cell(
        &m,
        "01-status-quo",
        "01-impact",
        "Manual audit scripts.",
        Some("red"),
    );
    fill_cell(
        &m,
        "02-event-sourcing",
        "01-impact",
        "Audit trail by construction.",
        Some("green"),
    );

    wai_cmd(tmp.path())
        .args(["matrix", "render"])
        .assert()
        .success();

    let html = fs::read_to_string(m.join("matrix.html")).unwrap();
    assert!(html.contains("Which storage engine?"), "problem banner");
    assert!(html.contains("01-impact"), "criterion row");
    assert!(html.contains("01-status-quo"), "approach column");
    assert!(html.contains("02-event-sourcing"), "approach column");
    assert!(html.contains("Manual audit scripts."), "cell fact");
    assert!(html.contains("red"), "judgment chip");
    assert!(html.contains("Assessment key"), "assessment key baked in");
}

#[test]
fn matrix_render_is_deterministic_byte_identical() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "a"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    let m = matrix_dir(tmp.path(), "my-app");
    fill_cell(&m, "01-status-quo", "01-c", "fact", Some("yellow"));

    wai_cmd(tmp.path())
        .args(["matrix", "render"])
        .assert()
        .success();
    let first = fs::read(m.join("matrix.html")).unwrap();
    wai_cmd(tmp.path())
        .args(["matrix", "render"])
        .assert()
        .success();
    let second = fs::read(m.join("matrix.html")).unwrap();
    assert_eq!(first, second, "same state must produce byte-identical HTML");
}

#[test]
fn matrix_render_empty_cell_is_not_a_judgment_color() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "a"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();

    wai_cmd(tmp.path())
        .args(["matrix", "render"])
        .assert()
        .success();

    let html = fs::read_to_string(matrix_dir(tmp.path(), "my-app").join("matrix.html")).unwrap();
    assert!(
        html.contains("not yet assessed"),
        "empty cell must show the placeholder"
    );
}

#[test]
fn matrix_render_neutral_is_a_judgment_not_incomplete() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "a"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "c"])
        .assert()
        .success();
    let m = matrix_dir(tmp.path(), "my-app");
    fill_cell(
        &m,
        "01-status-quo",
        "01-c",
        "Clear, nothing special.",
        Some("neutral"),
    );

    wai_cmd(tmp.path())
        .args(["matrix", "render"])
        .assert()
        .success();

    let html = fs::read_to_string(m.join("matrix.html")).unwrap();
    assert!(html.contains("Clear, nothing special."));
    assert!(html.contains("neutral"));
}

// ── numbering ─────────────────────────────────────────────────────────────────

#[test]
fn matrix_add_commands_number_sequentially() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    create_project(tmp.path(), "my-app");
    wai_cmd(tmp.path())
        .args(["matrix", "init", "problem"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "impact"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "criterion", "add", "risk"])
        .assert()
        .success();

    let m = matrix_dir(tmp.path(), "my-app");
    assert!(m.join("criteria/01-impact.md").exists());
    assert!(m.join("criteria/02-risk.md").exists());

    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "event-sourcing"])
        .assert()
        .success();
    wai_cmd(tmp.path())
        .args(["matrix", "approach", "add", "cqrs"])
        .assert()
        .success();
    assert!(m.join("approaches/02-event-sourcing").is_dir());
    assert!(m.join("approaches/03-cqrs").is_dir());
}
