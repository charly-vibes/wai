#![allow(unused_assignments)]

use clap::Parser;
use miette::Result;

mod cli;
mod commands;
mod config;
mod error;

use cli::Cli;

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
    commands::run(cli)
}
