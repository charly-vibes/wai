#![allow(unused_assignments)]

use clap::Parser;
use miette::Result;

mod cli;
mod commands;
mod config;
mod context;
mod error;
pub mod freshness;
mod guided_flows;
mod help;
mod json;
mod llm;
pub mod managed_block;
pub mod openspec;
mod output;
pub mod plugin;
mod state;
mod sync_core;
mod tutorial;
#[allow(dead_code)]
mod workflows;
mod workspace;

use cli::Cli;
use context::{CliContext, set_context};
use output::print_json_line;

fn main() -> Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .context_lines(2)
                .build(),
        )
    }))
    .ok();

    let args: Vec<String> = std::env::args().collect();

    // Handle --version --json before clap processes it (clap's built-in --version
    // doesn't participate in the global --json flag)
    let has_version = args.iter().any(|a| {
        a == "--version"
            || a == "-V"
            || (a.starts_with("-") && !a.starts_with("--") && a.contains('V') && !a.contains('h'))
    });
    if has_version
        && !args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "-jh")
    {
        let has_json = args.iter().any(|a| {
            a == "--json"
                || a == "-j"
                || (a.starts_with("-j") && !a.starts_with("--") && !a.contains('h'))
        });
        if has_json {
            use genesis::envelope::Envelope;
            let envelope = Envelope::success(
                genesis::envelope::EnvelopeKind::Version,
                serde_json::json!({
                    "name": "wai",
                    "version": cli::VERSION
                }),
                vec![],
                vec![],
            );
            let _ = print_json_line(&envelope);
            return Ok(());
        }
    }

    if let Some(output) = help::try_render_help(&args) {
        print!("{}", output);
        return Ok(());
    }

    let cli = Cli::parse();
    set_context(CliContext {
        json: cli.json,
        no_input: cli.no_input,
        yes: cli.yes,
        safe: cli.safe,
        verbose: cli.verbose,
        quiet: cli.quiet,
    });
    let guide = cli::build_guide();
    match commands::run(cli, &guide) {
        Ok(_) => Ok(()),
        Err(err) => {
            let context = context::current_context();
            if context.json {
                use genesis::envelope::{Envelope, ErrorResult, RemediationEntry};
                let err_result = ErrorResult::new(
                    "E000",
                    &err.to_string(),
                    None,
                    None,
                    None,
                    vec![],
                    vec![RemediationEntry {
                        command: "wai doctor".into(),
                        description: "run workspace health check".into(),
                    }],
                )
                .expect("remediation must be non-empty");
                let _ = print_json_line(&Envelope::error(err_result, vec![]));
            }
            Err(err)
        }
    }
}
