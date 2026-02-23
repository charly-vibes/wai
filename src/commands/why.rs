use miette::Result;

use super::require_project;

/// Entry point for `wai why <query> [--no-llm]`.
///
/// Phase 1 stub: wires the command into the CLI so the binary compiles.
/// Context gathering, LLM integration, and output formatting are added in
/// Phases 2-4 (wai-3x6, wai-y7g, wai-qrg).
pub fn run(query: String, no_llm: bool) -> Result<()> {
    let _project_root = require_project()?;

    if no_llm {
        // Phase 5 will route this to `wai search`. For now, delegate directly.
        return super::search::run(query, None, None, false, None);
    }

    // Placeholder until Phase 2-4 are implemented.
    // Shows the query so the plumbing is visibly connected.
    println!();
    println!("  ◆ wai why: {}", query);
    println!();
    println!("  LLM integration is not yet implemented.");
    println!("  Run `wai search {}` for keyword results.", query);
    println!();

    Ok(())
}
