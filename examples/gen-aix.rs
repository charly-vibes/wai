//! Self-hosting AIX generation for wai.
//!
//! Regenerates `llms.txt` and `llm.txt` from the current tool metadata.
//! Run via: `cargo run --example gen-aix` (or `just aix-gen`)
//!
//! Note: genesis::aix v0.4.0 exports `agents_block()`. Full structured
//! generation helpers (ProjectMeta, ModuleEntry, section builders) will
//! be available in a future genesis release — this example inline-generates
//! the formatted output for now.

use genesis::aix;

/// Current suite of genesis modules adopted by wai.
fn genesis_adoption_table() -> Vec<(&'static str, &'static str)> {
    vec![
        ("envelope", "Envelope, EnvelopeKind, ErrorResult"),
        ("suggestions", "Suggestion, SuggestionEngine"),
        ("managed_block", "managed block injector"),
        ("aix", "agents_block generation"),
        ("config", "ConfigFile trait, ConfigRegistry"),
        ("guide", "Verbosity, OutputFormat"),
        ("fixture", "Fixture builder"),
        ("feedback", "handle_feedback(), FeedbackArgs"),
        ("suite_linter", "LintCheck trait"),
        ("doctor", "DoctorCheck trait, DoctorRunner, DoctorReport"),
        ("cli", "completions, version-json pre-parse"),
        (
            "status",
            "StatusContributor, StatusBuilder, DoctorStatusBridge",
        ),
        ("scaffold", "Scaffold builder"),
        ("discovery", "tool manifest registration"),
    ]
}

fn generate_llms_txt() -> String {
    let mut out = String::new();
    out.push_str("# wai\n\n");
    out.push_str("> Command-line workflow manager for AI-driven development. Preserves research, reasoning, and design decisions alongside the spec — the *why* behind every architectural choice, not just what was built.\n\n");
    out.push_str("## Authorship\n\n");
    out.push_str("Part of the charly-vibes suite. Leer en español: https://charly-vibes.github.io/charly-vibes/\n\n");
    out.push_str("## Modules\n\n");
    out.push_str("| Module | Description |\n");
    out.push_str("|--------|-------------|\n");
    out.push_str("| cli | CLI argument parsing and subcommand dispatch |\n");
    out.push_str("| config | project and agent configuration |\n");
    out.push_str("| error | error types and safe-mode enforcement |\n");
    out.push_str("| help | help text formatting |\n");
    out.push_str("| json | JSON response envelope types |\n");
    out.push_str("| llm | LLM API client (Claude, Ollama) |\n");
    out.push_str("| managed_block | agent instruction block injection |\n");
    out.push_str("| output | envelope-based CLI output |\n");
    out.push_str("| plugin | plugin detection and passthrough |\n");
    out.push_str("| state | persistent state (active project, phase) |\n");
    out.push_str("| suggestions | context-sensitive error suggestions |\n");
    out.push_str("| workspace | workspace lifecycle (init, sync, doctor) |\n");
    out.push_str("| commands | subcommand implementations |\n\n");
    out.push_str("## Related\n\n");
    out.push_str("- Repository: https://github.com/gastownhall/wai\n");
    out.push_str("- crates.io: https://crates.io/crates/wai-cli\n");
    out.push_str("- charly-vibes suite: https://charly-vibes.github.io/charly-vibes/\n");
    out
}

