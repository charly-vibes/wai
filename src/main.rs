#![allow(unused_assignments)]

use clap::Parser;
use miette::Result;

mod cli;
mod commands;
mod config;
mod context;
mod error;
mod guided_flows;
mod help;
mod json;
pub mod managed_block;
pub mod openspec;
mod output;
pub mod plugin;
mod state;
mod tutorial;

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
