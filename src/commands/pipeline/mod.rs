//! Pipeline command module: TOML-defined multi-step agent workflows.

use miette::Result;

use crate::cli::PipelineCommands;
use crate::context::current_context;

mod definition;
mod gates;
pub(crate) mod orchestration;
mod model;
mod queries;
mod setup;
mod verify;

#[cfg(test)]
mod tests_coverage_oracle;
#[cfg(test)]
mod tests_definition;
#[cfg(test)]
mod tests_gates;
#[cfg(test)]
mod tests_locks;

// Re-export public items that other modules reference
pub use definition::load_pipeline_toml;
pub use model::{
    ApprovalGate, ArtifactLock, BypassEntry, OracleGate, PipelineDefinition, PipelineMetadataSection,
    PipelineRun, PipelineStep, ProceduralGate, StructuralGate, StepGate, ValidationIssue,
    ValidationLevel,
};
pub(crate) use orchestration::{
    clear_complete_pipeline_run, find_stale_runs, pipeline_current_status, run_is_incomplete,
    stale_threshold_days,
};
pub use verify::{artifact_hash, read_artifact_lock, render_prompt, write_artifact_lock};

pub fn run(cmd: PipelineCommands) -> Result<()> {
    match cmd {
        PipelineCommands::Status => cmd_status(),
        PipelineCommands::List => queries::cmd_list(),
        PipelineCommands::Init { name } => setup::cmd_init(&name),
        PipelineCommands::Start { name, topic, epic } => {
            orchestration::cmd_start(&name, topic.as_deref(), epic.as_deref())
        }
        PipelineCommands::Next => orchestration::cmd_next(),
        PipelineCommands::Current { json } => queries::cmd_current(json),
        PipelineCommands::Suggest { description } => queries::cmd_suggest(description.as_deref()),
        PipelineCommands::Approve => orchestration::cmd_approve(),
        PipelineCommands::Gc { yes } => orchestration::cmd_gc(yes),
        PipelineCommands::Preflight => orchestration::cmd_preflight(),
        PipelineCommands::Wait { pr, timeout } => orchestration::cmd_wait(&pr, timeout),
        PipelineCommands::Spawn { name, prompt, budget } => {
            orchestration::cmd_spawn(&name, &prompt, budget)
        }
        PipelineCommands::SpawnStatus { name } => orchestration::cmd_spawn_status(name.as_deref()),
        PipelineCommands::SpawnResume { name, followup } => {
            orchestration::cmd_spawn_resume(&name, followup.as_deref())
        }
        PipelineCommands::Show { name } => queries::cmd_show(&name),
        PipelineCommands::Gates { name, step } => {
            queries::cmd_gates(name.as_deref(), step.as_deref())
        }
        PipelineCommands::Check { oracle } => queries::cmd_check(oracle.as_deref()),
        PipelineCommands::Validate { name } => queries::cmd_validate(name.as_deref()),
        PipelineCommands::Bypass { reason } => orchestration::cmd_bypass(&reason),
        PipelineCommands::Lock => orchestration::cmd_lock(),
        PipelineCommands::Verify => verify::cmd_verify(),
    }
}

// ─── status ──────────────────────────────────────────────────────────────────

fn cmd_status() -> Result<()> {
    queries::cmd_current(current_context().json)
}
