use std::path::Path;

use crate::config::{self, STATE_FILE};
use crate::json::Suggestion;
use crate::state::{Phase, ProjectState};

/// Known workflow patterns that wai can detect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowPattern {
    /// Fresh project with no artifacts yet.
    NewProject,
    /// In research phase with few or no research artifacts.
    ResearchPhaseMinimal,
    /// In plan/design phase with designs ready — can move to implement.
    ReadyToImplement,
    /// Actively in implement phase.
    ImplementPhaseActive,
}

/// A detected workflow pattern with a human message and suggested next steps.
#[derive(Debug)]
pub struct WorkflowDetection {
    pub pattern: WorkflowPattern,
    pub message: String,
    pub suggestions: Vec<Suggestion>,
}

/// Snapshot of a project's state and artifact counts.
#[derive(Debug)]
pub struct ProjectContext {
    pub name: String,
    pub phase: Phase,
    pub research_count: usize,
    pub plan_count: usize,
    pub design_count: usize,
    pub handoff_count: usize,
}

/// Scan a project directory and build a context snapshot.
///
/// Returns `None` if the project directory or state file can't be read.
pub fn scan_project(project_root: &Path, project_name: &str) -> Option<ProjectContext> {
    let project_dir = config::project_path(project_root, project_name);
    if !project_dir.is_dir() {
        return None;
    }

    let state_path = project_dir.join(STATE_FILE);
    let phase = ProjectState::load(&state_path).ok()?.current;

    let research_count = count_artifacts(&project_dir.join(config::RESEARCH_DIR));
    let plan_count = count_artifacts(&project_dir.join(config::PLANS_DIR));
    let design_count = count_artifacts(&project_dir.join(config::DESIGNS_DIR));
    let handoff_count = count_artifacts(&project_dir.join(config::HANDOFFS_DIR));

    Some(ProjectContext {
        name: project_name.to_string(),
        phase,
        research_count,
        plan_count,
        design_count,
        handoff_count,
    })
}

/// Detect workflow patterns from a project context.
pub fn detect_patterns(ctx: &ProjectContext) -> Vec<WorkflowDetection> {
    let mut detections = Vec::new();

    let total_artifacts = ctx.research_count + ctx.plan_count + ctx.design_count;

    // New project: no artifacts at all, still in research phase
    if total_artifacts == 0 && ctx.phase == Phase::Research {
        detections.push(WorkflowDetection {
            pattern: WorkflowPattern::NewProject,
            message: "New project — start by adding research".to_string(),
            suggestions: vec![
                Suggestion {
                    label: "Add research".to_string(),
                    command: "wai add research \"...\"".to_string(),
                },
                Suggestion {
                    label: "Check project phase".to_string(),
                    command: "wai phase".to_string(),
                },
            ],
        });
        return detections;
    }

    // Research phase with minimal research (0-1 artifacts)
    if ctx.phase == Phase::Research && ctx.research_count <= 1 {
        detections.push(WorkflowDetection {
            pattern: WorkflowPattern::ResearchPhaseMinimal,
            message: "Research phase — add more research before advancing".to_string(),
            suggestions: vec![
                Suggestion {
                    label: "Add research".to_string(),
                    command: "wai add research \"...\"".to_string(),
                },
                Suggestion {
                    label: "Search existing artifacts".to_string(),
                    command: "wai search \"...\"".to_string(),
                },
            ],
        });
    }

    // Ready to implement: in plan or design phase with at least one design
    if matches!(ctx.phase, Phase::Plan | Phase::Design) && ctx.design_count > 0 {
        detections.push(WorkflowDetection {
            pattern: WorkflowPattern::ReadyToImplement,
            message: "Ready to implement — designs are in place".to_string(),
            suggestions: vec![
                Suggestion {
                    label: "Advance to implement phase".to_string(),
                    command: "wai phase set implement".to_string(),
                },
                Suggestion {
                    label: "Review designs".to_string(),
                    command: "wai search \"design\"".to_string(),
                },
            ],
        });
    }

    // Actively implementing
    if ctx.phase == Phase::Implement {
        detections.push(WorkflowDetection {
            pattern: WorkflowPattern::ImplementPhaseActive,
            message: "Implementation in progress".to_string(),
            suggestions: vec![
                Suggestion {
                    label: "Show project details".to_string(),
                    command: "wai show".to_string(),
                },
                Suggestion {
                    label: "Add implementation notes".to_string(),
                    command: "wai add plan \"...\"".to_string(),
                },
                Suggestion {
                    label: "Check status".to_string(),
                    command: "wai status".to_string(),
                },
            ],
        });
    }

    detections
}

/// Count files in a directory (non-recursive, files only).
fn count_artifacts(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_detected_when_no_artifacts() {
        let ctx = ProjectContext {
            name: "test".to_string(),
            phase: Phase::Research,
            research_count: 0,
            plan_count: 0,
            design_count: 0,
            handoff_count: 0,
        };
        let detections = detect_patterns(&ctx);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].pattern, WorkflowPattern::NewProject);
    }

    #[test]
    fn minimal_research_detected() {
        let ctx = ProjectContext {
            name: "test".to_string(),
            phase: Phase::Research,
            research_count: 1,
            plan_count: 0,
            design_count: 0,
            handoff_count: 0,
        };
        let detections = detect_patterns(&ctx);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].pattern, WorkflowPattern::ResearchPhaseMinimal);
    }

    #[test]
    fn ready_to_implement_in_design_phase() {
        let ctx = ProjectContext {
            name: "test".to_string(),
            phase: Phase::Design,
            research_count: 3,
            plan_count: 1,
            design_count: 2,
            handoff_count: 0,
        };
        let detections = detect_patterns(&ctx);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].pattern, WorkflowPattern::ReadyToImplement);
    }

    #[test]
    fn ready_to_implement_in_plan_phase() {
        let ctx = ProjectContext {
            name: "test".to_string(),
            phase: Phase::Plan,
            research_count: 2,
            plan_count: 1,
            design_count: 1,
            handoff_count: 0,
        };
        let detections = detect_patterns(&ctx);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].pattern, WorkflowPattern::ReadyToImplement);
    }

    #[test]
    fn implement_phase_active() {
        let ctx = ProjectContext {
            name: "test".to_string(),
            phase: Phase::Implement,
            research_count: 3,
            plan_count: 2,
            design_count: 1,
            handoff_count: 0,
        };
        let detections = detect_patterns(&ctx);
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].pattern, WorkflowPattern::ImplementPhaseActive);
    }

    #[test]
    fn no_patterns_for_review_phase() {
        let ctx = ProjectContext {
            name: "test".to_string(),
            phase: Phase::Review,
            research_count: 5,
            plan_count: 2,
            design_count: 2,
            handoff_count: 1,
        };
        let detections = detect_patterns(&ctx);
        assert!(detections.is_empty());
    }

    #[test]
    fn research_phase_with_plenty_of_research() {
        let ctx = ProjectContext {
            name: "test".to_string(),
            phase: Phase::Research,
            research_count: 5,
            plan_count: 0,
            design_count: 0,
            handoff_count: 0,
        };
        let detections = detect_patterns(&ctx);
        assert!(detections.is_empty());
    }
}
