//! `wai matrix` command group — CLI dispatch for the decision matrix.
//!
//! Thin layer over [`crate::matrix`]: resolves the project, maps
//! subcommands to core operations, prints results. The filesystem is the
//! source of truth — these commands exist only where one logical action
//! spans multiple directories.

use cliclack::log;
use miette::{IntoDiagnostic, Result};

use crate::cli::MatrixCommands;
use crate::context::{current_context, require_safe_mode};
use crate::matrix;

use super::{print_suggestions, require_project, resolve_project};
use crate::json::Suggestion;

pub fn run(cmd: MatrixCommands) -> Result<()> {
    let project_root = require_project()?;
    let resolved = resolve_project(&project_root, cmd.project())?;
    let dir = matrix::matrix_dir(&project_root, &resolved.name);

    match cmd {
        MatrixCommands::Init { problem, .. } => {
            require_safe_mode("initialize decision matrix")?;
            matrix::init(&dir, &problem)?;
            if !current_context().quiet {
                log::success(format!(
                    "Decision matrix initialized in '{}'",
                    dir.display()
                ))
                .into_diagnostic()?;
                print_suggestions(&[
                    Suggestion {
                        label: "Add an approach".to_string(),
                        command: "wai matrix approach add <name>".to_string(),
                    },
                    Suggestion {
                        label: "Add a criterion".to_string(),
                        command: "wai matrix criterion add <name>".to_string(),
                    },
                ]);
            }
            Ok(())
        }
        MatrixCommands::Criterion(crate::cli::MatrixCriterionCommands::Add { name, .. }) => {
            require_safe_mode("add matrix criterion")?;
            let path = matrix::add_criterion(&dir, &name)?;
            if !current_context().quiet {
                log::success(format!(
                    "Criterion '{}' added — cell directories created in every approach",
                    path.display()
                ))
                .into_diagnostic()?;
            }
            Ok(())
        }
        MatrixCommands::Approach(crate::cli::MatrixApproachCommands::Add { name, .. }) => {
            require_safe_mode("add matrix approach")?;
            let path = matrix::add_approach(&dir, &name)?;
            if !current_context().quiet {
                log::success(format!("Approach '{}' added", path.display())).into_diagnostic()?;
            }
            Ok(())
        }
        MatrixCommands::Decide {
            approach,
            rationale,
            ..
        } => {
            require_safe_mode("record matrix decision")?;
            let (decision_path, doc_path) =
                matrix::decide(&dir, &approach, &rationale, &resolved.name)?;
            if !current_context().quiet {
                log::success(format!(
                    "Decision recorded: {} — design doc scaffolded at {}",
                    decision_path.display(),
                    doc_path.display()
                ))
                .into_diagnostic()?;
            }
            Ok(())
        }
        MatrixCommands::Render { .. } => {
            let path = matrix::render(&dir)?;
            if !current_context().quiet {
                log::success(format!("Rendered matrix to {}", path.display())).into_diagnostic()?;
            }
            Ok(())
        }
        MatrixCommands::Lint { .. } => {
            let report = matrix::lint(&dir)?;
            for warning in &report.warnings {
                log::warning(warning).into_diagnostic()?;
            }
            if !current_context().quiet && report.errors.is_empty() && report.warnings.is_empty() {
                log::success("Matrix lint: all clear").into_diagnostic()?;
            }
            if report.errors.is_empty() {
                Ok(())
            } else {
                miette::bail!(
                    "Matrix lint failed with {} structural error(s):\n  {}",
                    report.errors.len(),
                    report.errors.join("\n  ")
                );
            }
        }
    }
}
