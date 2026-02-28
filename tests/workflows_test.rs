use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;
use wai::state::Phase;
use wai::workflows::{ProjectContext, WorkflowPattern, detect_patterns, scan_project};

// ─── detect_patterns unit tests ──────────────────────────────────────────────

fn ctx(phase: Phase, research: usize, plan: usize, design: usize) -> ProjectContext {
    ProjectContext {
        name: "test".to_string(),
        phase,
        research_count: research,
        plan_count: plan,
        design_count: design,
        handoff_count: 0,
    }
}

#[test]
fn new_project_at_zero_artifacts_in_research() {
    let detections = detect_patterns(&ctx(Phase::Research, 0, 0, 0));
    assert_eq!(detections.len(), 1);
    assert_eq!(detections[0].pattern, WorkflowPattern::NewProject);
}

#[test]
fn new_project_suggests_add_research() {
    let detections = detect_patterns(&ctx(Phase::Research, 0, 0, 0));
    let cmds: Vec<_> = detections[0]
        .suggestions
        .iter()
        .map(|s| s.command.as_str())
        .collect();
    assert!(
        cmds.iter().any(|c| c.contains("add research")),
        "new project should suggest adding research"
    );
}

#[test]
fn minimal_research_at_one_artifact() {
    let detections = detect_patterns(&ctx(Phase::Research, 1, 0, 0));
    assert_eq!(detections.len(), 1);
    assert_eq!(detections[0].pattern, WorkflowPattern::ResearchPhaseMinimal);
}

#[test]
fn research_threshold_triggers_advance_at_three() {
    let detections = detect_patterns(&ctx(Phase::Research, 3, 0, 0));
    assert_eq!(detections.len(), 1);
    assert_eq!(
        detections[0].pattern,
        WorkflowPattern::ResearchReadyToAdvance
    );
}

#[test]
fn two_research_artifacts_triggers_advance() {
    // threshold is >= 2, so 2 artifacts should suggest advancing to design
    let detections = detect_patterns(&ctx(Phase::Research, 2, 0, 0));
    assert_eq!(detections.len(), 1);
    assert_eq!(
        detections[0].pattern,
        WorkflowPattern::ResearchReadyToAdvance,
        "2 research artifacts should trigger ResearchReadyToAdvance"
    );
}

#[test]
fn ready_to_implement_requires_design_artifact() {
    // design phase with design artifact → ReadyToImplement
    let detections = detect_patterns(&ctx(Phase::Design, 2, 0, 1));
    assert!(
        detections
            .iter()
            .any(|d| d.pattern == WorkflowPattern::ReadyToImplement),
        "should suggest ReadyToImplement when design artifact exists"
    );
}

#[test]
fn ready_to_implement_needs_design_artifact() {
    // design phase but no design artifacts → no ReadyToImplement
    let detections = detect_patterns(&ctx(Phase::Design, 2, 1, 0));
    assert!(
        !detections
            .iter()
            .any(|d| d.pattern == WorkflowPattern::ReadyToImplement),
        "should not suggest ReadyToImplement with no design artifacts"
    );
}

#[test]
fn implement_phase_active_always_detected() {
    let detections = detect_patterns(&ctx(Phase::Implement, 3, 2, 1));
    assert!(
        detections
            .iter()
            .any(|d| d.pattern == WorkflowPattern::ImplementPhaseActive),
        "ImplementPhaseActive should always be detected in implement phase"
    );
}

#[test]
fn needs_research_in_plan_with_no_research() {
    let detections = detect_patterns(&ctx(Phase::Plan, 0, 1, 0));
    assert!(
        detections
            .iter()
            .any(|d| d.pattern == WorkflowPattern::NeedsResearch),
        "NeedsResearch should fire when in plan phase with no research"
    );
}

#[test]
fn no_needs_research_when_research_present() {
    let detections = detect_patterns(&ctx(Phase::Plan, 2, 1, 0));
    assert!(
        !detections
            .iter()
            .any(|d| d.pattern == WorkflowPattern::NeedsResearch),
        "NeedsResearch should not fire when research is present"
    );
}

