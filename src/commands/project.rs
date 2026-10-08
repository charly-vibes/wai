use miette::Result;
use owo_colors::OwoColorize;

use crate::cli::ProjectCommands;
use crate::config::{STATE_FILE, projects_dir};
use crate::state::ProjectState;
use genesis::suggestions::SuggestionEngine;

use super::{list_projects, require_project};

pub fn run(cmd: ProjectCommands) -> Result<()> {
    // Handle wrong-order detection first (doesn't require a workspace)
    if let ProjectCommands::External(ref args) = cmd {
        let patterns_owned = crate::cli::wai_subcommand_patterns();
        let valid_patterns: Vec<(&str, &str)> = patterns_owned
            .iter()
            .map(|(v, n)| (v.as_str(), n.as_str()))
            .collect();
        let engine = SuggestionEngine::new();
        let sub = args.first().map(|s| s.as_str()).unwrap_or("");

        if let Some(suggestion) = engine.suggest_order("project", sub, &valid_patterns) {
            miette::bail!(
                "{}. {}",
                suggestion.message(),
                "Run 'wai --help' to see available commands."
            );
        }
        miette::bail!(
            "Unknown subcommand 'project {}'. Run 'wai project --help' for available commands.",
            args.join(" ")
        );
    }

    let project_root = require_project()?;

    match cmd {
        ProjectCommands::Use { name, shell } => use_project(&project_root, name, shell),
        ProjectCommands::External(_) => unreachable!(),
    }
}

/// `wai project use [name]`: bind a project or list available ones.
fn use_project(
    project_root: &std::path::Path,
    name: Option<String>,
    shell: Option<String>,
) -> Result<()> {
    let mut projects = list_projects(project_root);
    projects.sort();

    if let Some(name) = name {
        use_named_project(project_root, &name, shell, &projects)
    } else {
        list_available_projects(project_root, &projects)
    }
}

/// Validate and bind a named project, printing the export line.
fn use_named_project(
    project_root: &std::path::Path,
    name: &str,
    shell: Option<String>,
    projects: &[String],
) -> Result<()> {
    let proj_dir = projects_dir(project_root).join(name);
    if !proj_dir.exists() {
        let available = if projects.is_empty() {
            "none".to_string()
        } else {
            projects.join(", ")
        };
        miette::bail!(
            "Project '{}' not found. Available projects: {}",
            name,
            available
        );
    }

    // Shell for the export line: --shell override beats SHELL
    // detection; without the flag detection is only a heuristic
    // (see detect_shell).
    let syntax = match shell.as_deref() {
        Some(s) => resolve_shell(s)?,
        None => detect_shell(),
    };
    println!("{}", syntax.export_line(name));

    // Print hint to stderr when stdout is a terminal
    if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        eprintln!(
            "{}",
            format!(
                "# Paste the line above, or run: eval $(wai project use {})",
                name
            )
            .dimmed()
        );
    }
    Ok(())
}

/// No args: list available projects with phases.
fn list_available_projects(project_root: &std::path::Path, projects: &[String]) -> Result<()> {
    if projects.is_empty() {
        println!("No projects. Create one with `wai new project <name>`.");
        return Ok(());
    }

    println!();
    for name in projects {
        let state_path = projects_dir(project_root).join(name).join(STATE_FILE);
        let phase = ProjectState::load(&state_path)
            .map(|s| s.current.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        println!("  {} {}  [{}]", "•".dimmed(), name.bold(), phase.dimmed());
    }
    println!();
    println!("  {} Set project: wai project use <name>", "→".dimmed());
    println!();
    Ok(())
}

/// Export statement syntax for the shell that will consume it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellSyntax {
    /// POSIX sh / bash / zsh compatible `export`
    Posix,
    /// fish `set -gx`
    Fish,
}

impl ShellSyntax {
    fn export_line(&self, name: &str) -> String {
        match self {
            ShellSyntax::Posix => format!("export WAI_PROJECT='{}'", name),
            ShellSyntax::Fish => format!("set -gx WAI_PROJECT '{}'", name),
        }
    }
}

/// Resolve an explicit `--shell` value.
fn resolve_shell(s: &str) -> Result<ShellSyntax> {
    match s.to_ascii_lowercase().as_str() {
        "posix" | "bash" | "zsh" | "sh" => Ok(ShellSyntax::Posix),
        "fish" => Ok(ShellSyntax::Fish),
        _ => miette::bail!(
            "Unknown shell '{}'. Supported values: posix (also bash, zsh, sh), fish.",
            s
        ),
    }
}

/// Detect the export syntax from the environment.
///
/// Heuristic only: `$SHELL` is the login shell, not necessarily the running
/// shell, so the explicit `--shell` override exists (wai-sib1). `FISH_VERSION`
/// is honored first because fish sets it in every fish session, including
/// non-interactive eval contexts.
fn detect_shell() -> ShellSyntax {
    if std::env::var_os("FISH_VERSION").is_some() {
        return ShellSyntax::Fish;
    }
    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.ends_with("/fish") || shell.ends_with("\\fish") {
        ShellSyntax::Fish
    } else {
        ShellSyntax::Posix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_shell_accepts_posix_and_fish() {
        assert_eq!(resolve_shell("posix").unwrap(), ShellSyntax::Posix);
        assert_eq!(resolve_shell("fish").unwrap(), ShellSyntax::Fish);
        assert_eq!(resolve_shell("bash").unwrap(), ShellSyntax::Posix);
        assert_eq!(resolve_shell("FISH").unwrap(), ShellSyntax::Fish);
    }

    #[test]
    fn resolve_shell_rejects_unknown_values() {
        assert!(resolve_shell("nushell").is_err());
        assert!(resolve_shell("").is_err());
    }

    #[test]
    fn export_lines_use_the_documented_syntax() {
        assert_eq!(
            ShellSyntax::Posix.export_line("myproj"),
            "export WAI_PROJECT='myproj'"
        );
        assert_eq!(
            ShellSyntax::Fish.export_line("myproj"),
            "set -gx WAI_PROJECT 'myproj'"
        );
    }
}
