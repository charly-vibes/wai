#![allow(unused_assignments)]

use clap::Parser;
use miette::Result;

mod cli;
mod commands;
mod config;
mod context;
mod error;
mod json;
mod output;
pub mod managed_block;
pub mod plugin;
mod state;

use cli::Cli;
use context::{CliContext, set_context};
use output::print_json_line;

fn main() -> Result<()> {
    // Install miette's fancy error handler
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

    let cli = Cli::parse();
    set_context(CliContext {
        json: cli.json,
        no_input: cli.no_input,
        yes: cli.yes,
        safe: cli.safe,
    });
    match commands::run(cli) {
        Ok(_) => Ok(()),
        Err(err) => {
            let context = context::current_context();
            if context.json {
                let payload = crate::error::ErrorPayload {
                    code: "wai::error::unknown".to_string(),
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