fn generate_llm_txt() -> String {
    let mut out = String::new();
    out.push_str("# wai\n\n");
    out.push_str(
        "> Workflow manager for AI-driven development — captures the *why* behind decisions.\n\n",
    );
    out.push_str("## What It Does\n\n");
    out.push_str(
        "wai is a CLI tool that helps teams document the reasoning, research, and design\n",
    );
    out.push_str(
        "decisions that shaped their code. When you revisit a project later, wai tells you\n",
    );
    out.push_str("*why* it was built that way.\n\n");
    out.push_str("- **Artifact tracking** — research, design, and plan documents organized by project and phase\n");
    out.push_str("- **PARA method** — Projects / Areas / Resources / Archives structure\n");
    out.push_str(
        "- **Phase workflow** — research → design → plan → implement → review → archive\n",
    );
    out.push_str("- **Session handoffs** — `wai close` creates a handoff doc; `wai prime` resumes where you left off\n");
    out.push_str("- **Plugin system** — integrates with beads (issues), git, and openspec\n");
    out.push_str("- **AI features** — `wai why` (LLM oracle), `wai reflect` (synthesizes context into CLAUDE.md)\n");
    out.push_str("- **Agent config sync** — single source of truth for AI assistant configs, synced to tool locations\n\n");
    out.push_str("## Key Commands\n\n");
    out.push_str("```\n");
    out.push_str("wai init                    # Initialize in current directory\n");
    out.push_str("wai status                  # Project status and suggestions\n");
    out.push_str("wai prime                   # Orient at session start\n");
    out.push_str("wai add research \"...\"      # Add research artifact\n");
    out.push_str("wai add design \"...\"        # Add design artifact\n");
    out.push_str("wai add plan \"...\"          # Add plan artifact\n");
    out.push_str("wai close                   # Session handoff\n");
    out.push_str("wai why \"why X?\"            # LLM-powered reasoning oracle\n");
    out.push_str("wai reflect                 # Synthesize context into CLAUDE.md\n");
    out.push_str("wai doctor                  # Workspace health check\n");
    out.push_str("wai sync                    # Sync agent configs\n");
    out.push_str("wai pipeline start          # Run an automated workflow pipeline\n");
    out.push_str("wai feedback bug            # File an issue with context attached\n");
    out.push_str("```\n\n");
    out.push_str("## Project Structure\n\n");
    out.push_str("```\n");
    out.push_str(".wai/\n");
    out.push_str("├── config.toml\n");
    out.push_str("├── projects/           # Active projects (phase-tracked)\n");
    out.push_str("├── areas/              # Ongoing responsibilities\n");
    out.push_str("├── resources/\n");
    out.push_str("│   └── agent-config/   # Skills, rules, context for AI assistants\n");
    out.push_str("└── archives/           # Completed work\n");
    out.push_str("```\n\n");
    out.push_str("## Source Layout\n\n");
    out.push_str("```\n");
    out.push_str("src/\n");
    out.push_str("├── main.rs\n");
    out.push_str("├── cli.rs              # Clap argument definitions\n");
    out.push_str("├── commands/           # One file per subcommand\n");
    out.push_str("│   ├── way.rs          # wai way — best practices checker\n");
    out.push_str("│   ├── status.rs       # wai status\n");
    out.push_str("│   ├── add.rs          # wai add\n");
    out.push_str("│   ├── search.rs       # wai search\n");
    out.push_str("│   └── ...\n");
    out.push_str("├── plugin.rs           # Plugin detection and dispatch\n");
    out.push_str("├── output.rs           # JSON output (wraps genesis::envelope)\n");
    out.push_str("├── managed_block.rs    # Content generation (via genesis::managed_block)\n");
    out.push_str("└── config.rs           # Config loading and paths\n");
    out.push_str("```\n\n");

    // Genesis adoption
    out.push_str("## Shared Infrastructure\n\n");
    out.push_str("wai is a consumer of the **genesis** shared crate (https://github.com/charly-vibes/genesis):\n\n");
    for (module, covers) in genesis_adoption_table() {
        out.push_str(&format!("- `genesis::{module}` — {covers}\n"));
    }

    out.push_str("\n## Installation\n\n");
    out.push_str("```bash\n");
    out.push_str("# Homebrew\n");
    out.push_str("brew tap charly-vibes/charly && brew install wai\n\n");
    out.push_str("# Cargo\n");
    out.push_str("cargo install wai-cli\n");
    out.push_str("```\n\n");
    out.push_str("## Links\n\n");
    out.push_str("- Repository: https://github.com/gastownhall/wai\n");
    out.push_str("- crates.io: https://crates.io/crates/wai-cli\n");
    out.push_str("- Changelog: CHANGELOG.md\n");
    out.push_str("- charly-vibes suite: https://charly-vibes.github.io/charly-vibes/\n");
    out
}

fn main() {
    // Generate llms.txt
    let llms_content = generate_llms_txt();
    let llms_path = std::path::Path::new("llms.txt");
    let stored_llms = std::fs::read_to_string(llms_path).unwrap_or_default();

    if llms_content != stored_llms {
        std::fs::write(llms_path, &llms_content).expect("write llms.txt");
        println!("✓ llms.txt regenerated");
    } else {
        println!("✓ llms.txt unchanged");
    }

    // Generate llm.txt
    let llm_content = generate_llm_txt();
    let llm_path = std::path::Path::new("llm.txt");
    let stored_llm = std::fs::read_to_string(llm_path).unwrap_or_default();

    if llm_content != stored_llm {
        std::fs::write(llm_path, &llm_content).expect("write llm.txt");
        println!("✓ llm.txt regenerated");
    } else {
        println!("✓ llm.txt unchanged");
    }

    // Use genesis::aix::agents_block (available in v0.4.0) as the adoption signal
    let _agent_block = aix::agents_block("wai", "Example agent block content");
}
