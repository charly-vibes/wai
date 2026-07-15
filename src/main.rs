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
mod suggestions;
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
            let envelope = serde_json::json!({
                "ok": true,
                "envelope_version": "0.2",
                "cli_version": cli::VERSION,
                "envelope_kind": "version",
                "data": {
                    "name": "wai",
                    "version": cli::VERSION
                },
                "warnings": [],
                "hints": [],
                "meta": {
                    "duration_ms": 0,
                    "tx": serde_json::Value::Null,
                    "request_id": serde_json::Value::Null
                }
            });
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
    match commands::run(cli) {
        Ok(_) => Ok(()),
        Err(err) => {
            let context = context::current_context();
            if context.json {
                let payload = crate::error::ErrorPayload {
                    code: err
                        .code()
                        .map(|c| format!("{}", c))
                        .unwrap_or_else(|| "wai::error::unknown".to_string()),
                    message: err.to_string(),
                    help: None,
                    details: None,
                };
                let _ = print_json_line(&payload);
            }
            Err(err)
        }
    }
}
