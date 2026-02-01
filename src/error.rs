#![allow(dead_code)]

use miette::Diagnostic;
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

    #[error("Bead '{id}' not found")]
    #[diagnostic(
        code(wai::bead::not_found),
        help("Run `wai show beads` to see available beads")
    )]
    BeadNotFound { id: String },

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

    #[error("Plugin '{name}' not found")]
    #[diagnostic(
        code(wai::plugin::not_found),
        help("Run `wai show plugins --available` to see installable plugins")
    )]
    PluginNotFound { name: String },

    #[error("Configuration error: {message}")]
    #[diagnostic(code(wai::config::invalid))]
    ConfigError { message: String },

    #[error("IO error: {0}")]
    #[diagnostic(code(wai::io::error))]
    Io(#[from] std::io::Error),
}
