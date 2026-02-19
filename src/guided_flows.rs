use cliclack::{confirm, input, log, outro};
use miette::{IntoDiagnostic, Result};

use crate::config::mark_tutorial_seen;

/// Show a guided walkthrough for adding the first research artifact
pub fn first_research_walkthrough() -> Result<()> {
    println!();
    log::step("First Research Artifact").into_diagnostic()?;
    log::info("Let's capture your first research note.").into_diagnostic()?;
    println!();

    let proceed = confirm("Would you like to add research now?")
        .interact()
        .into_diagnostic()
        .unwrap_or(false);

    if !proceed {
        log::info("You can add research anytime with: wai add research").into_diagnostic()?;
        return Ok(());
    }

    let topic: String = input("What are you researching?")
        .placeholder("e.g., authentication approaches, database options")
        .interact()
        .into_diagnostic()?;

    let content: String = input("What did you learn?")
        .placeholder("Key findings, options considered, trade-offs...")
        .interact()
        .into_diagnostic()?;

    println!();
    log::success("Research note captured!").into_diagnostic()?;
    log::info(&format!(
        "Research saved. Next steps:\n  • wai search \"{}\" - Find your notes\n  • wai add plan - Break into a plan",
        topic
    ))
    .into_diagnostic()?;

    Ok(())
}

/// Show a guided walkthrough for the first phase transition
pub fn first_phase_walkthrough() -> Result<()> {
    println!();
    log::step("Project Phases").into_diagnostic()?;
    log::info("Each project moves through phases as work progresses.").into_diagnostic()?;
    println!();
    println!("  1. research → Gather information");
    println!("  2. design → Make architectural decisions");
    println!("  3. plan → Break into tasks");
    println!("  4. implement → Write code");
    println!("  5. review → Validate against plans");
    println!("  6. archive → Wrap up");
    println!();

    let proceed = confirm("Ready to advance to the next phase?")
        .interact()
        .into_diagnostic()
        .unwrap_or(false);

    if proceed {
        log::success("Use 'wai phase next' to advance when ready.").into_diagnostic()?;
    } else {
        log::info("You can advance phases anytime with: wai phase next").into_diagnostic()?;
    }

    Ok(())
}

/// Show enhanced guidance text during init
pub fn enhanced_init_guidance(project_name: &str) -> Result<()> {
    println!();
    log::success(&format!("Initialized project: {}", project_name)).into_diagnostic()?;
    println!();
    log::info("You're all set! Here's what you can do next:").into_diagnostic()?;
    println!();
    println!("  💡 Capture your reasoning:");
    println!("     wai add research \"what I learned about this project\"");
    println!("     wai add design \"architecture decisions\"");
    println!("     wai add plan \"implementation approach\"");
    println!();
    println!("  📊 Track progress:");
    println!("     wai status           See project info");
    println!("     wai phase next       Advance to next phase");
    println!("     wai timeline <name>  View artifact history");
    println!();
    println!("  🤝 Hand off context:");
    println!("     wai handoff create <name>   Save session progress");
    println!();

    Ok(())
}
