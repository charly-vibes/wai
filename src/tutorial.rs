use crate::config::{UserConfig, mark_tutorial_seen};
use cliclack::{intro, log, outro};
use miette::{IntoDiagnostic, Result};
use std::thread;
use std::time::Duration;

/// Run the interactive quickstart tutorial
pub fn run() -> Result<()> {
    let user_config = UserConfig::load().unwrap_or_default();
    let is_first_run = !user_config.seen_tutorial;

    if is_first_run {
        intro("Welcome to wai! Let's get you started.").into_diagnostic()?;
        log::info("This tutorial will show you the core wai workflow.").into_diagnostic()?;
    } else {
        intro("wai Tutorial - Quickstart Guide").into_diagnostic()?;
    }

    // Step 1: What is wai?
    tutorial_step(
        "1/5",
        "What is wai?",
        "wai helps you track the *why* behind your work.",
        "It captures research, design decisions, and plans alongside your code.",
    )?;

    // Step 2: PARA Structure
    tutorial_step(
        "2/5",
        "The PARA Method",
        "wai organizes work into four categories:",
        "• Projects - Active work with a deadline\n\
         • Areas - Ongoing responsibilities\n\
         • Resources - Reference materials\n\
         • Archives - Completed work",
    )?;

    // Step 3: Project Phases
    tutorial_step(
        "3/5",
        "Project Phases",
        "Each project moves through phases:",
        "1. research → Gather information\n\
         2. design → Make architectural decisions\n\
         3. plan → Break into tasks\n\
         4. implement → Write code\n\
         5. review → Validate against plans\n\
         6. archive → Wrap up",
    )?;

    // Step 4: Core Commands
    tutorial_step(
        "4/5",
        "Core Commands",
        "Here are the essential commands:",
        "• wai init              Initialize wai in current directory\n\
         • wai new project       Create a new project\n\
         • wai status            Check project status\n\
         • wai add research      Capture research notes\n\
         • wai phase next        Advance to next phase",
    )?;

    // Step 5: Handoffs
    tutorial_step(
        "5/5",
        "Session Handoffs",
        "Before ending a session, create a handoff:",
        "• wai handoff create <project>\n\n\
         This captures what you did, decisions made, and next steps.\n\
         The next developer (or future you) can pick up where you left off.",
    )?;

    // Mark tutorial as seen
    if let Err(e) = mark_tutorial_seen() {
        eprintln!("Warning: Could not save tutorial progress: {}", e);
    }

    // Summary
    log::success("Tutorial complete!").into_diagnostic()?;
    println!();
    log::info("Next steps:").into_diagnostic()?;
    println!("  • Run 'wai init' to set up your first project");
    println!("  • Run 'wai status' anytime to see your project state");
    println!("  • Run 'wai tutorial' anytime to replay this tutorial");

    if is_first_run {
        outro("You're ready to start using wai!").into_diagnostic()?;
    } else {
        outro("Tutorial replay complete").into_diagnostic()?;
    }

    Ok(())
}

/// Display a tutorial step with consistent formatting
fn tutorial_step(step: &str, title: &str, description: &str, details: &str) -> Result<()> {
    println!();
    log::step(format!("{} {}", step, title)).into_diagnostic()?;
    log::info(description).into_diagnostic()?;
    println!();
    println!("{}", details);

    // Small pause for readability
    thread::sleep(Duration::from_millis(100));

    Ok(())
}

