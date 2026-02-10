#![allow(dead_code)]

use miette::Diagnostic;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum WaiError {
    #[error("No project initialized in current directory")]
    #[diagnostic(
        code(wai::project::not_initialized),
        help("Run `wai init` or `wai new project <name>` first")
    )]
    NotInitialized,

    #[error("Project already exists at {path}")]
    #[diagnostic(
        code(wai::project::already_exists),
        help("Use a different directory or run `wai init` to reinitialize")
    )]
    ProjectExists { path: String },

    #[error("Project '{name}' not found")]
    #[diagnostic(
        code(wai::project::not_found),
        help("Run `wai show` to see available projects")
    )]
    ProjectNotFound { name: String },

    #[error("Area '{name}' not found")]
    #[diagnostic(
        code(wai::area::not_found),
        help("Run `wai show` to see available areas")
    )]
    AreaNotFound { name: String },

    #[error("Resource '{name}' not found")]
    #[diagnostic(
        code(wai::resource::not_found),
        help("Run `wai show` to see available resources")
    )]
    ResourceNotFound { name: String },

    #[error("Invalid phase transition from '{from}' to '{to}'")]
    #[diagnostic(
        code(wai::phase::invalid_transition),
        help("Valid transitions from '{from}': {valid_targets}")
    )]
    InvalidPhaseTransition {
        from: String,
        to: String,
        valid_targets: String,
    },

    #[error("No active project context")]
    #[diagnostic(
        code(wai::project::no_context),
        help("Run a command within a project directory or specify --project <name>")
    )]
    NoProjectContext,

    #[error("Config sync error: {message}")]
    #[diagnostic(
        code(wai::sync::error),
        help("Check `.wai/resources/agent-config/.projections.yml` configuration")
    )]
    ConfigSyncError { message: String },

    #[error("Handoff error: {message}")]
    #[diagnostic(code(wai::handoff::error), help("{suggestion}"))]
    HandoffError { message: String, suggestion: String },

    #[error("Plugin '{name}' not found")]
    #[diagnostic(
        code(wai::plugin::not_found),
        help("Run `wai plugin list` to see available plugins")
    )]
    PluginNotFound { name: String },

    #[error("Non-interactive mode: {message}")]
    #[diagnostic(
        code(wai::cli::non_interactive),
        help("Re-run without --no-input or supply required flags")
    )]
    NonInteractive { message: String },

    #[error("Safe mode prevented action: {action}")]
    #[diagnostic(
        code(wai::cli::safe_mode),
        help("Re-run without --safe to allow this action")
    )]
    SafeModeViolation { action: String },

    #[error("Configuration error: {message}")]
    #[diagnostic(code(wai::config::invalid))]
    ConfigError { message: String },

    #[error("IO error: {0}")]
    #[diagnostic(code(wai::io::error))]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    #[diagnostic(code(wai::yaml::error))]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    pub help: Option<String>,
    pub details: Option<String>,
}

impl WaiError {
    pub fn as_payload(&self) -> ErrorPayload {
        let diagnostic = self as &dyn Diagnostic;
        ErrorPayload {
            code: diagnostic
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "wai::error::unknown".to_string()),
            message: self.to_string(),
            help: diagnostic.help().map(|h| h.to_string()),
            details: None,
        }
    }
}