#[test]
fn review_phase_produces_no_patterns() {
    let detections = detect_patterns(&ctx(Phase::Review, 5, 3, 2));
    assert!(
        detections.is_empty(),
        "review phase should produce no workflow patterns"
    );
}

#[test]
fn archive_phase_produces_no_patterns() {
    let detections = detect_patterns(&ctx(Phase::Archive, 5, 3, 2));
    assert!(
        detections.is_empty(),
        "archive phase should produce no workflow patterns"
    );
}

// ─── suggestion relevance ─────────────────────────────────────────────────────

#[test]
fn research_ready_to_advance_suggests_design_phase() {
    let detections = detect_patterns(&ctx(Phase::Research, 5, 0, 0));
    let d = detections
        .iter()
        .find(|d| d.pattern == WorkflowPattern::ResearchReadyToAdvance)
        .expect("ResearchReadyToAdvance should be present");

    assert!(
        d.suggestions.iter().any(|s| s.command.contains("design")),
        "advance suggestion should reference design phase"
    );
}

#[test]
fn implement_phase_suggestions_include_status_check() {
    let detections = detect_patterns(&ctx(Phase::Implement, 2, 1, 1));
    let d = detections
        .iter()
        .find(|d| d.pattern == WorkflowPattern::ImplementPhaseActive)
        .expect("ImplementPhaseActive should be present");

    assert!(
        d.suggestions.iter().any(|s| s.command.contains("status")),
        "implement phase should suggest wai status"
    );
}

// ─── scan_project filesystem integration ─────────────────────────────────────

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd
}

fn init_and_create(dir: &std::path::Path, project: &str) {
    wai_cmd(dir)
        .args(["init", "--name", "test-ws"])
        .assert()
        .success();
    wai_cmd(dir)
        .args(["new", "project", project])
        .assert()
        .success();
}

#[test]
fn scan_project_returns_none_for_missing_project() {
    let tmp = TempDir::new().unwrap();
    let result = scan_project(tmp.path(), "nonexistent");
    assert!(
        result.is_none(),
        "scan_project should return None for missing project"
    );
}

#[test]
fn scan_project_counts_research_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_and_create(tmp.path(), "myproj");

    // Write two research artifacts directly
    let research_dir = tmp.path().join(".wai/projects/myproj/research");
    fs::write(research_dir.join("2026-01-01-first.md"), "First note").unwrap();
    fs::write(research_dir.join("2026-01-02-second.md"), "Second note").unwrap();

    let ctx = scan_project(tmp.path(), "myproj").expect("scan_project should succeed");
    assert_eq!(ctx.research_count, 2, "should count 2 research artifacts");
    assert_eq!(ctx.plan_count, 0);
    assert_eq!(ctx.design_count, 0);
}

#[test]
fn scan_project_detects_correct_phase() {
    let tmp = TempDir::new().unwrap();
    init_and_create(tmp.path(), "phased");

    // Advance to design phase via CLI
    wai_cmd(tmp.path())
        .args(["phase", "set", "design"])
        .assert()
        .success();

    let ctx = scan_project(tmp.path(), "phased").expect("scan_project should succeed");
    assert_eq!(ctx.phase, Phase::Design);
}

#[test]
fn scan_then_detect_gives_correct_workflow_pattern() {
    let tmp = TempDir::new().unwrap();
    init_and_create(tmp.path(), "workflow-proj");

    // Add 3 research artifacts to trigger ResearchReadyToAdvance
    for i in 1..=3 {
        let research_dir = tmp.path().join(".wai/projects/workflow-proj/research");
        fs::write(
            research_dir.join(format!("2026-01-0{}-note.md", i)),
            format!("Research note {}", i),
        )
        .unwrap();
    }

    let ctx = scan_project(tmp.path(), "workflow-proj").expect("scan_project should succeed");
    let detections = detect_patterns(&ctx);

    assert!(
        detections
            .iter()
            .any(|d| d.pattern == WorkflowPattern::ResearchReadyToAdvance),
        "should detect ResearchReadyToAdvance after 3 research artifacts"
    );
}
